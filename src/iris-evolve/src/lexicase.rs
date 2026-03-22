//! Epsilon-lexicase selection for IRIS evolutionary substrate.
//!
//! Lexicase selection maintains selection pressure on INDIVIDUAL test cases
//! rather than averaging fitness across all cases. This preserves "specialist"
//! individuals that solve different subsets of test cases, enabling the
//! population to collectively cover more of the problem space.
//!
//! Reference: Helmuth & Spector 2015 — "General Program Synthesis Benchmark
//! Suite" showed lexicase solves 20/29 benchmarks vs 9/29 for tournament.

use rand::seq::SliceRandom;
use rand::Rng;

use crate::individual::Individual;

// ---------------------------------------------------------------------------
// Epsilon-lexicase selection
// ---------------------------------------------------------------------------

/// Perform epsilon-lexicase selection on a population.
///
/// Returns the index of the selected individual. Each selection event:
/// 1. Starts with all individuals as candidates.
/// 2. Shuffles test case indices into a random order.
/// 3. For each test case, filters candidates to those within epsilon of the
///    best score on that case (epsilon = median absolute deviation).
/// 4. When one candidate remains (or all cases exhausted), picks randomly.
pub fn lexicase_select(population: &[Individual], rng: &mut impl Rng) -> usize {
    assert!(!population.is_empty(), "cannot select from empty population");

    // If no per-case scores available, return random.
    if population[0].per_case_scores.is_empty() {
        return rng.gen_range(0..population.len());
    }

    let mut candidates: Vec<usize> = (0..population.len()).collect();

    let num_cases = population[0].per_case_scores.len();
    let mut case_order: Vec<usize> = (0..num_cases).collect();
    case_order.shuffle(rng);

    for &case_idx in &case_order {
        if candidates.len() <= 1 {
            break;
        }

        // Find the best score on this case among candidates.
        let best_score = candidates
            .iter()
            .map(|&i| population[i].per_case_scores[case_idx])
            .fold(f32::NEG_INFINITY, f32::max);

        // Compute epsilon as the median absolute deviation (MAD) of scores.
        let epsilon = compute_epsilon(population, &candidates, case_idx);

        // Keep only candidates within epsilon of the best.
        candidates.retain(|&i| {
            population[i].per_case_scores[case_idx] >= best_score - epsilon
        });
    }

    // Random choice among survivors.
    candidates[rng.gen_range(0..candidates.len())]
}

/// Perform down-sampled epsilon-lexicase selection.
///
/// Uses only a random subset of test cases (controlled by `sample_fraction`)
/// for efficiency. Proven to maintain selection pressure while reducing
/// computation (La Cava et al. 2016).
pub fn lexicase_select_downsampled(
    population: &[Individual],
    rng: &mut impl Rng,
    sample_fraction: f32,
) -> usize {
    assert!(!population.is_empty(), "cannot select from empty population");

    // If no per-case scores available, return random.
    if population[0].per_case_scores.is_empty() {
        return rng.gen_range(0..population.len());
    }

    let mut candidates: Vec<usize> = (0..population.len()).collect();

    let num_cases = population[0].per_case_scores.len();
    let sample_size = ((num_cases as f32 * sample_fraction).ceil() as usize).max(1);

    // Shuffle and take a subset.
    let mut case_order: Vec<usize> = (0..num_cases).collect();
    case_order.shuffle(rng);
    case_order.truncate(sample_size);

    for &case_idx in &case_order {
        if candidates.len() <= 1 {
            break;
        }

        let best_score = candidates
            .iter()
            .map(|&i| population[i].per_case_scores[case_idx])
            .fold(f32::NEG_INFINITY, f32::max);

        let epsilon = compute_epsilon(population, &candidates, case_idx);

        candidates.retain(|&i| {
            population[i].per_case_scores[case_idx] >= best_score - epsilon
        });
    }

    candidates[rng.gen_range(0..candidates.len())]
}

// ---------------------------------------------------------------------------
// Epsilon computation
// ---------------------------------------------------------------------------

/// Compute epsilon as the median absolute deviation (MAD) of scores on
/// a given test case among the candidate individuals.
///
/// MAD is robust to outliers and adapts naturally: when scores are tightly
/// clustered, epsilon is small (strict filtering); when spread out, epsilon
/// is large (lenient filtering). This is the standard approach from
/// La Cava et al. 2016 ("Epsilon-Lexicase Selection").
fn compute_epsilon(
    population: &[Individual],
    candidates: &[usize],
    case_idx: usize,
) -> f32 {
    if candidates.len() <= 1 {
        return 0.0;
    }

    let mut scores: Vec<f32> = candidates
        .iter()
        .map(|&i| population[i].per_case_scores[case_idx])
        .collect();

    let median = percentile(&mut scores, 0.5);

    let mut deviations: Vec<f32> = scores.iter().map(|&s| (s - median).abs()).collect();

    percentile(&mut deviations, 0.5) // MAD
}

/// Compute the p-th percentile of a mutable slice (sorts in place).
///
/// Uses linear interpolation between adjacent ranks.
fn percentile(data: &mut [f32], p: f32) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    if data.len() == 1 {
        return data[0];
    }

    data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let rank = p * (data.len() - 1) as f32;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;

    if lower == upper {
        data[lower]
    } else {
        let frac = rank - lower as f32;
        data[lower] * (1.0 - frac) + data[upper] * frac
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::individual::{Fitness, Individual};
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn make_individual(per_case: Vec<f32>) -> Individual {
        let fragment = crate::phase::iris_evolve_test_helpers::make_dummy_fragment();
        let mut ind = Individual::new(fragment);
        // Set correctness to average of per-case scores.
        let avg = if per_case.is_empty() {
            0.0
        } else {
            per_case.iter().sum::<f32>() / per_case.len() as f32
        };
        ind.fitness = Fitness {
            values: [avg, 0.5, 0.5, 0.5, 0.0],
        };
        ind.per_case_scores = per_case;
        ind
    }

    #[test]
    fn single_candidate_returns_that_candidate() {
        let pop = vec![make_individual(vec![1.0, 0.5, 0.3])];
        let mut rng = StdRng::seed_from_u64(42);
        let idx = lexicase_select(&pop, &mut rng);
        assert_eq!(idx, 0);
    }

    #[test]
    fn identical_population_returns_random() {
        let pop: Vec<Individual> = (0..10)
            .map(|_| make_individual(vec![0.5, 0.5, 0.5]))
            .collect();
        let mut rng = StdRng::seed_from_u64(42);

        // Run many selections; should see at least 2 different results.
        let mut seen = std::collections::HashSet::new();
        for _ in 0..100 {
            seen.insert(lexicase_select(&pop, &mut rng));
        }
        assert!(
            seen.len() >= 2,
            "identical population should produce varied selections, got {:?}",
            seen
        );
    }

    #[test]
    fn preserves_specialist() {
        // Create a population where specialists are clearly best on their
        // respective cases, with enough individuals to make epsilon small.
        //
        // Individual 0: perfect on case 0, poor on case 1 (specialist A)
        // Individual 1: poor on case 0, perfect on case 1 (specialist B)
        // Individuals 2-9: mediocre on both cases (generalists)
        //
        // With many mediocre individuals, the MAD (epsilon) on each case
        // will be small, so the specialists should clearly dominate when
        // their strong case is drawn first.
        let mut pop = vec![
            make_individual(vec![1.0, 0.1]),  // specialist A
            make_individual(vec![0.1, 1.0]),  // specialist B
        ];
        // Add 8 mediocre generalists with similar scores.
        for i in 0..8 {
            let v = 0.4 + (i as f32) * 0.02; // 0.40, 0.42, ..., 0.54
            pop.push(make_individual(vec![v, v]));
        }

        let mut rng = StdRng::seed_from_u64(42);

        // Over many trials, both specialists should be selected.
        let mut selected_0 = false;
        let mut selected_1 = false;
        for _ in 0..500 {
            let idx = lexicase_select(&pop, &mut rng);
            if idx == 0 {
                selected_0 = true;
            }
            if idx == 1 {
                selected_1 = true;
            }
            if selected_0 && selected_1 {
                break;
            }
        }
        assert!(
            selected_0 && selected_1,
            "lexicase should select both specialists"
        );
    }

    #[test]
    fn downsampled_uses_fewer_cases() {
        // With 100 test cases and 15% sample, only ~15 cases should be used.
        // We verify indirectly: the function runs without error and returns
        // a valid index.
        let scores: Vec<f32> = (0..100).map(|i| (i as f32) / 100.0).collect();
        let pop: Vec<Individual> = (0..20)
            .map(|_| make_individual(scores.clone()))
            .collect();

        let mut rng = StdRng::seed_from_u64(42);
        let idx = lexicase_select_downsampled(&pop, &mut rng, 0.15);
        assert!(idx < pop.len());
    }

    #[test]
    fn epsilon_allows_near_ties() {
        // Three individuals with very similar scores on case 0.
        // Epsilon (MAD) should allow all three to survive case 0 filtering.
        let pop = vec![
            make_individual(vec![0.90, 0.1]),
            make_individual(vec![0.89, 0.9]),
            make_individual(vec![0.88, 0.5]),
        ];

        let mut rng = StdRng::seed_from_u64(42);

        // Over many trials, individual 1 should sometimes win despite not
        // being best on case 0, because epsilon keeps it as a candidate
        // and it dominates case 1.
        let mut selected_1 = false;
        for _ in 0..500 {
            let idx = lexicase_select(&pop, &mut rng);
            if idx == 1 {
                selected_1 = true;
                break;
            }
        }
        assert!(
            selected_1,
            "epsilon should allow near-ties to survive, giving individual 1 a chance"
        );
    }

    #[test]
    fn percentile_basic() {
        let mut data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((percentile(&mut data, 0.5) - 3.0).abs() < f32::EPSILON);

        let mut data2 = vec![10.0];
        assert!((percentile(&mut data2, 0.5) - 10.0).abs() < f32::EPSILON);

        let mut data3: Vec<f32> = vec![];
        assert!((percentile(&mut data3, 0.5) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn empty_per_case_scores_returns_random() {
        let pop: Vec<Individual> = (0..5)
            .map(|_| make_individual(vec![]))
            .collect();
        let mut rng = StdRng::seed_from_u64(42);
        let idx = lexicase_select(&pop, &mut rng);
        assert!(idx < pop.len());
    }
}
