use rand::Rng;

use crate::individual::{Fitness, NUM_OBJECTIVES};

// ---------------------------------------------------------------------------
// Non-dominated sorting (NSGA-II)
// ---------------------------------------------------------------------------

/// Performs non-dominated sorting on a population.
///
/// Returns layers of Pareto fronts: `fronts[0]` is the non-dominated set,
/// `fronts[1]` is dominated only by `fronts[0]`, etc. Each entry is an index
/// into the original `fitnesses` slice.
pub fn non_dominated_sort(fitnesses: &[Fitness]) -> Vec<Vec<usize>> {
    let n = fitnesses.len();
    if n == 0 {
        return vec![];
    }

    // domination_count[i] = how many individuals dominate i
    let mut domination_count = vec![0usize; n];
    // dominated_set[i] = set of individuals that i dominates
    let mut dominated_set: Vec<Vec<usize>> = vec![vec![]; n];

    for i in 0..n {
        for j in (i + 1)..n {
            if fitnesses[i].dominates(&fitnesses[j]) {
                dominated_set[i].push(j);
                domination_count[j] += 1;
            } else if fitnesses[j].dominates(&fitnesses[i]) {
                dominated_set[j].push(i);
                domination_count[i] += 1;
            }
        }
    }

    let mut fronts: Vec<Vec<usize>> = vec![];

    // First front: individuals with domination count 0
    let mut current_front: Vec<usize> = (0..n)
        .filter(|&i| domination_count[i] == 0)
        .collect();

    while !current_front.is_empty() {
        let mut next_front = vec![];
        for &i in &current_front {
            for &j in &dominated_set[i] {
                domination_count[j] -= 1;
                if domination_count[j] == 0 {
                    next_front.push(j);
                }
            }
        }
        fronts.push(current_front);
        current_front = next_front;
    }

    fronts
}

// ---------------------------------------------------------------------------
// Crowding distance
// ---------------------------------------------------------------------------

/// Compute crowding distance for individuals within a single Pareto front.
///
/// `front_indices` are indices into `fitnesses`. Returns a crowding distance
/// value for each entry in `front_indices` (same order).
pub fn crowding_distance(front_indices: &[usize], fitnesses: &[Fitness]) -> Vec<f32> {
    let n = front_indices.len();
    if n <= 2 {
        return vec![f32::INFINITY; n];
    }

    let mut distances = vec![0.0f32; n];
    let num_objectives = NUM_OBJECTIVES;

    for obj in 0..num_objectives {
        // Sort front by this objective
        let mut sorted: Vec<usize> = (0..n).collect();
        sorted.sort_by(|&a, &b| {
            let fa = fitnesses[front_indices[a]].values[obj];
            let fb = fitnesses[front_indices[b]].values[obj];
            fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Boundary individuals get infinite distance
        distances[sorted[0]] = f32::INFINITY;
        distances[sorted[n - 1]] = f32::INFINITY;

        let f_min = fitnesses[front_indices[sorted[0]]].values[obj];
        let f_max = fitnesses[front_indices[sorted[n - 1]]].values[obj];
        let range = f_max - f_min;

        if range < f32::EPSILON {
            continue;
        }

        for k in 1..(n - 1) {
            let prev = fitnesses[front_indices[sorted[k - 1]]].values[obj];
            let next = fitnesses[front_indices[sorted[k + 1]]].values[obj];
            distances[sorted[k]] += (next - prev) / range;
        }
    }

    distances
}

// ---------------------------------------------------------------------------
// Tournament selection
// ---------------------------------------------------------------------------

/// Tournament selection with Pareto rank and crowding distance tie-breaking.
///
/// Selects one individual index from the population. Lower `pareto_rank` is
/// better; when equal, higher `crowding_distance` wins.
pub fn tournament_select(
    pareto_ranks: &[usize],
    crowding_distances: &[f32],
    k: usize,
    rng: &mut impl Rng,
) -> usize {
    let n = pareto_ranks.len();
    assert!(n > 0, "cannot select from empty population");
    let k = k.min(n);

    let mut best = rng.gen_range(0..n);
    for _ in 1..k {
        let candidate = rng.gen_range(0..n);
        if pareto_ranks[candidate] < pareto_ranks[best]
            || (pareto_ranks[candidate] == pareto_ranks[best]
                && crowding_distances[candidate] > crowding_distances[best])
        {
            best = candidate;
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_dominated_sort_simple() {
        let fitnesses = vec![
            Fitness { values: [1.0, 0.0, 0.5, 0.5, 0.0] }, // A: best in obj 0
            Fitness { values: [0.0, 1.0, 0.5, 0.5, 0.0] }, // B: best in obj 1
            Fitness { values: [0.0, 0.0, 0.0, 0.0, 0.0] }, // C: dominated by both
        ];
        let fronts = non_dominated_sort(&fitnesses);
        assert_eq!(fronts.len(), 2);
        // Front 0 should contain A and B
        assert!(fronts[0].contains(&0));
        assert!(fronts[0].contains(&1));
        // Front 1 should contain C
        assert!(fronts[1].contains(&2));
    }

    #[test]
    fn test_dominates() {
        let a = Fitness { values: [1.0, 1.0, 1.0, 1.0, 1.0] };
        let b = Fitness { values: [0.5, 0.5, 0.5, 0.5, 0.5] };
        assert!(a.dominates(&b));
        assert!(!b.dominates(&a));
        assert!(!a.dominates(&a));
    }

    #[test]
    fn test_crowding_distance_boundary() {
        let fitnesses = vec![
            Fitness { values: [0.0, 0.0, 0.0, 0.0, 0.0] },
            Fitness { values: [0.5, 0.5, 0.5, 0.5, 0.5] },
            Fitness { values: [1.0, 1.0, 1.0, 1.0, 1.0] },
        ];
        let front = vec![0, 1, 2];
        let dists = crowding_distance(&front, &fitnesses);
        assert!(dists[0].is_infinite());
        assert!(dists[2].is_infinite());
        assert!(dists[1].is_finite());
    }
}
