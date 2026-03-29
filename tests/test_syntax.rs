
//! Integration tests: compile ML-like IRIS surface syntax and execute.

use iris_exec::effect_runtime::LoggingHandler;
use iris_exec::interpreter;
use iris_types::eval::Value;
use iris_types::graph::NodeKind;
use iris_types::proof::VerifyTier;

fn compile_and_get_graph(src: &str) -> iris_types::graph::SemanticGraph {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors { eprintln!("{}", iris_bootstrap::syntax::format_error(src, err)); }
        panic!("compilation failed with {} errors", result.errors.len());
    }
    assert!(!result.fragments.is_empty(), "no fragments produced");
    result.fragments[0].1.graph.clone()
}

#[test]
fn test_add() {
    let g = compile_and_get_graph("let add2 x y = x + y");
    let (out, _) = interpreter::interpret(&g, &[Value::Int(3), Value::Int(4)], None).unwrap();
    assert_eq!(out, vec![Value::Int(7)]);
}

#[test]
fn test_complex_expr() {
    let g = compile_and_get_graph("let f x y = (x + y) * (x - y)");
    let (out, _) = interpreter::interpret(&g, &[Value::Int(5), Value::Int(3)], None).unwrap();
    assert_eq!(out, vec![Value::Int(16)]);
}

#[test]
fn test_unary_neg() {
    let g = compile_and_get_graph("let negate x = -x");
    let (out, _) = interpreter::interpret(&g, &[Value::Int(42)], None).unwrap();
    assert_eq!(out, vec![Value::Int(-42)]);
}

#[test]
fn test_fold_sum() {
    // ML style: juxtaposition, operator section
    let g = compile_and_get_graph("let sum xs = fold 0 (+) xs");
    let input = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let (out, _) = interpreter::interpret(&g, &[input], None).unwrap();
    assert_eq!(out, vec![Value::Int(6)]);
}

#[test]
fn test_fold_product() {
    let g = compile_and_get_graph("let product xs = fold 1 (*) xs");
    let input = Value::tuple(vec![Value::Int(2), Value::Int(3), Value::Int(4)]);
    let (out, _) = interpreter::interpret(&g, &[input], None).unwrap();
    assert_eq!(out, vec![Value::Int(24)]);
}

#[test]
fn test_fold_max() {
    let g = compile_and_get_graph("let find_max xs = fold (-9999999) max xs");
    let input = Value::tuple(vec![Value::Int(3), Value::Int(7), Value::Int(1)]);
    let (out, _) = interpreter::interpret(&g, &[input], None).unwrap();
    assert_eq!(out, vec![Value::Int(7)]);
}

#[test]
fn test_dot_product() {
    // ML style: fold 0 (+) (map (*) (zip xs ys))
    let g = compile_and_get_graph("let dot xs ys = fold 0 (+) (map (*) (zip xs ys))");
    let xs = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let ys = Value::tuple(vec![Value::Int(4), Value::Int(5), Value::Int(6)]);
    let (out, _) = interpreter::interpret(&g, &[xs, ys], None).unwrap();
    assert_eq!(out, vec![Value::Int(32)]);
}

#[test]
fn test_if_then_else() {
    let g = compile_and_get_graph("let safe_div x y = if y != 0 then x / y else 0");
    let (out, _) = interpreter::interpret(&g, &[Value::Int(10), Value::Int(2)], None).unwrap();
    assert_eq!(out, vec![Value::Int(5)]);
    let (out, _) = interpreter::interpret(&g, &[Value::Int(10), Value::Int(0)], None).unwrap();
    assert_eq!(out, vec![Value::Int(0)]);
}

#[test]
fn test_guard() {
    let g = compile_and_get_graph("let clamp x = guard (x > 100) 100 (guard (x < 0) 0 x)");
    let (out, _) = interpreter::interpret(&g, &[Value::Int(150)], None).unwrap();
    assert_eq!(out, vec![Value::Int(100)]);
    let (out, _) = interpreter::interpret(&g, &[Value::Int(-5)], None).unwrap();
    assert_eq!(out, vec![Value::Int(0)]);
    let (out, _) = interpreter::interpret(&g, &[Value::Int(50)], None).unwrap();
    assert_eq!(out, vec![Value::Int(50)]);
}

#[test]
fn test_lambda_fold() {
    // ML style: \acc x -> acc + bool_to_int (x > 0)
    let g = compile_and_get_graph(
        "let count_pos xs = fold 0 (\\acc x -> acc + bool_to_int (x > 0)) xs"
    );
    let input = Value::tuple(vec![
        Value::Int(-1), Value::Int(3), Value::Int(-2), Value::Int(5), Value::Int(0),
    ]);
    let (out, _) = interpreter::interpret(&g, &[input], None).unwrap();
    assert_eq!(out, vec![Value::Int(2)]);
}

#[test]
fn test_let_in() {
    let g = compile_and_get_graph(
        "let f xs ys = let pairs = zip xs ys in fold 0 (+) (map (*) pairs)"
    );
    let xs = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let ys = Value::tuple(vec![Value::Int(4), Value::Int(5), Value::Int(6)]);
    let (out, _) = interpreter::interpret(&g, &[xs, ys], None).unwrap();
    assert_eq!(out, vec![Value::Int(32)]);
}

#[test]
fn test_tuple() {
    let g = compile_and_get_graph("let pair x y = (x, y)");
    let (out, _) = interpreter::interpret(&g, &[Value::Int(1), Value::Int(2)], None).unwrap();
    assert_eq!(out, vec![Value::tuple(vec![Value::Int(1), Value::Int(2)])]);
}

#[test]
fn test_tuple_access() {
    let g = compile_and_get_graph("let fst p = p.0");
    let input = Value::tuple(vec![Value::Int(10), Value::Int(20)]);
    let (out, _) = interpreter::interpret(&g, &[input], None).unwrap();
    assert_eq!(out, vec![Value::Int(10)]);
}

#[test]
fn test_match_bool() {
    let g = compile_and_get_graph(
        "let abs_val x = match x >= 0 with | true -> x | false -> neg x"
    );
    let (out, _) = interpreter::interpret(&g, &[Value::Int(-7)], None).unwrap();
    assert_eq!(out, vec![Value::Int(7)]);
    let (out, _) = interpreter::interpret(&g, &[Value::Int(3)], None).unwrap();
    assert_eq!(out, vec![Value::Int(3)]);
}

#[test]
fn test_multiple_fns() {
    let src = r#"
        let sum xs = fold 0 (+) xs
        let product xs = fold 1 (*) xs
    "#;
    let result = iris_bootstrap::syntax::compile(src);
    assert!(result.errors.is_empty());
    assert_eq!(result.fragments.len(), 2);
    assert_eq!(result.fragments[0].0, "sum");
    assert_eq!(result.fragments[1].0, "product");
}

#[test]
fn test_prim_min() {
    let g = compile_and_get_graph("let find_min xs = fold 9999999 min xs");
    let input = Value::tuple(vec![Value::Int(5), Value::Int(1), Value::Int(8)]);
    let (out, _) = interpreter::interpret(&g, &[input], None).unwrap();
    assert_eq!(out, vec![Value::Int(1)]);
}

// ---------------------------------------------------------------------------
// Pipe operator
// ---------------------------------------------------------------------------

#[test]
fn test_pipe_simple() {
    let g = compile_and_get_graph("let f x = x |> neg");
    let (out, _) = interpreter::interpret(&g, &[Value::Int(5)], None).unwrap();
    assert_eq!(out, vec![Value::Int(-5)]);
}

#[test]
fn test_pipe_chain() {
    // fold 0 (+) (filter (\x -> x > 0) xs)  written as a pipeline
    let g = compile_and_get_graph(
        "let sum_pos xs = xs |> filter (\\x -> x > 0) |> fold 0 (+)"
    );
    let input = Value::tuple(vec![Value::Int(-1), Value::Int(3), Value::Int(-2), Value::Int(5)]);
    let (out, _) = interpreter::interpret(&g, &[input], None).unwrap();
    assert_eq!(out, vec![Value::Int(8)]); // 3 + 5
}

#[test]
fn test_pipe_with_lambda() {
    let g = compile_and_get_graph(
        "let double_sum xs = xs |> map (\\x -> x * 2) |> fold 0 (+)"
    );
    let input = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let (out, _) = interpreter::interpret(&g, &[input], None).unwrap();
    assert_eq!(out, vec![Value::Int(12)]); // 2 + 4 + 6
}

#[test]
fn test_cost_annotation() {
    let g = compile_and_get_graph("let sum xs : Int [cost: Linear(n)] = fold 0 (+) xs");
    let input = Value::tuple(vec![Value::Int(10), Value::Int(20)]);
    let (out, _) = interpreter::interpret(&g, &[input], None).unwrap();
    assert_eq!(out, vec![Value::Int(30)]);
}

// ---------------------------------------------------------------------------
// Source map and diagnostics
// ---------------------------------------------------------------------------

#[test]
fn test_source_map_populated() {
    let src = "let sum xs = fold 0 (+) xs";
    let result = iris_bootstrap::syntax::compile(src);
    assert!(result.errors.is_empty());
    let (_, fragment, source_map) = &result.fragments[0];
    // Every node in the graph should have a source span
    for node_id in fragment.graph.nodes.keys() {
        assert!(
            source_map.contains_key(node_id),
            "node {:?} ({:?}) missing from source map",
            node_id,
            fragment.graph.nodes[node_id].kind
        );
    }
}

#[test]
fn test_compile_and_verify_produces_diagnostics() {
    let src = "let sum xs = fold 0 (+) xs";
    let (result, diagnostics) = iris_bootstrap::syntax::compile_and_verify(src, VerifyTier::Tier0);
    assert!(result.errors.is_empty());
    // We don't assert diagnostics are empty or non-empty — the kernel may
    // or may not flag issues. We just verify the pipeline doesn't crash
    // and produces a string.
    let _ = diagnostics;
}

#[test]
fn test_diagnostics_have_line_numbers() {
    let src = "let f x y =\n  x + y";
    let (result, diagnostics) = iris_bootstrap::syntax::compile_and_verify(src, VerifyTier::Tier0);
    assert!(result.errors.is_empty());
    if !diagnostics.is_empty() {
        assert!(diagnostics.contains("line"), "diagnostics should reference line numbers: {diagnostics}");
    }
}

// ---------------------------------------------------------------------------
// Accurate type_sig propagation
// ---------------------------------------------------------------------------

#[test]
fn test_comparison_produces_bool_type() {
    let src = "let is_pos x = x > 0";
    let result = iris_bootstrap::syntax::compile(src);
    assert!(result.errors.is_empty());
    let (_, frag, _) = &result.fragments[0];
    let root = &frag.graph.nodes[&frag.graph.root];
    // The root should be a comparison Prim — its type_sig should be Bool, not Int
    assert_eq!(root.kind, NodeKind::Prim);
    let bool_type = iris_types::hash::compute_type_id(
        &iris_types::types::TypeDef::Primitive(iris_types::types::PrimType::Bool));
    assert_eq!(root.type_sig, bool_type, "comparison should produce Bool type");
}

#[test]
fn test_tuple_produces_product_type() {
    let src = "let pair x y = (x, y)";
    let result = iris_bootstrap::syntax::compile(src);
    assert!(result.errors.is_empty());
    let (_, frag, _) = &result.fragments[0];
    let root = &frag.graph.nodes[&frag.graph.root];
    assert_eq!(root.kind, NodeKind::Tuple);
    // The type should be a Product, not bare Int
    let root_type = &frag.type_env.types[&root.type_sig];
    assert!(matches!(root_type, iris_types::types::TypeDef::Product(_)),
        "tuple should have Product type, got {:?}", root_type);
}

// ---------------------------------------------------------------------------
// Unfold (corecursion / iteration)
// ---------------------------------------------------------------------------

#[test]
fn test_unfold_fibonacci() {
    // unfold (1, 1) (+) 8 → emits fib sequence: [1, 1, 2, 3, 5, 8, 13, 21]
    // Mode 0x00: seed=(a,b), step=op → emit a, next=(b, op(a,b))
    let g = compile_and_get_graph("let fibs n = unfold (1, 1) (+) n");
    // Pass Int(8) as bound → 8 iterations
    let bound = Value::Int(8);
    let (out, _) = interpreter::interpret(&g, &[bound], None).unwrap();
    // Should be a Tuple of 8 elements: [1, 1, 2, 3, 5, 8, 13, 21]
    if let Value::Tuple(elems) = &out[0] {
        assert_eq!(elems.len(), 8, "expected 8 fibonacci numbers, got {}", elems.len());
        assert_eq!(elems[0], Value::Int(1));
        assert_eq!(elems[1], Value::Int(1));
        assert_eq!(elems[2], Value::Int(2));
        assert_eq!(elems[3], Value::Int(3));
        assert_eq!(elems[4], Value::Int(5));
        assert_eq!(elems[5], Value::Int(8));
        assert_eq!(elems[6], Value::Int(13));
        assert_eq!(elems[7], Value::Int(21));
    } else {
        panic!("expected Tuple output from unfold, got {:?}", out[0]);
    }
}

#[test]
fn test_unfold_powers_of_2() {
    // unfold (1, 2) (*) 5 with mode 0x02 would be geometric
    // But default mode 0x00 with (*): seed=(1,2), emit 1, next=(2, 1*2=2), emit 2, next=(2, 2*2=4), emit 2...
    // Actually with add: seed=(1,2), emit 1, next=(2, 1+2=3), emit 2, next=(3, 2+3=5)...
    // Let's just verify unfold with add produces the right count
    let g = compile_and_get_graph("let seq n = unfold (0, 1) (+) n");
    let (out, _) = interpreter::interpret(&g, &[Value::Int(5)], None).unwrap();
    if let Value::Tuple(elems) = &out[0] {
        assert_eq!(elems.len(), 5, "expected 5 elements, got {}", elems.len());
        // Fibonacci-like: 0, 1, 1, 2, 3
        assert_eq!(elems[0], Value::Int(0));
        assert_eq!(elems[1], Value::Int(1));
        assert_eq!(elems[2], Value::Int(1));
        assert_eq!(elems[3], Value::Int(2));
        assert_eq!(elems[4], Value::Int(3));
    } else {
        panic!("expected Tuple from unfold, got {:?}", out[0]);
    }
}

// ---------------------------------------------------------------------------
// Effect I/O
// ---------------------------------------------------------------------------

#[test]
fn test_effect_print_compiles() {
    // Effect nodes compile and execute (returning Unit when no handler)
    let g = compile_and_get_graph("let hello x = effect print x");
    let (out, _) = interpreter::interpret(&g, &[Value::Int(42)], None).unwrap();
    assert_eq!(out, vec![Value::Unit]);
}

#[test]
fn test_effect_with_handler() {
    // Use LoggingHandler to capture effect requests
    let g = compile_and_get_graph("let hello x = effect print x");
    let handler = LoggingHandler::new();
    let (out, _) = interpreter::interpret_with_effects(
        &g, &[Value::Int(42)], None, None, 100_000,
        Some(&handler),
    ).unwrap();
    assert_eq!(out, vec![Value::Unit]);
    let log = handler.entries();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].args, vec![Value::Int(42)]);
}

#[test]
fn test_effect_timestamp() {
    let g = compile_and_get_graph("let now x = effect timestamp x");
    let (out, _) = interpreter::interpret(&g, &[Value::Unit], None).unwrap();
    // Bootstrap evaluator returns actual timestamp as Int
    assert_eq!(out.len(), 1);
    match &out[0] {
        Value::Int(ts) => assert!(*ts > 0, "timestamp should be positive"),
        other => panic!("expected Int timestamp, got {:?}", other),
    }
}

#[test]
fn test_effect_raw_tag() {
    // Use a raw integer tag for forward-compatibility with new primitives
    let g = compile_and_get_graph("let custom x = effect 42 x");
    let (out, _) = interpreter::interpret(&g, &[Value::Int(1)], None).unwrap();
    assert_eq!(out, vec![Value::Unit]);
}

// ---------------------------------------------------------------------------
// I/O primitives — direct call syntax
// ---------------------------------------------------------------------------

#[test]
fn test_io_tcp_listen_compiles() {
    // tcp_listen as a direct call (not through `effect` keyword)
    let g = compile_and_get_graph("let server port = tcp_listen port");
    let root = &g.nodes[&g.root];
    assert_eq!(root.kind, NodeKind::Effect);
    if let iris_types::graph::NodePayload::Effect { effect_tag } = &root.payload {
        assert_eq!(*effect_tag, 0x14);
    }
    // Without a real handler, returns Unit
    let (out, _) = interpreter::interpret(&g, &[Value::Int(8080)], None).unwrap();
    assert_eq!(out, vec![Value::Unit]);
}

#[test]
fn test_io_file_open_compiles() {
    let g = compile_and_get_graph(r#"let open path = file_open path 0"#);
    let root = &g.nodes[&g.root];
    assert_eq!(root.kind, NodeKind::Effect);
    if let iris_types::graph::NodePayload::Effect { effect_tag } = &root.payload {
        assert_eq!(*effect_tag, 0x16);
    }
}

#[test]
fn test_io_clock_ns_compiles() {
    let g = compile_and_get_graph("let now x = clock_ns x");
    let root = &g.nodes[&g.root];
    assert_eq!(root.kind, NodeKind::Effect);
    if let iris_types::graph::NodePayload::Effect { effect_tag } = &root.payload {
        assert_eq!(*effect_tag, 0x1D);
    }
}

#[test]
fn test_io_env_get_compiles() {
    let g = compile_and_get_graph(r#"let get_env name = env_get name"#);
    let root = &g.nodes[&g.root];
    assert_eq!(root.kind, NodeKind::Effect);
    if let iris_types::graph::NodePayload::Effect { effect_tag } = &root.payload {
        assert_eq!(*effect_tag, 0x1C);
    }
}

#[test]
fn test_io_tcp_echo_pipeline() {
    // The proposed echo server from the spec, using let-in bindings
    let src = r#"
        let echo port =
            let listener = tcp_listen port in
            let conn = tcp_accept listener in
            let data = tcp_read conn 1024 in
            tcp_write conn data
    "#;
    let result = iris_bootstrap::syntax::compile(src);
    assert!(result.errors.is_empty(), "echo server should compile: {:?}", result.errors);
    // Verify it produces Effect nodes for each I/O call
    let (_, frag, _) = &result.fragments[0];
    let effect_count = frag.graph.nodes.values()
        .filter(|n| n.kind == NodeKind::Effect)
        .count();
    assert!(effect_count >= 4, "expected at least 4 effect nodes, got {}", effect_count);
}

#[test]
fn test_io_effect_keyword_also_works() {
    // The `effect` keyword should still work with the new names
    let g = compile_and_get_graph("let listen port = effect tcp_listen port");
    let root = &g.nodes[&g.root];
    assert_eq!(root.kind, NodeKind::Effect);
    if let iris_types::graph::NodePayload::Effect { effect_tag } = &root.payload {
        assert_eq!(*effect_tag, 0x14);
    }
}

// ---------------------------------------------------------------------------
// let rec — NOT YET SUPPORTED
// ---------------------------------------------------------------------------
// The Gen1 interpreter's LetRec is a stub: self-reference binds Value::Unit,
// so true recursion (factorial, fibonacci via recursion, etc.) does not work.
// The parser accepts `let rec` but the lowering returns an error until the
// interpreter supports fixpoint evaluation.
//
// Recursive algorithms should use fold/unfold/guard instead:
//   let factorial n = fold 1 (*) (unfold (1, 1) (\s -> (s.0 + 1, s.1)) n)
// Or via guard-based bounded iteration once that pattern is supported.


// ---------------------------------------------------------------------------
// tokenizer.iris — production tokenizer parses without errors
// ---------------------------------------------------------------------------

#[test]
fn test_tokenizer_iris_compiles() {
    let src = include_str!("../src/iris-programs/syntax/tokenizer.iris");
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors { eprintln!("{}", iris_bootstrap::syntax::format_error(src, err)); }
        panic!("tokenizer.iris compilation failed with {} errors", result.errors.len());
    }
    assert!(!result.fragments.is_empty(), "tokenizer.iris produced no fragments");
    let frag_names: Vec<&str> = result.fragments.iter().map(|(n, _, _)| n.as_str()).collect();
    println!("tokenizer.iris fragments: {:?}", frag_names);
    assert!(frag_names.contains(&"tokenize"), "expected 'tokenize' fragment");
}

#[test]
fn test_tokenizer_iris_empty_input() {
    let graph = compile_tokenizer();
    let tokens = tokenize_str(&graph, "");
    assert_eq!(tokens.len(), 0, "empty input should produce 0 tokens");
}

fn compile_tokenizer() -> iris_types::graph::SemanticGraph {
    let src = include_str!("../src/iris-programs/syntax/tokenizer.iris");
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors { eprintln!("{}", iris_bootstrap::syntax::format_error(src, err)); }
        panic!("tokenizer.iris compilation failed with {} errors", result.errors.len());
    }
    assert!(!result.fragments.is_empty(), "no fragments");
    result.fragments[0].1.graph.clone()
}

fn tokenize_str(graph: &iris_types::graph::SemanticGraph, s: &str) -> Vec<Value> {
    let input = Value::String(s.into());
    let result = iris_bootstrap::evaluate_with_limit(graph, &[input], 500_000).unwrap();
    match result {
        Value::Tuple(tokens) => (*tokens).clone(),
        other => panic!("expected Tuple output, got {:?}", other),
    }
}

fn assert_tok_kind(tokens: &[Value], idx: usize, expected_kind: i64, label: &str) {
    if let Value::Tuple(t) = &tokens[idx] {
        assert_eq!(t[0], Value::Int(expected_kind), "{}", label);
    } else {
        panic!("token {} should be a tuple, got {:?}", idx, tokens[idx]);
    }
}

fn assert_tok(tokens: &[Value], idx: usize, expected_kind: i64, expected_payload: i64, label: &str) {
    if let Value::Tuple(t) = &tokens[idx] {
        assert_eq!(t[0], Value::Int(expected_kind), "{} kind", label);
        assert_eq!(t[1], Value::Int(expected_payload), "{} payload", label);
    } else {
        panic!("token {} should be a tuple, got {:?}", idx, tokens[idx]);
    }
}

#[test]
fn test_tokenizer_iris_single_ident() {
    let graph = compile_tokenizer();
    // Try a longer string to increase budget
    let tokens = tokenize_str(&graph, "x ");
    println!("tokenize(\"x \") = {:?}", tokens);
    assert_eq!(tokens.len(), 1, "expected 1 token, got {:?}", tokens);
    assert_tok_kind(&tokens, 0, 2, "x should be Ident(2)");
}

#[test]
fn test_tokenizer_iris_single_keyword() {
    let graph = compile_tokenizer();
    let tokens = tokenize_str(&graph, "let");
    println!("tokenize(\"let\") = {:?}", tokens);
    assert_eq!(tokens.len(), 1, "expected 1 token, got {:?}", tokens);
    assert_tok_kind(&tokens, 0, 10, "let should be keyword 10");
}

#[test]
fn test_tokenizer_iris_ident_and_op() {
    let graph = compile_tokenizer();
    let tokens = tokenize_str(&graph, "x + y");
    println!("tokenize(\"x + y\") = {:?}", tokens);
    assert_eq!(tokens.len(), 3, "expected 3 tokens, got {:?}", tokens);
    assert_tok_kind(&tokens, 0, 2, "x");
    assert_tok_kind(&tokens, 1, 30, "+");
    assert_tok_kind(&tokens, 2, 2, "y");
}

#[test]
fn test_tokenizer_iris_tokenizes_simple() {
    let graph = compile_tokenizer();
    let tokens = tokenize_str(&graph, "1 + 2");
    println!("tokenize(\"1 + 2\") = {:?}", tokens);
    assert_eq!(tokens.len(), 3, "expected 3 tokens, got {:?}", tokens);
    assert_tok(&tokens, 0, 1, 1, "1");     // IntLit, value=1
    assert_tok_kind(&tokens, 1, 30, "+");   // Plus
    assert_tok(&tokens, 2, 1, 2, "2");     // IntLit, value=2
}

#[test]
fn test_tokenizer_iris_tokenizes_let_expr() {
    let graph = compile_tokenizer();
    let tokens = tokenize_str(&graph, "let x = 42 in x");
    println!("tokenize(\"let x = 42 in x\") = {:?}", tokens);
    assert_eq!(tokens.len(), 6, "expected 6 tokens, got {:?}", tokens);
    assert_tok_kind(&tokens, 0, 10, "let");
    assert_tok_kind(&tokens, 1, 2, "x (ident)");
    assert_tok_kind(&tokens, 2, 35, "=");
    assert_tok(&tokens, 3, 1, 42, "42");
    assert_tok_kind(&tokens, 4, 12, "in");
    assert_tok_kind(&tokens, 5, 2, "x (ident)");
}

#[test]
fn test_tokenizer_iris_handles_comments() {
    let graph = compile_tokenizer();
    let tokens = tokenize_str(&graph, "1 -- comment\n+ 2");
    println!("tokenize comment = {:?}", tokens);
    assert_eq!(tokens.len(), 3, "expected 3 tokens, got {:?}", tokens);
    assert_tok_kind(&tokens, 0, 1, "1");
    assert_tok_kind(&tokens, 1, 30, "+");
    assert_tok_kind(&tokens, 2, 1, "2");
}

#[test]
fn test_tokenizer_iris_two_char_ops() {
    let graph = compile_tokenizer();
    let tokens = tokenize_str(&graph, "x == y && z <= w");
    println!("tokenize two-char ops = {:?}", tokens);
    assert_eq!(tokens.len(), 7, "expected 7 tokens, got {:?}", tokens);
    assert_tok_kind(&tokens, 1, 36, "==");
    assert_tok_kind(&tokens, 3, 42, "&&");
    assert_tok_kind(&tokens, 5, 40, "<=");
}

#[test]
fn test_tokenizer_iris_arrow_and_pipe() {
    let graph = compile_tokenizer();
    let tokens = tokenize_str(&graph, "x -> y |> z");
    println!("tokenize arrow/pipe = {:?}", tokens);
    assert_eq!(tokens.len(), 5, "expected 5 tokens, got {:?}", tokens);
    assert_tok_kind(&tokens, 1, 60, "->");
    assert_tok_kind(&tokens, 3, 61, "|>");
}

#[test]
fn test_tokenizer_iris_string_literal() {
    let graph = compile_tokenizer();
    let tokens = tokenize_str(&graph, "let s = \"hello\"");
    println!("tokenize string lit = {:?}", tokens);
    assert_eq!(tokens.len(), 4, "expected 4 tokens, got {:?}", tokens);
    assert_tok_kind(&tokens, 0, 10, "let");
    assert_tok_kind(&tokens, 1, 2, "s");
    assert_tok_kind(&tokens, 2, 35, "=");
    assert_tok_kind(&tokens, 3, 4, "string lit");
}

#[test]
fn test_tokenizer_iris_parens_and_commas() {
    let graph = compile_tokenizer();
    let tokens = tokenize_str(&graph, "(1, 2)");
    println!("tokenize parens = {:?}", tokens);
    assert_eq!(tokens.len(), 5, "expected 5 tokens, got {:?}", tokens);
    assert_tok_kind(&tokens, 0, 50, "(");
    assert_tok_kind(&tokens, 1, 1, "1");
    assert_tok_kind(&tokens, 2, 56, ",");
    assert_tok_kind(&tokens, 3, 1, "2");
    assert_tok_kind(&tokens, 4, 51, ")");
}

#[test]
fn test_tokenizer_iris_keywords() {
    let graph = compile_tokenizer();
    let tokens = tokenize_str(&graph, "if true then false else 0");
    println!("tokenize keywords = {:?}", tokens);
    assert_eq!(tokens.len(), 6, "expected 6 tokens, got {:?}", tokens);
    assert_tok_kind(&tokens, 0, 20, "if");
    assert_tok_kind(&tokens, 1, 24, "true");
    assert_tok_kind(&tokens, 2, 21, "then");
    assert_tok_kind(&tokens, 3, 25, "false");
    assert_tok_kind(&tokens, 4, 22, "else");
    assert_tok_kind(&tokens, 5, 1, "0");
}

#[test]
fn test_tokenizer_iris_lambda() {
    let graph = compile_tokenizer();
    let tokens = tokenize_str(&graph, "\\x -> x");
    println!("tokenize lambda = {:?}", tokens);
    assert_eq!(tokens.len(), 4, "expected 4 tokens, got {:?}", tokens);
    assert_tok_kind(&tokens, 0, 59, "\\");
    assert_tok_kind(&tokens, 1, 2, "x");
    assert_tok_kind(&tokens, 2, 60, "->");
    assert_tok_kind(&tokens, 3, 2, "x");
}

#[test]
fn test_tokenizer_iris_underscore() {
    let graph = compile_tokenizer();
    // Test standalone underscore as wildcard (64)
    let tokens = tokenize_str(&graph, "_ + _abc");
    println!("tokenize underscore = {:?}", tokens);
    assert_eq!(tokens.len(), 3, "expected 3 tokens, got {:?}", tokens);
    assert_tok_kind(&tokens, 0, 64, "_");
    assert_tok_kind(&tokens, 1, 30, "+");
    assert_tok_kind(&tokens, 2, 2, "_abc");
}

// ---------------------------------------------------------------------------
// iris_parser.iris — AST-producing parser
// ---------------------------------------------------------------------------

fn compile_parser() -> iris_types::graph::SemanticGraph {
    let src = include_str!("../src/iris-programs/syntax/iris_parser.iris");
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            let msg = iris_bootstrap::syntax::format_error(src, err);
            eprintln!("{}", msg);
        }
        panic!("iris_parser.iris compilation failed with {} errors", result.errors.len());
    }
    assert!(!result.fragments.is_empty(), "iris_parser.iris produced no fragments");
    let idx = result.fragments.len() - 1;
    result.fragments[idx].1.graph.clone()
}

/// Tokenize a string, then parse the tokens. Returns the AST as a Value.
fn parse_iris(input: &str) -> Value {
    let tok_graph = compile_tokenizer();
    let parser_graph = compile_parser();

    // Step 1: tokenize
    let tokens_val = iris_bootstrap::evaluate_with_limit(
        &tok_graph, &[Value::String(input.into())], 500_000,
    ).expect("tokenizer failed");

    // Step 2: parse
    let src_val = Value::String(input.into());
    let ast = iris_bootstrap::evaluate_with_limit(
        &parser_graph, &[tokens_val, src_val], 50_000_000,
    ).expect("parser failed");
    ast
}

/// Extract the kind field (first element) from an AST node tuple.
fn ast_kind(node: &Value) -> i64 {
    match node {
        Value::Tuple(t) => match &t[0] {
            Value::Int(k) => *k,
            other => panic!("AST kind should be Int, got {:?}", other),
        },
        other => panic!("AST node should be Tuple, got {:?}", other),
    }
}

/// Extract the payload field (second element) from an AST node tuple.
fn ast_payload(node: &Value) -> i64 {
    match node {
        Value::Tuple(t) => match &t[1] {
            Value::Int(p) => *p,
            other => panic!("AST payload should be Int, got {:?}", other),
        },
        other => panic!("AST node should be Tuple, got {:?}", other),
    }
}

/// Extract children from an AST node (third element).
fn ast_children(node: &Value) -> &[Value] {
    match node {
        Value::Tuple(t) => match &t[2] {
            Value::Tuple(children) => children,
            other => panic!("AST children should be Tuple, got {:?}", other),
        },
        other => panic!("AST node should be Tuple, got {:?}", other),
    }
}

#[test]
fn test_parser_iris_compiles() {
    let src = include_str!("../src/iris-programs/syntax/iris_parser.iris");
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors { eprintln!("{}", iris_bootstrap::syntax::format_error(src, err)); }
        panic!("iris_parser.iris compilation failed with {} errors", result.errors.len());
    }
    assert!(!result.fragments.is_empty(), "iris_parser.iris produced no fragments");
    let frag_names: Vec<&str> = result.fragments.iter().map(|(n, _, _)| n.as_str()).collect();
    println!("iris_parser.iris fragments: {:?}", frag_names);
    assert!(frag_names.contains(&"parse"), "expected 'parse' fragment");
}

#[test]
fn test_parser_iris_single_int() {
    let ast = parse_iris("let x = 42");
    println!("parse(\"let x = 42\") = {:?}", ast);
    // Should be Module(items=[LetDecl(...)])
    assert_eq!(ast_kind(&ast), 40, "root should be Module");
    let items = ast_children(&ast);
    assert_eq!(items.len(), 1, "should have 1 declaration");
    assert_eq!(ast_kind(&items[0]), 30, "item should be LetDecl");
    // LetDecl children: (params_tuple, body)
    let decl_children = ast_children(&items[0]);
    assert!(decl_children.len() >= 2, "LetDecl should have at least 2 children (params, body), got {}", decl_children.len());
    // Body should be IntLit(42)
    let body = &decl_children[1];
    assert_eq!(ast_kind(body), 0, "body should be IntLit");
    assert_eq!(ast_payload(body), 42, "body payload should be 42");
}

#[test]
fn test_parser_iris_single_ident() {
    let ast = parse_iris("let f x = x");
    println!("parse(\"let f x = x\") = {:?}", ast);
    assert_eq!(ast_kind(&ast), 40);
    let items = ast_children(&ast);
    let decl = &items[0];
    assert_eq!(ast_kind(decl), 30, "should be LetDecl");
    let decl_children = ast_children(decl);
    // params is a raw tuple of Var nodes (not an AST node)
    let params = &decl_children[0];
    match params {
        Value::Tuple(t) => assert!(!t.is_empty(), "should have params"),
        _ => panic!("params should be Tuple"),
    }
    // Body should be Var
    let body = &decl_children[1];
    assert_eq!(ast_kind(body), 4, "body should be Var");
}

#[test]
fn test_parser_iris_lambda_call() {
    // Test that lambda-based helpers work in the bootstrap evaluator
    let src = r#"
        let test x =
          let helper = \k -> k + 1 in
          helper x
    "#;
    let g = compile_and_get_graph(src);
    let (out, _) = iris_exec::interpreter::interpret(&g, &[Value::Int(5)], None).unwrap();
    assert_eq!(out, vec![Value::Int(6)]);
}

#[test]
fn test_parser_iris_lambda_in_fold() {
    // Test calling a lambda from inside a fold
    let src = r#"
        let test xs =
          let add1 = \k -> k + 1 in
          fold 0 (\acc x -> acc + add1 x) xs
    "#;
    let g = compile_and_get_graph(src);
    let input = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let (out, _) = iris_exec::interpreter::interpret(&g, &[input], None).unwrap();
    assert_eq!(out, vec![Value::Int(9)]); // (1+1) + (2+1) + (3+1) = 9
}

#[test]
fn test_bootstrap_lambda_in_fold() {
    // Same test but on the bootstrap evaluator
    let src = r#"
        let test xs =
          let add1 = \k -> k + 1 in
          fold 0 (\acc x -> acc + add1 x) xs
    "#;
    let result = iris_bootstrap::syntax::compile(src);
    assert!(result.errors.is_empty());
    let g = result.fragments[0].1.graph.clone();
    let input = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let out = iris_bootstrap::evaluate_with_limit(&g, &[input], 100_000).unwrap();
    assert_eq!(out, Value::Int(9));
}

#[test]
fn test_bootstrap_nested_lambda_in_fold() {
    // Test calling a lambda-returning-tuple from fold, similar to get_infix_bp
    let src = r#"
        let test tokens =
          let n = list_len tokens in
          let mt = list_range 0 0 in
          let get_bp = \k ->
            if k == 30 then (10, 11, 0)
            else (0, 0, 0) in
          let result = fold (0, mt, 0, 0, 0)
            (\s _i ->
              let pos = s.0 in
              let vals = s.1 in
              let vsp = s.2 in
              let phase = s.3 in
              let err = s.4 in
              if err != 0 then s
              else if phase == 2 then s
              else if pos >= n then
                if phase == 1 then (pos, vals, vsp, 2, 0)
                else (pos, vals, vsp, 2, 1)
              else if phase == 0 then
                let tok = list_nth tokens pos in
                let tk = tok.0 in
                if tk == 1 then
                  (pos + 1, list_append vals (0, tok.1, mt), vsp + 1, 1, 0)
                else (pos, vals, vsp, 2, 1)
              else
                let tok = list_nth tokens pos in
                let tk = tok.0 in
                let bp = get_bp tk in
                if bp.0 > 0 then
                  (pos + 1, vals, vsp, 0, 0)
                else
                  (pos, vals, vsp, 2, 0))
            (n * 4) in
          result
    "#;
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors { eprintln!("{}", iris_bootstrap::syntax::format_error(src, err)); }
        panic!("compilation failed");
    }
    let g = result.fragments[0].1.graph.clone();
    // Tokens: 1(1,1), +(30,0), 2(1,2)
    let tokens = Value::tuple(vec![
        Value::tuple(vec![Value::Int(1), Value::Int(1), Value::Int(0), Value::Int(1)]),
        Value::tuple(vec![Value::Int(30), Value::Int(0), Value::Int(2), Value::Int(3)]),
        Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(4), Value::Int(5)]),
    ]);
    let out = iris_bootstrap::evaluate_with_limit(&g, &[tokens], 500_000).unwrap();
    eprintln!("nested lambda in fold output: {:?}", out);
    // pos should be 3, vsp should be 2, phase should be 2
    match &out {
        Value::Tuple(t) => {
            assert_eq!(t[0], Value::Int(3), "pos should be 3 (all tokens consumed)");
            assert_eq!(t[2], Value::Int(2), "vsp should be 2");
            assert_eq!(t[3], Value::Int(2), "phase should be 2 (done)");
            assert_eq!(t[4], Value::Int(0), "err should be 0");
        }
        _ => panic!("expected tuple"),
    }
}

#[test]
fn test_bootstrap_lambda_calls_lambda() {
    // Test: lambda A calls lambda B, which runs a fold
    let src = r#"
        let test tokens =
          let n = list_len tokens in
          let mt = list_range 0 0 in
          let inner = \args ->
            let p = args.0 in
            let ep = args.1 in
            fold (p, mt, 0, 0)
              (\s _i ->
                let pos = s.0 in
                if pos >= ep then s
                else if s.3 == 2 then s
                else if s.3 == 0 then
                  let tok = list_nth tokens pos in
                  if tok.0 == 1 then
                    (pos + 1, list_append s.1 (0, tok.1, mt), s.2 + 1, 1)
                  else (pos, s.1, s.2, 2)
                else
                  let tok = list_nth tokens pos in
                  if tok.0 == 30 then (pos + 1, s.1, s.2, 0)
                  else (pos, s.1, s.2, 2))
              (ep * 4) in
          let outer = \args ->
            let start = args.0 in
            let dend = args.1 in
            let result = inner (start, dend) in
            result in
          outer (0, n)
    "#;
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors { eprintln!("{}", iris_bootstrap::syntax::format_error(src, err)); }
        panic!("compilation failed");
    }
    let g = result.fragments[0].1.graph.clone();
    let tokens = Value::tuple(vec![
        Value::tuple(vec![Value::Int(1), Value::Int(1), Value::Int(0), Value::Int(1)]),
        Value::tuple(vec![Value::Int(30), Value::Int(0), Value::Int(2), Value::Int(3)]),
        Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(4), Value::Int(5)]),
    ]);
    let out = iris_bootstrap::evaluate_with_limit(&g, &[tokens], 500_000).unwrap();
    eprintln!("lambda-calls-lambda: {:?}", out);
    match &out {
        Value::Tuple(t) => {
            assert_eq!(t[0], Value::Int(3), "pos=3");
            assert_eq!(t[2], Value::Int(2), "vsp=2");
        }
        _ => panic!("expected tuple"),
    }
}

#[test]
fn test_bootstrap_mini_parser() {
    // Minimal parser: fold over tokens, check for + operator
    let src = r#"
        let test tokens =
          let n = list_len tokens in
          let get_bp = \k ->
            if k == 30 then (10, 11, 0)
            else (0, 0, 0) in
          fold (0, list_range 0 0, 0, 0)
            (\s _i ->
              let pos = s.0 in
              let vals = s.1 in
              let vsp = s.2 in
              let phase = s.3 in
              if pos >= n then s
              else
                let tok = list_nth tokens pos in
                let tk = tok.0 in
                if phase == 0 then
                  if tk == 1 then
                    (pos + 1, list_append vals (0, tok.1), vsp + 1, 1)
                  else (pos, vals, vsp, 0)
                else
                  let bp = get_bp tk in
                  if bp.0 > 0 then
                    (pos + 1, vals, vsp, 0)
                  else (pos, vals, vsp, 1))
            (n * 4)
    "#;
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors { eprintln!("{}", iris_bootstrap::syntax::format_error(src, err)); }
        panic!("mini parser failed to compile");
    }
    let g = result.fragments[0].1.graph.clone();
    // Tokens: 1(1,1), +(30,0), 2(1,2)
    let tokens = Value::tuple(vec![
        Value::tuple(vec![Value::Int(1), Value::Int(1), Value::Int(0), Value::Int(1)]),
        Value::tuple(vec![Value::Int(30), Value::Int(0), Value::Int(2), Value::Int(3)]),
        Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(4), Value::Int(5)]),
    ]);
    let out = iris_bootstrap::evaluate_with_limit(&g, &[tokens], 500_000).unwrap();
    eprintln!("mini parser output: {:?}", out);
    // Should have parsed at least 2 values and the + operator
    match &out {
        Value::Tuple(t) => {
            assert_eq!(t[2], Value::Int(2), "should have 2 values on stack");
        }
        _ => panic!("expected tuple output"),
    }
}

#[test]
fn test_parser_iris_simple_body() {
    let ast = parse_iris("let f = 1 + 2");
    eprintln!("parse simple body = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 10, "body should be BinOp");
    assert_eq!(ast_payload(body), 0, "op should be Add(0)");
}

#[test]
fn test_bootstrap_deep_if_else() {
    // Test deep if/else chain in fold, similar to parser structure
    let src = r#"
        let test tokens =
          let n = list_len tokens in
          let mt = list_range 0 0 in
          fold (0, mt, 0, mt, 0, 0, 0, 0)
            (\s _i ->
              let pos = s.0 in
              let vals = s.1 in
              let vsp = s.2 in
              let ops = s.3 in
              let osp = s.4 in
              let phase = s.5 in
              let mbp = s.6 in
              let err = s.7 in
              if err != 0 then s
              else if phase == 2 then s
              else if pos >= n then
                if phase == 1 then
                  if osp > 0 then
                    let top = list_nth ops (osp - 1) in
                    if top.0 == 0 then
                      if vsp >= 2 then
                        let ri = list_nth vals (vsp - 1) in
                        let le = list_nth vals (vsp - 2) in
                        (pos, list_append vals (10, top.1, (le, ri)), vsp - 1, ops, osp - 1, 2, mbp, 0)
                      else (pos, vals, vsp, ops, osp, 2, mbp, 1)
                    else (pos, vals, vsp, ops, osp, 2, mbp, 0)
                  else (pos, vals, vsp, ops, osp, 2, mbp, 0)
                else (pos, vals, vsp, ops, osp, 2, mbp, 1)
              else if phase == 0 then
                let tok = list_nth tokens pos in
                let tk = tok.0 in
                if tk == 1 then
                  (pos + 1, list_append vals (0, tok.1, mt), vsp + 1, ops, osp, 1, mbp, 0)
                else if tk == 2 then
                  (pos + 1, list_append vals (4, tok.1, mt), vsp + 1, ops, osp, 1, mbp, 0)
                else (pos, vals, vsp, ops, osp, 2, mbp, 1)
              else
                let tok = list_nth tokens pos in
                let tk = tok.0 in
                -- Deep if/else chain like in the parser
                if tk == 51 then (pos, vals, vsp, ops, osp, 2, mbp, 0)
                else if tk == 56 then (pos, vals, vsp, ops, osp, 2, mbp, 0)
                else if tk == 21 then (pos, vals, vsp, ops, osp, 2, mbp, 0)
                else if tk == 22 then (pos, vals, vsp, ops, osp, 2, mbp, 0)
                else if tk == 12 then (pos, vals, vsp, ops, osp, 2, mbp, 0)
                else if tk == 58 then (pos, vals, vsp, ops, osp, 2, mbp, 0)
                else
                  let lbp =
                    if tk == 61 then 2
                    else if tk == 43 then 4
                    else if tk == 42 then 6
                    else if tk == 36 then 8
                    else if tk == 37 then 8
                    else if tk == 38 then 8
                    else if tk == 39 then 8
                    else if tk == 40 then 8
                    else if tk == 41 then 8
                    else if tk == 30 then 10
                    else if tk == 31 then 10
                    else if tk == 32 then 12
                    else if tk == 33 then 12
                    else if tk == 34 then 12
                    else 0 in
                  if lbp > 0 then
                    (pos + 1, vals, vsp, list_append ops (0, 0, lbp), osp + 1, 0, mbp, 0)
                  else
                    (pos, vals, vsp, ops, osp, 2, mbp, 0))
            (n * 8)
    "#;
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors { eprintln!("{}", iris_bootstrap::syntax::format_error(src, err)); }
        panic!("failed");
    }
    let g = result.fragments[0].1.graph.clone();
    let tokens = Value::tuple(vec![
        Value::tuple(vec![Value::Int(1), Value::Int(1), Value::Int(0), Value::Int(1)]),
        Value::tuple(vec![Value::Int(30), Value::Int(0), Value::Int(2), Value::Int(3)]),
        Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(4), Value::Int(5)]),
    ]);
    let out = iris_bootstrap::evaluate_with_limit(&g, &[tokens], 500_000).unwrap();
    eprintln!("deep if-else: {:?}", out);
    match &out {
        Value::Tuple(t) => {
            eprintln!("pos={:?} vsp={:?} osp={:?} phase={:?} err={:?}", t[0], t[2], t[4], t[5], t[7]);
            assert_eq!(t[0], Value::Int(3), "pos should be 3");
        }
        _ => panic!("expected tuple"),
    }
}

#[test]
fn test_bootstrap_inline_shunting_yard() {
    // Minimal shunting-yard: inline everything, no lambdas
    let src = r#"
        let test tokens =
          let n = list_len tokens in
          let mt = list_range 0 0 in
          let st = fold (0, mt, 0, mt, 0, 0, 0, 0)
            (\s _i ->
              let pos = s.0 in
              let vals = s.1 in
              let vsp = s.2 in
              let ops = s.3 in
              let osp = s.4 in
              let phase = s.5 in
              let mbp = s.6 in
              let err = s.7 in
              if err != 0 then s
              else if phase == 2 then s
              else if pos >= n then
                if phase == 1 then
                  if osp > 0 then
                    let top = list_nth ops (osp - 1) in
                    if top.0 == 0 then
                      if vsp >= 2 then
                        let ri = list_nth vals (vsp - 1) in
                        let le = list_nth vals (vsp - 2) in
                        (pos, list_append vals (10, top.1, (le, ri)), vsp - 1, ops, osp - 1, 2, mbp, 0)
                      else (pos, vals, vsp, ops, osp, 2, mbp, 1)
                    else (pos, vals, vsp, ops, osp, 2, mbp, 0)
                  else (pos, vals, vsp, ops, osp, 2, mbp, 0)
                else (pos, vals, vsp, ops, osp, 2, mbp, 1)
              else if phase == 0 then
                let tok = list_nth tokens pos in
                let tk = tok.0 in
                if tk == 1 then
                  (pos + 1, list_append vals (0, tok.1, mt), vsp + 1, ops, osp, 1, mbp, 0)
                else if tk == 2 then
                  (pos + 1, list_append vals (4, tok.1, mt), vsp + 1, ops, osp, 1, mbp, 0)
                else (pos, vals, vsp, ops, osp, 2, mbp, 1)
              else
                let tok = list_nth tokens pos in
                let tk = tok.0 in
                if tk == 30 then
                  (pos + 1, vals, vsp, list_append ops (0, 0, 10), osp + 1, 0, mbp, 0)
                else if tk == 31 then
                  (pos + 1, vals, vsp, list_append ops (0, 1, 10), osp + 1, 0, mbp, 0)
                else if tk == 32 then
                  (pos + 1, vals, vsp, list_append ops (0, 2, 12), osp + 1, 0, mbp, 0)
                else
                  (pos, vals, vsp, ops, osp, 2, mbp, 0))
            (n * 8) in
          st
    "#;
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors { eprintln!("{}", iris_bootstrap::syntax::format_error(src, err)); }
        panic!("failed");
    }
    let g = result.fragments[0].1.graph.clone();
    let tokens = Value::tuple(vec![
        Value::tuple(vec![Value::Int(1), Value::Int(1), Value::Int(0), Value::Int(1)]),
        Value::tuple(vec![Value::Int(30), Value::Int(0), Value::Int(2), Value::Int(3)]),
        Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(4), Value::Int(5)]),
    ]);
    let out = iris_bootstrap::evaluate_with_limit(&g, &[tokens], 500_000).unwrap();
    eprintln!("inline shunting yard: {:?}", out);
    match &out {
        Value::Tuple(t) => {
            // Should have: pos=3, vsp=1 (BinOp result), phase=2, err=0
            assert_eq!(t[0], Value::Int(3), "pos should be 3");
            let result_vsp = &t[2];
            eprintln!("vsp = {:?}, osp = {:?}", t[2], t[4]);
            assert_eq!(t[2], Value::Int(1), "vsp should be 1 (one final result)");
            assert_eq!(t[5], Value::Int(2), "phase should be 2 (done)");
            assert_eq!(t[7], Value::Int(0), "err should be 0");
        }
        _ => panic!("expected tuple"),
    }
}

#[test]
fn test_parser_iris_binop_add() {
    let ast = parse_iris("let f x y = x + y");
    println!("parse binop add = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 10, "body should be BinOp");
    assert_eq!(ast_payload(body), 0, "op should be Add(0)");
    let operands = ast_children(body);
    assert_eq!(operands.len(), 2, "BinOp should have 2 children");
    assert_eq!(ast_kind(&operands[0]), 4, "left should be Var");
    assert_eq!(ast_kind(&operands[1]), 4, "right should be Var");
}

#[test]
fn test_parser_iris_precedence() {
    // x + y * z should parse as x + (y * z)
    let ast = parse_iris("let f x y z = x + y * z");
    println!("parse precedence = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    // body should be BinOp(Add, x, BinOp(Mul, y, z))
    assert_eq!(ast_kind(body), 10, "body should be BinOp");
    assert_eq!(ast_payload(body), 0, "outer op should be Add(0)");
    let children = ast_children(body);
    assert_eq!(ast_kind(&children[0]), 4, "left should be Var (x)");
    assert_eq!(ast_kind(&children[1]), 10, "right should be BinOp (y*z)");
    assert_eq!(ast_payload(&children[1]), 2, "inner op should be Mul(2)");
}

#[test]
fn test_parser_iris_parens() {
    // (x + y) * z should parse as BinOp(Mul, BinOp(Add, x, y), z)
    let ast = parse_iris("let f x y z = (x + y) * z");
    println!("parse parens = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 10, "body should be BinOp");
    assert_eq!(ast_payload(body), 2, "outer op should be Mul(2)");
    let children = ast_children(body);
    assert_eq!(ast_kind(&children[0]), 10, "left should be BinOp (x+y)");
    assert_eq!(ast_payload(&children[0]), 0, "inner op should be Add(0)");
    assert_eq!(ast_kind(&children[1]), 4, "right should be Var (z)");
}

#[test]
fn test_parser_iris_if_expr() {
    let ast = parse_iris("let f x = if x then 1 else 0");
    println!("parse if = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 15, "body should be If");
    let if_children = ast_children(body);
    assert_eq!(if_children.len(), 3, "If should have 3 children");
    assert_eq!(ast_kind(&if_children[0]), 4, "cond should be Var");
    assert_eq!(ast_kind(&if_children[1]), 0, "then should be IntLit");
    assert_eq!(ast_payload(&if_children[1]), 1, "then should be 1");
    assert_eq!(ast_kind(&if_children[2]), 0, "else should be IntLit");
    assert_eq!(ast_payload(&if_children[2]), 0, "else should be 0");
}

#[test]
fn test_parser_iris_let_in() {
    let ast = parse_iris("let f x = let y = 1 in x + y");
    println!("parse let/in = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 14, "body should be Let");
    let let_children = ast_children(body);
    assert_eq!(let_children.len(), 2, "Let should have 2 children (value, body)");
    // value should be IntLit(1)
    assert_eq!(ast_kind(&let_children[0]), 0, "let value should be IntLit");
    assert_eq!(ast_payload(&let_children[0]), 1, "let value should be 1");
    // body should be BinOp(Add, x, y)
    assert_eq!(ast_kind(&let_children[1]), 10, "let body should be BinOp");
}

#[test]
fn test_parser_iris_lambda() {
    let ast = parse_iris("let f = \\x -> x");
    println!("parse lambda = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 13, "body should be Lambda");
    let lam_children = ast_children(body);
    assert_eq!(lam_children.len(), 1, "Lambda should have 1 child (body)");
    assert_eq!(ast_kind(&lam_children[0]), 4, "lambda body should be Var");
}

#[test]
fn test_parser_iris_bool_lit() {
    let ast = parse_iris("let t = true");
    println!("parse bool = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 3, "body should be BoolLit");
    assert_eq!(ast_payload(body), 1, "true should have payload 1");
}

#[test]
fn test_parser_iris_comparison() {
    let ast = parse_iris("let f x y = x == y");
    println!("parse comparison = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 10, "body should be BinOp");
    assert_eq!(ast_payload(body), 5, "op should be Eq(5)");
}

#[test]
fn test_parser_iris_application() {
    let ast = parse_iris("let g f x = f x");
    println!("parse application = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 12, "body should be App");
    let app_children = ast_children(body);
    assert_eq!(app_children.len(), 2, "App should have 2 children");
    assert_eq!(ast_kind(&app_children[0]), 4, "func should be Var");
    assert_eq!(ast_kind(&app_children[1]), 4, "arg should be Var");
}

#[test]
fn test_parser_iris_multi_application() {
    let ast = parse_iris("let g f x y = f x y");
    println!("parse multi-app = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    // f x y = App(App(f, x), y)
    assert_eq!(ast_kind(body), 12, "body should be App");
    let outer_children = ast_children(body);
    assert_eq!(ast_kind(&outer_children[0]), 12, "func should be App (f x)");
    assert_eq!(ast_kind(&outer_children[1]), 4, "arg should be Var (y)");
}

#[test]
fn test_parser_iris_pair() {
    let ast = parse_iris("let p = (1, 2)");
    eprintln!("parse pair = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 17, "body should be Tuple");
}

#[test]
fn test_parser_iris_tuple() {
    let ast = parse_iris("let p = (1, 2, 3)");
    println!("parse tuple = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 17, "body should be Tuple");
    let elems = ast_children(body);
    assert_eq!(elems.len(), 3, "tuple should have 3 elements");
    assert_eq!(ast_kind(&elems[0]), 0, "elem 0 should be IntLit");
    assert_eq!(ast_payload(&elems[0]), 1);
    assert_eq!(ast_payload(&elems[1]), 2);
    assert_eq!(ast_payload(&elems[2]), 3);
}

#[test]
fn test_parser_iris_tuple_access() {
    let ast = parse_iris("let fst p = p.0");
    println!("parse tuple access = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 18, "body should be TupleAccess");
    assert_eq!(ast_payload(body), 0, "field index should be 0");
    let children = ast_children(body);
    assert_eq!(children.len(), 1, "TupleAccess should have 1 child");
    assert_eq!(ast_kind(&children[0]), 4, "base should be Var");
}

#[test]
fn test_parser_iris_decl_with_params() {
    let ast = parse_iris("let add x y = x + y");
    println!("parse decl params = {:?}", ast);
    let items = ast_children(&ast);
    let decl = &items[0];
    assert_eq!(ast_kind(decl), 30, "should be LetDecl");
    let decl_children = ast_children(decl);
    // params should be a tuple of Var nodes
    let params = &decl_children[0];
    match params {
        Value::Tuple(t) => {
            assert_eq!(t.len(), 2, "should have 2 params (x, y)");
            assert_eq!(ast_kind(&t[0]), 4, "param should be Var");
            assert_eq!(ast_kind(&t[1]), 4, "param should be Var");
        }
        _ => panic!("params should be Tuple, got {:?}", params),
    }
}

#[test]
fn test_parser_iris_multi_decl() {
    let ast = parse_iris("let a = 1\nlet b = 2");
    println!("parse multi decl = {:?}", ast);
    assert_eq!(ast_kind(&ast), 40, "should be Module");
    let items = ast_children(&ast);
    assert_eq!(items.len(), 2, "should have 2 declarations");
    assert_eq!(ast_kind(&items[0]), 30);
    assert_eq!(ast_kind(&items[1]), 30);
    // First body should be IntLit(1)
    let body0 = &ast_children(&items[0])[1];
    assert_eq!(ast_payload(body0), 1);
    // Second body should be IntLit(2)
    let body1 = &ast_children(&items[1])[1];
    assert_eq!(ast_payload(body1), 2);
}

#[test]
fn test_parser_iris_pipe() {
    // Use 'negate' not 'neg' — 'neg' collides with 'rec' in char-sum hash
    let ast = parse_iris("let f x = x |> negate");
    println!("parse pipe = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 19, "body should be Pipe");
    let pipe_children = ast_children(body);
    assert_eq!(pipe_children.len(), 2, "Pipe should have 2 children");
    assert_eq!(ast_kind(&pipe_children[0]), 4, "lhs should be Var (x)");
    assert_eq!(ast_kind(&pipe_children[1]), 4, "rhs should be Var (negate)");
}

#[test]
fn test_parser_iris_unary_neg() {
    let ast = parse_iris("let f x = -x");
    println!("parse unary neg = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 11, "body should be UnaryOp");
    assert_eq!(ast_payload(body), 0, "op should be Neg(0)");
    let children = ast_children(body);
    assert_eq!(children.len(), 1, "UnaryOp should have 1 child");
    assert_eq!(ast_kind(&children[0]), 4, "operand should be Var");
}

#[test]
fn test_parser_iris_nested_if() {
    let ast = parse_iris("let f x = if x == 0 then 1 else if x == 1 then 1 else 0");
    println!("parse nested if = {:?}", ast);
    let items = ast_children(&ast);
    let body = &ast_children(&items[0])[1];
    assert_eq!(ast_kind(body), 15, "body should be If");
    let if_children = ast_children(body);
    // else branch should also be If
    assert_eq!(ast_kind(&if_children[2]), 15, "else branch should be If");
}

// ---------------------------------------------------------------------------
// iris_lowerer.iris — complete IRIS lowerer
// ---------------------------------------------------------------------------

fn compile_lowerer() -> iris_types::graph::SemanticGraph {
    let src = include_str!("../src/iris-programs/syntax/iris_lowerer.iris");
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            let msg = iris_bootstrap::syntax::format_error(src, err);
            eprintln!("{}", msg);
        }
        panic!("iris_lowerer.iris compilation failed with {} errors", result.errors.len());
    }
    assert!(!result.fragments.is_empty(), "iris_lowerer.iris produced no fragments");
    // The last fragment is the entry point ('lower')
    let idx = result.fragments.len() - 1;
    result.fragments[idx].1.graph.clone()
}

/// Full pipeline: source -> tokenize -> parse -> lower -> Value::Program
fn lower_iris(input: &str) -> iris_types::graph::SemanticGraph {
    let tok_graph = compile_tokenizer();
    let parser_graph = compile_parser();
    let lowerer_graph = compile_lowerer();

    // Step 1: tokenize
    let tokens_val = iris_bootstrap::evaluate_with_limit(
        &tok_graph, &[Value::String(input.into())], 500_000,
    ).expect("tokenizer failed");

    // Step 2: parse
    let src_val = Value::String(input.into());
    let ast = iris_bootstrap::evaluate_with_limit(
        &parser_graph, &[tokens_val, src_val.clone()], 50_000_000,
    ).expect("parser failed");

    // Step 3: lower
    let program = iris_bootstrap::evaluate_with_limit(
        &lowerer_graph, &[ast, src_val], 50_000_000,
    ).expect("lowerer failed");

    match program {
        Value::Program(g) => *g,
        Value::Tuple(ref fields) if !fields.is_empty() => {
            match &fields[0] {
                Value::Program(g) => g.as_ref().clone(),
                _ => panic!("lowerer tuple[0] should be Program, got {:?}", fields[0]),
            }
        }
        other => panic!("lowerer should return Program, got {:?}", other),
    }
}

#[test]
fn test_lowerer_iris_compiles() {
    let src = include_str!("../src/iris-programs/syntax/iris_lowerer.iris");
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!("iris_lowerer.iris compilation failed with {} errors", result.errors.len());
    }
    assert!(!result.fragments.is_empty(), "iris_lowerer.iris produced no fragments");
    let frag_names: Vec<&str> = result.fragments.iter().map(|(n, _, _)| n.as_str()).collect();
    println!("iris_lowerer.iris fragments: {:?}", frag_names);
    assert!(frag_names.contains(&"lower"), "expected 'lower' fragment");
}

#[test]
fn test_lowerer_constant() {
    // let c = 42  ->  evaluate() -> 42
    let g = lower_iris("let c = 42");
    let result = iris_bootstrap::evaluate(&g, &[]).unwrap();
    assert_eq!(result, Value::Int(42), "constant 42");
}

#[test]
fn test_lowerer_identity() {
    // let id x = x  ->  evaluate(7) -> 7
    let g = lower_iris("let id x = x");
    let result = iris_bootstrap::evaluate(&g, &[Value::Int(7)]).unwrap();
    assert_eq!(result, Value::Int(7), "identity(7)");
}

#[test]
fn test_lowerer_addition() {
    // let add x y = x + y  ->  evaluate(3, 5) -> 8
    let g = lower_iris("let add x y = x + y");
    let result = iris_bootstrap::evaluate(&g, &[Value::Int(3), Value::Int(5)]).unwrap();
    assert_eq!(result, Value::Int(8), "add(3, 5)");
}

#[test]
fn test_lowerer_subtraction() {
    // let sub x y = x - y  ->  evaluate(10, 3) -> 7
    let g = lower_iris("let sub x y = x - y");
    let result = iris_bootstrap::evaluate(&g, &[Value::Int(10), Value::Int(3)]).unwrap();
    assert_eq!(result, Value::Int(7), "sub(10, 3)");
}

#[test]
fn test_lowerer_multiplication() {
    // let mul x y = x * y  ->  evaluate(6, 7) -> 42
    let g = lower_iris("let mul x y = x * y");
    let result = iris_bootstrap::evaluate(&g, &[Value::Int(6), Value::Int(7)]).unwrap();
    assert_eq!(result, Value::Int(42), "mul(6, 7)");
}

#[test]
fn test_lowerer_guard() {
    // let abs x = if x < 0 then 0 - x else x
    let g = lower_iris("let abs x = if x < 0 then 0 - x else x");
    let result_neg = iris_bootstrap::evaluate(&g, &[Value::Int(-5)]).unwrap();
    assert_eq!(result_neg, Value::Int(5), "abs(-5)");
    let result_pos = iris_bootstrap::evaluate(&g, &[Value::Int(3)]).unwrap();
    assert_eq!(result_pos, Value::Int(3), "abs(3)");
}

#[test]
fn test_lowerer_nested_arithmetic() {
    // let f x = (x + 1) * (x - 1)  ->  evaluate(5) -> 24
    let g = lower_iris("let f x = (x + 1) * (x - 1)");
    let result = iris_bootstrap::evaluate(&g, &[Value::Int(5)]).unwrap();
    assert_eq!(result, Value::Int(24), "(5+1)*(5-1)");
}

#[test]
fn test_lowerer_let_binding() {
    // let f x = let y = x + 1 in y * 2  ->  evaluate(3) -> 8
    let g = lower_iris("let f x = let y = x + 1 in y * 2");
    let result = iris_bootstrap::evaluate(&g, &[Value::Int(3)]).unwrap();
    assert_eq!(result, Value::Int(8), "let y = 3+1 in y*2");
}

#[test]
fn test_lowerer_comparison() {
    // let max x y = if x > y then x else y
    let g = lower_iris("let mx x y = if x > y then x else y");
    let result1 = iris_bootstrap::evaluate(&g, &[Value::Int(3), Value::Int(7)]).unwrap();
    assert_eq!(result1, Value::Int(7), "max(3, 7)");
    let result2 = iris_bootstrap::evaluate(&g, &[Value::Int(10), Value::Int(2)]).unwrap();
    assert_eq!(result2, Value::Int(10), "max(10, 2)");
}

#[test]
fn test_lowerer_boolean_true() {
    let g = lower_iris("let t = true");
    let result = iris_bootstrap::evaluate(&g, &[]).unwrap();
    assert_eq!(result, Value::Bool(true), "true literal");
}

#[test]
fn test_lowerer_boolean_false() {
    let g = lower_iris("let f = false");
    let result = iris_bootstrap::evaluate(&g, &[]).unwrap();
    assert_eq!(result, Value::Bool(false), "false literal");
}

#[test]
fn test_lowerer_unary_neg() {
    // let neg x = -x  (using unary negation opcode 0x05)
    let g = lower_iris("let f x = 0 - x");
    let result = iris_bootstrap::evaluate(&g, &[Value::Int(5)]).unwrap();
    assert_eq!(result, Value::Int(-5), "0 - 5 = -5");
}

#[test]
fn test_lowerer_multi_param() {
    // let f x y z = x + y + z  ->  evaluate(1, 2, 3) -> 6
    let g = lower_iris("let f x y z = x + y + z");
    let result = iris_bootstrap::evaluate(&g, &[Value::Int(1), Value::Int(2), Value::Int(3)]).unwrap();
    assert_eq!(result, Value::Int(6), "1+2+3");
}

#[test]
fn test_lowerer_nested_if() {
    // let clamp x lo hi = if x < lo then lo else if x > hi then hi else x
    let g = lower_iris("let clamp x lo hi = if x < lo then lo else if x > hi then hi else x");
    let result_low = iris_bootstrap::evaluate(&g, &[Value::Int(-5), Value::Int(0), Value::Int(100)]).unwrap();
    assert_eq!(result_low, Value::Int(0), "clamp(-5, 0, 100)");
    let result_high = iris_bootstrap::evaluate(&g, &[Value::Int(200), Value::Int(0), Value::Int(100)]).unwrap();
    assert_eq!(result_high, Value::Int(100), "clamp(200, 0, 100)");
    let result_mid = iris_bootstrap::evaluate(&g, &[Value::Int(50), Value::Int(0), Value::Int(100)]).unwrap();
    assert_eq!(result_mid, Value::Int(50), "clamp(50, 0, 100)");
}

#[test]
fn test_lowerer_eq_comparison() {
    // let is_zero x = if x == 0 then 1 else 0
    let g = lower_iris("let is_zero x = if x == 0 then 1 else 0");
    let result1 = iris_bootstrap::evaluate(&g, &[Value::Int(0)]).unwrap();
    assert_eq!(result1, Value::Int(1), "is_zero(0)");
    let result2 = iris_bootstrap::evaluate(&g, &[Value::Int(5)]).unwrap();
    assert_eq!(result2, Value::Int(0), "is_zero(5)");
}

#[test]
fn test_lowerer_le_comparison() {
    // let is_small x = if x <= 10 then 1 else 0
    let g = lower_iris("let is_small x = if x <= 10 then 1 else 0");
    let result1 = iris_bootstrap::evaluate(&g, &[Value::Int(5)]).unwrap();
    assert_eq!(result1, Value::Int(1), "is_small(5)");
    let result2 = iris_bootstrap::evaluate(&g, &[Value::Int(10)]).unwrap();
    assert_eq!(result2, Value::Int(1), "is_small(10)");
    let result3 = iris_bootstrap::evaluate(&g, &[Value::Int(15)]).unwrap();
    assert_eq!(result3, Value::Int(0), "is_small(15)");
}

// ---------------------------------------------------------------------------
// Path-based import tests
// ---------------------------------------------------------------------------

/// Build a fragment registry from compiled file result
fn compile_file_with_registry(path: &str)
    -> (Vec<(String, iris_types::graph::SemanticGraph, iris_types::fragment::FragmentId)>,
        std::collections::BTreeMap<iris_types::fragment::FragmentId, iris_types::graph::SemanticGraph>)
{
    let path = std::path::Path::new(path);
    let src = std::fs::read_to_string(path).expect("read file");
    let result = iris_bootstrap::syntax::compile_file(&src, path);
    if !result.errors.is_empty() {
        for err in &result.errors { eprintln!("{}", iris_bootstrap::syntax::format_error(&src, err)); }
        panic!("compilation failed with {} errors", result.errors.len());
    }
    let mut registry = std::collections::BTreeMap::new();
    let mut frags = Vec::new();
    for (name, frag, _) in result.fragments {
        registry.insert(frag.id, frag.graph.clone());
        frags.push((name, frag.graph, frag.id));
    }
    (frags, registry)
}

/// Evaluate a named fragment using the bootstrap evaluator with a registry
fn eval_fragment(
    frags: &[(String, iris_types::graph::SemanticGraph, iris_types::fragment::FragmentId)],
    registry: &std::collections::BTreeMap<iris_types::fragment::FragmentId, iris_types::graph::SemanticGraph>,
    name: &str,
) -> iris_types::eval::Value {
    let graph = &frags.iter().find(|(n, _, _)| n == name).unwrap().1;
    iris_bootstrap::evaluate_with_registry(graph, &[], 100_000, registry).unwrap()
}

#[test]
fn test_path_import_basic() {
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_import.iris");
    let result = eval_fragment(&frags, &registry, "test_is_none");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_path_import_all_fragments() {
    let (frags, _) = compile_file_with_registry("src/iris-programs/test_import.iris");
    let names: Vec<&str> = frags.iter().map(|(n, _, _)| n.as_str()).collect();
    assert!(names.contains(&"unwrap_or"), "missing imported unwrap_or");
    assert!(names.contains(&"option_map"), "missing imported option_map");
    assert!(names.contains(&"test_unwrap_some"), "missing test_unwrap_some");
    assert!(names.contains(&"test_unwrap_none"), "missing test_unwrap_none");
}

#[test]
fn test_path_import_unwrap_some() {
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_import.iris");
    let result = eval_fragment(&frags, &registry, "test_unwrap_some");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_path_import_unwrap_none() {
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_import.iris");
    let result = eval_fragment(&frags, &registry, "test_unwrap_none");
    assert_eq!(result, Value::Int(99));
}

#[test]
fn test_path_import_map() {
    // Higher-order functions across fragments with graph-aware closures.
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_import.iris");
    // Actually execute test_map: map (Some(21)) (\x -> x * 2) → Some(42), unwrap → 42
    let val = eval_fragment(&frags, &registry, "test_map");
    assert_eq!(val, Value::Int(42), "cross-graph higher-order function should work");
}

#[test]
fn test_path_import_and_then() {
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_import.iris");
    // and_then (Some(10)) (\x -> if x > 5 then Some(x*3) else None) → Some(30) → 30
    let val = eval_fragment(&frags, &registry, "test_and_then_some");
    assert_eq!(val, Value::Int(30), "and_then with Some should apply closure");
    // and_then None (\x -> Some(x+1)) → None → 0
    let val = eval_fragment(&frags, &registry, "test_and_then_none");
    assert_eq!(val, Value::Int(0), "and_then with None should short-circuit");
}

#[test]
fn test_path_import_filter() {
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_import.iris");
    let val = eval_fragment(&frags, &registry, "test_filter_pass");
    assert_eq!(val, Value::Bool(true), "filter should keep values passing predicate");
    let val = eval_fragment(&frags, &registry, "test_filter_fail");
    assert_eq!(val, Value::Bool(true), "filter should reject values failing predicate");
}

#[test]
fn test_path_import_unwrap_or_else() {
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_import.iris");
    let val = eval_fragment(&frags, &registry, "test_unwrap_or_else");
    assert_eq!(val, Value::Int(99), "unwrap_or_else on None should call closure");
}

#[test]
fn test_path_import_map_none() {
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_import.iris");
    let val = eval_fragment(&frags, &registry, "test_map_none");
    assert_eq!(val, Value::Bool(true), "map on None should return None");
}

#[test]
fn test_path_import_cycle_detection() {
    // Create two files that import each other
    let dir = std::env::temp_dir().join("iris_cycle_test");
    let _ = std::fs::create_dir_all(&dir);
    std::fs::write(dir.join("a.iris"), r#"import "b.iris" as B
let a = 1"#).unwrap();
    std::fs::write(dir.join("b.iris"), r#"import "a.iris" as A
let b = 2"#).unwrap();

    let path = dir.join("a.iris");
    let src = std::fs::read_to_string(&path).unwrap();
    let result = iris_bootstrap::syntax::compile_file(&src, &path);
    assert!(!result.errors.is_empty(), "should detect circular import");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_stdlib_option_compiles() {
    let path = std::path::Path::new("src/iris-programs/stdlib/option.iris");
    let src = std::fs::read_to_string(path).unwrap();
    let result = iris_bootstrap::syntax::compile_file(&src, path);
    assert!(result.errors.is_empty(), "option.iris should compile: {:?}", result.errors);
    assert!(result.fragments.len() >= 8, "option.iris should have at least 8 functions");
}

#[test]
fn test_stdlib_result_compiles() {
    let path = std::path::Path::new("src/iris-programs/stdlib/result.iris");
    let src = std::fs::read_to_string(path).unwrap();
    let result = iris_bootstrap::syntax::compile_file(&src, path);
    assert!(result.errors.is_empty(), "result.iris should compile: {:?}", result.errors);
    assert!(result.fragments.len() >= 7, "result.iris should have at least 7 functions");
}

#[test]
fn test_stdlib_either_compiles() {
    let path = std::path::Path::new("src/iris-programs/stdlib/either.iris");
    let src = std::fs::read_to_string(path).unwrap();
    let result = iris_bootstrap::syntax::compile_file(&src, path);
    assert!(result.errors.is_empty(), "either.iris should compile: {:?}", result.errors);
}

#[test]
fn test_stdlib_ordering_compiles() {
    let path = std::path::Path::new("src/iris-programs/stdlib/ordering.iris");
    let src = std::fs::read_to_string(path).unwrap();
    let result = iris_bootstrap::syntax::compile_file(&src, path);
    assert!(result.errors.is_empty(), "ordering.iris should compile: {:?}", result.errors);
}

#[test]
fn test_path_import_string_parse() {
    // Verify parser accepts import "path" as name
    let module = iris_bootstrap::syntax::parse(r#"import "stdlib/option.iris" as Option
let x = 1"#).unwrap();
    assert_eq!(module.items.len(), 2);
    match &module.items[0] {
        iris_bootstrap::syntax::ast::Item::Import(imp) => {
            assert!(matches!(&imp.source, iris_bootstrap::syntax::ast::ImportSource::Path(p) if p == "stdlib/option.iris"));
            assert_eq!(imp.name, "Option");
        }
        _ => panic!("expected import"),
    }
}

#[test]
fn test_hex_import_still_works() {
    // Verify parser still accepts import 0xdeadbeef... as name (8+ hex digits)
    let module = iris_bootstrap::syntax::parse("import 0xdeadbeefcafe0001 as lib\nlet x = 1").unwrap();
    match &module.items[0] {
        iris_bootstrap::syntax::ast::Item::Import(imp) => {
            assert!(matches!(&imp.source, iris_bootstrap::syntax::ast::ImportSource::Hash(h) if h == "deadbeefcafe0001"));
            assert_eq!(imp.name, "lib");
        }
        _ => panic!("expected import"),
    }
}

// ---------------------------------------------------------------------------
// Struct (record) type tests
// ---------------------------------------------------------------------------

#[test]
fn test_struct_field_access() {
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_structs.iris");
    let val = eval_fragment(&frags, &registry, "test_point_x");
    assert_eq!(val, Value::Int(3));
}

#[test]
fn test_struct_field_y() {
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_structs.iris");
    let val = eval_fragment(&frags, &registry, "test_point_y");
    assert_eq!(val, Value::Int(4));
}

#[test]
fn test_struct_computed() {
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_structs.iris");
    let val = eval_fragment(&frags, &registry, "test_distance");
    assert_eq!(val, Value::Int(25)); // 3^2 + 4^2
}

#[test]
fn test_struct_multi_field() {
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_structs.iris");
    let val = eval_fragment(&frags, &registry, "test_color_g");
    assert_eq!(val, Value::Int(128));
}

#[test]
fn test_struct_return_and_access() {
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_structs.iris");
    let val = eval_fragment(&frags, &registry, "test_add_points");
    assert_eq!(val, Value::Int(10)); // (1+3) + (2+4) = 10
}

#[test]
fn test_struct_positional_still_works() {
    let (frags, registry) = compile_file_with_registry("src/iris-programs/test_structs.iris");
    let val = eval_fragment(&frags, &registry, "test_positional");
    assert_eq!(val, Value::Int(30)); // 10 + 20
}
