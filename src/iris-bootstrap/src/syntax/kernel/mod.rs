//! `iris-kernel`: The LCF-style proof kernel for the IRIS system.
//!
//! This is the ONLY trusted component in IRIS. It is small, correct, and
//! auditable. The `Theorem` type can only be constructed by the `Kernel`
//! methods, enforcing logical soundness at the type-system level.
//!
//! ## Lean 4 Formalization
//!
//! The kernel inference rules have been formalized in Lean 4 at
//! `lean/IrisKernel/`. When the `lean-kernel` feature is enabled,
//! the Rust kernel delegates to the Lean-compiled implementation via
//! FFI, so the running code IS the formally proven code. Proof hashing
//! remains in Rust (audit trail, not soundness).
//!
//! - `lean/IrisKernel/Types.lean` — Core types (mirrors Rust types)
//! - `lean/IrisKernel/Rules.lean` — Derivation inductive (20 rules)
//! - `lean/IrisKernel/Kernel.lean` — Executable kernel functions
//! - `lean/IrisKernel/KernelCorrectness.lean` — Correctness proofs
//! - `lean/IrisKernel/FFI.lean` — C-callable exports
//!
//! Feature flags:
//! - `lean-ffi`: Links with Lean runtime (requires Lean toolchain)
//! - `lean-kernel`: Delegates kernel rules to Lean (requires lean-ffi)
#![allow(unused_imports)]

pub mod checker;
pub mod cost_checker;
pub mod error;
pub mod kernel;
#[allow(unsafe_code)]
pub mod lean_bridge;
pub mod lia_solver;
pub mod property_test;
pub mod theorem;
pub mod zk;

// Re-export key types for convenience.
pub use checker::{
    type_check, type_check_graded, diagnose,
    CostWarning, MutationHint, ProofFailureDiagnosis, VerificationReport,
};
pub use error::{CheckError, KernelError};
pub use kernel::Kernel;
pub use theorem::{Binding, Context, Judgment, Theorem};
pub use zk::{
    generate_zk_proof, verify_zk_proof, prove_program, verify_listing,
    ZkError, ZkProof, ZkPublicInputs, ZkPrivateWitness,
    MarketListing, MarketVerification,
};
