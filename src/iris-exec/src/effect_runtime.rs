//! `RuntimeEffectHandler` — concrete implementation of `EffectHandler` that
//! performs real I/O, file operations, networking, timing, and randomness.
//!
//! This is the production handler that `CapabilityGuardHandler` wraps.
//! It implements every effect tag that the bootstrap evaluator can dispatch.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use iris_types::eval::{EffectError, EffectHandler, EffectRequest, EffectTag, Value};

// ---------------------------------------------------------------------------
// JIT W^X memory region
// ---------------------------------------------------------------------------

/// A JIT-compiled code region with W^X (Write XOR Execute) enforcement.
///
/// Lifecycle:
///   1. `mmap` with PROT_READ | PROT_WRITE (writable, not executable)
///   2. Copy machine code bytes into the region
///   3. `mprotect` to PROT_READ | PROT_EXEC (executable, not writable)
///   4. Region is never simultaneously writable AND executable
///   5. `munmap` on drop
///
/// The region is immutable after compilation — there is no mechanism to
/// re-enable writes. To modify JIT code, compile a new region.
#[cfg(feature = "jit")]
struct JitRegion {
    ptr: *mut u8,
    size: usize,
    code_len: usize,
}

#[cfg(feature = "jit")]
unsafe impl Send for JitRegion {}
#[cfg(feature = "jit")]
unsafe impl Sync for JitRegion {}

#[cfg(feature = "jit")]
impl Drop for JitRegion {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr as *mut libc::c_void, self.size);
        }
    }
}

#[cfg(feature = "jit")]
impl JitRegion {
    /// Compile code bytes into an executable W^X region.
    ///
    /// Returns an error if the code is empty, exceeds the size limit,
    /// or if mmap/mprotect fails.
    fn compile(code: &[u8]) -> Result<Self, String> {
        const MAX_JIT_SIZE: usize = 1024 * 1024; // 1 MiB limit

        if code.is_empty() {
            return Err("JIT: empty code buffer".into());
        }
        if code.len() > MAX_JIT_SIZE {
            return Err(format!("JIT: code size {} exceeds 1 MiB limit", code.len()));
        }

        let page_size = 4096usize;
        let aligned_size = (code.len() + page_size - 1) & !(page_size - 1);

        unsafe {
            // Step 1: Allocate RW pages (writable, NOT executable)
            let ptr = libc::mmap(
                std::ptr::null_mut(),
                aligned_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
                -1,
                0,
            );

            if ptr == libc::MAP_FAILED {
                return Err("JIT: mmap failed".into());
            }

            // Step 2: Copy code into the writable region
            std::ptr::copy_nonoverlapping(code.as_ptr(), ptr as *mut u8, code.len());

            // Step 3: Flip to RX (W^X transition — drop write, enable execute)
            let rc = libc::mprotect(
                ptr,
                aligned_size,
                libc::PROT_READ | libc::PROT_EXEC,
            );

            if rc != 0 {
                libc::munmap(ptr, aligned_size);
                return Err("JIT: mprotect RW→RX failed".into());
            }

            Ok(JitRegion {
                ptr: ptr as *mut u8,
                size: aligned_size,
                code_len: code.len(),
            })
        }
    }

    /// Call the compiled code with up to 6 integer arguments (System V AMD64 ABI).
    ///
    /// # Safety
    /// The caller must ensure the code in this region is valid x86-64 that:
    /// - Returns via `ret` instruction
    /// - Does not access memory outside its arguments
    /// - Follows System V AMD64 calling convention
    unsafe fn call(&self, args: &[i64]) -> i64 {
        type Fn0 = unsafe extern "C" fn() -> i64;
        type Fn1 = unsafe extern "C" fn(i64) -> i64;
        type Fn2 = unsafe extern "C" fn(i64, i64) -> i64;
        type Fn3 = unsafe extern "C" fn(i64, i64, i64) -> i64;
        type Fn4 = unsafe extern "C" fn(i64, i64, i64, i64) -> i64;
        type Fn5 = unsafe extern "C" fn(i64, i64, i64, i64, i64) -> i64;
        type Fn6 = unsafe extern "C" fn(i64, i64, i64, i64, i64, i64) -> i64;

        let fptr = self.ptr as *const ();
        match args.len() {
            0 => {
                let f: Fn0 = std::mem::transmute(fptr);
                f()
            }
            1 => {
                let f: Fn1 = std::mem::transmute(fptr);
                f(args[0])
            }
            2 => {
                let f: Fn2 = std::mem::transmute(fptr);
                f(args[0], args[1])
            }
            3 => {
                let f: Fn3 = std::mem::transmute(fptr);
                f(args[0], args[1], args[2])
            }
            4 => {
                let f: Fn4 = std::mem::transmute(fptr);
                f(args[0], args[1], args[2], args[3])
            }
            5 => {
                let f: Fn5 = std::mem::transmute(fptr);
                f(args[0], args[1], args[2], args[3], args[4])
            }
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
}

/// A concrete `EffectHandler` that performs real I/O.
///
/// Manages file handles and TCP connections via integer handle tables.
/// Thread-safe: all mutable state is behind `Mutex`/`RwLock`.
pub struct RuntimeEffectHandler {
    /// Open file handles: handle_id -> File.
    files: Mutex<HandleTable<std::fs::File>>,
    /// Open TCP streams: handle_id -> TcpStream.
    streams: Mutex<HandleTable<TcpStream>>,
    /// TCP listeners: handle_id -> TcpListener.
    listeners: Mutex<HandleTable<TcpListener>>,
    /// Shared atomic state for AtomicRead/Write/Swap/Add.
    atomic_state: RwLock<HashMap<String, Value>>,
    /// JIT code regions: handle_id -> (ptr, mapped_size).
    /// Each region is mmap'd RW, code is copied in, then mprotect'd to RX (W^X).
    #[cfg(feature = "jit")]
    jit_regions: Mutex<HandleTable<JitRegion>>,
}

struct HandleTable<T> {
    entries: HashMap<i64, T>,
    next_id: i64,
}

impl<T> HandleTable<T> {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            next_id: 1,
        }
    }

    fn insert(&mut self, item: T) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.insert(id, item);
        id
    }

    fn get(&self, id: i64) -> Option<&T> {
        self.entries.get(&id)
    }

    fn get_mut(&mut self, id: i64) -> Option<&mut T> {
        self.entries.get_mut(&id)
    }

    fn remove(&mut self, id: i64) -> Option<T> {
        self.entries.remove(&id)
    }
}

impl RuntimeEffectHandler {
    pub fn new() -> Self {
        Self {
            files: Mutex::new(HandleTable::new()),
            streams: Mutex::new(HandleTable::new()),
            listeners: Mutex::new(HandleTable::new()),
            atomic_state: RwLock::new(HashMap::new()),
            #[cfg(feature = "jit")]
            jit_regions: Mutex::new(HandleTable::new()),
        }
    }
}

impl Default for RuntimeEffectHandler {
    fn default() -> Self {
        Self::new()
    }
}

fn err(tag: EffectTag, msg: impl Into<String>) -> EffectError {
    EffectError { tag, message: msg.into() }
}

fn expect_string(tag: EffectTag, args: &[Value], idx: usize) -> Result<String, EffectError> {
    match args.get(idx) {
        Some(Value::String(s)) => Ok(s.clone()),
        Some(Value::Bytes(b)) => Ok(String::from_utf8_lossy(b).into_owned()),
        _ => Err(err(tag, format!("arg[{}] must be String", idx))),
    }
}

fn expect_int(tag: EffectTag, args: &[Value], idx: usize) -> Result<i64, EffectError> {
    match args.get(idx) {
        Some(Value::Int(n)) => Ok(*n),
        _ => Err(err(tag, format!("arg[{}] must be Int", idx))),
    }
}

fn expect_bytes(tag: EffectTag, args: &[Value], idx: usize) -> Result<Vec<u8>, EffectError> {
    match args.get(idx) {
        Some(Value::Bytes(b)) => Ok(b.clone()),
        Some(Value::String(s)) => Ok(s.as_bytes().to_vec()),
        _ => Err(err(tag, format!("arg[{}] must be Bytes", idx))),
    }
}

impl EffectHandler for RuntimeEffectHandler {
    fn handle(&self, request: EffectRequest) -> Result<Value, EffectError> {
        let tag = request.tag;
        let args = &request.args;

        match tag {
            // ---------------------------------------------------------------
            // Print / Log / ReadLine
            // ---------------------------------------------------------------
            EffectTag::Print => {
                for arg in args {
                    match arg {
                        Value::String(s) => eprint!("{}", s),
                        Value::Int(n) => eprint!("{}", n),
                        other => eprint!("{:?}", other),
                    }
                }
                let _ = std::io::stderr().flush();
                Ok(Value::Unit)
            }

            EffectTag::Log => {
                for arg in args {
                    match arg {
                        Value::String(s) => eprintln!("[LOG] {}", s),
                        other => eprintln!("[LOG] {:?}", other),
                    }
                }
                Ok(Value::Unit)
            }

            EffectTag::ReadLine => {
                let mut line = String::new();
                std::io::stdin()
                    .read_line(&mut line)
                    .map_err(|e| err(tag, e.to_string()))?;
                Ok(Value::String(line))
            }

            // ---------------------------------------------------------------
            // Time / Random / Sleep
            // ---------------------------------------------------------------
            EffectTag::Timestamp => {
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64;
                Ok(Value::Int(ts))
            }

            EffectTag::ClockNs => {
                let ns = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as i64;
                Ok(Value::Int(ns))
            }

            EffectTag::Random => {
                // Simple PRNG using system time — NOT cryptographic.
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos();
                // xorshift-ish from timestamp nanos.
                let r = (seed ^ (seed >> 17) ^ (seed >> 31)) as i64;
                Ok(Value::Int(r.wrapping_abs()))
            }

            EffectTag::RandomBytes => {
                let count = expect_int(tag, args, 0)?.max(0) as usize;
                let mut buf = vec![0u8; count];
                // Read from /dev/urandom for real randomness.
                if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
                    let _ = f.read_exact(&mut buf);
                }
                Ok(Value::Bytes(buf))
            }

            EffectTag::Sleep | EffectTag::SleepMs => {
                let ms = expect_int(tag, args, 0)?.max(0) as u64;
                std::thread::sleep(std::time::Duration::from_millis(ms));
                Ok(Value::Unit)
            }

            // ---------------------------------------------------------------
            // File operations (path-based)
            // ---------------------------------------------------------------
            EffectTag::FileRead => {
                let path = expect_string(tag, args, 0)?;
                let contents = std::fs::read_to_string(&path)
                    .map_err(|e| err(tag, format!("{}: {}", path, e)))?;
                Ok(Value::Bytes(contents.into_bytes()))
            }

            EffectTag::FileWrite => {
                let path = expect_string(tag, args, 0)?;
                let data = expect_bytes(tag, args, 1)?;
                std::fs::write(&path, &data)
                    .map_err(|e| err(tag, format!("{}: {}", path, e)))?;
                Ok(Value::Unit)
            }

            EffectTag::FileOpen => {
                let path = expect_string(tag, args, 0)?;
                let mode = expect_int(tag, args, 1).unwrap_or(0);
                let file = match mode {
                    0 => std::fs::File::open(&path),
                    1 => std::fs::File::create(&path),
                    2 => std::fs::OpenOptions::new().append(true).create(true).open(&path),
                    _ => return Err(err(tag, format!("invalid mode {}", mode))),
                };
                let file = file.map_err(|e| err(tag, format!("{}: {}", path, e)))?;
                let handle = self.files.lock().unwrap().insert(file);
                Ok(Value::Int(handle))
            }

            EffectTag::FileReadBytes => {
                let handle = expect_int(tag, args, 0)?;
                let max_bytes = expect_int(tag, args, 1)?.max(0) as usize;
                let mut files = self.files.lock().unwrap();
                let file = files.get_mut(handle)
                    .ok_or_else(|| err(tag, format!("invalid handle {}", handle)))?;
                let mut buf = vec![0u8; max_bytes];
                let n = file.read(&mut buf)
                    .map_err(|e| err(tag, e.to_string()))?;
                buf.truncate(n);
                Ok(Value::Bytes(buf))
            }

            EffectTag::FileWriteBytes => {
                let handle = expect_int(tag, args, 0)?;
                let data = expect_bytes(tag, args, 1)?;
                let mut files = self.files.lock().unwrap();
                let file = files.get_mut(handle)
                    .ok_or_else(|| err(tag, format!("invalid handle {}", handle)))?;
                let n = file.write(&data)
                    .map_err(|e| err(tag, e.to_string()))?;
                Ok(Value::Int(n as i64))
            }

            EffectTag::FileClose => {
                let handle = expect_int(tag, args, 0)?;
                self.files.lock().unwrap().remove(handle);
                Ok(Value::Unit)
            }

            EffectTag::FileStat => {
                let path = expect_string(tag, args, 0)?;
                let meta = std::fs::metadata(&path)
                    .map_err(|e| err(tag, format!("{}: {}", path, e)))?;
                let size = meta.len() as i64;
                let modified_ns = meta.modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos() as i64)
                    .unwrap_or(0);
                let is_dir = if meta.is_dir() { 1i64 } else { 0 };
                Ok(Value::tuple(vec![
                    Value::Int(size),
                    Value::Int(modified_ns),
                    Value::Int(is_dir),
                ]))
            }

            EffectTag::DirList => {
                let path = expect_string(tag, args, 0)?;
                let mut entries = Vec::new();
                for entry in std::fs::read_dir(&path)
                    .map_err(|e| err(tag, format!("{}: {}", path, e)))?
                {
                    if let Ok(e) = entry {
                        entries.push(Value::String(
                            e.file_name().to_string_lossy().into_owned(),
                        ));
                    }
                }
                Ok(Value::tuple(entries))
            }

            // ---------------------------------------------------------------
            // Environment
            // ---------------------------------------------------------------
            EffectTag::EnvGet => {
                let name = expect_string(tag, args, 0)?;
                match std::env::var(&name) {
                    Ok(val) => Ok(Value::String(val)),
                    Err(_) => Ok(Value::Unit),
                }
            }

            // ---------------------------------------------------------------
            // TCP networking
            // ---------------------------------------------------------------
            EffectTag::TcpConnect => {
                let host = expect_string(tag, args, 0)?;
                let port = expect_int(tag, args, 1)? as u16;
                let stream = TcpStream::connect(format!("{}:{}", host, port))
                    .map_err(|e| err(tag, e.to_string()))?;
                let handle = self.streams.lock().unwrap().insert(stream);
                Ok(Value::Int(handle))
            }

            EffectTag::TcpRead => {
                let handle = expect_int(tag, args, 0)?;
                let max_bytes = expect_int(tag, args, 1)?.max(0) as usize;
                let mut streams = self.streams.lock().unwrap();
                let stream = streams.get_mut(handle)
                    .ok_or_else(|| err(tag, format!("invalid handle {}", handle)))?;
                let mut buf = vec![0u8; max_bytes];
                let n = stream.read(&mut buf)
                    .map_err(|e| err(tag, e.to_string()))?;
                buf.truncate(n);
                Ok(Value::Bytes(buf))
            }

            EffectTag::TcpWrite => {
                let handle = expect_int(tag, args, 0)?;
                let data = expect_bytes(tag, args, 1)?;
                let mut streams = self.streams.lock().unwrap();
                let stream = streams.get_mut(handle)
                    .ok_or_else(|| err(tag, format!("invalid handle {}", handle)))?;
                let n = stream.write(&data)
                    .map_err(|e| err(tag, e.to_string()))?;
                Ok(Value::Int(n as i64))
            }

            EffectTag::TcpClose => {
                let handle = expect_int(tag, args, 0)?;
                self.streams.lock().unwrap().remove(handle);
                Ok(Value::Unit)
            }

            EffectTag::TcpListen => {
                let port = expect_int(tag, args, 0)? as u16;
                let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
                    .map_err(|e| err(tag, e.to_string()))?;
                let handle = self.listeners.lock().unwrap().insert(listener);
                Ok(Value::Int(handle))
            }

            EffectTag::TcpAccept => {
                let handle = expect_int(tag, args, 0)?;
                let listeners = self.listeners.lock().unwrap();
                let listener = listeners.get(handle)
                    .ok_or_else(|| err(tag, format!("invalid handle {}", handle)))?;
                let (stream, _addr) = listener.accept()
                    .map_err(|e| err(tag, e.to_string()))?;
                drop(listeners);
                let stream_handle = self.streams.lock().unwrap().insert(stream);
                Ok(Value::Int(stream_handle))
            }

            // ---------------------------------------------------------------
            // Atomic state (in-process shared memory)
            // ---------------------------------------------------------------
            EffectTag::AtomicRead => {
                let key = expect_string(tag, args, 0)?;
                let state = self.atomic_state.read().unwrap();
                Ok(state.get(&key).cloned().unwrap_or(Value::Unit))
            }

            EffectTag::AtomicWrite => {
                let key = expect_string(tag, args, 0)?;
                let val = args.get(1).cloned().unwrap_or(Value::Unit);
                self.atomic_state.write().unwrap().insert(key, val);
                Ok(Value::Unit)
            }

            EffectTag::AtomicSwap => {
                let key = expect_string(tag, args, 0)?;
                let new_val = args.get(1).cloned().unwrap_or(Value::Unit);
                let mut state = self.atomic_state.write().unwrap();
                let old = state.insert(key, new_val).unwrap_or(Value::Unit);
                Ok(old)
            }

            EffectTag::AtomicAdd => {
                let key = expect_string(tag, args, 0)?;
                let delta = expect_int(tag, args, 1)?;
                let mut state = self.atomic_state.write().unwrap();
                let old = state.entry(key).or_insert(Value::Int(0));
                if let Value::Int(n) = old {
                    let prev = *n;
                    *n += delta;
                    Ok(Value::Int(prev))
                } else {
                    Err(err(tag, "AtomicAdd: value is not Int"))
                }
            }

            // RwLock* — map to the same atomic state with read/write semantics.
            EffectTag::RwLockRead => {
                let key = expect_string(tag, args, 0)?;
                let state = self.atomic_state.read().unwrap();
                Ok(state.get(&key).cloned().unwrap_or(Value::Unit))
            }

            EffectTag::RwLockWrite => {
                let key = expect_string(tag, args, 0)?;
                let val = args.get(1).cloned().unwrap_or(Value::Unit);
                self.atomic_state.write().unwrap().insert(key, val);
                Ok(Value::Unit)
            }

            EffectTag::RwLockRelease => {
                // No-op — Rust's RwLock releases automatically.
                Ok(Value::Unit)
            }

            // ---------------------------------------------------------------
            // Threading (stub — returns error, real threading needs OS layer)
            // ---------------------------------------------------------------
            EffectTag::ThreadSpawn | EffectTag::ThreadJoin => {
                Err(err(tag, "threading effects require OS-layer integration"))
            }

            // ---------------------------------------------------------------
            // HTTP (stub — IRIS programs should build these from TCP)
            // ---------------------------------------------------------------
            EffectTag::HttpGet | EffectTag::HttpPost => {
                Err(err(tag, "HTTP effects not implemented; use TCP primitives"))
            }

            // ---------------------------------------------------------------
            // DB (stub — IRIS programs should build these from file/TCP)
            // ---------------------------------------------------------------
            EffectTag::DbQuery | EffectTag::DbExecute => {
                Err(err(tag, "DB effects not implemented; use file/TCP primitives"))
            }

            // ---------------------------------------------------------------
            // IPC channels (stub)
            // ---------------------------------------------------------------
            EffectTag::SendMessage | EffectTag::RecvMessage => {
                Err(err(tag, "IPC channel effects not implemented"))
            }

            // ---------------------------------------------------------------
            // JIT — W^X memory-mapped code execution
            // ---------------------------------------------------------------
            #[cfg(feature = "jit")]
            EffectTag::MmapExec => {
                let code = expect_bytes(tag, args, 0)?;
                let region = JitRegion::compile(&code)
                    .map_err(|msg| err(tag, msg))?;
                let handle = self.jit_regions.lock().unwrap().insert(region);
                Ok(Value::Int(handle))
            }

            #[cfg(feature = "jit")]
            EffectTag::CallNative => {
                let handle = expect_int(tag, args, 0)?;
                let call_args: Vec<i64> = match args.get(1) {
                    Some(Value::Tuple(elems)) => {
                        elems.iter().map(|v| match v {
                            Value::Int(n) => *n,
                            _ => 0,
                        }).collect()
                    }
                    Some(Value::Int(n)) => vec![*n],
                    _ => vec![],
                };

                let regions = self.jit_regions.lock().unwrap();
                let region = regions.get(handle)
                    .ok_or_else(|| err(tag, format!("invalid JIT handle {}", handle)))?;

                let result = unsafe { region.call(&call_args) };
                Ok(Value::Int(result))
            }

            // JIT disabled — feature not enabled
            #[cfg(not(feature = "jit"))]
            EffectTag::MmapExec | EffectTag::CallNative => {
                Err(err(tag, "JIT is disabled; rebuild with --features jit"))
            }

            EffectTag::FfiCall => {
                Err(err(tag, "FfiCall is disabled; use the CLCU hardware layer"))
            }

            // ---------------------------------------------------------------
            // Custom / unknown
            // ---------------------------------------------------------------
            EffectTag::Custom(byte) => {
                Err(err(tag, format!("unknown custom effect tag 0x{:02x}", byte)))
            }
        }
    }
}

// Wrap in Arc for shared ownership across evaluation contexts.
impl RuntimeEffectHandler {
    /// Create a shared (Arc-wrapped) handler for use across threads.
    pub fn shared() -> Arc<Self> {
        Arc::new(Self::new())
    }
}

// ---------------------------------------------------------------------------
// NoOpHandler — returns Unit for everything
// ---------------------------------------------------------------------------

/// An `EffectHandler` that silently returns `Value::Unit` for all effects.
/// Useful for testing pure computations where effects should be no-ops.
pub struct NoOpHandler;

impl EffectHandler for NoOpHandler {
    fn handle(&self, _request: EffectRequest) -> Result<Value, EffectError> {
        Ok(Value::Unit)
    }
}

// ---------------------------------------------------------------------------
// LoggingHandler — captures effect requests for inspection
// ---------------------------------------------------------------------------

/// An `EffectHandler` that records all effect requests and returns `Value::Unit`.
/// Useful for testing that programs issue the expected effects.
pub struct LoggingHandler {
    log: Mutex<Vec<EffectRequest>>,
}

impl LoggingHandler {
    pub fn new() -> Self {
        Self {
            log: Mutex::new(Vec::new()),
        }
    }

    /// Return all captured effect requests.
    pub fn requests(&self) -> Vec<EffectRequest> {
        self.log.lock().unwrap().clone()
    }

    /// Return all captured requests (alias for `requests()`).
    pub fn entries(&self) -> Vec<EffectRequest> {
        self.requests()
    }

    /// Clear all captured requests.
    pub fn clear(&self) {
        self.log.lock().unwrap().clear();
    }

    /// Return the number of captured requests.
    pub fn len(&self) -> usize {
        self.log.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for LoggingHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectHandler for LoggingHandler {
    fn handle(&self, request: EffectRequest) -> Result<Value, EffectError> {
        self.log.lock().unwrap().push(request);
        Ok(Value::Unit)
    }
}
