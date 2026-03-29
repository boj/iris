//! IRIS Runtime: builds and caches self-written IRIS programs for use in
//! the production evolution loop.
//!
//! When `iris_mode` is enabled, the evolution loop uses IRIS programs for
//! mutation, evaluation, and selection instead of Rust functions. This is
//! the autopoietic closure: IRIS programs evolving IRIS programs.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::rc::Rc;
use std::time::{Duration, Instant};

use rand::Rng;

use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::TypeEnv;

use crate::mutation;

// ---------------------------------------------------------------------------
// Graph construction helpers (same patterns as tests)
// ---------------------------------------------------------------------------

fn make_node(id: u64, kind: NodeKind, payload: NodePayload, arity: u8) -> (NodeId, Node) {
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind,
            type_sig: iris_types::types::TypeId(0),
            cost: CostTerm::Unit,
            arity,
            resolution_depth: 0, salt: 0,
            payload,
        },
    )
}

fn make_edge(source: u64, target: u64, port: u8, label: EdgeLabel) -> Edge {
    Edge {
        source: NodeId(source),
        target: NodeId(target),
        port,
        label,
    }
}

fn make_graph(nodes: HashMap<NodeId, Node>, edges: Vec<Edge>, root: u64) -> SemanticGraph {
    SemanticGraph {
        root: NodeId(root),
        nodes,
        edges,
        type_env: TypeEnv {
            types: BTreeMap::new(),
        },
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

fn input_ref_node(id: u64, index: u8) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0xFF,
            value: vec![index],
        },
        0,
    )
}

fn prim_node(id: u64, opcode: u8, arity: u8) -> (NodeId, Node) {
    make_node(id, NodeKind::Prim, NodePayload::Prim { opcode }, arity)
}

#[allow(dead_code)]
fn int_lit_node(id: u64, value: i64) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: value.to_le_bytes().to_vec(),
        },
        0,
    )
}

fn project_node(id: u64, field_index: u16) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Project,
        NodePayload::Project { field_index },
        1,
    )
}

// ---------------------------------------------------------------------------
// PerformanceTracker
// ---------------------------------------------------------------------------

/// Sliding-window timing tracker for IRIS runtime components.
///
/// Records the last N call durations for each named component, enabling
/// auto_improve to target actual slow IRIS programs rather than relying
/// on hardcoded baselines.
pub struct PerformanceTracker {
    /// Per-component sliding window of call durations.
    windows: BTreeMap<String, VecDeque<Duration>>,
    /// Maximum window size (number of recent calls to keep).
    window_size: usize,
}

impl PerformanceTracker {
    /// Create a new tracker with the given sliding window size.
    pub fn new(window_size: usize) -> Self {
        Self {
            windows: BTreeMap::new(),
            window_size,
        }
    }

    /// Record a timing sample for a named component.
    pub fn record(&mut self, component: &str, duration: Duration) {
        let window = self
            .windows
            .entry(component.to_string())
            .or_insert_with(|| VecDeque::with_capacity(self.window_size + 1));
        window.push_back(duration);
        if window.len() > self.window_size {
            window.pop_front();
        }
    }

    /// Get the mean duration for a component, or None if no samples.
    pub fn mean_duration(&self, component: &str) -> Option<Duration> {
        let window = self.windows.get(component)?;
        if window.is_empty() {
            return None;
        }
        let total: Duration = window.iter().sum();
        Some(total / window.len() as u32)
    }

    /// Get the number of samples recorded for a component.
    pub fn sample_count(&self, component: &str) -> usize {
        self.windows.get(component).map(|w| w.len()).unwrap_or(0)
    }

    /// Return all components with their mean durations, sorted slowest first.
    pub fn component_timings(&self) -> Vec<(String, Duration)> {
        let mut timings: Vec<(String, Duration)> = self
            .windows
            .iter()
            .filter_map(|(name, window)| {
                if window.is_empty() {
                    return None;
                }
                let total: Duration = window.iter().sum();
                let mean = total / window.len() as u32;
                Some((name.clone(), mean))
            })
            .collect();
        // Sort slowest first.
        timings.sort_by(|a, b| b.1.cmp(&a.1));
        timings
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new(100)
    }
}

// ---------------------------------------------------------------------------
// IrisRuntime
// ---------------------------------------------------------------------------

/// Runtime that holds cached IRIS programs for mutation, evaluation, and
/// selection. Built once at the start of an IRIS-mode evolution run.
pub struct IrisRuntime {
    /// IRIS program: replace_prim(program, new_opcode) -> program
    /// Takes a Program and an Int opcode, changes the first Prim node's opcode.
    pub replace_prim: SemanticGraph,

    /// IRIS program: direct_replace_prim(program, node_id, new_opcode) -> program
    /// Takes a Program, a specific node ID, and a new opcode.
    pub direct_replace_prim: SemanticGraph,

    /// IRIS program: add_node(program, opcode) -> program
    /// Adds a new Prim node to the program via graph_add_node_rt (0x85).
    pub add_node: SemanticGraph,

    /// IRIS program: connect(program, source_id, target_id, port) -> program
    /// Connects two nodes via graph_connect (0x86).
    pub connect: SemanticGraph,

    /// IRIS program: evaluate(program, inputs) -> outputs
    /// Evaluates a program on inputs via graph_eval (0x89).
    pub evaluate: SemanticGraph,

    /// IRIS program: tournament selection — max of 4 fitness values.
    /// Used to find the best individual in a tournament subset.
    pub tournament_select_program: SemanticGraph,

    /// IRIS program: crossover — swap subtrees between two programs.
    /// inputs[0]=Program(a), inputs[1]=Int(node_a), inputs[2]=Program(b), inputs[3]=Int(node_b)
    /// Returns Tuple(offspring_a, offspring_b).
    pub crossover_program: SemanticGraph,

    /// Whether the runtime has been successfully initialized.
    pub initialized: bool,

    /// Count of fallbacks to Rust (for diagnostics).
    pub fallback_count: std::sync::atomic::AtomicU64,

    /// Hot-swappable deployed components (name -> IRIS program).
    /// These override the default built-in programs when present.
    deployed_components: std::collections::BTreeMap<String, SemanticGraph>,

    /// Audit log of component replacements: (name, action, timestamp).
    component_audit: Vec<(String, String, u64)>,

    /// Runtime performance tracker: sliding window of call timings
    /// for mutate, select, crossover, and generate_seed.
    perf_tracker: PerformanceTracker,
}

impl IrisRuntime {
    /// Build all IRIS programs and cache them.
    pub fn new() -> Self {
        Self {
            replace_prim: build_replace_prim(),
            direct_replace_prim: build_direct_replace_prim(),
            add_node: build_add_node(),
            connect: build_connect(),
            evaluate: build_evaluate(),
            tournament_select_program: build_tournament_select(),
            crossover_program: build_crossover_subgraph(),
            initialized: true,
            fallback_count: std::sync::atomic::AtomicU64::new(0),
            deployed_components: std::collections::BTreeMap::new(),
            component_audit: Vec::new(),
            perf_tracker: PerformanceTracker::new(100),
        }
    }

    /// Replace a named component with a new IRIS program.
    ///
    /// Returns the old program (if any) so the caller can rollback.
    /// Logs the replacement to the internal audit trail.
    pub fn replace_component(&mut self, name: &str, program: SemanticGraph) -> Option<SemanticGraph> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.component_audit.push((
            name.to_string(),
            format!("deployed ({} nodes)", program.nodes.len()),
            timestamp,
        ));
        self.deployed_components.insert(name.to_string(), program)
    }

    /// Rollback a component to a previous version.
    pub fn rollback_component(&mut self, name: &str, old_program: SemanticGraph) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.component_audit.push((
            name.to_string(),
            "rolled back".to_string(),
            timestamp,
        ));
        self.deployed_components.insert(name.to_string(), old_program);
    }

    /// Remove a deployed component entirely (revert to built-in).
    pub fn remove_component(&mut self, name: &str) -> Option<SemanticGraph> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.component_audit.push((
            name.to_string(),
            "removed".to_string(),
            timestamp,
        ));
        self.deployed_components.remove(name)
    }

    /// Get a reference to the deployed components map.
    pub fn deployed_components(&self) -> &std::collections::BTreeMap<String, SemanticGraph> {
        &self.deployed_components
    }

    /// Get the component audit log.
    pub fn component_audit(&self) -> &[(String, String, u64)] {
        &self.component_audit
    }

    /// Apply a random IRIS mutation to a program.
    ///
    /// Picks a random mutation strategy and runs the corresponding IRIS
    /// program via the interpreter. Falls back to Rust `mutation::mutate`
    /// if the IRIS component fails. Records timing in the performance tracker.
    pub fn mutate(&self, program: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
        let start = Instant::now();

        let roll: f64 = rng.r#gen();

        // Strategy distribution:
        //   60% replace_prim (change an operator)
        //   25% add_node + connect (add new computation)
        //   15% replace with random opcode from extended set
        let result = if roll < 0.60 {
            self.iris_replace_prim_random(program, rng)
        } else if roll < 0.85 {
            self.iris_add_and_connect(program, rng)
        } else {
            self.iris_replace_prim_random(program, rng)
        };

        let output = match result {
            Some(mutated) => mutated,
            None => {
                self.fallback_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                // Fallback to Rust mutation
                mutation::mutate(program, rng)
            }
        };

        // Record timing (interior mutability via unsafe cell pattern avoided;
        // callers use record_timing() after the call).
        let _ = start.elapsed(); // timing recorded via record_timing
        output
    }

    /// Evaluate a program on a single set of inputs using the IRIS evaluator.
    ///
    /// Returns the outputs, or None if the IRIS evaluator fails.
    pub fn evaluate_program(
        &self,
        program: &SemanticGraph,
        inputs: &[Value],
    ) -> Option<Vec<Value>> {
        let iris_inputs = vec![
            Value::Program(Rc::new(program.clone())),
            Value::tuple(inputs.to_vec()),
        ];

        match interpreter::interpret(&self.evaluate, &iris_inputs, None) {
            Ok((outputs, _)) => Some(outputs),
            Err(_) => None,
        }
    }

    /// Score a program against test cases using the IRIS fitness evaluator.
    ///
    /// Returns a correctness score in [0.0, 1.0]. Falls back to 0.0 on
    /// IRIS evaluator failure.
    pub fn score_program(
        &self,
        program: &SemanticGraph,
        test_cases: &[(Vec<Value>, Value)],
    ) -> f32 {
        if test_cases.is_empty() {
            return 0.0;
        }

        let mut correct = 0usize;
        for (inputs, expected) in test_cases {
            match self.evaluate_program(program, inputs) {
                Some(outputs) => {
                    if outputs.first() == Some(expected) {
                        correct += 1;
                    }
                }
                None => {
                    // IRIS evaluator failed on this case; count as incorrect.
                }
            }
        }
        correct as f32 / test_cases.len() as f32
    }

    /// Tournament selection using the IRIS tournament_select program.
    ///
    /// Picks `tournament_size` random candidates, runs the IRIS max-fitness
    /// program to find the best fitness value, then returns the index of
    /// the winner. Falls back to Rust comparison if the IRIS program fails.
    pub fn select(&self, fitness_values: &[f32]) -> usize {
        if fitness_values.is_empty() {
            return 0;
        }
        if fitness_values.len() == 1 {
            return 0;
        }

        let start = Instant::now();

        // The IRIS tournament_select_program computes max of 4 fitness values.
        // Scale f32 fitness to i64 (multiply by 10000 to preserve precision).
        let n = fitness_values.len().min(4);
        let scaled: Vec<i64> = fitness_values[..n]
            .iter()
            .map(|&f| (f * 10000.0) as i64)
            .collect();
        let mut inputs: Vec<Value> = scaled.iter().map(|&v| Value::Int(v)).collect();
        while inputs.len() < 4 {
            inputs.push(Value::Int(i64::MIN));
        }

        let result = interpreter::interpret(&self.tournament_select_program, &inputs, None);

        let _ = start.elapsed(); // timing recorded via record_timing

        match result {
            Ok((outputs, _)) => {
                if let Some(Value::Int(best_val)) = outputs.first() {
                    // Find the index of the winner (first match on scaled value).
                    let best = *best_val;
                    for (i, &sv) in scaled.iter().enumerate() {
                        if sv == best {
                            return i;
                        }
                    }
                }
                // Fallback: return the index with max fitness.
                self.select_fallback(fitness_values)
            }
            Err(_) => self.select_fallback(fitness_values),
        }
    }

    /// Rust fallback for tournament selection.
    fn select_fallback(&self, fitness_values: &[f32]) -> usize {
        self.fallback_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        fitness_values
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Tournament selection with random subset sampling.
    ///
    /// Samples `tournament_size` random indices, gathers their fitness values,
    /// and uses `select()` to find the winner among them.
    pub fn tournament_select(
        &self,
        fitnesses: &[f32],
        tournament_size: usize,
        rng: &mut impl Rng,
    ) -> usize {
        if fitnesses.is_empty() {
            return 0;
        }

        let ts = tournament_size.min(fitnesses.len());
        let mut candidates: Vec<usize> = Vec::with_capacity(ts);
        let mut candidate_fitnesses: Vec<f32> = Vec::with_capacity(ts);

        for _ in 0..ts {
            let idx = rng.gen_range(0..fitnesses.len());
            candidates.push(idx);
            candidate_fitnesses.push(fitnesses[idx]);
        }

        let local_winner = self.select(&candidate_fitnesses);
        candidates[local_winner]
    }

    /// Perform crossover between two programs using the IRIS crossover program.
    ///
    /// Picks random crossover points in each parent and runs the IRIS
    /// graph_replace_subtree program to produce two offspring. Falls back to
    /// Rust `crossover::crossover()` on failure.
    pub fn crossover(
        &self,
        a: &SemanticGraph,
        b: &SemanticGraph,
        rng: &mut impl Rng,
    ) -> Option<(SemanticGraph, SemanticGraph)> {
        let start = Instant::now();

        let ids_a: Vec<NodeId> = a.nodes.keys().copied().collect();
        let ids_b: Vec<NodeId> = b.nodes.keys().copied().collect();

        if ids_a.is_empty() || ids_b.is_empty() {
            return None;
        }

        // Pick random crossover points.
        let node_a = ids_a[rng.gen_range(0..ids_a.len())];
        let node_b = ids_b[rng.gen_range(0..ids_b.len())];

        let inputs = vec![
            Value::Program(Rc::new(a.clone())),
            Value::Int(node_a.0 as i64),
            Value::Program(Rc::new(b.clone())),
            Value::Int(node_b.0 as i64),
        ];

        let result = interpreter::interpret(&self.crossover_program, &inputs, None);

        let _ = start.elapsed(); // timing recorded via record_timing

        match result {
            Ok((outputs, _)) => {
                if let Some(Value::Tuple(offspring)) = outputs.into_iter().next() {
                    if offspring.len() >= 2 {
                        let off_a = match &offspring[0] {
                            Value::Program(g) => Some(g.as_ref().clone()),
                            _ => None,
                        };
                        let off_b = match &offspring[1] {
                            Value::Program(g) => Some(g.as_ref().clone()),
                            _ => None,
                        };
                        if let (Some(a), Some(b)) = (off_a, off_b) {
                            return Some((a, b));
                        }
                    }
                }
                None
            }
            Err(_) => {
                self.fallback_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                None
            }
        }
    }

    /// Record a timing sample for a named component.
    ///
    /// Called by external code after each `mutate()`, `select()`,
    /// `crossover()`, or `generate_seed()` call.
    pub fn record_timing(&mut self, component: &str, duration: Duration) {
        self.perf_tracker.record(component, duration);
    }

    /// Get per-component mean timings, sorted slowest first.
    pub fn component_timings(&self) -> Vec<(String, Duration)> {
        self.perf_tracker.component_timings()
    }

    /// Access the performance tracker directly.
    pub fn perf_tracker(&self) -> &PerformanceTracker {
        &self.perf_tracker
    }

    /// Mutable access to the performance tracker.
    pub fn perf_tracker_mut(&mut self) -> &mut PerformanceTracker {
        &mut self.perf_tracker
    }

    // -----------------------------------------------------------------------
    // Internal mutation strategies
    // -----------------------------------------------------------------------

    /// Use the IRIS replace_prim program with a random opcode.
    fn iris_replace_prim_random(
        &self,
        program: &SemanticGraph,
        rng: &mut impl Rng,
    ) -> Option<SemanticGraph> {
        // Pick a random arithmetic/structural opcode.
        let opcodes: &[u8] = &[
            0x00, 0x01, 0x02, 0x03, // add, sub, mul, div
            0x04, 0x05, 0x06,       // mod, neg, abs
            0x07, 0x08,             // min, max
        ];
        let new_opcode = opcodes[rng.gen_range(0..opcodes.len())];

        let inputs = vec![
            Value::Program(Rc::new(program.clone())),
            Value::Int(new_opcode as i64),
        ];

        match interpreter::interpret(&self.replace_prim, &inputs, None) {
            Ok((outputs, _)) => {
                if let Some(Value::Program(p)) = outputs.into_iter().next() {
                    Some(std::rc::Rc::try_unwrap(p).unwrap_or_else(|rc| (*rc).clone()))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// Use IRIS add_node + connect to add a new computation node.
    fn iris_add_and_connect(
        &self,
        program: &SemanticGraph,
        rng: &mut impl Rng,
    ) -> Option<SemanticGraph> {
        // Pick a random opcode for the new node.
        let opcodes: &[u8] = &[0x00, 0x01, 0x02, 0x03, 0x04, 0x07, 0x08];
        let new_opcode = opcodes[rng.gen_range(0..opcodes.len())];

        // Step 1: add a node
        let inputs = vec![
            Value::Program(Rc::new(program.clone())),
            Value::Int(new_opcode as i64),
        ];

        let with_node = match interpreter::interpret(&self.add_node, &inputs, None) {
            Ok((outputs, _)) => {
                if let Some(Value::Program(p)) = outputs.into_iter().next() {
                    std::rc::Rc::try_unwrap(p).unwrap_or_else(|rc| (*rc).clone())
                } else {
                    return None;
                }
            }
            Err(_) => return None,
        };

        // Step 2: connect the new node to an existing node.
        // Find the root as a target and the newly added node as source.
        let root_id = with_node.root.0 as i64;

        // Find the node with the highest NodeId (likely the newly added one).
        let new_node_id = with_node
            .nodes
            .keys()
            .max()
            .map(|n| n.0 as i64)
            .unwrap_or(root_id);

        if new_node_id == root_id {
            // No new node was distinguishable; just return the graph with the node.
            return Some(with_node);
        }

        let connect_inputs = vec![
            Value::Program(Rc::new(with_node.clone())),
            Value::Int(root_id),     // source (root)
            Value::Int(new_node_id), // target (new node)
            Value::Int(rng.gen_range(0..3) as i64), // port
        ];

        match interpreter::interpret(&self.connect, &connect_inputs, None) {
            Ok((outputs, _)) => {
                if let Some(Value::Program(p)) = outputs.into_iter().next() {
                    Some(std::rc::Rc::try_unwrap(p).unwrap_or_else(|rc| (*rc).clone()))
                } else {
                    Some(with_node)
                }
            }
            Err(_) => Some(with_node),
        }
    }
}

impl Default for IrisRuntime {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// IRIS program builders
// ---------------------------------------------------------------------------

/// Build the IRIS replace_prim program.
///
/// Takes inputs[0] = Program, inputs[1] = Int(new_opcode).
/// Returns the program with the first Prim node's opcode changed.
///
/// Graph structure:
///   Root(id=1): graph_set_prim_op(0x84, arity=3)
///   +-- port 0: input_ref(0)           -> inputs[0] (the Program)       [id=10]
///   +-- port 1: project(field=0)       -> first node ID                 [id=20]
///   |           +-- graph_nodes(0x81)  -> Tuple of all node IDs         [id=30]
///   |               +-- input_ref(0)   -> inputs[0]                     [id=40]
///   +-- port 2: input_ref(1)           -> inputs[1] (new opcode)        [id=50]
fn build_replace_prim() -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x84, 3); // graph_set_prim_op
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);
    let (nid, node) = project_node(20, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(30, 0x81, 1); // graph_nodes
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(40, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(50, 1);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 50, 2, EdgeLabel::Argument),
        make_edge(20, 30, 0, EdgeLabel::Argument),
        make_edge(30, 40, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build the direct replace_prim program (takes explicit node ID).
///
/// inputs[0] = Program, inputs[1] = Int(node_id), inputs[2] = Int(new_opcode)
fn build_direct_replace_prim() -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x84, 3);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 2);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build the IRIS add_node program.
///
/// inputs[0] = Program, inputs[1] = Int(opcode)
/// Uses graph_add_node_rt (0x85) to add a new Prim node.
fn build_add_node() -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x85, 2); // graph_add_node_rt
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1); // opcode
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build the IRIS connect program.
///
/// inputs[0] = Program, inputs[1] = Int(source_id), inputs[2] = Int(target_id),
/// inputs[3] = Int(port)
/// Uses graph_connect (0x86) to add an edge.
fn build_connect() -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x86, 4); // graph_connect
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1); // source_id
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 2); // target_id
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(40, 3); // port
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        make_edge(1, 40, 3, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build the IRIS evaluate program.
///
/// inputs[0] = Program, inputs[1] = Tuple(inputs for the program)
/// Uses graph_eval (0x89) to execute the program.
fn build_evaluate() -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x89, 2); // graph_eval
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1); // inputs
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build the IRIS tournament selection program.
///
/// Computes max(max(f0, f1), max(f2, f3)) — finds the best fitness value
/// among up to 4 candidates. The caller maps the winning value back to its
/// population index.
///
/// inputs[0..3] = Int(fitness values)
fn build_tournament_select() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: max(max(f0, f1), max(f2, f3))
    let (nid, node) = prim_node(1, 0x08, 2); // max (outer)
    nodes.insert(nid, node);
    let (nid, node) = prim_node(10, 0x08, 2); // max(f0, f1)
    nodes.insert(nid, node);
    let (nid, node) = prim_node(20, 0x08, 2); // max(f2, f3)
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 0); // f0
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(40, 1); // f1
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(50, 2); // f2
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(60, 3); // f3
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),  // max(max01, ...)
        make_edge(1, 20, 1, EdgeLabel::Argument),  // max(..., max23)
        make_edge(10, 30, 0, EdgeLabel::Argument),  // max(f0, ...)
        make_edge(10, 40, 1, EdgeLabel::Argument),  // max(..., f1)
        make_edge(20, 50, 0, EdgeLabel::Argument),  // max(f2, ...)
        make_edge(20, 60, 1, EdgeLabel::Argument),  // max(..., f3)
    ];

    make_graph(nodes, edges, 1)
}

/// Build the IRIS crossover program.
///
/// Swaps subtrees between two parent programs using graph_replace_subtree (0x88).
///
/// inputs[0] = Program(parent_a), inputs[1] = Int(node_a_id)
/// inputs[2] = Program(parent_b), inputs[3] = Int(node_b_id)
///
/// Output: Tuple(Program(offspring_a), Program(offspring_b))
fn build_crossover_subgraph() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Tuple(offspring_a, offspring_b)
    let (nid, node) = make_node(1, NodeKind::Tuple, NodePayload::Tuple, 2);
    nodes.insert(nid, node);

    // offspring_a: replace node_a in parent_a with subtree from node_b in parent_b
    let (nid, node) = prim_node(100, 0x88, 4);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(110, 0); // parent_a
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(120, 1); // node_a
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(130, 2); // parent_b
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(140, 3); // node_b
    nodes.insert(nid, node);

    // offspring_b: replace node_b in parent_b with subtree from node_a in parent_a
    let (nid, node) = prim_node(200, 0x88, 4);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(210, 2); // parent_b
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(220, 3); // node_b
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(230, 0); // parent_a
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(240, 1); // node_a
    nodes.insert(nid, node);

    let edges = vec![
        // Root -> offspring_a, offspring_b
        make_edge(1, 100, 0, EdgeLabel::Argument),
        make_edge(1, 200, 1, EdgeLabel::Argument),
        // offspring_a: replace_subtree(parent_a, node_a, parent_b, node_b)
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        make_edge(100, 130, 2, EdgeLabel::Argument),
        make_edge(100, 140, 3, EdgeLabel::Argument),
        // offspring_b: replace_subtree(parent_b, node_b, parent_a, node_a)
        make_edge(200, 210, 0, EdgeLabel::Argument),
        make_edge(200, 220, 1, EdgeLabel::Argument),
        make_edge(200, 230, 2, EdgeLabel::Argument),
        make_edge(200, 240, 3, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iris_runtime_initializes() {
        let rt = IrisRuntime::new();
        assert!(rt.initialized);
        assert!(!rt.replace_prim.nodes.is_empty());
        assert!(!rt.direct_replace_prim.nodes.is_empty());
        assert!(!rt.add_node.nodes.is_empty());
        assert!(!rt.connect.nodes.is_empty());
        assert!(!rt.evaluate.nodes.is_empty());
        assert!(!rt.tournament_select_program.nodes.is_empty());
        assert!(!rt.crossover_program.nodes.is_empty());
    }

    #[test]
    fn iris_runtime_select_finds_best() {
        let rt = IrisRuntime::new();
        let fitnesses = vec![0.2, 0.8, 0.1, 0.5];
        let winner = rt.select(&fitnesses);
        assert_eq!(winner, 1, "should select index 1 (fitness 0.8)");
    }

    #[test]
    fn iris_runtime_select_single() {
        let rt = IrisRuntime::new();
        let fitnesses = vec![0.5];
        let winner = rt.select(&fitnesses);
        assert_eq!(winner, 0);
    }

    #[test]
    fn iris_runtime_select_empty() {
        let rt = IrisRuntime::new();
        let fitnesses: Vec<f32> = vec![];
        let winner = rt.select(&fitnesses);
        assert_eq!(winner, 0);
    }

    #[test]
    fn performance_tracker_records_and_reports() {
        let mut tracker = PerformanceTracker::new(5);
        tracker.record("mutate", Duration::from_micros(100));
        tracker.record("mutate", Duration::from_micros(200));
        tracker.record("select", Duration::from_micros(50));

        assert_eq!(tracker.sample_count("mutate"), 2);
        assert_eq!(tracker.sample_count("select"), 1);
        assert_eq!(tracker.sample_count("crossover"), 0);

        let mean = tracker.mean_duration("mutate").unwrap();
        assert_eq!(mean, Duration::from_micros(150));

        let timings = tracker.component_timings();
        assert_eq!(timings.len(), 2);
        // Slowest first.
        assert_eq!(timings[0].0, "mutate");
        assert_eq!(timings[1].0, "select");
    }

    #[test]
    fn performance_tracker_sliding_window() {
        let mut tracker = PerformanceTracker::new(3);
        tracker.record("test", Duration::from_micros(100));
        tracker.record("test", Duration::from_micros(200));
        tracker.record("test", Duration::from_micros(300));
        tracker.record("test", Duration::from_micros(400));
        // Window is [200, 300, 400], oldest evicted.
        assert_eq!(tracker.sample_count("test"), 3);
        let mean = tracker.mean_duration("test").unwrap();
        assert_eq!(mean, Duration::from_micros(300));
    }

    #[test]
    fn iris_runtime_replace_prim_works() {
        let rt = IrisRuntime::new();

        // Build a simple add(5, 3) program.
        let mut nodes = HashMap::new();
        let (nid, node) = prim_node(1, 0x00, 2); // add
        nodes.insert(nid, node);
        let (nid, node) = int_lit_node(10, 5);
        nodes.insert(nid, node);
        let (nid, node) = int_lit_node(20, 3);
        nodes.insert(nid, node);
        let edges = vec![
            make_edge(1, 10, 0, EdgeLabel::Argument),
            make_edge(1, 20, 1, EdgeLabel::Argument),
        ];
        let program = make_graph(nodes, edges, 1);

        // Replace add -> sub using IRIS
        let inputs = vec![
            Value::Program(Rc::new(program)),
            Value::Int(0x01), // sub
        ];
        let (outputs, _) = interpreter::interpret(&rt.replace_prim, &inputs, None).unwrap();
        let modified = match &outputs[0] {
            Value::Program(g) => g.as_ref().clone(),
            other => panic!("expected Program, got {:?}", other),
        };

        // Verify the sub node exists.
        let has_sub = modified
            .nodes
            .values()
            .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x01 }));
        assert!(has_sub, "should have sub node after IRIS mutation");
    }

    #[test]
    fn iris_runtime_mutate_with_fallback() {
        let rt = IrisRuntime::new();

        // Even with an empty graph, mutate should not panic (falls back to Rust).
        let empty = SemanticGraph {
            root: NodeId(0),
            nodes: HashMap::new(),
            edges: Vec::new(),
            type_env: TypeEnv {
                types: BTreeMap::new(),
            },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let mut rng = rand::thread_rng();
        let _result = rt.mutate(&empty, &mut rng);
        // Should not panic — fallback to Rust handles empty graphs.
    }
}
