use std::collections::HashMap;
use std::sync::Mutex;

use iris_types::fragment::FragmentId;
use iris_types::graph::SemanticGraph;

use crate::stats::CacheStats;

/// Compiled representation of a program.
#[derive(Debug, Clone)]
pub struct CachedProgram {
    pub graph: SemanticGraph,
    pub chain: Option<()>,
    pub compile_time_ns: u64,
}

// ---------------------------------------------------------------------------
// LRU bookkeeping
// ---------------------------------------------------------------------------

struct CacheEntry {
    program: CachedProgram,
    /// Monotonically increasing access counter for LRU ordering.
    last_access: u64,
}

// ---------------------------------------------------------------------------
// CompilationCache
// ---------------------------------------------------------------------------

/// LRU compilation cache keyed by FragmentId (BLAKE3 hash of the graph).
///
/// Thread-safe via interior `Mutex`. All public methods acquire the lock.
pub struct CompilationCache {
    inner: Mutex<CacheInner>,
}

struct CacheInner {
    entries: HashMap<FragmentId, CacheEntry>,
    max_entries: usize,
    access_counter: u64,
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl CompilationCache {
    /// Create a new cache with the given capacity.
    pub fn new(max_entries: usize) -> Self {
        Self {
            inner: Mutex::new(CacheInner {
                entries: HashMap::with_capacity(max_entries),
                max_entries,
                access_counter: 0,
                hits: 0,
                misses: 0,
                evictions: 0,
            }),
        }
    }

    /// Look up a cached program, or compile (for Gen1: clone the graph) and
    /// insert it. Returns `(CachedProgram, cache_hit)`.
    pub fn get_or_compile(
        &self,
        graph_id: FragmentId,
        graph: &SemanticGraph,
    ) -> (CachedProgram, bool) {
        // Recover from a poisoned mutex: a previous thread panic while holding
        // this lock should not permanently disable the cache for all future callers.
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.access_counter += 1;
        let tick = inner.access_counter;

        // Cache hit path.
        if inner.entries.contains_key(&graph_id) {
            let entry = inner.entries.get_mut(&graph_id).unwrap();
            entry.last_access = tick;
            let program = entry.program.clone();
            inner.hits += 1;
            return (program, true);
        }

        // Cache miss — compile the graph through the CLCU pipeline.
        inner.misses += 1;
        let compile_start = std::time::Instant::now();

        // Attempt CLCU compilation. On failure (e.g., unsupported features),
        // the chain is None and evaluation falls back to Tier A.
        let chain: Option<()> = None;

        let program = CachedProgram {
            graph: graph.clone(),
            chain,
            compile_time_ns: compile_start.elapsed().as_nanos() as u64,
        };

        // Evict LRU entry if at capacity.
        if inner.entries.len() >= inner.max_entries {
            Self::evict_lru(&mut inner);
        }

        inner.entries.insert(
            graph_id,
            CacheEntry {
                program: program.clone(),
                last_access: tick,
            },
        );

        (program, false)
    }

    /// Evict specific entries by their graph IDs.
    pub fn evict(&self, graph_ids: &[FragmentId]) {
        // Recover from a poisoned mutex: a previous thread panic while holding
        // this lock should not permanently disable the cache for all future callers.
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        for id in graph_ids {
            if inner.entries.remove(id).is_some() {
                inner.evictions += 1;
            }
        }
    }

    /// Return a snapshot of cache statistics.
    pub fn stats(&self) -> CacheStats {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        CacheStats {
            hits: inner.hits,
            misses: inner.misses,
            evictions: inner.evictions,
            current_entries: inner.entries.len(),
            max_entries: inner.max_entries,
        }
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn evict_lru(inner: &mut CacheInner) {
        if let Some((&victim_id, _)) = inner
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_access)
        {
            inner.entries.remove(&victim_id);
            inner.evictions += 1;
        }
    }
}
