//! Stigmergic field for guiding evolutionary search (SPEC Section 6.7).
//!
//! A sparse voxel grid in 64-dim embedding space where programs deposit
//! signals (fitness, failure). Signals decay over generations, creating
//! a shared "pheromone trail" that biases mutation direction and parent
//! selection toward high-fitness regions and away from failure regions.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// VoxelKey — quantized embedding coordinates
// ---------------------------------------------------------------------------

/// Quantized first 8 dimensions of an embedding vector (LSH-like).
///
/// Using the first 8 dims provides a coarse spatial index while keeping
/// the key small. The resolution parameter controls quantization granularity.
pub type VoxelKey = [i16; 8];

/// Number of embedding dimensions used for voxel keys.
const VOXEL_DIMS: usize = 8;

// ---------------------------------------------------------------------------
// VoxelSignal — accumulated signal at a voxel
// ---------------------------------------------------------------------------

/// Accumulated signal deposited at a single voxel location.
#[derive(Debug, Clone)]
pub struct VoxelSignal {
    /// Sum of fitness values deposited at this voxel.
    pub fitness_sum: f32,
    /// Number of fitness deposits.
    pub fitness_count: u32,
    /// Number of failed programs deposited here.
    pub failure_count: u32,
    /// Generation when this voxel was last updated.
    pub last_updated: u64,
}

impl VoxelSignal {
    /// Create a new empty signal.
    fn new(generation: u64) -> Self {
        Self {
            fitness_sum: 0.0,
            fitness_count: 0,
            failure_count: 0,
            last_updated: generation,
        }
    }

    /// Average fitness at this voxel, or 0.0 if no deposits.
    fn avg_fitness(&self) -> f32 {
        if self.fitness_count == 0 {
            0.0
        } else {
            self.fitness_sum / self.fitness_count as f32
        }
    }

    /// Failure rate at this voxel: failures / total deposits.
    fn failure_rate(&self) -> f32 {
        let total = self.fitness_count + self.failure_count;
        if total == 0 {
            0.0
        } else {
            self.failure_count as f32 / total as f32
        }
    }

    /// Total number of programs that have visited this voxel.
    fn density(&self) -> u32 {
        self.fitness_count + self.failure_count
    }

    /// True if the signal is negligible (below threshold).
    fn is_negligible(&self, min_signal: f32) -> bool {
        self.fitness_sum.abs() < min_signal && self.failure_count == 0
    }
}

// ---------------------------------------------------------------------------
// SignalReading — interpolated signal at a query point
// ---------------------------------------------------------------------------

/// Interpolated signal reading at a point in embedding space.
#[derive(Debug, Clone)]
pub struct SignalReading {
    /// Average fitness across nearby voxels (weighted by proximity).
    pub avg_fitness: f32,
    /// Failure rate across nearby voxels.
    pub failure_rate: f32,
    /// Total density (number of programs that have been nearby).
    pub density: u32,
}

impl SignalReading {
    /// A zero reading (no signal).
    pub const ZERO: Self = Self {
        avg_fitness: 0.0,
        failure_rate: 0.0,
        density: 0,
    };
}

// ---------------------------------------------------------------------------
// StigmergicField — the sparse voxel grid
// ---------------------------------------------------------------------------

/// Sparse voxel grid in embedding space for stigmergic signaling.
///
/// Programs deposit fitness and failure signals at their embedding
/// locations. These signals decay over generations, creating a shared
/// landscape that guides evolutionary search:
/// - High-fitness regions attract mutation/selection.
/// - High-failure regions repel mutation/selection.
pub struct StigmergicField {
    /// Sparse map from quantized voxel key to accumulated signal.
    voxels: HashMap<VoxelKey, VoxelSignal>,
    /// Voxel size in embedding space (quantization step).
    resolution: f32,
    /// Multiplicative decay per generation (e.g., 0.95 = half-life ~14 gens).
    decay_rate: f32,
}

impl StigmergicField {
    /// Create a new empty stigmergic field.
    ///
    /// # Arguments
    /// - `resolution`: voxel size in embedding space. Smaller = finer grid.
    pub fn new(resolution: f32) -> Self {
        Self {
            voxels: HashMap::new(),
            resolution: resolution.max(1e-6), // prevent division by zero
            decay_rate: 0.95,
        }
    }

    /// Create a new field with a custom decay rate.
    pub fn with_decay_rate(resolution: f32, decay_rate: f32) -> Self {
        Self {
            voxels: HashMap::new(),
            resolution: resolution.max(1e-6),
            decay_rate: decay_rate.clamp(0.0, 1.0),
        }
    }

    /// Number of active voxels in the field.
    pub fn voxel_count(&self) -> usize {
        self.voxels.len()
    }

    /// Deposit a signal at a program's embedding location.
    ///
    /// # Arguments
    /// - `embedding`: the program's embedding vector (any dimensionality >= 8).
    /// - `fitness`: the program's fitness score (sum of objectives or correctness).
    /// - `failed`: whether the program failed evaluation entirely.
    /// - `generation`: current generation number.
    pub fn deposit(&mut self, embedding: &[f32], fitness: f32, failed: bool, generation: u64) {
        let key = self.quantize(embedding);
        let signal = self.voxels.entry(key).or_insert_with(|| VoxelSignal::new(generation));

        if failed {
            signal.failure_count += 1;
        } else {
            signal.fitness_sum += fitness;
            signal.fitness_count += 1;
        }
        signal.last_updated = generation;
    }

    /// Read the signal at a location (interpolated from the voxel and neighbors).
    ///
    /// Reads the center voxel plus its 2*VOXEL_DIMS axis-aligned neighbors
    /// (one step in each direction along each axis). Weights contributions
    /// by inverse Manhattan distance.
    pub fn read(&self, embedding: &[f32]) -> SignalReading {
        let center_key = self.quantize(embedding);

        // Collect the center voxel and axis-aligned neighbors.
        let mut total_weight = 0.0f32;
        let mut weighted_fitness = 0.0f32;
        let mut weighted_failure = 0.0f32;
        let mut total_density = 0u32;

        // Center voxel gets weight 1.0.
        if let Some(signal) = self.voxels.get(&center_key) {
            let w = 1.0;
            total_weight += w;
            weighted_fitness += signal.avg_fitness() * w;
            weighted_failure += signal.failure_rate() * w;
            total_density += signal.density();
        }

        // Axis-aligned neighbors get weight 0.5 (one step away).
        for dim in 0..VOXEL_DIMS.min(center_key.len()) {
            for &delta in &[-1i16, 1i16] {
                let mut neighbor = center_key;
                neighbor[dim] = center_key[dim].saturating_add(delta);
                if neighbor == center_key {
                    continue; // saturating_add didn't change (at bounds)
                }
                if let Some(signal) = self.voxels.get(&neighbor) {
                    let w = 0.5;
                    total_weight += w;
                    weighted_fitness += signal.avg_fitness() * w;
                    weighted_failure += signal.failure_rate() * w;
                    total_density += signal.density();
                }
            }
        }

        if total_weight < 1e-12 {
            return SignalReading::ZERO;
        }

        SignalReading {
            avg_fitness: weighted_fitness / total_weight,
            failure_rate: weighted_failure / total_weight,
            density: total_density,
        }
    }

    /// Decay all signals by the decay rate.
    ///
    /// Called once per generation. Multiplies all fitness_sum values by
    /// decay_rate, effectively creating exponential decay with
    /// half-life = ln(2) / ln(1/decay_rate) generations.
    pub fn decay(&mut self) {
        let rate = self.decay_rate;
        for signal in self.voxels.values_mut() {
            signal.fitness_sum *= rate;
            // Decay failure count: multiply and truncate.
            // This converts to f32, decays, and rounds down.
            signal.failure_count = (signal.failure_count as f32 * rate) as u32;
            // Decay fitness count similarly so avg_fitness stays meaningful.
            signal.fitness_count = (signal.fitness_count as f32 * rate) as u32;
        }
    }

    /// Prune empty/negligible voxels to save memory.
    ///
    /// Removes voxels where the signal magnitude is below `min_signal`
    /// and there are no failure deposits.
    pub fn prune(&mut self, min_signal: f32) {
        self.voxels.retain(|_, signal| !signal.is_negligible(min_signal));
    }

    /// Compute the gradient direction at a point in embedding space.
    ///
    /// Returns a vector (length = min(embedding.len(), 64)) pointing toward
    /// high-fitness regions and away from high-failure regions. This can be
    /// used to bias mutation perturbations.
    ///
    /// The gradient is estimated via finite differences along each axis:
    /// for each dimension, compare the signal one voxel step in each
    /// direction.
    pub fn gradient(&self, embedding: &[f32]) -> Vec<f32> {
        let ndims = embedding.len().min(64);
        let mut grad = vec![0.0f32; ndims];

        let center_key = self.quantize(embedding);

        // For each dimension, compute the signal difference between
        // the +1 and -1 neighbors along that axis.
        for dim in 0..VOXEL_DIMS.min(ndims) {
            let mut key_plus = center_key;
            let mut key_minus = center_key;
            key_plus[dim] = center_key[dim].saturating_add(1);
            key_minus[dim] = center_key[dim].saturating_add(-1);

            let signal_plus = self.voxels.get(&key_plus).map(|s| {
                s.avg_fitness() - s.failure_rate()
            }).unwrap_or(0.0);

            let signal_minus = self.voxels.get(&key_minus).map(|s| {
                s.avg_fitness() - s.failure_rate()
            }).unwrap_or(0.0);

            // Gradient points toward higher (fitness - failure_rate).
            grad[dim] = signal_plus - signal_minus;
        }

        // Normalize the gradient to unit length (if non-zero).
        let mag: f32 = grad.iter().map(|x| x * x).sum::<f32>().sqrt();
        if mag > 1e-12 {
            for g in &mut grad {
                *g /= mag;
            }
        }

        grad
    }

    /// Quantize an embedding vector to a voxel key.
    ///
    /// Takes the first 8 dimensions of the embedding, divides each by
    /// the resolution, and rounds to the nearest i16.
    fn quantize(&self, embedding: &[f32]) -> VoxelKey {
        let mut key = [0i16; VOXEL_DIMS];
        let inv_res = 1.0 / self.resolution;
        for i in 0..VOXEL_DIMS.min(embedding.len()) {
            // Round to nearest integer voxel coordinate.
            let quantized = (embedding[i] * inv_res).round();
            // Clamp to i16 range.
            key[i] = quantized.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        }
        key
    }
}

// ---------------------------------------------------------------------------
// Mutation bias helper
// ---------------------------------------------------------------------------

/// Apply stigmergic gradient bias to an embedding perturbation.
///
/// Blends a random perturbation vector with the stigmergic gradient,
/// controlled by `gradient_weight` in [0.0, 1.0].
///
/// - `perturbation`: the original random mutation perturbation.
/// - `gradient`: the stigmergic gradient at the current location.
/// - `gradient_weight`: how much to blend the gradient in (0.0 = pure random, 1.0 = pure gradient).
///
/// Returns the biased perturbation vector.
pub fn bias_perturbation(
    perturbation: &[f32],
    gradient: &[f32],
    gradient_weight: f32,
) -> Vec<f32> {
    let w = gradient_weight.clamp(0.0, 1.0);
    let len = perturbation.len();
    let mut biased = Vec::with_capacity(len);

    for i in 0..len {
        let g = if i < gradient.len() { gradient[i] } else { 0.0 };
        biased.push(perturbation[i] * (1.0 - w) + g * w);
    }

    biased
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a simple 64-dim embedding with the first few dims set.
    fn make_embedding(first_dims: &[f32]) -> Vec<f32> {
        let mut emb = vec![0.0; 64];
        for (i, &v) in first_dims.iter().enumerate() {
            if i < 64 {
                emb[i] = v;
            }
        }
        emb
    }

    #[test]
    fn test_deposit_and_read_roundtrip() {
        let mut field = StigmergicField::new(1.0);
        let emb = make_embedding(&[5.0, 3.0, 1.0]);

        // Deposit a successful program.
        field.deposit(&emb, 0.8, false, 0);

        let reading = field.read(&emb);
        assert!((reading.avg_fitness - 0.8).abs() < 1e-6,
            "Expected avg_fitness ~0.8, got {}", reading.avg_fitness);
        assert_eq!(reading.failure_rate, 0.0);
        assert_eq!(reading.density, 1);
    }

    #[test]
    fn test_deposit_multiple_signals() {
        let mut field = StigmergicField::new(1.0);
        let emb = make_embedding(&[5.0, 3.0, 1.0]);

        field.deposit(&emb, 0.6, false, 0);
        field.deposit(&emb, 0.8, false, 0);
        field.deposit(&emb, 0.0, true, 0);

        let reading = field.read(&emb);
        // avg_fitness = (0.6 + 0.8) / 2 = 0.7
        assert!((reading.avg_fitness - 0.7).abs() < 1e-5,
            "Expected avg_fitness ~0.7, got {}", reading.avg_fitness);
        // failure_rate = 1 / 3 = 0.333...
        assert!((reading.failure_rate - 1.0 / 3.0).abs() < 1e-5,
            "Expected failure_rate ~0.333, got {}", reading.failure_rate);
        assert_eq!(reading.density, 3);
    }

    #[test]
    fn test_decay_reduces_signals() {
        let mut field = StigmergicField::with_decay_rate(1.0, 0.5);
        let emb = make_embedding(&[10.0, 10.0]);

        field.deposit(&emb, 1.0, false, 0);
        // Before decay: fitness_sum=1.0, count=1.
        let before = field.read(&emb);
        assert!(before.avg_fitness > 0.0);

        field.decay();
        // After one decay: fitness_sum=0.5, count should decay too.
        let after = field.read(&emb);
        // The signal should be reduced.
        assert!(after.avg_fitness <= before.avg_fitness + 1e-6,
            "Signal should not increase after decay: before={}, after={}",
            before.avg_fitness, after.avg_fitness);

        // Apply many decay steps.
        for _ in 0..20 {
            field.decay();
        }
        let final_reading = field.read(&emb);
        assert!(final_reading.avg_fitness < 0.01,
            "After many decays, signal should be near zero: {}", final_reading.avg_fitness);
    }

    #[test]
    fn test_decay_with_default_rate() {
        let mut field = StigmergicField::new(1.0);
        let emb = make_embedding(&[1.0, 2.0]);

        // Deposit a large signal.
        for _ in 0..100 {
            field.deposit(&emb, 1.0, false, 0);
        }

        let before = field.read(&emb);
        assert!(before.avg_fitness > 0.0);

        // Decay 14 times (approximately one half-life at 0.95 rate).
        for _ in 0..14 {
            field.decay();
        }

        let after = field.read(&emb);
        // After ~14 generations (half-life), fitness_sum should be roughly halved.
        // fitness_sum = 100 * 0.95^14 ≈ 48.8, count = 100 * 0.95^14 ≈ 48
        // avg_fitness should stay roughly the same since both decay equally.
        // But the total density drops.
        assert!(after.density < before.density,
            "Density should decrease after decay: before={}, after={}",
            before.density, after.density);
    }

    #[test]
    fn test_gradient_points_toward_high_fitness() {
        let mut field = StigmergicField::new(1.0);

        // Place high fitness at +x direction.
        let high = make_embedding(&[10.0, 0.0]);
        field.deposit(&high, 1.0, false, 0);

        // Place low fitness at -x direction.
        let low = make_embedding(&[-10.0, 0.0]);
        field.deposit(&low, 0.1, false, 0);

        // Query gradient at origin.
        let origin = make_embedding(&[0.0, 0.0]);
        let _grad = field.gradient(&origin);

        // With resolution=1.0, the voxels at [10,0,...] and [-10,0,...] are
        // far from origin. Let's instead use points that are exactly 1 voxel apart.
        let mut field2 = StigmergicField::new(1.0);
        let pos = make_embedding(&[1.0, 0.0]);
        let neg = make_embedding(&[-1.0, 0.0]);
        field2.deposit(&pos, 1.0, false, 0);
        field2.deposit(&neg, 0.1, false, 0);

        let grad2 = field2.gradient(&origin);
        // Gradient dim 0 should be positive (pointing toward +x = high fitness).
        assert!(grad2[0] > 0.0,
            "Gradient should point toward high fitness: grad[0]={}", grad2[0]);
    }

    #[test]
    fn test_gradient_away_from_failures() {
        let mut field = StigmergicField::new(1.0);

        // Place failures at +x direction.
        let fail_loc = make_embedding(&[1.0, 0.0]);
        for _ in 0..10 {
            field.deposit(&fail_loc, 0.0, true, 0);
        }

        // Place success at -x direction.
        let succ_loc = make_embedding(&[-1.0, 0.0]);
        field.deposit(&succ_loc, 0.5, false, 0);

        let origin = make_embedding(&[0.0, 0.0]);
        let grad = field.gradient(&origin);

        // Gradient dim 0 should be negative (pointing away from failures, toward success).
        assert!(grad[0] < 0.0,
            "Gradient should point away from failures: grad[0]={}", grad[0]);
    }

    #[test]
    fn test_prune_removes_dead_voxels() {
        let mut field = StigmergicField::new(1.0);
        let emb1 = make_embedding(&[1.0, 0.0]);
        let emb2 = make_embedding(&[100.0, 0.0]);

        // Deposit a tiny signal at emb1 and a large one at emb2.
        field.deposit(&emb1, 0.001, false, 0);
        field.deposit(&emb2, 100.0, false, 0);

        assert_eq!(field.voxel_count(), 2);

        // Prune with a threshold that kills the weak signal but keeps the strong one.
        field.prune(0.01);

        // The weak voxel (0.001 < 0.01) should be pruned.
        // The strong voxel (100.0 > 0.01) should remain.
        assert_eq!(field.voxel_count(), 1,
            "After prune, weak voxel should be removed: got {} voxels", field.voxel_count());

        // emb2 should still be readable.
        let reading = field.read(&emb2);
        assert!(reading.avg_fitness > 0.0, "Strong voxel should survive prune");
        assert!(reading.density > 0, "Strong voxel density should be > 0");
    }

    #[test]
    fn test_prune_keeps_failure_voxels() {
        let mut field = StigmergicField::new(1.0);
        let emb = make_embedding(&[5.0, 5.0]);

        // Deposit only a failure (fitness_sum = 0, but failure_count > 0).
        field.deposit(&emb, 0.0, true, 0);

        assert_eq!(field.voxel_count(), 1);

        // Prune — the failure voxel should NOT be pruned because
        // failure_count > 0 is valuable information.
        field.prune(0.01);
        assert_eq!(field.voxel_count(), 1,
            "Failure voxels should not be pruned");
    }

    #[test]
    fn test_quantization_deterministic() {
        let field = StigmergicField::new(0.5);
        let emb = make_embedding(&[1.23, -4.56, 0.01, 7.89]);

        let k1 = field.quantize(&emb);
        let k2 = field.quantize(&emb);
        assert_eq!(k1, k2, "Quantization should be deterministic");
    }

    #[test]
    fn test_quantization_groups_nearby_points() {
        let field = StigmergicField::new(1.0);

        let a = make_embedding(&[1.4, 2.4, 3.4]);
        let b = make_embedding(&[1.5, 2.5, 3.5]);

        let ka = field.quantize(&a);
        let _kb = field.quantize(&b);
        // With resolution=1.0, 1.4 rounds to 1 and 1.5 rounds to 2.
        // So they may or may not be the same voxel depending on rounding.
        // But points within the same voxel should definitely match.
        let c = make_embedding(&[1.1, 2.1, 3.1]);
        let kc = field.quantize(&c);
        assert_eq!(ka, kc, "Points within same voxel should have same key");
    }

    #[test]
    fn test_empty_field_reads_zero() {
        let field = StigmergicField::new(1.0);
        let emb = make_embedding(&[42.0, -17.0]);

        let reading = field.read(&emb);
        assert_eq!(reading.avg_fitness, 0.0);
        assert_eq!(reading.failure_rate, 0.0);
        assert_eq!(reading.density, 0);
    }

    #[test]
    fn test_read_interpolates_neighbors() {
        let mut field = StigmergicField::new(1.0);

        // Deposit signal at a neighbor voxel (one step away along dim 0).
        let neighbor = make_embedding(&[1.0, 0.0]);
        field.deposit(&neighbor, 0.9, false, 0);

        // Read at origin — should pick up the neighbor signal with reduced weight.
        let origin = make_embedding(&[0.0, 0.0]);
        let reading = field.read(&origin);

        assert!(reading.avg_fitness > 0.0,
            "Should interpolate signal from neighbor: {}", reading.avg_fitness);
        // But it should be less than the raw signal (neighbor weighted at 0.5).
        assert!(reading.avg_fitness < 0.91,
            "Interpolated signal should be attenuated: {}", reading.avg_fitness);
    }

    #[test]
    fn test_bias_perturbation() {
        let perturbation = vec![1.0, 0.0, 0.0, 0.0];
        let gradient = vec![0.0, 1.0, 0.0, 0.0];

        // No gradient bias.
        let biased = bias_perturbation(&perturbation, &gradient, 0.0);
        assert_eq!(biased, perturbation);

        // Full gradient bias.
        let biased = bias_perturbation(&perturbation, &gradient, 1.0);
        assert_eq!(biased, gradient);

        // Half and half.
        let biased = bias_perturbation(&perturbation, &gradient, 0.5);
        assert!((biased[0] - 0.5).abs() < 1e-6);
        assert!((biased[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_short_embedding() {
        // Embeddings shorter than 8 dims should still work.
        let mut field = StigmergicField::new(1.0);
        let short = vec![1.0, 2.0];

        field.deposit(&short, 0.5, false, 0);
        let reading = field.read(&short);
        assert!((reading.avg_fitness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_gradient_normalized() {
        let mut field = StigmergicField::new(1.0);

        let pos = make_embedding(&[1.0, 0.0]);
        let neg = make_embedding(&[-1.0, 0.0]);
        field.deposit(&pos, 10.0, false, 0);
        field.deposit(&neg, 0.0, false, 0);

        let origin = make_embedding(&[0.0, 0.0]);
        let grad = field.gradient(&origin);

        // Gradient should be unit length.
        let mag: f32 = grad.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((mag - 1.0).abs() < 1e-5,
            "Gradient should be normalized: magnitude={}", mag);
    }

    #[test]
    fn test_gradient_zero_when_no_signal() {
        let field = StigmergicField::new(1.0);
        let origin = make_embedding(&[0.0, 0.0]);
        let grad = field.gradient(&origin);

        // All zeros (no signal anywhere).
        assert!(grad.iter().all(|&g| g == 0.0),
            "Gradient should be zero when no signal");
    }
}
