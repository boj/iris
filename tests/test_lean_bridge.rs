//! Integration tests for the Lean FFI bridge.
//!
//! Verifies the wire format encoding and that the Lean bridge
//! agrees with the Rust cost_leq implementation.

use iris_bootstrap::syntax::kernel::cost_checker;
use iris_bootstrap::syntax::kernel::lean_bridge::{encode_cost_bound, lean_check_cost_leq};
use iris_types::cost::{CostBound, CostVar};

#[test]
fn test_encode_all_variants() {
    let cases: Vec<CostBound> = vec![
        CostBound::Unknown,
        CostBound::Zero,
        CostBound::Constant(0),
        CostBound::Constant(42),
        CostBound::Linear(CostVar(0)),
        CostBound::NLogN(CostVar(5)),
        CostBound::Polynomial(CostVar(0), 2),
        CostBound::Sum(Box::new(CostBound::Zero), Box::new(CostBound::Constant(1))),
        CostBound::Sup(vec![CostBound::Constant(1), CostBound::Constant(2)]),
        CostBound::Inf(vec![CostBound::Linear(CostVar(0))]),
        CostBound::Par(Box::new(CostBound::Constant(1)), Box::new(CostBound::Constant(2))),
        CostBound::Mul(Box::new(CostBound::Linear(CostVar(0))), Box::new(CostBound::Constant(3))),
    ];

    for cost in &cases {
        let mut buf = Vec::new();
        encode_cost_bound(cost, &mut buf);
        assert!(!buf.is_empty(), "Encoding {cost:?} should produce bytes");
    }
}

#[test]
fn test_encode_nested() {
    let nested = CostBound::Sum(
        Box::new(CostBound::Sum(
            Box::new(CostBound::Constant(1)),
            Box::new(CostBound::Constant(2)),
        )),
        Box::new(CostBound::Mul(
            Box::new(CostBound::Linear(CostVar(0))),
            Box::new(CostBound::Polynomial(CostVar(1), 3)),
        )),
    );

    let mut buf = Vec::new();
    encode_cost_bound(&nested, &mut buf);
    assert!(buf.len() > 10, "Nested cost should produce multiple bytes");
}

#[test]
fn test_encode_cost_pair() {
    let mut buf = Vec::new();
    encode_cost_bound(&CostBound::Zero, &mut buf);
    encode_cost_bound(&CostBound::Constant(42), &mut buf);

    assert_eq!(buf[0], 0x01); // Zero tag
    assert_eq!(buf[1], 0x02); // Constant tag
    let k = u64::from_le_bytes(buf[2..10].try_into().unwrap());
    assert_eq!(k, 42);
}

#[test]
fn test_lean_cost_leq_matches_rust_100_pairs() {
    let n = CostVar(0);
    let m = CostVar(1);

    let costs: Vec<CostBound> = vec![
        CostBound::Unknown, CostBound::Zero,
        CostBound::Constant(0), CostBound::Constant(1), CostBound::Constant(5), CostBound::Constant(100),
        CostBound::Linear(n), CostBound::Linear(m),
        CostBound::NLogN(n), CostBound::Polynomial(n, 2),
    ];

    let mut total = 0;
    for a in &costs {
        for b in &costs {
            let rust = cost_checker::cost_leq(a, b);
            let lean = lean_check_cost_leq(a, b);
            assert_eq!(rust, lean, "Disagreement on cost_leq({a:?}, {b:?})");
            total += 1;
        }
    }
    assert!(total >= 100);
}

#[test]
fn test_lean_cost_leq_composite_pairs() {
    let n = CostVar(0);
    let sum_a = CostBound::Sum(Box::new(CostBound::Constant(1)), Box::new(CostBound::Constant(2)));
    let sum_b = CostBound::Sum(Box::new(CostBound::Constant(3)), Box::new(CostBound::Constant(4)));
    assert!(lean_check_cost_leq(&sum_a, &sum_b));
    assert!(!lean_check_cost_leq(&sum_b, &sum_a));

    // Unknown no longer absorbs all costs — only Unknown <= Unknown is valid.
    assert!(!lean_check_cost_leq(&CostBound::Zero, &CostBound::Unknown));
    assert!(!lean_check_cost_leq(&CostBound::Unknown, &CostBound::Linear(n)));
}

#[test]
fn test_lean_bridge_loads() {
    // Unknown no longer absorbs all costs — only Unknown <= Unknown is valid.
    assert!(!lean_check_cost_leq(&CostBound::Zero, &CostBound::Unknown));
    assert!(lean_check_cost_leq(&CostBound::Zero, &CostBound::Zero));
    assert!(!lean_check_cost_leq(&CostBound::Unknown, &CostBound::Zero));
}

#[test]
fn test_amortized_encoding_fallback() {
    use iris_types::cost::PotentialFn;
    let amortized = CostBound::Amortized(
        Box::new(CostBound::Constant(42)),
        PotentialFn { description: "test".to_string() },
    );

    let mut buf = Vec::new();
    encode_cost_bound(&amortized, &mut buf);
    // Should encode as the inner cost (Constant 42): tag 0x02 + 42 as u64 LE
    assert_eq!(buf[0], 0x02);
    assert_eq!(u64::from_le_bytes(buf[1..9].try_into().unwrap()), 42);
}
