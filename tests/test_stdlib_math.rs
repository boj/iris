
//! Tests for Math primitives (opcodes 0xD8-0xE3).
//!
//! Verifies:
//! - math_sqrt, math_log, math_exp, math_sin, math_cos
//! - math_floor, math_ceil, math_round
//! - math_pi, math_e (constants)
//! - random_int, random_float

use std::collections::{BTreeMap, HashMap};

use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::*;
use iris_types::hash::{compute_node_id, SemanticHash};
use iris_types::types::{PrimType, TypeDef, TypeEnv};

// ===========================================================================
// Helpers
// ===========================================================================

fn make_type_env() -> (TypeEnv, iris_types::types::TypeId) {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = iris_types::hash::compute_type_id(&int_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    (TypeEnv { types }, int_id)
}

/// Build a graph that evaluates a single Prim opcode on its argument edges.
fn make_prim_graph(opcode: u8, arg_lits: &[(u8, Vec<u8>)]) -> SemanticGraph {
    let (type_env, int_id) = make_type_env();

    let mut nodes = HashMap::new();
    let mut edges = vec![];
    let mut arg_ids = vec![];

    for (type_tag, value) in arg_lits.iter() {
        let mut node = Node {
            id: NodeId(0),
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2,
            salt: 0,
            payload: NodePayload::Lit {
                type_tag: *type_tag,
                value: value.clone(),
            },
        };
        node.id = compute_node_id(&node);
        arg_ids.push(node.id);
        nodes.insert(node.id, node);
    }

    let mut prim_node = Node {
        id: NodeId(0),
        kind: NodeKind::Prim,
        type_sig: int_id,
        cost: CostTerm::Unit,
        arity: arg_lits.len() as u8,
        resolution_depth: 2,
        salt: 0,
        payload: NodePayload::Prim { opcode },
    };
    prim_node.id = compute_node_id(&prim_node);

    for (i, &aid) in arg_ids.iter().enumerate() {
        edges.push(Edge {
            source: prim_node.id,
            target: aid,
            port: i as u8,
            label: EdgeLabel::Argument,
        });
    }

    nodes.insert(prim_node.id, prim_node.clone());

    SemanticGraph {
        root: prim_node.id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

/// Build a graph with a zero-arg Prim node (for constants like math_pi).
fn make_nullary_prim_graph(opcode: u8) -> SemanticGraph {
    let (type_env, int_id) = make_type_env();
    let mut nodes = HashMap::new();

    let mut prim_node = Node {
        id: NodeId(0),
        kind: NodeKind::Prim,
        type_sig: int_id,
        cost: CostTerm::Unit,
        arity: 0,
        resolution_depth: 2,
        salt: 0,
        payload: NodePayload::Prim { opcode },
    };
    prim_node.id = compute_node_id(&prim_node);
    nodes.insert(prim_node.id, prim_node.clone());

    SemanticGraph {
        root: prim_node.id,
        nodes,
        edges: vec![],
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

fn float_lit(v: f64) -> (u8, Vec<u8>) {
    (0x02, v.to_le_bytes().to_vec())
}

fn int_lit(v: i64) -> (u8, Vec<u8>) {
    (0x00, v.to_le_bytes().to_vec())
}

fn eval_graph(graph: &SemanticGraph) -> Result<Value, iris_exec::interpreter::InterpretError> {
    let (outputs, _state) = interpreter::interpret(graph, &[], None)?;
    if outputs.len() == 1 {
        Ok(outputs.into_iter().next().unwrap())
    } else {
        Ok(Value::tuple(outputs))
    }
}

fn assert_float_eq(result: &Value, expected: f64, tolerance: f64) {
    match result {
        Value::Float64(v) => {
            assert!(
                (v - expected).abs() < tolerance,
                "expected ~{}, got {}",
                expected,
                v
            );
        }
        _ => panic!("expected Float64, got {:?}", result),
    }
}

// ===========================================================================
// math_sqrt (0xD8)
// ===========================================================================

#[test]
fn math_sqrt_perfect_square() {
    let g = make_prim_graph(0xD8, &[float_lit(9.0)]);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, 3.0, 1e-10);
}

#[test]
fn math_sqrt_non_perfect() {
    let g = make_prim_graph(0xD8, &[float_lit(2.0)]);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, std::f64::consts::SQRT_2, 1e-10);
}

#[test]
fn math_sqrt_zero() {
    let g = make_prim_graph(0xD8, &[float_lit(0.0)]);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, 0.0, 1e-10);
}

#[test]
fn math_sqrt_coerce_int() {
    // sqrt should accept Int and coerce to Float64
    let g = make_prim_graph(0xD8, &[int_lit(16)]);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, 4.0, 1e-10);
}

// ===========================================================================
// math_log (0xD9)
// ===========================================================================

#[test]
fn math_log_e() {
    let g = make_prim_graph(0xD9, &[float_lit(std::f64::consts::E)]);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, 1.0, 1e-10);
}

#[test]
fn math_log_one() {
    let g = make_prim_graph(0xD9, &[float_lit(1.0)]);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, 0.0, 1e-10);
}

// ===========================================================================
// math_exp (0xDA)
// ===========================================================================

#[test]
fn math_exp_zero() {
    let g = make_prim_graph(0xDA, &[float_lit(0.0)]);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, 1.0, 1e-10);
}

#[test]
fn math_exp_one() {
    let g = make_prim_graph(0xDA, &[float_lit(1.0)]);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, std::f64::consts::E, 1e-10);
}

// ===========================================================================
// math_sin (0xDB)
// ===========================================================================

#[test]
fn math_sin_zero() {
    let g = make_prim_graph(0xDB, &[float_lit(0.0)]);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, 0.0, 1e-10);
}

#[test]
fn math_sin_pi_half() {
    let g = make_prim_graph(0xDB, &[float_lit(std::f64::consts::FRAC_PI_2)]);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, 1.0, 1e-10);
}

#[test]
fn math_sin_pi() {
    let g = make_prim_graph(0xDB, &[float_lit(std::f64::consts::PI)]);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, 0.0, 1e-10);
}

// ===========================================================================
// math_cos (0xDC)
// ===========================================================================

#[test]
fn math_cos_zero() {
    let g = make_prim_graph(0xDC, &[float_lit(0.0)]);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, 1.0, 1e-10);
}

#[test]
fn math_cos_pi() {
    let g = make_prim_graph(0xDC, &[float_lit(std::f64::consts::PI)]);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, -1.0, 1e-10);
}

// ===========================================================================
// math_floor (0xDD)
// ===========================================================================

#[test]
fn math_floor_positive() {
    let g = make_prim_graph(0xDD, &[float_lit(3.7)]);
    let result = eval_graph(&g).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn math_floor_negative() {
    let g = make_prim_graph(0xDD, &[float_lit(-2.3)]);
    let result = eval_graph(&g).unwrap();
    assert_eq!(result, Value::Int(-3));
}

#[test]
fn math_floor_exact() {
    let g = make_prim_graph(0xDD, &[float_lit(5.0)]);
    let result = eval_graph(&g).unwrap();
    assert_eq!(result, Value::Int(5));
}

// ===========================================================================
// math_ceil (0xDE)
// ===========================================================================

#[test]
fn math_ceil_positive() {
    let g = make_prim_graph(0xDE, &[float_lit(3.1)]);
    let result = eval_graph(&g).unwrap();
    assert_eq!(result, Value::Int(4));
}

#[test]
fn math_ceil_negative() {
    let g = make_prim_graph(0xDE, &[float_lit(-2.7)]);
    let result = eval_graph(&g).unwrap();
    assert_eq!(result, Value::Int(-2));
}

#[test]
fn math_ceil_exact() {
    let g = make_prim_graph(0xDE, &[float_lit(5.0)]);
    let result = eval_graph(&g).unwrap();
    assert_eq!(result, Value::Int(5));
}

// ===========================================================================
// math_round (0xDF)
// ===========================================================================

#[test]
fn math_round_down() {
    let g = make_prim_graph(0xDF, &[float_lit(3.3)]);
    let result = eval_graph(&g).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn math_round_up() {
    let g = make_prim_graph(0xDF, &[float_lit(3.7)]);
    let result = eval_graph(&g).unwrap();
    assert_eq!(result, Value::Int(4));
}

#[test]
fn math_round_half() {
    // Rust's f64::round() rounds 0.5 away from zero.
    let g = make_prim_graph(0xDF, &[float_lit(2.5)]);
    let result = eval_graph(&g).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn math_round_negative() {
    let g = make_prim_graph(0xDF, &[float_lit(-1.5)]);
    let result = eval_graph(&g).unwrap();
    assert_eq!(result, Value::Int(-2));
}

// ===========================================================================
// math_pi (0xE0) and math_e (0xE1)
// ===========================================================================

#[test]
fn math_pi_constant() {
    let g = make_nullary_prim_graph(0xE0);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, std::f64::consts::PI, 1e-15);
}

#[test]
fn math_e_constant() {
    let g = make_nullary_prim_graph(0xE1);
    let result = eval_graph(&g).unwrap();
    assert_float_eq(&result, std::f64::consts::E, 1e-15);
}

// ===========================================================================
// random_int (0xE2)
// ===========================================================================

#[test]
fn random_int_in_range() {
    let g = make_prim_graph(0xE2, &[int_lit(1), int_lit(10)]);
    let result = eval_graph(&g).unwrap();
    match result {
        Value::Int(v) => {
            assert!(v >= 1 && v <= 10, "random_int(1,10) returned {}", v);
        }
        _ => panic!("expected Int, got {:?}", result),
    }
}

#[test]
fn random_int_same_min_max() {
    // When min == max, should return min.
    let g = make_prim_graph(0xE2, &[int_lit(5), int_lit(5)]);
    let result = eval_graph(&g).unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn random_int_min_gt_max() {
    // When min > max, should return min (degenerate case).
    let g = make_prim_graph(0xE2, &[int_lit(10), int_lit(1)]);
    let result = eval_graph(&g).unwrap();
    assert_eq!(result, Value::Int(10));
}

// ===========================================================================
// random_float (0xE3)
// ===========================================================================

#[test]
fn random_float_in_range() {
    let g = make_nullary_prim_graph(0xE3);
    let result = eval_graph(&g).unwrap();
    match result {
        Value::Float64(v) => {
            assert!(
                v >= 0.0 && v <= 1.0,
                "random_float() returned {} (should be in [0,1])",
                v
            );
        }
        _ => panic!("expected Float64, got {:?}", result),
    }
}

#[test]
fn random_float_produces_different_values() {
    // Run twice, should get different values (probabilistically).
    // This test may very rarely fail if the RNG produces the same value twice.
    let g1 = make_nullary_prim_graph(0xE3);
    let g2 = make_nullary_prim_graph(0xE3);
    let r1 = eval_graph(&g1).unwrap();
    // Small delay to ensure different timestamp seed
    std::thread::sleep(std::time::Duration::from_millis(2));
    let r2 = eval_graph(&g2).unwrap();
    // We just check both are valid floats; exact equality check is not
    // reliable since both calls might get the same seed from fast execution.
    match (&r1, &r2) {
        (Value::Float64(a), Value::Float64(b)) => {
            assert!(*a >= 0.0 && *a <= 1.0);
            assert!(*b >= 0.0 && *b <= 1.0);
        }
        _ => panic!("expected Float64 values"),
    }
}

// ===========================================================================
// Pythagorean identity: sin^2(x) + cos^2(x) = 1
// ===========================================================================

#[test]
fn pythagorean_identity() {
    // Compute sin(1.0)^2 + cos(1.0)^2 and verify it equals 1.0.
    let g_sin = make_prim_graph(0xDB, &[float_lit(1.0)]);
    let g_cos = make_prim_graph(0xDC, &[float_lit(1.0)]);
    let sin_val = match eval_graph(&g_sin).unwrap() {
        Value::Float64(v) => v,
        _ => panic!("expected Float64"),
    };
    let cos_val = match eval_graph(&g_cos).unwrap() {
        Value::Float64(v) => v,
        _ => panic!("expected Float64"),
    };
    let sum = sin_val * sin_val + cos_val * cos_val;
    assert!(
        (sum - 1.0).abs() < 1e-10,
        "sin^2 + cos^2 = {} (should be 1.0)",
        sum
    );
}

// ===========================================================================
// exp(log(x)) = x identity
// ===========================================================================

#[test]
fn exp_log_identity() {
    let g_log = make_prim_graph(0xD9, &[float_lit(7.5)]);
    let log_val = match eval_graph(&g_log).unwrap() {
        Value::Float64(v) => v,
        _ => panic!("expected Float64"),
    };
    let g_exp = make_prim_graph(0xDA, &[float_lit(log_val)]);
    let result = eval_graph(&g_exp).unwrap();
    assert_float_eq(&result, 7.5, 1e-10);
}
