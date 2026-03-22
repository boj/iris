use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::individual::{Fitness, Individual};

// ---------------------------------------------------------------------------
// EvolutionResult
// ---------------------------------------------------------------------------

/// Output of an evolutionary run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionResult {
    /// Best individual found (highest sum of fitness components).
    pub best_individual: Individual,
    /// Pareto-optimal front at termination.
    pub pareto_front: Vec<Individual>,
    /// Number of generations actually run.
    pub generations_run: usize,
    /// Total wall-clock time (stored as nanoseconds for serde compatibility).
    #[serde(
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub total_time: Duration,
    /// Per-generation fitness history (best fitness per generation).
    pub history: Vec<GenerationSnapshot>,
}

fn serialize_duration<S: serde::Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_u128(d.as_nanos())
}

fn deserialize_duration<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
    let nanos = u128::deserialize(d)?;
    Ok(Duration::from_nanos(nanos as u64))
}

// ---------------------------------------------------------------------------
// GenerationSnapshot
// ---------------------------------------------------------------------------

/// Snapshot of a single generation's state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationSnapshot {
    /// Generation index (0-based).
    pub generation: usize,
    /// Best fitness in this generation.
    pub best_fitness: Fitness,
    /// Average fitness across the population.
    pub avg_fitness: Fitness,
    /// Size of the Pareto front (rank 0).
    pub pareto_front_size: usize,
    /// Current evolutionary phase.
    pub phase: crate::population::Phase,
}
