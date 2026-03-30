use serde::{Deserialize, Serialize};

use crate::fragment::FragmentRef;
use crate::types::{LIAFormula, TypeRef};

// ---------------------------------------------------------------------------
// GuardSpec
// ---------------------------------------------------------------------------

/// Contract for a neural computation node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuardSpec {
    pub input_type: TypeRef,
    pub output_type: TypeRef,
    pub preconditions: Vec<LIAFormula>,
    pub postconditions: Vec<LIAFormula>,
    pub error_bound: ErrorBound,
    pub fallback: Option<FragmentRef>,
}

// ---------------------------------------------------------------------------
// ErrorBound
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorBound {
    Exact,
    Statistical {
        confidence: f64,
        epsilon: f64,
    },
    Classification {
        accuracy: f64,
    },
    Unverified,
}

// ---------------------------------------------------------------------------
// BlobRef
// ---------------------------------------------------------------------------

/// Content-addressed reference to a weight blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlobRef {
    /// BLAKE3 hash of the weight blob contents.
    pub hash: [u8; 32],
    /// Size in bytes.
    pub size: u64,
}

impl Default for BlobRef {
    fn default() -> Self {
        Self {
            hash: [0u8; 32],
            size: 0,
        }
    }
}

impl Default for GuardSpec {
    fn default() -> Self {
        Self {
            input_type: crate::types::TypeId(0),
            output_type: crate::types::TypeId(0),
            preconditions: vec![],
            postconditions: vec![],
            error_bound: ErrorBound::Unverified,
            fallback: None,
        }
    }
}
