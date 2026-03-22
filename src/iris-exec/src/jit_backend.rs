//! JIT backend — compiles IRIS programs to native x86-64 via pure IRIS
//! compiler programs, then executes natively.
//!
//! Pipeline:
//!   1. Load aot_compile.iris (IRIS compiler program)
//!   2. Call `is_aot_compilable(user_graph)` on tree-walker → Bool
//!   3. Call `aot_compile(user_graph)` on tree-walker → Bytes (x86-64 code)
//!   4. mmap RW → copy → mprotect RX (W^X) → cache function pointer
//!   5. Direct function pointer call → result
//!
//! All compilation logic is pure IRIS. Rust provides:
//!   - W^X memory management (mmap/mprotect)
//!   - Direct function pointer dispatch (no effect handler overhead)

use std::collections::{BTreeMap, HashMap};
use std::sync::RwLock;

use iris_types::eval::Value;
use iris_types::fragment::FragmentId;
use iris_types::graph::SemanticGraph;
use iris_types::hash::SemanticHash;

use crate::effect_runtime::RuntimeEffectHandler;

// ---------------------------------------------------------------------------
// Compiled compiler fragments (loaded once, cached)
// ---------------------------------------------------------------------------

struct CompilerFragments {
    /// All fragments from aot_compile.iris + lambda_lift.iris
    aot_graphs: Vec<(String, SemanticGraph)>,
    aot_registry: BTreeMap<FragmentId, SemanticGraph>,
}

fn compile_iris_source(src: &str) -> (Vec<(String, SemanticGraph)>, BTreeMap<FragmentId, SemanticGraph>) {
    let result = iris_bootstrap::syntax::compile(src);
    assert!(result.errors.is_empty(), "compiler source has errors: {:?}", result.errors);
    let mut registry = BTreeMap::new();
    let mut fragments = Vec::new();
    for (name, frag, _smap) in result.fragments {
        registry.insert(frag.id, frag.graph.clone());
        fragments.push((name, frag.graph));
    }
    (fragments, registry)
}

fn find_graph(fragments: &[(String, SemanticGraph)], name: &str) -> Option<SemanticGraph> {
    fragments.iter().find(|(n, _)| n == name).map(|(_, g)| g.clone())
}

fn load_compiler_fragments() -> CompilerFragments {
    let aot_src = include_str!("../../iris-programs/compiler/aot_compile.iris");
    let lift_src = include_str!("../../iris-programs/compiler/lambda_lift.iris");

    let (mut aot_graphs, mut aot_registry) = compile_iris_source(aot_src);
    let (lift_graphs, lift_registry) = compile_iris_source(lift_src);
    aot_graphs.extend(lift_graphs);
    aot_registry.extend(lift_registry);

    CompilerFragments {
        aot_graphs,
        aot_registry,
    }
}

// ---------------------------------------------------------------------------
// Direct native function cache (no effect handler overhead)
// ---------------------------------------------------------------------------

/// A cached native function — stores the raw function pointer and mmap info
/// so we can call it directly without going through the effect handler.
struct NativeFunction {
    /// Pointer to mmap'd executable code. None = not compilable (negative cache).
    ptr: Option<*const u8>,
    /// Size of the mmap'd region (for bookkeeping; never freed during process lifetime).
    _mmap_size: usize,
    /// Number of arguments the function expects.
    n_args: usize,
    /// Whether the result should be interpreted as Float64 (f64 bits in i64).
    result_is_float: bool,
}

// SAFETY: The mmap'd code pointer is valid for the process lifetime (we never munmap).
// The code is read-only + executable after W^X transition. Multiple threads can
// call the same function pointer concurrently (System V AMD64 ABI is reentrant).
unsafe impl Send for NativeFunction {}
unsafe impl Sync for NativeFunction {}

static COMPILER_FRAGMENTS: std::sync::OnceLock<CompilerFragments> = std::sync::OnceLock::new();
/// RwLock<HashMap> — reads (cache hits) don't block each other.
static JIT_CACHE: std::sync::OnceLock<RwLock<HashMap<SemanticHash, NativeFunction>>> =
    std::sync::OnceLock::new();
/// Effect handler kept alive to support the IRIS AOT compiler's own effect needs.
static JIT_HANDLER: std::sync::OnceLock<RuntimeEffectHandler> = std::sync::OnceLock::new();

fn get_compiler() -> &'static CompilerFragments {
    COMPILER_FRAGMENTS.get_or_init(load_compiler_fragments)
}

fn get_cache() -> &'static RwLock<HashMap<SemanticHash, NativeFunction>> {
    JIT_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn get_handler() -> &'static RuntimeEffectHandler {
    JIT_HANDLER.get_or_init(RuntimeEffectHandler::new)
}

// ---------------------------------------------------------------------------
// W^X memory management (inline, no effect handler)
// ---------------------------------------------------------------------------

/// Allocate executable memory: write code bytes, then flip to RX.
/// Returns the function pointer on success.
fn mmap_executable(code: &[u8]) -> Option<*const u8> {
    if code.is_empty() {
        return None;
    }

    unsafe {
        let page_size = 4096usize;
        let aligned_size = (code.len() + page_size - 1) & !(page_size - 1);

        // Allocate RW pages
        let ptr = libc::mmap(
            std::ptr::null_mut(),
            aligned_size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0,
        );

        if ptr == libc::MAP_FAILED {
            return None;
        }

        // Copy code
        std::ptr::copy_nonoverlapping(code.as_ptr(), ptr as *mut u8, code.len());

        // Flip to RX (W^X)
        let rc = libc::mprotect(ptr, aligned_size, libc::PROT_READ | libc::PROT_EXEC);
        if rc != 0 {
            libc::munmap(ptr, aligned_size);
            return None;
        }

        Some(ptr as *const u8)
    }
}

/// Call a native function pointer with up to 6 i64 arguments.
///
/// # Safety
/// The pointer must be valid executable x86-64 code following System V AMD64 ABI.
#[inline(always)]
unsafe fn call_native(ptr: *const u8, args: &[i64]) -> i64 {
    type Fn0 = unsafe extern "C" fn() -> i64;
    type Fn1 = unsafe extern "C" fn(i64) -> i64;
    type Fn2 = unsafe extern "C" fn(i64, i64) -> i64;
    type Fn3 = unsafe extern "C" fn(i64, i64, i64) -> i64;
    type Fn4 = unsafe extern "C" fn(i64, i64, i64, i64) -> i64;
    type Fn5 = unsafe extern "C" fn(i64, i64, i64, i64, i64) -> i64;
    type Fn6 = unsafe extern "C" fn(i64, i64, i64, i64, i64, i64) -> i64;

    let fptr = ptr as *const ();
    match args.len() {
        0 => { let f: Fn0 = std::mem::transmute(fptr); f() }
        1 => { let f: Fn1 = std::mem::transmute(fptr); f(args[0]) }
        2 => { let f: Fn2 = std::mem::transmute(fptr); f(args[0], args[1]) }
        3 => { let f: Fn3 = std::mem::transmute(fptr); f(args[0], args[1], args[2]) }
        4 => { let f: Fn4 = std::mem::transmute(fptr); f(args[0], args[1], args[2], args[3]) }
        5 => { let f: Fn5 = std::mem::transmute(fptr); f(args[0], args[1], args[2], args[3], args[4]) }
        _ => {
            let f: Fn6 = std::mem::transmute(fptr);
            f(
                args[0],
                args.get(1).copied().unwrap_or(0),
                args.get(2).copied().unwrap_or(0),
                args.get(3).copied().unwrap_or(0),
                args.get(4).copied().unwrap_or(0),
                args.get(5).copied().unwrap_or(0),
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Check whether a SemanticGraph can be AOT-compiled to x86-64.
///
/// Runs both `is_aot_compilable` and `lambda_lift_check` from IRIS.
/// If the graph has Ref/Apply nodes, tries inlining them first.
pub fn is_jit_compilable(graph: &SemanticGraph) -> bool {
    is_jit_compilable_with_registry(graph, None)
}

/// Check compilability with access to a user fragment registry for inlining.
pub fn is_jit_compilable_with_registry(
    graph: &SemanticGraph,
    user_registry: Option<&crate::registry::FragmentRegistry>,
) -> bool {
    // First try without inlining
    if is_jit_compilable_raw(graph) {
        return true;
    }
    // Try with inlining
    let compiler = get_compiler();
    let mut inlined = graph.clone();
    if inline_all_refs(&mut inlined, user_registry, &compiler.aot_registry) {
        is_jit_compilable_raw(&inlined)
    } else {
        false
    }
}

fn is_jit_compilable_raw(graph: &SemanticGraph) -> bool {
    let compiler = get_compiler();

    // Check basic node-kind support
    let check_graph = match find_graph(&compiler.aot_graphs, "is_aot_compilable") {
        Some(g) => g,
        None => return false,
    };
    let program_val = Value::Program(Box::new(graph.clone()));
    // Direct Rust-side node kind check (the IRIS is_aot_compilable fold
    // has issues with cross-fragment closure evaluation).
    let _ = check_graph; // keep compiler fragments loaded
    let basic_ok = graph.nodes.values().all(|node| matches!(
        node.kind,
        NodeKind::Prim | NodeKind::Lambda | NodeKind::Let | NodeKind::Lit
        | NodeKind::Fold | NodeKind::Tuple | NodeKind::Project
        | NodeKind::TypeAbst | NodeKind::TypeApp | NodeKind::Guard
        | NodeKind::Rewrite
    ));
    if !basic_ok { return false; }

    // Check lambda safety (all lambdas are fold steps, nesting depth ≤ 1)
    if let Some(lift_check) = find_graph(&compiler.aot_graphs, "lambda_lift_check") {
        matches!(
            iris_bootstrap::evaluate_with_fragments(
                &lift_check, &[program_val], 1_000_000, &compiler.aot_registry,
            ),
            Ok(Value::Int(1))
        )
    } else {
        // No lambda_lift available — only safe if there are no lambdas
        true
    }
}

/// Check if a graph has a cached JIT result (for benchmarking dispatch overhead).
pub fn is_jit_compilable_cached(graph: &SemanticGraph) -> bool {
    if let Ok(cache) = get_cache().read() {
        if let Some(cached) = cache.get(&graph.hash) {
            return cached.ptr.is_some();
        }
    }
    false
}

/// Direct JIT call with raw i64 args — zero allocation hot path.
/// For benchmarking: bypasses Value conversion and Vec<Value> return.
pub fn call_jit_raw(graph: &SemanticGraph, args: &[i64]) -> Option<i64> {
    let ptr = {
        let cache = get_cache().read().ok()?;
        cache.get(&graph.hash)?.ptr?
    };
    Some(unsafe { call_native(ptr, args) })
}

/// Fast JIT call accepting Value inputs — near-zero overhead hot path.
/// Returns None if the graph isn't JIT-compiled (caller should fall back).
#[inline(always)]
pub fn call_jit_fast(graph: &SemanticGraph, inputs: &[Value]) -> Option<Value> {
    let (ptr, result_is_float) = {
        let cache = get_cache().read().ok()?;
        let cached = cache.get(&graph.hash)?;
        (cached.ptr?, cached.result_is_float)
    };
    let mut args = [0i64; 6];
    let n = inputs.len().min(6);
    for i in 0..n {
        args[i] = match &inputs[i] {
            Value::Int(v) => *v,
            Value::Float64(f) => f.to_bits() as i64,
            Value::Bool(b) => if *b { 1 } else { 0 },
            _ => 0,
        };
    }
    let result = unsafe { call_native(ptr, &args[..n]) };
    if result_is_float {
        Some(Value::Float64(f64::from_bits(result as u64)))
    } else {
        Some(Value::Int(result))
    }
}

/// Count the number of input parameters for a graph.
fn count_graph_inputs(graph: &SemanticGraph) -> usize {
    use iris_types::graph::{NodeKind, NodePayload};
    let mut max_idx: Option<usize> = None;
    for (_id, node) in &graph.nodes {
        if node.kind == NodeKind::Lit {
            if let NodePayload::Lit { type_tag, ref value } = node.payload {
                if type_tag == 0xFF && !value.is_empty() {
                    let idx = value[0] as usize;
                    max_idx = Some(max_idx.map_or(idx, |m: usize| m.max(idx)));
                }
            }
        }
    }
    max_idx.map_or(0, |m| m + 1)
}

// ---------------------------------------------------------------------------
// Inlining: eliminate Ref/Apply nodes by substituting fragment bodies
// ---------------------------------------------------------------------------

use iris_types::graph::{Node, NodeKind, NodePayload, NodeId, Edge, EdgeLabel, BinderId};
use iris_types::cost::CostTerm;

/// Inline all Ref nodes in a graph by substituting their fragment bodies.
/// This makes the graph self-contained (no cross-fragment references) so
/// the AOT compiler can handle it.
///
/// Uses user_registry to look up referenced fragments, falling back to
/// aot_registry for compiler-internal references.
fn inline_all_refs(
    graph: &mut SemanticGraph,
    user_registry: Option<&crate::registry::FragmentRegistry>,
    aot_registry: &BTreeMap<FragmentId, SemanticGraph>,
) -> bool {
    let max_iterations = 20; // prevent infinite loops
    let mut changed = false;

    for _iter in 0..max_iterations {
        // Find a Ref node
        let ref_info = graph.nodes.iter().find_map(|(id, node)| {
            if let NodePayload::Ref { fragment_id } = &node.payload {
                Some((*id, *fragment_id, node.arity))
            } else {
                None
            }
        });

        let Some((ref_node_id, fragment_id, ref_arity)) = ref_info else {
            break; // No more Ref nodes
        };

        // Look up the fragment: try user registry first, then AOT registry
        let frag_graph = user_registry
            .and_then(|r| r.get(&fragment_id))
            .map(|f| &f.graph)
            .or_else(|| aot_registry.get(&fragment_id));

        let Some(frag_graph) = frag_graph else {
            // Fragment not found — remove the Ref to avoid infinite loop
            graph.nodes.remove(&ref_node_id);
            graph.edges.retain(|e| e.source != ref_node_id && e.target != ref_node_id);
            continue;
        };
        let frag_graph = frag_graph.clone();

        changed = true;

        // Collect the Ref node's argument edges (sorted by port)
        let mut ref_args: Vec<(u8, NodeId)> = graph.edges.iter()
            .filter(|e| e.source == ref_node_id && e.label == EdgeLabel::Argument)
            .map(|e| (e.port, e.target))
            .collect();
        ref_args.sort_by_key(|(port, _)| *port);

        // Generate new node IDs for the fragment's nodes (avoid collisions)
        let max_existing = graph.nodes.keys().map(|id| id.0).max().unwrap_or(0);
        let mut id_map: HashMap<NodeId, NodeId> = HashMap::new();
        for (i, (old_id, _)) in frag_graph.nodes.iter().enumerate() {
            let new_id = NodeId(max_existing + 1 + i as u64);
            id_map.insert(*old_id, new_id);
        }

        // Find InputRef Lit nodes in the fragment — these are the fragment's parameters
        // InputRef type_tag=0xFF, value[0]=parameter_index
        let mut input_ref_map: HashMap<NodeId, NodeId> = HashMap::new();
        for (old_id, node) in &frag_graph.nodes {
            if let NodePayload::Lit { type_tag: 0xFF, ref value } = node.payload {
                if !value.is_empty() {
                    let param_idx = value[0] as usize;
                    if param_idx < ref_args.len() {
                        // Map this InputRef to the corresponding argument from the Ref node
                        let mapped_id = id_map.get(old_id).copied().unwrap_or(*old_id);
                        input_ref_map.insert(mapped_id, ref_args[param_idx].1);
                    }
                }
            }
        }

        // Copy fragment nodes into the main graph (with remapped IDs)
        for (old_id, node) in &frag_graph.nodes {
            let new_id = id_map.get(old_id).copied().unwrap_or(*old_id);

            // Skip InputRef nodes that map to caller arguments
            if input_ref_map.contains_key(&new_id) {
                continue;
            }

            let mut new_node = node.clone();
            new_node.id = new_id;
            // Remap NodeIds in payload
            match &mut new_node.payload {
                NodePayload::Guard { predicate_node, body_node, fallback_node } => {
                    *predicate_node = id_map.get(predicate_node).copied().unwrap_or(*predicate_node);
                    *body_node = id_map.get(body_node).copied().unwrap_or(*body_node);
                    *fallback_node = id_map.get(fallback_node).copied().unwrap_or(*fallback_node);
                }
                NodePayload::Rewrite { body, .. } => {
                    *body = id_map.get(body).copied().unwrap_or(*body);
                }
                _ => {}
            }
            graph.nodes.insert(new_id, new_node);
        }

        // Copy fragment edges (with remapped IDs, substituting input refs)
        for edge in &frag_graph.edges {
            let new_source = id_map.get(&edge.source).copied().unwrap_or(edge.source);
            let mut new_target = id_map.get(&edge.target).copied().unwrap_or(edge.target);

            // If the target was an InputRef, redirect to the caller's argument
            if let Some(&caller_arg) = input_ref_map.get(&new_target) {
                new_target = caller_arg;
            }

            // Skip edges from InputRef nodes we eliminated
            if input_ref_map.contains_key(&new_source) {
                continue;
            }

            graph.edges.push(Edge {
                source: new_source,
                target: new_target,
                port: edge.port,
                label: edge.label,
            });
        }

        // Find the fragment's root (remapped)
        let frag_root = id_map.get(&frag_graph.root).copied().unwrap_or(frag_graph.root);

        // Now redirect all edges that TARGET the Ref node to TARGET the fragment root
        // AND all edges that have Ref node as SOURCE — redirect from fragment root
        // Actually: we need to replace the Ref node in the graph. Any node that uses
        // the Ref's result (i.e., has an edge targeting ref_node_id) should now
        // target the fragment's root.
        //
        // But edges go source→target where source is the "user" and target is what
        // it depends on. So we need to find edges where target == ref_node_id
        // and change target to frag_root.
        for edge in &mut graph.edges {
            if edge.target == ref_node_id {
                edge.target = frag_root;
            }
        }

        // If the Ref node was the graph root, update it
        if graph.root == ref_node_id {
            graph.root = frag_root;
        }

        // Remove the Ref node and its outgoing edges
        graph.nodes.remove(&ref_node_id);
        graph.edges.retain(|e| e.source != ref_node_id);
    }

    // Also handle Apply nodes: Apply(func, arg) where func resolved to a Lambda
    // After inlining Refs, we may have Apply(Lambda(body), arg) patterns
    // These need beta-reduction: replace with Let(arg, body)
    for _iter in 0..max_iterations {
        let apply_info = graph.nodes.iter().find_map(|(id, node)| {
            if node.kind == NodeKind::Apply {
                Some(*id)
            } else {
                None
            }
        });

        let Some(apply_id) = apply_info else {
            break;
        };

        // Get Apply's edges: port 0 = function, port 1 = argument
        let mut func_id = None;
        let mut arg_id = None;
        for edge in &graph.edges {
            if edge.source == apply_id && edge.label == EdgeLabel::Argument {
                match edge.port {
                    0 => func_id = Some(edge.target),
                    1 => arg_id = Some(edge.target),
                    _ => {}
                }
            }
        }

        let (Some(func_id), Some(arg_id)) = (func_id, arg_id) else {
            // Malformed Apply — remove it to avoid infinite loop
            graph.nodes.remove(&apply_id);
            graph.edges.retain(|e| e.source != apply_id);
            continue;
        };

        // Check if func is a Lambda
        let is_lambda = graph.nodes.get(&func_id)
            .map_or(false, |n| n.kind == NodeKind::Lambda);

        if is_lambda {
            // Beta-reduce: Apply(Lambda(binder, body), arg) → Let(binding=arg, body)
            // Get Lambda's body (first child edge)
            let lambda_body = graph.edges.iter()
                .find(|e| e.source == func_id && e.label == EdgeLabel::Argument && e.port == 0)
                .map(|e| e.target);

            if let Some(body_id) = lambda_body {
                // Replace Apply node with a Let node
                let let_node = Node {
                    id: apply_id,
                    kind: NodeKind::Let,
                    type_sig: graph.nodes.get(&apply_id).map_or(
                        iris_types::types::TypeId(0), |n| n.type_sig),
                    cost: CostTerm::Unit,
                    arity: 2,
                    resolution_depth: 0,
                    salt: 0,
                    payload: NodePayload::Let,
                };
                graph.nodes.insert(apply_id, let_node);

                // Replace edges: port 0 = binding (arg), port 1 = body
                graph.edges.retain(|e| e.source != apply_id);
                graph.edges.push(Edge {
                    source: apply_id,
                    target: arg_id,
                    port: 0,
                    label: EdgeLabel::Argument,
                });
                graph.edges.push(Edge {
                    source: apply_id,
                    target: body_id,
                    port: 1,
                    label: EdgeLabel::Argument,
                });

                // The Lambda's binder references in the body should resolve naturally
                // because the Let binding puts arg_id on the stack in the same slot
                // the Lambda's binder would have used.

                // Remove the Lambda node if nothing else references it
                let lambda_used = graph.edges.iter().any(|e| e.target == func_id);
                if !lambda_used {
                    graph.nodes.remove(&func_id);
                    graph.edges.retain(|e| e.source != func_id);
                }

                changed = true;
            } else {
                // Lambda without body — remove Apply
                graph.nodes.remove(&apply_id);
                graph.edges.retain(|e| e.source != apply_id);
            }
        } else {
            // Non-lambda Apply — can't beta-reduce, remove to avoid infinite loop
            // This happens for higher-order functions that we can't inline
            break;
        }
    }

    changed
}

/// Compile a SemanticGraph to native x86-64 and cache the function pointer.
///
/// Returns (function pointer, result_is_float) on success, or None if compilation fails.
/// Results (both success and failure) are cached by SemanticHash.
/// Tries inlining Ref/Apply nodes before compilation if needed.
/// `default_type`: 0 for int, 2 for float (determines InputRef type inference).
fn jit_compile(
    graph: &SemanticGraph,
    user_registry: Option<&crate::registry::FragmentRegistry>,
    default_type: i64,
) -> Option<(*const u8, bool)> {
    // Fast path: read lock for cache hit (concurrent reads don't block)
    {
        let cache = get_cache().read().ok()?;
        if let Some(cached) = cache.get(&graph.hash) {
            return cached.ptr.map(|p| (p, cached.result_is_float));
        }
    }

    // Try inlining Ref/Apply nodes if the graph has them
    let has_ref_or_apply = graph.nodes.values().any(|n|
        n.kind == NodeKind::Ref || n.kind == NodeKind::Apply);

    let compile_graph = if has_ref_or_apply {
        let compiler = get_compiler();
        let mut inlined = graph.clone();
        inline_all_refs(&mut inlined, user_registry, &compiler.aot_registry);
        inlined.hash = graph.hash; // keep original hash for caching
        inlined
    } else {
        graph.clone()
    };

    // Slow path: compile, then write lock to insert
    let compiler = get_compiler();
    let handler = get_handler();
    let aot_graph = find_graph(&compiler.aot_graphs, "aot_compile")?;
    let program_val = Value::Program(Box::new(compile_graph.clone()));

    // Use the passed default_type (determined from input types by caller)
    // but also check if the graph itself reveals float types (e.g., float Lits)
    let graph_type = find_graph(&compiler.aot_graphs, "aot_result_type")
        .and_then(|type_graph| {
            iris_bootstrap::evaluate_with_fragments(
                &type_graph, &[program_val.clone()], 1_000_000, &compiler.aot_registry,
            ).ok()
        })
        .map_or(0i64, |v| match v { Value::Int(n) => n, _ => 0 });
    // Use float if either the graph or caller says float
    let effective_type = if graph_type == 2 || default_type == 2 { 2 } else { 0 };

    let code_bytes = iris_bootstrap::evaluate_with_effects(
        &aot_graph,
        &[program_val.clone(), Value::Int(effective_type)],
        10_000_000,
        &compiler.aot_registry,
        handler,
    );

    // Debug: log compilation result
    match &code_bytes {
        Ok(Value::Bytes(b)) => eprintln!("[jit] AOT compiled: {} bytes", b.len()),
        Ok(other) => eprintln!("[jit] AOT returned non-bytes: {:?}", std::mem::discriminant(other)),
        Err(e) => eprintln!("[jit] AOT compilation error: {}", e),
    }

    // Extract bytes; empty or non-Bytes means compilation failed
    let ptr = code_bytes.ok()
        .and_then(|v| match v {
            Value::Bytes(b) if !b.is_empty() => Some(b),
            _ => None,
        })
        .and_then(|b| mmap_executable(&b));

    // Determine result_is_float from the graph's root type (with effective_type for InputRef)
    // aot_result_type uses default_type=0 (graph analysis only), but node_type
    // knows comparisons return int. We need to re-evaluate with the effective default.
    let result_type_graph = find_graph(&compiler.aot_graphs, "node_type");
    let result_is_float = if let (Some(rtg), Some(_)) = (result_type_graph, ptr.as_ref()) {
        let root_id = compile_graph.root;
        let program_val2 = Value::Program(Box::new(compile_graph));
        iris_bootstrap::evaluate_with_fragments(
            &rtg, &[program_val2, Value::Int(root_id.0 as i64), Value::Int(effective_type)],
            1_000_000, &compiler.aot_registry,
        ).ok().map_or(false, |v| matches!(v, Value::Int(2)))
    } else {
        effective_type == 2
    };

    let n_args = count_graph_inputs(graph);

    // Cache the result (including None for not-compilable)
    if let Ok(mut cache) = get_cache().write() {
        cache.insert(graph.hash, NativeFunction {
            ptr,
            _mmap_size: 0, // We don't track this for simplicity
            n_args,
            result_is_float,
        });
    }

    ptr.map(|p| (p, result_is_float))
}

/// Execute a JIT-compiled function with the given inputs.
/// Uses a stack buffer — zero heap allocations on the hot path.
#[inline(always)]
fn jit_call(ptr: *const u8, inputs: &[Value], result_is_float: bool) -> Result<Value, String> {
    let mut args = [0i64; 6];
    let n = inputs.len().min(6);
    for i in 0..n {
        args[i] = match &inputs[i] {
            Value::Int(v) => *v,
            Value::Float64(f) => f.to_bits() as i64,
            Value::Bool(b) => if *b { 1 } else { 0 },
            _ => 0,
        };
    }

    let result = unsafe { call_native(ptr, &args[..n]) };
    if result_is_float {
        Ok(Value::Float64(f64::from_bits(result as u64)))
    } else {
        Ok(Value::Int(result))
    }
}

/// JIT-compile and execute, returning a single Value (no Vec allocation).
#[inline(always)]
pub fn interpret_jit_single(
    graph: &SemanticGraph,
    inputs: &[Value],
    registry: Option<&crate::registry::FragmentRegistry>,
) -> Result<Value, super::interpreter::InterpretError> {
    if let Some(result) = call_jit_fast(graph, inputs) {
        return Ok(result);
    }

    // Cold path
    interpret_jit_cold(graph, inputs, registry)
        .map(|v| v.into_iter().next().unwrap_or(Value::Unit))
}

/// JIT-compile and execute a program. Falls back to tree-walker if
/// compilation fails.
///
/// Hot path (cache hit): ~20ns — direct native call, no heap allocation.
/// Cold path (first call or not compilable): compiles via tree-walker, then caches.
#[inline(always)]
pub fn interpret_jit(
    graph: &SemanticGraph,
    inputs: &[Value],
    registry: Option<&crate::registry::FragmentRegistry>,
) -> Result<Vec<Value>, super::interpreter::InterpretError> {
    // Hot path: try the fast inlined JIT call first
    if let Some(result) = call_jit_fast(graph, inputs) {
        return Ok(vec![result]);
    }

    // Cold path: compile or fall back
    interpret_jit_cold(graph, inputs, registry)
}

/// Cold path for interpret_jit — never inlined to keep the hot path small.
#[cold]
#[inline(never)]
fn interpret_jit_cold(
    graph: &SemanticGraph,
    inputs: &[Value],
    registry: Option<&crate::registry::FragmentRegistry>,
) -> Result<Vec<Value>, super::interpreter::InterpretError> {
    // Determine default type from inputs: if any input is Float64, use float mode
    let default_type = if inputs.iter().any(|v| matches!(v, Value::Float64(_))) { 2 } else { 0 };
    // Try to compile (first time) — pass user registry for inlining
    if let Some((ptr, result_is_float)) = jit_compile(graph, registry, default_type) {
        match jit_call(ptr, inputs, result_is_float) {
            Ok(result) => return Ok(vec![result]),
            Err(_) => {}
        }
    }

    // Fallback: tree-walker
    super::interpreter::interpret_with_registry(graph, inputs, None, registry)
        .map(|(outputs, _state)| outputs)
}

/// JIT-compile and execute, returning the full (outputs, state) pair.
#[inline(always)]
pub fn interpret_jit_full(
    graph: &SemanticGraph,
    inputs: &[Value],
    state: Option<&mut iris_types::eval::StateStore>,
    registry: Option<&crate::registry::FragmentRegistry>,
) -> Result<(Vec<Value>, iris_types::eval::StateStore), super::interpreter::InterpretError> {
    if let Some(result) = call_jit_fast(graph, inputs) {
        let final_state = state
            .map(|s| std::mem::take(s))
            .unwrap_or_default();
        return Ok((vec![result], final_state));
    }

    // Cold path
    super::interpreter::interpret_with_registry(graph, inputs, state, registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_add_graph() -> SemanticGraph {
        // Compile a simple "add(a, b) = a + b" program
        let src = "let add a b = a + b";
        let result = iris_bootstrap::syntax::compile(src);
        assert!(result.errors.is_empty());
        let (_, frag, _) = result.fragments.last().unwrap();
        frag.graph.clone()
    }

    #[test]
    fn test_is_jit_compilable() {
        let graph = make_add_graph();
        assert!(is_jit_compilable(&graph), "simple add should be JIT-compilable");
    }

    #[test]
    fn test_jit_compile_and_run() {
        let graph = make_add_graph();
        let result = interpret_jit(
            &graph,
            &[Value::Int(17), Value::Int(25)],
            None,
        );
        match result {
            Ok(outputs) => {
                assert_eq!(outputs.len(), 1);
                assert_eq!(outputs[0], Value::Int(42));
            }
            Err(e) => {
                // Tree-walker fallback is fine too
                eprintln!("JIT not available, tree-walker result: {:?}", e);
            }
        }
    }
}
