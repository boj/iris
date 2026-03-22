/*
 * tso_ring.c — TSO lock-free ring buffer
 *
 * Exploits x86-64 Total Store Order (TSO) for zero-cost synchronization.
 * No MFENCE, no LOCK prefix, no atomic RMW — only compiler barriers.
 * Single-producer / single-consumer design.
 *
 * Sequence counter protocol: odd = writing, even = stable.
 * Futex fallback after TSO_SPIN_LIMIT spin iterations.
 *
 * See SPEC.md Section 9.8.
 */

#include "clcu.h"

#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>
#include <linux/futex.h>
#include <sys/syscall.h>
#include <limits.h>

/* ────────────────────────────────────────────────────────────────────────────
 * Ring buffer structure — cache-line-padded to avoid false sharing
 * ──────────────────────────────────────────────────────────────────────────── */
struct tso_ring {
    /* Producer-owned cache line */
    volatile uint64_t write_pos __attribute__((aligned(64)));
    uint8_t _pad_producer[56];

    /* Consumer-owned cache line */
    volatile uint64_t read_pos __attribute__((aligned(64)));
    uint8_t _pad_consumer[56];

    /* Slot array — each slot is 64 bytes (one cache line) */
    tso_slot_t slots[TSO_RING_SIZE] __attribute__((aligned(64)));
};

/* ────────────────────────────────────────────────────────────────────────────
 * Compiler barrier — prevents compiler reordering, no hardware fence needed
 * on x86-64 TSO. Store-store and load-load ordering is guaranteed by hardware.
 * ──────────────────────────────────────────────────────────────────────────── */
#define COMPILER_BARRIER() __asm__ volatile("" ::: "memory")

/* ────────────────────────────────────────────────────────────────────────────
 * Futex helpers — kernel-assisted wait/wake for when spinning is wasteful
 * ──────────────────────────────────────────────────────────────────────────── */
static inline long futex_wait(volatile uint64_t *addr, uint32_t expected) {
    /* futex operates on 32-bit values; we use the low 32 bits of the
     * 64-bit position. This is safe because the ring size is << 2^32. */
    return syscall(SYS_futex, (volatile uint32_t *)addr, FUTEX_WAIT,
                   expected, NULL, NULL, 0);
}

static inline long futex_wake(volatile uint64_t *addr, int count) {
    return syscall(SYS_futex, (volatile uint32_t *)addr, FUTEX_WAKE,
                   count, NULL, NULL, 0);
}

/* ────────────────────────────────────────────────────────────────────────────
 * tso_ring_create — allocate ring buffer with mmap for page alignment
 * ──────────────────────────────────────────────────────────────────────────── */
tso_ring_t *tso_ring_create(void) {
    /* Allocate ring structure, page-aligned for TLB friendliness */
    size_t ring_size = sizeof(tso_ring_t);

    /* Round up to page boundary */
    size_t page_size = (size_t)sysconf(_SC_PAGESIZE);
    size_t alloc_size = (ring_size + page_size - 1) & ~(page_size - 1);

    void *mem = mmap(NULL, alloc_size,
                     PROT_READ | PROT_WRITE,
                     MAP_PRIVATE | MAP_ANONYMOUS,
                     -1, 0);
    if (mem == MAP_FAILED) return NULL;

    /* Try transparent huge pages for the slot array */
    madvise(mem, alloc_size, MADV_HUGEPAGE);

    tso_ring_t *ring = (tso_ring_t *)mem;
    ring->write_pos = 0;
    ring->read_pos = 0;

    /* Zero all slots */
    memset(ring->slots, 0, sizeof(ring->slots));

    return ring;
}

/* ────────────────────────────────────────────────────────────────────────────
 * tso_ring_destroy
 * ──────────────────────────────────────────────────────────────────────────── */
void tso_ring_destroy(tso_ring_t *ring) {
    if (!ring) return;

    size_t ring_size = sizeof(tso_ring_t);
    size_t page_size = (size_t)sysconf(_SC_PAGESIZE);
    size_t alloc_size = (ring_size + page_size - 1) & ~(page_size - 1);

    munmap(ring, alloc_size);
}

/* ────────────────────────────────────────────────────────────────────────────
 * tso_produce — write data into the ring buffer
 *
 * Protocol:
 *   1. Check if ring is full (write_pos - read_pos >= RING_SIZE)
 *   2. Write slot data
 *   3. Compiler barrier
 *   4. Increment write_pos (store becomes visible in program order via TSO)
 *   5. Optionally wake consumer via futex
 *
 * Returns: 0 on success, -1 if ring is full
 * ──────────────────────────────────────────────────────────────────────────── */
int tso_produce(tso_ring_t *ring, const void *data, size_t len) {
    if (!ring || !data) return -1;

    uint64_t wp = ring->write_pos;
    /*
     * Read consumer's read_pos. Under TSO, this load completes before
     * subsequent loads (load-load ordering guaranteed by hardware).
     */
    uint64_t rp = __atomic_load_n(&ring->read_pos, __ATOMIC_RELAXED);

    /* Ring full? */
    if (wp - rp >= TSO_RING_SIZE) {
        /* Spin briefly then give up */
        for (int spin = 0; spin < TSO_SPIN_LIMIT; spin++) {
            __asm__ volatile("pause");
            rp = __atomic_load_n(&ring->read_pos, __ATOMIC_RELAXED);
            if (wp - rp < TSO_RING_SIZE) goto have_space;
        }
        return -1; /* Still full after spinning */
    }

have_space:
    {
        tso_slot_t *slot = &ring->slots[wp & TSO_RING_MASK];

        /* Step 1: Mark slot as being written (odd sequence = writing) */
        __atomic_store_n(&slot->sequence, slot->sequence + 1, __ATOMIC_RELAXED);
        COMPILER_BARRIER();

        /* Step 2: Write data into slot */
        size_t copy_len = (len < TSO_SLOT_DATA_SIZE) ? len : TSO_SLOT_DATA_SIZE;
        memcpy((void *)slot->data, data, copy_len);
        if (copy_len < TSO_SLOT_DATA_SIZE) {
            memset((void *)(slot->data + copy_len), 0, TSO_SLOT_DATA_SIZE - copy_len);
        }

        /* Step 3: Mark slot as stable (even sequence = stable) */
        COMPILER_BARRIER();
        __atomic_store_n(&slot->sequence, slot->sequence + 1, __ATOMIC_RELAXED);

        /* Step 4: Advance write position.
         * TSO guarantees: this store is visible to consumers AFTER the
         * data stores above (store-store ordering is free on x86). */
        COMPILER_BARRIER();
        __atomic_store_n(&ring->write_pos, wp + 1, __ATOMIC_RELAXED);
    }

    /* Wake consumer if it might be sleeping on futex */
    futex_wake(&ring->write_pos, 1);

    return 0;
}

/* ────────────────────────────────────────────────────────────────────────────
 * tso_consume — read data from the ring buffer
 *
 * Protocol:
 *   1. Check if ring is empty (read_pos >= write_pos)
 *   2. Spin briefly if empty, then futex wait
 *   3. Compiler barrier (prevents compiler reordering load of data before
 *      load of write_pos; hardware already guarantees load-load order on TSO)
 *   4. Read slot data, verify sequence counter for consistency
 *   5. Advance read_pos
 *
 * Returns: 0 on success, -1 if ring is empty (after spin + futex timeout)
 * ──────────────────────────────────────────────────────────────────────────── */
int tso_consume(tso_ring_t *ring, void *data, size_t len) {
    if (!ring || !data) return -1;

    uint64_t rp = ring->read_pos;
    uint64_t wp = __atomic_load_n(&ring->write_pos, __ATOMIC_RELAXED);

    /* Ring empty? Spin then futex wait */
    if (rp >= wp) {
        for (int spin = 0; spin < TSO_SPIN_LIMIT; spin++) {
            __asm__ volatile("pause");
            wp = __atomic_load_n(&ring->write_pos, __ATOMIC_RELAXED);
            if (rp < wp) goto have_data;
        }
        /* Futex wait — block until producer wakes us */
        futex_wait(&ring->write_pos, (uint32_t)wp);
        wp = __atomic_load_n(&ring->write_pos, __ATOMIC_RELAXED);
        if (rp >= wp) return -1; /* Spurious wakeup, still empty */
    }

have_data:
    COMPILER_BARRIER();
    {
        const tso_slot_t *slot = &ring->slots[rp & TSO_RING_MASK];

        /* Read sequence counter before data */
        uint64_t seq1 = __atomic_load_n(&slot->sequence, __ATOMIC_RELAXED);
        if (seq1 & 1) {
            /* Slot is being written — shouldn't happen with proper SPSC usage,
             * but handle gracefully by retrying */
            return -1;
        }

        COMPILER_BARRIER();

        /* Read data */
        size_t copy_len = (len < TSO_SLOT_DATA_SIZE) ? len : TSO_SLOT_DATA_SIZE;
        memcpy(data, (const void *)slot->data, copy_len);

        COMPILER_BARRIER();

        /* Verify sequence counter after data read */
        uint64_t seq2 = __atomic_load_n(&slot->sequence, __ATOMIC_RELAXED);
        if (seq1 != seq2) {
            /* Torn read — data was modified during our read.
             * This indicates a protocol violation (writer overlapped reader). */
            return -1;
        }

        /* Advance read position.
         * TSO guarantees: this store is visible to the producer AFTER our
         * data reads above (store-store ordering). */
        COMPILER_BARRIER();
        __atomic_store_n(&ring->read_pos, rp + 1, __ATOMIC_RELAXED);
    }

    /* Wake producer if it might be spinning on fullness check */
    futex_wake(&ring->read_pos, 1);

    return 0;
}
