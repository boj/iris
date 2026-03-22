use serde::{Deserialize, Serialize};

use iris_types::fragment::{Fragment, FragmentId};

// ---------------------------------------------------------------------------
// Fitness
// ---------------------------------------------------------------------------

/// Number of fitness objectives.
pub const NUM_OBJECTIVES: usize = 5;

/// 5-objective fitness vector: [correctness, performance, verifiability, cost, novelty].
/// Each component in [0.0, 1.0]. No scalarization; ranked via NSGA-II.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Fitness {
    pub values: [f32; NUM_OBJECTIVES],
}

impl Fitness {
    pub const ZERO: Self = Self {
        values: [0.0; NUM_OBJECTIVES],
    };

    /// Correctness component (index 0).
    pub fn correctness(&self) -> f32 {
        self.values[0]
    }

    /// Performance component (index 1).
    pub fn performance(&self) -> f32 {
        self.values[1]
    }

    /// Verifiability component (index 2).
    pub fn verifiability(&self) -> f32 {
        self.values[2]
    }

    /// Cost component (index 3).
    pub fn cost(&self) -> f32 {
        self.values[3]
    }

    /// Novelty component (index 4).
    pub fn novelty(&self) -> f32 {
        self.values[4]
    }

    /// Set a single fitness objective, clamping to [0.0, 1.0].
    ///
    /// Prefer this over direct `values[i] = ...` to ensure the invariant
    /// that all fitness components are in the valid range.
    pub fn set_value(&mut self, index: usize, value: f32) {
        if index < NUM_OBJECTIVES {
            self.values[index] = value.clamp(0.0, 1.0);
        }
    }

    /// Clamp all fitness components to [0.0, 1.0] in place.
    pub fn clamp_to_valid(&mut self) {
        for v in self.values.iter_mut() {
            *v = v.clamp(0.0, 1.0);
        }
    }

    /// True if `self` Pareto-dominates `other` (better or equal in all
    /// objectives, strictly better in at least one).
    pub fn dominates(&self, other: &Fitness) -> bool {
        let mut dominated = false;
        for i in 0..NUM_OBJECTIVES {
            if self.values[i] < other.values[i] {
                return false;
            }
            if self.values[i] > other.values[i] {
                dominated = true;
            }
        }
        dominated
    }
}

// ---------------------------------------------------------------------------
// IndividualMeta (SPEC Section 6.1 — 64 bytes = 1 cache line)
// ---------------------------------------------------------------------------

/// Compact metadata fitting a single 64-byte cache line.
/// Used for fast scanning during selection/ranking.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct IndividualMeta {
    /// Truncated FragmentId (first 8 bytes).
    pub fragment_id: [u8; 8],
    /// 16-dim compressed embedding projection (f16 stored as u16).
    pub embedding: [u16; 16],
    /// Fitness vector stored as f16 (u16 bits).
    pub fitness: [u16; 4],
    /// NSGA-II Pareto rank (0 = front 0).
    pub pareto_rank: u16,
    /// Crowding distance (f16 stored as u16).
    pub crowding_dist: u16,
    /// Age in generations.
    pub age: u16,
    /// Lineage hash for ancestry tracking.
    pub lineage_hash: u32,
    /// Flags (phase, elitism, etc.).
    pub flags: u16,
    /// L2 verification tier achieved (0-3).
    pub verify_tier: u8,
    /// Padding to reach 64 bytes.
    pub _pad: [u8; 3],
}

impl IndividualMeta {
    /// Create metadata from a fragment and fitness values.
    pub fn new(fragment_id: &FragmentId, fitness: &Fitness, generation: u16) -> Self {
        let mut fid_trunc = [0u8; 8];
        fid_trunc.copy_from_slice(&fragment_id.0[..8]);

        Self {
            fragment_id: fid_trunc,
            embedding: [0u16; 16],
            fitness: [
                f32_to_f16_bits(fitness.values[0]),
                f32_to_f16_bits(fitness.values[1]),
                f32_to_f16_bits(fitness.values[2]),
                f32_to_f16_bits(fitness.values[3]),
            ],
            pareto_rank: 0,
            crowding_dist: 0,
            age: generation,
            lineage_hash: 0,
            flags: 0,
            verify_tier: 0,
            _pad: [0; 3],
        }
    }
}

/// Minimal f32-to-f16 bit conversion (truncation, not rounding).
fn f32_to_f16_bits(v: f32) -> u16 {
    let bits = v.to_bits();
    let sign = (bits >> 16) & 0x8000;
    let exponent = ((bits >> 23) & 0xFF) as i32;
    let mantissa = bits & 0x007F_FFFF;

    if exponent == 0xFF {
        // Inf/NaN
        return (sign | 0x7C00 | if mantissa != 0 { 0x0200 } else { 0 }) as u16;
    }

    let new_exp = exponent - 127 + 15;
    if new_exp >= 31 {
        return (sign | 0x7C00) as u16; // overflow -> Inf
    }
    if new_exp <= 0 {
        return sign as u16; // underflow -> 0
    }

    (sign | ((new_exp as u32) << 10) | (mantissa >> 13)) as u16
}

// ---------------------------------------------------------------------------
// Individual
// ---------------------------------------------------------------------------

/// A full individual in the evolutionary population.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Individual {
    /// Compact cache-line-sized metadata.
    pub meta: IndividualMeta,
    /// Full fragment (genome).
    pub fragment: Fragment,
    /// Decoded fitness vector (f32 precision, used in selection).
    pub fitness: Fitness,
    /// NSGA-II Pareto rank (0 = non-dominated front).
    pub pareto_rank: usize,
    /// Crowding distance within the Pareto front.
    pub crowding_distance: f32,
    /// Per-test-case correctness scores (0.0-1.0 each).
    /// Used by lexicase selection to preserve specialists that solve
    /// different subsets of test cases.
    pub per_case_scores: Vec<f32>,
    /// Number of proof-failure diagnoses from graded verification.
    /// Stored as a count rather than the full Vec to keep Individual
    /// serializable without pulling in iris-kernel types.
    #[serde(default)]
    pub proof_failure_count: usize,
    /// Generation until which this individual is protected from replacement.
    /// Analyzer skeletons are protected for the first N generations to
    /// guarantee they survive to proper evaluation and selection.
    #[serde(default)]
    pub skeleton_protected_until: u64,
}

impl Individual {
    /// Create an individual from a fragment with zero fitness.
    pub fn new(fragment: Fragment) -> Self {
        let fitness = Fitness::ZERO;
        let meta = IndividualMeta::new(&fragment.id, &fitness, 0);
        Self {
            meta,
            fragment,
            fitness,
            pareto_rank: usize::MAX,
            crowding_distance: 0.0,
            per_case_scores: vec![],
            proof_failure_count: 0,
            skeleton_protected_until: 0,
        }
    }

    /// Create an individual from a fragment with skeleton protection.
    /// Protected individuals cannot be replaced until the given generation.
    pub fn new_protected(fragment: Fragment, protect_until: u64) -> Self {
        let mut ind = Self::new(fragment);
        ind.skeleton_protected_until = protect_until;
        ind
    }
}
