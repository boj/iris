use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::fragment::FragmentId;
use crate::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// StateStore — explicit threaded state for stateful computation
// ---------------------------------------------------------------------------

/// A key-value store threaded through computation as an explicit parameter.
///
/// Modeled after Haskell's State monad: state is passed in and returned out,
/// preserving referential transparency for the proof kernel. The underlying
/// representation is a sorted map for deterministic iteration order.
pub type StateStore = BTreeMap<String, Value>;

// ---------------------------------------------------------------------------
// EvalTier
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvalTier {
    /// Cheap: interpreter only, no HW counters, 10 test cases.
    A,
    /// Full counters: interpreter + perf_event_open, 50-200 test cases.
    B,
    /// JIT, async: JIT + full counters + optional rewrite rules, 200+ test cases.
    C,
}

// ---------------------------------------------------------------------------
// HwCounters
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HwCounters {
    pub instructions_retired: u64,
    pub cycles: u64,
    pub ipc: f32,
    pub l1d_miss_rate: f32,
    pub llc_miss_rate: f32,
    pub dtlb_miss_rate: f32,
    pub branch_miss_rate: f32,
}

// ---------------------------------------------------------------------------
// KnowledgeGraph — first-class graph data type for cognitive architectures
// ---------------------------------------------------------------------------

/// A node in a knowledge graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KGNode {
    pub id: String,
    pub label: String,
    /// Arbitrary key-value properties attached to this node.
    pub properties: BTreeMap<String, Value>,
}

/// A directed, weighted edge in a knowledge graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KGEdge {
    pub source: String,
    pub target: String,
    /// Semantic edge type: "is_a", "has_part", "causes", etc.
    pub edge_type: String,
    /// Hebbian-adjustable weight.
    pub weight: f64,
}

/// A general-purpose knowledge graph with nodes and directed, weighted edges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: BTreeMap<String, KGNode>,
    pub edges: Vec<KGEdge>,
}

impl KnowledgeGraph {
    /// Create an empty knowledge graph.
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: Vec::new(),
        }
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Value
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// FutureHandle — handle to an async computation result
// ---------------------------------------------------------------------------

/// A handle to a concurrently-executing computation. The inner `Mutex`
/// holds `None` while the computation is in-flight and `Some(value)` once
/// it has resolved.
///
/// `FutureHandle` wraps `Arc<Mutex<Option<Value>>>` with manual trait
/// implementations for the derive-heavy `Value` ecosystem (Serialize,
/// Deserialize, PartialEq, Debug, Clone).
#[derive(Clone)]
pub struct FutureHandle(pub Arc<Mutex<Option<Value>>>);

impl FutureHandle {
    /// Create a new pending (unresolved) future.
    pub fn pending() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    /// Resolve the future with a value.
    pub fn resolve(&self, val: Value) {
        let mut guard = self.0.lock().unwrap();
        *guard = Some(val);
    }

    /// Try to get the resolved value. Returns `None` if still pending.
    pub fn try_get(&self) -> Option<Value> {
        self.0.lock().unwrap().clone()
    }
}

impl fmt::Debug for FutureHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self.0.lock() {
            Ok(guard) => {
                if guard.is_some() {
                    "resolved"
                } else {
                    "pending"
                }
            }
            Err(_) => "poisoned",
        };
        write!(f, "FutureHandle({})", status)
    }
}

impl PartialEq for FutureHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Serialize for FutureHandle {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Serialize the resolved value if available, otherwise Unit.
        let guard = self.0.lock().unwrap();
        match &*guard {
            Some(val) => val.serialize(serializer),
            None => Value::Unit.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for FutureHandle {
    fn deserialize<D: serde::Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        // Futures cannot be meaningfully deserialized; return a resolved Unit.
        Ok(FutureHandle(Arc::new(Mutex::new(Some(Value::Unit)))))
    }
}

// ---------------------------------------------------------------------------
// Value
// ---------------------------------------------------------------------------

/// Runtime value produced by evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Int(i64),
    Nat(u64),
    Float64(f64),
    Float32(f32),
    Bool(bool),
    Bytes(Vec<u8>),
    Unit,
    Tuple(#[serde(with = "rc_vec_serde")] Rc<Vec<Value>>),
    Tagged(u16, Box<Value>),
    /// Explicit threaded state — a key-value store passed through computation.
    State(StateStore),
    /// First-class knowledge graph.
    Graph(KnowledgeGraph),
    /// A reified program graph — enables runtime self-modification.
    /// Programs can inspect and modify their own graph structure, then
    /// evaluate the modified version via `graph_eval`.
    ///
    /// Uses `Rc` for copy-on-write semantics: read-only operations
    /// (graph_eval, graph_get_*) borrow without cloning, while mutations
    /// clone only when refcount > 1.
    Program(Rc<SemanticGraph>),
    /// Handle to a concurrently-executing computation.
    /// `None` inside the mutex = pending; `Some` = resolved.
    Future(FutureHandle),
    /// UTF-8 string value.
    String(String),
    /// A suspended computation — evaluated lazily on access.
    /// Contains a SemanticGraph (the step function) and captured state.
    /// When forced, produces (element, next_thunk) or Unit (end of stream).
    Thunk(Arc<SemanticGraph>, Box<Value>),
    /// Lazy integer range [start, end). Produced by list_range, consumed by
    /// fold/map/filter without materializing the full Vec<Value>.
    Range(i64, i64),
}

// SAFETY: Value was Send+Sync before the Rc<Vec<Value>> change (all other
// variants are Send+Sync). Rc is not thread-safe, but in practice Values
// are deep-copied (via serde or manual reconstruction) at thread boundaries,
// and single-threaded evaluation never races on the refcount.
unsafe impl Send for Value {}
unsafe impl Sync for Value {}

/// Serde support for `Rc<Vec<Value>>` — serializes as a plain `Vec<Value>`.
mod rc_vec_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(rc: &Rc<Vec<Value>>, s: S) -> Result<S::Ok, S::Error> {
        rc.as_ref().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Rc<Vec<Value>>, D::Error> {
        Vec::<Value>::deserialize(d).map(Rc::new)
    }
}

// ---------------------------------------------------------------------------
// Value helpers
// ---------------------------------------------------------------------------

impl Value {
    /// Create a Tuple from a Vec (wraps in Rc).
    #[inline]
    pub fn tuple(elems: Vec<Value>) -> Self {
        Value::Tuple(Rc::new(elems))
    }

    /// Get a reference to tuple elements, or None.
    #[inline]
    pub fn as_tuple(&self) -> Option<&[Value]> {
        match self {
            Value::Tuple(rc) => Some(rc.as_slice()),
            _ => None,
        }
    }

    /// Unwrap into owned Vec, cloning only if Rc is shared.
    #[inline]
    pub fn into_tuple_vec(self) -> Option<Vec<Value>> {
        match self {
            Value::Tuple(rc) => Some(Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone())),
            Value::Range(s, e) => {
                if e <= s { Some(vec![]) }
                else { Some((s..e).map(Value::Int).collect()) }
            }
            _ => None,
        }
    }

    /// Length of a collection (Tuple or Range).
    #[inline]
    pub fn collection_len(&self) -> Option<usize> {
        match self {
            Value::Tuple(rc) => Some(rc.len()),
            Value::Range(s, e) => Some(if *e > *s { (*e - *s) as usize } else { 0 }),
            _ => None,
        }
    }

    /// Get element at index from a collection (Tuple or Range).
    #[inline]
    pub fn collection_get(&self, idx: usize) -> Option<Value> {
        match self {
            Value::Tuple(rc) => rc.get(idx).cloned(),
            Value::Range(s, e) => {
                let len = if *e > *s { (*e - *s) as usize } else { 0 };
                if idx < len { Some(Value::Int(*s + idx as i64)) } else { None }
            }
            _ => None,
        }
    }

    /// Borrow the inner SemanticGraph without cloning (for read-only ops).
    #[inline]
    pub fn as_program(&self) -> Option<&SemanticGraph> {
        match self {
            Value::Program(rc) => Some(rc.as_ref()),
            Value::Tuple(t) if !t.is_empty() => {
                if let Value::Program(rc) = &t[0] {
                    Some(rc.as_ref())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Extract an owned SemanticGraph, cloning only if the Rc is shared.
    #[inline]
    pub fn into_program(self) -> Option<SemanticGraph> {
        match self {
            Value::Program(rc) => Some(Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone())),
            Value::Tuple(t) => {
                let inner = Rc::try_unwrap(t).unwrap_or_else(|rc| (*rc).clone());
                if let Some(Value::Program(rc)) = inner.into_iter().next() {
                    Some(Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// EvalResult
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalResult {
    pub outputs: Vec<Vec<Value>>,
    /// FNV-1a hash of outputs.
    pub outputs_hash: u64,
    pub correctness_score: f32,
    /// Per-test-case correctness scores (0.0-1.0 each).
    /// Used by lexicase selection to preserve specialist individuals.
    pub per_case_scores: Vec<f32>,
    pub wall_time_ns: u64,
    pub compile_time_ns: u64,
    pub counters: Option<HwCounters>,
    pub tier_executed: EvalTier,
    pub cache_hit: bool,
    pub graph_hash: FragmentId,
}

// ---------------------------------------------------------------------------
// EvalError (6 variants)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EvalError {
    CompilationError {
        graph_hash: FragmentId,
        reason: String,
    },
    Timeout {
        graph_hash: FragmentId,
        wall_time_ns: u64,
        limit_ns: u64,
    },
    MemoryExceeded {
        graph_hash: FragmentId,
        bytes_used: usize,
        limit: usize,
    },
    ExecutionFault {
        graph_hash: FragmentId,
        signal: i32,
    },
    ArenaExhausted {
        numa_node: u8,
        utilization_pct: f32,
    },
    PerfCounterUnavailable {
        errno: i32,
    },
}

// ---------------------------------------------------------------------------
// TestCase
// ---------------------------------------------------------------------------

/// Input + optional expected output for evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestCase {
    pub inputs: Vec<Value>,
    pub expected_output: Option<Vec<Value>>,
    /// Optional initial state for stateful computation.
    /// If `None`, the interpreter starts with an empty `StateStore`.
    #[serde(default)]
    pub initial_state: Option<StateStore>,
    /// Optional expected final state after execution.
    /// Used for correctness scoring of stateful programs.
    #[serde(default)]
    pub expected_state: Option<StateStore>,
}

// ---------------------------------------------------------------------------
// EffectTag — categorizes the kind of I/O effect requested
// ---------------------------------------------------------------------------

/// Identifies the category of an effect. Effect nodes in the graph carry a
/// `u8` tag; this enum gives semantic meaning to the well-known tags while
/// reserving 0x0E-0xFF for user-defined extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectTag {
    /// Output a value (returns Unit).
    Print,
    /// Read a line of text (returns Bytes).
    ReadLine,
    /// GET a URL (returns Bytes).
    HttpGet,
    /// POST to a URL (returns Bytes).
    HttpPost,
    /// Read file contents (returns Bytes).
    FileRead,
    /// Write file contents (returns Unit).
    FileWrite,
    /// Execute a DB query (returns Tuple of results).
    DbQuery,
    /// Execute a DB mutation (returns Int rows affected).
    DbExecute,
    /// Sleep for N milliseconds (returns Unit).
    Sleep,
    /// Get current Unix timestamp in milliseconds (returns Int).
    Timestamp,
    /// Get a random integer (returns Int).
    Random,
    /// Log a message at info level (returns Unit).
    Log,
    /// Send a message to an IPC channel (returns Unit).
    SendMessage,
    /// Receive a message from an IPC channel (returns Value).
    RecvMessage,

    // ----- Raw I/O primitives (0x10-0x1F) -----
    // Syscall-level building blocks that HTTP clients, database drivers,
    // and everything else gets built FROM.

    /// Connect to a TCP endpoint: (host: String, port: Int) -> Connection handle (Int).
    TcpConnect,
    /// Read from a TCP connection: (conn: Int, max_bytes: Int) -> Bytes.
    TcpRead,
    /// Write to a TCP connection: (conn: Int, data: Bytes) -> Int (bytes written).
    TcpWrite,
    /// Close a TCP connection: (conn: Int) -> Unit.
    TcpClose,
    /// Listen on a TCP port: (port: Int) -> Listener handle (Int).
    TcpListen,
    /// Accept a connection on a listener: (listener: Int) -> Connection handle (Int).
    TcpAccept,

    /// Open a file: (path: String, mode: Int) -> Handle (Int). mode: 0=read, 1=write, 2=append.
    FileOpen,
    /// Read bytes from a file handle: (handle: Int, max_bytes: Int) -> Bytes.
    FileReadBytes,
    /// Write bytes to a file handle: (handle: Int, data: Bytes) -> Int (bytes written).
    FileWriteBytes,
    /// Close a file handle: (handle: Int) -> Unit.
    FileClose,
    /// Stat a file path: (path: String) -> Tuple(size: Int, modified_ns: Int, is_dir: Int).
    FileStat,
    /// List directory entries: (path: String) -> Tuple of Strings.
    DirList,

    /// Get an environment variable: (name: String) -> String (or Unit if not set).
    EnvGet,
    /// Get current nanosecond timestamp: () -> Int.
    ClockNs,
    /// Generate random bytes: (count: Int) -> Bytes.
    RandomBytes,
    /// Sleep for N milliseconds: (milliseconds: Int) -> Unit.
    SleepMs,

    // ----- Threading / atomic primitives (0x20-0x28) -----

    /// Spawn a thread: (program: Program) -> Future handle.
    ThreadSpawn,
    /// Join a thread: (handle: Future) -> result Value.
    ThreadJoin,
    /// Atomic read: (ref: String) -> Value from state.
    AtomicRead,
    /// Atomic write: (ref: String, value: Value) -> Unit.
    AtomicWrite,
    /// Atomic swap: (ref: String, new_value: Value) -> old Value.
    AtomicSwap,
    /// Atomic add: (ref: String, delta: Int) -> old Value.
    AtomicAdd,
    /// RwLock read: (lock: String) -> Value.
    RwLockRead,
    /// RwLock write: (lock: String, value: Value) -> Unit.
    RwLockWrite,
    /// RwLock release: (lock: String) -> Unit.
    RwLockRelease,

    // ----- JIT primitives (0x29-0x2A) -----

    /// Make bytes executable via mmap: (code: Bytes) -> Int (function pointer).
    /// Allocates RW memory, copies code, flips to RX (W^X). The returned
    /// integer is a raw function pointer suitable for `CallNative`.
    MmapExec,
    /// Call a JIT-compiled native function: (fn_ptr: Int, args: Tuple(Int...)) -> Int.
    /// Invokes the function pointer with up to 6 integer arguments using the
    /// System V AMD64 calling convention and returns the result in rax.
    CallNative,

    // ----- FFI primitives (0x2B) -----

    /// Call a foreign function via dlopen/dlsym: (lib_path: String, func_name: String, args: Tuple) -> Value.
    /// Opens the shared library at `lib_path` (or uses NULL for libc), looks up `func_name`,
    /// and calls it with the provided arguments using the System V AMD64 calling convention.
    /// Integer arguments map to i64, Float64 arguments map to f64 (via XMM registers).
    /// Returns the result as Int (i64 from rax).
    FfiCall,

    /// User-defined effect (0x0E-0x0F, 0x2C-0xFF).
    Custom(u8),
}

impl EffectTag {
    /// Convert a raw `u8` tag (from `NodePayload::Effect { effect_tag }`)
    /// into an `EffectTag`.
    pub fn from_u8(tag: u8) -> Self {
        match tag {
            0x00 => Self::Print,
            0x01 => Self::ReadLine,
            0x02 => Self::HttpGet,
            0x03 => Self::HttpPost,
            0x04 => Self::FileRead,
            0x05 => Self::FileWrite,
            0x06 => Self::DbQuery,
            0x07 => Self::DbExecute,
            0x08 => Self::Sleep,
            0x09 => Self::Timestamp,
            0x0A => Self::Random,
            0x0B => Self::Log,
            0x0C => Self::SendMessage,
            0x0D => Self::RecvMessage,
            // Raw I/O primitives
            0x10 => Self::TcpConnect,
            0x11 => Self::TcpRead,
            0x12 => Self::TcpWrite,
            0x13 => Self::TcpClose,
            0x14 => Self::TcpListen,
            0x15 => Self::TcpAccept,
            0x16 => Self::FileOpen,
            0x17 => Self::FileReadBytes,
            0x18 => Self::FileWriteBytes,
            0x19 => Self::FileClose,
            0x1A => Self::FileStat,
            0x1B => Self::DirList,
            0x1C => Self::EnvGet,
            0x1D => Self::ClockNs,
            0x1E => Self::RandomBytes,
            0x1F => Self::SleepMs,
            // Threading / atomic primitives
            0x20 => Self::ThreadSpawn,
            0x21 => Self::ThreadJoin,
            0x22 => Self::AtomicRead,
            0x23 => Self::AtomicWrite,
            0x24 => Self::AtomicSwap,
            0x25 => Self::AtomicAdd,
            0x26 => Self::RwLockRead,
            0x27 => Self::RwLockWrite,
            0x28 => Self::RwLockRelease,
            // JIT primitives
            0x29 => Self::MmapExec,
            0x2A => Self::CallNative,
            // FFI primitives
            0x2B => Self::FfiCall,
            other => Self::Custom(other),
        }
    }

    /// Convert to the canonical `u8` wire representation.
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Print => 0x00,
            Self::ReadLine => 0x01,
            Self::HttpGet => 0x02,
            Self::HttpPost => 0x03,
            Self::FileRead => 0x04,
            Self::FileWrite => 0x05,
            Self::DbQuery => 0x06,
            Self::DbExecute => 0x07,
            Self::Sleep => 0x08,
            Self::Timestamp => 0x09,
            Self::Random => 0x0A,
            Self::Log => 0x0B,
            Self::SendMessage => 0x0C,
            Self::RecvMessage => 0x0D,
            // Raw I/O primitives
            Self::TcpConnect => 0x10,
            Self::TcpRead => 0x11,
            Self::TcpWrite => 0x12,
            Self::TcpClose => 0x13,
            Self::TcpListen => 0x14,
            Self::TcpAccept => 0x15,
            Self::FileOpen => 0x16,
            Self::FileReadBytes => 0x17,
            Self::FileWriteBytes => 0x18,
            Self::FileClose => 0x19,
            Self::FileStat => 0x1A,
            Self::DirList => 0x1B,
            Self::EnvGet => 0x1C,
            Self::ClockNs => 0x1D,
            Self::RandomBytes => 0x1E,
            Self::SleepMs => 0x1F,
            // Threading / atomic primitives
            Self::ThreadSpawn => 0x20,
            Self::ThreadJoin => 0x21,
            Self::AtomicRead => 0x22,
            Self::AtomicWrite => 0x23,
            Self::AtomicSwap => 0x24,
            Self::AtomicAdd => 0x25,
            Self::RwLockRead => 0x26,
            Self::RwLockWrite => 0x27,
            Self::RwLockRelease => 0x28,
            // JIT primitives
            Self::MmapExec => 0x29,
            Self::CallNative => 0x2A,
            // FFI primitives
            Self::FfiCall => 0x2B,
            Self::Custom(v) => v,
        }
    }
}

// ---------------------------------------------------------------------------
// EffectRequest — description of an effect to be performed
// ---------------------------------------------------------------------------

/// A pure description of an I/O effect. The interpreter constructs this
/// from an Effect node's tag and evaluated arguments, then yields it to
/// the `EffectHandler`. This keeps the computation graph pure: effects
/// are descriptions, not actions.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectRequest {
    pub tag: EffectTag,
    pub args: Vec<Value>,
}

// ---------------------------------------------------------------------------
// EffectSet — for effect typing
// ---------------------------------------------------------------------------

/// A set of effect tags that a function may perform. Used for effect typing:
/// a function declared `pure` must have an empty effect set, while a function
/// that performs I/O lists the specific effects it uses.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectSet {
    /// Sorted, deduplicated set of effect tags (as u8).
    tags: Vec<u8>,
}

impl EffectSet {
    /// Empty effect set (pure function).
    pub fn pure() -> Self {
        Self { tags: Vec::new() }
    }

    /// Create from a single effect tag.
    pub fn singleton(tag: u8) -> Self {
        Self { tags: vec![tag] }
    }

    /// Create from multiple tags.
    pub fn from_tags(mut tags: Vec<u8>) -> Self {
        tags.sort();
        tags.dedup();
        Self { tags }
    }

    /// Union of two effect sets.
    pub fn union(&self, other: &Self) -> Self {
        let mut tags = self.tags.clone();
        tags.extend_from_slice(&other.tags);
        tags.sort();
        tags.dedup();
        Self { tags }
    }

    /// Check if this set is a subset of `other`.
    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.tags.iter().all(|t| other.tags.contains(t))
    }

    /// Check if the set is empty (pure).
    pub fn is_pure(&self) -> bool {
        self.tags.is_empty()
    }

    /// Get the tags as a slice.
    pub fn tags(&self) -> &[u8] {
        &self.tags
    }
}

impl Default for EffectSet {
    fn default() -> Self {
        Self::pure()
    }
}

// ---------------------------------------------------------------------------
// EffectError
// ---------------------------------------------------------------------------

/// Error returned by an effect handler when the effect cannot be performed.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectError {
    pub tag: EffectTag,
    pub message: String,
}

impl fmt::Display for EffectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "effect {:?} failed: {}", self.tag, self.message)
    }
}

impl std::error::Error for EffectError {}

// ---------------------------------------------------------------------------
// EffectHandler trait
// ---------------------------------------------------------------------------

/// Trait for handling effects at runtime. The runtime provides an
/// implementation that determines how each effect is performed.
pub trait EffectHandler {
    /// Handle an effect request and return the result value.
    fn handle(&self, request: EffectRequest) -> Result<Value, EffectError>;
}

/// Blanket implementation: a shared reference to an EffectHandler is itself
/// an EffectHandler. This enables `CapabilityGuardHandler::new(&handler, caps)`
/// where `handler: impl EffectHandler`.
impl<'a, T: EffectHandler + ?Sized> EffectHandler for &'a T {
    fn handle(&self, request: EffectRequest) -> Result<Value, EffectError> {
        (**self).handle(request)
    }
}
