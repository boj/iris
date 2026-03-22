/*
 * tlb_map.c — TLB-encoded page-per-bucket hash map
 *
 * DEPRECATED: The TLB-as-data-structure approach was identified by the
 * engineering council as a category error. User-space cannot control TLB
 * behavior. This code is retained for reference but should not be used
 * in production. Use standard HashMap implementations instead.
 *
 * Original design: Page-per-bucket where each hash bucket occupies its own
 * virtual page. Uses hardware CRC32 for hashing, demand-paged with
 * MAP_NORESERVE. The claim that "TLB hardware acts as the fast-path cache"
 * is incorrect — the TLB transparently caches page translations for all
 * memory accesses; this design does not gain special TLB treatment.
 *
 * See SPEC.md Section 9.9 (removed).
 */

#include "clcu.h"

#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>

/* ────────────────────────────────────────────────────────────────────────────
 * Bucket header — lives at the start of each page
 * ──────────────────────────────────────────────────────────────────────────── */
typedef struct {
    uint32_t occupied;     /* Number of entries in this bucket  */
    uint32_t capacity;     /* Max entries that fit in one page  */
    uint32_t materialized; /* 1 if page has been mapped, 0 if PROT_NONE */
    uint32_t _pad;
} tlb_bucket_t;

#define BUCKET_HEADER_SIZE  sizeof(tlb_bucket_t)
#define PAGE_SIZE_4K        4096

/* ────────────────────────────────────────────────────────────────────────────
 * Hash function — hardware CRC32 via __builtin_ia32_crc32di when available
 * ──────────────────────────────────────────────────────────────────────────── */
static inline uint64_t tlb_hash(const void *key, uint32_t key_size) {
    uint64_t h = 0xFFFFFFFF;
    const uint8_t *k = (const uint8_t *)key;

#if defined(__SSE4_2__) && defined(__x86_64__)
    /* Hardware CRC32 path */
    uint32_t i = 0;
    for (; i + 8 <= key_size; i += 8) {
        uint64_t word;
        memcpy(&word, k + i, 8);
        h = __builtin_ia32_crc32di(h, word);
    }
    for (; i + 4 <= key_size; i += 4) {
        uint32_t word;
        memcpy(&word, k + i, 4);
        h = __builtin_ia32_crc32si((uint32_t)h, word);
    }
    for (; i < key_size; i++) {
        h = __builtin_ia32_crc32qi((uint32_t)h, k[i]);
    }
#else
    /* Software fallback: FNV-1a */
    h = 0xcbf29ce484222325ULL;
    for (uint32_t i = 0; i < key_size; i++) {
        h ^= k[i];
        h *= 0x100000001b3ULL;
    }
#endif

    return h;
}

/* ────────────────────────────────────────────────────────────────────────────
 * Internal struct definition
 * ──────────────────────────────────────────────────────────────────────────── */
struct tlb_map {
    void     *base;              /* Base of virtual address reservation */
    uint64_t  num_buckets;       /* Number of buckets (pages) — power of 2 */
    uint32_t  key_size;
    uint32_t  value_size;
    uint32_t  entry_size;        /* key_size + value_size */
    uint32_t  entries_per_page;  /* How many entries fit in one 4K page */
    uint64_t  total_entries;     /* Total entries across all buckets */
    size_t    total_size;        /* Total reserved virtual address space */
};

/* ────────────────────────────────────────────────────────────────────────────
 * Round up to next power of 2
 * ──────────────────────────────────────────────────────────────────────────── */
static uint64_t next_pow2(uint64_t v) {
    if (v == 0) return 1;
    v--;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v |= v >> 32;
    return v + 1;
}

/* ────────────────────────────────────────────────────────────────────────────
 * tlb_map_create — reserve virtual address space, demand-paged
 * ──────────────────────────────────────────────────────────────────────────── */
tlb_map_t *tlb_map_create(uint32_t key_size, uint32_t value_size,
                           uint64_t expected_entries) {
    if (key_size == 0 || value_size == 0) return NULL;

    tlb_map_t *map = calloc(1, sizeof(tlb_map_t));
    if (!map) return NULL;

    map->key_size   = key_size;
    map->value_size = value_size;
    map->entry_size = key_size + value_size;

    /* Compute entries per page (4K page minus bucket header) */
    if (map->entry_size == 0 || map->entry_size > (PAGE_SIZE_4K - BUCKET_HEADER_SIZE)) {
        free(map);
        return NULL;
    }
    map->entries_per_page = (PAGE_SIZE_4K - BUCKET_HEADER_SIZE) / map->entry_size;

    /* Target ~70% load factor per bucket */
    uint64_t target_per_bucket = (map->entries_per_page * 7) / 10;
    if (target_per_bucket == 0) target_per_bucket = 1;
    map->num_buckets = next_pow2((expected_entries / target_per_bucket) + 1);

    /* Clamp to reasonable range */
    if (map->num_buckets < 16) map->num_buckets = 16;
    if (map->num_buckets > (1ULL << 24)) map->num_buckets = (1ULL << 24); /* 16M max */

    map->total_size = map->num_buckets * PAGE_SIZE_4K;
    map->total_entries = 0;

    /* Reserve virtual address space without committing physical pages */
    map->base = mmap(NULL, map->total_size,
                     PROT_NONE,
                     MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE,
                     -1, 0);
    if (map->base == MAP_FAILED) {
        free(map);
        return NULL;
    }

    /* Hint: random access pattern */
    madvise(map->base, map->total_size, MADV_RANDOM);

    return map;
}

/* ────────────────────────────────────────────────────────────────────────────
 * Materialize a bucket page on first access
 * ──────────────────────────────────────────────────────────────────────────── */
static int bucket_materialize(tlb_map_t *map, uint64_t bucket_idx) {
    void *page_addr = (char *)map->base + (bucket_idx * PAGE_SIZE_4K);

    /* Re-map this specific page with read/write backing */
    void *result = mmap(page_addr, PAGE_SIZE_4K,
                        PROT_READ | PROT_WRITE,
                        MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED,
                        -1, 0);
    if (result == MAP_FAILED) return -1;

    /* Initialize bucket header */
    tlb_bucket_t *bucket = (tlb_bucket_t *)page_addr;
    bucket->occupied = 0;
    bucket->capacity = map->entries_per_page;
    bucket->materialized = 1;
    bucket->_pad = 0;

    return 0;
}

/* ────────────────────────────────────────────────────────────────────────────
 * Get bucket pointer, materializing if needed
 * ──────────────────────────────────────────────────────────────────────────── */
static tlb_bucket_t *get_bucket(tlb_map_t *map, uint64_t bucket_idx, int create) {
    void *page_addr = (char *)map->base + (bucket_idx * PAGE_SIZE_4K);
    tlb_bucket_t *bucket = (tlb_bucket_t *)page_addr;

    /*
     * Check if the page is materialized. We rely on a sentinel value:
     * unmaterialized pages are PROT_NONE and will segfault on access,
     * so we use mmap MAP_FIXED to materialize on first insert.
     *
     * For lookups on unmaterialized pages, we return NULL (not found).
     * For inserts, we materialize on demand.
     */
    if (create) {
        /* Attempt to read the materialized flag — if the page is PROT_NONE,
         * this would segfault. Instead, we try to materialize unconditionally
         * and the MAP_FIXED re-map is idempotent for already-mapped pages
         * (though it clears the data). So we need a tracking mechanism. */

        /* Use a simple approach: try reading, if page is accessible it's
         * already materialized. We use mincore() to check. */
        unsigned char vec;
        int rc = mincore(page_addr, PAGE_SIZE_4K, &vec);
        if (rc != 0 || !(vec & 1)) {
            /* Page not resident — materialize it */
            if (bucket_materialize(map, bucket_idx) != 0) return NULL;
        }
    } else {
        /* For lookup: check if page is resident */
        unsigned char vec;
        int rc = mincore(page_addr, PAGE_SIZE_4K, &vec);
        if (rc != 0 || !(vec & 1)) {
            return NULL; /* Not materialized = no entries */
        }
    }

    return bucket;
}

/* ────────────────────────────────────────────────────────────────────────────
 * tlb_map_insert
 * ──────────────────────────────────────────────────────────────────────────── */
int tlb_map_insert(tlb_map_t *map, const void *key, const void *value) {
    if (!map || !key || !value) return -1;

    uint64_t h = tlb_hash(key, map->key_size);
    uint64_t bucket_idx = h & (map->num_buckets - 1);

    tlb_bucket_t *bucket = get_bucket(map, bucket_idx, 1);
    if (!bucket) return -1;

    /* Check for existing key (update in place) */
    uint8_t *entries = (uint8_t *)bucket + BUCKET_HEADER_SIZE;
    for (uint32_t i = 0; i < bucket->occupied; i++) {
        uint8_t *entry = entries + (i * map->entry_size);
        if (memcmp(entry, key, map->key_size) == 0) {
            memcpy(entry + map->key_size, value, map->value_size);
            return 0;
        }
    }

    /* Bucket full? */
    if (bucket->occupied >= bucket->capacity) return -1;

    /* Append new entry */
    uint8_t *new_entry = entries + (bucket->occupied * map->entry_size);
    memcpy(new_entry, key, map->key_size);
    memcpy(new_entry + map->key_size, value, map->value_size);
    bucket->occupied++;
    map->total_entries++;

    return 0;
}

/* ────────────────────────────────────────────────────────────────────────────
 * tlb_map_lookup — the TLB is the fast path here
 * ──────────────────────────────────────────────────────────────────────────── */
const void *tlb_map_lookup(const tlb_map_t *map, const void *key) {
    if (!map || !key) return NULL;

    uint64_t h = tlb_hash(key, map->key_size);
    uint64_t bucket_idx = h & (map->num_buckets - 1);

    /* This page access IS the TLB lookup — if recently accessed, the TLB
     * has the translation cached and this completes in ~1 cycle */
    tlb_bucket_t *bucket = get_bucket((tlb_map_t *)map, bucket_idx, 0);
    if (!bucket) return NULL;

    /* Linear scan within the page (cache-friendly, all data co-located) */
    const uint8_t *entries = (const uint8_t *)bucket + BUCKET_HEADER_SIZE;
    for (uint32_t i = 0; i < bucket->occupied; i++) {
        const uint8_t *entry = entries + (i * map->entry_size);
        if (memcmp(entry, key, map->key_size) == 0) {
            return entry + map->key_size; /* Return pointer to value */
        }
    }

    return NULL;
}

/* ────────────────────────────────────────────────────────────────────────────
 * tlb_map_delete
 * ──────────────────────────────────────────────────────────────────────────── */
int tlb_map_delete(tlb_map_t *map, const void *key) {
    if (!map || !key) return -1;

    uint64_t h = tlb_hash(key, map->key_size);
    uint64_t bucket_idx = h & (map->num_buckets - 1);

    tlb_bucket_t *bucket = get_bucket(map, bucket_idx, 0);
    if (!bucket) return -1;

    uint8_t *entries = (uint8_t *)bucket + BUCKET_HEADER_SIZE;
    for (uint32_t i = 0; i < bucket->occupied; i++) {
        uint8_t *entry = entries + (i * map->entry_size);
        if (memcmp(entry, key, map->key_size) == 0) {
            /* Swap with last entry (order doesn't matter in hash bucket) */
            if (i < bucket->occupied - 1) {
                uint8_t *last = entries + ((bucket->occupied - 1) * map->entry_size);
                memcpy(entry, last, map->entry_size);
            }
            bucket->occupied--;
            map->total_entries--;
            return 0;
        }
    }

    return -1; /* Not found */
}

/* ────────────────────────────────────────────────────────────────────────────
 * tlb_map_prefetch — non-blocking prefetch of key's bucket page into TLB
 * ──────────────────────────────────────────────────────────────────────────── */
void tlb_map_prefetch(const tlb_map_t *map, const void *key) {
    if (!map || !key) return;

    uint64_t h = tlb_hash(key, map->key_size);
    uint64_t bucket_idx = h & (map->num_buckets - 1);
    const void *page_addr = (const char *)map->base + (bucket_idx * PAGE_SIZE_4K);
    __builtin_prefetch(page_addr, 0, 3);
}

/* ────────────────────────────────────────────────────────────────────────────
 * tlb_map_destroy — release all pages and free map
 * ──────────────────────────────────────────────────────────────────────────── */
void tlb_map_destroy(tlb_map_t *map) {
    if (!map) return;
    if (map->base && map->base != MAP_FAILED) {
        munmap(map->base, map->total_size);
    }
    free(map);
}
