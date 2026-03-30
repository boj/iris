//! Persistent fragment cache for cross-generation self-improvement.
//!
//! Stores evolved fragments on disk keyed by BLAKE3 FragmentId.
//! A manifest maps (source_file, function_name) to the current best
//! FragmentId, enabling improved versions to persist across runs.
//!
//! Cache layout:
//!   $IRIS_HOME/fragments/          (or ~/.iris/fragments/)
//!     manifest.json                name → FragmentId hex
//!     {hex16}.frag                 wire-format Fragment files
//!
//! The cache is content-addressed: identical programs share one .frag file.
//! The manifest tracks which version is "current" for each named function.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use iris_types::fragment::{Fragment, FragmentId};
use iris_types::hash::compute_fragment_id;
use iris_types::wire;

/// Resolve the fragment cache directory.
/// Priority: $IRIS_HOME/fragments > ~/.iris/fragments > .iris/fragments
pub fn cache_dir() -> PathBuf {
    if let Ok(home) = std::env::var("IRIS_HOME") {
        PathBuf::from(home).join("fragments")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".iris").join("fragments")
    } else {
        PathBuf::from(".iris").join("fragments")
    }
}

/// Manifest entry: tracks the current best fragment for a named function.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ManifestEntry {
    /// Full hex FragmentId of the current best version.
    fragment_id: String,
    /// Generation number (increments each time a better version is found).
    generation: u64,
    /// Unix timestamp of the last improvement.
    #[serde(default)]
    improved_at: u64,
}

type Manifest = BTreeMap<String, ManifestEntry>;

fn manifest_path(dir: &Path) -> PathBuf {
    dir.join("manifest.json")
}

fn frag_path(dir: &Path, hex_id: &str) -> PathBuf {
    dir.join(format!("{}.frag", &hex_id[..16.min(hex_id.len())]))
}

/// Format a FragmentId as a hex string.
pub fn fragment_id_hex(id: &FragmentId) -> String {
    id.0.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_id(id: &FragmentId) -> String {
    fragment_id_hex(id)
}

fn load_manifest(dir: &Path) -> Manifest {
    let path = manifest_path(dir);
    match std::fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => BTreeMap::new(),
    }
}

fn save_manifest(dir: &Path, manifest: &Manifest) -> Result<(), String> {
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("failed to create cache dir: {}", e))?;
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("failed to serialize manifest: {}", e))?;
    // Atomic write: tmp + rename
    let tmp = manifest_path(dir).with_extension("json.tmp");
    std::fs::write(&tmp, &json)
        .map_err(|e| format!("failed to write manifest: {}", e))?;
    std::fs::rename(&tmp, manifest_path(dir))
        .map_err(|e| format!("failed to rename manifest: {}", e))?;
    Ok(())
}

/// Save a fragment to the cache and update the manifest.
/// Returns the hex FragmentId.
pub fn save_fragment(dir: &Path, name: &str, fragment: &Fragment) -> Result<String, String> {
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("failed to create cache dir: {}", e))?;

    let hex = hex_id(&fragment.id);
    let path = frag_path(dir, &hex);

    // Write wire-format fragment
    let bytes = wire::serialize_fragment(fragment);
    std::fs::write(&path, &bytes)
        .map_err(|e| format!("failed to write fragment: {}", e))?;

    // Update manifest
    let mut manifest = load_manifest(dir);
    let next_gen = manifest.get(name).map_or(1, |e| e.generation + 1);
    manifest.insert(name.to_string(), ManifestEntry {
        fragment_id: hex.clone(),
        generation: next_gen,
        improved_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    });
    save_manifest(dir, &manifest)?;

    Ok(hex)
}

/// Load the cached fragment for a named function, if one exists.
/// Verifies BLAKE3 integrity on load.
pub fn load_fragment(dir: &Path, name: &str) -> Option<Fragment> {
    let manifest = load_manifest(dir);
    let entry = manifest.get(name)?;
    let path = frag_path(dir, &entry.fragment_id);
    let bytes = std::fs::read(&path).ok()?;
    let fragment = wire::deserialize_fragment(&bytes).ok()?;

    // Verify BLAKE3 integrity
    let loaded_id = hex_id(&compute_fragment_id(&fragment));
    if loaded_id != entry.fragment_id {
        eprintln!("[cache] BLAKE3 mismatch for '{}': expected {}, got {}",
            name, &entry.fragment_id[..16], &loaded_id[..16]);
        return None;
    }

    Some(fragment)
}

/// Get the generation number for a named function (0 if never improved).
pub fn generation(dir: &Path, name: &str) -> u64 {
    let manifest = load_manifest(dir);
    manifest.get(name).map_or(0, |e| e.generation)
}

/// List all cached function names with their generation and FragmentId.
pub fn list_cached(dir: &Path) -> Vec<(String, u64, String)> {
    let manifest = load_manifest(dir);
    manifest.into_iter()
        .map(|(name, entry)| (name, entry.generation, entry.fragment_id))
        .collect()
}

/// Remove a cached fragment and its manifest entry.
pub fn remove_fragment(dir: &Path, name: &str) -> bool {
    let mut manifest = load_manifest(dir);
    if let Some(entry) = manifest.remove(name) {
        let path = frag_path(dir, &entry.fragment_id);
        let _ = std::fs::remove_file(path);
        let _ = save_manifest(dir, &manifest);
        true
    } else {
        false
    }
}

/// Clear the entire cache.
pub fn clear_cache(dir: &Path) {
    let _ = std::fs::remove_file(manifest_path(dir));
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "frag") {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_types::fragment::{Boundary, FragmentContracts, FragmentMeta};
    use iris_types::graph::*;
    use iris_types::hash::SemanticHash;
    use iris_types::types::TypeEnv;
    use std::collections::BTreeMap as BTM;
    use std::collections::HashMap;

    fn make_fragment(name: &str, lit_val: i64) -> Fragment {
        let node = Node {
            id: NodeId(lit_val as u64),
            kind: NodeKind::Lit,
            type_sig: iris_types::types::TypeId(0),
            cost: iris_types::cost::CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0x00,
                value: lit_val.to_le_bytes().to_vec(),
            },
        };
        let graph = SemanticGraph {
            root: NodeId(lit_val as u64),
            nodes: {
                let mut m = HashMap::new();
                m.insert(NodeId(lit_val as u64), node);
                m
            },
            edges: vec![],
            type_env: TypeEnv { types: BTM::new() },
            cost: iris_types::cost::CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };
        let mut frag = Fragment {
            id: FragmentId([0; 32]),
            graph,
            boundary: Boundary { inputs: vec![], outputs: vec![] },
            type_env: TypeEnv { types: BTM::new() },
            imports: vec![],
            metadata: FragmentMeta {
                name: Some(name.to_string()),
                created_at: 0,
                generation: 0,
                lineage_hash: 0,
            },
            proof: None,
            contracts: FragmentContracts::default(),
        };
        frag.id = compute_fragment_id(&frag);
        frag
    }

    #[test]
    fn round_trip() {
        let dir = std::env::temp_dir().join("iris-cache-test-rt");
        let _ = std::fs::remove_dir_all(&dir);

        let frag = make_fragment("test_fn", 42);
        let hex = save_fragment(&dir, "test_fn", &frag).unwrap();
        assert!(!hex.is_empty());

        let loaded = load_fragment(&dir, "test_fn").unwrap();
        assert_eq!(frag.id, loaded.id);
        assert_eq!(generation(&dir, "test_fn"), 1);

        // Save again → generation increments
        let frag2 = make_fragment("test_fn", 99);
        save_fragment(&dir, "test_fn", &frag2).unwrap();
        assert_eq!(generation(&dir, "test_fn"), 2);

        let loaded2 = load_fragment(&dir, "test_fn").unwrap();
        assert_eq!(frag2.id, loaded2.id);

        // Old fragment file still exists (content-addressed, not deleted)
        let old_path = frag_path(&dir, &hex);
        assert!(old_path.exists(), "old fragment should persist on disk");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_returns_none() {
        let dir = std::env::temp_dir().join("iris-cache-test-miss");
        let _ = std::fs::remove_dir_all(&dir);

        assert!(load_fragment(&dir, "nonexistent").is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_and_clear() {
        let dir = std::env::temp_dir().join("iris-cache-test-list");
        let _ = std::fs::remove_dir_all(&dir);

        save_fragment(&dir, "fn_a", &make_fragment("fn_a", 1)).unwrap();
        save_fragment(&dir, "fn_b", &make_fragment("fn_b", 2)).unwrap();

        let cached = list_cached(&dir);
        assert_eq!(cached.len(), 2);

        clear_cache(&dir);
        assert!(list_cached(&dir).is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
