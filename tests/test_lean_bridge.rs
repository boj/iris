//! Integration tests for the Lean FFI bridge.
//!
//! Verifies the wire format encoding and that the Lean bridge
//! agrees with the Rust cost_leq implementation.

use iris_bootstrap::syntax::kernel::cost_checker;
use iris_bootstrap::syntax::kernel::lean_bridge::{
    encode_cost_bound, encode_context, encode_node_id, encode_type_id,
    encode_binder_id, encode_judgment, decode_context, decode_node_id,
    decode_type_id, decode_binder_id, decode_judgment, decode_cost_bound,
    decode_lean_result, lean_check_cost_leq,
};
use iris_bootstrap::syntax::kernel::theorem::{Binding, Context, Judgment};
use iris_types::cost::{CostBound, CostVar};
use iris_types::graph::{BinderId, NodeId};
use iris_types::types::TypeId;

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
fn test_amortized_encoding() {
    use iris_types::cost::PotentialFn;
    let amortized = CostBound::Amortized(
        Box::new(CostBound::Constant(42)),
        PotentialFn { description: "test".to_string() },
    );

    let mut buf = Vec::new();
    encode_cost_bound(&amortized, &mut buf);
    // Should encode with Amortized tag 0x0B, then inner cost (Constant 42): tag 0x02 + 42 as u64 LE
    assert_eq!(buf[0], 0x0B);
    assert_eq!(buf[1], 0x02);
    assert_eq!(u64::from_le_bytes(buf[2..10].try_into().unwrap()), 42);
}

#[test]
fn test_hwscaled_encoding() {
    use iris_types::cost::HWParamRef;
    let hwscaled = CostBound::HWScaled(
        Box::new(CostBound::Linear(CostVar(3))),
        HWParamRef([0xAB; 32]),
    );

    let mut buf = Vec::new();
    encode_cost_bound(&hwscaled, &mut buf);
    // Should encode with HWScaled tag 0x0C, then inner cost (Linear): tag 0x03 + var as u32 LE
    assert_eq!(buf[0], 0x0C);
    assert_eq!(buf[1], 0x03);
    assert_eq!(u32::from_le_bytes(buf[2..6].try_into().unwrap()), 3);
}

// ===========================================================================
// Wire format round-trip tests for kernel types
// ===========================================================================

#[test]
fn test_node_id_round_trip() {
    let node = NodeId(0xDEADBEEFCAFEBABE);
    let mut buf = Vec::new();
    encode_node_id(node, &mut buf);
    assert_eq!(buf.len(), 8);
    let (decoded, pos) = decode_node_id(&buf, 0).unwrap();
    assert_eq!(decoded, node);
    assert_eq!(pos, 8);
}

#[test]
fn test_type_id_round_trip() {
    let ty = TypeId(42);
    let mut buf = Vec::new();
    encode_type_id(ty, &mut buf);
    let (decoded, pos) = decode_type_id(&buf, 0).unwrap();
    assert_eq!(decoded, ty);
    assert_eq!(pos, 8);
}

#[test]
fn test_binder_id_round_trip() {
    let binder = BinderId(123);
    let mut buf = Vec::new();
    encode_binder_id(binder, &mut buf);
    let (decoded, pos) = decode_binder_id(&buf, 0).unwrap();
    assert_eq!(decoded, binder);
    assert_eq!(pos, 4);
}

#[test]
fn test_context_empty_round_trip() {
    let ctx = Context::empty();
    let mut buf = Vec::new();
    encode_context(&ctx, &mut buf);
    let (decoded, pos) = decode_context(&buf, 0).unwrap();
    assert_eq!(decoded, ctx);
    assert_eq!(pos, 2); // just the u16 count
}

#[test]
fn test_context_with_bindings_round_trip() {
    let ctx = Context {
        bindings: vec![
            Binding { name: BinderId(1), type_id: TypeId(100) },
            Binding { name: BinderId(2), type_id: TypeId(200) },
            Binding { name: BinderId(3), type_id: TypeId(300) },
        ],
    };
    let mut buf = Vec::new();
    encode_context(&ctx, &mut buf);
    let (decoded, _pos) = decode_context(&buf, 0).unwrap();
    assert_eq!(decoded, ctx);
}

#[test]
fn test_judgment_round_trip() {
    let j = Judgment {
        context: Context {
            bindings: vec![
                Binding { name: BinderId(1), type_id: TypeId(10) },
            ],
        },
        node_id: NodeId(42),
        type_ref: TypeId(99),
        cost: CostBound::Sum(
            Box::new(CostBound::Constant(5)),
            Box::new(CostBound::Zero),
        ),
    };
    let mut buf = Vec::new();
    encode_judgment(&j, &mut buf);
    let (decoded, _pos) = decode_judgment(&buf, 0).unwrap();
    assert_eq!(decoded.context, j.context);
    assert_eq!(decoded.node_id, j.node_id);
    assert_eq!(decoded.type_ref, j.type_ref);
    // Cost comparison: check the structure matches
    assert_eq!(format!("{:?}", decoded.cost), format!("{:?}", j.cost));
}

#[test]
fn test_cost_bound_round_trip_all_variants() {
    let cases: Vec<CostBound> = vec![
        CostBound::Unknown,
        CostBound::Zero,
        CostBound::Constant(0),
        CostBound::Constant(999),
        CostBound::Linear(CostVar(7)),
        CostBound::NLogN(CostVar(3)),
        CostBound::Polynomial(CostVar(1), 4),
        CostBound::Sum(Box::new(CostBound::Constant(1)), Box::new(CostBound::Constant(2))),
        CostBound::Par(Box::new(CostBound::Zero), Box::new(CostBound::Constant(5))),
        CostBound::Mul(Box::new(CostBound::Linear(CostVar(0))), Box::new(CostBound::Constant(3))),
        CostBound::Sup(vec![CostBound::Constant(1), CostBound::Constant(2)]),
        CostBound::Inf(vec![CostBound::Linear(CostVar(0))]),
    ];

    for cost in &cases {
        let mut buf = Vec::new();
        encode_cost_bound(cost, &mut buf);
        let (decoded, pos) = decode_cost_bound(&buf, 0).unwrap();
        assert_eq!(pos, buf.len(), "Should consume all bytes for {cost:?}");
        assert_eq!(format!("{decoded:?}"), format!("{cost:?}"), "Round-trip failed for {cost:?}");
    }
}

#[test]
fn test_decode_lean_result_success() {
    let j = Judgment {
        context: Context::empty(),
        node_id: NodeId(0),
        type_ref: TypeId(0),
        cost: CostBound::Zero,
    };
    let mut buf = vec![1u8]; // success byte
    encode_judgment(&j, &mut buf);
    let result = decode_lean_result(&buf);
    assert!(result.is_some());
    let decoded = result.unwrap();
    assert_eq!(decoded.node_id, NodeId(0));
    assert_eq!(decoded.type_ref, TypeId(0));
}

#[test]
fn test_decode_lean_result_failure() {
    let buf = vec![0u8, 1]; // failure byte + error code
    let result = decode_lean_result(&buf);
    assert!(result.is_none());
}

#[test]
fn test_decode_lean_result_empty() {
    let result = decode_lean_result(&[]);
    assert!(result.is_none());
}
