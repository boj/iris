//! Serializable evolution state for checkpoint/restore/fork.
//!
//! Evolution state must be serializable so it can be:
//! - Checkpointed to disk for crash recovery
//! - Distributed across machines for parallel evolution
//! - Forked to explore multiple evolutionary trajectories
//!
//! All types used here already derive `Serialize`/`Deserialize`.
//!
//! ## Integrity protection
//!
//! Every checkpoint is integrity-protected with HMAC-BLAKE3.  The MAC is
//! stored in a separate `.mac` sidecar file next to the JSON payload.  On
//! load we verify the MAC before deserializing; a mismatch causes an
//! `InvalidData` error, preventing checkpoint-poisoning attacks.
//!
//! **Key management note**: The HMAC key is currently a compile-time
//! constant (`CHECKPOINT_HMAC_KEY`).  This is strictly better than no
//! authentication, but a production deployment should derive the key from
//! a secret stored outside the binary (environment variable, secret store,
//! HSM, etc.).

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config::EvolutionConfig;
use crate::individual::Individual;
use crate::population::Phase;

// ---------------------------------------------------------------------------
// HMAC key (compile-time constant)
// ---------------------------------------------------------------------------

/// HMAC-BLAKE3 key used for checkpoint integrity.
///
/// **TODO**: Replace with a runtime-configured key from an environment
/// variable or secret store before deploying in production.  The constant
/// provides authentication against tampering by external processes but does
/// not protect against an attacker who can read the binary.
const CHECKPOINT_HMAC_KEY: &[u8] = b"iris-checkpoint-hmac-v1-CHANGE-ME";

/// Compute HMAC-BLAKE3 over `data` using the built-in key.
///
/// BLAKE3 natively supports keyed hashing (256-bit key).  We derive a
/// 32-byte key by hashing the constant with BLAKE3 itself so the key is
/// always exactly the right size.
fn hmac_blake3(data: &[u8]) -> [u8; 32] {
    // Derive a fixed 32-byte key.
    let key_bytes = {
        let h = blake3::hash(CHECKPOINT_HMAC_KEY);
        *h.as_bytes()
    };
    let keyed = blake3::keyed_hash(&key_bytes, data);
    *keyed.as_bytes()
}

/// Hex-encode a 32-byte MAC for storage.
fn mac_to_hex(mac: &[u8; 32]) -> String {
    mac.iter().fold(String::with_capacity(64), |mut s, b| {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
        s
    })
}

/// Decode a 64-char hex string back to 32 bytes.  Returns `None` on error.
fn mac_from_hex(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
        let hi = (chunk[0] as char).to_digit(16)?;
        let lo = (chunk[1] as char).to_digit(16)?;
        out[i] = (hi * 16 + lo) as u8;
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// EvolutionCheckpoint
// ---------------------------------------------------------------------------

/// A snapshot of the full evolution state, suitable for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionCheckpoint {
    /// Current generation number.
    pub generation: u64,
    /// The full population (all demes flattened).
    pub population: Vec<Individual>,
    /// Serialized novelty archive entries (behavior descriptors).
    pub novelty_archive: Vec<Vec<u8>>,
    /// Current evolutionary phase.
    pub phase: Phase,
    /// Best fitness values across all objectives.
    pub best_fitness: [f32; 5],
    /// The evolution configuration used.
    pub config: EvolutionConfig,
    /// Unix timestamp (seconds since epoch) when the checkpoint was created.
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// save / load
// ---------------------------------------------------------------------------

/// Serialize an `EvolutionCheckpoint` to a JSON file with integrity MAC.
///
/// Writes two files:
/// - `<path>` — JSON payload
/// - `<path>.mac` — 64-char hex HMAC-BLAKE3 of the JSON bytes
pub fn save_checkpoint(checkpoint: &EvolutionCheckpoint, path: &str) -> Result<(), io::Error> {
    let json = serde_json::to_string(checkpoint)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Compute MAC over the serialized bytes before writing.
    let mac = hmac_blake3(json.as_bytes());
    let mac_hex = mac_to_hex(&mac);

    // Write payload.
    std::fs::write(Path::new(path), json.as_bytes())?;

    // Write MAC sidecar.
    let mac_path = format!("{}.mac", path);
    std::fs::write(Path::new(&mac_path), mac_hex.as_bytes())?;

    Ok(())
}

/// Deserialize an `EvolutionCheckpoint` from a JSON file, verifying MAC.
///
/// Reads `<path>` and `<path>.mac`.  Returns `InvalidData` if the MAC is
/// absent, malformed, or does not match the payload — preventing checkpoint
/// poisoning.
pub fn load_checkpoint(path: &str) -> Result<EvolutionCheckpoint, io::Error> {
    let json_bytes = std::fs::read(Path::new(path))?;

    // Read MAC sidecar.
    let mac_path = format!("{}.mac", path);
    let mac_hex_bytes = std::fs::read(Path::new(&mac_path)).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("checkpoint MAC sidecar missing or unreadable: {}", e),
        )
    })?;
    let mac_hex = std::str::from_utf8(&mac_hex_bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "checkpoint MAC is not UTF-8"))?
        .trim();

    let stored_mac = mac_from_hex(mac_hex).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "checkpoint MAC is not a valid 64-char hex string",
        )
    })?;

    // Verify MAC.
    let computed_mac = hmac_blake3(&json_bytes);
    if computed_mac != stored_mac {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "checkpoint integrity check failed: MAC mismatch (possible tampering)",
        ));
    }

    serde_json::from_slice(&json_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase::iris_evolve_test_helpers::make_dummy_fragment;

    #[test]
    fn checkpoint_round_trip() {
        let ind = Individual::new(make_dummy_fragment());

        let checkpoint = EvolutionCheckpoint {
            generation: 42,
            population: vec![ind],
            novelty_archive: vec![vec![1, 2, 3], vec![4, 5, 6]],
            phase: Phase::Exploration,
            best_fitness: [0.9, 0.8, 0.7, 0.6, 0.5],
            config: EvolutionConfig::default(),
            timestamp: 1700000000,
        };

        // Serialize to JSON string (no MAC — inline test of the data structure).
        let json = serde_json::to_string(&checkpoint).expect("serialize");

        // Deserialize back.
        let loaded: EvolutionCheckpoint =
            serde_json::from_str(&json).expect("deserialize");

        assert_eq!(loaded.generation, 42);
        assert_eq!(loaded.population.len(), 1);
        assert_eq!(loaded.novelty_archive.len(), 2);
        assert_eq!(loaded.phase, Phase::Exploration);
        assert_eq!(loaded.best_fitness, [0.9, 0.8, 0.7, 0.6, 0.5]);
        assert_eq!(loaded.timestamp, 1700000000);
    }

    #[test]
    fn checkpoint_save_load_file() {
        let ind = Individual::new(make_dummy_fragment());

        let checkpoint = EvolutionCheckpoint {
            generation: 7,
            population: vec![ind],
            novelty_archive: vec![],
            phase: Phase::SteadyState,
            best_fitness: [1.0, 0.0, 0.0, 0.0, 0.0],
            config: EvolutionConfig::default(),
            timestamp: 1700000001,
        };

        let dir = std::env::temp_dir();
        let path = dir
            .join("iris_checkpoint_test.json")
            .to_string_lossy()
            .to_string();

        save_checkpoint(&checkpoint, &path).expect("save");
        let loaded = load_checkpoint(&path).expect("load");

        assert_eq!(loaded.generation, 7);
        assert_eq!(loaded.phase, Phase::SteadyState);
        assert_eq!(loaded.population.len(), 1);

        // Cleanup.
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{}.mac", &path));
    }

    #[test]
    fn checkpoint_tamper_detected() {
        let ind = Individual::new(make_dummy_fragment());

        let checkpoint = EvolutionCheckpoint {
            generation: 99,
            population: vec![ind],
            novelty_archive: vec![],
            phase: Phase::Exploration,
            best_fitness: [0.5; 5],
            config: EvolutionConfig::default(),
            timestamp: 1700000002,
        };

        let dir = std::env::temp_dir();
        let path = dir
            .join("iris_checkpoint_tamper_test.json")
            .to_string_lossy()
            .to_string();
        let mac_path = format!("{}.mac", &path);

        save_checkpoint(&checkpoint, &path).expect("save");

        // Tamper with the payload.
        let mut data = std::fs::read(&path).unwrap();
        if let Some(b) = data.first_mut() {
            *b = b.wrapping_add(1);
        }
        std::fs::write(&path, &data).unwrap();

        // Load should fail with MAC mismatch.
        let result = load_checkpoint(&path);
        assert!(result.is_err(), "tamper should be detected");
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("MAC mismatch"));

        // Cleanup.
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&mac_path);
    }
}
