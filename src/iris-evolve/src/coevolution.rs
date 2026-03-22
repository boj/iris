//! Competitive coevolution of programs and test cases.
//!
//! Evolves test cases alongside programs in an arms race (Digital Red Queen).
//! Test cases that break programs are rewarded; programs that pass tests are
//! rewarded. This prevents stagnation by continuously raising the difficulty
//! bar.
//!
//! Phase Transition 2: Individual -> Ecology.

use rand::Rng;
use std::rc::Rc;
use iris_exec::ExecutionService;
use iris_types::eval::{EvalTier, TestCase, Value};

use crate::config::EvolutionConfig;
use crate::individual::Fitness;
use crate::population::Deme;

// ---------------------------------------------------------------------------
// TestIndividual
// ---------------------------------------------------------------------------

/// An individual in the co-evolved test population.
#[derive(Debug, Clone)]
pub struct TestIndividual {
    /// The test case (inputs + expected output).
    pub test_case: TestCase,
    /// Fitness: fraction of programs this test fails (higher = harder test).
    pub fitness: f32,
}

// ---------------------------------------------------------------------------
// CoevolutionEngine
// ---------------------------------------------------------------------------

/// Engine for competitive coevolution of programs and test cases.
///
/// Programs are rewarded for passing tests. Tests are rewarded for failing
/// programs. This creates an arms race that prevents fitness stagnation.
pub struct CoevolutionEngine {
    /// The program population (evolved via standard IRIS mechanisms).
    pub program_deme: Deme,
    /// The test case population (evolved via mutation of inputs/outputs).
    pub test_population: Vec<TestIndividual>,
    /// Generation counter.
    pub generation: usize,
}

impl CoevolutionEngine {
    /// Create a new coevolution engine.
    ///
    /// - `program_deme`: initial program population
    /// - `seed_tests`: initial test cases to seed the test population
    pub fn new(program_deme: Deme, seed_tests: Vec<TestCase>) -> Self {
        let test_population = seed_tests
            .into_iter()
            .map(|tc| TestIndividual {
                test_case: tc,
                fitness: 0.0,
            })
            .collect();

        Self {
            program_deme,
            test_population,
            generation: 0,
        }
    }

    /// Run one coevolutionary step.
    ///
    /// 1. Evaluate all programs on all test cases.
    /// 2. Program fitness = fraction of tests passed.
    /// 3. Test fitness = fraction of programs failed.
    /// 4. Select + reproduce programs (via Deme.step with current tests).
    /// 5. Select + reproduce tests (tournament on test fitness).
    /// 6. Mutate tests.
    pub fn step(
        &mut self,
        exec: &dyn ExecutionService,
        config: &EvolutionConfig,
        rng: &mut impl Rng,
    ) {
        let num_programs = self.program_deme.individuals.len();
        let num_tests = self.test_population.len();

        if num_programs == 0 || num_tests == 0 {
            return;
        }

        // 1. Evaluate all programs on all test cases (cross-evaluation).
        // results[prog_idx][test_idx] = true if program passed that test.
        let test_cases: Vec<TestCase> = self
            .test_population
            .iter()
            .map(|t| t.test_case.clone())
            .collect();

        let graphs: Vec<_> = self
            .program_deme
            .individuals
            .iter()
            .map(|ind| ind.fragment.graph.clone())
            .collect();

        // Evaluate each program against all tests.
        let mut pass_matrix = vec![vec![false; num_tests]; num_programs];

        for (prog_idx, graph) in graphs.iter().enumerate() {
            match exec.evaluate_individual(graph, &test_cases, EvalTier::A) {
                Ok(result) => {
                    // Per-case scores tell us which tests passed.
                    for (test_idx, &score) in result.per_case_scores.iter().enumerate() {
                        if test_idx < num_tests {
                            pass_matrix[prog_idx][test_idx] = score >= 1.0;
                        }
                    }
                }
                Err(_) => {
                    // Program failed entirely: all tests "pass" (fail the program).
                }
            }
        }

        // 2. Program fitness = fraction of tests passed (clamped to [0, 1]).
        for (prog_idx, ind) in self.program_deme.individuals.iter_mut().enumerate() {
            let passed = pass_matrix[prog_idx].iter().filter(|&&p| p).count();
            let frac = (passed as f32 / num_tests as f32).clamp(0.0, 1.0);
            ind.fitness = Fitness {
                values: [frac, 0.5, 0.5, 0.5, 0.0],
            };
            // Per-case scores for lexicase.
            ind.per_case_scores = pass_matrix[prog_idx]
                .iter()
                .map(|&p| if p { 1.0 } else { 0.0 })
                .collect();
        }

        // 3. Test fitness = fraction of programs failed (clamped to [0, 1]).
        for (test_idx, test_ind) in self.test_population.iter_mut().enumerate() {
            let failed = pass_matrix
                .iter()
                .filter(|row| !row[test_idx])
                .count();
            test_ind.fitness = (failed as f32 / num_programs as f32).clamp(0.0, 1.0);
        }

        // 4. Evolve programs using the Deme step (which does NSGA-II ranking,
        //    lexicase selection, crossover, mutation).
        self.program_deme
            .step(exec, &test_cases, config, rng);

        // 5. Evolve test cases: tournament selection + mutation.
        let new_tests = self.reproduce_tests(rng);
        self.test_population = new_tests;

        self.generation += 1;
    }

    /// Tournament selection + mutation for the test population.
    fn reproduce_tests(&self, rng: &mut impl Rng) -> Vec<TestIndividual> {
        let n = self.test_population.len();
        if n == 0 {
            return vec![];
        }

        // Keep best 25% as elites.
        let mut sorted_indices: Vec<usize> = (0..n).collect();
        sorted_indices.sort_by(|&a, &b| {
            self.test_population[b]
                .fitness
                .partial_cmp(&self.test_population[a].fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let num_elites = (n / 4).max(1);
        let mut next_gen: Vec<TestIndividual> = sorted_indices
            .iter()
            .take(num_elites)
            .map(|&i| self.test_population[i].clone())
            .collect();

        // Fill the rest with tournament-selected + mutated tests.
        while next_gen.len() < n {
            let parent_idx = self.tournament_select_test(3, rng);
            let parent = &self.test_population[parent_idx];
            let child_test = mutate_test_case(&parent.test_case, rng);
            next_gen.push(TestIndividual {
                test_case: child_test,
                fitness: 0.0,
            });
        }

        next_gen.truncate(n);
        next_gen
    }

    /// Tournament selection on test fitness.
    fn tournament_select_test(&self, k: usize, rng: &mut impl Rng) -> usize {
        let n = self.test_population.len();
        let k = k.min(n);
        let mut best = rng.gen_range(0..n);
        for _ in 1..k {
            let candidate = rng.gen_range(0..n);
            if self.test_population[candidate].fitness > self.test_population[best].fitness {
                best = candidate;
            }
        }
        best
    }

    /// Average test fitness (fraction of programs failed).
    pub fn avg_test_fitness(&self) -> f32 {
        if self.test_population.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.test_population.iter().map(|t| t.fitness).sum();
        sum / self.test_population.len() as f32
    }

    /// Best program fitness (fraction of tests passed).
    pub fn best_program_fitness(&self) -> f32 {
        self.program_deme
            .best_individual()
            .map(|ind| ind.fitness.correctness())
            .unwrap_or(0.0)
    }
}

// ---------------------------------------------------------------------------
// Test case mutation operators
// ---------------------------------------------------------------------------

/// Mutate a test case by perturbing inputs or changing expected outputs.
pub fn mutate_test_case(tc: &TestCase, rng: &mut impl Rng) -> TestCase {
    let op = rng.gen_range(0..4u8);
    match op {
        0 => perturb_int_input(tc, rng),
        1 => add_or_remove_list_element(tc, rng),
        2 => change_expected_output(tc, rng),
        3 => generate_random_test_case(tc, rng),
        _ => tc.clone(),
    }
}

/// Perturb an integer input by a small delta.
fn perturb_int_input(tc: &TestCase, rng: &mut impl Rng) -> TestCase {
    let mut new_tc = tc.clone();
    if new_tc.inputs.is_empty() {
        return new_tc;
    }

    let idx = rng.gen_range(0..new_tc.inputs.len());
    if let Value::Int(ref mut v) = new_tc.inputs[idx] {
        let delta_choice = rng.gen_range(0..3u8);
        let delta: i64 = match delta_choice {
            0 => {
                if rng.r#gen::<bool>() { 1 } else { -1 }
            }
            1 => {
                if rng.r#gen::<bool>() { 10 } else { -10 }
            }
            _ => {
                if rng.r#gen::<bool>() { 100 } else { -100 }
            }
        };
        *v = v.saturating_add(delta);
        // Clear expected output since input changed.
        new_tc.expected_output = None;
    }

    new_tc
}

/// Add or remove an element from a tuple/list input.
fn add_or_remove_list_element(tc: &TestCase, rng: &mut impl Rng) -> TestCase {
    let mut new_tc = tc.clone();
    if new_tc.inputs.is_empty() {
        return new_tc;
    }

    let idx = rng.gen_range(0..new_tc.inputs.len());
    if let Value::Tuple(ref mut elems) = new_tc.inputs[idx] {
        let elems = Rc::make_mut(elems);
        if !elems.is_empty() && rng.r#gen::<bool>() {
            // Remove a random element.
            let remove_idx = rng.gen_range(0..elems.len());
            elems.remove(remove_idx);
        } else {
            // Add a random integer element.
            let val = Value::Int(rng.gen_range(-100..=100));
            elems.push(val);
        }
        new_tc.expected_output = None;
    }

    new_tc
}

/// Change the expected output (clear it, forcing re-evaluation).
fn change_expected_output(tc: &TestCase, rng: &mut impl Rng) -> TestCase {
    let mut new_tc = tc.clone();
    // Randomly either clear expected output or perturb it.
    if let Some(ref mut expected) = new_tc.expected_output {
        if !expected.is_empty() {
            let idx = rng.gen_range(0..expected.len());
            if let Value::Int(ref mut v) = expected[idx] {
                let delta: i64 = if rng.r#gen::<bool>() { 1 } else { -1 };
                *v = v.saturating_add(delta);
            }
        }
    } else {
        // No expected output; leave as-is.
    }
    new_tc
}

/// Generate a completely new random test case based on the template.
fn generate_random_test_case(template: &TestCase, rng: &mut impl Rng) -> TestCase {
    let inputs: Vec<Value> = template
        .inputs
        .iter()
        .map(|v| match v {
            Value::Int(_) => Value::Int(rng.gen_range(-1000..=1000)),
            Value::Tuple(elems) => {
                let len = rng.gen_range(1..=elems.len().max(3));
                let new_elems: Vec<Value> = (0..len)
                    .map(|_| Value::Int(rng.gen_range(-100..=100)))
                    .collect();
                Value::Tuple(Rc::new(new_elems))
            }
            Value::Bool(_) => Value::Bool(rng.r#gen::<bool>()),
            other => other.clone(),
        })
        .collect();

    TestCase {
        inputs,
        expected_output: None,
        initial_state: None,
        expected_state: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_mutate_test_case_does_not_panic() {
        let tc = TestCase {
            inputs: vec![Value::Int(42), Value::Int(10)],
            expected_output: Some(vec![Value::Int(52)]),
            initial_state: None,
            expected_state: None,
        };

        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..100 {
            let _ = mutate_test_case(&tc, &mut rng);
        }
    }

    #[test]
    fn test_mutate_test_case_with_tuple() {
        let tc = TestCase {
            inputs: vec![Value::Tuple(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
            expected_output: Some(vec![Value::Int(6)]),
            initial_state: None,
            expected_state: None,
        };

        let mut rng = StdRng::seed_from_u64(123);
        let mut lengths_seen = std::collections::HashSet::new();
        for _ in 0..200 {
            let mutated = mutate_test_case(&tc, &mut rng);
            if let Some(Value::Tuple(elems)) = mutated.inputs.first() {
                lengths_seen.insert(elems.len());
            }
        }
        // Should see at least 2 different tuple lengths.
        assert!(
            lengths_seen.len() >= 2,
            "tuple mutation should produce varied lengths, got {:?}",
            lengths_seen
        );
    }

    #[test]
    fn test_perturb_int_produces_different_values() {
        let tc = TestCase {
            inputs: vec![Value::Int(50)],
            expected_output: Some(vec![Value::Int(50)]),
            initial_state: None,
            expected_state: None,
        };

        let mut rng = StdRng::seed_from_u64(99);
        let mut values_seen = std::collections::HashSet::new();
        for _ in 0..100 {
            let mutated = perturb_int_input(&tc, &mut rng);
            if let Value::Int(v) = mutated.inputs[0] {
                values_seen.insert(v);
            }
        }
        assert!(
            values_seen.len() >= 3,
            "perturb should produce varied values, got {:?}",
            values_seen
        );
    }
}
