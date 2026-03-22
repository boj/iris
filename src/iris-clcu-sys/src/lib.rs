//! Raw FFI bindings to the iris-clcu C library.
//!
//! Mirrors the types and functions declared in `clcu.h`. The C library is
//! compiled and statically linked via the build.rs in this crate.

#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::os::raw::c_void;

// ---------------------------------------------------------------------------
// Constants matching clcu.h
// ---------------------------------------------------------------------------

pub const CLCU_CONTAINER_SIZE: usize = 64;
pub const CLCU_HEADER_SIZE: usize = 16;
pub const CLCU_PAYLOAD_SIZE: usize = 48;
pub const CLCU_MAX_MICRO_OPS: usize = 8;
pub const CLCU_MICRO_OP_SIZE: usize = 6;
pub const ARENA_SIZE: usize = 2 * 1024 * 1024;
pub const CONTAINERS_PER_ARENA: u32 = (ARENA_SIZE / CLCU_CONTAINER_SIZE) as u32;

// Next-container sentinel values.
pub const CLCU_NEXT_SEQUENTIAL: i32 = 0;
pub const CLCU_NEXT_TERMINAL: i32 = 0x7FFFFFFF;
pub const CLCU_NEXT_YIELD: i32 = 0x7FFFFFFE;

// Execution status flags.
pub const EXEC_STATUS_OK: u32 = 0x0;
pub const EXEC_STATUS_HALTED: u32 = 0x1;
pub const EXEC_STATUS_FAULT: u32 = 0x2;
pub const EXEC_STATUS_YIELDED: u32 = 0x4;

// Flag defines.
pub const CLCU_FLAG_IS_ENTRY: u8 = 1 << 7;
pub const CLCU_FLAG_IS_CONTINUATION: u8 = 1 << 6;
pub const CLCU_FLAG_PREFETCH_VALID: u8 = 1 << 5;

// ---------------------------------------------------------------------------
// C struct mirror: clcu_container_t — 64 bytes, cache-line aligned
// ---------------------------------------------------------------------------

/// Matches `clcu_container_t` from clcu.h exactly.
///
/// Memory layout (packed, 64-byte aligned):
///   [0..2)   magic_and_version (u16)
///   [2]      container_index (u8)
///   [3]      flags (u8)
///   [4..8)   next_container (i32, LE)
///   [8..12)  prefetch_target (i32, LE)
///   [12..14) mask_immediate (u16, LE)
///   [14]     zmm_live_in (u8)
///   [15]     zmm_live_out (u8)
///   [16..64) payload (48 bytes)
#[repr(C, align(64))]
#[derive(Clone, Copy)]
pub struct clcu_container_t {
    pub magic_and_version: u16,
    pub container_index: u8,
    pub flags: u8,
    pub next_container: i32,
    pub prefetch_target: i32,
    pub mask_immediate: u16,
    pub zmm_live_in: u8,
    pub zmm_live_out: u8,
    pub payload: [u8; CLCU_PAYLOAD_SIZE],
}

// Sanity check that the Rust repr matches the C struct size.
const _: () = assert!(std::mem::size_of::<clcu_container_t>() == 64);

impl Default for clcu_container_t {
    fn default() -> Self {
        // Safety: zero-initialized container is valid (NOP payload, no flags).
        unsafe { std::mem::zeroed() }
    }
}

// ---------------------------------------------------------------------------
// C struct mirror: clcu_arena_t
// ---------------------------------------------------------------------------

/// Opaque arena handle — we only pass pointers to/from C.
#[repr(C)]
pub struct clcu_arena_t {
    pub base: *mut c_void,
    pub next_free: u32,
    pub capacity: u32,
}

// ---------------------------------------------------------------------------
// C struct mirror: exec_state_t (scalar fallback layout)
//
// When compiled without __AVX512F__, the C struct uses:
//   int32_t zmm[32][16]   — 32 registers x 16 lanes x 4 bytes = 2048 bytes
//   uint16_t k[8]         — 8 mask registers x 2 bytes = 16 bytes
//   const clcu_container_t *pc  — pointer
//   uint64_t cycle_count
//   uint32_t status_flags
//   uint32_t containers_executed
//
// With __AVX512F__, the zmm field uses __m512i (also 64 bytes each, same
// total size) and k uses __mmask16 (also uint16_t). The memory layout is
// compatible.
// ---------------------------------------------------------------------------

/// ZMM register bank — 32 registers, each 16 x i32 = 64 bytes.
pub const ZMM_LANES: usize = 16;
pub const ZMM_COUNT: usize = 32;
pub const MASK_COUNT: usize = 8;

#[repr(C, align(64))]
#[derive(Clone, Copy)]
pub struct exec_state_t {
    /// 32 ZMM registers, each 16 x i32 (64 bytes).
    pub zmm: [[i32; ZMM_LANES]; ZMM_COUNT],
    /// 8 mask registers (k0-k7), each 16 bits.
    pub k: [u16; MASK_COUNT],
    /// Current container pointer (PC).
    pub pc: *const clcu_container_t,
    /// Accumulated cycle count.
    pub cycle_count: u64,
    /// Execution status flags.
    pub status_flags: u32,
    /// Number of containers executed.
    pub containers_executed: u32,
}

impl Default for exec_state_t {
    fn default() -> Self {
        Self {
            zmm: [[0i32; ZMM_LANES]; ZMM_COUNT],
            k: [0u16; MASK_COUNT],
            pc: std::ptr::null(),
            cycle_count: 0,
            status_flags: EXEC_STATUS_OK,
            containers_executed: 0,
        }
    }
}

// SAFETY: exec_state_t contains a `*const clcu_container_t` field (`pc`) that
// is written by the C interpreter during execution and read back only by the C
// interpreter itself.  Rust code never dereferences this pointer.  All other
// fields (zmm registers, mask registers, counters, status flags) are plain
// integer data with no aliasing.  Because exec_state_t is exclusively owned by
// its creator (passed by exclusive mutable reference to `execute_chain`), it
// can be safely moved to another thread — hence Send.  Sync is NOT implemented
// because concurrent mutation from multiple threads would be unsound.
unsafe impl Send for exec_state_t {}

// ---------------------------------------------------------------------------
// FFI function declarations
// ---------------------------------------------------------------------------

unsafe extern "C" {
    /// Initialize the interpreter dispatch table. Must be called once before
    /// any `execute_*` calls.
    pub fn interpreter_init();

    /// Execute a single 64-byte container.
    pub fn execute_container(state: *mut exec_state_t, c: *const clcu_container_t);

    /// Execute a chain of containers starting at `entry`, following
    /// continuation pointers until terminal or yield.
    pub fn execute_chain(state: *mut exec_state_t, entry: *const clcu_container_t);

    /// Create a new 2MB arena for container allocation.
    /// Returns null on failure.
    pub fn arena_create() -> *mut clcu_arena_t;

    /// Allocate `n` contiguous containers from the arena.
    /// Returns a pointer to the first container, or null if out of space.
    pub fn arena_alloc(arena: *mut clcu_arena_t, n: u32) -> *mut clcu_container_t;

    /// Destroy an arena and release its memory.
    pub fn arena_destroy(arena: *mut clcu_arena_t);
}

// ---------------------------------------------------------------------------
// Micro-op encoding helpers (matching the C uop_* inline functions)
//
// C micro-op byte layout:
//   [0]   opcode
//   [1]   [7:5]=dst_reg  [4:2]=src1_reg  [1:0]=src2_reg_hi
//   [2]   [7]=src2_reg_lo  [6:4]=width  [3:0]=modifier
//   [3-5] immediate (24-bit, little-endian)
// ---------------------------------------------------------------------------

/// Encode a single 6-byte micro-op in the C interpreter's format.
///
/// Register indices are 3 bits (0-7). Width and modifier are packed into
/// byte 2 as specified in clcu.h.
pub fn encode_micro_op(
    opcode: u8,
    dst: u8,
    src1: u8,
    src2: u8,
    width: u8,
    modifier: u8,
    immediate: u32,
) -> [u8; 6] {
    let dst3 = dst & 0x07;
    let src1_3 = src1 & 0x07;
    let src2_3 = src2 & 0x07;

    let byte1 = (dst3 << 5) | (src1_3 << 2) | ((src2_3 >> 1) & 0x03);
    let byte2 = ((src2_3 & 0x01) << 7) | ((width & 0x07) << 4) | (modifier & 0x0F);

    let imm_bytes = immediate.to_le_bytes();

    [opcode, byte1, byte2, imm_bytes[0], imm_bytes[1], imm_bytes[2]]
}
