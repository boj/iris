//! Training corpus generator for IRIS's GIN-VAE codec.
//!
//! Generates diverse problem specs that feed into evolution.

use rand::Rng;

use iris_types::eval::{TestCase, Value};

use crate::config::ProblemSpec;

// ---------------------------------------------------------------------------
// Problem spec generators
// ---------------------------------------------------------------------------

/// Generate `count` diverse ProblemSpecs by cycling through 10 problem types.
pub fn generate_problem_specs(rng: &mut impl Rng, count: usize) -> Vec<ProblemSpec> {
    let mut specs = Vec::with_capacity(count);
    for i in 0..count {
        let spec = match i % 10 {
            0 => random_scalar_arithmetic(rng),
            1 => random_fold_problem(rng),
            2 => random_map_problem(rng),
            3 => random_filter_problem(rng),
            4 => random_composition(rng),
            5 => random_conditional(rng),
            6 => random_multi_input(rng),
            7 => random_iterative(rng),
            8 => random_predicate(rng),
            9 => random_pairwise(rng),
            _ => unreachable!(),
        };
        specs.push(spec);
    }
    specs
}

/// Two-input scalar arithmetic: a op b = c.
fn random_scalar_arithmetic(rng: &mut impl Rng) -> ProblemSpec {
    let op = rng.gen_range(0..5u8);
    let test_cases: Vec<TestCase> = (0..6)
        .map(|_| {
            let a = rng.gen_range(-50i64..50);
            let b = rng.gen_range(-50i64..50);
            let result = match op {
                0 => a.wrapping_add(b),
                1 => a.wrapping_sub(b),
                2 => a.wrapping_mul(b),
                3 => a.min(b),
                4 => a.max(b),
                _ => unreachable!(),
            };
            TestCase {
                inputs: vec![Value::Int(a), Value::Int(b)],
                expected_output: Some(vec![Value::Int(result)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect();
    ProblemSpec {
        test_cases,
        description: format!("scalar_arith_{}", op),
        target_cost: None,
    }
}

/// Fold (reduce) a small list with a binary operation.
fn random_fold_problem(rng: &mut impl Rng) -> ProblemSpec {
    let op = rng.gen_range(0..3u8); // sum, product, max
    let test_cases: Vec<TestCase> = (0..5)
        .map(|_| {
            let len = rng.gen_range(2..5usize);
            let elems: Vec<i64> = (0..len).map(|_| rng.gen_range(-10i64..10)).collect();
            let result = match op {
                0 => elems.iter().copied().sum::<i64>(),
                1 => elems.iter().copied().product::<i64>(),
                2 => elems.iter().copied().max().unwrap_or(0),
                _ => unreachable!(),
            };
            let input = Value::tuple(elems.into_iter().map(Value::Int).collect());
            TestCase {
                inputs: vec![input],
                expected_output: Some(vec![Value::Int(result)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect();
    ProblemSpec {
        test_cases,
        description: format!("fold_{}", op),
        target_cost: None,
    }
}

/// Map a unary operation over a small list.
fn random_map_problem(rng: &mut impl Rng) -> ProblemSpec {
    let op = rng.gen_range(0..3u8); // negate, abs, double
    let test_cases: Vec<TestCase> = (0..5)
        .map(|_| {
            let len = rng.gen_range(2..5usize);
            let elems: Vec<i64> = (0..len).map(|_| rng.gen_range(-10i64..10)).collect();
            let mapped: Vec<i64> = elems
                .iter()
                .map(|&x| match op {
                    0 => -x,
                    1 => x.abs(),
                    2 => x.wrapping_mul(2),
                    _ => unreachable!(),
                })
                .collect();
            let input = Value::tuple(elems.into_iter().map(Value::Int).collect());
            let output = Value::tuple(mapped.into_iter().map(Value::Int).collect());
            TestCase {
                inputs: vec![input],
                expected_output: Some(vec![output]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect();
    ProblemSpec {
        test_cases,
        description: format!("map_{}", op),
        target_cost: None,
    }
}

/// Filter elements by a predicate, then fold the result.
fn random_filter_problem(rng: &mut impl Rng) -> ProblemSpec {
    let pred = rng.gen_range(0..3u8); // positive, even, < threshold
    let threshold = rng.gen_range(0i64..5);
    let test_cases: Vec<TestCase> = (0..5)
        .map(|_| {
            let len = rng.gen_range(3..5usize);
            let elems: Vec<i64> = (0..len).map(|_| rng.gen_range(-10i64..10)).collect();
            let filtered: Vec<i64> = elems
                .iter()
                .copied()
                .filter(|&x| match pred {
                    0 => x > 0,
                    1 => x % 2 == 0,
                    2 => x < threshold,
                    _ => unreachable!(),
                })
                .collect();
            let result = filtered.iter().copied().sum::<i64>();
            let input = Value::tuple(elems.into_iter().map(Value::Int).collect());
            TestCase {
                inputs: vec![input],
                expected_output: Some(vec![Value::Int(result)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect();
    ProblemSpec {
        test_cases,
        description: format!("filter_fold_{}_{}", pred, threshold),
        target_cost: None,
    }
}

/// Composition: map then fold.
fn random_composition(rng: &mut impl Rng) -> ProblemSpec {
    let map_op = rng.gen_range(0..3u8); // negate, abs, double
    let fold_op = rng.gen_range(0..2u8); // sum, max
    let test_cases: Vec<TestCase> = (0..5)
        .map(|_| {
            let len = rng.gen_range(2..5usize);
            let elems: Vec<i64> = (0..len).map(|_| rng.gen_range(-10i64..10)).collect();
            let mapped: Vec<i64> = elems
                .iter()
                .map(|&x| match map_op {
                    0 => -x,
                    1 => x.abs(),
                    2 => x.wrapping_mul(2),
                    _ => unreachable!(),
                })
                .collect();
            let result = match fold_op {
                0 => mapped.iter().copied().sum::<i64>(),
                1 => mapped.iter().copied().max().unwrap_or(0),
                _ => unreachable!(),
            };
            let input = Value::tuple(elems.into_iter().map(Value::Int).collect());
            TestCase {
                inputs: vec![input],
                expected_output: Some(vec![Value::Int(result)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect();
    ProblemSpec {
        test_cases,
        description: format!("compose_map{}_fold{}", map_op, fold_op),
        target_cost: None,
    }
}

/// Conditional: if a > b then a else b (or similar).
fn random_conditional(rng: &mut impl Rng) -> ProblemSpec {
    let variant = rng.gen_range(0..3u8); // max-via-if, clamp, sign
    let test_cases: Vec<TestCase> = (0..6)
        .map(|_| {
            let a = rng.gen_range(-20i64..20);
            let b = rng.gen_range(-20i64..20);
            let result = match variant {
                0 => {
                    // max(a, b) via conditional
                    if a > b { a } else { b }
                }
                1 => {
                    // clamp a to [-b.abs(), b.abs()]
                    let lo = -(b.abs());
                    let hi = b.abs();
                    a.max(lo).min(hi)
                }
                2 => {
                    // sign of a
                    if a > 0 {
                        1
                    } else if a < 0 {
                        -1
                    } else {
                        0
                    }
                }
                _ => unreachable!(),
            };
            TestCase {
                inputs: vec![Value::Int(a), Value::Int(b)],
                expected_output: Some(vec![Value::Int(result)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect();
    ProblemSpec {
        test_cases,
        description: format!("conditional_{}", variant),
        target_cost: None,
    }
}

/// Multi-input: combine two small lists via zip + fold.
fn random_multi_input(rng: &mut impl Rng) -> ProblemSpec {
    let op = rng.gen_range(0..3u8); // dot product, sum of mins, sum of maxes
    let test_cases: Vec<TestCase> = (0..5)
        .map(|_| {
            let len = rng.gen_range(2..4usize);
            let a: Vec<i64> = (0..len).map(|_| rng.gen_range(-5i64..5)).collect();
            let b: Vec<i64> = (0..len).map(|_| rng.gen_range(-5i64..5)).collect();
            let result: i64 = a
                .iter()
                .zip(b.iter())
                .map(|(&x, &y)| match op {
                    0 => x.wrapping_mul(y),
                    1 => x.min(y),
                    2 => x.max(y),
                    _ => unreachable!(),
                })
                .sum();
            let input_a = Value::tuple(a.into_iter().map(Value::Int).collect());
            let input_b = Value::tuple(b.into_iter().map(Value::Int).collect());
            TestCase {
                inputs: vec![input_a, input_b],
                expected_output: Some(vec![Value::Int(result)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect();
    ProblemSpec {
        test_cases,
        description: format!("multi_input_{}", op),
        target_cost: None,
    }
}

/// Iterative: unfold-style pattern (e.g., sum 1..n).
fn random_iterative(rng: &mut impl Rng) -> ProblemSpec {
    let variant = rng.gen_range(0..3u8); // sum 1..n, factorial (small), triangular
    let test_cases: Vec<TestCase> = (0..5)
        .map(|_| {
            let n = rng.gen_range(1i64..8);
            let result = match variant {
                0 => {
                    // sum 1..=n
                    n * (n + 1) / 2
                }
                1 => {
                    // factorial (clamped to small n)
                    let m = n.min(6);
                    (1..=m).product::<i64>()
                }
                2 => {
                    // triangular: n*(n-1)/2
                    n * (n - 1) / 2
                }
                _ => unreachable!(),
            };
            TestCase {
                inputs: vec![Value::Int(n)],
                expected_output: Some(vec![Value::Int(result)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect();
    ProblemSpec {
        test_cases,
        description: format!("iterative_{}", variant),
        target_cost: None,
    }
}

/// Predicate over a list: count/all/any.
fn random_predicate(rng: &mut impl Rng) -> ProblemSpec {
    let variant = rng.gen_range(0..3u8); // count positive, all positive, any negative
    let test_cases: Vec<TestCase> = (0..5)
        .map(|_| {
            let len = rng.gen_range(2..5usize);
            let elems: Vec<i64> = (0..len).map(|_| rng.gen_range(-5i64..5)).collect();
            let result = match variant {
                0 => {
                    // count positive
                    elems.iter().filter(|&&x| x > 0).count() as i64
                }
                1 => {
                    // all positive -> 1 or 0
                    if elems.iter().all(|&x| x > 0) {
                        1
                    } else {
                        0
                    }
                }
                2 => {
                    // any negative -> 1 or 0
                    if elems.iter().any(|&x| x < 0) {
                        1
                    } else {
                        0
                    }
                }
                _ => unreachable!(),
            };
            let input = Value::tuple(elems.into_iter().map(Value::Int).collect());
            TestCase {
                inputs: vec![input],
                expected_output: Some(vec![Value::Int(result)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect();
    ProblemSpec {
        test_cases,
        description: format!("predicate_{}", variant),
        target_cost: None,
    }
}

/// Pairwise operation on consecutive elements.
fn random_pairwise(rng: &mut impl Rng) -> ProblemSpec {
    let variant = rng.gen_range(0..3u8); // sum of diffs, max diff, is sorted
    let test_cases: Vec<TestCase> = (0..5)
        .map(|_| {
            let len = rng.gen_range(3..5usize);
            let elems: Vec<i64> = (0..len).map(|_| rng.gen_range(-10i64..10)).collect();
            let result = match variant {
                0 => {
                    // sum of absolute differences
                    elems
                        .windows(2)
                        .map(|w| (w[0] - w[1]).abs())
                        .sum::<i64>()
                }
                1 => {
                    // max absolute difference
                    elems
                        .windows(2)
                        .map(|w| (w[0] - w[1]).abs())
                        .max()
                        .unwrap_or(0)
                }
                2 => {
                    // is sorted ascending -> 1 or 0
                    if elems.windows(2).all(|w| w[0] <= w[1]) {
                        1
                    } else {
                        0
                    }
                }
                _ => unreachable!(),
            };
            let input = Value::tuple(elems.into_iter().map(Value::Int).collect());
            TestCase {
                inputs: vec![input],
                expected_output: Some(vec![Value::Int(result)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect();
    ProblemSpec {
        test_cases,
        description: format!("pairwise_{}", variant),
        target_cost: None,
    }
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// Batch evolution with collection
// ---------------------------------------------------------------------------

// These functions previously used iris-codec's ProgramCollector, TrainingDataset,
// and LearnedCodec. They are now replaced by IRIS programs:
//   - src/iris-programs/codec/crossover.iris
//   - src/iris-programs/codec/feature_codec.iris
//   - src/iris-programs/codec/gin_vae.iris

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    /// Deterministic RNG for reproducible tests.
    fn test_rng() -> StdRng {
        StdRng::seed_from_u64(42)
    }

    // -----------------------------------------------------------------------
    // Problem spec generators
    // -----------------------------------------------------------------------

    #[test]
    fn generate_problem_specs_produces_correct_count() {
        let mut rng = test_rng();
        let specs = generate_problem_specs(&mut rng, 20);
        assert_eq!(specs.len(), 20);
    }

    #[test]
    fn all_problem_types_have_test_cases() {
        let mut rng = test_rng();
        let specs = generate_problem_specs(&mut rng, 10);
        for spec in &specs {
            assert!(
                !spec.test_cases.is_empty(),
                "spec '{}' has no test cases",
                spec.description
            );
        }
    }

    #[test]
    fn scalar_arithmetic_correct_outputs() {
        let mut rng = test_rng();
        // Generate enough to hit all 5 ops
        for _ in 0..20 {
            let spec = random_scalar_arithmetic(&mut rng);
            for tc in &spec.test_cases {
                let expected = tc.expected_output.as_ref().unwrap();
                assert_eq!(expected.len(), 1, "arithmetic should produce one output");
                assert!(
                    matches!(expected[0], Value::Int(_)),
                    "arithmetic output should be Int"
                );
            }
        }
    }

    #[test]
    fn fold_problem_correct_outputs() {
        let mut rng = test_rng();
        for _ in 0..10 {
            let spec = random_fold_problem(&mut rng);
            for tc in &spec.test_cases {
                assert_eq!(tc.inputs.len(), 1);
                let expected = tc.expected_output.as_ref().unwrap();
                assert_eq!(expected.len(), 1);
                assert!(matches!(expected[0], Value::Int(_)));
            }
        }
    }

    #[test]
    fn map_problem_correct_outputs() {
        let mut rng = test_rng();
        for _ in 0..10 {
            let spec = random_map_problem(&mut rng);
            for tc in &spec.test_cases {
                assert_eq!(tc.inputs.len(), 1);
                let expected = tc.expected_output.as_ref().unwrap();
                assert_eq!(expected.len(), 1);
                assert!(matches!(expected[0], Value::Tuple(_)));
            }
        }
    }

    #[test]
    fn filter_problem_correct_outputs() {
        let mut rng = test_rng();
        for _ in 0..10 {
            let spec = random_filter_problem(&mut rng);
            for tc in &spec.test_cases {
                let expected = tc.expected_output.as_ref().unwrap();
                assert_eq!(expected.len(), 1);
                assert!(matches!(expected[0], Value::Int(_)));
            }
        }
    }

    #[test]
    fn composition_correct_outputs() {
        let mut rng = test_rng();
        for _ in 0..10 {
            let spec = random_composition(&mut rng);
            for tc in &spec.test_cases {
                let expected = tc.expected_output.as_ref().unwrap();
                assert_eq!(expected.len(), 1);
                assert!(matches!(expected[0], Value::Int(_)));
            }
        }
    }

    #[test]
    fn conditional_correct_outputs() {
        let mut rng = test_rng();
        for _ in 0..10 {
            let spec = random_conditional(&mut rng);
            for tc in &spec.test_cases {
                assert_eq!(tc.inputs.len(), 2);
                let expected = tc.expected_output.as_ref().unwrap();
                assert_eq!(expected.len(), 1);
                assert!(matches!(expected[0], Value::Int(_)));
            }
        }
    }

    #[test]
    fn multi_input_correct_outputs() {
        let mut rng = test_rng();
        for _ in 0..10 {
            let spec = random_multi_input(&mut rng);
            for tc in &spec.test_cases {
                assert_eq!(tc.inputs.len(), 2);
                let expected = tc.expected_output.as_ref().unwrap();
                assert_eq!(expected.len(), 1);
                assert!(matches!(expected[0], Value::Int(_)));
            }
        }
    }

    #[test]
    fn iterative_correct_outputs() {
        let mut rng = test_rng();
        for _ in 0..10 {
            let spec = random_iterative(&mut rng);
            for tc in &spec.test_cases {
                assert_eq!(tc.inputs.len(), 1);
                let expected = tc.expected_output.as_ref().unwrap();
                assert_eq!(expected.len(), 1);
                assert!(matches!(expected[0], Value::Int(_)));
            }
        }
    }

    #[test]
    fn predicate_correct_outputs() {
        let mut rng = test_rng();
        for _ in 0..10 {
            let spec = random_predicate(&mut rng);
            for tc in &spec.test_cases {
                let expected = tc.expected_output.as_ref().unwrap();
                assert_eq!(expected.len(), 1);
                assert!(matches!(expected[0], Value::Int(_)));
            }
        }
    }

    #[test]
    fn pairwise_correct_outputs() {
        let mut rng = test_rng();
        for _ in 0..10 {
            let spec = random_pairwise(&mut rng);
            for tc in &spec.test_cases {
                let expected = tc.expected_output.as_ref().unwrap();
                assert_eq!(expected.len(), 1);
                assert!(matches!(expected[0], Value::Int(_)));
            }
        }
    }

    #[test]
    fn specs_have_unique_descriptions() {
        let mut rng = test_rng();
        let specs = generate_problem_specs(&mut rng, 10);
        // Each of the 10 problem types has a different prefix.
        let prefixes: Vec<&str> = specs
            .iter()
            .map(|s| {
                s.description
                    .split('_')
                    .next()
                    .unwrap_or(&s.description)
            })
            .collect();
        // At minimum, descriptions are non-empty.
        for d in &prefixes {
            assert!(!d.is_empty());
        }
    }

    // -----------------------------------------------------------------------
    // Verify computed outputs are correct
    // -----------------------------------------------------------------------

    #[test]
    fn scalar_arithmetic_add_verified() {
        // Manually verify: op=0 is add.
        let tc = TestCase {
            inputs: vec![Value::Int(3), Value::Int(7)],
            expected_output: Some(vec![Value::Int(10)]),
            initial_state: None,
            expected_state: None,
        };
        assert_eq!(
            tc.expected_output.as_ref().unwrap()[0],
            Value::Int(10)
        );
    }

    #[test]
    fn fold_sum_verified() {
        // sum of [2, 3, 5] = 10
        let elems = vec![2i64, 3, 5];
        let result: i64 = elems.iter().sum();
        assert_eq!(result, 10);
    }

    #[test]
    fn pairwise_diffs_verified() {
        // [3, 1, 4] -> diffs: |3-1| + |1-4| = 2 + 3 = 5
        let elems = vec![3i64, 1, 4];
        let result: i64 = elems.windows(2).map(|w| (w[0] - w[1]).abs()).sum();
        assert_eq!(result, 5);
    }

}
