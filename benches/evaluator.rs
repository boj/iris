//! Criterion micro-benchmarks for IRIS evaluator hotpaths.
//!
//! Run with: cargo bench --bench evaluator
//! Results in: target/criterion/

use std::collections::{BTreeMap, HashMap};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use iris_bootstrap::{evaluate, evaluate_with_fragments};
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::fragment::FragmentId;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Graph builders
// ---------------------------------------------------------------------------

fn make_node(id: u64, kind: NodeKind, payload: NodePayload) -> (NodeId, Node) {
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind,
            type_sig: TypeId(0),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload,
        },
    )
}

fn make_graph(nodes: Vec<(NodeId, Node)>, edges: Vec<Edge>, root: u64) -> SemanticGraph {
    let node_map: HashMap<NodeId, Node> = nodes.into_iter().collect();
    SemanticGraph {
        root: NodeId(root),
        nodes: node_map,
        edges,
        type_env: TypeEnv {
            types: BTreeMap::new(),
        },
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

/// Lit(42) — simplest possible program
fn lit_graph(val: i64) -> SemanticGraph {
    let node = make_node(
        1,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: val.to_le_bytes().to_vec(),
        },
    );
    make_graph(vec![node], vec![], 1)
}

/// add(x, y) — single Prim node with two Lit inputs
fn add_graph(a: i64, b: i64) -> SemanticGraph {
    let add_node = make_node(1, NodeKind::Prim, NodePayload::Prim { opcode: 0x01 });
    let lit_a = make_node(
        2,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: a.to_le_bytes().to_vec(),
        },
    );
    let lit_b = make_node(
        3,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: b.to_le_bytes().to_vec(),
        },
    );
    make_graph(
        vec![add_node, lit_a, lit_b],
        vec![
            Edge { source: NodeId(1), target: NodeId(2), port: 0, label: EdgeLabel::Argument },
            Edge { source: NodeId(1), target: NodeId(3), port: 1, label: EdgeLabel::Argument },
        ],
        1,
    )
}

/// Nested add chain: add(add(add(..., 1), 1), 1) — N levels deep
fn nested_add_graph(depth: u64) -> SemanticGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut next_id = 1u64;

    let base = make_node(
        next_id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: 0i64.to_le_bytes().to_vec(),
        },
    );
    let base_id = next_id;
    nodes.push(base);
    next_id += 1;

    let mut current = base_id;

    for _ in 0..depth {
        let lit = make_node(
            next_id,
            NodeKind::Lit,
            NodePayload::Lit {
                type_tag: 0x00,
                value: 1i64.to_le_bytes().to_vec(),
            },
        );
        let lit_id = next_id;
        nodes.push(lit);
        next_id += 1;

        let add = make_node(next_id, NodeKind::Prim, NodePayload::Prim { opcode: 0x01 });
        let add_id = next_id;
        nodes.push(add);
        next_id += 1;

        edges.push(Edge { source: NodeId(add_id), target: NodeId(current), port: 0, label: EdgeLabel::Argument });
        edges.push(Edge { source: NodeId(add_id), target: NodeId(lit_id), port: 1, label: EdgeLabel::Argument });

        current = add_id;
    }

    make_graph(nodes, edges, current)
}

/// fold(0, +, [1..n]) via Fold node
fn fold_sum_graph(n: i64) -> SemanticGraph {
    let fold = make_node(1, NodeKind::Fold, NodePayload::Fold { recursion_descriptor: vec![] });
    let base = make_node(
        2,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: 0i64.to_le_bytes().to_vec(),
        },
    );
    let step = make_node(3, NodeKind::Prim, NodePayload::Prim { opcode: 0x01 });

    let mut list_nodes = Vec::new();
    let mut list_edges = Vec::new();
    let tuple = make_node(4, NodeKind::Tuple, NodePayload::Tuple);
    list_nodes.push(tuple);

    for i in 0..n {
        let id = 10 + i as u64;
        let lit = make_node(
            id,
            NodeKind::Lit,
            NodePayload::Lit {
                type_tag: 0x00,
                value: (i + 1).to_le_bytes().to_vec(),
            },
        );
        list_nodes.push(lit);
        list_edges.push(Edge {
            source: NodeId(4),
            target: NodeId(id),
            port: i as u8,
            label: EdgeLabel::Argument,
        });
    }

    let mut all_nodes = vec![fold, base, step];
    all_nodes.extend(list_nodes);

    let mut all_edges = vec![
        Edge { source: NodeId(1), target: NodeId(2), port: 0, label: EdgeLabel::Argument },
        Edge { source: NodeId(1), target: NodeId(3), port: 1, label: EdgeLabel::Argument },
        Edge { source: NodeId(1), target: NodeId(4), port: 2, label: EdgeLabel::Argument },
    ];
    all_edges.extend(list_edges);

    make_graph(all_nodes, all_edges, 1)
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

fn bench_lit_eval(c: &mut Criterion) {
    let graph = lit_graph(42);
    c.bench_function("eval_lit", |b| {
        b.iter(|| evaluate(black_box(&graph), black_box(&[])))
    });
}

fn bench_prim_add(c: &mut Criterion) {
    let graph = add_graph(3, 5);
    c.bench_function("eval_add(3,5)", |b| {
        b.iter(|| evaluate(black_box(&graph), black_box(&[])))
    });
}

fn bench_nested_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_add");
    for depth in [10, 50, 100, 500] {
        let graph = nested_add_graph(depth);
        group.bench_with_input(BenchmarkId::from_parameter(depth), &graph, |b, g| {
            b.iter(|| evaluate(black_box(g), black_box(&[])))
        });
    }
    group.finish();
}

fn bench_fold_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("fold_sum");
    for n in [5, 10, 50, 100] {
        let graph = fold_sum_graph(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &graph, |b, g| {
            b.iter(|| evaluate(black_box(g), black_box(&[])))
        });
    }
    group.finish();
}

fn bench_graph_construction(c: &mut Criterion) {
    c.bench_function("graph_construct_100_nodes", |b| {
        b.iter(|| black_box(nested_add_graph(100)))
    });
}

fn bench_syntax_compile(c: &mut Criterion) {
    let src = "let add x y = x + y\nlet double x = add x x\nlet quad x = double (double x)";
    c.bench_function("syntax_compile_3fn", |b| {
        b.iter(|| iris_bootstrap::syntax::compile(black_box(src)))
    });
}

fn bench_syntax_compile_large(c: &mut Criterion) {
    let mut src = String::new();
    for i in 0..50 {
        src.push_str(&format!("let f{} x = x + {}\n", i, i));
    }
    c.bench_function("syntax_compile_50fn", |b| {
        b.iter(|| iris_bootstrap::syntax::compile(black_box(&src)))
    });
}

fn bench_benchmarks_game(c: &mut Criterion) {
    let mut group = c.benchmark_group("benchmarks_game");
    group.sample_size(10);

    let nbody_src = include_str!("../benchmark/n-body/n-body.iris");
    let fannkuch_src = include_str!("../benchmark/fannkuch-redux/fannkuch-redux.iris");
    let pidigits_src = include_str!("../benchmark/pidigits/pidigits.iris");

    let compile_and_find = |src: &str, name: &str| -> (SemanticGraph, BTreeMap<FragmentId, SemanticGraph>) {
        let result = iris_bootstrap::syntax::compile(src);
        let mut reg = BTreeMap::new();
        let mut target = None;
        for (n, frag, _) in &result.fragments {
            reg.insert(frag.id, frag.graph.clone());
            if n == name {
                target = Some(frag.graph.clone());
            }
        }
        (target.unwrap(), reg)
    };

    let (nbody_graph, nbody_reg) = compile_and_find(nbody_src, "run");
    let (fannkuch_graph, fannkuch_reg) = compile_and_find(fannkuch_src, "count_flips");
    let (pidigits_graph, pidigits_reg) = compile_and_find(pidigits_src, "bench");

    group.bench_function("n-body(N=0)", |b| {
        b.iter(|| {
            evaluate_with_fragments(
                black_box(&nbody_graph),
                black_box(&[Value::Int(0)]),
                10_000_000,
                &nbody_reg,
            )
        })
    });

    group.bench_function("fannkuch(3elem)", |b| {
        b.iter(|| {
            let inputs = vec![
                Value::Tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)].into()),
                Value::Int(30),
            ];
            evaluate_with_fragments(
                black_box(&fannkuch_graph),
                black_box(&inputs),
                10_000_000,
                &fannkuch_reg,
            )
        })
    });

    group.bench_function("pidigits(10)", |b| {
        b.iter(|| {
            evaluate_with_fragments(
                black_box(&pidigits_graph),
                black_box(&[Value::Int(10)]),
                10_000_000,
                &pidigits_reg,
            )
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// JIT benchmarks (requires --features jit)
// ---------------------------------------------------------------------------

#[cfg(feature = "jit")]
fn bench_jit(c: &mut Criterion) {
    use iris_exec::effect_runtime::RuntimeEffectHandler;
    use iris_types::eval::{EffectHandler, EffectRequest, EffectTag};

    let handler = RuntimeEffectHandler::new();

    // x86-64: mov rax, imm64; ret
    fn x86_const(value: i64) -> Vec<u8> {
        let mut code = vec![0x48, 0xB8];
        code.extend_from_slice(&value.to_le_bytes());
        code.push(0xC3);
        code
    }

    // x86-64: lea rax, [rdi + rsi]; ret
    fn x86_add() -> Vec<u8> {
        vec![0x48, 0x8D, 0x04, 0x37, 0xC3]
    }

    // x86-64: mov rax, rdi; imul rax, rsi; ret
    fn x86_mul() -> Vec<u8> {
        vec![0x48, 0x89, 0xF8, 0x48, 0x0F, 0xAF, 0xC6, 0xC3]
    }

    // Compile all JIT functions once
    let compile = |code: Vec<u8>| -> i64 {
        match handler.handle(EffectRequest {
            tag: EffectTag::MmapExec,
            args: vec![Value::Bytes(code)],
        }).unwrap() {
            Value::Int(h) => h,
            _ => panic!("expected Int handle"),
        }
    };

    let call = |handle: i64, args: Vec<Value>| -> Value {
        handler.handle(EffectRequest {
            tag: EffectTag::CallNative,
            args: vec![Value::Int(handle), Value::Tuple(args.into())],
        }).unwrap()
    };

    let h_const = compile(x86_const(42));
    let h_add = compile(x86_add());
    let h_mul = compile(x86_mul());

    let mut group = c.benchmark_group("jit");

    // JIT: return constant
    group.bench_function("const_42", |b| {
        b.iter(|| call(black_box(h_const), vec![]))
    });

    // JIT: add two ints
    group.bench_function("add(3,5)", |b| {
        b.iter(|| call(black_box(h_add), vec![Value::Int(3), Value::Int(5)]))
    });

    // JIT: multiply two ints
    group.bench_function("mul(6,7)", |b| {
        b.iter(|| call(black_box(h_mul), vec![Value::Int(6), Value::Int(7)]))
    });

    // Compare: interpreter eval_add vs JIT add
    let add_graph = {
        let a = make_node(1, NodeKind::Lit, NodePayload::Lit {
            type_tag: 0x00,
            value: 3i64.to_le_bytes().to_vec(),
        });
        let b = make_node(2, NodeKind::Lit, NodePayload::Lit {
            type_tag: 0x00,
            value: 5i64.to_le_bytes().to_vec(),
        });
        let add = make_node(3, NodeKind::Prim, NodePayload::Prim { opcode: 0x01 });
        make_graph(
            vec![a, b, add],
            vec![
                Edge { source: NodeId(3), target: NodeId(1), port: 0, label: EdgeLabel::Argument },
                Edge { source: NodeId(3), target: NodeId(2), port: 1, label: EdgeLabel::Argument },
            ],
            3,
        )
    };

    group.bench_function("interp_add(3,5)", |b| {
        b.iter(|| evaluate(black_box(&add_graph), black_box(&[])))
    });

    group.finish();
}

#[cfg(feature = "jit")]
criterion_group!(
    jit_benches,
    bench_lit_eval,
    bench_prim_add,
    bench_nested_add,
    bench_fold_sum,
    bench_graph_construction,
    bench_syntax_compile,
    bench_syntax_compile_large,
    bench_benchmarks_game,
    bench_jit,
);

#[cfg(feature = "jit")]
criterion_main!(jit_benches);

#[cfg(not(feature = "jit"))]
criterion_group!(
    benches,
    bench_lit_eval,
    bench_prim_add,
    bench_nested_add,
    bench_fold_sum,
    bench_graph_construction,
    bench_syntax_compile,
    bench_syntax_compile_large,
    bench_benchmarks_game,
);

#[cfg(not(feature = "jit"))]
criterion_main!(benches);
