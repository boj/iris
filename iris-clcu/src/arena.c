/*
 * arena.c — Huge-page arena allocator for CLCU containers
 *
 * Allocates 2MB huge page arenas via mmap(MAP_HUGETLB).
 * Falls back to regular pages + MADV_HUGEPAGE if huge pages unavailable.
 * Each arena holds 32,768 contiguous 64-byte-aligned containers.
 *
 * See SPEC.md Section 9.2.
 */

#include "clcu.h"

#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>

/* MAP_HUGE_2MB may not be defined on older headers */
#ifndef MAP_HUGE_2MB
#define MAP_HUGE_2MB (21 << MAP_HUGE_SHIFT)
#endif

#ifndef MAP_HUGE_SHIFT
#define MAP_HUGE_SHIFT 26
#endif

clcu_arena_t *arena_create(void) {
    clcu_arena_t *a = calloc(1, sizeof(clcu_arena_t));
    if (!a) return NULL;

    /* Try huge pages first */
    a->base = mmap(NULL, ARENA_SIZE,
                   PROT_READ | PROT_WRITE,
                   MAP_PRIVATE | MAP_ANONYMOUS | MAP_HUGETLB | MAP_HUGE_2MB,
                   -1, 0);

    if (a->base == MAP_FAILED) {
        /* Fallback: regular pages with transparent huge page hint */
        a->base = mmap(NULL, ARENA_SIZE,
                       PROT_READ | PROT_WRITE,
                       MAP_PRIVATE | MAP_ANONYMOUS,
                       -1, 0);
        if (a->base == MAP_FAILED) {
            free(a);
            return NULL;
        }
        madvise(a->base, ARENA_SIZE, MADV_HUGEPAGE);
    }

    a->next_free = 0;
    a->capacity  = CONTAINERS_PER_ARENA;

    return a;
}

void *arena_alloc(clcu_arena_t *arena, uint32_t n) {
    if (!arena || !arena->base) return NULL;
    if (arena->next_free + n > arena->capacity) return NULL;

    void *ptr = (char *)arena->base + ((uint64_t)arena->next_free * CLCU_CONTAINER_SIZE);
    arena->next_free += n;
    return ptr;
}

void arena_destroy(clcu_arena_t *arena) {
    if (!arena) return;
    if (arena->base && arena->base != MAP_FAILED) {
        munmap(arena->base, ARENA_SIZE);
    }
    free(arena);
}
