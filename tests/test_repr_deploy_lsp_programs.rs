
//! Behavioral tests for iris-repr, iris-deploy, and iris-lsp IRIS programs.
//!
//! These tests EXECUTE the .iris programs through the bootstrap evaluator
//! and full interpreter, verifying actual behavior rather than just parsing.

use std::collections::HashMap;
use std::rc::Rc;
use std::fs;

use iris_bootstrap::evaluate;
use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

// ===========================================================================
// Helpers: compile .iris source and evaluate functions
// ===========================================================================

/// Compile a source file and return the named fragment plus a registry
/// of all fragments for cross-fragment resolution.
fn compile_with_registry(src: &str) -> (Vec<(String, SemanticGraph)>, FragmentRegistry) {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!("compilation failed with {} errors", result.errors.len());
    }
    let mut registry = FragmentRegistry::new();
    let mut fragments = Vec::new();
    for (name, fragment, _) in &result.fragments {
        registry.register(fragment.clone());
        fragments.push((name.clone(), fragment.graph.clone()));
    }
    (fragments, registry)
}

/// Get a named fragment from a compiled list.
fn get_named(fragments: &[(String, SemanticGraph)], name: &str) -> SemanticGraph {
    for (fname, graph) in fragments {
        if fname == name {
            return graph.clone();
        }
    }
    panic!("fragment '{}' not found in {:?}", name, fragments.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>());
}

fn load_iris(path: &str) -> String {
    let full_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path);
    fs::read_to_string(&full_path).unwrap_or_else(|e| panic!("failed to read {}: {}", full_path, e))
}

/// Evaluate using the full interpreter with registry support.
fn eval_with_registry(graph: &SemanticGraph, inputs: &[Value], registry: &FragmentRegistry) -> Value {
    let (out, _) = interpreter::interpret_with_registry(graph, inputs, None, Some(registry))
        .unwrap_or_else(|e| panic!("interpret failed: {:?}", e));
    assert!(!out.is_empty(), "no output from interpreter");
    out.into_iter().next().unwrap()
}

/// Evaluate a self-contained function (no cross-fragment refs) using bootstrap.
fn eval(graph: &SemanticGraph, inputs: &[Value]) -> Value {
    evaluate(graph, inputs).unwrap_or_else(|e| panic!("evaluation failed: {}", e))
}

fn make_test_graph(node_count: usize) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let root_id = NodeId(1);
    nodes.insert(
        root_id,
        Node {
            id: root_id,
            kind: NodeKind::Prim,
            type_sig: TypeId(0),
            cost: CostTerm::Unit,
            arity: 2,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Prim { opcode: 0x00 },
        },
    );

    for i in 1..node_count {
        let nid = NodeId((i + 1) as u64);
        nodes.insert(
            nid,
            Node {
                id: nid,
                kind: NodeKind::Lit,
                type_sig: TypeId(0),
                cost: CostTerm::Unit,
                arity: 0,
                resolution_depth: 0,
                salt: 0,
                payload: NodePayload::Lit {
                    type_tag: 0x00,
                    value: (i as i64).to_le_bytes().to_vec(),
                },
            },
        );
        edges.push(Edge {
            source: root_id,
            target: nid,
            port: (i - 1) as u8,
            label: EdgeLabel::Argument,
        });
    }

    SemanticGraph {
        root: root_id,
        nodes,
        edges,
        type_env: TypeEnv {
            types: std::collections::BTreeMap::new(),
        },
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

// ===========================================================================
// Full execution validation: compile + evaluate functions from every .iris file
// ===========================================================================

/// Evaluate at least one function from every .iris file in src/iris-programs/repr.
#[test]
fn eval_all_repr_programs() {
    // abstract_machine.iris: fits_machine, select_backend
    {
        let src = load_iris("src/iris-programs/repr/abstract_machine.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "fits_machine");
        assert_eq!(eval_with_registry(&g, &[Value::Int(10), Value::Int(100), Value::Int(50)], &reg), Value::Int(1));
        // Exceeds stack limit
        assert_eq!(eval_with_registry(&g, &[Value::Int(10), Value::Int(2000), Value::Int(50)], &reg), Value::Int(0));
        let g2 = get_named(&frags, "select_backend");
        // Small program, no neural, no JIT -> VM (0)
        assert_eq!(eval_with_registry(&g2, &[Value::Int(10), Value::Int(0), Value::Int(0), Value::Int(0)], &reg), Value::Int(0));
        // Large program with JIT -> JIT (1)
        assert_eq!(eval_with_registry(&g2, &[Value::Int(200), Value::Int(0), Value::Int(1), Value::Int(0)], &reg), Value::Int(1));
    }

    // compare_programs.iris: uses graph inputs (tested via make_test_graph)
    {
        let src = load_iris("src/iris-programs/repr/compare_programs.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "compare_programs");
        let g1 = make_test_graph(3);
        let g2 = make_test_graph(3);
        assert_eq!(eval_with_registry(&g, &[Value::Program(Rc::new(g1.clone())), Value::Program(Rc::new(g2))], &reg), Value::Int(1));
        let g3 = make_test_graph(5);
        assert_eq!(eval_with_registry(&g, &[Value::Program(Rc::new(g1)), Value::Program(Rc::new(g3))], &reg), Value::Int(0));
    }

    // component.iris: classify_component, is_valid_component, make_mutation
    {
        let src = load_iris("src/iris-programs/repr/component.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "classify_component");
        assert_eq!(eval_with_registry(&g, &[Value::Int(0)], &reg), Value::Int(0));
        assert_eq!(eval_with_registry(&g, &[Value::Int(2)], &reg), Value::Int(2));
        assert_eq!(eval_with_registry(&g, &[Value::Int(5)], &reg), Value::Int(-1));
        let g2 = get_named(&frags, "registry_contains");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(100), Value::Int(42)], &reg), Value::Int(1)); // found
        assert_eq!(eval_with_registry(&g2, &[Value::Int(0), Value::Int(42)], &reg), Value::Int(0)); // empty registry
        let g3 = get_named(&frags, "make_mutation");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(42)], &reg),
            Value::tuple(vec![Value::Int(0), Value::Int(42), Value::Int(1)]));
    }

    // cost.iris: is_leaf_cost, compare_costs, universalize_cost
    {
        let src = load_iris("src/iris-programs/repr/cost.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "is_leaf_cost");
        assert_eq!(eval_with_registry(&g, &[Value::Int(0)], &reg), Value::Int(1)); // Unknown is leaf
        assert_eq!(eval_with_registry(&g, &[Value::Int(5)], &reg), Value::Int(1)); // Polynomial is leaf
        assert_eq!(eval_with_registry(&g, &[Value::Int(6)], &reg), Value::Int(0)); // Sum is not leaf
        let g2 = get_named(&frags, "compare_costs");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(1), Value::Int(0), Value::Int(1), Value::Int(0)], &reg), Value::Int(0));
        assert_eq!(eval_with_registry(&g2, &[Value::Int(1), Value::Int(0), Value::Int(3), Value::Int(0)], &reg), Value::Int(-1));
        let g3 = get_named(&frags, "universalize_cost");
        // HWScaled(inner=Linear(5)) -> (3, 5)
        assert_eq!(eval_with_registry(&g3, &[Value::Int(10), Value::Int(0), Value::Int(3), Value::Int(5)], &reg),
            Value::tuple(vec![Value::Int(3), Value::Int(5)]));
        // Non-HW passes through
        assert_eq!(eval_with_registry(&g3, &[Value::Int(2), Value::Int(42), Value::Int(0), Value::Int(0)], &reg),
            Value::tuple(vec![Value::Int(2), Value::Int(42)]));
    }

    // eval.iris: classify_eval_tier, tier_test_count, compute_ipc, is_valid_value_type
    {
        let src = load_iris("src/iris-programs/repr/eval.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "classify_eval_tier");
        assert_eq!(eval_with_registry(&g, &[Value::Int(10), Value::Int(0), Value::Int(0)], &reg), Value::Int(0)); // tier A
        assert_eq!(eval_with_registry(&g, &[Value::Int(30), Value::Int(0), Value::Int(0)], &reg), Value::Int(1)); // tier B
        assert_eq!(eval_with_registry(&g, &[Value::Int(10), Value::Int(1), Value::Int(0)], &reg), Value::Int(2)); // effects -> tier C
        let g2 = get_named(&frags, "tier_test_count");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(0)], &reg), Value::Int(10));
        assert_eq!(eval_with_registry(&g2, &[Value::Int(1)], &reg), Value::Int(100));
        let g3 = get_named(&frags, "compute_ipc");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(3000), Value::Int(1000)], &reg), Value::Int(3000));
        assert_eq!(eval_with_registry(&g3, &[Value::Int(0), Value::Int(0)], &reg), Value::Int(0));
        let g4 = get_named(&frags, "is_valid_value_type");
        assert_eq!(eval_with_registry(&g4, &[Value::Int(0)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g4, &[Value::Int(14)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g4, &[Value::Int(15)], &reg), Value::Int(0));
    }

    // fragment_signature.iris: boundary_port_count, validate_boundary, validate_fragment
    {
        let src = load_iris("src/iris-programs/repr/fragment_signature.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "boundary_port_count");
        assert_eq!(eval_with_registry(&g, &[Value::Int(3), Value::Int(1)], &reg), Value::Int(4));
        let g2 = get_named(&frags, "validate_boundary");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(2), Value::Int(1)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g2, &[Value::Int(2), Value::Int(0)], &reg), Value::Int(0));
        let g3 = get_named(&frags, "validate_fragment");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(2), Value::Int(1), Value::Int(100), Value::Int(0), Value::Int(3)], &reg), Value::Int(1));
    }

    // fragment_size.iris: fragment_size (needs graph)
    {
        let src = load_iris("src/iris-programs/repr/fragment_size.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "fragment_size");
        let test_graph = make_test_graph(3);
        // 3 nodes -> 3 * 38 + 14 = 128
        assert_eq!(eval_with_registry(&g, &[Value::Program(Rc::new(test_graph))], &reg), Value::Int(128));
    }

    // guard.iris: is_valid_error_bound, validate_statistical, compare_error_bounds
    {
        let src = load_iris("src/iris-programs/repr/guard.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "is_valid_error_bound");
        assert_eq!(eval_with_registry(&g, &[Value::Int(0)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g, &[Value::Int(3)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g, &[Value::Int(4)], &reg), Value::Int(0));
        let g2 = get_named(&frags, "validate_statistical");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(950), Value::Int(10)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g2, &[Value::Int(1001), Value::Int(10)], &reg), Value::Int(0));
        let g3 = get_named(&frags, "compare_error_bounds");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(0), Value::Int(3)], &reg), Value::Int(-1));
        assert_eq!(eval_with_registry(&g3, &[Value::Int(2), Value::Int(2)], &reg), Value::Int(0));
    }

    // hash.iris: hash_mix
    {
        let src = load_iris("src/iris-programs/repr/hash.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "hash_mix");
        let result = eval_with_registry(&g, &[Value::Int(0), Value::Int(42)], &reg);
        // Verify determinism
        assert_eq!(result, eval_with_registry(&g, &[Value::Int(0), Value::Int(42)], &reg));
    }

    // proof.iris: classify_verify_tier, tier_time_limit, validate_proof_receipt
    {
        let src = load_iris("src/iris-programs/repr/proof.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "classify_verify_tier");
        assert_eq!(eval_with_registry(&g, &[Value::Int(0), Value::Int(0), Value::Int(0)], &reg), Value::Int(0));
        assert_eq!(eval_with_registry(&g, &[Value::Int(1), Value::Int(1), Value::Int(0)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g, &[Value::Int(0), Value::Int(0), Value::Int(1)], &reg), Value::Int(3));
        let g2 = get_named(&frags, "tier_time_limit");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(0)], &reg), Value::Int(10));
        assert_eq!(eval_with_registry(&g2, &[Value::Int(2)], &reg), Value::Int(60000));
        let g3 = get_named(&frags, "validate_proof_receipt");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(1), Value::Int(100), Value::Int(5)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g3, &[Value::Int(5), Value::Int(100), Value::Int(5)], &reg), Value::Int(0));
    }

    // resolution_level.iris: resolution_depth
    {
        let src = load_iris("src/iris-programs/repr/resolution_level.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "resolution_depth");
        assert_eq!(eval_with_registry(&g, &[Value::Int(0)], &reg), Value::Int(0));
        assert_eq!(eval_with_registry(&g, &[Value::Int(2)], &reg), Value::Int(2));
    }

    // types.iris: is_function_type, is_recursive_type, is_polymorphic, compute_type_hash
    {
        let src = load_iris("src/iris-programs/repr/types.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "is_function_type");
        assert_eq!(eval_with_registry(&g, &[Value::Int(5)], &reg), Value::Int(1)); // Arrow
        assert_eq!(eval_with_registry(&g, &[Value::Int(0)], &reg), Value::Int(0)); // Primitive
        let g2 = get_named(&frags, "is_recursive_type");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(3)], &reg), Value::Int(1)); // Recursive
        assert_eq!(eval_with_registry(&g2, &[Value::Int(0)], &reg), Value::Int(0)); // Primitive
        let g3 = get_named(&frags, "is_polymorphic");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(4)], &reg), Value::Int(1)); // ForAll
        assert_eq!(eval_with_registry(&g3, &[Value::Int(8)], &reg), Value::Int(1)); // Exists
        assert_eq!(eval_with_registry(&g3, &[Value::Int(0)], &reg), Value::Int(0)); // Primitive
        let g4 = get_named(&frags, "compute_type_hash");
        // Deterministic: same inputs produce same hash
        let h1 = eval_with_registry(&g4, &[Value::Int(5), Value::Int(42)], &reg);
        let h2 = eval_with_registry(&g4, &[Value::Int(5), Value::Int(42)], &reg);
        assert_eq!(h1, h2);
        // Different inputs produce different hashes
        let h3 = eval_with_registry(&g4, &[Value::Int(0), Value::Int(42)], &reg);
        assert_ne!(h1, h3);
    }

    // wire_format.iris: encode_u16_le
    {
        let src = load_iris("src/iris-programs/repr/wire_format.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "encode_u16_le");
        assert_eq!(eval_with_registry(&g, &[Value::Int(256)], &reg), Value::tuple(vec![Value::Int(0), Value::Int(1)]));
    }
}

/// Evaluate at least one function from every .iris file in src/iris-programs/deploy.
#[test]
fn eval_all_deploy_programs() {
    // elf_native.iris: encode_u16_le, validate_ident, validate_code_size, validate_elf
    {
        let src = load_iris("src/iris-programs/deploy/elf_native.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "encode_u16_le");
        assert_eq!(eval_with_registry(&g, &[Value::Int(258)], &reg),
            Value::tuple(vec![Value::Int(2), Value::Int(1)]));
        let g2 = get_named(&frags, "validate_ident");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(127), Value::Int(69), Value::Int(76), Value::Int(70),
            Value::Int(2), Value::Int(1), Value::Int(1)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g2, &[Value::Int(0), Value::Int(69), Value::Int(76), Value::Int(70),
            Value::Int(2), Value::Int(1), Value::Int(1)], &reg), Value::Int(0));
        let g3 = get_named(&frags, "validate_code_size");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(1000)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g3, &[Value::Int(0)], &reg), Value::Int(0));
        let g4 = get_named(&frags, "validate_elf");
        assert_eq!(eval_with_registry(&g4, &[Value::Int(1000), Value::Int(2)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g4, &[Value::Int(1000), Value::Int(7)], &reg), Value::Int(0));
    }

    // serialize_bytecode.iris: serialize_magic_bytes, encode_u32_le, is_valid_opcode, validate_bytecode_header
    {
        let src = load_iris("src/iris-programs/deploy/serialize_bytecode.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "serialize_magic_bytes");
        assert_eq!(eval_with_registry(&g, &[], &reg),
            Value::tuple(vec![Value::Int(73), Value::Int(82), Value::Int(73), Value::Int(83)]));
        let g2 = get_named(&frags, "encode_u32_le");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(1)], &reg),
            Value::tuple(vec![Value::Int(1), Value::Int(0), Value::Int(0), Value::Int(0)]));
        let g3 = get_named(&frags, "is_valid_opcode");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(16)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g3, &[Value::Int(0)], &reg), Value::Int(0));
        let g4 = get_named(&frags, "validate_bytecode_header");
        assert_eq!(eval_with_registry(&g4, &[Value::Int(1397051977), Value::Int(1)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g4, &[Value::Int(0), Value::Int(1)], &reg), Value::Int(0));
        let g5 = get_named(&frags, "roundtrip_header_check");
        assert_eq!(eval_with_registry(&g5, &[Value::Int(2), Value::Int(10), Value::Int(3)], &reg), Value::Int(1));
    }

    // shared_lib.iris: vm_arith, vm_compare, validate_invoke_inputs, instruction_wire_size
    {
        let src = load_iris("src/iris-programs/deploy/shared_lib.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "vm_arith");
        assert_eq!(eval_with_registry(&g, &[Value::Int(10), Value::Int(5), Value::Int(0)], &reg), Value::Int(15));
        assert_eq!(eval_with_registry(&g, &[Value::Int(10), Value::Int(3), Value::Int(1)], &reg), Value::Int(7));
        assert_eq!(eval_with_registry(&g, &[Value::Int(6), Value::Int(7), Value::Int(2)], &reg), Value::Int(42));
        let g2 = get_named(&frags, "vm_compare");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(3), Value::Int(5), Value::Int(1)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g2, &[Value::Int(5), Value::Int(5), Value::Int(1)], &reg), Value::Int(0));
        assert_eq!(eval_with_registry(&g2, &[Value::Int(5), Value::Int(5), Value::Int(0)], &reg), Value::Int(1));
        let g3 = get_named(&frags, "validate_invoke_inputs");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(2), Value::Int(2), Value::Int(0), Value::Int(0)], &reg), Value::Int(0));
        assert_eq!(eval_with_registry(&g3, &[Value::Int(2000), Value::Int(2), Value::Int(0), Value::Int(0)], &reg), Value::Int(-1));
        let g4 = get_named(&frags, "instruction_wire_size");
        assert_eq!(eval_with_registry(&g4, &[Value::Int(1)], &reg), Value::Int(5));
        assert_eq!(eval_with_registry(&g4, &[Value::Int(2)], &reg), Value::Int(4));
        assert_eq!(eval_with_registry(&g4, &[Value::Int(16)], &reg), Value::Int(3));
    }

    // standalone.iris: standalone_step, standalone_jump, parse_digit, count_digits
    {
        let src = load_iris("src/iris-programs/deploy/standalone.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "standalone_step");
        assert_eq!(eval_with_registry(&g, &[Value::Int(10), Value::Int(5), Value::Int(16)], &reg),
            Value::tuple(vec![Value::Int(15), Value::Int(0), Value::Int(-1)]));
        assert_eq!(eval_with_registry(&g, &[Value::Int(5), Value::Int(5), Value::Int(32)], &reg),
            Value::tuple(vec![Value::Int(1), Value::Int(0), Value::Int(-1)]));
        let g2 = get_named(&frags, "standalone_jump");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(10), Value::Int(3)], &reg), Value::Int(14));
        let g3 = get_named(&frags, "parse_digit");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(48)], &reg), Value::Int(0));
        assert_eq!(eval_with_registry(&g3, &[Value::Int(57)], &reg), Value::Int(9));
        assert_eq!(eval_with_registry(&g3, &[Value::Int(65)], &reg), Value::Int(-1));
        let g4 = get_named(&frags, "count_digits");
        assert_eq!(eval_with_registry(&g4, &[Value::Int(0)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g4, &[Value::Int(42)], &reg), Value::Int(2));
        assert_eq!(eval_with_registry(&g4, &[Value::Int(12345)], &reg), Value::Int(5));
    }
}

/// Evaluate at least one function from every .iris file in src/iris-programs/lsp.
#[test]
fn eval_all_lsp_programs() {
    // completion.iris: is_ident_char, keyword_category, make_completion_item, estimate_completion_count
    {
        let src = load_iris("src/iris-programs/lsp/completion.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "is_ident_char");
        assert_eq!(eval_with_registry(&g, &[Value::Int(97)], &reg), Value::Int(1)); // 'a'
        assert_eq!(eval_with_registry(&g, &[Value::Int(32)], &reg), Value::Int(0)); // space
        let g2 = get_named(&frags, "keyword_category");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(108)], &reg), Value::Int(0)); // 'l' -> binding
        assert_eq!(eval_with_registry(&g2, &[Value::Int(116)], &reg), Value::Int(1)); // 't' -> type
        let g3 = get_named(&frags, "make_completion_item");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(99), Value::Int(14), Value::Int(7)], &reg),
            Value::tuple(vec![Value::Int(99), Value::Int(14), Value::Int(7)]));
        let g4 = get_named(&frags, "estimate_completion_count");
        // Empty prefix: all items (21 + 80 + 35 = 136)
        assert_eq!(eval_with_registry(&g4, &[Value::Int(0), Value::Int(0)], &reg), Value::Int(136));
    }

    // diagnostics.iris: offset_to_position, position_line, position_col, should_report, total_diagnostics
    {
        let src = load_iris("src/iris-programs/lsp/diagnostics.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "offset_to_position");
        assert_eq!(eval_with_registry(&g, &[Value::Int(5), Value::Int(12)], &reg), Value::Int(500012));
        let g2 = get_named(&frags, "position_line");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(500012)], &reg), Value::Int(5));
        let g3 = get_named(&frags, "position_col");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(500012)], &reg), Value::Int(12));
        let g4 = get_named(&frags, "should_report");
        assert_eq!(eval_with_registry(&g4, &[Value::Int(1), Value::Int(2)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g4, &[Value::Int(3), Value::Int(2)], &reg), Value::Int(0));
        let g5 = get_named(&frags, "total_diagnostics");
        assert_eq!(eval_with_registry(&g5, &[Value::Int(1), Value::Int(2), Value::Int(3)], &reg), Value::Int(6));
        let g6 = get_named(&frags, "minimum_check_tier");
        assert_eq!(eval_with_registry(&g6, &[Value::Int(10), Value::Int(0), Value::Int(0), Value::Int(0)], &reg), Value::Int(0));
        assert_eq!(eval_with_registry(&g6, &[Value::Int(10), Value::Int(0), Value::Int(0), Value::Int(1)], &reg), Value::Int(3));
        let g7 = get_named(&frags, "run_diagnostic_pipeline");
        assert_eq!(eval_with_registry(&g7, &[Value::Int(0), Value::Int(0), Value::Int(0), Value::Int(0)], &reg),
            Value::tuple(vec![Value::Int(1), Value::Int(0), Value::Int(0)]));
        assert_eq!(eval_with_registry(&g7, &[Value::Int(1), Value::Int(3), Value::Int(2), Value::Int(1)], &reg),
            Value::tuple(vec![Value::Int(0), Value::Int(2), Value::Int(1)]))
    }

    // document.iris: doc_open, doc_close, doc_exists, doc_update, is_valid_version, can_open_document
    {
        let src = load_iris("src/iris-programs/lsp/document.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "doc_open");
        assert_eq!(eval_with_registry(&g, &[Value::Int(12345), Value::Int(500)], &reg),
            Value::tuple(vec![Value::Int(12345), Value::Int(500), Value::Int(1)]));
        let g2 = get_named(&frags, "doc_exists");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(1)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g2, &[Value::Int(0)], &reg), Value::Int(0));
        let g3 = get_named(&frags, "doc_update");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(12345), Value::Int(1), Value::Int(600)], &reg),
            Value::tuple(vec![Value::Int(12345), Value::Int(600), Value::Int(2)]));
        let g4 = get_named(&frags, "is_valid_version");
        assert_eq!(eval_with_registry(&g4, &[Value::Int(1), Value::Int(2)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g4, &[Value::Int(3), Value::Int(2)], &reg), Value::Int(0));
        let g5 = get_named(&frags, "can_open_document");
        assert_eq!(eval_with_registry(&g5, &[Value::Int(100)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g5, &[Value::Int(10000)], &reg), Value::Int(0));
        let g6 = get_named(&frags, "document_line_count");
        assert_eq!(eval_with_registry(&g6, &[Value::Int(1000), Value::Int(50)], &reg), Value::Int(51));
    }

    // hover.iris: is_ident_byte, scan_word_end, identify_primitive, resolve_hover
    {
        let src = load_iris("src/iris-programs/lsp/hover.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "is_ident_byte");
        assert_eq!(eval_with_registry(&g, &[Value::Int(97)], &reg), Value::Int(1)); // 'a'
        assert_eq!(eval_with_registry(&g, &[Value::Int(95)], &reg), Value::Int(1)); // '_'
        assert_eq!(eval_with_registry(&g, &[Value::Int(32)], &reg), Value::Int(0)); // space
        let g2 = get_named(&frags, "scan_word_end");
        // All ident bytes -> 4 (max scan)
        assert_eq!(eval_with_registry(&g2, &[Value::Int(97), Value::Int(98), Value::Int(99), Value::Int(100)], &reg), Value::Int(4));
        // First byte not ident -> 0
        assert_eq!(eval_with_registry(&g2, &[Value::Int(32), Value::Int(97), Value::Int(98), Value::Int(99)], &reg), Value::Int(0));
        let g3 = get_named(&frags, "identify_primitive");
        // "add" -> opcode=0, arity=2
        assert_eq!(eval_with_registry(&g3, &[Value::Int(3), Value::Int(97), Value::Int(100)], &reg), Value::Int(0 * 256 + 2));
        // unknown
        assert_eq!(eval_with_registry(&g3, &[Value::Int(10), Value::Int(120), Value::Int(121)], &reg), Value::Int(-1));
        let g4 = get_named(&frags, "resolve_hover");
        // "add" -> primitive (1)
        assert_eq!(eval_with_registry(&g4, &[Value::Int(3), Value::Int(97), Value::Int(100)], &reg), Value::Int(1));
    }

    // lsp_server.iris: parse_content_length, is_request, is_notification, transition_state, dispatch
    {
        let src = load_iris("src/iris-programs/lsp/lsp_server.iris");
        let (frags, reg) = compile_with_registry(&src);
        let g = get_named(&frags, "parse_content_length");
        assert_eq!(eval_with_registry(&g, &[Value::Int(100)], &reg), Value::Int(100));
        assert_eq!(eval_with_registry(&g, &[Value::Int(0)], &reg), Value::Int(-1));
        let g2 = get_named(&frags, "is_request");
        assert_eq!(eval_with_registry(&g2, &[Value::Int(1)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g2, &[Value::Int(0)], &reg), Value::Int(0));
        let g3 = get_named(&frags, "is_notification");
        assert_eq!(eval_with_registry(&g3, &[Value::Int(0)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g3, &[Value::Int(1)], &reg), Value::Int(0));
        let g4 = get_named(&frags, "transition_state");
        assert_eq!(eval_with_registry(&g4, &[Value::Int(0), Value::Int(0)], &reg), Value::Int(1));
        assert_eq!(eval_with_registry(&g4, &[Value::Int(1), Value::Int(7)], &reg), Value::Int(2));
        let g5 = get_named(&frags, "dispatch");
        assert_eq!(eval_with_registry(&g5, &[Value::Int(0), Value::Int(1)], &reg),
            Value::tuple(vec![Value::Int(1), Value::Int(0), Value::Int(0)]));
        assert_eq!(eval_with_registry(&g5, &[Value::Int(8), Value::Int(0)], &reg),
            Value::tuple(vec![Value::Int(0), Value::Int(0), Value::Int(1)]));
    }
}

// ===========================================================================
// src/iris-programs/repr/resolution_level.iris
// ===========================================================================

#[test]
fn resolution_depth_mapping() {
    let src = load_iris("src/iris-programs/repr/resolution_level.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "resolution_depth");
    assert_eq!(eval(&g, &[Value::Int(0)]), Value::Int(0));
    assert_eq!(eval(&g, &[Value::Int(1)]), Value::Int(1));
    assert_eq!(eval(&g, &[Value::Int(2)]), Value::Int(2));
}

#[test]
fn resolution_is_visible() {
    let src = load_iris("src/iris-programs/repr/resolution_level.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "is_visible");
    assert_eq!(eval(&g, &[Value::Int(0), Value::Int(0)]), Value::Int(1));
    assert_eq!(eval(&g, &[Value::Int(1), Value::Int(0)]), Value::Int(0));
    assert_eq!(eval(&g, &[Value::Int(1), Value::Int(1)]), Value::Int(1));
    assert_eq!(eval(&g, &[Value::Int(2), Value::Int(2)]), Value::Int(1));
}

#[test]
fn resolution_mixed_visibility() {
    let src = load_iris("src/iris-programs/repr/resolution_level.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "mixed_is_visible");
    // override=2, depth=2, default=0 -> visible
    assert_eq!(eval_with_registry(&g, &[Value::Int(2), Value::Int(2), Value::Int(0)], &reg), Value::Int(1));
    // override=-1 (use default=0), depth=1 -> hidden
    assert_eq!(eval_with_registry(&g, &[Value::Int(1), Value::Int(-1), Value::Int(0)], &reg), Value::Int(0));
}

#[test]
fn resolution_boundary_preservation() {
    let src = load_iris("src/iris-programs/repr/resolution_level.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "check_boundary_preservation");
    let test_graph = make_test_graph(3);
    assert_eq!(eval(&g, &[Value::Program(Rc::new(test_graph)), Value::Int(42)]), Value::Int(1));
}

// ===========================================================================
// src/iris-programs/repr/wire_format.iris
// ===========================================================================

#[test]
fn wire_format_encode_u16_le() {
    let src = load_iris("src/iris-programs/repr/wire_format.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "encode_u16_le");
    assert_eq!(eval(&g, &[Value::Int(256)]), Value::tuple(vec![Value::Int(0), Value::Int(1)]));
    assert_eq!(eval(&g, &[Value::Int(4660)]), Value::tuple(vec![Value::Int(52), Value::Int(18)]));
}

#[test]
fn wire_format_encode_u32_le() {
    let src = load_iris("src/iris-programs/repr/wire_format.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "encode_u32_le");
    assert_eq!(eval(&g, &[Value::Int(1)]), Value::tuple(vec![Value::Int(1), Value::Int(0), Value::Int(0), Value::Int(0)]));
    assert_eq!(eval(&g, &[Value::Int(256)]), Value::tuple(vec![Value::Int(0), Value::Int(1), Value::Int(0), Value::Int(0)]));
    assert_eq!(eval(&g, &[Value::Int(305419896)]), Value::tuple(vec![Value::Int(0x78), Value::Int(0x56), Value::Int(0x34), Value::Int(0x12)]));
}

#[test]
fn wire_format_roundtrip_u16() {
    let src = load_iris("src/iris-programs/repr/wire_format.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "roundtrip_u16");
    for v in [0, 1, 255, 256, 65535] {
        assert_eq!(eval_with_registry(&g, &[Value::Int(v)], &reg), Value::Int(1), "roundtrip_u16 failed for {}", v);
    }
}

#[test]
fn wire_format_roundtrip_u32() {
    let src = load_iris("src/iris-programs/repr/wire_format.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "roundtrip_u32");
    for v in [0, 1, 65535, 16777216] {
        assert_eq!(eval_with_registry(&g, &[Value::Int(v)], &reg), Value::Int(1), "roundtrip_u32 failed for {}", v);
    }
}

#[test]
fn wire_format_validate_header() {
    let src = load_iris("src/iris-programs/repr/wire_format.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "validate_header");
    assert_eq!(eval(&g, &[Value::Int(1397051977), Value::Int(1)]), Value::Int(1));
    assert_eq!(eval(&g, &[Value::Int(0), Value::Int(1)]), Value::Int(0));
    assert_eq!(eval(&g, &[Value::Int(1397051977), Value::Int(2)]), Value::Int(0));
}

#[test]
fn wire_format_magic_bytes() {
    let src = load_iris("src/iris-programs/repr/wire_format.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "magic_bytes");
    assert_eq!(eval(&g, &[]), Value::tuple(vec![Value::Int(73), Value::Int(82), Value::Int(73), Value::Int(83)]));
}

#[test]
fn wire_format_encode_node_kind() {
    let src = load_iris("src/iris-programs/repr/wire_format.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "encode_node_kind");
    assert_eq!(eval(&g, &[Value::Int(0)]), Value::Int(0));
    assert_eq!(eval(&g, &[Value::Int(19)]), Value::Int(19));
    assert_eq!(eval(&g, &[Value::Int(20)]), Value::Int(255));
    assert_eq!(eval(&g, &[Value::Int(-1)]), Value::Int(255));
}

#[test]
fn wire_format_decode_node() {
    let src = load_iris("src/iris-programs/repr/wire_format.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "decode_node");
    // Prim (kind 0) -> payload_size(0) = 2, consumed = 1+2 = 3
    let result = eval_with_registry(&g, &[Value::Int(0), Value::Int(5)], &reg);
    if let Value::Tuple(fields) = &result {
        assert_eq!(fields[0], Value::Int(0));
        assert_eq!(fields[2], Value::Int(3));
    } else { panic!("expected tuple, got {:?}", result); }
    // Invalid kind
    let result = eval_with_registry(&g, &[Value::Int(99), Value::Int(0)], &reg);
    if let Value::Tuple(fields) = &result {
        assert_eq!(fields[0], Value::Int(-1));
    }
}

#[test]
fn wire_format_payload_sizes() {
    let src = load_iris("src/iris-programs/repr/wire_format.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "payload_size");
    assert_eq!(eval(&g, &[Value::Int(0)]), Value::Int(2));   // Prim
    assert_eq!(eval(&g, &[Value::Int(1)]), Value::Int(1));   // Apply
    assert_eq!(eval(&g, &[Value::Int(5)]), Value::Int(14));  // Lit
    assert_eq!(eval(&g, &[Value::Int(6)]), Value::Int(33));  // Ref
    assert_eq!(eval(&g, &[Value::Int(17)]), Value::Int(25)); // Guard
}

// ===========================================================================
// src/iris-programs/repr/hash.iris
// ===========================================================================

#[test]
fn hash_mix_deterministic() {
    let src = load_iris("src/iris-programs/repr/hash.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "hash_mix");
    let h1 = eval(&g, &[Value::Int(0), Value::Int(42)]);
    let h2 = eval(&g, &[Value::Int(0), Value::Int(42)]);
    assert_eq!(h1, h2, "hash_mix must be deterministic");
}

#[test]
fn hash_mix_distinct_inputs() {
    let src = load_iris("src/iris-programs/repr/hash.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "hash_mix");
    let h1 = eval(&g, &[Value::Int(0), Value::Int(1)]);
    let h2 = eval(&g, &[Value::Int(0), Value::Int(2)]);
    assert_ne!(h1, h2, "hash_mix(0,1) != hash_mix(0,2)");
    let h3 = eval(&g, &[Value::Int(0), Value::Int(100)]);
    let h4 = eval(&g, &[Value::Int(0), Value::Int(101)]);
    assert_ne!(h3, h4, "hash_mix(0,100) != hash_mix(0,101)");
}

#[test]
fn hash_mix_positive_output() {
    let src = load_iris("src/iris-programs/repr/hash.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "hash_mix");
    for i in 0..20 {
        if let Value::Int(v) = eval(&g, &[Value::Int(i), Value::Int(i * 7 + 3)]) {
            assert!(v >= 0, "hash_mix output should be non-negative, got {}", v);
        }
    }
}

#[test]
fn hash_verify_distinct() {
    let src = load_iris("src/iris-programs/repr/hash.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "verify_distinct");
    assert_eq!(eval_with_registry(&g, &[Value::Int(5), Value::Int(5)], &reg), Value::Int(1));
    assert_eq!(eval_with_registry(&g, &[Value::Int(1), Value::Int(2)], &reg), Value::Int(1));
    assert_eq!(eval_with_registry(&g, &[Value::Int(0), Value::Int(100)], &reg), Value::Int(1));
}

#[test]
fn hash_compute_node_id_deterministic() {
    let src = load_iris("src/iris-programs/repr/hash.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "compute_node_id");
    let inputs = vec![Value::Int(0), Value::Int(42), Value::Int(2), Value::Int(0), Value::Int(0), Value::Int(0), Value::Int(5)];
    let h1 = eval_with_registry(&g, &inputs, &reg);
    let h2 = eval_with_registry(&g, &inputs, &reg);
    assert_eq!(h1, h2, "compute_node_id must be deterministic");
    if let Value::Int(v) = h1 { assert!(v > 0, "node ID should be positive"); }
}

#[test]
fn hash_different_kinds_different_ids() {
    let src = load_iris("src/iris-programs/repr/hash.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "compute_node_id");
    let h_prim = eval_with_registry(&g, &[Value::Int(0), Value::Int(42), Value::Int(2), Value::Int(0), Value::Int(0), Value::Int(0), Value::Int(5)], &reg);
    let h_lit = eval_with_registry(&g, &[Value::Int(5), Value::Int(42), Value::Int(2), Value::Int(0), Value::Int(0), Value::Int(0), Value::Int(5)], &reg);
    assert_ne!(h_prim, h_lit, "Prim and Lit nodes should have different IDs");
}

// ===========================================================================
// src/iris-programs/deploy/elf_native.iris
// ===========================================================================

#[test]
fn elf_ident_bytes_correct() {
    let src = load_iris("src/iris-programs/deploy/elf_native.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "build_ident");
    assert_eq!(eval(&g, &[]), Value::tuple(vec![
        Value::Int(127), Value::Int(69), Value::Int(76), Value::Int(70),
        Value::Int(2), Value::Int(1), Value::Int(1), Value::Int(0),
    ]));
}

#[test]
fn elf_type_machine_bytes() {
    let src = load_iris("src/iris-programs/deploy/elf_native.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "build_elf_type_machine_bytes");
    let result = eval_with_registry(&g, &[], &reg);
    assert_eq!(result, Value::tuple(vec![Value::Int(2), Value::Int(0), Value::Int(62), Value::Int(0)]));
}

#[test]
fn elf_entry_point_bytes() {
    let src = load_iris("src/iris-programs/deploy/elf_native.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "build_elf_entry_bytes");
    let result = eval_with_registry(&g, &[], &reg);
    if let Value::Tuple(bytes) = result {
        assert_eq!(bytes[0], Value::Int(0x78)); // 4194424 & 0xFF
        assert_eq!(bytes[1], Value::Int(0x00));
        assert_eq!(bytes[2], Value::Int(0x40));
    } else { panic!("expected tuple"); }
}

#[test]
fn elf_phoff_bytes() {
    let src = load_iris("src/iris-programs/deploy/elf_native.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "build_elf_phoff_bytes");
    let result = eval_with_registry(&g, &[], &reg);
    if let Value::Tuple(bytes) = result {
        assert_eq!(bytes[0], Value::Int(64));
        assert_eq!(bytes[1], Value::Int(0));
    } else { panic!("expected tuple"); }
}

#[test]
fn elf_phdr_type_bytes() {
    let src = load_iris("src/iris-programs/deploy/elf_native.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "build_phdr_type_bytes");
    assert_eq!(eval_with_registry(&g, &[], &reg), Value::tuple(vec![Value::Int(1), Value::Int(0), Value::Int(0), Value::Int(0)]));
}

#[test]
fn elf_phdr_flags_bytes() {
    let src = load_iris("src/iris-programs/deploy/elf_native.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "build_phdr_flags_bytes");
    assert_eq!(eval_with_registry(&g, &[], &reg), Value::tuple(vec![Value::Int(5), Value::Int(0), Value::Int(0), Value::Int(0)]));
}

#[test]
fn elf_validate_ident() {
    let src = load_iris("src/iris-programs/deploy/elf_native.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "validate_ident");
    assert_eq!(eval(&g, &[Value::Int(127), Value::Int(69), Value::Int(76), Value::Int(70), Value::Int(2), Value::Int(1), Value::Int(1)]), Value::Int(1));
    assert_eq!(eval(&g, &[Value::Int(0), Value::Int(69), Value::Int(76), Value::Int(70), Value::Int(2), Value::Int(1), Value::Int(1)]), Value::Int(0));
}

#[test]
fn elf_validate_complete() {
    let src = load_iris("src/iris-programs/deploy/elf_native.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "validate_elf");
    assert_eq!(eval_with_registry(&g, &[Value::Int(1000), Value::Int(2)], &reg), Value::Int(1));
    assert_eq!(eval_with_registry(&g, &[Value::Int(1000), Value::Int(7)], &reg), Value::Int(0));
    assert_eq!(eval_with_registry(&g, &[Value::Int(0), Value::Int(2)], &reg), Value::Int(0));
}

#[test]
fn elf_sizes_bytes() {
    let src = load_iris("src/iris-programs/deploy/elf_native.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "build_elf_sizes_bytes");
    let result = eval_with_registry(&g, &[], &reg);
    if let Value::Tuple(bytes) = result {
        // ehsize = 64 -> (64, 0)
        assert_eq!(bytes[0], Value::Int(64));
        assert_eq!(bytes[1], Value::Int(0));
        // phentsize = 56 -> (56, 0)
        assert_eq!(bytes[2], Value::Int(56));
        assert_eq!(bytes[3], Value::Int(0));
        // phnum = 1 -> (1, 0)
        assert_eq!(bytes[4], Value::Int(1));
        assert_eq!(bytes[5], Value::Int(0));
    } else { panic!("expected tuple"); }
}

// ===========================================================================
// src/iris-programs/deploy/serialize_bytecode.iris
// ===========================================================================

#[test]
fn serialize_magic_bytes() {
    let src = load_iris("src/iris-programs/deploy/serialize_bytecode.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "serialize_magic_bytes");
    assert_eq!(eval(&g, &[]), Value::tuple(vec![Value::Int(73), Value::Int(82), Value::Int(73), Value::Int(83)]));
}

#[test]
fn serialize_encode_instruction_simple() {
    let src = load_iris("src/iris-programs/deploy/serialize_bytecode.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "encode_instruction");
    // Add (opcode 16) with source_node=0, no operand
    let result = eval_with_registry(&g, &[Value::Int(0), Value::Int(16), Value::Int(0)], &reg);
    if let Value::Tuple(bytes) = &result {
        assert_eq!(bytes[0], Value::Int(3)); // byte_count = 3
        assert_eq!(bytes[3], Value::Int(16)); // tag = Add
    } else { panic!("expected tuple, got {:?}", result); }
}

#[test]
fn serialize_encode_instruction_with_u16_operand() {
    let src = load_iris("src/iris-programs/deploy/serialize_bytecode.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "encode_instruction");
    // LoadConst (opcode 1) with operand 258
    let result = eval_with_registry(&g, &[Value::Int(0), Value::Int(1), Value::Int(258)], &reg);
    if let Value::Tuple(bytes) = &result {
        assert_eq!(bytes[0], Value::Int(5)); // 5 bytes
        assert_eq!(bytes[3], Value::Int(1)); // tag = LoadConst
        assert_eq!(bytes[4], Value::Int(2)); // operand low
        assert_eq!(bytes[5], Value::Int(1)); // operand high
    } else { panic!("expected tuple"); }
}

#[test]
fn serialize_encode_instruction_with_u8_operand() {
    let src = load_iris("src/iris-programs/deploy/serialize_bytecode.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "encode_instruction");
    // LoadArg (opcode 2) with operand 3
    let result = eval_with_registry(&g, &[Value::Int(0), Value::Int(2), Value::Int(3)], &reg);
    if let Value::Tuple(bytes) = &result {
        assert_eq!(bytes[0], Value::Int(4)); // 4 bytes
        assert_eq!(bytes[3], Value::Int(2)); // tag = LoadArg
        assert_eq!(bytes[4], Value::Int(3)); // operand
    } else { panic!("expected tuple"); }
}

#[test]
fn serialize_encode_int_value() {
    let src = load_iris("src/iris-programs/deploy/serialize_bytecode.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "encode_int_value");
    let result = eval_with_registry(&g, &[Value::Int(42)], &reg);
    if let Value::Tuple(bytes) = &result {
        assert_eq!(bytes[0], Value::Int(0)); // tag = Int
        assert_eq!(bytes[1], Value::Int(42)); // low byte
    } else { panic!("expected tuple"); }
}

#[test]
fn serialize_roundtrip_header() {
    let src = load_iris("src/iris-programs/deploy/serialize_bytecode.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "roundtrip_header_check");
    assert_eq!(eval_with_registry(&g, &[Value::Int(2), Value::Int(10), Value::Int(3)], &reg), Value::Int(1));
    assert_eq!(eval_with_registry(&g, &[Value::Int(0), Value::Int(0), Value::Int(0)], &reg), Value::Int(1));
}

#[test]
fn serialize_valid_opcodes() {
    let src = load_iris("src/iris-programs/deploy/serialize_bytecode.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "is_valid_opcode");
    assert_eq!(eval(&g, &[Value::Int(1)]), Value::Int(1));
    assert_eq!(eval(&g, &[Value::Int(16)]), Value::Int(1));
    assert_eq!(eval(&g, &[Value::Int(32)]), Value::Int(1));
    assert_eq!(eval(&g, &[Value::Int(240)]), Value::Int(1));
    assert_eq!(eval(&g, &[Value::Int(0)]), Value::Int(0));
    assert_eq!(eval(&g, &[Value::Int(100)]), Value::Int(0));
}

// ===========================================================================
// src/iris-programs/deploy/standalone.iris
// ===========================================================================

#[test]
fn standalone_vm_step_add() {
    let src = load_iris("src/iris-programs/deploy/standalone.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "standalone_step");
    assert_eq!(eval(&g, &[Value::Int(10), Value::Int(5), Value::Int(16)]),
        Value::tuple(vec![Value::Int(15), Value::Int(0), Value::Int(-1)]));
}

#[test]
fn standalone_vm_step_sub() {
    let src = load_iris("src/iris-programs/deploy/standalone.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "standalone_step");
    assert_eq!(eval(&g, &[Value::Int(10), Value::Int(3), Value::Int(17)]),
        Value::tuple(vec![Value::Int(7), Value::Int(0), Value::Int(-1)]));
}

#[test]
fn standalone_vm_step_mul() {
    let src = load_iris("src/iris-programs/deploy/standalone.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "standalone_step");
    assert_eq!(eval(&g, &[Value::Int(6), Value::Int(7), Value::Int(18)]),
        Value::tuple(vec![Value::Int(42), Value::Int(0), Value::Int(-1)]));
}

#[test]
fn standalone_vm_step_div_zero() {
    let src = load_iris("src/iris-programs/deploy/standalone.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "standalone_step");
    assert_eq!(eval(&g, &[Value::Int(10), Value::Int(0), Value::Int(19)]),
        Value::tuple(vec![Value::Int(0), Value::Int(0), Value::Int(-1)]));
}

#[test]
fn standalone_vm_step_comparison() {
    let src = load_iris("src/iris-programs/deploy/standalone.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "standalone_step");
    assert_eq!(eval(&g, &[Value::Int(5), Value::Int(5), Value::Int(32)]),
        Value::tuple(vec![Value::Int(1), Value::Int(0), Value::Int(-1)]));
    assert_eq!(eval(&g, &[Value::Int(3), Value::Int(5), Value::Int(34)]),
        Value::tuple(vec![Value::Int(1), Value::Int(0), Value::Int(-1)]));
}

#[test]
fn standalone_vm_step_neg_abs() {
    let src = load_iris("src/iris-programs/deploy/standalone.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "standalone_step");
    assert_eq!(eval(&g, &[Value::Int(-7), Value::Int(99), Value::Int(21)]),
        Value::tuple(vec![Value::Int(7), Value::Int(99), Value::Int(0)]));
    assert_eq!(eval(&g, &[Value::Int(-7), Value::Int(0), Value::Int(22)]),
        Value::tuple(vec![Value::Int(7), Value::Int(0), Value::Int(0)]));
}

#[test]
fn standalone_vm_run_execution() {
    let src = load_iris("src/iris-programs/deploy/standalone.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "vm_run");
    let instructions = Value::tuple(vec![Value::Int(16)]); // Single Add
    let result = eval_with_registry(&g, &[Value::Int(10), Value::Int(3), instructions], &reg);
    if let Value::Tuple(fields) = &result {
        assert_eq!(fields[0], Value::Int(13)); // 10 + 3
        assert_eq!(fields[1], Value::Int(1)); // 1 step
    } else { panic!("expected tuple, got {:?}", result); }
}

#[test]
fn standalone_fold_step() {
    let src = load_iris("src/iris-programs/deploy/standalone.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "fold_step");
    assert_eq!(eval(&g, &[Value::Int(10), Value::Int(5), Value::Int(0)]), Value::Int(15));
    assert_eq!(eval(&g, &[Value::Int(4), Value::Int(3), Value::Int(2)]), Value::Int(12));
    assert_eq!(eval(&g, &[Value::Int(7), Value::Int(3), Value::Int(7)]), Value::Int(3));
    assert_eq!(eval(&g, &[Value::Int(7), Value::Int(3), Value::Int(8)]), Value::Int(7));
}

// ===========================================================================
// src/iris-programs/deploy/shared_lib.iris
// ===========================================================================

#[test]
fn shared_lib_validate_inputs() {
    let src = load_iris("src/iris-programs/deploy/shared_lib.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "validate_invoke_inputs");
    assert_eq!(eval_with_registry(&g, &[Value::Int(2), Value::Int(2), Value::Int(0), Value::Int(0)], &reg), Value::Int(0));
    assert_eq!(eval_with_registry(&g, &[Value::Int(2000), Value::Int(2), Value::Int(0), Value::Int(0)], &reg), Value::Int(-1));
    assert_eq!(eval_with_registry(&g, &[Value::Int(1), Value::Int(3), Value::Int(0), Value::Int(0)], &reg), Value::Int(-1));
    assert_eq!(eval_with_registry(&g, &[Value::Int(2), Value::Int(2), Value::Int(1), Value::Int(0)], &reg), Value::Int(-1));
}

#[test]
fn shared_lib_vm_arith() {
    let src = load_iris("src/iris-programs/deploy/shared_lib.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "vm_arith");
    assert_eq!(eval(&g, &[Value::Int(10), Value::Int(5), Value::Int(0)]), Value::Int(15));
    assert_eq!(eval(&g, &[Value::Int(10), Value::Int(3), Value::Int(1)]), Value::Int(7));
    assert_eq!(eval(&g, &[Value::Int(6), Value::Int(7), Value::Int(2)]), Value::Int(42));
    assert_eq!(eval(&g, &[Value::Int(20), Value::Int(4), Value::Int(3)]), Value::Int(5));
    assert_eq!(eval(&g, &[Value::Int(20), Value::Int(0), Value::Int(3)]), Value::Int(0));
}

#[test]
fn shared_lib_validate() {
    let src = load_iris("src/iris-programs/deploy/shared_lib.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "validate_shared_lib");
    assert_eq!(eval(&g, &[Value::Int(100), Value::Int(2)]), Value::Int(1));
    assert_eq!(eval(&g, &[Value::Int(5), Value::Int(2)]), Value::Int(0));
    assert_eq!(eval(&g, &[Value::Int(100), Value::Int(7)]), Value::Int(0));
}

// ===========================================================================
// src/iris-programs/lsp/lsp_server.iris
// ===========================================================================

#[test]
fn lsp_server_state_transitions() {
    let src = load_iris("src/iris-programs/lsp/lsp_server.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "transition_state");
    assert_eq!(eval_with_registry(&g, &[Value::Int(0), Value::Int(0)], &reg), Value::Int(1));
    assert_eq!(eval_with_registry(&g, &[Value::Int(1), Value::Int(7)], &reg), Value::Int(2));
    assert_eq!(eval_with_registry(&g, &[Value::Int(2), Value::Int(8)], &reg), Value::Int(3));
    assert_eq!(eval_with_registry(&g, &[Value::Int(1), Value::Int(5)], &reg), Value::Int(1));
}

#[test]
fn lsp_server_method_validation() {
    let src = load_iris("src/iris-programs/lsp/lsp_server.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "is_method_valid");
    assert_eq!(eval_with_registry(&g, &[Value::Int(0), Value::Int(0)], &reg), Value::Int(1));
    assert_eq!(eval_with_registry(&g, &[Value::Int(0), Value::Int(5)], &reg), Value::Int(0));
    assert_eq!(eval_with_registry(&g, &[Value::Int(1), Value::Int(5)], &reg), Value::Int(1));
    assert_eq!(eval_with_registry(&g, &[Value::Int(1), Value::Int(8)], &reg), Value::Int(0));
    assert_eq!(eval_with_registry(&g, &[Value::Int(2), Value::Int(8)], &reg), Value::Int(1));
    assert_eq!(eval_with_registry(&g, &[Value::Int(2), Value::Int(5)], &reg), Value::Int(0));
}

#[test]
fn lsp_server_dispatch() {
    let src = load_iris("src/iris-programs/lsp/lsp_server.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "dispatch");
    assert_eq!(eval_with_registry(&g, &[Value::Int(0), Value::Int(1)], &reg),
        Value::tuple(vec![Value::Int(1), Value::Int(0), Value::Int(0)]));
    assert_eq!(eval_with_registry(&g, &[Value::Int(2), Value::Int(0)], &reg),
        Value::tuple(vec![Value::Int(0), Value::Int(1), Value::Int(0)]));
    assert_eq!(eval_with_registry(&g, &[Value::Int(8), Value::Int(0)], &reg),
        Value::tuple(vec![Value::Int(0), Value::Int(0), Value::Int(1)]));
}

#[test]
fn lsp_server_process_message() {
    let src = load_iris("src/iris-programs/lsp/lsp_server.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "process_message");
    // Initialize in uninitialized state
    let result = eval_with_registry(&g, &[Value::Int(0), Value::Int(0), Value::Int(1)], &reg);
    if let Value::Tuple(f) = &result {
        assert_eq!(f[0], Value::Int(1)); // new state = initialized
        assert_eq!(f[1], Value::Int(1)); // needs response
        assert_eq!(f[4], Value::Int(0)); // no error
    } else { panic!("expected tuple"); }
    // Completion in uninitialized (invalid)
    let result = eval_with_registry(&g, &[Value::Int(0), Value::Int(5), Value::Int(1)], &reg);
    if let Value::Tuple(f) = &result {
        assert_eq!(f[0], Value::Int(0)); // state unchanged
        assert_eq!(f[4], Value::Int(-32600)); // InvalidRequest
    } else { panic!("expected tuple"); }
}

#[test]
fn lsp_server_content_length() {
    let src = load_iris("src/iris-programs/lsp/lsp_server.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "parse_content_length");
    assert_eq!(eval(&g, &[Value::Int(100)]), Value::Int(100));
    assert_eq!(eval(&g, &[Value::Int(0)]), Value::Int(-1));
    assert_eq!(eval(&g, &[Value::Int(99999999)]), Value::Int(-1));
}

// ===========================================================================
// src/iris-programs/lsp/diagnostics.iris
// ===========================================================================

#[test]
fn diagnostics_pipeline() {
    let src = load_iris("src/iris-programs/lsp/diagnostics.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "run_diagnostic_pipeline");
    assert_eq!(eval(&g, &[Value::Int(0), Value::Int(0), Value::Int(0), Value::Int(0)]),
        Value::tuple(vec![Value::Int(1), Value::Int(0), Value::Int(0)]));
    assert_eq!(eval(&g, &[Value::Int(1), Value::Int(3), Value::Int(2), Value::Int(1)]),
        Value::tuple(vec![Value::Int(0), Value::Int(2), Value::Int(1)]));
}

#[test]
fn diagnostics_make_and_inspect() {
    let src = load_iris("src/iris-programs/lsp/diagnostics.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "make_diagnostic");
    assert_eq!(eval(&g, &[Value::Int(5), Value::Int(10), Value::Int(5), Value::Int(20), Value::Int(1), Value::Int(42)]),
        Value::tuple(vec![Value::Int(5), Value::Int(10), Value::Int(5), Value::Int(20), Value::Int(1), Value::Int(42)]));
}

#[test]
fn diagnostics_compare_sort() {
    let src = load_iris("src/iris-programs/lsp/diagnostics.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "compare_diagnostics");
    assert_eq!(eval(&g, &[Value::Int(1), Value::Int(5), Value::Int(1), Value::Int(5)]), Value::Int(0));
    assert_eq!(eval(&g, &[Value::Int(1), Value::Int(5), Value::Int(2), Value::Int(5)]), Value::Int(-1));
    assert_eq!(eval(&g, &[Value::Int(1), Value::Int(10), Value::Int(1), Value::Int(5)]), Value::Int(1));
}

#[test]
fn diagnostics_total_count() {
    let src = load_iris("src/iris-programs/lsp/diagnostics.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "total_diagnostics");
    assert_eq!(eval(&g, &[Value::Int(1), Value::Int(2), Value::Int(3)]), Value::Int(6));
}

#[test]
fn diagnostics_should_report() {
    let src = load_iris("src/iris-programs/lsp/diagnostics.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "should_report");
    assert_eq!(eval(&g, &[Value::Int(1), Value::Int(2)]), Value::Int(1)); // error <= warning
    assert_eq!(eval(&g, &[Value::Int(3), Value::Int(2)]), Value::Int(0)); // info > warning
}

// ===========================================================================
// src/iris-programs/lsp/hover.iris
// ===========================================================================

#[test]
fn hover_identify_primitive() {
    let src = load_iris("src/iris-programs/lsp/hover.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "identify_primitive");
    // "add" = len=3, 'a'=97, 'd'=100
    assert_eq!(eval(&g, &[Value::Int(3), Value::Int(97), Value::Int(100)]), Value::Int(0 * 256 + 2));
    // "eq" = len=2, 'e'=101, 'q'=113
    assert_eq!(eval(&g, &[Value::Int(2), Value::Int(101), Value::Int(113)]), Value::Int(32 * 256 + 2));
    // unknown
    assert_eq!(eval(&g, &[Value::Int(10), Value::Int(120), Value::Int(121)]), Value::Int(-1));
}

#[test]
fn hover_resolve_type() {
    let src = load_iris("src/iris-programs/lsp/hover.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "resolve_hover");
    // "add" -> primitive (1)
    assert_eq!(eval_with_registry(&g, &[Value::Int(3), Value::Int(97), Value::Int(100)], &reg), Value::Int(1));
}

#[test]
fn hover_is_ident_byte() {
    let src = load_iris("src/iris-programs/lsp/hover.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "is_ident_byte");
    assert_eq!(eval(&g, &[Value::Int(97)]), Value::Int(1)); // 'a'
    assert_eq!(eval(&g, &[Value::Int(122)]), Value::Int(1)); // 'z'
    assert_eq!(eval(&g, &[Value::Int(95)]), Value::Int(1)); // '_'
    assert_eq!(eval(&g, &[Value::Int(48)]), Value::Int(1)); // '0'
    assert_eq!(eval(&g, &[Value::Int(32)]), Value::Int(0)); // space
}

// ===========================================================================
// src/iris-programs/lsp/document.iris
// ===========================================================================

#[test]
fn document_open_close_cycle() {
    let src = load_iris("src/iris-programs/lsp/document.iris");
    let (frags, _) = compile_with_registry(&src);
    let g_open = get_named(&frags, "doc_open");
    let g_close = get_named(&frags, "doc_close");
    let g_exists = get_named(&frags, "doc_exists");
    assert_eq!(eval(&g_open, &[Value::Int(12345), Value::Int(500)]),
        Value::tuple(vec![Value::Int(12345), Value::Int(500), Value::Int(1)]));
    assert_eq!(eval(&g_exists, &[Value::Int(1)]), Value::Int(1));
    assert_eq!(eval(&g_exists, &[Value::Int(0)]), Value::Int(0));
    assert_eq!(eval(&g_close, &[Value::Int(12345)]), Value::Int(0));
}

#[test]
fn document_update_version() {
    let src = load_iris("src/iris-programs/lsp/document.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "doc_update");
    assert_eq!(eval(&g, &[Value::Int(12345), Value::Int(1), Value::Int(600)]),
        Value::tuple(vec![Value::Int(12345), Value::Int(600), Value::Int(2)]));
}

#[test]
fn document_version_validation() {
    let src = load_iris("src/iris-programs/lsp/document.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "is_valid_version");
    assert_eq!(eval(&g, &[Value::Int(1), Value::Int(2)]), Value::Int(1));
    assert_eq!(eval(&g, &[Value::Int(3), Value::Int(2)]), Value::Int(0));
}

// ===========================================================================
// src/iris-programs/lsp/completion.iris
// ===========================================================================

#[test]
fn completion_ident_char() {
    let src = load_iris("src/iris-programs/lsp/completion.iris");
    let (frags, _) = compile_with_registry(&src);
    let g = get_named(&frags, "is_ident_char");
    assert_eq!(eval(&g, &[Value::Int(97)]), Value::Int(1));  // 'a'
    assert_eq!(eval(&g, &[Value::Int(65)]), Value::Int(1));  // 'A'
    assert_eq!(eval(&g, &[Value::Int(48)]), Value::Int(1));  // '0'
    assert_eq!(eval(&g, &[Value::Int(95)]), Value::Int(1));  // '_'
    assert_eq!(eval(&g, &[Value::Int(32)]), Value::Int(0));  // space
}

#[test]
fn completion_estimate_count() {
    let src = load_iris("src/iris-programs/lsp/completion.iris");
    let (frags, reg) = compile_with_registry(&src);
    let g = get_named(&frags, "estimate_completion_count");
    let result = eval_with_registry(&g, &[Value::Int(0), Value::Int(0)], &reg);
    if let Value::Int(v) = result {
        assert_eq!(v, 136); // 21 + 80 + 35
    } else { panic!("expected Int"); }
}

// ===========================================================================
// Cross-cutting
// ===========================================================================

#[test]
fn all_programs_parse_deterministically() {
    let dirs = ["src/iris-programs/repr", "src/iris-programs/deploy", "src/iris-programs/lsp"];
    for dir in &dirs {
        let full_dir = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), dir);
        let entries = fs::read_dir(&full_dir).unwrap();
        for entry in entries {
            let path = entry.unwrap().path();
            if path.extension().map(|e| e == "iris").unwrap_or(false) {
                let src = fs::read_to_string(&path).unwrap();
                let m1 = iris_bootstrap::syntax::parse(&src).unwrap();
                let m2 = iris_bootstrap::syntax::parse(&src).unwrap();
                assert_eq!(m1.items.len(), m2.items.len(), "non-deterministic parse for {}", path.display());
            }
        }
    }
}

#[test]
fn no_empty_programs() {
    let dirs = ["src/iris-programs/repr", "src/iris-programs/deploy", "src/iris-programs/lsp"];
    for dir in &dirs {
        let full_dir = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), dir);
        let entries = fs::read_dir(&full_dir).unwrap();
        for entry in entries {
            let path = entry.unwrap().path();
            if path.extension().map(|e| e == "iris").unwrap_or(false) {
                let src = fs::read_to_string(&path).unwrap();
                assert!(!src.trim().is_empty(), "{} is empty", path.display());
                let module = iris_bootstrap::syntax::parse(&src).unwrap();
                assert!(!module.items.is_empty(), "{} has no items", path.display());
            }
        }
    }
}
