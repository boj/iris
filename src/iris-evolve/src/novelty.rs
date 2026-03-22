//! Novelty search for behavioral diversity (Lehman & Stanley 2011).
//!
//! Rewards behavioral DISTANCE from the archive of previously-seen behaviors.
//! A program that produces novel outputs gets a fitness bonus even if those
//! outputs are incorrect, helping the population escape local optima.

use iris_types::eval::Value;

// ---------------------------------------------------------------------------
// BehaviorDescriptor
// ---------------------------------------------------------------------------

/// A compact descriptor of a program's behavior: the hash of its outputs
/// on all test cases. Two programs with the same behavior descriptor
/// produce identical outputs on every test case.
pub type BehaviorDescriptor = Vec<u8>;

// ---------------------------------------------------------------------------
// NoveltyArchive
// ---------------------------------------------------------------------------

/// Maximum number of behaviors stored in the archive.
///
/// Prevents O(n²) scoring as the archive grows.  When the limit is reached,
/// the oldest entries are evicted (FIFO) to make room.  10,000 entries is
/// large enough to capture diverse behavior space without quadratic slowdown.
const MAX_ARCHIVE_SIZE: usize = 10_000;

/// Archive of previously-seen behaviors for novelty search.
///
/// Novelty is computed as the mean distance to the k nearest neighbors
/// in the archive. New behaviors are added to the archive only if their
/// novelty exceeds a threshold.  Archive size is capped at `MAX_ARCHIVE_SIZE`
/// to prevent O(n²) scoring overhead.
pub struct NoveltyArchive {
    behaviors: Vec<BehaviorDescriptor>,
    k_nearest: usize,
}

impl NoveltyArchive {
    /// Create a new empty novelty archive.
    ///
    /// `k_nearest` controls how many neighbors are used for the novelty
    /// score (typically 15-25).
    pub fn new(k_nearest: usize) -> Self {
        Self {
            behaviors: Vec::new(),
            k_nearest: k_nearest.max(1),
        }
    }

    /// Return the number of behaviors in the archive.
    pub fn len(&self) -> usize {
        self.behaviors.len()
    }

    /// Return whether the archive is empty.
    pub fn is_empty(&self) -> bool {
        self.behaviors.is_empty()
    }

    /// Compute the novelty score for a behavior descriptor.
    ///
    /// Novelty = mean Hamming distance to the k nearest neighbors in the
    /// archive. If the archive has fewer than k entries, all entries are
    /// used. If the archive is empty, returns 1.0 (maximally novel).
    pub fn novelty_score(&self, behavior: &BehaviorDescriptor) -> f32 {
        if self.behaviors.is_empty() {
            return 1.0;
        }

        // Compute distance to every behavior in the archive.
        let mut distances: Vec<f32> = self
            .behaviors
            .iter()
            .map(|b| behavior_distance(behavior, b))
            .collect();

        // Sort ascending to find k nearest.
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Mean of k nearest distances.
        let k = self.k_nearest.min(distances.len());
        let sum: f32 = distances[..k].iter().sum();
        sum / k as f32
    }

    /// Add a behavior to the archive if its novelty exceeds the threshold.
    ///
    /// Returns `true` if the behavior was added.
    ///
    /// When the archive would exceed `MAX_ARCHIVE_SIZE`, the oldest entry
    /// (index 0) is evicted before inserting the new one, keeping memory and
    /// O(n) scoring overhead bounded.
    pub fn add(&mut self, behavior: BehaviorDescriptor, threshold: f32) -> bool {
        let score = self.novelty_score(&behavior);
        if score >= threshold {
            self.evict_if_full();
            self.behaviors.push(behavior);
            true
        } else {
            false
        }
    }

    /// Unconditionally add a behavior to the archive.
    ///
    /// Evicts the oldest entry if the archive is at capacity.
    pub fn add_unchecked(&mut self, behavior: BehaviorDescriptor) {
        self.evict_if_full();
        self.behaviors.push(behavior);
    }

    /// Evict the oldest (first) entry if the archive is at capacity.
    fn evict_if_full(&mut self) {
        if self.behaviors.len() >= MAX_ARCHIVE_SIZE {
            self.behaviors.remove(0);
        }
    }

    /// Compute a behavior descriptor from evaluation results.
    ///
    /// The descriptor is a compact hash of all outputs the program produced
    /// across all test cases. Programs with identical outputs on all test
    /// cases will have the same descriptor.
    pub fn behavior_from_results(results: &[Vec<Value>]) -> BehaviorDescriptor {
        // Use a simple FNV-1a style rolling hash of all output values.
        let mut buf = Vec::with_capacity(results.len() * 16);
        for outputs in results {
            for val in outputs {
                value_to_behavior_bytes(val, &mut buf);
            }
            // Separator between test cases.
            buf.push(0xFF);
        }

        // Hash down to a fixed-size descriptor using blake3.
        let hash = blake3::hash(&buf);
        hash.as_bytes().to_vec()
    }
}

// ---------------------------------------------------------------------------
// Distance metric
// ---------------------------------------------------------------------------

/// Compute the normalized Hamming distance between two behavior descriptors.
///
/// Returns a value in [0.0, 1.0] where 0.0 means identical behaviors and
/// 1.0 means maximally different.
fn behavior_distance(a: &BehaviorDescriptor, b: &BehaviorDescriptor) -> f32 {
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return 0.0;
    }

    let mut diff = 0u32;
    let min_len = a.len().min(b.len());

    // Compare overlapping bytes.
    for i in 0..min_len {
        if a[i] != b[i] {
            diff += 1;
        }
    }

    // Bytes beyond the shorter descriptor are all different.
    diff += (max_len - min_len) as u32;

    diff as f32 / max_len as f32
}

// ---------------------------------------------------------------------------
// Value serialization for behavior descriptors
// ---------------------------------------------------------------------------

/// Serialize a Value into bytes for behavior descriptor computation.
fn value_to_behavior_bytes(val: &Value, buf: &mut Vec<u8>) {
    match val {
        Value::Int(v) => {
            buf.push(0x00);
            buf.extend_from_slice(&v.to_le_bytes());
        }
        Value::Nat(v) => {
            buf.push(0x01);
            buf.extend_from_slice(&v.to_le_bytes());
        }
        Value::Float64(v) => {
            buf.push(0x02);
            buf.extend_from_slice(&v.to_le_bytes());
        }
        Value::Float32(v) => {
            buf.push(0x03);
            buf.extend_from_slice(&v.to_le_bytes());
        }
        Value::Bool(v) => {
            buf.push(0x04);
            buf.push(if *v { 1 } else { 0 });
        }
        Value::Bytes(v) => {
            buf.push(0x05);
            buf.extend_from_slice(&(v.len() as u32).to_le_bytes());
            buf.extend_from_slice(v);
        }
        Value::Unit => {
            buf.push(0x06);
        }
        Value::Tuple(elems) => {
            buf.push(0x07);
            buf.extend_from_slice(&(elems.len() as u32).to_le_bytes());
            for e in elems.iter() {
                value_to_behavior_bytes(e, buf);
            }
        }
        Value::Tagged(tag, inner) => {
            buf.push(0x08);
            buf.extend_from_slice(&tag.to_le_bytes());
            value_to_behavior_bytes(inner, buf);
        }
        Value::State(store) => {
            buf.push(0x09);
            buf.extend_from_slice(&(store.len() as u32).to_le_bytes());
            for (k, v) in store {
                buf.extend_from_slice(k.as_bytes());
                buf.push(0x00);
                value_to_behavior_bytes(v, buf);
            }
        }
        Value::Graph(_) => {
            buf.push(0x0A);
        }
        Value::Program(_) => {
            buf.push(0x0B);
        }
        Value::Future(_) => {
            buf.push(0x0C);
        }
        Value::String(s) => {
            buf.push(0x0D);
            buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
            buf.extend_from_slice(s.as_bytes());
        }
        Value::Thunk(sg, state) => {
            buf.push(0x0E);
            buf.extend_from_slice(&sg.hash.0[..16]);
            value_to_behavior_bytes(state, buf);
        }
        Value::Range(s, e) => {
            buf.push(0x0F);
            buf.extend_from_slice(&s.to_le_bytes());
            buf.extend_from_slice(&e.to_le_bytes());
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_archive_gives_max_novelty() {
        let archive = NoveltyArchive::new(15);
        let behavior = vec![1, 2, 3];
        assert_eq!(archive.novelty_score(&behavior), 1.0);
    }

    #[test]
    fn test_identical_behavior_gives_zero_novelty() {
        let mut archive = NoveltyArchive::new(15);
        let behavior = vec![1, 2, 3, 4, 5];
        archive.add_unchecked(behavior.clone());

        let score = archive.novelty_score(&behavior);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_different_behavior_gives_positive_novelty() {
        let mut archive = NoveltyArchive::new(15);
        archive.add_unchecked(vec![0, 0, 0, 0]);

        let novel = vec![255, 255, 255, 255];
        let score = archive.novelty_score(&novel);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_add_with_threshold() {
        let mut archive = NoveltyArchive::new(15);
        // First behavior always added (empty archive -> novelty = 1.0).
        assert!(archive.add(vec![1, 2, 3], 0.1));
        assert_eq!(archive.len(), 1);

        // Same behavior should NOT be added (novelty = 0.0 < threshold).
        assert!(!archive.add(vec![1, 2, 3], 0.1));
        assert_eq!(archive.len(), 1);

        // Very different behavior should be added.
        assert!(archive.add(vec![255, 254, 253], 0.1));
        assert_eq!(archive.len(), 2);
    }

    #[test]
    fn test_behavior_from_results() {
        let r1 = vec![vec![Value::Int(42)]];
        let r2 = vec![vec![Value::Int(42)]];
        let r3 = vec![vec![Value::Int(99)]];

        let b1 = NoveltyArchive::behavior_from_results(&r1);
        let b2 = NoveltyArchive::behavior_from_results(&r2);
        let b3 = NoveltyArchive::behavior_from_results(&r3);

        // Same outputs -> same descriptor.
        assert_eq!(b1, b2);

        // Different outputs -> different descriptor.
        assert_ne!(b1, b3);
    }

    #[test]
    fn test_k_nearest_neighbors() {
        let mut archive = NoveltyArchive::new(2);

        // Add 3 behaviors.
        archive.add_unchecked(vec![0, 0, 0, 0]);
        archive.add_unchecked(vec![10, 10, 10, 10]);
        archive.add_unchecked(vec![255, 255, 255, 255]);

        // With k=1, a behavior close to [0,0,0,0] should have low novelty.
        let mut archive_k1 = NoveltyArchive::new(1);
        archive_k1.add_unchecked(vec![0, 0, 0, 0]);
        archive_k1.add_unchecked(vec![10, 10, 10, 10]);
        archive_k1.add_unchecked(vec![255, 255, 255, 255]);

        let close = vec![1, 0, 0, 0];
        let score_k1 = archive_k1.novelty_score(&close);
        // Nearest neighbor is [0,0,0,0], distance = 0.25.
        assert!(score_k1 < 0.5, "k=1 score should be low: {}", score_k1);

        // With k=2, the score is the mean of the 2 nearest.
        let score_k2 = archive.novelty_score(&close);
        assert!(score_k2 > 0.0);

        // A behavior far from all should have high novelty.
        let far = vec![128, 128, 128, 128];
        let far_score = archive.novelty_score(&far);
        assert!(far_score > score_k2, "far should have higher novelty than close");
    }

    #[test]
    fn test_behavior_distance_symmetry() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        assert_eq!(behavior_distance(&a, &b), behavior_distance(&b, &a));
    }

    #[test]
    fn test_behavior_distance_identity() {
        let a = vec![1, 2, 3];
        assert_eq!(behavior_distance(&a, &a), 0.0);
    }

    #[test]
    fn test_behavior_distance_different_lengths() {
        let a = vec![1, 2, 3];
        let b = vec![1, 2, 3, 4, 5];
        // Extra bytes count as different.
        let d = behavior_distance(&a, &b);
        assert!(d > 0.0);
    }
}
