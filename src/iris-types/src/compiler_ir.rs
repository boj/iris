//! Intermediate representation types for each compiler pass.
//!
//! The pipeline transforms SemanticGraph through these IRs:
//!   SemanticGraph -> MonoGraph -> FirstOrderGraph -> PredicatedGraph
//!   -> LoopGraph -> TrampolineGraph -> PrimitiveGraph -> LayoutGraph
//!   -> MicroOpSequence -> AllocatedSequence -> CLCUChain

use std::collections::BTreeMap;

use crate::cost::CostBound;
use crate::graph::{BinderId, Edge, NodeId};
use crate::guard::BlobRef;
use crate::types::{TypeEnv, TypeRef};

// ---------------------------------------------------------------------------
// Shared: Lightweight node used by all mid-level IRs
// ---------------------------------------------------------------------------

/// A simplified node used across IR stages (post-monomorphization).
#[derive(Debug, Clone, PartialEq)]
pub struct IrNode {
    pub id: NodeId,
    pub kind: IrNodeKind,
    pub type_ref: TypeRef,
}

/// Node kinds that appear across IR stages. Each pass removes some variants
/// and may introduce new ones.
#[derive(Debug, Clone, PartialEq)]
pub enum IrNodeKind {
    // -- Carried from SemanticGraph (some removed per pass) --
    Prim { opcode: u8 },
    Apply,
    Lambda { binder: BinderId, captured_count: u32 },
    Let,
    Match { arm_count: u16, arm_patterns: Vec<u8> },
    Lit { type_tag: u8, value: Vec<u8> },
    Ref { fragment_id: crate::fragment::FragmentId },
    Neural { weight_blob: BlobRef, param_count: u64 },
    Fold { recursion_descriptor: Vec<u8> },
    Unfold { recursion_descriptor: Vec<u8> },
    Effect { effect_tag: u8 },
    Tuple,
    Inject { tag_index: u16 },
    Project { field_index: u16 },
    LetRec { binder: BinderId, decrease: crate::types::DecreaseWitness },
    Guard { predicate_node: NodeId, body_node: NodeId, fallback_node: NodeId },
    Rewrite { rule_id: crate::graph::RewriteRuleId, body: NodeId },
    Extern { name: [u8; 32], type_sig: TypeRef },

    // -- Introduced by pass 2 (defunctionalization) --
    /// Direct call to a defunctionalized procedure.
    DirectCall { procedure_idx: usize },
    /// Closure construction: tag + captured values.
    MakeClosure { tag: ClosureTag },
    /// Tag-dispatch: replaces Apply on closure values.
    ClosureDispatch,

    // -- Introduced by pass 3 (match lowering) --
    /// Generate a predicate mask from a comparison.
    MaskGen { mask_kind: MaskKind },
    /// Predicated select: choose between two values based on mask.
    Select,

    // -- Introduced by pass 4 (fold/recursion lowering) --
    LoopHeader { loop_id: u32 },
    LoopBody { loop_id: u32 },
    LoopExit { loop_id: u32 },
    LoopBackedge { loop_id: u32 },

    // -- Introduced by pass 5 (effect lowering) --
    EffectYield { effect_tag: u8 },
    EffectResume { effect_tag: u8 },

    // -- Introduced by pass 6 (neural lowering) --
    /// Inline FMA sequence for tiny networks.
    InlineFma { param_count: u64 },
    /// Tiled matrix multiply for small networks.
    TiledMatMul { rows: u32, cols: u32 },
    /// External call for large networks.
    ExternCall { name: [u8; 32] },
    /// Load weight blob from weight arena.
    WeightLoad { blob: BlobRef },
    /// Activation function (ReLU, etc.).
    Activation { kind: u8 },
}

// ---------------------------------------------------------------------------
// ClosureTag
// ---------------------------------------------------------------------------

/// 16-bit discriminant for defunctionalized closures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClosureTag(pub u16);

// ---------------------------------------------------------------------------
// MaskKind
// ---------------------------------------------------------------------------

/// The kind of comparison used in mask generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaskKind {
    /// Equality comparison.
    Eq,
    /// Less-than comparison.
    Lt,
    /// Less-or-equal comparison.
    Le,
    /// Tag comparison for sum type dispatch.
    TagEq(u16),
}

// ---------------------------------------------------------------------------
// Procedure (defunctionalization output)
// ---------------------------------------------------------------------------

/// A defunctionalized function body.
#[derive(Debug, Clone, PartialEq)]
pub struct Procedure {
    pub tag: ClosureTag,
    pub binder: BinderId,
    pub captured_types: Vec<TypeRef>,
    pub body_nodes: Vec<NodeId>,
    pub body_edges: Vec<Edge>,
    pub return_type: TypeRef,
}

// ---------------------------------------------------------------------------
// LayoutAnnotation (pass 7)
// ---------------------------------------------------------------------------

/// Physical data layout annotation for a value.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutAnnotation {
    /// Single ZMM register (scalars broadcast to all lanes).
    Scalar,
    /// Struct-of-arrays: one ZMM per field.
    StructOfArrays { field_count: u16 },
    /// Tagged union: tag in high bits of lane 0.
    Tagged { tag_bits: u8 },
    /// Arena-allocated with BFS layout.
    ArenaRef { element_size: u32 },
    /// Sized vector across multiple ZMMs.
    Vector { zmm_count: u16 },
}

// ---------------------------------------------------------------------------
// Stage 1: MonoGraph
// ---------------------------------------------------------------------------

/// Ground types only, no polymorphism (ForAll/TypeAbst/TypeApp removed).
#[derive(Debug, Clone, PartialEq)]
pub struct MonoGraph {
    pub root: NodeId,
    pub nodes: BTreeMap<NodeId, IrNode>,
    pub edges: Vec<Edge>,
    pub type_env: TypeEnv,
    pub cost: CostBound,
}

// ---------------------------------------------------------------------------
// Stage 2: FirstOrderGraph
// ---------------------------------------------------------------------------

/// No higher-order functions (Lambda/Apply removed).
/// Functions replaced by Procedures with ClosureTags.
#[derive(Debug, Clone, PartialEq)]
pub struct FirstOrderGraph {
    pub root: NodeId,
    pub nodes: BTreeMap<NodeId, IrNode>,
    pub edges: Vec<Edge>,
    pub type_env: TypeEnv,
    pub procedures: Vec<Procedure>,
    pub cost: CostBound,
}

// ---------------------------------------------------------------------------
// Stage 3: PredicatedGraph
// ---------------------------------------------------------------------------

/// No Match nodes; uses MaskGen + Select for predicated execution.
#[derive(Debug, Clone, PartialEq)]
pub struct PredicatedGraph {
    pub root: NodeId,
    pub nodes: BTreeMap<NodeId, IrNode>,
    pub edges: Vec<Edge>,
    pub type_env: TypeEnv,
    pub procedures: Vec<Procedure>,
    pub cost: CostBound,
}

// ---------------------------------------------------------------------------
// Stage 4: LoopGraph
// ---------------------------------------------------------------------------

/// No Fold/LetRec; uses LoopHeader/LoopBody/LoopExit/LoopBackedge.
#[derive(Debug, Clone, PartialEq)]
pub struct LoopGraph {
    pub root: NodeId,
    pub nodes: BTreeMap<NodeId, IrNode>,
    pub edges: Vec<Edge>,
    pub type_env: TypeEnv,
    pub procedures: Vec<Procedure>,
    pub cost: CostBound,
}

// ---------------------------------------------------------------------------
// Stage 5: TrampolineGraph
// ---------------------------------------------------------------------------

/// No Effect nodes; uses EffectYield/EffectResume pairs.
#[derive(Debug, Clone, PartialEq)]
pub struct TrampolineGraph {
    pub root: NodeId,
    pub nodes: BTreeMap<NodeId, IrNode>,
    pub edges: Vec<Edge>,
    pub type_env: TypeEnv,
    pub procedures: Vec<Procedure>,
    pub cost: CostBound,
}

// ---------------------------------------------------------------------------
// Stage 6: PrimitiveGraph
// ---------------------------------------------------------------------------

/// No Neural nodes; concrete compute ops (InlineFma/TiledMatMul/ExternCall).
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveGraph {
    pub root: NodeId,
    pub nodes: BTreeMap<NodeId, IrNode>,
    pub edges: Vec<Edge>,
    pub type_env: TypeEnv,
    pub procedures: Vec<Procedure>,
    pub cost: CostBound,
}

// ---------------------------------------------------------------------------
// Stage 7: LayoutGraph
// ---------------------------------------------------------------------------

/// Every value annotated with physical data layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutGraph {
    pub root: NodeId,
    pub nodes: BTreeMap<NodeId, IrNode>,
    pub edges: Vec<Edge>,
    pub type_env: TypeEnv,
    pub procedures: Vec<Procedure>,
    pub layouts: BTreeMap<NodeId, LayoutAnnotation>,
    pub cost: CostBound,
}

// ---------------------------------------------------------------------------
// Stage 8: MicroOp + MicroOpSequence
// ---------------------------------------------------------------------------

/// A single virtual micro-operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MicroOp {
    /// CLCU opcode (1 byte).
    pub opcode: u8,
    /// Destination virtual register.
    pub dst: VReg,
    /// Source 1 virtual register.
    pub src1: VReg,
    /// Source 2 virtual register.
    pub src2: VReg,
    /// Register width (in bytes; 64 for ZMM).
    pub width: u8,
    /// Modifier flags (mask mode, rounding, etc.).
    pub modifier: u8,
    /// 24-bit immediate value.
    pub immediate: u32,
}

/// Virtual register name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VReg(pub u16);

/// Linear sequence of virtual micro-ops.
#[derive(Debug, Clone, PartialEq)]
pub struct MicroOpSequence {
    pub ops: Vec<MicroOp>,
    /// Map from NodeId to the range of ops that implement it.
    pub node_map: BTreeMap<NodeId, (usize, usize)>,
    /// Total virtual registers used.
    pub vreg_count: u16,
}

// ---------------------------------------------------------------------------
// Stage 9: AllocatedSequence
// ---------------------------------------------------------------------------

/// Physical ZMM register (0-31).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysReg(pub u8);

/// An allocated micro-op with physical registers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocatedOp {
    pub opcode: u8,
    pub dst: PhysReg,
    pub src1: PhysReg,
    pub src2: PhysReg,
    pub width: u8,
    pub modifier: u8,
    pub immediate: u32,
}

/// Micro-ops with physical ZMM register assignments.
#[derive(Debug, Clone, PartialEq)]
pub struct AllocatedSequence {
    pub ops: Vec<AllocatedOp>,
    /// Spill slots used (zmm8-15 range, then arena).
    pub spill_slots: u8,
}

// ---------------------------------------------------------------------------
// Stage 10: CLCUChain
// ---------------------------------------------------------------------------

/// Flags packed into a single byte for a CLCU container header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerFlags {
    /// 2-bit mask source: 00 = none, 01 = register-based.
    pub mask_source: u8,
    /// Whether this is a continuation container.
    pub is_continuation: bool,
    /// Number of micro-ops in the payload (0-8).
    pub op_count: u8,
}

impl ContainerFlags {
    pub fn to_byte(self) -> u8 {
        (self.mask_source & 0x03)
            | (if self.is_continuation { 0x04 } else { 0 })
            | ((self.op_count & 0x0F) << 4)
    }
}

/// 6-byte packed micro-op for a CLCU container payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedMicroOp {
    /// 1-byte opcode.
    pub opcode: u8,
    /// Packed dst(4 bits) | src1(4 bits).
    pub dst_src1: u8,
    /// Packed src2(4 bits) | modifier(4 bits).
    pub src2_mod: u8,
    /// 3-byte immediate (little-endian).
    pub immediate: [u8; 3],
}

impl PackedMicroOp {
    pub fn from_allocated(op: &AllocatedOp) -> Self {
        let imm_bytes = op.immediate.to_le_bytes();
        PackedMicroOp {
            opcode: op.opcode,
            dst_src1: (op.dst.0 & 0x0F) | ((op.src1.0 & 0x0F) << 4),
            src2_mod: (op.src2.0 & 0x0F) | ((op.modifier & 0x0F) << 4),
            immediate: [imm_bytes[0], imm_bytes[1], imm_bytes[2]],
        }
    }

    /// Serialize to 6 bytes.
    pub fn to_bytes(&self) -> [u8; 6] {
        [
            self.opcode,
            self.dst_src1,
            self.src2_mod,
            self.immediate[0],
            self.immediate[1],
            self.immediate[2],
        ]
    }
}

/// 16-byte CLCU container header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerHeader {
    /// Magic (1 byte) + version (1 byte).
    pub magic: u8,
    pub version: u8,
    /// Container index within the chain.
    pub container_index: u8,
    /// Packed flags.
    pub flags: ContainerFlags,
    /// Signed offset to next container (in 64-byte units).
    pub next_container: i32,
    /// Signed offset to prefetch target (in 64-byte units).
    pub prefetch_target: i32,
    /// Mask immediate for conditional continuation.
    pub mask_immediate: u16,
    /// Bitmask of zmm registers live on entry.
    pub zmm_live_in: u8,
    /// Bitmask of zmm registers live on exit.
    pub zmm_live_out: u8,
}

impl ContainerHeader {
    /// Serialize to 16 bytes.
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut buf = [0u8; 16];
        buf[0] = self.magic;
        buf[1] = self.version;
        buf[2] = self.container_index;
        buf[3] = self.flags.to_byte();
        buf[4..8].copy_from_slice(&self.next_container.to_le_bytes());
        buf[8..12].copy_from_slice(&self.prefetch_target.to_le_bytes());
        buf[12..14].copy_from_slice(&self.mask_immediate.to_le_bytes());
        buf[14] = self.zmm_live_in;
        buf[15] = self.zmm_live_out;
        buf
    }
}

/// A single 64-byte CLCU container ready for arena placement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CLCUContainer {
    pub header: ContainerHeader,
    /// Up to 8 packed micro-ops (48 bytes total).
    pub payload: Vec<PackedMicroOp>,
}

impl CLCUContainer {
    /// Serialize to exactly 64 bytes.
    pub fn to_bytes(&self) -> [u8; 64] {
        let mut buf = [0u8; 64];
        buf[0..16].copy_from_slice(&self.header.to_bytes());
        for (i, op) in self.payload.iter().enumerate() {
            let offset = 16 + i * 6;
            if offset + 6 <= 64 {
                buf[offset..offset + 6].copy_from_slice(&op.to_bytes());
            }
        }
        buf
    }
}

/// The final output: a chain of CLCU containers.
#[derive(Debug, Clone, PartialEq)]
pub struct CLCUChain {
    pub containers: Vec<CLCUContainer>,
}

// ---------------------------------------------------------------------------
// CLCU Opcodes (subset for Gen1)
// ---------------------------------------------------------------------------

/// Known CLCU opcode constants.
pub mod opcodes {
    pub const NOP: u8 = 0x00;
    pub const VADD: u8 = 0x01;
    pub const VSUB: u8 = 0x02;
    pub const VMUL: u8 = 0x03;
    pub const VDIV: u8 = 0x04;
    pub const VFMA: u8 = 0x05;
    pub const VCMP_EQ: u8 = 0x06;
    pub const VCMP_LT: u8 = 0x07;
    pub const VCMP_LE: u8 = 0x08;
    pub const VBLEND: u8 = 0x09;
    pub const VBROADCAST: u8 = 0x0A;
    pub const VLOAD: u8 = 0x0B;
    pub const VSTORE: u8 = 0x0C;
    pub const VGATHER: u8 = 0x0D;
    pub const VSCATTER: u8 = 0x0E;
    pub const VMOV: u8 = 0x0F;
    pub const VAND: u8 = 0x10;
    pub const VOR: u8 = 0x11;
    pub const VXOR: u8 = 0x12;
    pub const VNEG: u8 = 0x13;
    pub const VABS: u8 = 0x14;
    pub const VSHL: u8 = 0x15;
    pub const VSHR: u8 = 0x16;
    pub const VMOD: u8 = 0x17;
    pub const VCVT: u8 = 0x18;
    pub const VCALL: u8 = 0x19;
    pub const VRET: u8 = 0x1A;
    pub const VYIELD: u8 = 0x1B;
    pub const VRESUME: u8 = 0x1C;
    pub const VSPILL: u8 = 0x1D;
    pub const VRELOAD: u8 = 0x1E;
    pub const VTAG_READ: u8 = 0x1F;
    pub const VTAG_WRITE: u8 = 0x20;
    pub const VCONST: u8 = 0x21;
    pub const VMIN: u8 = 0x22;
    pub const VMAX: u8 = 0x23;
    pub const VSQRT: u8 = 0x24;
    pub const VRELU: u8 = 0x25;
}
