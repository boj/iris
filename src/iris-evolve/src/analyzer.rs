//! Analytic decomposition of test cases for pattern detection and skeleton
//! construction.
//!
//! Before evolution starts, this module examines input-output relationships
//! across test cases to determine what kind of program is needed, then builds
//! a near-correct program skeleton. Evolution can then refine constants and
//! handle edge cases rather than searching from random seeds.

use std::collections::{BTreeMap, HashMap};

use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::{TestCase, Value};
use iris_types::fragment::{Boundary, Fragment, FragmentId, FragmentMeta};
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::{compute_fragment_id, compute_node_id, SemanticHash};
use iris_types::types::{PrimType, TypeDef, TypeEnv, TypeId};

use std::sync::atomic::{AtomicU32, Ordering};
static ANALYZER_COUNTER: AtomicU32 = AtomicU32::new(0);

// ---------------------------------------------------------------------------
// DetectedPattern
// ---------------------------------------------------------------------------

/// A pattern detected from analyzing test case input-output relationships.
#[derive(Debug, Clone, PartialEq)]
pub enum DetectedPattern {
    /// Output is always the same regardless of input.
    Constant(Value),

    /// Output equals one of the inputs.
    Identity { input_index: usize },

    /// Output is a simple arithmetic function of two inputs.
    /// op: 0x00=add, 0x01=sub, 0x02=mul, 0x03=div, 0x04=mod, 0x07=min, 0x08=max
    BinaryArithmetic {
        op: u8,
        left_input: usize,
        right_input: usize,
    },

    /// Output depends on which value a dispatch input takes.
    ConditionalDispatch {
        dispatch_input: usize,
        branches: Vec<(i64, DetectedPattern)>,
    },

    /// Output is a fold over a list input.
    /// op: 0x00=add, 0x02=mul, 0x07=min, 0x08=max
    FoldReduction {
        base: i64,
        op: u8,
        input_index: usize,
    },

    /// Output is a map over a list input.
    /// op: the opcode applied element-wise (binary op applied as f(x, x)).
    MapTransform {
        op: u8,
        input_index: usize,
    },

    /// Output is a composition: map then fold.
    MapFoldComposition {
        map_op: u8,
        fold_op: u8,
        fold_base: i64,
        input_index: usize,
    },

    /// Output is clamp(input, lo, hi) = max(lo, min(hi, input)).
    Clamp {
        lo: i64,
        hi: i64,
    },

    /// Output is factorial of input: n! = product(1..=n).
    Factorial,

    /// Output is power: input[0]^input[1].
    Power,

    /// Output is zip + map(op) + fold(base, fold_op) over two list inputs.
    /// E.g., manhattan distance = zip + map(abs(sub)) + fold(0, add).
    ZipMapFold {
        pair_op: u8,
        map_unary: Option<u8>,
        fold_op: u8,
        fold_base: i64,
    },

    /// Output is polynomial evaluation: sum(coeff[i] * x^i).
    Polynomial,

    /// Output is the index of a target value in a list, or -1 if not found.
    LinearSearch,

    /// Output is the nth Fibonacci number.
    Fibonacci,

    /// Output is op(fold1(list), fold2(list)) — two folds combined.
    /// E.g., average = div(fold(0,add,list), fold(0,count,list))
    TwoFoldScalar {
        fold1_base: i64,
        fold1_op: u8,
        fold2_base: i64,
        fold2_op: u8,  // 0xFE = count (len)
        combine_op: u8,
        input_index: usize,
    },

    /// Output is map(|x| op(fold_val, x), list) — fold then map.
    /// E.g., distance_from_max = map(|x| max - x, list) where max = fold(MIN, max, list)
    FoldThenMap {
        fold_base: i64,
        fold_op: u8,
        map_op: u8,
        /// true = fold_val op x, false = x op fold_val
        fold_first: bool,
        input_index: usize,
    },

    /// Output is map(|x| x op arg, list) where arg is a separate input.
    /// E.g., add_constant(list, c) = map(|x| x + c, list)
    MapWithArg {
        op: u8,
        list_input: usize,
        arg_input: usize,
    },

    /// Output is count of elements satisfying element cmp_op threshold.
    /// E.g., count_greater_than(list, t) = count(x > t for x in list)
    ConditionalCount {
        cmp_op: u8,
        list_input: usize,
        threshold_input: usize,
    },

    /// Output is zip_map_fold(a, b) + bias — dot product plus bias.
    ZipMapFoldPlusBias {
        pair_op: u8,
        fold_op: u8,
        fold_base: i64,
        bias_input: usize,
    },

    /// Output is fold(base, op, take(list, n)) — fold over first n elements.
    FoldAfterTake {
        fold_base: i64,
        fold_op: u8,
        list_input: usize,
        count_input: usize,
    },

    /// Output is a scan (prefix sums/products).
    /// E.g., cumulative_sum = scan(0, add, list)
    Scan {
        base: i64,
        op: u8,
        input_index: usize,
    },

    /// Output is binary_to_decimal: fold(|acc, bit| acc*2 + bit, 0, bits)
    BinaryToDecimal,

    /// Output is digit_sum: sum of digits via unfold(n, mod10/div10) + fold(add)
    DigitSum,

    /// Output is count_divisors: count of i in 1..=n where n%i==0
    CountDivisors,

    /// Output is is_prime: 1 if count_divisors==2, else 0
    IsPrime,

    /// Output is variance_numerator: sum((x - mean)^2) where mean = sum/len.
    VarianceNumerator { input_index: usize },

    /// Output is collatz_step: if x%2==0 then x/2 else 3*x+1.
    CollatzStep,

    /// Output is a*input + b for constants a, b (single scalar input).
    /// E.g., apply_twice(f=+3) = x + 6 => a=1, b=6.
    UnaryAffine { a: i64, b: i64 },

    /// Output is fold(base, op, list) then mod constant.
    /// E.g., checksum = sum(list) mod 256.
    FoldThenMod {
        fold_base: i64,
        fold_op: u8,
        mod_val: i64,
        input_index: usize,
    },

    /// Output is concat(list, [element]) — append element to list.
    ConcatElement {
        list_input: usize,
        elem_input: usize,
    },

    /// Output is take(list, len(list)-1) — all but last element.
    InitList { input_index: usize },

    /// Output is [x, x, ..., x] (n times) — repeat element.
    Repeat {
        elem_input: usize,
        count_input: usize,
    },

    /// Output is [0, 1, 2, ..., n-1] — range.
    Range,

    /// Output is a linear combination of elements at fixed indices.
    /// ops: list of (coeff, index) — output = sum(coeff * input[index]).
    /// E.g., mat_trace: [(1, 0), (1, 3)] => input[0] + input[3].
    ElementAccess {
        /// Expressions: (coefficient, input_element_index)
        terms: Vec<(i64, usize)>,
        input_index: usize,
    },

    /// Output is a*d - b*c for a 4-element list [a,b,c,d].
    MatDet2x2 { input_index: usize },

    /// Output is [a*x+b*y, c*x+d*y] for matrix [a,b,c,d] and vector [x,y].
    MatVecMul2x2,

    /// State operation: state[key] += constant (no output, just state change).
    StateIncrement {
        key: String,
        delta: i64,
    },

    /// State operation: state[key] += input, output = new state[key].
    StateAccumulate {
        key: String,
        input_index: usize,
    },

    /// Output is sum of top k elements from a list.
    TopKSum {
        k: usize,
        input_index: usize,
    },

    /// Output is pairwise average of consecutive elements.
    /// [a,b,c,d] -> [(a+b)/2, (b+c)/2, (c+d)/2]
    MovingAvg2 { input_index: usize },

    /// Output is 1 if list A starts with prefix B, 0 otherwise.
    StartsWith,

    /// Output is 1 if list contains target value, 0 otherwise.
    /// E.g., has_fold([0,1,8,3]) → 1 (contains 8)
    Contains {
        target: i64,
        list_input: usize,
    },

    /// Output is map(|x| if x==old then new else x, list) — conditional map.
    /// E.g., replace_opcode([1,2,1,3], 1, 9) → [9,2,9,3]
    ConditionalMap {
        list_input: usize,
        old_input: usize,
        new_input: usize,
    },

    /// Output is the count of distinct values in a list.
    /// E.g., count_unique([1,2,1,3,1]) → 3
    CountUnique { input_index: usize },

    /// Output is the longest consecutive run of non-zero values.
    /// E.g., program_depth([1,2,0,3,4,5,0]) → 3
    ProgramDepth { input_index: usize },

    /// Output is the mode (most frequent element) of a list.
    /// E.g., most_common([1,2,1,3,1]) → 1
    MostCommon { input_index: usize },

    /// Output is the input list with a value inserted at a given index.
    /// E.g., insert_at([1,2,3], 1, 99) → [1,99,2,3]
    InsertAt {
        list_input: usize,
        idx_input: usize,
        val_input: usize,
    },

    /// Output is the input list with the element at a given index removed.
    /// E.g., delete_at([1,2,3,4], 2) → [1,2,4]
    DeleteAt {
        list_input: usize,
        idx_input: usize,
    },

    /// Output is the input list with elements at two positions swapped.
    /// E.g., swap_elements([1,2,3,4], 1, 3) → [1,4,3,2]
    SwapElements {
        list_input: usize,
        i_input: usize,
        j_input: usize,
    },

    /// Output is two lists interleaved element-by-element.
    /// E.g., interleave([1,2,3], [4,5,6]) → [1,4,2,5,3,6]
    Interleave {
        a_input: usize,
        b_input: usize,
    },

    /// Output is each element of the list duplicated.
    /// E.g., double_elements([1,2,3]) → [1,1,2,2,3,3]
    DoubleElements { input_index: usize },

    /// Output is the list with each element paired with its index (flattened).
    /// E.g., zip_with_index([10,20,30]) → [0,10,1,20,2,30]
    ZipWithIndex { input_index: usize },

    /// Output is a binary op applied to two elements extracted from a list.
    /// E.g., eval_rpn([a, b, 1]) → a - b  ⟹  op=0x01, i=0, j=1
    ListElementBinaryOp {
        op: u8,
        i: usize,
        j: usize,
        input_index: usize,
    },

    /// Count occurrences of a value at even (stride-2) positions in a flat list.
    /// E.g., out_degree([0,1,0,2,1,2], node=0) → 2
    StridedCountEq {
        list_input: usize,
        value_input: usize,
    },

    /// Check if a (src,tgt) pair exists at stride-2 positions in a flat edge list.
    /// E.g., has_edge([0,1,0,2,1,2], 0, 2) → 1
    StridedPairMatch {
        list_input: usize,
        src_input: usize,
        tgt_input: usize,
    },

    /// One pass of bubble sort: pairwise swap-if-unordered on consecutive pairs.
    /// E.g., [3,1,4,1,5] → [1,3,1,4,5]
    BubbleSortPass { input_index: usize },

    /// Decode run-length encoding: [val, count, val, count, ...] → expanded list.
    /// E.g., [1,2,2,3,3,1] → [1,1,2,2,2,3]
    DecodeRLE { input_index: usize },

    /// Encode run-length: consecutive equal elements → [val, count, ...].
    /// E.g., [1,1,2,2,2,3] → [1,2,2,3,3,1]
    EncodeRLE { input_index: usize },

    /// Could not detect a pattern.
    Unknown,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Analyze test cases to detect the input-output pattern.
pub fn analyze_test_cases(test_cases: &[TestCase]) -> DetectedPattern {
    let patterns = analyze_all_patterns(test_cases);
    patterns.into_iter().next().unwrap_or(DetectedPattern::Unknown)
}

/// Analyze test cases and return ALL detected patterns.
///
/// Multiple patterns may match (e.g., both FoldReduction and
/// MapFoldComposition). Returning all of them lets us inject multiple
/// skeletons, increasing the chance that at least one survives evolution.
pub fn analyze_all_patterns(test_cases: &[TestCase]) -> Vec<DetectedPattern> {
    if test_cases.is_empty() {
        return vec![];
    }

    let with_expected: Vec<&TestCase> = test_cases
        .iter()
        .filter(|tc| tc.expected_output.is_some())
        .collect();

    if with_expected.is_empty() {
        return vec![];
    }

    let mut patterns = Vec::new();

    // Try each detector. Collect ALL matches, not just the first.
    if let Some(pattern) = detect_constant(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_identity(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_clamp(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_binary_arithmetic(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_conditional_dispatch(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_factorial(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_power(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_fold(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_map(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_map_fold(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_zip_map_fold(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_polynomial(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_linear_search(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_fibonacci(&with_expected) {
        patterns.push(pattern);
    }
    // New detectors for harder algorithms
    if let Some(pattern) = detect_two_fold_scalar(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_fold_then_map(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_map_with_arg(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_conditional_count(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_zip_map_fold_plus_bias(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_fold_after_take(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_scan(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_binary_to_decimal(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_digit_sum(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_count_divisors(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_is_prime(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_variance_numerator(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_collatz_step(&with_expected) {
        patterns.push(pattern);
    }
    // --- New v3 detectors ---
    if let Some(pattern) = detect_unary_affine(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_fold_then_mod(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_concat_element(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_init_list(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_repeat(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_range(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_element_access(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_mat_det_2x2(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_mat_vec_mul_2x2(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_state_increment(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_state_accumulate(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_top_k_sum(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_moving_avg_2(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_starts_with(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_contains(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_conditional_map(&with_expected) {
        patterns.push(pattern);
    }
    // --- v5 detectors ---
    if let Some(pattern) = detect_count_unique(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_program_depth(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_most_common(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_insert_at(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_delete_at(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_swap_elements(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_interleave(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_double_elements(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_zip_with_index(&with_expected) {
        patterns.push(pattern);
    }
    // --- v7 detectors ---
    if let Some(pattern) = detect_list_element_binary_op(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_strided_count_eq(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_strided_pair_match(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_bubble_sort_pass(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_decode_rle(&with_expected) {
        patterns.push(pattern);
    }
    if let Some(pattern) = detect_encode_rle(&with_expected) {
        patterns.push(pattern);
    }

    patterns
}

/// Build a program skeleton from a detected pattern.
///
/// Returns `None` if the pattern is `Unknown` or cannot be translated
/// into a valid program graph.
pub fn build_skeleton(pattern: &DetectedPattern) -> Option<Fragment> {
    let (type_env, int_id) = int_type_env();
    let graph = build_skeleton_graph(pattern, &type_env, int_id)?;
    Some(graph_to_fragment(graph))
}

/// Build skeletons for ALL detected patterns.
pub fn build_all_skeletons(test_cases: &[TestCase]) -> Vec<Fragment> {
    let patterns = analyze_all_patterns(test_cases);
    patterns
        .iter()
        .filter_map(|p| build_skeleton(p))
        .collect()
}

// ---------------------------------------------------------------------------
// Helpers: extract scalar value
// ---------------------------------------------------------------------------

fn expected_output(tc: &TestCase) -> Option<&Value> {
    tc.expected_output.as_ref().and_then(|v| v.first())
}

fn as_int(v: &Value) -> Option<i64> {
    match v {
        Value::Int(n) => Some(*n),
        _ => None,
    }
}

fn as_int_list(v: &Value) -> Option<Vec<i64>> {
    match v {
        Value::Tuple(elems) => {
            let mut result = Vec::with_capacity(elems.len());
            for e in elems.iter() {
                result.push(as_int(e)?);
            }
            Some(result)
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Pattern detectors
// ---------------------------------------------------------------------------

fn detect_constant(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    let first_out = expected_output(test_cases[0])?;
    for tc in &test_cases[1..] {
        let out = expected_output(tc)?;
        if out != first_out {
            return None;
        }
    }
    Some(DetectedPattern::Constant(first_out.clone()))
}

fn detect_identity(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs == 0 {
        return None;
    }

    for i in 0..num_inputs {
        let all_match = test_cases.iter().all(|tc| {
            if let Some(out) = expected_output(tc) {
                tc.inputs.get(i).map_or(false, |inp| inp == out)
            } else {
                false
            }
        });
        if all_match {
            return Some(DetectedPattern::Identity { input_index: i });
        }
    }
    None
}

/// Try all binary arithmetic operations on all pairs of scalar inputs.
fn detect_binary_arithmetic(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 2 {
        return None;
    }

    let ops = arith_ops();

    for i in 0..num_inputs {
        for j in 0..num_inputs {
            if i == j {
                continue;
            }
            for &(opcode, op_fn) in ops {
                let all_match = test_cases.iter().all(|tc| {
                    let out = match expected_output(tc).and_then(as_int) {
                        Some(v) => v,
                        None => return false,
                    };
                    let a = match tc.inputs.get(i).and_then(as_int) {
                        Some(v) => v,
                        None => return false,
                    };
                    let b = match tc.inputs.get(j).and_then(as_int) {
                        Some(v) => v,
                        None => return false,
                    };
                    op_fn(a, b) == Some(out)
                });
                if all_match {
                    return Some(DetectedPattern::BinaryArithmetic {
                        op: opcode,
                        left_input: i,
                        right_input: j,
                    });
                }
            }
        }
    }
    None
}

/// Detect conditional dispatch: one input selects the operation applied
/// to the remaining inputs.
fn detect_conditional_dispatch(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 3 {
        return None;
    }

    for d in 0..num_inputs {
        let mut groups: BTreeMap<i64, Vec<&TestCase>> = BTreeMap::new();
        let mut all_int = true;

        for tc in test_cases {
            match tc.inputs.get(d).and_then(as_int) {
                Some(key) => groups.entry(key).or_default().push(tc),
                None => {
                    all_int = false;
                    break;
                }
            }
        }

        if !all_int || groups.len() < 2 {
            continue;
        }

        let mut branches = Vec::new();
        let mut all_detected = true;

        for (&key, group) in &groups {
            if let Some(sub_pattern) = detect_binary_arithmetic_remapped(group, d) {
                branches.push((key, sub_pattern));
            } else {
                all_detected = false;
                break;
            }
        }

        if all_detected && !branches.is_empty() {
            return Some(DetectedPattern::ConditionalDispatch {
                dispatch_input: d,
                branches,
            });
        }
    }
    None
}

/// Detect binary arithmetic on test cases with one input index excluded.
fn detect_binary_arithmetic_remapped(
    test_cases: &[&TestCase],
    exclude_index: usize,
) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 3 {
        return None;
    }

    let remapped: Vec<usize> = (0..num_inputs).filter(|&i| i != exclude_index).collect();
    if remapped.len() < 2 {
        return None;
    }

    let ops = arith_ops();

    for &ri in &remapped {
        for &rj in &remapped {
            if ri == rj {
                continue;
            }
            for &(opcode, op_fn) in ops {
                let all_match = test_cases.iter().all(|tc| {
                    let out = match expected_output(tc).and_then(as_int) {
                        Some(v) => v,
                        None => return false,
                    };
                    let a = match tc.inputs.get(ri).and_then(as_int) {
                        Some(v) => v,
                        None => return false,
                    };
                    let b = match tc.inputs.get(rj).and_then(as_int) {
                        Some(v) => v,
                        None => return false,
                    };
                    op_fn(a, b) == Some(out)
                });
                if all_match {
                    return Some(DetectedPattern::BinaryArithmetic {
                        op: opcode,
                        left_input: ri,
                        right_input: rj,
                    });
                }
            }
        }
    }
    None
}

/// Detect fold reduction: list input -> scalar output via fold.
fn detect_fold(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }

    let num_inputs = test_cases[0].inputs.len();

    let fold_ops: &[(u8, fn(i64, i64) -> i64, &[i64])] = &[
        (0x00, |a, b| a.wrapping_add(b), &[0]),
        (0x02, |a, b| a.wrapping_mul(b), &[1]),
        (0x07, |a, b| a.min(b), &[i64::MAX]),
        (0x08, |a, b| a.max(b), &[i64::MIN]),
        (0x12, |a, b| a ^ b, &[0]),
    ];

    for input_idx in 0..num_inputs {
        let first_input_is_list = matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_));
        let first_output_is_scalar = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Int(_)))
            .unwrap_or(false);

        if !first_input_is_list || !first_output_is_scalar {
            continue;
        }

        for &(opcode, op_fn, bases) in fold_ops {
            for &base in bases {
                let all_match = test_cases.iter().all(|tc| {
                    let out = match expected_output(tc).and_then(as_int) {
                        Some(v) => v,
                        None => return false,
                    };
                    let list = match as_int_list(&tc.inputs[input_idx]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let result = list.iter().fold(base, |acc, &x| op_fn(acc, x));
                    result == out
                });
                if all_match {
                    return Some(DetectedPattern::FoldReduction {
                        base,
                        op: opcode,
                        input_index: input_idx,
                    });
                }
            }
        }
    }
    None
}

/// Detect map transform: list -> list of same length.
fn detect_map(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }

    let num_inputs = test_cases[0].inputs.len();

    // Map ops using binary self-application f(x, x):
    // add(x,x)=2x, mul(x,x)=x^2
    // Or unary: neg(x)=-x, abs(x)=|x|
    let map_ops: &[(u8, fn(i64) -> Option<i64>)] = &[
        (0x00, |x| Some(x.wrapping_add(x))), // double
        (0x02, |x| Some(x.wrapping_mul(x))), // square
        (0x05, |x| Some(x.wrapping_neg())),  // negate
        (0x06, |x| Some(x.abs())),           // abs
    ];

    for input_idx in 0..num_inputs {
        for &(opcode, op_fn) in map_ops {
            let all_match = test_cases.iter().all(|tc| {
                let input_list = match as_int_list(&tc.inputs[input_idx]) {
                    Some(v) => v,
                    None => return false,
                };
                let output_list = match expected_output(tc).and_then(as_int_list) {
                    Some(v) => v,
                    None => return false,
                };
                if input_list.len() != output_list.len() {
                    return false;
                }
                input_list
                    .iter()
                    .zip(output_list.iter())
                    .all(|(&inp, &out)| op_fn(inp) == Some(out))
            });
            if all_match {
                return Some(DetectedPattern::MapTransform {
                    op: opcode,
                    input_index: input_idx,
                });
            }
        }
    }
    None
}

/// Detect map+fold composition: list -> scalar via map then fold.
fn detect_map_fold(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }

    let num_inputs = test_cases[0].inputs.len();

    let map_ops: &[(u8, fn(i64) -> i64)] = &[
        (0x00, |x| x.wrapping_add(x)), // double
        (0x02, |x| x.wrapping_mul(x)), // square
        (0x06, |x| x.abs()),           // abs
    ];

    let fold_ops: &[(u8, fn(i64, i64) -> i64, &[i64])] = &[
        (0x00, |a, b| a.wrapping_add(b), &[0]),
        (0x02, |a, b| a.wrapping_mul(b), &[1]),
        (0x07, |a, b| a.min(b), &[i64::MAX]),
        (0x08, |a, b| a.max(b), &[i64::MIN]),
    ];

    for input_idx in 0..num_inputs {
        let first_input_is_list = matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_));
        let first_output_is_scalar = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Int(_)))
            .unwrap_or(false);

        if !first_input_is_list || !first_output_is_scalar {
            continue;
        }

        for &(map_op, map_fn) in map_ops {
            for &(fold_op, fold_fn, bases) in fold_ops {
                for &base in bases {
                    let all_match = test_cases.iter().all(|tc| {
                        let out = match expected_output(tc).and_then(as_int) {
                            Some(v) => v,
                            None => return false,
                        };
                        let list = match as_int_list(&tc.inputs[input_idx]) {
                            Some(v) => v,
                            None => return false,
                        };
                        let mapped: Vec<i64> = list.iter().map(|&x| map_fn(x)).collect();
                        let result = mapped.iter().fold(base, |acc, &x| fold_fn(acc, x));
                        result == out
                    });
                    if all_match {
                        return Some(DetectedPattern::MapFoldComposition {
                            map_op,
                            fold_op,
                            fold_base: base,
                            input_index: input_idx,
                        });
                    }
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// New pattern detectors
// ---------------------------------------------------------------------------

/// Detect clamp: output = max(lo, min(hi, input)).
/// Checks if output is always clamped within a constant range.
fn detect_clamp(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 3 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 1 {
        return None;
    }

    // All inputs and outputs must be scalar ints.
    let mut pairs: Vec<(i64, i64)> = Vec::new();
    for tc in test_cases {
        let inp = as_int(&tc.inputs[0])?;
        let out = expected_output(tc).and_then(as_int)?;
        pairs.push((inp, out));
    }

    // Deduce lo and hi from the test cases.
    // Find cases where output != input to detect the bounds.
    let mut lo = i64::MIN;
    let mut hi = i64::MAX;

    for &(inp, out) in &pairs {
        if inp < out {
            // Input was below the lower bound, clamped up to `out`.
            lo = lo.max(out);
        } else if inp > out {
            // Input was above the upper bound, clamped down to `out`.
            hi = hi.min(out);
        }
    }

    // Validate: lo must be <= hi, and all test cases must match.
    if lo > hi || lo == i64::MIN || hi == i64::MAX {
        return None;
    }

    let all_match = pairs.iter().all(|&(inp, out)| {
        let expected = inp.max(lo).min(hi);
        expected == out
    });

    if all_match {
        Some(DetectedPattern::Clamp { lo, hi })
    } else {
        None
    }
}

/// Detect factorial: output = n! for single scalar input.
fn detect_factorial(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 3 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 1 {
        return None;
    }

    fn factorial(n: i64) -> Option<i64> {
        if n < 0 {
            return None;
        }
        let mut result: i64 = 1;
        for i in 2..=n {
            result = result.checked_mul(i)?;
        }
        Some(result)
    }

    let all_match = test_cases.iter().all(|tc| {
        let inp = match as_int(&tc.inputs[0]) {
            Some(v) => v,
            None => return false,
        };
        let out = match expected_output(tc).and_then(as_int) {
            Some(v) => v,
            None => return false,
        };
        factorial(inp) == Some(out)
    });

    if all_match {
        Some(DetectedPattern::Factorial)
    } else {
        None
    }
}

/// Detect power: output = input[0]^input[1].
fn detect_power(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 3 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 2 {
        return None;
    }

    fn int_pow(base: i64, exp: i64) -> Option<i64> {
        if exp < 0 {
            return None;
        }
        let mut result: i64 = 1;
        for _ in 0..exp {
            result = result.checked_mul(base)?;
        }
        Some(result)
    }

    let all_match = test_cases.iter().all(|tc| {
        let base = match as_int(&tc.inputs[0]) {
            Some(v) => v,
            None => return false,
        };
        let exp = match as_int(&tc.inputs[1]) {
            Some(v) => v,
            None => return false,
        };
        let out = match expected_output(tc).and_then(as_int) {
            Some(v) => v,
            None => return false,
        };
        int_pow(base, exp) == Some(out)
    });

    if all_match {
        Some(DetectedPattern::Power)
    } else {
        None
    }
}

/// Detect zip+map+fold over two list inputs.
/// E.g., manhattan = zip(a,b) -> map(|pair| abs(pair.0 - pair.1)) -> fold(0, add)
/// E.g., dot_product = zip(a,b) -> map(|pair| pair.0 * pair.1) -> fold(0, add)
/// E.g., weighted_sum = zip(a,b) -> map(|pair| pair.0 * pair.1) -> fold(0, add)
fn detect_zip_map_fold(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 2 {
        return None;
    }

    // Both inputs must be lists.
    let first_input0_is_list = matches!(&test_cases[0].inputs[0], Value::Tuple(_));
    let first_input1_is_list = matches!(&test_cases[0].inputs[1], Value::Tuple(_));
    let first_output_is_scalar = expected_output(test_cases[0])
        .map(|v| matches!(v, Value::Int(_)))
        .unwrap_or(false);

    if !first_input0_is_list || !first_input1_is_list || !first_output_is_scalar {
        return None;
    }

    // Try combinations of pair_op, optional unary map, fold_op.
    let pair_ops: &[(u8, fn(i64, i64) -> i64)] = &[
        (0x00, |a, b| a.wrapping_add(b)),
        (0x01, |a, b| a.wrapping_sub(b)),
        (0x02, |a, b| a.wrapping_mul(b)),
    ];
    let unary_ops: &[(Option<u8>, fn(i64) -> i64)] = &[
        (None, |x| x),
        (Some(0x06), |x| x.abs()),
    ];
    let fold_ops: &[(u8, fn(i64, i64) -> i64, i64)] = &[
        (0x00, |a, b| a.wrapping_add(b), 0),
        (0x02, |a, b| a.wrapping_mul(b), 1),
        (0x08, |a, b| a.max(b), i64::MIN),
        (0x07, |a, b| a.min(b), i64::MAX),
    ];

    for &(pair_op, pair_fn) in pair_ops {
        for &(map_unary, unary_fn) in unary_ops {
            for &(fold_op, fold_fn, fold_base) in fold_ops {
                let all_match = test_cases.iter().all(|tc| {
                    let list_a = match as_int_list(&tc.inputs[0]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let list_b = match as_int_list(&tc.inputs[1]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let out = match expected_output(tc).and_then(as_int) {
                        Some(v) => v,
                        None => return false,
                    };
                    if list_a.len() != list_b.len() {
                        return false;
                    }
                    let mapped: Vec<i64> = list_a.iter().zip(list_b.iter())
                        .map(|(&a, &b)| unary_fn(pair_fn(a, b)))
                        .collect();
                    let result = mapped.iter().fold(fold_base, |acc, &x| fold_fn(acc, x));
                    result == out
                });
                if all_match {
                    return Some(DetectedPattern::ZipMapFold {
                        pair_op,
                        map_unary,
                        fold_op,
                        fold_base,
                    });
                }
            }
        }
    }
    None
}

/// Detect polynomial evaluation: output = sum(coeff[i] * x^i).
/// Input: (coefficients_list, x_scalar).
fn detect_polynomial(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 2 {
        return None;
    }

    let first_input0_is_list = matches!(&test_cases[0].inputs[0], Value::Tuple(_));
    let first_input1_is_scalar = matches!(&test_cases[0].inputs[1], Value::Int(_));
    let first_output_is_scalar = expected_output(test_cases[0])
        .map(|v| matches!(v, Value::Int(_)))
        .unwrap_or(false);

    if !first_input0_is_list || !first_input1_is_scalar || !first_output_is_scalar {
        return None;
    }

    let all_match = test_cases.iter().all(|tc| {
        let coeffs = match as_int_list(&tc.inputs[0]) {
            Some(v) => v,
            None => return false,
        };
        let x = match as_int(&tc.inputs[1]) {
            Some(v) => v,
            None => return false,
        };
        let out = match expected_output(tc).and_then(as_int) {
            Some(v) => v,
            None => return false,
        };
        // Evaluate polynomial: sum(coeff[i] * x^i)
        let mut result: i64 = 0;
        let mut x_pow: i64 = 1;
        for &c in &coeffs {
            result = result.wrapping_add(c.wrapping_mul(x_pow));
            x_pow = x_pow.wrapping_mul(x);
        }
        result == out
    });

    if all_match {
        Some(DetectedPattern::Polynomial)
    } else {
        None
    }
}

/// Detect linear search: output is the index where list[output] == target,
/// or -1 if not found.
fn detect_linear_search(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 2 {
        return None;
    }

    let first_input0_is_list = matches!(&test_cases[0].inputs[0], Value::Tuple(_));
    let first_input1_is_scalar = matches!(&test_cases[0].inputs[1], Value::Int(_));
    let first_output_is_scalar = expected_output(test_cases[0])
        .map(|v| matches!(v, Value::Int(_)))
        .unwrap_or(false);

    if !first_input0_is_list || !first_input1_is_scalar || !first_output_is_scalar {
        return None;
    }

    let all_match = test_cases.iter().all(|tc| {
        let list = match as_int_list(&tc.inputs[0]) {
            Some(v) => v,
            None => return false,
        };
        let target = match as_int(&tc.inputs[1]) {
            Some(v) => v,
            None => return false,
        };
        let out = match expected_output(tc).and_then(as_int) {
            Some(v) => v,
            None => return false,
        };
        let expected_idx = list.iter().position(|&x| x == target)
            .map(|i| i as i64)
            .unwrap_or(-1);
        expected_idx == out
    });

    if all_match {
        Some(DetectedPattern::LinearSearch)
    } else {
        None
    }
}

/// Detect Fibonacci: output = fib(n) for single scalar input.
fn detect_fibonacci(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 3 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 1 {
        return None;
    }

    fn fib(n: i64) -> Option<i64> {
        if n < 0 { return None; }
        if n > 50 { return None; }  // avoid huge computations
        let (mut a, mut b): (i64, i64) = (0, 1);
        for _ in 0..n {
            let next = a.checked_add(b)?;
            a = b;
            b = next;
        }
        Some(a)
    }

    let all_match = test_cases.iter().all(|tc| {
        let inp = match as_int(&tc.inputs[0]) {
            Some(v) => v,
            None => return false,
        };
        let out = match expected_output(tc).and_then(as_int) {
            Some(v) => v,
            None => return false,
        };
        fib(inp) == Some(out)
    });

    if all_match {
        Some(DetectedPattern::Fibonacci)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// New advanced pattern detectors
// ---------------------------------------------------------------------------

/// Detect two-fold-scalar: output = combine_op(fold1(list), fold2(list)).
/// E.g., average = div(sum, len), range = sub(max, min).
fn detect_two_fold_scalar(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();

    // 0xFE = count (len)
    let fold_ops_extended: &[(u8, fn(&[i64]) -> i64)] = &[
        (0x00, |list: &[i64]| list.iter().sum()),
        (0x07, |list: &[i64]| *list.iter().min().unwrap_or(&0)),
        (0x08, |list: &[i64]| *list.iter().max().unwrap_or(&0)),
        (0xFE, |list: &[i64]| list.len() as i64),
    ];

    let combine_ops: &[(u8, fn(i64, i64) -> Option<i64>)] = &[
        (0x00, |a, b| Some(a.wrapping_add(b))),
        (0x01, |a, b| Some(a.wrapping_sub(b))),
        (0x02, |a, b| Some(a.wrapping_mul(b))),
        (0x03, |a, b| if b != 0 { Some(a / b) } else { None }),
    ];

    for input_idx in 0..num_inputs {
        let first_input_is_list = matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_));
        let first_output_is_scalar = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Int(_)))
            .unwrap_or(false);

        if !first_input_is_list || !first_output_is_scalar {
            continue;
        }

        for &(f1_op, f1_fn) in fold_ops_extended {
            for &(f2_op, f2_fn) in fold_ops_extended {
                if f1_op == f2_op {
                    continue; // skip same fold (would be covered by simpler patterns)
                }
                for &(c_op, c_fn) in combine_ops {
                    let all_match = test_cases.iter().all(|tc| {
                        let out = match expected_output(tc).and_then(as_int) {
                            Some(v) => v,
                            None => return false,
                        };
                        let list = match as_int_list(&tc.inputs[input_idx]) {
                            Some(v) => v,
                            None => return false,
                        };
                        if list.is_empty() {
                            return false;
                        }
                        let v1 = f1_fn(&list);
                        let v2 = f2_fn(&list);
                        c_fn(v1, v2) == Some(out)
                    });
                    if all_match {
                        let (f1_base, f2_base) = (
                            match f1_op { 0x00 => 0, 0x07 => i64::MAX, 0x08 => i64::MIN, 0xFE => 0, _ => 0 },
                            match f2_op { 0x00 => 0, 0x07 => i64::MAX, 0x08 => i64::MIN, 0xFE => 0, _ => 0 },
                        );
                        return Some(DetectedPattern::TwoFoldScalar {
                            fold1_base: f1_base,
                            fold1_op: f1_op,
                            fold2_base: f2_base,
                            fold2_op: f2_op,
                            combine_op: c_op,
                            input_index: input_idx,
                        });
                    }
                }
            }
        }
    }
    None
}

/// Detect fold-then-map: output_list = map(|x| op(fold_val, x), input_list).
/// E.g., distance_from_max: map(|x| max - x, list) where max = fold(MIN, max, list).
fn detect_fold_then_map(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();

    let fold_ops: &[(u8, fn(i64, i64) -> i64, i64)] = &[
        (0x00, |a, b| a.wrapping_add(b), 0),
        (0x07, |a, b| a.min(b), i64::MAX),
        (0x08, |a, b| a.max(b), i64::MIN),
    ];

    let map_ops: &[(u8, fn(i64, i64) -> Option<i64>)] = &[
        (0x00, |a, b| Some(a.wrapping_add(b))),
        (0x01, |a, b| Some(a.wrapping_sub(b))),
        (0x02, |a, b| Some(a.wrapping_mul(b))),
        (0x03, |a, b| if b != 0 { Some(a / b) } else { None }),
    ];

    for input_idx in 0..num_inputs {
        let first_input_is_list = matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_));
        let first_output_is_list = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Tuple(_)))
            .unwrap_or(false);

        if !first_input_is_list || !first_output_is_list {
            continue;
        }

        for &(fold_opcode, fold_fn, fold_base) in fold_ops {
            for &(map_opcode, map_fn) in map_ops {
                // Try fold_first = true: output[i] = map_op(fold_val, input[i])
                let all_match_ff = test_cases.iter().all(|tc| {
                    let input_list = match as_int_list(&tc.inputs[input_idx]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let output_list = match expected_output(tc).and_then(as_int_list) {
                        Some(v) => v,
                        None => return false,
                    };
                    if input_list.len() != output_list.len() || input_list.is_empty() {
                        return false;
                    }
                    let fold_val = input_list.iter().fold(fold_base, |acc, &x| fold_fn(acc, x));
                    input_list.iter().zip(output_list.iter())
                        .all(|(&inp, &out)| map_fn(fold_val, inp) == Some(out))
                });
                if all_match_ff {
                    return Some(DetectedPattern::FoldThenMap {
                        fold_base,
                        fold_op: fold_opcode,
                        map_op: map_opcode,
                        fold_first: true,
                        input_index: input_idx,
                    });
                }

                // Try fold_first = false: output[i] = map_op(input[i], fold_val)
                let all_match_fl = test_cases.iter().all(|tc| {
                    let input_list = match as_int_list(&tc.inputs[input_idx]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let output_list = match expected_output(tc).and_then(as_int_list) {
                        Some(v) => v,
                        None => return false,
                    };
                    if input_list.len() != output_list.len() || input_list.is_empty() {
                        return false;
                    }
                    let fold_val = input_list.iter().fold(fold_base, |acc, &x| fold_fn(acc, x));
                    input_list.iter().zip(output_list.iter())
                        .all(|(&inp, &out)| map_fn(inp, fold_val) == Some(out))
                });
                if all_match_fl {
                    return Some(DetectedPattern::FoldThenMap {
                        fold_base,
                        fold_op: fold_opcode,
                        map_op: map_opcode,
                        fold_first: false,
                        input_index: input_idx,
                    });
                }
            }
        }
    }
    None
}

/// Detect map-with-arg: output = map(|x| x op arg, list).
/// E.g., add_constant(list, c) = map(|x| x + c, list).
fn detect_map_with_arg(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 2 {
        return None;
    }

    let ops: &[(u8, fn(i64, i64) -> Option<i64>)] = &[
        (0x00, |a, b| Some(a.wrapping_add(b))),
        (0x01, |a, b| Some(a.wrapping_sub(b))),
        (0x02, |a, b| Some(a.wrapping_mul(b))),
        (0x03, |a, b| if b != 0 { Some(a / b) } else { None }),
    ];

    for list_idx in 0..num_inputs {
        for arg_idx in 0..num_inputs {
            if list_idx == arg_idx {
                continue;
            }
            for &(opcode, op_fn) in ops {
                let all_match = test_cases.iter().all(|tc| {
                    let input_list = match as_int_list(&tc.inputs[list_idx]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let arg = match as_int(&tc.inputs[arg_idx]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let output_list = match expected_output(tc).and_then(as_int_list) {
                        Some(v) => v,
                        None => return false,
                    };
                    if input_list.len() != output_list.len() {
                        return false;
                    }
                    input_list.iter().zip(output_list.iter())
                        .all(|(&inp, &out)| op_fn(inp, arg) == Some(out))
                });
                if all_match {
                    return Some(DetectedPattern::MapWithArg {
                        op: opcode,
                        list_input: list_idx,
                        arg_input: arg_idx,
                    });
                }
            }
        }
    }
    None
}

/// Detect conditional count: output = count(x cmp_op threshold for x in list).
/// E.g., count_greater_than(list, t) = count where x > t.
fn detect_conditional_count(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 2 {
        return None;
    }

    let cmp_ops: &[(u8, fn(i64, i64) -> bool)] = &[
        (0x23, |a, b| a > b),     // gt
        (0x22, |a, b| a < b),     // lt
        (0x25, |a, b| a >= b),    // ge
        (0x24, |a, b| a <= b),    // le
        (0x20, |a, b| a == b),    // eq
        (0x21, |a, b| a != b),    // ne
    ];

    for list_idx in 0..num_inputs {
        for thresh_idx in 0..num_inputs {
            if list_idx == thresh_idx {
                continue;
            }
            for &(cmp_op, cmp_fn) in cmp_ops {
                let all_match = test_cases.iter().all(|tc| {
                    let list = match as_int_list(&tc.inputs[list_idx]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let thresh = match as_int(&tc.inputs[thresh_idx]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let out = match expected_output(tc).and_then(as_int) {
                        Some(v) => v,
                        None => return false,
                    };
                    let count = list.iter().filter(|&&x| cmp_fn(x, thresh)).count() as i64;
                    count == out
                });
                if all_match {
                    return Some(DetectedPattern::ConditionalCount {
                        cmp_op,
                        list_input: list_idx,
                        threshold_input: thresh_idx,
                    });
                }
            }
        }
    }
    None
}

/// Detect zip_map_fold + bias: output = dot(a, b) + bias.
fn detect_zip_map_fold_plus_bias(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 3 {
        return None;
    }

    // Check which input is the scalar bias.
    for bias_idx in 0..3 {
        let list_indices: Vec<usize> = (0..3).filter(|&i| i != bias_idx).collect();
        let i0 = list_indices[0];
        let i1 = list_indices[1];

        let both_lists = matches!(&test_cases[0].inputs[i0], Value::Tuple(_))
            && matches!(&test_cases[0].inputs[i1], Value::Tuple(_));
        let bias_is_scalar = matches!(&test_cases[0].inputs[bias_idx], Value::Int(_));
        let output_is_scalar = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Int(_)))
            .unwrap_or(false);

        if !both_lists || !bias_is_scalar || !output_is_scalar {
            continue;
        }

        // Try pair_op=mul, fold_op=add (dot product) + bias
        let all_match = test_cases.iter().all(|tc| {
            let list_a = match as_int_list(&tc.inputs[i0]) {
                Some(v) => v,
                None => return false,
            };
            let list_b = match as_int_list(&tc.inputs[i1]) {
                Some(v) => v,
                None => return false,
            };
            let bias = match as_int(&tc.inputs[bias_idx]) {
                Some(v) => v,
                None => return false,
            };
            let out = match expected_output(tc).and_then(as_int) {
                Some(v) => v,
                None => return false,
            };
            if list_a.len() != list_b.len() {
                return false;
            }
            let dot: i64 = list_a.iter().zip(list_b.iter())
                .map(|(&a, &b)| a.wrapping_mul(b))
                .sum();
            dot.wrapping_add(bias) == out
        });
        if all_match {
            return Some(DetectedPattern::ZipMapFoldPlusBias {
                pair_op: 0x02,  // mul
                fold_op: 0x00,  // add
                fold_base: 0,
                bias_input: bias_idx,
            });
        }
    }
    None
}

/// Detect fold-after-take: output = fold(base, op, take(list, n)).
fn detect_fold_after_take(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 2 {
        return None;
    }

    let fold_ops: &[(u8, fn(i64, i64) -> i64, i64)] = &[
        (0x00, |a, b| a.wrapping_add(b), 0),
        (0x02, |a, b| a.wrapping_mul(b), 1),
        (0x07, |a, b| a.min(b), i64::MAX),
        (0x08, |a, b| a.max(b), i64::MIN),
    ];

    for list_idx in 0..num_inputs {
        for count_idx in 0..num_inputs {
            if list_idx == count_idx {
                continue;
            }
            let first_list = matches!(&test_cases[0].inputs[list_idx], Value::Tuple(_));
            let first_count = matches!(&test_cases[0].inputs[count_idx], Value::Int(_));
            let first_out_scalar = expected_output(test_cases[0])
                .map(|v| matches!(v, Value::Int(_)))
                .unwrap_or(false);

            if !first_list || !first_count || !first_out_scalar {
                continue;
            }

            for &(fold_opcode, fold_fn, fold_base) in fold_ops {
                let all_match = test_cases.iter().all(|tc| {
                    let list = match as_int_list(&tc.inputs[list_idx]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let n = match as_int(&tc.inputs[count_idx]) {
                        Some(v) if v >= 0 => v as usize,
                        _ => return false,
                    };
                    let out = match expected_output(tc).and_then(as_int) {
                        Some(v) => v,
                        None => return false,
                    };
                    let taken: Vec<i64> = list.into_iter().take(n).collect();
                    let result = taken.iter().fold(fold_base, |acc, &x| fold_fn(acc, x));
                    result == out
                });
                if all_match {
                    return Some(DetectedPattern::FoldAfterTake {
                        fold_base,
                        fold_op: fold_opcode,
                        list_input: list_idx,
                        count_input: count_idx,
                    });
                }
            }
        }
    }
    None
}

/// Detect scan (prefix sum/product): output[i] = fold(base, op, input[0..=i]).
fn detect_scan(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();

    let scan_ops: &[(u8, fn(i64, i64) -> i64, i64)] = &[
        (0x00, |a, b| a.wrapping_add(b), 0),   // prefix sum
        (0x02, |a, b| a.wrapping_mul(b), 1),    // prefix product
        (0x07, |a, b| a.min(b), i64::MAX),      // prefix min
        (0x08, |a, b| a.max(b), i64::MIN),      // prefix max
    ];

    for input_idx in 0..num_inputs {
        let first_input_is_list = matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_));
        let first_output_is_list = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Tuple(_)))
            .unwrap_or(false);

        if !first_input_is_list || !first_output_is_list {
            continue;
        }

        for &(opcode, op_fn, base) in scan_ops {
            let all_match = test_cases.iter().all(|tc| {
                let input_list = match as_int_list(&tc.inputs[input_idx]) {
                    Some(v) => v,
                    None => return false,
                };
                let output_list = match expected_output(tc).and_then(as_int_list) {
                    Some(v) => v,
                    None => return false,
                };
                if input_list.len() != output_list.len() {
                    return false;
                }
                let mut acc = base;
                for (i, &x) in input_list.iter().enumerate() {
                    acc = op_fn(acc, x);
                    if acc != output_list[i] {
                        return false;
                    }
                }
                true
            });
            if all_match {
                return Some(DetectedPattern::Scan {
                    base,
                    op: opcode,
                    input_index: input_idx,
                });
            }
        }
    }
    None
}

/// Detect binary to decimal: output = fold(|acc, bit| acc*2 + bit, 0, bits).
fn detect_binary_to_decimal(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 1 {
        return None;
    }

    let first_input_is_list = matches!(&test_cases[0].inputs[0], Value::Tuple(_));
    let first_output_is_scalar = expected_output(test_cases[0])
        .map(|v| matches!(v, Value::Int(_)))
        .unwrap_or(false);

    if !first_input_is_list || !first_output_is_scalar {
        return None;
    }

    let all_match = test_cases.iter().all(|tc| {
        let bits = match as_int_list(&tc.inputs[0]) {
            Some(v) => v,
            None => return false,
        };
        let out = match expected_output(tc).and_then(as_int) {
            Some(v) => v,
            None => return false,
        };
        // fold(|acc, bit| acc*2 + bit, 0, bits)
        let result = bits.iter().fold(0i64, |acc, &bit| acc * 2 + bit);
        result == out
    });

    if all_match {
        Some(DetectedPattern::BinaryToDecimal)
    } else {
        None
    }
}

/// Detect digit_sum: output = sum of digits of input.
fn detect_digit_sum(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 1 {
        return None;
    }

    let all_match = test_cases.iter().all(|tc| {
        let n = match as_int(&tc.inputs[0]) {
            Some(v) if v >= 0 => v,
            _ => return false,
        };
        let out = match expected_output(tc).and_then(as_int) {
            Some(v) => v,
            None => return false,
        };
        let mut sum = 0i64;
        let mut val = n;
        if val == 0 {
            return out == 0;
        }
        while val > 0 {
            sum += val % 10;
            val /= 10;
        }
        sum == out
    });

    if all_match {
        Some(DetectedPattern::DigitSum)
    } else {
        None
    }
}

/// Detect count_divisors: output = number of divisors of n.
fn detect_count_divisors(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 1 {
        return None;
    }

    let all_match = test_cases.iter().all(|tc| {
        let n = match as_int(&tc.inputs[0]) {
            Some(v) if v >= 1 => v,
            _ => return false,
        };
        let out = match expected_output(tc).and_then(as_int) {
            Some(v) => v,
            None => return false,
        };
        let count = (1..=n).filter(|&i| n % i == 0).count() as i64;
        count == out
    });

    if all_match {
        Some(DetectedPattern::CountDivisors)
    } else {
        None
    }
}

/// Detect is_prime: output = 1 if n is prime, 0 otherwise.
fn detect_is_prime(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 3 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 1 {
        return None;
    }

    let all_match = test_cases.iter().all(|tc| {
        let n = match as_int(&tc.inputs[0]) {
            Some(v) => v,
            _ => return false,
        };
        let out = match expected_output(tc).and_then(as_int) {
            Some(v) => v,
            None => return false,
        };
        let is_prime = if n < 2 {
            false
        } else {
            (2..n).all(|i| n % i != 0)
        };
        let expected = if is_prime { 1 } else { 0 };
        expected == out
    });

    if all_match {
        Some(DetectedPattern::IsPrime)
    } else {
        None
    }
}

/// Detect variance_numerator: sum((x - mean)^2) where mean = sum/len.
fn detect_variance_numerator(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        let first_input_is_list = matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_));
        let first_output_is_scalar = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Int(_)))
            .unwrap_or(false);

        if !first_input_is_list || !first_output_is_scalar {
            continue;
        }

        let all_match = test_cases.iter().all(|tc| {
            let list = match as_int_list(&tc.inputs[input_idx]) {
                Some(v) => v,
                None => return false,
            };
            let out = match expected_output(tc).and_then(as_int) {
                Some(v) => v,
                None => return false,
            };
            if list.is_empty() {
                return out == 0;
            }
            let sum: i64 = list.iter().sum();
            let len = list.len() as i64;
            let mean = sum / len; // integer division
            let var_num: i64 = list.iter().map(|&x| (x - mean) * (x - mean)).sum();
            var_num == out
        });

        if all_match {
            return Some(DetectedPattern::VarianceNumerator { input_index: input_idx });
        }
    }
    None
}

/// Detect collatz_step: if x%2==0 then x/2 else 3*x+1.
fn detect_collatz_step(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 3 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 1 {
        return None;
    }

    let all_match = test_cases.iter().all(|tc| {
        let x = match as_int(&tc.inputs[0]) {
            Some(v) => v,
            None => return false,
        };
        let out = match expected_output(tc).and_then(as_int) {
            Some(v) => v,
            None => return false,
        };
        let expected = if x % 2 == 0 { x / 2 } else { 3 * x + 1 };
        expected == out
    });

    if all_match {
        Some(DetectedPattern::CollatzStep)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// New v3 pattern detectors
// ---------------------------------------------------------------------------

/// Detect unary affine: output = a*input + b for single scalar input.
/// E.g., apply_twice(f=+3) => output = input + 6, so a=1, b=6.
fn detect_unary_affine(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 1 {
        return None;
    }

    // All must be scalar -> scalar.
    let mut pairs: Vec<(i64, i64)> = Vec::new();
    for tc in test_cases {
        let inp = as_int(&tc.inputs[0])?;
        let out = expected_output(tc).and_then(as_int)?;
        pairs.push((inp, out));
    }

    // Try to find a, b such that out = a*inp + b.
    // With two points (x1, y1) and (x2, y2):
    // a = (y2 - y1) / (x2 - x1) if x1 != x2
    // b = y1 - a * x1
    let (x1, y1) = pairs[0];
    // Find a second pair with different input.
    let second = pairs.iter().find(|&&(x, _)| x != x1)?;
    let (x2, y2) = *second;

    let dx = x2 - x1;
    let dy = y2 - y1;

    if dx == 0 || dy % dx != 0 {
        return None;
    }
    let a = dy / dx;
    let b = y1 - a * x1;

    // Verify all test cases match.
    let all_match = pairs.iter().all(|&(x, y)| a * x + b == y);
    if all_match {
        Some(DetectedPattern::UnaryAffine { a, b })
    } else {
        None
    }
}

/// Detect fold-then-mod: output = fold(base, op, list) mod constant.
/// E.g., checksum = sum(list) mod 256.
fn detect_fold_then_mod(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();

    let fold_ops: &[(u8, fn(i64, i64) -> i64, i64)] = &[
        (0x00, |a, b| a.wrapping_add(b), 0),
        (0x02, |a, b| a.wrapping_mul(b), 1),
    ];

    // Try common moduli.
    let moduli: &[i64] = &[256, 128, 64, 1000, 100, 10, 2];

    for input_idx in 0..num_inputs {
        let first_input_is_list = matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_));
        let first_output_is_scalar = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Int(_)))
            .unwrap_or(false);

        if !first_input_is_list || !first_output_is_scalar {
            continue;
        }

        for &(fold_opcode, fold_fn, fold_base) in fold_ops {
            for &m in moduli {
                let all_match = test_cases.iter().all(|tc| {
                    let out = match expected_output(tc).and_then(as_int) {
                        Some(v) => v,
                        None => return false,
                    };
                    let list = match as_int_list(&tc.inputs[input_idx]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let folded = list.iter().fold(fold_base, |acc, &x| fold_fn(acc, x));
                    let result = ((folded % m) + m) % m; // ensure positive mod
                    result == out
                });
                if all_match {
                    return Some(DetectedPattern::FoldThenMod {
                        fold_base,
                        fold_op: fold_opcode,
                        mod_val: m,
                        input_index: input_idx,
                    });
                }
            }
        }
    }
    None
}

/// Detect concat_element: output = concat(list, [element]).
/// E.g., stack_push([1,2], 3) => [1,2,3].
fn detect_concat_element(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 2 {
        return None;
    }

    for list_idx in 0..num_inputs {
        for elem_idx in 0..num_inputs {
            if list_idx == elem_idx {
                continue;
            }
            let first_list = matches!(&test_cases[0].inputs[list_idx], Value::Tuple(_));
            let first_elem = matches!(&test_cases[0].inputs[elem_idx], Value::Int(_));
            let first_output_is_list = expected_output(test_cases[0])
                .map(|v| matches!(v, Value::Tuple(_)))
                .unwrap_or(false);

            if !first_list || !first_elem || !first_output_is_list {
                continue;
            }

            let all_match = test_cases.iter().all(|tc| {
                let list = match as_int_list(&tc.inputs[list_idx]) {
                    Some(v) => v,
                    None => return false,
                };
                let elem = match as_int(&tc.inputs[elem_idx]) {
                    Some(v) => v,
                    None => return false,
                };
                let output = match expected_output(tc).and_then(as_int_list) {
                    Some(v) => v,
                    None => return false,
                };
                let mut expected = list.clone();
                expected.push(elem);
                expected == output
            });
            if all_match {
                return Some(DetectedPattern::ConcatElement {
                    list_input: list_idx,
                    elem_input: elem_idx,
                });
            }
        }
    }
    None
}

/// Detect init_list: output = take(list, len(list)-1) — all but last.
fn detect_init_list(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        let first_input_is_list = matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_));
        let first_output_is_list = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Tuple(_)))
            .unwrap_or(false);

        if !first_input_is_list || !first_output_is_list {
            continue;
        }

        let all_match = test_cases.iter().all(|tc| {
            let input_list = match as_int_list(&tc.inputs[input_idx]) {
                Some(v) => v,
                None => return false,
            };
            let output_list = match expected_output(tc).and_then(as_int_list) {
                Some(v) => v,
                None => return false,
            };
            if input_list.is_empty() {
                return output_list.is_empty();
            }
            let expected: Vec<i64> = input_list[..input_list.len() - 1].to_vec();
            expected == output_list
        });
        if all_match {
            return Some(DetectedPattern::InitList { input_index: input_idx });
        }
    }
    None
}

/// Detect repeat: output = [x, x, ..., x] (n times).
fn detect_repeat(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 2 {
        return None;
    }

    for elem_idx in 0..num_inputs {
        for count_idx in 0..num_inputs {
            if elem_idx == count_idx {
                continue;
            }
            let first_elem = matches!(&test_cases[0].inputs[elem_idx], Value::Int(_));
            let first_count = matches!(&test_cases[0].inputs[count_idx], Value::Int(_));
            let first_output_is_list = expected_output(test_cases[0])
                .map(|v| matches!(v, Value::Tuple(_)))
                .unwrap_or(false);

            if !first_elem || !first_count || !first_output_is_list {
                continue;
            }

            let all_match = test_cases.iter().all(|tc| {
                let elem = match as_int(&tc.inputs[elem_idx]) {
                    Some(v) => v,
                    None => return false,
                };
                let count = match as_int(&tc.inputs[count_idx]) {
                    Some(v) if v >= 0 => v as usize,
                    _ => return false,
                };
                let output = match expected_output(tc).and_then(as_int_list) {
                    Some(v) => v,
                    None => return false,
                };
                output.len() == count && output.iter().all(|&x| x == elem)
            });
            if all_match {
                return Some(DetectedPattern::Repeat {
                    elem_input: elem_idx,
                    count_input: count_idx,
                });
            }
        }
    }
    None
}

/// Detect range: output = [0, 1, 2, ..., n-1].
fn detect_range(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 1 {
        return None;
    }

    let all_match = test_cases.iter().all(|tc| {
        let n = match as_int(&tc.inputs[0]) {
            Some(v) if v >= 0 => v as usize,
            _ => return false,
        };
        let output = match expected_output(tc).and_then(as_int_list) {
            Some(v) => v,
            None => return false,
        };
        if output.len() != n {
            return false;
        }
        output.iter().enumerate().all(|(i, &v)| v == i as i64)
    });
    if all_match {
        Some(DetectedPattern::Range)
    } else {
        None
    }
}

/// Detect element_access: output is a linear combination of list elements.
/// E.g., mat_trace([a,b,c,d]) = a+d => terms=[(1,0),(1,3)].
fn detect_element_access(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        let first_input_is_list = matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_));
        let first_output_is_scalar = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Int(_)))
            .unwrap_or(false);

        if !first_input_is_list || !first_output_is_scalar {
            continue;
        }

        // Get the list lengths — they must be consistent.
        let list_len = match as_int_list(&test_cases[0].inputs[input_idx]) {
            Some(v) => v.len(),
            None => continue,
        };
        if list_len == 0 || list_len > 16 {
            continue;
        }

        // Try all single-element access: output = list[i]
        for i in 0..list_len {
            let all_match = test_cases.iter().all(|tc| {
                let list = match as_int_list(&tc.inputs[input_idx]) {
                    Some(v) => v,
                    None => return false,
                };
                let out = match expected_output(tc).and_then(as_int) {
                    Some(v) => v,
                    None => return false,
                };
                list.len() > i && list[i] == out
            });
            if all_match {
                return Some(DetectedPattern::ElementAccess {
                    terms: vec![(1, i)],
                    input_index: input_idx,
                });
            }
        }

        // Try sum of two elements: output = list[i] + list[j]
        for i in 0..list_len {
            for j in (i+1)..list_len {
                let all_match = test_cases.iter().all(|tc| {
                    let list = match as_int_list(&tc.inputs[input_idx]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let out = match expected_output(tc).and_then(as_int) {
                        Some(v) => v,
                        None => return false,
                    };
                    list.len() > j && list[i] + list[j] == out
                });
                if all_match {
                    return Some(DetectedPattern::ElementAccess {
                        terms: vec![(1, i), (1, j)],
                        input_index: input_idx,
                    });
                }
            }
        }

        // Try difference of products: output = list[i]*list[j] - list[k]*list[l]
        // (covers mat_det: a*d - b*c)
        // Handled separately by detect_mat_det_2x2.
    }
    None
}

/// Detect mat_det_2x2: output = list[0]*list[3] - list[1]*list[2].
fn detect_mat_det_2x2(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        let list = match as_int_list(&test_cases[0].inputs[input_idx]) {
            Some(v) if v.len() == 4 => v,
            _ => continue,
        };
        let out = match expected_output(test_cases[0]).and_then(as_int) {
            Some(v) => v,
            None => continue,
        };
        // Quick check: does list[0]*list[3] - list[1]*list[2] == out?
        if list[0] * list[3] - list[1] * list[2] != out {
            continue;
        }

        let all_match = test_cases.iter().all(|tc| {
            let list = match as_int_list(&tc.inputs[input_idx]) {
                Some(v) if v.len() == 4 => v,
                _ => return false,
            };
            let out = match expected_output(tc).and_then(as_int) {
                Some(v) => v,
                None => return false,
            };
            list[0] * list[3] - list[1] * list[2] == out
        });
        if all_match {
            return Some(DetectedPattern::MatDet2x2 { input_index: input_idx });
        }
    }
    None
}

/// Detect mat_vec_mul_2x2: [a,b,c,d] * [x,y] => [a*x+b*y, c*x+d*y].
fn detect_mat_vec_mul_2x2(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 2 {
        return None;
    }

    // Check if input0 is length-4 list and input1 is length-2 list.
    let mat_list = as_int_list(&test_cases[0].inputs[0]);
    let vec_list = as_int_list(&test_cases[0].inputs[1]);
    let out_list = expected_output(test_cases[0]).and_then(as_int_list);

    match (&mat_list, &vec_list, &out_list) {
        (Some(m), Some(v), Some(o)) if m.len() == 4 && v.len() == 2 && o.len() == 2 => {}
        _ => return None,
    }

    let all_match = test_cases.iter().all(|tc| {
        let mat = match as_int_list(&tc.inputs[0]) {
            Some(v) if v.len() == 4 => v,
            _ => return false,
        };
        let vec = match as_int_list(&tc.inputs[1]) {
            Some(v) if v.len() == 2 => v,
            _ => return false,
        };
        let out = match expected_output(tc).and_then(as_int_list) {
            Some(v) if v.len() == 2 => v,
            _ => return false,
        };
        out[0] == mat[0] * vec[0] + mat[1] * vec[1]
            && out[1] == mat[2] * vec[0] + mat[3] * vec[1]
    });
    if all_match {
        Some(DetectedPattern::MatVecMul2x2)
    } else {
        None
    }
}

/// Detect state_increment: state[key] += constant, output = Unit.
fn detect_state_increment(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    // All test cases must have expected_state.
    // Check if all test cases have no inputs and Unit output.
    let first = test_cases[0];
    if first.expected_state.is_none() || first.initial_state.is_none() {
        return None;
    }

    let init0 = first.initial_state.as_ref()?;
    let exp0 = first.expected_state.as_ref()?;

    // Find a key that differs by a constant across all test cases.
    for (key, init_val) in init0 {
        let init_n = as_int(init_val)?;
        let exp_n = as_int(exp0.get(key)?)?;
        let delta = exp_n - init_n;

        let all_match = test_cases.iter().all(|tc| {
            let init = match &tc.initial_state {
                Some(s) => s,
                None => return false,
            };
            let exp = match &tc.expected_state {
                Some(s) => s,
                None => return false,
            };
            let iv = match init.get(key).and_then(as_int) {
                Some(v) => v,
                None => return false,
            };
            let ev = match exp.get(key).and_then(as_int) {
                Some(v) => v,
                None => return false,
            };
            ev - iv == delta
        });
        if all_match {
            return Some(DetectedPattern::StateIncrement {
                key: key.clone(),
                delta,
            });
        }
    }
    None
}

/// Detect state_accumulate: state[key] += input, output = new state[key].
fn detect_state_accumulate(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let first = test_cases[0];
    if first.expected_state.is_none() || first.initial_state.is_none() {
        return None;
    }
    let num_inputs = first.inputs.len();
    if num_inputs < 1 {
        return None;
    }

    let init0 = first.initial_state.as_ref()?;
    let _exp0 = first.expected_state.as_ref()?;

    for (key, _init_val) in init0 {
        for input_idx in 0..num_inputs {
            let all_match = test_cases.iter().all(|tc| {
                let init = match &tc.initial_state {
                    Some(s) => s,
                    None => return false,
                };
                let exp = match &tc.expected_state {
                    Some(s) => s,
                    None => return false,
                };
                let iv = match init.get(key).and_then(as_int) {
                    Some(v) => v,
                    None => return false,
                };
                let ev = match exp.get(key).and_then(as_int) {
                    Some(v) => v,
                    None => return false,
                };
                let inp = match as_int(&tc.inputs[input_idx]) {
                    Some(v) => v,
                    None => return false,
                };
                let out = match expected_output(tc).and_then(as_int) {
                    Some(v) => v,
                    None => return false,
                };
                // state[key] = old + input, output = new state[key]
                ev == iv + inp && out == ev
            });
            if all_match {
                return Some(DetectedPattern::StateAccumulate {
                    key: key.clone(),
                    input_index: input_idx,
                });
            }
        }
    }
    None
}

/// Detect top_k_sum: output = sum of k largest elements.
fn detect_top_k_sum(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        let first_input_is_list = matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_));
        let first_output_is_scalar = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Int(_)))
            .unwrap_or(false);

        if !first_input_is_list || !first_output_is_scalar {
            continue;
        }

        for k in 2..=4 {
            let all_match = test_cases.iter().all(|tc| {
                let list = match as_int_list(&tc.inputs[input_idx]) {
                    Some(v) => v,
                    None => return false,
                };
                let out = match expected_output(tc).and_then(as_int) {
                    Some(v) => v,
                    None => return false,
                };
                if list.len() < k {
                    return false;
                }
                let mut sorted = list.clone();
                sorted.sort_unstable();
                sorted.reverse();
                let top_k_sum: i64 = sorted.iter().take(k).sum();
                top_k_sum == out
            });
            if all_match {
                return Some(DetectedPattern::TopKSum { k, input_index: input_idx });
            }
        }
    }
    None
}

/// Detect moving_avg_2: output[i] = (input[i] + input[i+1]) / 2.
fn detect_moving_avg_2(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        let first_input_is_list = matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_));
        let first_output_is_list = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Tuple(_)))
            .unwrap_or(false);

        if !first_input_is_list || !first_output_is_list {
            continue;
        }

        let all_match = test_cases.iter().all(|tc| {
            let input_list = match as_int_list(&tc.inputs[input_idx]) {
                Some(v) => v,
                None => return false,
            };
            let output_list = match expected_output(tc).and_then(as_int_list) {
                Some(v) => v,
                None => return false,
            };
            if input_list.len() < 2 || output_list.len() != input_list.len() - 1 {
                return false;
            }
            (0..output_list.len()).all(|i| {
                output_list[i] == (input_list[i] + input_list[i + 1]) / 2
            })
        });
        if all_match {
            return Some(DetectedPattern::MovingAvg2 { input_index: input_idx });
        }
    }
    None
}

/// Detect starts_with: output = 1 if list A starts with prefix B, 0 otherwise.
fn detect_starts_with(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs != 2 {
        return None;
    }

    let both_lists = matches!(&test_cases[0].inputs[0], Value::Tuple(_))
        && matches!(&test_cases[0].inputs[1], Value::Tuple(_));
    let output_scalar = expected_output(test_cases[0])
        .map(|v| matches!(v, Value::Int(_)))
        .unwrap_or(false);

    if !both_lists || !output_scalar {
        return None;
    }

    let all_match = test_cases.iter().all(|tc| {
        let list_a = match as_int_list(&tc.inputs[0]) {
            Some(v) => v,
            None => return false,
        };
        let list_b = match as_int_list(&tc.inputs[1]) {
            Some(v) => v,
            None => return false,
        };
        let out = match expected_output(tc).and_then(as_int) {
            Some(v) => v,
            None => return false,
        };
        let starts = if list_b.len() > list_a.len() {
            false
        } else {
            list_a[..list_b.len()] == list_b[..]
        };
        let expected = if starts { 1 } else { 0 };
        expected == out
    });
    if all_match {
        Some(DetectedPattern::StartsWith)
    } else {
        None
    }
}

/// Detect contains pattern: does list contain a specific constant value?
/// Output is 1 if the target value appears in the list, 0 otherwise.
/// Tries all constant targets that appear in the test case outputs (0 or 1).
fn detect_contains(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();

    for list_idx in 0..num_inputs {
        let first_input_is_list = matches!(&test_cases[0].inputs[list_idx], Value::Tuple(_));
        let first_output_is_scalar = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Int(_)))
            .unwrap_or(false);

        if !first_input_is_list || !first_output_is_scalar {
            continue;
        }

        // All outputs must be 0 or 1
        let all_binary = test_cases.iter().all(|tc| {
            matches!(expected_output(tc).and_then(as_int), Some(0) | Some(1))
        });
        if !all_binary {
            continue;
        }

        // Collect all values that appear in the lists
        let mut candidate_targets = std::collections::BTreeSet::new();
        for tc in test_cases.iter() {
            if let Some(list) = as_int_list(&tc.inputs[list_idx]) {
                for &v in &list {
                    candidate_targets.insert(v);
                }
            }
        }

        for target in &candidate_targets {
            let all_match = test_cases.iter().all(|tc| {
                let list = match as_int_list(&tc.inputs[list_idx]) {
                    Some(v) => v,
                    None => return false,
                };
                let out = match expected_output(tc).and_then(as_int) {
                    Some(v) => v,
                    None => return false,
                };
                let contains = list.contains(target);
                let expected = if contains { 1 } else { 0 };
                expected == out
            });
            if all_match {
                return Some(DetectedPattern::Contains {
                    target: *target,
                    list_input: list_idx,
                });
            }
        }
    }
    None
}

/// Detect conditional map: map(|x| if x==old then new else x, list).
/// Takes 3 inputs: list, old_value, new_value. Output is list with substitutions.
fn detect_conditional_map(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 {
        return None;
    }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 3 {
        return None;
    }

    for list_idx in 0..num_inputs {
        for old_idx in 0..num_inputs {
            for new_idx in 0..num_inputs {
                if list_idx == old_idx || list_idx == new_idx || old_idx == new_idx {
                    continue;
                }

                let all_match = test_cases.iter().all(|tc| {
                    let input_list = match as_int_list(&tc.inputs[list_idx]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let old_val = match as_int(&tc.inputs[old_idx]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let new_val = match as_int(&tc.inputs[new_idx]) {
                        Some(v) => v,
                        None => return false,
                    };
                    let output_list = match expected_output(tc).and_then(as_int_list) {
                        Some(v) => v,
                        None => return false,
                    };
                    if input_list.len() != output_list.len() {
                        return false;
                    }
                    input_list.iter().zip(output_list.iter()).all(|(&inp, &out)| {
                        if inp == old_val { out == new_val } else { out == inp }
                    })
                });
                if all_match {
                    return Some(DetectedPattern::ConditionalMap {
                        list_input: list_idx,
                        old_input: old_idx,
                        new_input: new_idx,
                    });
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Shared arithmetic ops table
// ---------------------------------------------------------------------------

fn arith_ops() -> &'static [(u8, fn(i64, i64) -> Option<i64>)] {
    &[
        (0x00, |a, b| Some(a.wrapping_add(b))),
        (0x01, |a, b| Some(a.wrapping_sub(b))),
        (0x02, |a, b| Some(a.wrapping_mul(b))),
        (0x03, |a, b| if b != 0 { Some(a / b) } else { None }),
        (0x04, |a, b| if b != 0 { Some(a % b) } else { None }),
        (0x07, |a, b| Some(a.min(b))),
        (0x08, |a, b| Some(a.max(b))),
    ]
}

// ---------------------------------------------------------------------------
// Graph construction helpers
// ---------------------------------------------------------------------------

fn int_type_env() -> (TypeEnv, TypeId) {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = iris_types::hash::compute_type_id(&int_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    (TypeEnv { types }, int_id)
}

fn make_unique_node(kind: NodeKind, payload: NodePayload, type_sig: TypeId, arity: u8) -> Node {
    let depth = (ANALYZER_COUNTER.fetch_add(1, Ordering::Relaxed) % 256) as u8;
    let mut node = Node {
        id: NodeId(0),
        kind,
        type_sig,
        cost: CostTerm::Unit,
        arity,
        resolution_depth: depth, salt: 0,
        payload,
    };
    node.id = compute_node_id(&node);
    node
}

fn compute_hash(nodes: &HashMap<NodeId, Node>, edges: &[Edge]) -> SemanticHash {
    let mut hasher = blake3::Hasher::new();
    let mut sorted_nids: Vec<_> = nodes.keys().collect();
    sorted_nids.sort();
    for nid in sorted_nids {
        hasher.update(&nid.0.to_le_bytes());
    }
    for edge in edges {
        hasher.update(&edge.source.0.to_le_bytes());
        hasher.update(&edge.target.0.to_le_bytes());
        hasher.update(&[edge.port, edge.label as u8]);
    }
    SemanticHash(*hasher.finalize().as_bytes())
}

fn graph_to_fragment(graph: SemanticGraph) -> Fragment {
    let boundary = Boundary {
        inputs: vec![],
        outputs: vec![(graph.root, graph.nodes[&graph.root].type_sig)],
    };
    let type_env = graph.type_env.clone();
    let mut fragment = Fragment {
        id: FragmentId([0; 32]),
        graph,
        boundary,
        type_env,
        imports: vec![],
        metadata: FragmentMeta {
            name: None,
            created_at: 0,
            generation: 0,
            lineage_hash: 0,
        },
        proof: None,
        contracts: Default::default(),    };
    fragment.id = compute_fragment_id(&fragment);
    fragment
}

fn incorporate_child(
    parent_nodes: &mut HashMap<NodeId, Node>,
    parent_edges: &mut Vec<Edge>,
    child: &SemanticGraph,
) -> NodeId {
    for (nid, node) in &child.nodes {
        parent_nodes.insert(*nid, node.clone());
    }
    parent_edges.extend(child.edges.iter().cloned());
    child.root
}

fn make_lit_int(value: i64, type_env: &TypeEnv, int_id: TypeId) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: value.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let root = node.id;
    nodes.insert(root, node);
    let hash = compute_hash(&nodes, &[]);
    SemanticGraph {
        root,
        nodes,
        edges: vec![],
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

fn make_lit_bool(value: bool, type_env: &TypeEnv, int_id: TypeId) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 4,
            value: vec![if value { 1 } else { 0 }],
        },
        int_id,
        0,
    );
    let root = node.id;
    nodes.insert(root, node);
    let hash = compute_hash(&nodes, &[]);
    SemanticGraph {
        root,
        nodes,
        edges: vec![],
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build an empty Tuple node (arity 0) that acts as an input placeholder.
/// The interpreter resolves this to positional input 0 for fold/map/unfold.
fn make_input_placeholder(type_env: &TypeEnv, int_id: TypeId) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let node = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let root = node.id;
    nodes.insert(root, node);
    let hash = compute_hash(&nodes, &[]);
    SemanticGraph {
        root,
        nodes,
        edges: vec![],
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build a Lit node with type_tag=0xFF that references positional input N.
/// The interpreter resolves this during eval_lit by looking up
/// BinderId(0xFFFF_0000 + index) from the environment.
fn make_input_ref(index: u8, type_env: &TypeEnv, int_id: TypeId) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0xFF,
            value: vec![index],
        },
        int_id,
        0,
    );
    let root = node.id;
    nodes.insert(root, node);
    let hash = compute_hash(&nodes, &[]);
    SemanticGraph {
        root,
        nodes,
        edges: vec![],
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

fn build_binary_prim(
    opcode: u8,
    a: &SemanticGraph,
    b: &SemanticGraph,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();
    let a_root = incorporate_child(&mut nodes, &mut edges, a);
    let b_root = incorporate_child(&mut nodes, &mut edges, b);
    let prim_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode },
        int_id,
        2,
    );
    let root = prim_node.id;
    nodes.insert(root, prim_node);
    edges.push(Edge {
        source: root,
        target: a_root,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: root,
        target: b_root,
        port: 1,
        label: EdgeLabel::Argument,
    });
    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

// ---------------------------------------------------------------------------
// Skeleton construction
// ---------------------------------------------------------------------------

fn build_skeleton_graph(
    pattern: &DetectedPattern,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> Option<SemanticGraph> {
    match pattern {
        DetectedPattern::Constant(val) => build_constant_skeleton(val, type_env, int_id),

        DetectedPattern::Identity { input_index } => {
            build_identity_skeleton(*input_index, type_env, int_id)
        }

        DetectedPattern::BinaryArithmetic {
            op,
            left_input,
            right_input,
        } => build_binary_arithmetic_skeleton(*op, *left_input, *right_input, type_env, int_id),

        DetectedPattern::ConditionalDispatch {
            dispatch_input,
            branches,
        } => build_conditional_dispatch_skeleton(*dispatch_input, branches, type_env, int_id),

        DetectedPattern::FoldReduction {
            base,
            op,
            input_index,
        } => Some(build_fold_skeleton(*base, *op, *input_index, type_env, int_id)),

        DetectedPattern::MapTransform { op, input_index } => {
            Some(build_map_skeleton(*op, *input_index, type_env, int_id))
        }

        DetectedPattern::MapFoldComposition {
            map_op,
            fold_op,
            fold_base,
            input_index,
        } => Some(build_map_fold_skeleton(
            *map_op, *fold_op, *fold_base, *input_index, type_env, int_id,
        )),

        DetectedPattern::Clamp { lo, hi } => {
            Some(build_clamp_skeleton(*lo, *hi, type_env, int_id))
        }

        DetectedPattern::Factorial => {
            Some(build_factorial_skeleton(type_env, int_id))
        }

        DetectedPattern::Power => {
            Some(build_power_skeleton(type_env, int_id))
        }

        DetectedPattern::ZipMapFold {
            pair_op,
            map_unary,
            fold_op,
            fold_base,
        } => Some(build_zip_map_fold_skeleton(
            *pair_op, *map_unary, *fold_op, *fold_base, type_env, int_id,
        )),

        DetectedPattern::Polynomial => {
            Some(build_polynomial_skeleton(type_env, int_id))
        }

        DetectedPattern::LinearSearch => {
            Some(build_linear_search_skeleton(type_env, int_id))
        }

        DetectedPattern::Fibonacci => {
            Some(build_fibonacci_skeleton(type_env, int_id))
        }

        DetectedPattern::TwoFoldScalar {
            fold1_base, fold1_op, fold2_base, fold2_op, combine_op, input_index,
        } => Some(build_two_fold_scalar_skeleton(
            *fold1_base, *fold1_op, *fold2_base, *fold2_op, *combine_op, *input_index, type_env, int_id,
        )),

        DetectedPattern::FoldThenMap {
            fold_base, fold_op, map_op, fold_first, input_index,
        } => Some(build_fold_then_map_skeleton(
            *fold_base, *fold_op, *map_op, *fold_first, *input_index, type_env, int_id,
        )),

        DetectedPattern::MapWithArg { op, list_input, arg_input } => {
            Some(build_map_with_arg_skeleton(*op, *list_input, *arg_input, type_env, int_id))
        }

        DetectedPattern::ConditionalCount { cmp_op, list_input, threshold_input } => {
            Some(build_conditional_count_skeleton(*cmp_op, *list_input, *threshold_input, type_env, int_id))
        }

        DetectedPattern::ZipMapFoldPlusBias { pair_op, fold_op, fold_base, bias_input } => {
            Some(build_zip_map_fold_plus_bias_skeleton(*pair_op, *fold_op, *fold_base, *bias_input, type_env, int_id))
        }

        DetectedPattern::FoldAfterTake { fold_base, fold_op, list_input, count_input } => {
            Some(build_fold_after_take_skeleton(*fold_base, *fold_op, *list_input, *count_input, type_env, int_id))
        }

        DetectedPattern::Scan { base, op, input_index } => {
            Some(build_scan_skeleton(*base, *op, *input_index, type_env, int_id))
        }

        DetectedPattern::BinaryToDecimal => {
            Some(build_binary_to_decimal_skeleton(type_env, int_id))
        }

        DetectedPattern::DigitSum => {
            Some(build_digit_sum_skeleton(type_env, int_id))
        }

        DetectedPattern::CountDivisors => {
            Some(build_count_divisors_skeleton(type_env, int_id))
        }

        DetectedPattern::IsPrime => {
            Some(build_is_prime_skeleton(type_env, int_id))
        }

        DetectedPattern::VarianceNumerator { input_index } => {
            Some(build_variance_numerator_skeleton(*input_index, type_env, int_id))
        }

        DetectedPattern::CollatzStep => {
            Some(build_collatz_step_skeleton(type_env, int_id))
        }

        // --- New v3 patterns ---
        DetectedPattern::UnaryAffine { a, b } => {
            Some(build_unary_affine_skeleton(*a, *b, type_env, int_id))
        }

        DetectedPattern::FoldThenMod { fold_base, fold_op, mod_val, input_index } => {
            Some(build_fold_then_mod_skeleton(*fold_base, *fold_op, *mod_val, *input_index, type_env, int_id))
        }

        DetectedPattern::ConcatElement { list_input, elem_input } => {
            Some(build_concat_element_skeleton(*list_input, *elem_input, type_env, int_id))
        }

        DetectedPattern::InitList { input_index } => {
            Some(build_init_list_skeleton(*input_index, type_env, int_id))
        }

        DetectedPattern::Repeat { elem_input, count_input } => {
            Some(build_repeat_skeleton(*elem_input, *count_input, type_env, int_id))
        }

        DetectedPattern::Range => {
            Some(build_range_skeleton(type_env, int_id))
        }

        DetectedPattern::ElementAccess { terms, input_index } => {
            Some(build_element_access_skeleton(terms, *input_index, type_env, int_id))
        }

        DetectedPattern::MatDet2x2 { input_index } => {
            Some(build_mat_det_2x2_skeleton(*input_index, type_env, int_id))
        }

        DetectedPattern::MatVecMul2x2 => {
            Some(build_mat_vec_mul_2x2_skeleton(type_env, int_id))
        }

        DetectedPattern::StateIncrement { key, delta } => {
            Some(build_state_increment_skeleton(key, *delta, type_env, int_id))
        }

        DetectedPattern::StateAccumulate { key, input_index } => {
            Some(build_state_accumulate_skeleton(key, *input_index, type_env, int_id))
        }

        DetectedPattern::TopKSum { k, input_index } => {
            Some(build_top_k_sum_skeleton(*k, *input_index, type_env, int_id))
        }

        DetectedPattern::MovingAvg2 { input_index } => {
            Some(build_moving_avg_2_skeleton(*input_index, type_env, int_id))
        }

        DetectedPattern::StartsWith => {
            Some(build_starts_with_skeleton(type_env, int_id))
        }

        DetectedPattern::Contains { target, list_input } => {
            Some(build_contains_skeleton(*target, *list_input, type_env, int_id))
        }

        DetectedPattern::ConditionalMap { list_input, old_input, new_input } => {
            Some(build_conditional_map_skeleton(*list_input, *old_input, *new_input, type_env, int_id))
        }

        // --- v5 patterns ---
        DetectedPattern::CountUnique { input_index } => {
            Some(build_count_unique_skeleton(*input_index, type_env, int_id))
        }
        DetectedPattern::ProgramDepth { input_index } => {
            Some(build_program_depth_skeleton(*input_index, type_env, int_id))
        }
        DetectedPattern::MostCommon { input_index } => {
            Some(build_most_common_skeleton(*input_index, type_env, int_id))
        }
        DetectedPattern::InsertAt { list_input, idx_input, val_input } => {
            Some(build_insert_at_skeleton(*list_input, *idx_input, *val_input, type_env, int_id))
        }
        DetectedPattern::DeleteAt { list_input, idx_input } => {
            Some(build_delete_at_skeleton(*list_input, *idx_input, type_env, int_id))
        }
        DetectedPattern::SwapElements { list_input, i_input, j_input } => {
            Some(build_swap_elements_skeleton(*list_input, *i_input, *j_input, type_env, int_id))
        }
        DetectedPattern::Interleave { a_input, b_input } => {
            Some(build_interleave_skeleton(*a_input, *b_input, type_env, int_id))
        }
        DetectedPattern::DoubleElements { input_index } => {
            Some(build_double_elements_skeleton(*input_index, type_env, int_id))
        }
        DetectedPattern::ZipWithIndex { input_index } => {
            Some(build_zip_with_index_skeleton(*input_index, type_env, int_id))
        }

        // --- v7 patterns ---
        DetectedPattern::ListElementBinaryOp { op, i, j, input_index } => {
            Some(build_list_element_binary_op_skeleton(*op, *i, *j, *input_index, type_env, int_id))
        }
        DetectedPattern::StridedCountEq { list_input, value_input } => {
            Some(build_strided_count_eq_skeleton(*list_input, *value_input, type_env, int_id))
        }
        DetectedPattern::StridedPairMatch { list_input, src_input, tgt_input } => {
            Some(build_strided_pair_match_skeleton(*list_input, *src_input, *tgt_input, type_env, int_id))
        }
        DetectedPattern::BubbleSortPass { input_index } => {
            Some(build_bubble_sort_pass_skeleton(*input_index, type_env, int_id))
        }
        DetectedPattern::DecodeRLE { input_index } => {
            Some(build_decode_rle_skeleton(*input_index, type_env, int_id))
        }
        DetectedPattern::EncodeRLE { input_index } => {
            Some(build_encode_rle_skeleton(*input_index, type_env, int_id))
        }

        DetectedPattern::Unknown => None,
    }
}

fn build_constant_skeleton(
    val: &Value,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> Option<SemanticGraph> {
    match val {
        Value::Int(n) => Some(make_lit_int(*n, type_env, int_id)),
        Value::Bool(b) => Some(make_lit_bool(*b, type_env, int_id)),
        Value::Tuple(elems) => {
            let mut nodes = HashMap::new();
            let mut edges = Vec::new();
            let mut child_ids = Vec::new();

            for elem in elems.iter() {
                let child_graph = build_constant_skeleton(elem, type_env, int_id)?;
                let child_root = incorporate_child(&mut nodes, &mut edges, &child_graph);
                child_ids.push(child_root);
            }

            let tuple_node = make_unique_node(
                NodeKind::Tuple,
                NodePayload::Tuple,
                int_id,
                child_ids.len() as u8,
            );
            let root = tuple_node.id;
            nodes.insert(root, tuple_node);

            for (port, &child_id) in child_ids.iter().enumerate() {
                edges.push(Edge {
                    source: root,
                    target: child_id,
                    port: port as u8,
                    label: EdgeLabel::Argument,
                });
            }

            let hash = compute_hash(&nodes, &edges);
            Some(SemanticGraph {
                root,
                nodes,
                edges,
                type_env: type_env.clone(),
                cost: CostBound::Unknown,
                resolution: Resolution::Implementation,
                hash,
            })
        }
        _ => None,
    }
}

/// Build an identity skeleton using fold(0, add, input).
///
/// The fold reads positional input 0 from BinderId(0xFFFF_0000). When
/// the input is a scalar Int(x), fold treats it as a single-element
/// collection [x] and computes 0 + x = x.
fn build_identity_skeleton(
    input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> Option<SemanticGraph> {
    Some(build_fold_skeleton(0, 0x00, input_index, type_env, int_id))
}

/// Build a skeleton with the correct opcode for binary arithmetic.
///
/// Uses input-reference Lit nodes (type_tag=0xFF) so the interpreter
/// can directly resolve positional inputs from Prim argument positions.
fn build_binary_arithmetic_skeleton(
    op: u8,
    left_input: usize,
    right_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> Option<SemanticGraph> {
    let left_ref = make_input_ref(left_input as u8, type_env, int_id);
    let right_ref = make_input_ref(right_input as u8, type_env, int_id);
    Some(build_binary_prim(op, &left_ref, &right_ref, type_env, int_id))
}

/// Build a Guard-based skeleton for conditional dispatch.
///
/// Uses input-reference Lit nodes (type_tag=0xFF) so the interpreter can
/// directly resolve positional inputs from Guard predicates and branch
/// bodies. Builds a chain of Guard nodes: for each branch, test whether
/// the dispatch input equals the branch value, and if so execute the
/// corresponding arithmetic operation.
fn build_conditional_dispatch_skeleton(
    dispatch_input: usize,
    branches: &[(i64, DetectedPattern)],
    type_env: &TypeEnv,
    int_id: TypeId,
) -> Option<SemanticGraph> {
    if branches.is_empty() {
        return None;
    }

    // Build sub-graphs for each branch body.
    let mut branch_graphs: Vec<(i64, SemanticGraph)> = Vec::new();
    for (val, pattern) in branches {
        let body = match pattern {
            DetectedPattern::BinaryArithmetic { op, left_input, right_input } => {
                let left_ref = make_input_ref(*left_input as u8, type_env, int_id);
                let right_ref = make_input_ref(*right_input as u8, type_env, int_id);
                build_binary_prim(*op, &left_ref, &right_ref, type_env, int_id)
            }
            _ => {
                // Fallback: identity on input 0
                make_input_ref(0, type_env, int_id)
            }
        };
        branch_graphs.push((*val, body));
    }

    // Build the Guard chain from the last branch backwards.
    // The final fallback is the last branch's body (unconditional).
    let (_, fallback_graph) = branch_graphs.pop().unwrap();

    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Incorporate the fallback body.
    let mut current_root = incorporate_child(&mut nodes, &mut edges, &fallback_graph);

    // Build Guard nodes from remaining branches in reverse order so the
    // first branch is checked first.
    for (val, body_graph) in branch_graphs.into_iter().rev() {
        // Predicate: Prim(eq, InputRef(dispatch_input), Lit(val))
        let dispatch_ref = make_input_ref(dispatch_input as u8, type_env, int_id);
        let lit_val = make_lit_int(val, type_env, int_id);
        let predicate = build_binary_prim(0x20, &dispatch_ref, &lit_val, type_env, int_id);
        let predicate_root = incorporate_child(&mut nodes, &mut edges, &predicate);

        // Body for this branch
        let body_root = incorporate_child(&mut nodes, &mut edges, &body_graph);

        // Guard node
        let guard_node = make_unique_node(
            NodeKind::Guard,
            NodePayload::Guard {
                predicate_node: predicate_root,
                body_node: body_root,
                fallback_node: current_root,
            },
            int_id,
            0,
        );
        current_root = guard_node.id;
        nodes.insert(guard_node.id, guard_node);
    }

    let hash = compute_hash(&nodes, &edges);
    Some(SemanticGraph {
        root: current_root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    })
}

/// Build fold(base, op, input).
///
/// The fold reads its collection from BinderId(0xFFFF_0000) (positional
/// input 0) when only 2 argument edges are present.
fn build_fold_skeleton(
    base: i64,
    op: u8,
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: base.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: op },
        int_id,
        2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![],
        },
        int_id,
        2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);

    edges.push(Edge {
        source: root,
        target: base_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: root,
        target: step_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build map(input, op).
///
/// Structure: Prim(0x30=map) with an empty Tuple placeholder (resolves
/// to positional input 0) and a Prim(op) step function.
fn build_map_skeleton(
    op: u8,
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let placeholder = make_input_placeholder(type_env, int_id);
    let placeholder_root = incorporate_child(&mut nodes, &mut edges, &placeholder);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: op },
        int_id,
        2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let map_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 },
        int_id,
        2,
    );
    let root = map_node.id;
    nodes.insert(root, map_node);

    edges.push(Edge {
        source: root,
        target: placeholder_root,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: root,
        target: step_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build fold(base, fold_op, map(input, map_op)).
fn build_map_fold_skeleton(
    map_op: u8,
    fold_op: u8,
    fold_base: i64,
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let map_graph = build_map_skeleton(map_op, 0, type_env, int_id);

    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: fold_base.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: fold_op },
        int_id,
        2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    let map_root = incorporate_child(&mut nodes, &mut edges, &map_graph);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![],
        },
        int_id,
        3,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);

    edges.push(Edge {
        source: root,
        target: base_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: root,
        target: fold_step_id,
        port: 1,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: root,
        target: map_root,
        port: 2,
        label: EdgeLabel::Argument,
    });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build Fibonacci skeleton: fold(0, max, unfold((1,1), add, n)).
///
/// The unfold with seed (1,1) and add step produces [1, 1, 2, 3, 5, 8, 13, ...].
/// Taking the max gives fib(n) because the sequence is non-decreasing for n>=1.
/// For n=0: unfold produces [], fold(0, max, []) = 0 = fib(0).
fn build_fibonacci_skeleton(
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Seed tuple (1, 1)
    let lit_1a = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let lit_1b = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let lit_1a_id = lit_1a.id;
    let lit_1b_id = lit_1b.id;
    nodes.insert(lit_1a_id, lit_1a);
    nodes.insert(lit_1b_id, lit_1b);

    let seed_tuple = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 2);
    let seed_id = seed_tuple.id;
    nodes.insert(seed_id, seed_tuple);
    edges.push(Edge { source: seed_id, target: lit_1a_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: seed_id, target: lit_1b_id, port: 1, label: EdgeLabel::Argument });

    // Step: add (Fibonacci: (a,b) -> emit a, state = (b, a+b))
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 }, // add
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // Input placeholder for iteration bound (scalar n)
    let input_ref = make_input_ref(0, type_env, int_id);
    let input_root = incorporate_child(&mut nodes, &mut edges, &input_ref);

    // No termination predicate
    let no_term = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let no_term_id = no_term.id;
    nodes.insert(no_term_id, no_term);

    // Unfold: generates [1, 1, 2, 3, 5, 8, 13, ...] (n elements)
    let unfold_node = make_unique_node(
        NodeKind::Unfold,
        NodePayload::Unfold { recursion_descriptor: vec![] },
        int_id, 4,
    );
    let unfold_id = unfold_node.id;
    nodes.insert(unfold_id, unfold_node);
    edges.push(Edge { source: unfold_id, target: seed_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: no_term_id, port: 2, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: input_root, port: 3, label: EdgeLabel::Argument });

    // Fold(0, max, unfold_result) to take the maximum = fib(n)
    let fold_base = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let fold_base_id = fold_base.id;
    nodes.insert(fold_base_id, fold_base);

    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x08 }, // max
        int_id, 2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id, 3,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge { source: fold_id, target: fold_base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: unfold_id, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build clamp(input, lo, hi) = max(lo, min(hi, input)).
///
/// Structure: Prim(max, Lit(lo), Prim(min, Lit(hi), InputRef(0)))
fn build_clamp_skeleton(
    lo: i64,
    hi: i64,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let input = make_input_ref(0, type_env, int_id);
    let hi_lit = make_lit_int(hi, type_env, int_id);
    let lo_lit = make_lit_int(lo, type_env, int_id);

    // min(hi, input)
    let min_graph = build_binary_prim(0x07, &hi_lit, &input, type_env, int_id);
    // max(lo, min(hi, input))
    build_binary_prim(0x08, &lo_lit, &min_graph, type_env, int_id)
}

/// Build factorial skeleton: fold(1, mul, unfold(1, max, no_term, n)).
///
/// The unfold with scalar seed=1 and max step produces [1, 2, 3, ..., n]
/// because scalar unfold does: output=max(state,state)=state, state+=1.
/// Then fold(1, mul, [1, 2, 3, ..., n]) = n!.
///
/// factorial(0) = fold(1, mul, []) = 1.
/// factorial(5) = fold(1, mul, [1,2,3,4,5]) = 120.
fn build_factorial_skeleton(
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Unfold seed: scalar 1 (start counting from 1)
    let seed_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let seed_id = seed_node.id;
    nodes.insert(seed_id, seed_node);

    // Step: max — for scalar state, output=max(state,state)=state, then state+=1.
    // This produces [1, 2, 3, 4, 5, ...].
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x08 }, // max
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // Input placeholder (scalar n, interpreted as iteration count)
    let input_ref = make_input_ref(0, type_env, int_id);
    let input_root = incorporate_child(&mut nodes, &mut edges, &input_ref);

    // No termination predicate
    let no_term = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let no_term_id = no_term.id;
    nodes.insert(no_term_id, no_term);

    // Unfold: generates [1, 2, 3, ..., n]
    let unfold_node = make_unique_node(
        NodeKind::Unfold,
        NodePayload::Unfold { recursion_descriptor: vec![] },
        int_id, 4,
    );
    let unfold_id = unfold_node.id;
    nodes.insert(unfold_id, unfold_node);
    edges.push(Edge { source: unfold_id, target: seed_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: no_term_id, port: 2, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: input_root, port: 3, label: EdgeLabel::Argument });

    // Fold(1, mul, unfold_result) to compute product
    let fold_base = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let fold_base_id = fold_base.id;
    nodes.insert(fold_base_id, fold_base);

    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x02 }, // mul
        int_id, 2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id, 3,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge { source: fold_id, target: fold_base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: unfold_id, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build power skeleton: fold(1, mul, repeat(x, n)).
///
/// Uses Unfold to generate [x, x, x, ...] (n times), then fold(1, mul).
fn build_power_skeleton(
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Seed: (x, x) — state that just repeats x
    // Actually, we need the base value x from input[0].
    // Unfold seed = InputRef(0) (the base)
    let input_x = make_input_ref(0, type_env, int_id);
    let input_x_root = incorporate_child(&mut nodes, &mut edges, &input_x);

    // Seed tuple: (x, x)
    let seed_tuple = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 2);
    let seed_id = seed_tuple.id;
    nodes.insert(seed_id, seed_tuple);
    // Duplicate input_x for second element
    let input_x2 = make_input_ref(0, type_env, int_id);
    let input_x2_root = incorporate_child(&mut nodes, &mut edges, &input_x2);
    edges.push(Edge { source: seed_id, target: input_x_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: seed_id, target: input_x2_root, port: 1, label: EdgeLabel::Argument });

    // Step: add (identity — keeps emitting x)
    // The unfold step for repeat just returns the same value.
    // With add step on (x, x) -> emits x, next = (x, x+x). Not right for repeat.
    // Actually let's use a different approach:
    // Unfold emits the first element of the state and the step produces next state.
    // For repeat(x, n), we want to emit x n times.
    // Use step = max (so (x,x) -> max(x,x)=x, keeps emitting x).
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x08 }, // max — identity for equal values
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // Input n (iteration count)
    let input_n = make_input_ref(1, type_env, int_id);
    let input_n_root = incorporate_child(&mut nodes, &mut edges, &input_n);

    // No termination
    let no_term = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let no_term_id = no_term.id;
    nodes.insert(no_term_id, no_term);

    // Unfold: generates [x, x, x, ...] n times
    let unfold_node = make_unique_node(
        NodeKind::Unfold,
        NodePayload::Unfold { recursion_descriptor: vec![] },
        int_id, 4,
    );
    let unfold_id = unfold_node.id;
    nodes.insert(unfold_id, unfold_node);
    edges.push(Edge { source: unfold_id, target: seed_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: no_term_id, port: 2, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: input_n_root, port: 3, label: EdgeLabel::Argument });

    // Fold(1, mul, unfold_result) to compute x^n
    let fold_base = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let fold_base_id = fold_base.id;
    nodes.insert(fold_base_id, fold_base);

    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x02 }, // mul
        int_id, 2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id, 3,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge { source: fold_id, target: fold_base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: unfold_id, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build zip(input0, input1) -> map(pair_op) -> fold(base, fold_op).
fn build_zip_map_fold_skeleton(
    pair_op: u8,
    map_unary: Option<u8>,
    fold_op: u8,
    fold_base: i64,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Input refs for two list inputs.
    let input0 = make_input_ref(0, type_env, int_id);
    let input0_root = incorporate_child(&mut nodes, &mut edges, &input0);

    let input1 = make_input_ref(1, type_env, int_id);
    let input1_root = incorporate_child(&mut nodes, &mut edges, &input1);

    // Zip node.
    let zip_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x32 }, // zip
        int_id, 2,
    );
    let zip_id = zip_node.id;
    nodes.insert(zip_id, zip_node);
    edges.push(Edge { source: zip_id, target: input0_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: zip_id, target: input1_root, port: 1, label: EdgeLabel::Argument });

    // Pair operation step.
    let pair_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: pair_op },
        int_id, 2,
    );
    let pair_step_id = pair_step.id;
    nodes.insert(pair_step_id, pair_step);

    // Map node: map(zipped, pair_op).
    let map_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 }, // map
        int_id, 2,
    );
    let map_id = map_node.id;
    nodes.insert(map_id, map_node);
    edges.push(Edge { source: map_id, target: zip_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: map_id, target: pair_step_id, port: 1, label: EdgeLabel::Argument });

    // If there's a unary map operation (e.g., abs), wrap in another map.
    let fold_collection_root = if let Some(unary_op) = map_unary {
        let unary_step = make_unique_node(
            NodeKind::Prim,
            NodePayload::Prim { opcode: unary_op },
            int_id, 2,
        );
        let unary_step_id = unary_step.id;
        nodes.insert(unary_step_id, unary_step);

        let map2_node = make_unique_node(
            NodeKind::Prim,
            NodePayload::Prim { opcode: 0x30 }, // map
            int_id, 2,
        );
        let map2_id = map2_node.id;
        nodes.insert(map2_id, map2_node);
        edges.push(Edge { source: map2_id, target: map_id, port: 0, label: EdgeLabel::Argument });
        edges.push(Edge { source: map2_id, target: unary_step_id, port: 1, label: EdgeLabel::Argument });
        map2_id
    } else {
        map_id
    };

    // Fold base.
    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: fold_base.to_le_bytes().to_vec(),
        },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    // Fold step.
    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: fold_op },
        int_id, 2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    // Fold node.
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id, 3,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge { source: fold_id, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: fold_collection_root, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build polynomial evaluation skeleton.
///
/// poly_eval(coeffs, x) = sum(coeffs[i] * x^i)
///
/// Uses unfold to generate [1, x, x^2, x^3, ...] (powers of x),
/// then zip with coefficients, map with mul, fold with add.
///
/// Simpler approach: Horner's method via fold.
/// poly_eval([c0, c1, c2], x) = c0 + x*(c1 + x*c2)
/// = fold over reversed coeffs: acc*x + coeff
/// But we can't easily reverse in the graph.
///
/// Alternative: fold with index tracking is complex.
/// Simplest: zip(coeffs, powers_of_x) -> map(mul) -> fold(0, add).
/// But generating powers_of_x requires unfold.
///
/// For simplicity, build a fold-based skeleton that approximates the
/// structure. Evolution can refine from there.
fn build_polynomial_skeleton(
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    // Build: fold over coefficients with accumulator tracking.
    // Horner's method: fold(0, |acc, c| acc * x + c, reversed_coeffs)
    // Since we can't reverse, use: fold(0, |acc, c| acc + c * x^i, coeffs)
    // which requires index tracking.
    //
    // Simplest working approach: build the zip+map+fold pattern.
    // Generate x^i via unfold, zip with coeffs, map(mul), fold(0, add).

    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Input refs
    let input_coeffs = make_input_ref(0, type_env, int_id);
    let coeffs_root = incorporate_child(&mut nodes, &mut edges, &input_coeffs);

    let input_x = make_input_ref(1, type_env, int_id);
    let x_root = incorporate_child(&mut nodes, &mut edges, &input_x);

    // Unfold seed: (1, x) — state is (current_power, x)
    // Emits current_power, next state = (current_power * x, x)
    let lit_1 = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let lit_1_id = lit_1.id;
    nodes.insert(lit_1_id, lit_1);

    let seed_tuple = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 2);
    let seed_id = seed_tuple.id;
    nodes.insert(seed_id, seed_tuple);
    edges.push(Edge { source: seed_id, target: lit_1_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: seed_id, target: x_root, port: 1, label: EdgeLabel::Argument });

    // Step: mul (current_power * x to get next power)
    let mul_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x02 }, // mul
        int_id, 2,
    );
    let mul_step_id = mul_step.id;
    nodes.insert(mul_step_id, mul_step);

    // No termination
    let no_term = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let no_term_id = no_term.id;
    nodes.insert(no_term_id, no_term);

    // Count: length of coefficients (use a fold to count)
    // Actually, we need the number of iterations = number of coefficients.
    // We can use a fold(0, add_one, coeffs) to count, but we don't have add_one.
    // Simpler: use the coefficients input directly as the iteration limiter.
    // Unfold with port 3 = coeffs_input (length determines iterations).

    // But we already used coeffs_root. Need another ref.
    let input_coeffs2 = make_input_ref(0, type_env, int_id);
    let coeffs2_root = incorporate_child(&mut nodes, &mut edges, &input_coeffs2);

    // Unfold: generates [1, x, x^2, ...] (len(coeffs) times)
    // Uses geometric mode (0x02) so tuple state (a,b) evolves as
    // (op(a,b), b) instead of Fibonacci's (b, op(a,b)).
    // With seed (1, x) and mul: 1, 1*x=x, x*x=x^2, x^2*x=x^3, ...
    let unfold_node = make_unique_node(
        NodeKind::Unfold,
        NodePayload::Unfold { recursion_descriptor: vec![0x02] },
        int_id, 4,
    );
    let unfold_id = unfold_node.id;
    nodes.insert(unfold_id, unfold_node);
    edges.push(Edge { source: unfold_id, target: seed_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: mul_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: no_term_id, port: 2, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: coeffs2_root, port: 3, label: EdgeLabel::Argument });

    // Zip(coeffs, powers)
    let zip_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x32 }, // zip
        int_id, 2,
    );
    let zip_id = zip_node.id;
    nodes.insert(zip_id, zip_node);
    edges.push(Edge { source: zip_id, target: coeffs_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: zip_id, target: unfold_id, port: 1, label: EdgeLabel::Argument });

    // Map(zip_result, mul) to compute coeff * x^i
    let pair_mul = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x02 }, // mul
        int_id, 2,
    );
    let pair_mul_id = pair_mul.id;
    nodes.insert(pair_mul_id, pair_mul);

    let map_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 }, // map
        int_id, 2,
    );
    let map_id = map_node.id;
    nodes.insert(map_id, map_node);
    edges.push(Edge { source: map_id, target: zip_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: map_id, target: pair_mul_id, port: 1, label: EdgeLabel::Argument });

    // Fold(0, add, mapped) to sum up
    let fold_base = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let fold_base_id = fold_base.id;
    nodes.insert(fold_base_id, fold_base);

    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 }, // add
        int_id, 2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id, 3,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge { source: fold_id, target: fold_base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: map_id, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build linear search skeleton.
///
/// linear_search(list, target) = index where list[index] == target, or -1.
///
/// Uses fold in search mode (recursion_descriptor = [0x03]):
///   fold(-1, eq, list, target)
/// The interpreter iterates over the list, comparing each element to the
/// target using eq. Returns the index of the first match, or -1 if none.
fn build_linear_search_skeleton(
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // InputRef(0) = list, InputRef(1) = target
    let input_list = make_input_ref(0, type_env, int_id);
    let list_root = incorporate_child(&mut nodes, &mut edges, &input_list);

    let input_target = make_input_ref(1, type_env, int_id);
    let target_root = incorporate_child(&mut nodes, &mut edges, &input_target);

    // Fold base: -1 (not found)
    let fold_base = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: (-1i64).to_le_bytes().to_vec() },
        int_id, 0,
    );
    let fold_base_id = fold_base.id;
    nodes.insert(fold_base_id, fold_base);

    // Step: eq — comparison operator for matching elements
    let eq_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x20 }, // eq
        int_id, 2,
    );
    let eq_step_id = eq_step.id;
    nodes.insert(eq_step_id, eq_step);

    // Fold node in search mode: fold(-1, eq, list, target)
    // Port 0: base (-1), Port 1: eq, Port 2: list, Port 3: target
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x03] },
        int_id, 4,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge { source: fold_id, target: fold_base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: eq_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: list_root, port: 2, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: target_root, port: 3, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

// ---------------------------------------------------------------------------
// New skeleton builders
// ---------------------------------------------------------------------------

/// Build two-fold-scalar skeleton: combine_op(fold1(list), fold2(list)).
/// For fold2_op == 0xFE (count/len), we use fold(0, count_step, list) where
/// count_step just adds 1 per element. We implement this as a fold with a
/// special recursion_descriptor [0x05] = "count mode".
fn build_two_fold_scalar_skeleton(
    fold1_base: i64,
    fold1_op: u8,
    fold2_base: i64,
    fold2_op: u8,
    combine_op: u8,
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Fold 1: fold(fold1_base, fold1_op, input)
    let f1_base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: fold1_base.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let f1_base_id = f1_base_node.id;
    nodes.insert(f1_base_id, f1_base_node);

    let f1_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: fold1_op },
        int_id, 2,
    );
    let f1_step_id = f1_step.id;
    nodes.insert(f1_step_id, f1_step);

    let fold1_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id, 2,
    );
    let fold1_id = fold1_node.id;
    nodes.insert(fold1_id, fold1_node);
    edges.push(Edge { source: fold1_id, target: f1_base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold1_id, target: f1_step_id, port: 1, label: EdgeLabel::Argument });

    // Fold 2: if fold2_op == 0xFE, use count mode; else normal fold
    let f2_base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: fold2_base.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let f2_base_id = f2_base_node.id;
    nodes.insert(f2_base_id, f2_base_node);

    let f2_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: if fold2_op == 0xFE { 0x00 } else { fold2_op } },
        int_id, 2,
    );
    let f2_step_id = f2_step.id;
    nodes.insert(f2_step_id, f2_step);

    let fold2_desc = if fold2_op == 0xFE { vec![0x05] } else { vec![] };
    let fold2_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: fold2_desc },
        int_id, 2,
    );
    let fold2_id = fold2_node.id;
    nodes.insert(fold2_id, fold2_node);
    edges.push(Edge { source: fold2_id, target: f2_base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold2_id, target: f2_step_id, port: 1, label: EdgeLabel::Argument });

    // Combine: combine_op(fold1, fold2)
    let combine_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: combine_op },
        int_id, 2,
    );
    let root = combine_node.id;
    nodes.insert(root, combine_node);
    edges.push(Edge { source: root, target: fold1_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: fold2_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build fold-then-map skeleton: map(|x| op(fold_val, x), list).
/// First computes fold_val = fold(base, fold_op, input), then maps with
/// map_op combining fold_val and each element.
///
/// Uses recursion_descriptor [0x06] on the outer Fold to signal "fold-then-map" mode:
/// the interpreter should compute the fold first, then map each element.
fn build_fold_then_map_skeleton(
    fold_base: i64,
    fold_op: u8,
    map_op: u8,
    fold_first: bool,
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Fold: compute the aggregate value (e.g., max, sum)
    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: fold_base.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: fold_op },
        int_id, 2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id, 2,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge { source: fold_id, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });

    // Map step: the binary op that combines fold_val with each element
    let map_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: map_op },
        int_id, 2,
    );
    let map_step_id = map_step.id;
    nodes.insert(map_step_id, map_step);

    // Fold-then-map composite node: uses recursion_descriptor [0x06] for fold-then-map,
    // or [0x07] for map-then-fold-first (fold_val comes first in the op).
    let desc_byte = if fold_first { 0x06 } else { 0x07 };
    let ftm_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![desc_byte] },
        int_id, 3,
    );
    let ftm_id = ftm_node.id;
    nodes.insert(ftm_id, ftm_node);
    // port 0: the fold sub-graph (computes the aggregate)
    edges.push(Edge { source: ftm_id, target: fold_id, port: 0, label: EdgeLabel::Argument });
    // port 1: the map step op
    edges.push(Edge { source: ftm_id, target: map_step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: ftm_id,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build map-with-arg skeleton: map(|x| x op arg, list).
/// Uses the fold-then-map infrastructure but with the "arg" coming from
/// a separate input instead of a fold.
///
/// Structure: Prim(map, InputRef(list_input), composed_step)
/// where composed_step applies op(x, InputRef(arg_input)).
///
/// We use recursion_descriptor [0x08] on a Fold node to signal
/// "map-with-external-arg" mode.
fn build_map_with_arg_skeleton(
    op: u8,
    list_input: usize,
    arg_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Input ref for the external argument
    let arg_ref = make_input_ref(arg_input as u8, type_env, int_id);
    let arg_root = incorporate_child(&mut nodes, &mut edges, &arg_ref);

    // Input ref for the list
    let list_ref = make_input_ref(list_input as u8, type_env, int_id);
    let list_root = incorporate_child(&mut nodes, &mut edges, &list_ref);

    // Map step: op
    let map_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: op },
        int_id, 2,
    );
    let map_step_id = map_step.id;
    nodes.insert(map_step_id, map_step);

    // Fold node in map-with-arg mode [0x08]:
    // port 0: arg value (the external argument)
    // port 1: map step op
    // port 2: collection (the list)
    let ftm_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x08] },
        int_id, 3,
    );
    let ftm_id = ftm_node.id;
    nodes.insert(ftm_id, ftm_node);
    edges.push(Edge { source: ftm_id, target: arg_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: ftm_id, target: map_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: ftm_id, target: list_root, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: ftm_id,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build conditional-count skeleton: count(x cmp_op threshold for x in list).
/// Uses recursion_descriptor [0x09] = conditional count mode.
/// port 0: threshold, port 1: cmp step, port 2: collection
fn build_conditional_count_skeleton(
    cmp_op: u8,
    list_input: usize,
    threshold_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Threshold input ref
    let thresh_ref = make_input_ref(threshold_input as u8, type_env, int_id);
    let thresh_root = incorporate_child(&mut nodes, &mut edges, &thresh_ref);

    // List input ref
    let list_ref = make_input_ref(list_input as u8, type_env, int_id);
    let list_root = incorporate_child(&mut nodes, &mut edges, &list_ref);

    // Comparison step
    let cmp_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: cmp_op },
        int_id, 2,
    );
    let cmp_step_id = cmp_step.id;
    nodes.insert(cmp_step_id, cmp_step);

    // Fold node in conditional count mode [0x09]:
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x09] },
        int_id, 3,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge { source: fold_id, target: thresh_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: cmp_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: list_root, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build zip_map_fold + bias skeleton: dot(a, b) + bias.
fn build_zip_map_fold_plus_bias_skeleton(
    pair_op: u8,
    fold_op: u8,
    fold_base: i64,
    bias_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    // Build the zip_map_fold part (for inputs 0 and 1, skipping bias)
    let (i0, i1) = if bias_input == 0 { (1, 2) } else if bias_input == 1 { (0, 2) } else { (0, 1) };

    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // We need to rebuild with correct input indices
    // Input refs for the two lists
    let input0 = make_input_ref(i0 as u8, type_env, int_id);
    let input0_root = incorporate_child(&mut nodes, &mut edges, &input0);
    let input1 = make_input_ref(i1 as u8, type_env, int_id);
    let input1_root = incorporate_child(&mut nodes, &mut edges, &input1);

    // Zip
    let zip_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x32 },
        int_id, 2,
    );
    let zip_id = zip_node.id;
    nodes.insert(zip_id, zip_node);
    edges.push(Edge { source: zip_id, target: input0_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: zip_id, target: input1_root, port: 1, label: EdgeLabel::Argument });

    // Map with pair_op
    let pair_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: pair_op },
        int_id, 2,
    );
    let pair_step_id = pair_step.id;
    nodes.insert(pair_step_id, pair_step);

    let map_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 },
        int_id, 2,
    );
    let map_id = map_node.id;
    nodes.insert(map_id, map_node);
    edges.push(Edge { source: map_id, target: zip_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: map_id, target: pair_step_id, port: 1, label: EdgeLabel::Argument });

    // Fold
    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: fold_base.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: fold_op },
        int_id, 2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id, 3,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge { source: fold_id, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: map_id, port: 2, label: EdgeLabel::Argument });

    // Add bias: fold_result + bias
    let bias_ref = make_input_ref(bias_input as u8, type_env, int_id);
    let bias_root = incorporate_child(&mut nodes, &mut edges, &bias_ref);

    let add_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 }, // add
        int_id, 2,
    );
    let root = add_node.id;
    nodes.insert(root, add_node);
    edges.push(Edge { source: root, target: fold_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: bias_root, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build fold-after-take skeleton: fold(base, op, take(list, n)).
fn build_fold_after_take_skeleton(
    fold_base: i64,
    fold_op: u8,
    list_input: usize,
    count_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Input refs
    let list_ref = make_input_ref(list_input as u8, type_env, int_id);
    let list_root = incorporate_child(&mut nodes, &mut edges, &list_ref);

    let count_ref = make_input_ref(count_input as u8, type_env, int_id);
    let count_root = incorporate_child(&mut nodes, &mut edges, &count_ref);

    // Take(list, n)
    let take_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x33 }, // take
        int_id, 2,
    );
    let take_id = take_node.id;
    nodes.insert(take_id, take_node);
    edges.push(Edge { source: take_id, target: list_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: take_id, target: count_root, port: 1, label: EdgeLabel::Argument });

    // Fold(base, op, take_result)
    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: fold_base.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: fold_op },
        int_id, 2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id, 3,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge { source: fold_id, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: take_id, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build scan skeleton using recursion_descriptor [0x04] = "scan mode".
/// Scan is like fold but emits intermediate accumulator values.
fn build_scan_skeleton(
    base: i64,
    op: u8,
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: base.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: op },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x04] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);

    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build binary-to-decimal skeleton: fold(|acc, bit| acc*2 + bit, 0, bits).
/// Uses recursion_descriptor [0x0A] = "fold with compound step" mode.
/// The step applies: acc = acc * 2 + element.
fn build_binary_to_decimal_skeleton(
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Base: 0
    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    // Multiplier: 2
    let mul_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 2i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let mul_id = mul_node.id;
    nodes.insert(mul_id, mul_node);

    // Fold with compound step [0x0A]:
    // port 0: base (0)
    // port 1: multiplier (2)
    // No port 2 = use positional input
    // Step: acc = acc * multiplier + element
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x0A] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: mul_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build digit_sum skeleton: unfold(n, mod10/div10) -> fold(0, add).
/// Uses unfold with recursion_descriptor [0x0B] = "digit extraction" mode.
fn build_digit_sum_skeleton(
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Input ref for n
    let input_n = make_input_ref(0, type_env, int_id);
    let input_root = incorporate_child(&mut nodes, &mut edges, &input_n);

    // Unfold: extracts digits using mod 10 / div 10.
    // recursion_descriptor [0x0B] signals digit extraction mode.
    // Seed = input, step = mod (opcode 0x04), uses 10 as divisor.
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x04 }, // mod
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let no_term = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let no_term_id = no_term.id;
    nodes.insert(no_term_id, no_term);

    let unfold_node = make_unique_node(
        NodeKind::Unfold,
        NodePayload::Unfold { recursion_descriptor: vec![0x0B] },
        int_id, 3,
    );
    let unfold_id = unfold_node.id;
    nodes.insert(unfold_id, unfold_node);
    edges.push(Edge { source: unfold_id, target: input_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: no_term_id, port: 2, label: EdgeLabel::Argument });

    // Fold(0, add, unfold_result)
    let fold_base = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let fold_base_id = fold_base.id;
    nodes.insert(fold_base_id, fold_base);

    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 }, // add
        int_id, 2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id, 3,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge { source: fold_id, target: fold_base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: unfold_id, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build count_divisors skeleton: fold(0, count_if_divides, unfold(1..=n)).
/// Uses unfold to generate [1, 2, ..., n], then a conditional count fold
/// that checks if n % i == 0.
/// Uses recursion_descriptor [0x0C] = "count divisors" mode.
fn build_count_divisors_skeleton(
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Input ref for n
    let input_n = make_input_ref(0, type_env, int_id);
    let input_root = incorporate_child(&mut nodes, &mut edges, &input_n);

    // Build unfold that generates [1, 2, ..., n]
    let seed_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let seed_id = seed_node.id;
    nodes.insert(seed_id, seed_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x08 }, // max (identity for scalar unfold)
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let no_term = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let no_term_id = no_term.id;
    nodes.insert(no_term_id, no_term);

    let unfold_node = make_unique_node(
        NodeKind::Unfold,
        NodePayload::Unfold { recursion_descriptor: vec![] },
        int_id, 4,
    );
    let unfold_id = unfold_node.id;
    nodes.insert(unfold_id, unfold_node);
    edges.push(Edge { source: unfold_id, target: seed_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: no_term_id, port: 2, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: input_root, port: 3, label: EdgeLabel::Argument });

    // Input ref for n (again, for the conditional count)
    let input_n2 = make_input_ref(0, type_env, int_id);
    let input_n2_root = incorporate_child(&mut nodes, &mut edges, &input_n2);

    // Mod step for divisibility check
    let mod_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x04 }, // mod
        int_id, 2,
    );
    let mod_step_id = mod_step.id;
    nodes.insert(mod_step_id, mod_step);

    // Fold in count-divisors mode [0x0C]:
    // port 0: n (the number to test divisibility against)
    // port 1: mod step
    // port 2: collection [1..=n]
    // Iterates: for each i in [1..=n], if n % i == 0 then count++
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x0C] },
        int_id, 3,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge { source: fold_id, target: input_n2_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: mod_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: unfold_id, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build is_prime skeleton: count_divisors(n) == 2 ? 1 : 0.
/// Wraps count_divisors in an eq comparison with 2.
fn build_is_prime_skeleton(
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let count_div_graph = build_count_divisors_skeleton(type_env, int_id);

    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let count_root = incorporate_child(&mut nodes, &mut edges, &count_div_graph);

    // Lit(2)
    let lit_2 = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 2i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let lit_2_id = lit_2.id;
    nodes.insert(lit_2_id, lit_2);

    // Eq(count_divisors, 2) -> Bool
    let eq_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x20 }, // eq
        int_id, 2,
    );
    let eq_id = eq_node.id;
    nodes.insert(eq_id, eq_node);
    edges.push(Edge { source: eq_id, target: count_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: eq_id, target: lit_2_id, port: 1, label: EdgeLabel::Argument });

    // bool_to_int(eq_result)
    let b2i_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x44 }, // bool_to_int
        int_id, 1,
    );
    let root = b2i_node.id;
    nodes.insert(root, b2i_node);
    edges.push(Edge { source: root, target: eq_id, port: 0, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build variance_numerator skeleton: sum((x - mean)^2).
///
/// Uses fold-then-map mode with a twist: first compute mean = sum/len,
/// then map(|x| (x - mean)^2), then fold(0, add).
///
/// Implemented using recursion_descriptor [0x0D] = "variance numerator" mode.
/// port 0: unused (base=0), port 1: step (add)
/// The interpreter computes: mean = sum/len, then sum((x-mean)^2).
fn build_variance_numerator_skeleton(
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 }, // add
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x0D] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build collatz_step skeleton: if x%2==0 then x/2 else 3*x+1.
///
/// Uses Guard nodes:
/// Guard(predicate=eq(mod(x,2),0), body=div(x,2), fallback=add(mul(3,x),1))
fn build_collatz_step_skeleton(
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Build predicate: eq(mod(InputRef(0), 2), 0)
    let pred_input = make_input_ref(0, type_env, int_id);
    let pred_lit2 = make_lit_int(2, type_env, int_id);
    let mod_node = build_binary_prim(0x04, &pred_input, &pred_lit2, type_env, int_id);
    let pred_lit0 = make_lit_int(0, type_env, int_id);
    let predicate = build_binary_prim(0x20, &mod_node, &pred_lit0, type_env, int_id);
    let predicate_root = incorporate_child(&mut nodes, &mut edges, &predicate);

    // Build body (even case): div(InputRef(0), 2) = x / 2
    let body_input = make_input_ref(0, type_env, int_id);
    let body_lit2 = make_lit_int(2, type_env, int_id);
    let body = build_binary_prim(0x03, &body_input, &body_lit2, type_env, int_id);
    let body_root = incorporate_child(&mut nodes, &mut edges, &body);

    // Build fallback (odd case): add(mul(3, InputRef(0)), 1) = 3*x + 1
    let fb_lit3 = make_lit_int(3, type_env, int_id);
    let fb_input = make_input_ref(0, type_env, int_id);
    let mul3x = build_binary_prim(0x02, &fb_lit3, &fb_input, type_env, int_id);
    let fb_lit1 = make_lit_int(1, type_env, int_id);
    let fallback = build_binary_prim(0x00, &mul3x, &fb_lit1, type_env, int_id);
    let fallback_root = incorporate_child(&mut nodes, &mut edges, &fallback);

    // Guard node
    let guard_node = make_unique_node(
        NodeKind::Guard,
        NodePayload::Guard {
            predicate_node: predicate_root,
            body_node: body_root,
            fallback_node: fallback_root,
        },
        int_id, 0,
    );
    let root = guard_node.id;
    nodes.insert(root, guard_node);

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

// ---------------------------------------------------------------------------
// New v3 skeleton builders
// ---------------------------------------------------------------------------

/// Build unary affine skeleton: output = a*input + b.
/// Structure: Prim(add, Prim(mul, InputRef(0), Lit(a)), Lit(b))
/// Special case: if a==1, just Prim(add, InputRef(0), Lit(b)).
fn build_unary_affine_skeleton(
    a: i64,
    b: i64,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let input = make_input_ref(0, type_env, int_id);
    if a == 1 {
        let lit_b = make_lit_int(b, type_env, int_id);
        build_binary_prim(0x00, &input, &lit_b, type_env, int_id) // add(input, b)
    } else if b == 0 {
        let lit_a = make_lit_int(a, type_env, int_id);
        build_binary_prim(0x02, &lit_a, &input, type_env, int_id) // mul(a, input)
    } else {
        let lit_a = make_lit_int(a, type_env, int_id);
        let mul_graph = build_binary_prim(0x02, &lit_a, &input, type_env, int_id);
        let lit_b = make_lit_int(b, type_env, int_id);
        build_binary_prim(0x00, &mul_graph, &lit_b, type_env, int_id) // add(mul(a, input), b)
    }
}

/// Build fold-then-mod skeleton: fold(base, op, list) mod constant.
/// Structure: Prim(mod, Fold(base, op, input), Lit(mod_val))
fn build_fold_then_mod_skeleton(
    fold_base: i64,
    fold_op: u8,
    mod_val: i64,
    input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let fold_graph = build_fold_skeleton(fold_base, fold_op, input_index, type_env, int_id);
    let lit_mod = make_lit_int(mod_val, type_env, int_id);
    build_binary_prim(0x04, &fold_graph, &lit_mod, type_env, int_id) // mod(fold_result, mod_val)
}

/// Build concat_element skeleton: concat(InputRef(list), Tuple(InputRef(elem))).
/// Structure: Prim(0x35=concat, InputRef(list), Tuple(InputRef(elem)))
fn build_concat_element_skeleton(
    list_input: usize,
    elem_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Input ref for list
    let list_ref = make_input_ref(list_input as u8, type_env, int_id);
    let list_root = incorporate_child(&mut nodes, &mut edges, &list_ref);

    // Input ref for element, wrapped in a single-element Tuple
    let elem_ref = make_input_ref(elem_input as u8, type_env, int_id);
    let elem_root = incorporate_child(&mut nodes, &mut edges, &elem_ref);

    let wrapper_tuple = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 1);
    let wrapper_id = wrapper_tuple.id;
    nodes.insert(wrapper_id, wrapper_tuple);
    edges.push(Edge { source: wrapper_id, target: elem_root, port: 0, label: EdgeLabel::Argument });

    // Concat node
    let concat_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x35 }, // concat
        int_id, 2,
    );
    let root = concat_node.id;
    nodes.insert(root, concat_node);
    edges.push(Edge { source: root, target: list_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: wrapper_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build init_list skeleton: take(list, fold(0, count, list) - 1).
/// We use recursion_descriptor [0x0E] = "init list mode" on a Fold.
/// The interpreter takes all but the last element.
fn build_init_list_skeleton(
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Placeholder for input
    let placeholder = make_input_placeholder(type_env, int_id);
    let placeholder_root = incorporate_child(&mut nodes, &mut edges, &placeholder);

    // Fold in init mode [0x0E]:
    // Takes the input collection and returns all elements except the last.
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 }, // add (unused, just needs a step)
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x0E] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: placeholder_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build repeat skeleton: unfold that emits x, n times.
/// Uses unfold with seed=(x,x), step=max (keeps emitting x), count=n.
fn build_repeat_skeleton(
    elem_input: usize,
    count_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Seed tuple: (x, x)
    let input_x1 = make_input_ref(elem_input as u8, type_env, int_id);
    let x1_root = incorporate_child(&mut nodes, &mut edges, &input_x1);
    let input_x2 = make_input_ref(elem_input as u8, type_env, int_id);
    let x2_root = incorporate_child(&mut nodes, &mut edges, &input_x2);

    let seed_tuple = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 2);
    let seed_id = seed_tuple.id;
    nodes.insert(seed_id, seed_tuple);
    edges.push(Edge { source: seed_id, target: x1_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: seed_id, target: x2_root, port: 1, label: EdgeLabel::Argument });

    // Step: max (keeps emitting the same value)
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x08 }, // max
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // No termination
    let no_term = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let no_term_id = no_term.id;
    nodes.insert(no_term_id, no_term);

    // Count: InputRef(count_input)
    let input_n = make_input_ref(count_input as u8, type_env, int_id);
    let n_root = incorporate_child(&mut nodes, &mut edges, &input_n);

    // Unfold
    let unfold_node = make_unique_node(
        NodeKind::Unfold,
        NodePayload::Unfold { recursion_descriptor: vec![] },
        int_id, 4,
    );
    let root = unfold_node.id;
    nodes.insert(root, unfold_node);
    edges.push(Edge { source: root, target: seed_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: no_term_id, port: 2, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: n_root, port: 3, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build range skeleton: unfold that emits counter, n times.
/// Uses unfold with scalar seed=0, step=max (identity for scalar), count=n.
/// Scalar unfold emits state then increments: 0, 1, 2, ..., n-1.
fn build_range_skeleton(
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Seed: 0
    let seed_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let seed_id = seed_node.id;
    nodes.insert(seed_id, seed_node);

    // Step: max (for scalar unfold: emit state, state += 1)
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x08 }, // max
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // No termination
    let no_term = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let no_term_id = no_term.id;
    nodes.insert(no_term_id, no_term);

    // Count: InputRef(0) = n
    let input_n = make_input_ref(0, type_env, int_id);
    let n_root = incorporate_child(&mut nodes, &mut edges, &input_n);

    // Unfold: generates [0, 1, 2, ..., n-1]
    let unfold_node = make_unique_node(
        NodeKind::Unfold,
        NodePayload::Unfold { recursion_descriptor: vec![] },
        int_id, 4,
    );
    let root = unfold_node.id;
    nodes.insert(root, unfold_node);
    edges.push(Edge { source: root, target: seed_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: no_term_id, port: 2, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: n_root, port: 3, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build element_access skeleton: output = sum(coeff_i * input[index_i]).
/// Uses Project nodes to extract elements.
fn build_element_access_skeleton(
    terms: &[(i64, usize)],
    input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    if terms.is_empty() {
        return make_lit_int(0, type_env, int_id);
    }

    // Build Project(input, index) for each term.
    let mut term_graphs: Vec<SemanticGraph> = Vec::new();
    for &(coeff, index) in terms {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();

        let input_copy = make_input_ref(input_index as u8, type_env, int_id);
        let input_root = incorporate_child(&mut nodes, &mut edges, &input_copy);

        let project_node = make_unique_node(
            NodeKind::Project,
            NodePayload::Project { field_index: index as u16 },
            int_id, 1,
        );
        let project_id = project_node.id;
        nodes.insert(project_id, project_node);
        edges.push(Edge { source: project_id, target: input_root, port: 0, label: EdgeLabel::Argument });

        let hash = compute_hash(&nodes, &edges);
        let project_graph = SemanticGraph {
            root: project_id,
            nodes,
            edges,
            type_env: type_env.clone(),
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash,
        };

        if coeff == 1 {
            term_graphs.push(project_graph);
        } else {
            let coeff_lit = make_lit_int(coeff, type_env, int_id);
            term_graphs.push(build_binary_prim(0x02, &coeff_lit, &project_graph, type_env, int_id));
        }
    }

    // Sum the term graphs.
    let mut result = term_graphs.remove(0);
    for term_graph in term_graphs {
        result = build_binary_prim(0x00, &result, &term_graph, type_env, int_id);
    }
    result
}

/// Build mat_det_2x2 skeleton: list[0]*list[3] - list[1]*list[2].
fn build_mat_det_2x2_skeleton(
    input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    // Build Project(input, 0), Project(input, 1), Project(input, 2), Project(input, 3)
    let mut projects: Vec<SemanticGraph> = Vec::new();
    for idx in 0u16..4u16 {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();

        let input_copy = make_input_ref(input_index as u8, type_env, int_id);
        let input_root = incorporate_child(&mut nodes, &mut edges, &input_copy);

        let project_node = make_unique_node(
            NodeKind::Project,
            NodePayload::Project { field_index: idx },
            int_id, 1,
        );
        let project_id = project_node.id;
        nodes.insert(project_id, project_node);
        edges.push(Edge { source: project_id, target: input_root, port: 0, label: EdgeLabel::Argument });

        let hash = compute_hash(&nodes, &edges);
        projects.push(SemanticGraph {
            root: project_id,
            nodes,
            edges,
            type_env: type_env.clone(),
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash,
        });
    }

    // ad = projects[0] * projects[3]
    let ad = build_binary_prim(0x02, &projects[0], &projects[3], type_env, int_id);
    // bc = projects[1] * projects[2]
    let bc = build_binary_prim(0x02, &projects[1], &projects[2], type_env, int_id);
    // ad - bc
    build_binary_prim(0x01, &ad, &bc, type_env, int_id)
}

/// Build mat_vec_mul_2x2 skeleton: [a*x+b*y, c*x+d*y].
/// Uses recursion_descriptor [0x0F] = "mat_vec_mul_2x2" mode on a Fold.
fn build_mat_vec_mul_2x2_skeleton(
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // InputRef(0) = matrix, InputRef(1) = vector
    let input_mat = make_input_ref(0, type_env, int_id);
    let mat_root = incorporate_child(&mut nodes, &mut edges, &input_mat);

    let input_vec = make_input_ref(1, type_env, int_id);
    let vec_root = incorporate_child(&mut nodes, &mut edges, &input_vec);

    // Fold in mat_vec_mul mode [0x0F]:
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x02 }, // mul (used in computation)
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x0F] },
        int_id, 3,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: mat_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: vec_root, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build state_increment skeleton: state[key] += delta.
/// Uses recursion_descriptor [0x10] = "state increment" mode on a Fold.
fn build_state_increment_skeleton(
    key: &str,
    delta: i64,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Encode key as a Lit with type_tag=0x02 (Bytes)
    let key_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0x05, value: key.as_bytes().to_vec() },
        int_id, 0,
    );
    let key_id = key_node.id;
    nodes.insert(key_id, key_node);

    // Delta value
    let delta_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: delta.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let delta_id = delta_node.id;
    nodes.insert(delta_id, delta_node);

    // Fold in state_increment mode [0x10]:
    // port 0: key, port 1: delta
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x10] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: key_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: delta_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build state_accumulate skeleton: state[key] += input, output = new state[key].
/// Uses recursion_descriptor [0x11] = "state accumulate" mode.
fn build_state_accumulate_skeleton(
    key: &str,
    input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Key
    let key_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0x05, value: key.as_bytes().to_vec() },
        int_id, 0,
    );
    let key_id = key_node.id;
    nodes.insert(key_id, key_node);

    // InputRef for the value to accumulate
    let input_ref = make_input_ref(input_index as u8, type_env, int_id);
    let input_root = incorporate_child(&mut nodes, &mut edges, &input_ref);

    // Fold in state_accumulate mode [0x11]:
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x11] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: key_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: input_root, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build top_k_sum skeleton: sort descending, take k, fold(0, add).
/// Uses recursion_descriptor [0x12] = "top k sum" mode.
fn build_top_k_sum_skeleton(
    k: usize,
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // k value
    let k_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: (k as i64).to_le_bytes().to_vec() },
        int_id, 0,
    );
    let k_id = k_node.id;
    nodes.insert(k_id, k_node);

    // Step (add)
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // Fold in top_k_sum mode [0x12]:
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x12] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: k_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build moving_avg_2 skeleton: output[i] = (input[i] + input[i+1]) / 2.
/// Uses recursion_descriptor [0x13] = "moving average" mode on a Fold.
fn build_moving_avg_2_skeleton(
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Base (unused)
    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    // Step (add, used in computation)
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // Fold in moving_avg mode [0x13]:
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x13] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build starts_with skeleton: 1 if list A starts with prefix B, else 0.
/// Uses recursion_descriptor [0x14] = "starts_with" mode on a Fold.
fn build_starts_with_skeleton(
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // InputRef(0) = list, InputRef(1) = prefix
    let input_list = make_input_ref(0, type_env, int_id);
    let list_root = incorporate_child(&mut nodes, &mut edges, &input_list);

    let input_prefix = make_input_ref(1, type_env, int_id);
    let prefix_root = incorporate_child(&mut nodes, &mut edges, &input_prefix);

    // Step (eq, for comparison)
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x20 }, // eq
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // Fold in starts_with mode [0x14]:
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x14] },
        int_id, 3,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: list_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: prefix_root, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build contains skeleton: check if list contains target value.
/// Uses recursion_descriptor [0x15] = contains mode.
/// port 0: target (Lit), port 1: cmp step (eq), port 2: collection
fn build_contains_skeleton(
    target: i64,
    list_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Target value as Lit
    let target_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: target.to_le_bytes().to_vec(),
        },
        int_id, 0,
    );
    let target_id = target_node.id;
    nodes.insert(target_id, target_node);

    // List input ref
    let list_ref = make_input_ref(list_input as u8, type_env, int_id);
    let list_root = incorporate_child(&mut nodes, &mut edges, &list_ref);

    // Comparison step (eq)
    let cmp_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x20 }, // eq
        int_id, 2,
    );
    let cmp_step_id = cmp_step.id;
    nodes.insert(cmp_step_id, cmp_step);

    // Fold node in contains mode [0x15]:
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x15] },
        int_id, 3,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: target_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: cmp_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: list_root, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build conditional_map skeleton: map(|x| if x==old then new else x, list).
/// Uses recursion_descriptor [0x16] = conditional_map mode.
/// port 0: old value, port 1: cmp step (eq), port 2: collection, port 3: new value
fn build_conditional_map_skeleton(
    list_input: usize,
    old_input: usize,
    new_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Old value input ref
    let old_ref = make_input_ref(old_input as u8, type_env, int_id);
    let old_root = incorporate_child(&mut nodes, &mut edges, &old_ref);

    // List input ref
    let list_ref = make_input_ref(list_input as u8, type_env, int_id);
    let list_root = incorporate_child(&mut nodes, &mut edges, &list_ref);

    // New value input ref
    let new_ref = make_input_ref(new_input as u8, type_env, int_id);
    let new_root = incorporate_child(&mut nodes, &mut edges, &new_ref);

    // Comparison step (eq)
    let cmp_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x20 }, // eq
        int_id, 2,
    );
    let cmp_step_id = cmp_step.id;
    nodes.insert(cmp_step_id, cmp_step);

    // Fold node in conditional_map mode [0x16]:
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x16] },
        int_id, 4,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: old_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: cmp_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: list_root, port: 2, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: new_root, port: 3, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

// ---------------------------------------------------------------------------
// v5 detectors
// ---------------------------------------------------------------------------

/// Detect count_unique: number of distinct values in a list.
fn detect_count_unique(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() { return None; }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_)) { continue; }
        if !matches!(expected_output(test_cases[0]), Some(Value::Int(_))) { continue; }

        let all_match = test_cases.iter().all(|tc| {
            let list = match as_int_list(&tc.inputs[input_idx]) {
                Some(v) => v, None => return false,
            };
            let out = match expected_output(tc).and_then(as_int) {
                Some(v) => v, None => return false,
            };
            let mut seen = std::collections::BTreeSet::new();
            for &v in &list { seen.insert(v); }
            seen.len() as i64 == out
        });
        if all_match {
            return Some(DetectedPattern::CountUnique { input_index: input_idx });
        }
    }
    None
}

/// Detect program_depth: longest consecutive run of non-zero values.
fn detect_program_depth(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() { return None; }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_)) { continue; }
        if !matches!(expected_output(test_cases[0]), Some(Value::Int(_))) { continue; }

        let all_match = test_cases.iter().all(|tc| {
            let list = match as_int_list(&tc.inputs[input_idx]) {
                Some(v) => v, None => return false,
            };
            let out = match expected_output(tc).and_then(as_int) {
                Some(v) => v, None => return false,
            };
            let mut max_run = 0i64;
            let mut cur_run = 0i64;
            for &v in &list {
                if v != 0 { cur_run += 1; if cur_run > max_run { max_run = cur_run; } }
                else { cur_run = 0; }
            }
            max_run == out
        });
        if all_match {
            return Some(DetectedPattern::ProgramDepth { input_index: input_idx });
        }
    }
    None
}

/// Detect most_common: mode (most frequent element) of a list.
fn detect_most_common(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() { return None; }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_)) { continue; }
        if !matches!(expected_output(test_cases[0]), Some(Value::Int(_))) { continue; }

        let all_match = test_cases.iter().all(|tc| {
            let list = match as_int_list(&tc.inputs[input_idx]) {
                Some(v) => v, None => return false,
            };
            let out = match expected_output(tc).and_then(as_int) {
                Some(v) => v, None => return false,
            };
            if list.is_empty() { return false; }
            let mut freq = std::collections::BTreeMap::new();
            for &v in &list { *freq.entry(v).or_insert(0i64) += 1; }
            let max_freq = freq.values().copied().max().unwrap_or(0);
            // Find the value with max frequency (first in iteration order for ties)
            let mode = freq.iter()
                .filter(|&(_, &count)| count == max_freq)
                .map(|(&val, _)| val)
                .next()
                .unwrap();
            mode == out
        });
        if all_match {
            return Some(DetectedPattern::MostCommon { input_index: input_idx });
        }
    }
    None
}

/// Detect insert_at: insert value at index position in list.
fn detect_insert_at(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() { return None; }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 3 { return None; }

    for list_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[list_idx], Value::Tuple(_)) { continue; }
        for idx_idx in 0..num_inputs {
            if idx_idx == list_idx { continue; }
            if !matches!(&test_cases[0].inputs[idx_idx], Value::Int(_)) { continue; }
            for val_idx in 0..num_inputs {
                if val_idx == list_idx || val_idx == idx_idx { continue; }
                if !matches!(&test_cases[0].inputs[val_idx], Value::Int(_)) { continue; }

                let all_match = test_cases.iter().all(|tc| {
                    let list = match as_int_list(&tc.inputs[list_idx]) {
                        Some(v) => v, None => return false,
                    };
                    let idx = match as_int(&tc.inputs[idx_idx]) {
                        Some(v) if v >= 0 => v as usize, _ => return false,
                    };
                    let val = match as_int(&tc.inputs[val_idx]) {
                        Some(v) => v, None => return false,
                    };
                    let out = match expected_output(tc).and_then(as_int_list) {
                        Some(v) => v, None => return false,
                    };
                    if idx > list.len() { return false; }
                    let mut expected = list.clone();
                    expected.insert(idx, val);
                    expected == out
                });
                if all_match {
                    return Some(DetectedPattern::InsertAt {
                        list_input: list_idx, idx_input: idx_idx, val_input: val_idx,
                    });
                }
            }
        }
    }
    None
}

/// Detect delete_at: remove element at index from list.
fn detect_delete_at(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() { return None; }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 2 { return None; }

    for list_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[list_idx], Value::Tuple(_)) { continue; }
        for idx_idx in 0..num_inputs {
            if idx_idx == list_idx { continue; }
            if !matches!(&test_cases[0].inputs[idx_idx], Value::Int(_)) { continue; }

            let all_match = test_cases.iter().all(|tc| {
                let list = match as_int_list(&tc.inputs[list_idx]) {
                    Some(v) => v, None => return false,
                };
                let idx = match as_int(&tc.inputs[idx_idx]) {
                    Some(v) if v >= 0 => v as usize, _ => return false,
                };
                let out = match expected_output(tc).and_then(as_int_list) {
                    Some(v) => v, None => return false,
                };
                if idx >= list.len() { return false; }
                let mut expected = list.clone();
                expected.remove(idx);
                expected == out
            });
            if all_match {
                return Some(DetectedPattern::DeleteAt {
                    list_input: list_idx, idx_input: idx_idx,
                });
            }
        }
    }
    None
}

/// Detect swap_elements: swap elements at two positions in a list.
fn detect_swap_elements(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() { return None; }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 3 { return None; }

    for list_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[list_idx], Value::Tuple(_)) { continue; }
        for i_idx in 0..num_inputs {
            if i_idx == list_idx { continue; }
            if !matches!(&test_cases[0].inputs[i_idx], Value::Int(_)) { continue; }
            for j_idx in 0..num_inputs {
                if j_idx == list_idx || j_idx == i_idx { continue; }
                if !matches!(&test_cases[0].inputs[j_idx], Value::Int(_)) { continue; }

                let all_match = test_cases.iter().all(|tc| {
                    let list = match as_int_list(&tc.inputs[list_idx]) {
                        Some(v) => v, None => return false,
                    };
                    let i = match as_int(&tc.inputs[i_idx]) {
                        Some(v) if v >= 0 => v as usize, _ => return false,
                    };
                    let j = match as_int(&tc.inputs[j_idx]) {
                        Some(v) if v >= 0 => v as usize, _ => return false,
                    };
                    let out = match expected_output(tc).and_then(as_int_list) {
                        Some(v) => v, None => return false,
                    };
                    if i >= list.len() || j >= list.len() { return false; }
                    let mut expected = list.clone();
                    expected.swap(i, j);
                    expected == out
                });
                if all_match {
                    return Some(DetectedPattern::SwapElements {
                        list_input: list_idx, i_input: i_idx, j_input: j_idx,
                    });
                }
            }
        }
    }
    None
}

/// Detect interleave: merge two lists alternately.
fn detect_interleave(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() { return None; }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 2 { return None; }

    for a_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[a_idx], Value::Tuple(_)) { continue; }
        for b_idx in 0..num_inputs {
            if b_idx == a_idx { continue; }
            if !matches!(&test_cases[0].inputs[b_idx], Value::Tuple(_)) { continue; }

            let all_match = test_cases.iter().all(|tc| {
                let a = match as_int_list(&tc.inputs[a_idx]) {
                    Some(v) => v, None => return false,
                };
                let b = match as_int_list(&tc.inputs[b_idx]) {
                    Some(v) => v, None => return false,
                };
                let out = match expected_output(tc).and_then(as_int_list) {
                    Some(v) => v, None => return false,
                };
                if a.len() != b.len() { return false; }
                let mut expected = Vec::with_capacity(a.len() + b.len());
                for i in 0..a.len() {
                    expected.push(a[i]);
                    expected.push(b[i]);
                }
                expected == out
            });
            if all_match {
                return Some(DetectedPattern::Interleave {
                    a_input: a_idx, b_input: b_idx,
                });
            }
        }
    }
    None
}

/// Detect double_elements: each element appears twice.
fn detect_double_elements(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() { return None; }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_)) { continue; }

        let all_match = test_cases.iter().all(|tc| {
            let list = match as_int_list(&tc.inputs[input_idx]) {
                Some(v) => v, None => return false,
            };
            let out = match expected_output(tc).and_then(as_int_list) {
                Some(v) => v, None => return false,
            };
            if out.len() != list.len() * 2 { return false; }
            for (i, &v) in list.iter().enumerate() {
                if out[2 * i] != v || out[2 * i + 1] != v { return false; }
            }
            true
        });
        if all_match {
            return Some(DetectedPattern::DoubleElements { input_index: input_idx });
        }
    }
    None
}

/// Detect zip_with_index: pair each element with its index (flattened).
fn detect_zip_with_index(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.is_empty() { return None; }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_)) { continue; }

        let all_match = test_cases.iter().all(|tc| {
            let list = match as_int_list(&tc.inputs[input_idx]) {
                Some(v) => v, None => return false,
            };
            let out = match expected_output(tc).and_then(as_int_list) {
                Some(v) => v, None => return false,
            };
            if out.len() != list.len() * 2 { return false; }
            for (i, &v) in list.iter().enumerate() {
                if out[2 * i] != i as i64 || out[2 * i + 1] != v { return false; }
            }
            true
        });
        if all_match {
            return Some(DetectedPattern::ZipWithIndex { input_index: input_idx });
        }
    }
    None
}

// ---------------------------------------------------------------------------
// v7 detectors
// ---------------------------------------------------------------------------

/// Detect list element binary op: output = list[i] op list[j].
/// E.g., eval_rpn([a, b, 1]) → a - b.
fn detect_list_element_binary_op(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 { return None; }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_)) { continue; }
        let first_out_scalar = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Int(_)))
            .unwrap_or(false);
        if !first_out_scalar { continue; }

        let list_len = match as_int_list(&test_cases[0].inputs[input_idx]) {
            Some(v) => v.len(), None => continue,
        };
        if list_len < 2 || list_len > 16 { continue; }

        // Try all pairs (i, j) with i != j.
        let ops: [(u8, fn(i64, i64) -> i64); 5] = [
            (0x00, |a, b| a.wrapping_add(b)),
            (0x01, |a, b| a.wrapping_sub(b)),
            (0x02, |a, b| a.wrapping_mul(b)),
            (0x07, |a, b| a.min(b)),
            (0x08, |a, b| a.max(b)),
        ];

        for i in 0..list_len {
            for j in 0..list_len {
                if i == j { continue; }
                for &(opcode, op_fn) in &ops {
                    let all_match = test_cases.iter().all(|tc| {
                        let list = match as_int_list(&tc.inputs[input_idx]) {
                            Some(v) => v, None => return false,
                        };
                        let out = match expected_output(tc).and_then(as_int) {
                            Some(v) => v, None => return false,
                        };
                        list.len() > i.max(j) && op_fn(list[i], list[j]) == out
                    });
                    if all_match {
                        return Some(DetectedPattern::ListElementBinaryOp {
                            op: opcode, i, j, input_index: input_idx,
                        });
                    }
                }
            }
        }
    }
    None
}

/// Detect strided count: count occurrences of value at even positions.
/// E.g., out_degree([0,1,0,2,1,2], 0) → 2.
fn detect_strided_count_eq(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 { return None; }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 2 { return None; }

    for list_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[list_idx], Value::Tuple(_)) { continue; }
        for val_idx in 0..num_inputs {
            if val_idx == list_idx { continue; }
            if !matches!(&test_cases[0].inputs[val_idx], Value::Int(_)) { continue; }

            let all_match = test_cases.iter().all(|tc| {
                let list = match as_int_list(&tc.inputs[list_idx]) {
                    Some(v) => v, None => return false,
                };
                let target = match as_int(&tc.inputs[val_idx]) {
                    Some(v) => v, None => return false,
                };
                let out = match expected_output(tc).and_then(as_int) {
                    Some(v) => v, None => return false,
                };
                // Count target at even positions (stride 2).
                let count = list.iter().step_by(2).filter(|&&x| x == target).count() as i64;
                count == out
            });
            if all_match {
                return Some(DetectedPattern::StridedCountEq {
                    list_input: list_idx, value_input: val_idx,
                });
            }
        }
    }
    None
}

/// Detect strided pair match: check if (src, tgt) pair exists at stride-2 positions.
/// E.g., has_edge([0,1,0,2,1,2], 0, 2) → 1.
fn detect_strided_pair_match(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 { return None; }
    let num_inputs = test_cases[0].inputs.len();
    if num_inputs < 3 { return None; }

    for list_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[list_idx], Value::Tuple(_)) { continue; }

        for src_idx in 0..num_inputs {
            if src_idx == list_idx { continue; }
            if !matches!(&test_cases[0].inputs[src_idx], Value::Int(_)) { continue; }

            for tgt_idx in 0..num_inputs {
                if tgt_idx == list_idx || tgt_idx == src_idx { continue; }
                if !matches!(&test_cases[0].inputs[tgt_idx], Value::Int(_)) { continue; }

                let all_match = test_cases.iter().all(|tc| {
                    let list = match as_int_list(&tc.inputs[list_idx]) {
                        Some(v) => v, None => return false,
                    };
                    let src = match as_int(&tc.inputs[src_idx]) {
                        Some(v) => v, None => return false,
                    };
                    let tgt = match as_int(&tc.inputs[tgt_idx]) {
                        Some(v) => v, None => return false,
                    };
                    let out = match expected_output(tc).and_then(as_int) {
                        Some(v) => v, None => return false,
                    };
                    // Check if (src, tgt) pair exists at stride 2.
                    let found = list.chunks(2).any(|pair| pair.len() == 2 && pair[0] == src && pair[1] == tgt);
                    let expected = if found { 1 } else { 0 };
                    expected == out
                });
                if all_match {
                    return Some(DetectedPattern::StridedPairMatch {
                        list_input: list_idx,
                        src_input: src_idx,
                        tgt_input: tgt_idx,
                    });
                }
            }
        }
    }
    None
}

/// Detect bubble sort pass: one pass of pairwise swap-if-unordered.
/// E.g., [3,1,4,1,5] → [1,3,1,4,5].
fn detect_bubble_sort_pass(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 { return None; }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_)) { continue; }
        let first_out_is_list = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Tuple(_)))
            .unwrap_or(false);
        if !first_out_is_list { continue; }

        let all_match = test_cases.iter().all(|tc| {
            let list = match as_int_list(&tc.inputs[input_idx]) {
                Some(v) => v, None => return false,
            };
            let out = match expected_output(tc).and_then(as_int_list) {
                Some(v) => v, None => return false,
            };
            if list.len() != out.len() { return false; }
            // Simulate one pass of bubble sort.
            let mut sim = list.clone();
            for i in 0..sim.len().saturating_sub(1) {
                if sim[i] > sim[i + 1] {
                    sim.swap(i, i + 1);
                }
            }
            sim == out
        });
        if all_match {
            return Some(DetectedPattern::BubbleSortPass { input_index: input_idx });
        }
    }
    None
}

/// Detect decode_rle: [val, count, val, count, ...] → expanded list.
fn detect_decode_rle(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 { return None; }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_)) { continue; }
        let first_out_is_list = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Tuple(_)))
            .unwrap_or(false);
        if !first_out_is_list { continue; }

        let all_match = test_cases.iter().all(|tc| {
            let list = match as_int_list(&tc.inputs[input_idx]) {
                Some(v) => v, None => return false,
            };
            let out = match expected_output(tc).and_then(as_int_list) {
                Some(v) => v, None => return false,
            };
            if list.len() % 2 != 0 { return false; }
            // Decode: pairs (val, count).
            let mut decoded = Vec::new();
            for pair in list.chunks(2) {
                if pair.len() != 2 { return false; }
                let val = pair[0];
                let count = pair[1];
                if count < 0 || count > 1000 { return false; }
                for _ in 0..count {
                    decoded.push(val);
                }
            }
            decoded == out
        });
        if all_match {
            return Some(DetectedPattern::DecodeRLE { input_index: input_idx });
        }
    }
    None
}

/// Detect encode_rle: consecutive equal elements → [val, count, ...].
fn detect_encode_rle(test_cases: &[&TestCase]) -> Option<DetectedPattern> {
    if test_cases.len() < 2 { return None; }
    let num_inputs = test_cases[0].inputs.len();

    for input_idx in 0..num_inputs {
        if !matches!(&test_cases[0].inputs[input_idx], Value::Tuple(_)) { continue; }
        let first_out_is_list = expected_output(test_cases[0])
            .map(|v| matches!(v, Value::Tuple(_)))
            .unwrap_or(false);
        if !first_out_is_list { continue; }

        let all_match = test_cases.iter().all(|tc| {
            let list = match as_int_list(&tc.inputs[input_idx]) {
                Some(v) => v, None => return false,
            };
            let out = match expected_output(tc).and_then(as_int_list) {
                Some(v) => v, None => return false,
            };
            if list.is_empty() { return out.is_empty(); }
            // Encode: group consecutive equal values.
            let mut encoded = Vec::new();
            let mut cur_val = list[0];
            let mut cur_count = 1i64;
            for &v in &list[1..] {
                if v == cur_val {
                    cur_count += 1;
                } else {
                    encoded.push(cur_val);
                    encoded.push(cur_count);
                    cur_val = v;
                    cur_count = 1;
                }
            }
            encoded.push(cur_val);
            encoded.push(cur_count);
            encoded == out
        });
        if all_match {
            return Some(DetectedPattern::EncodeRLE { input_index: input_idx });
        }
    }
    None
}

// ---------------------------------------------------------------------------
// v5 skeleton builders
// ---------------------------------------------------------------------------

/// Build count_unique skeleton: fold mode 0x17.
fn build_count_unique_skeleton(
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 }, // unused, fold mode handles it
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x17] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build program_depth skeleton: fold mode 0x18.
fn build_program_depth_skeleton(
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x18] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build most_common skeleton: fold mode 0x19.
fn build_most_common_skeleton(
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x19] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build insert_at skeleton: fold mode 0x1A.
/// port 0: idx, port 1: step (unused), port 2: list, port 3: val
fn build_insert_at_skeleton(
    list_input: usize,
    idx_input: usize,
    val_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let idx_ref = make_input_ref(idx_input as u8, type_env, int_id);
    let idx_root = incorporate_child(&mut nodes, &mut edges, &idx_ref);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let list_ref = make_input_ref(list_input as u8, type_env, int_id);
    let list_root = incorporate_child(&mut nodes, &mut edges, &list_ref);

    let val_ref = make_input_ref(val_input as u8, type_env, int_id);
    let val_root = incorporate_child(&mut nodes, &mut edges, &val_ref);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x1A] },
        int_id, 4,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: idx_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: list_root, port: 2, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: val_root, port: 3, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build delete_at skeleton: fold mode 0x1B.
/// port 0: idx, port 1: step (unused), port 2: list
fn build_delete_at_skeleton(
    list_input: usize,
    idx_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let idx_ref = make_input_ref(idx_input as u8, type_env, int_id);
    let idx_root = incorporate_child(&mut nodes, &mut edges, &idx_ref);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let list_ref = make_input_ref(list_input as u8, type_env, int_id);
    let list_root = incorporate_child(&mut nodes, &mut edges, &list_ref);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x1B] },
        int_id, 3,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: idx_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: list_root, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build swap_elements skeleton: fold mode 0x1C.
/// port 0: i, port 1: step (unused), port 2: list, port 3: j
fn build_swap_elements_skeleton(
    list_input: usize,
    i_input: usize,
    j_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let i_ref = make_input_ref(i_input as u8, type_env, int_id);
    let i_root = incorporate_child(&mut nodes, &mut edges, &i_ref);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let list_ref = make_input_ref(list_input as u8, type_env, int_id);
    let list_root = incorporate_child(&mut nodes, &mut edges, &list_ref);

    let j_ref = make_input_ref(j_input as u8, type_env, int_id);
    let j_root = incorporate_child(&mut nodes, &mut edges, &j_ref);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x1C] },
        int_id, 4,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: i_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: list_root, port: 2, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: j_root, port: 3, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build interleave skeleton: fold mode 0x1D.
/// port 0: list_a (via input ref), port 1: step (unused), port 2: list_b
fn build_interleave_skeleton(
    a_input: usize,
    b_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let a_ref = make_input_ref(a_input as u8, type_env, int_id);
    let a_root = incorporate_child(&mut nodes, &mut edges, &a_ref);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let b_ref = make_input_ref(b_input as u8, type_env, int_id);
    let b_root = incorporate_child(&mut nodes, &mut edges, &b_ref);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x1D] },
        int_id, 3,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: a_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: b_root, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build double_elements skeleton: fold mode 0x1E.
fn build_double_elements_skeleton(
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x1E] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build zip_with_index skeleton: fold mode 0x1F.
fn build_zip_with_index_skeleton(
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x1F] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

// ---------------------------------------------------------------------------
// v7 skeleton builders
// ---------------------------------------------------------------------------

/// Build list element binary op skeleton: Project(input, i) op Project(input, j).
fn build_list_element_binary_op_skeleton(
    op: u8,
    i: usize,
    j: usize,
    input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Project(input, i)
    let input_a = make_input_ref(input_index as u8, type_env, int_id);
    let input_a_root = incorporate_child(&mut nodes, &mut edges, &input_a);
    let proj_a = make_unique_node(
        NodeKind::Project,
        NodePayload::Project { field_index: i as u16 },
        int_id, 1,
    );
    let proj_a_id = proj_a.id;
    nodes.insert(proj_a_id, proj_a);
    edges.push(Edge { source: proj_a_id, target: input_a_root, port: 0, label: EdgeLabel::Argument });

    // Project(input, j)
    let input_b = make_input_ref(input_index as u8, type_env, int_id);
    let input_b_root = incorporate_child(&mut nodes, &mut edges, &input_b);
    let proj_b = make_unique_node(
        NodeKind::Project,
        NodePayload::Project { field_index: j as u16 },
        int_id, 1,
    );
    let proj_b_id = proj_b.id;
    nodes.insert(proj_b_id, proj_b);
    edges.push(Edge { source: proj_b_id, target: input_b_root, port: 0, label: EdgeLabel::Argument });

    // Prim(op, proj_a, proj_b)
    let prim_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: op },
        int_id, 2,
    );
    let root = prim_node.id;
    nodes.insert(root, prim_node);
    edges.push(Edge { source: root, target: proj_a_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: proj_b_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build strided count eq skeleton: fold mode 0x20.
/// port 0: target value (InputRef), port 1: step (eq), port 2: list (InputRef)
fn build_strided_count_eq_skeleton(
    list_input: usize,
    value_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // port 0: target value
    let val_ref = make_input_ref(value_input as u8, type_env, int_id);
    let val_root = incorporate_child(&mut nodes, &mut edges, &val_ref);

    // port 1: eq (comparison op)
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x20 }, // eq
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // port 2: list
    let list_ref = make_input_ref(list_input as u8, type_env, int_id);
    let list_root = incorporate_child(&mut nodes, &mut edges, &list_ref);

    // Fold with mode 0x20
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x20] },
        int_id, 3,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: val_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: list_root, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build strided pair match skeleton: fold mode 0x21.
/// port 0: src (InputRef), port 1: step (eq), port 2: list, port 3: tgt
fn build_strided_pair_match_skeleton(
    list_input: usize,
    src_input: usize,
    tgt_input: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // port 0: src
    let src_ref = make_input_ref(src_input as u8, type_env, int_id);
    let src_root = incorporate_child(&mut nodes, &mut edges, &src_ref);

    // port 1: eq
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x20 }, // eq
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // port 2: list
    let list_ref = make_input_ref(list_input as u8, type_env, int_id);
    let list_root = incorporate_child(&mut nodes, &mut edges, &list_ref);

    // port 3: tgt
    let tgt_ref = make_input_ref(tgt_input as u8, type_env, int_id);
    let tgt_root = incorporate_child(&mut nodes, &mut edges, &tgt_ref);

    // Fold with mode 0x21
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x21] },
        int_id, 4,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: src_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: list_root, port: 2, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: tgt_root, port: 3, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build bubble sort pass skeleton: fold mode 0x22.
fn build_bubble_sort_pass_skeleton(
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x07 }, // min (placeholder step)
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x22] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build decode_rle skeleton: fold mode 0x23.
fn build_decode_rle_skeleton(
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 }, // add (placeholder)
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x23] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build encode_rle skeleton: fold mode 0x24.
fn build_encode_rle_skeleton(
    _input_index: usize,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 }, // add (placeholder)
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![0x24] },
        int_id, 2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tc(inputs: Vec<Value>, expected: Value) -> TestCase {
        TestCase {
            inputs,
            expected_output: Some(vec![expected]),
            initial_state: None,
            expected_state: None,
        }
    }

    #[test]
    fn test_detect_constant() {
        let cases = vec![
            tc(vec![Value::Int(0)], Value::Int(42)),
            tc(vec![Value::Int(99)], Value::Int(42)),
            tc(vec![Value::Int(-5)], Value::Int(42)),
        ];
        let pattern = analyze_test_cases(&cases);
        assert_eq!(pattern, DetectedPattern::Constant(Value::Int(42)));
    }

    #[test]
    fn test_detect_identity() {
        let cases = vec![
            tc(vec![Value::Int(0)], Value::Int(0)),
            tc(vec![Value::Int(7)], Value::Int(7)),
            tc(vec![Value::Int(-3)], Value::Int(-3)),
            tc(vec![Value::Int(100)], Value::Int(100)),
        ];
        let pattern = analyze_test_cases(&cases);
        assert_eq!(pattern, DetectedPattern::Identity { input_index: 0 });
    }

    #[test]
    fn test_detect_binary_add() {
        let cases = vec![
            tc(vec![Value::Int(1), Value::Int(2)], Value::Int(3)),
            tc(vec![Value::Int(0), Value::Int(0)], Value::Int(0)),
            tc(vec![Value::Int(10), Value::Int(-3)], Value::Int(7)),
            tc(vec![Value::Int(-5), Value::Int(-5)], Value::Int(-10)),
        ];
        let pattern = analyze_test_cases(&cases);
        assert_eq!(
            pattern,
            DetectedPattern::BinaryArithmetic {
                op: 0x00,
                left_input: 0,
                right_input: 1,
            }
        );
    }

    #[test]
    fn test_detect_calculator() {
        let cases = vec![
            tc(
                vec![Value::Int(3), Value::Int(0), Value::Int(5)],
                Value::Int(8),
            ),
            tc(
                vec![Value::Int(10), Value::Int(1), Value::Int(3)],
                Value::Int(7),
            ),
            tc(
                vec![Value::Int(0), Value::Int(0), Value::Int(0)],
                Value::Int(0),
            ),
            tc(
                vec![Value::Int(7), Value::Int(1), Value::Int(7)],
                Value::Int(0),
            ),
            tc(
                vec![Value::Int(1), Value::Int(0), Value::Int(1)],
                Value::Int(2),
            ),
            tc(
                vec![Value::Int(5), Value::Int(1), Value::Int(2)],
                Value::Int(3),
            ),
        ];
        let pattern = analyze_test_cases(&cases);
        match &pattern {
            DetectedPattern::ConditionalDispatch {
                dispatch_input,
                branches,
            } => {
                assert_eq!(*dispatch_input, 1);
                assert!(branches.len() >= 2);

                let b0 = branches.iter().find(|(k, _)| *k == 0);
                assert!(b0.is_some());
                match &b0.unwrap().1 {
                    DetectedPattern::BinaryArithmetic {
                        op,
                        left_input,
                        right_input,
                    } => {
                        assert_eq!(*op, 0x00); // add
                        assert_eq!(*left_input, 0);
                        assert_eq!(*right_input, 2);
                    }
                    other => panic!("Expected BinaryArithmetic for branch 0, got {:?}", other),
                }

                let b1 = branches.iter().find(|(k, _)| *k == 1);
                assert!(b1.is_some());
                match &b1.unwrap().1 {
                    DetectedPattern::BinaryArithmetic {
                        op,
                        left_input,
                        right_input,
                    } => {
                        assert_eq!(*op, 0x01); // sub
                        assert_eq!(*left_input, 0);
                        assert_eq!(*right_input, 2);
                    }
                    other => panic!("Expected BinaryArithmetic for branch 1, got {:?}", other),
                }
            }
            other => panic!("Expected ConditionalDispatch, got {:?}", other),
        }
    }

    #[test]
    fn test_detect_fold_sum() {
        let cases = vec![
            tc(
                vec![Value::Tuple(vec![
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                ])],
                Value::Int(6),
            ),
            tc(vec![Value::Tuple(vec![Value::Int(10)])], Value::Int(10)),
            tc(
                vec![Value::Tuple(vec![Value::Int(0), Value::Int(0)])],
                Value::Int(0),
            ),
        ];
        let pattern = analyze_test_cases(&cases);
        assert_eq!(
            pattern,
            DetectedPattern::FoldReduction {
                base: 0,
                op: 0x00,
                input_index: 0,
            }
        );
    }

    #[test]
    fn test_detect_map_double() {
        let cases = vec![
            tc(
                vec![Value::Tuple(vec![
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                ])],
                Value::Tuple(vec![Value::Int(2), Value::Int(4), Value::Int(6)]),
            ),
            tc(
                vec![Value::Tuple(vec![Value::Int(0)])],
                Value::Tuple(vec![Value::Int(0)]),
            ),
            tc(
                vec![Value::Tuple(vec![Value::Int(5), Value::Int(10)])],
                Value::Tuple(vec![Value::Int(10), Value::Int(20)]),
            ),
        ];
        let pattern = analyze_test_cases(&cases);
        assert_eq!(
            pattern,
            DetectedPattern::MapTransform {
                op: 0x00, // add(x, x) = 2x
                input_index: 0,
            }
        );
    }

    #[test]
    fn test_detect_map_fold_sum_of_squares() {
        let cases = vec![
            tc(
                vec![Value::Tuple(vec![
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                ])],
                Value::Int(14), // 1 + 4 + 9
            ),
            tc(vec![Value::Tuple(vec![Value::Int(0)])], Value::Int(0)),
            tc(
                vec![Value::Tuple(vec![Value::Int(4), Value::Int(3)])],
                Value::Int(25), // 16 + 9
            ),
        ];
        let pattern = analyze_test_cases(&cases);
        assert_eq!(
            pattern,
            DetectedPattern::MapFoldComposition {
                map_op: 0x02,  // mul(x, x) = x^2
                fold_op: 0x00, // add
                fold_base: 0,
                input_index: 0,
            }
        );
    }

    #[test]
    fn test_detect_unknown() {
        let cases = vec![
            tc(vec![Value::Int(1)], Value::Int(17)),
            tc(vec![Value::Int(2)], Value::Int(42)),
            tc(vec![Value::Int(3)], Value::Int(-5)),
        ];
        let pattern = analyze_test_cases(&cases);
        assert_eq!(pattern, DetectedPattern::Unknown);
    }

    #[test]
    fn test_build_constant_skeleton() {
        let pattern = DetectedPattern::Constant(Value::Int(42));
        let fragment = build_skeleton(&pattern);
        assert!(fragment.is_some());
    }

    #[test]
    fn test_build_fold_skeleton() {
        let pattern = DetectedPattern::FoldReduction {
            base: 0,
            op: 0x00,
            input_index: 0,
        };
        let fragment = build_skeleton(&pattern);
        assert!(fragment.is_some());
    }

    #[test]
    fn test_build_map_skeleton() {
        let pattern = DetectedPattern::MapTransform {
            op: 0x00,
            input_index: 0,
        };
        let fragment = build_skeleton(&pattern);
        assert!(fragment.is_some());
    }

    #[test]
    fn test_build_unknown_returns_none() {
        let pattern = DetectedPattern::Unknown;
        let fragment = build_skeleton(&pattern);
        assert!(fragment.is_none());
    }

    #[test]
    fn test_execute_skeletons() {
        use iris_exec::interpreter::interpret;

        fn list(vals: &[i64]) -> Value {
            Value::Tuple(vals.iter().map(|&v| Value::Int(v)).collect())
        }

        // Test average skeleton
        let avg_pattern = DetectedPattern::TwoFoldScalar {
            fold1_base: 0, fold1_op: 0x00, fold2_base: 0, fold2_op: 0xFE,
            combine_op: 0x03, input_index: 0,
        };
        let avg_frag = build_skeleton(&avg_pattern).unwrap();
        let result = interpret(&avg_frag.graph, &[list(&[2, 4, 6])], None);
        eprintln!("average([2,4,6]) result: {:?}", result);

        // Test scan skeleton
        let scan_pattern = DetectedPattern::Scan { base: 0, op: 0x00, input_index: 0 };
        let scan_frag = build_skeleton(&scan_pattern).unwrap();
        let result = interpret(&scan_frag.graph, &[list(&[1, 2, 3, 4])], None);
        eprintln!("cumulative_sum([1,2,3,4]) result: {:?}", result);

        // Test fold-then-map skeleton
        let ftm_pattern = DetectedPattern::FoldThenMap {
            fold_base: i64::MIN, fold_op: 0x08, map_op: 0x01,
            fold_first: true, input_index: 0,
        };
        let ftm_frag = build_skeleton(&ftm_pattern).unwrap();
        let result = interpret(&ftm_frag.graph, &[list(&[1, 5, 3])], None);
        eprintln!("distance_from_max([1,5,3]) result: {:?}", result);

        // Test variance numerator skeleton
        let var_pattern = DetectedPattern::VarianceNumerator { input_index: 0 };
        let var_frag = build_skeleton(&var_pattern).unwrap();
        let result = interpret(&var_frag.graph, &[list(&[1, 2, 3])], None);
        eprintln!("variance_numerator([1,2,3]) result: {:?}", result);
        assert_eq!(result.unwrap().0, vec![Value::Int(2)]);

        let result = interpret(&var_frag.graph, &[list(&[0, 4])], None);
        eprintln!("variance_numerator([0,4]) result: {:?}", result);
        assert_eq!(result.unwrap().0, vec![Value::Int(8)]);

        // Test collatz step skeleton
        let collatz_pattern = DetectedPattern::CollatzStep;
        let collatz_frag = build_skeleton(&collatz_pattern).unwrap();
        let result = interpret(&collatz_frag.graph, &[Value::Int(4)], None);
        eprintln!("collatz_step(4) result: {:?}", result);
        assert_eq!(result.unwrap().0, vec![Value::Int(2)]);

        let result = interpret(&collatz_frag.graph, &[Value::Int(7)], None);
        eprintln!("collatz_step(7) result: {:?}", result);
        assert_eq!(result.unwrap().0, vec![Value::Int(22)]);

        let result = interpret(&collatz_frag.graph, &[Value::Int(1)], None);
        eprintln!("collatz_step(1) result: {:?}", result);
        assert_eq!(result.unwrap().0, vec![Value::Int(4)]);
    }

    #[test]
    fn test_detect_new_patterns() {
        fn list(vals: &[i64]) -> Value {
            Value::Tuple(vals.iter().map(|&v| Value::Int(v)).collect())
        }

        // Average: detected as TwoFoldScalar(sum, count, div)
        let avg_cases = vec![
            tc(vec![list(&[2, 4, 6])], Value::Int(4)),
            tc(vec![list(&[10])], Value::Int(10)),
            tc(vec![list(&[1, 2, 3, 4, 5])], Value::Int(3)),
            tc(vec![list(&[0, 0, 0, 0])], Value::Int(0)),
        ];
        let avg_patterns = analyze_all_patterns(&avg_cases);
        assert!(avg_patterns.iter().any(|p| matches!(p, DetectedPattern::TwoFoldScalar { .. })));

        // Cumulative sum: detected as Scan
        let cum_cases = vec![
            tc(vec![list(&[1, 2, 3, 4])], list(&[1, 3, 6, 10])),
            tc(vec![list(&[5])], list(&[5])),
            tc(vec![list(&[1, 1, 1])], list(&[1, 2, 3])),
        ];
        let cum_patterns = analyze_all_patterns(&cum_cases);
        assert!(cum_patterns.iter().any(|p| matches!(p, DetectedPattern::Scan { .. })));

        // Distance from max: detected as FoldThenMap
        let dfm_cases = vec![
            tc(vec![list(&[1, 5, 3])], list(&[4, 0, 2])),
            tc(vec![list(&[3, 3, 3])], list(&[0, 0, 0])),
            tc(vec![list(&[10])], list(&[0])),
        ];
        let dfm_patterns = analyze_all_patterns(&dfm_cases);
        assert!(dfm_patterns.iter().any(|p| matches!(p, DetectedPattern::FoldThenMap { .. })));

        // Variance numerator: detected as VarianceNumerator
        let var_cases = vec![
            tc(vec![list(&[1, 2, 3])], Value::Int(2)),
            tc(vec![list(&[5, 5, 5])], Value::Int(0)),
            tc(vec![list(&[0, 4])], Value::Int(8)),
        ];
        let var_patterns = analyze_all_patterns(&var_cases);
        assert!(var_patterns.iter().any(|p| matches!(p, DetectedPattern::VarianceNumerator { .. })));
    }
}
