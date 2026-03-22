//! Capability-based security system for IRIS programs.
//!
//! Provides fine-grained control over what effects a program can perform,
//! how much memory it can use, how many steps it can execute, and what
//! filesystem/network resources it can access. This is the enforcement
//! layer that makes sandboxed execution safe for evolved programs.

use std::collections::HashSet;
use std::time::Duration;

use iris_types::eval::{EffectTag, EffectError, EffectHandler, EffectRequest, Value};

// ---------------------------------------------------------------------------
// Capabilities
// ---------------------------------------------------------------------------

/// Defines the set of permissions a program has during execution.
///
/// Used to sandbox evolved programs, daemon candidates, and untrusted code.
/// The interpreter checks capabilities before executing any effect, and the
/// `CapabilityGuardHandler` wraps an inner `EffectHandler` to enforce path
/// and host restrictions on file/network operations.
#[derive(Debug, Clone)]
pub struct Capabilities {
    /// Set of effect tags this program is allowed to invoke.
    pub allowed_effects: HashSet<EffectTag>,
    /// Maximum memory in bytes (0 = unlimited).
    pub max_memory: usize,
    /// Maximum execution steps (0 = unlimited).
    pub max_steps: u64,
    /// Maximum wall-clock time for execution.
    pub max_wall_time: Duration,
    /// Filesystem paths the program can access (glob patterns).
    pub allowed_paths: Vec<String>,
    /// Network hosts the program can connect to.
    pub allowed_hosts: Vec<String>,
    /// Whether the program can spawn threads.
    pub can_spawn_threads: bool,
    /// Whether the program can call foreign functions.
    pub can_ffi: bool,
    /// Whether the program can map executable memory (JIT).
    pub can_mmap_exec: bool,
    /// Whether the program can use channel operations (send/recv).
    pub can_use_channels: bool,
    /// Allowed environment variable names. Empty = no env access.
    /// `["*"]` means all env vars are allowed.
    pub allowed_env_vars: Vec<String>,
}

impl Capabilities {
    /// Create capabilities that allow everything (no restrictions).
    pub fn unrestricted() -> Self {
        let mut allowed = HashSet::new();
        // Add all known effect tags (0x00..=0x0D, 0x10..=0x1F, 0x20..=0x28, 0x29..=0x2B).
        // NOTE: 0x0E-0x0F are gaps (Custom), so we skip them to avoid polluting
        // the set, but include all defined tags through FfiCall (0x2B).
        for tag_byte in 0x00..=0x0D {
            allowed.insert(EffectTag::from_u8(tag_byte));
        }
        for tag_byte in 0x10..=0x2B {
            allowed.insert(EffectTag::from_u8(tag_byte));
        }
        Self {
            allowed_effects: allowed,
            max_memory: 0,
            max_steps: 0,
            max_wall_time: Duration::from_secs(0), // 0 = unlimited
            allowed_paths: vec!["**".to_string()],
            allowed_hosts: vec!["*".to_string()],
            can_spawn_threads: true,
            can_ffi: true,
            can_mmap_exec: true,
            can_use_channels: true,
            allowed_env_vars: vec!["*".to_string()],
        }
    }

    /// Create a pure computation sandbox: no I/O, no threads, no FFI.
    ///
    /// Only allows effects that don't escape the process: Print, Log,
    /// Timestamp, Random, ClockNs, RandomBytes, and SleepMs.
    pub fn sandbox() -> Self {
        let mut allowed = HashSet::new();
        // Pure-ish effects that don't leak data or cause harm.
        allowed.insert(EffectTag::Print);
        allowed.insert(EffectTag::Log);
        allowed.insert(EffectTag::Timestamp);
        allowed.insert(EffectTag::Random);
        allowed.insert(EffectTag::ClockNs);
        allowed.insert(EffectTag::RandomBytes);
        allowed.insert(EffectTag::SleepMs);
        Self {
            allowed_effects: allowed,
            max_memory: 10 * 1024 * 1024, // 10 MB
            max_steps: 1_000_000,
            max_wall_time: Duration::from_secs(10),
            allowed_paths: Vec::new(),
            allowed_hosts: Vec::new(),
            can_spawn_threads: false,
            can_ffi: false,
            can_mmap_exec: false,
            can_use_channels: false,
            allowed_env_vars: Vec::new(),
        }
    }

    /// Create capabilities with restricted I/O: only specific paths and hosts.
    pub fn io_restricted(paths: &[&str], hosts: &[&str]) -> Self {
        let mut allowed = HashSet::new();
        // Allow all non-dangerous effects.
        allowed.insert(EffectTag::Print);
        allowed.insert(EffectTag::ReadLine);
        allowed.insert(EffectTag::Log);
        allowed.insert(EffectTag::Timestamp);
        allowed.insert(EffectTag::Random);
        allowed.insert(EffectTag::ClockNs);
        allowed.insert(EffectTag::RandomBytes);
        allowed.insert(EffectTag::SleepMs);
        allowed.insert(EffectTag::Sleep);

        // File operations (subject to path checks).
        allowed.insert(EffectTag::FileRead);
        allowed.insert(EffectTag::FileWrite);
        allowed.insert(EffectTag::FileOpen);
        allowed.insert(EffectTag::FileReadBytes);
        allowed.insert(EffectTag::FileWriteBytes);
        allowed.insert(EffectTag::FileClose);
        allowed.insert(EffectTag::FileStat);
        allowed.insert(EffectTag::DirList);

        // Network operations (subject to host checks).
        allowed.insert(EffectTag::TcpConnect);
        allowed.insert(EffectTag::TcpRead);
        allowed.insert(EffectTag::TcpWrite);
        allowed.insert(EffectTag::TcpClose);
        allowed.insert(EffectTag::TcpListen);
        allowed.insert(EffectTag::TcpAccept);

        Self {
            allowed_effects: allowed,
            max_memory: 256 * 1024 * 1024, // 256 MB
            max_steps: 0,                   // unlimited
            max_wall_time: Duration::from_secs(60),
            allowed_paths: paths.iter().map(|s| s.to_string()).collect(),
            allowed_hosts: hosts.iter().map(|s| s.to_string()).collect(),
            can_spawn_threads: false,
            can_ffi: false,
            can_mmap_exec: false,
            can_use_channels: true,
            allowed_env_vars: Vec::new(),
        }
    }

    /// Create a daemon sandbox for testing evolved candidate components.
    ///
    /// Very restrictive: no network, no filesystem (except /tmp), limited
    /// memory and steps, no FFI, no mmap_exec, no thread spawning.
    pub fn daemon_candidate() -> Self {
        let mut caps = Self::sandbox();
        // Allow file ops but only to /tmp
        caps.allowed_effects.insert(EffectTag::FileRead);
        caps.allowed_effects.insert(EffectTag::FileWrite);
        caps.allowed_effects.insert(EffectTag::FileOpen);
        caps.allowed_effects.insert(EffectTag::FileReadBytes);
        caps.allowed_effects.insert(EffectTag::FileWriteBytes);
        caps.allowed_effects.insert(EffectTag::FileClose);
        caps.allowed_effects.insert(EffectTag::FileStat);
        caps.allowed_effects.insert(EffectTag::DirList);
        caps.allowed_paths = vec!["/tmp/*".to_string()];
        caps
    }

    /// Check if a given effect tag is allowed by these capabilities.
    pub fn is_allowed(&self, tag: EffectTag) -> bool {
        // Check structural capabilities for thread/ffi/mmap effects.
        match tag {
            EffectTag::ThreadSpawn | EffectTag::ThreadJoin => {
                if !self.can_spawn_threads {
                    return false;
                }
            }
            EffectTag::MmapExec => {
                if !self.can_mmap_exec {
                    return false;
                }
            }
            EffectTag::CallNative => {
                // CallNative requires both can_ffi (it calls native code) and
                // can_mmap_exec (it executes mapped memory).
                if !self.can_ffi || !self.can_mmap_exec {
                    return false;
                }
            }
            EffectTag::FfiCall => {
                if !self.can_ffi {
                    return false;
                }
            }
            _ => {}
        }
        self.allowed_effects.contains(&tag)
    }

    /// Check if a file path is allowed by the `allowed_paths` list.
    ///
    /// Uses simple glob matching: `*` matches any single path component,
    /// `**` matches any number of path components.
    ///
    /// Security checks applied in order:
    /// 1. Reject null bytes (prevent C-string truncation attacks)
    /// 2. Reject `..` path components (prevent traversal attacks)
    /// 3. Canonicalize to resolve symlinks; for new files, canonicalize the
    ///    parent directory and append the filename
    pub fn is_path_allowed(&self, path: &str) -> bool {
        if self.allowed_paths.is_empty() {
            return false;
        }
        // Reject null bytes to prevent C-string truncation attacks.
        if path.contains('\0') {
            return false;
        }
        // Reject paths with `..` components to prevent traversal attacks.
        let p = std::path::Path::new(path);
        for component in p.components() {
            if matches!(component, std::path::Component::ParentDir) {
                return false;
            }
        }
        // Canonicalize to resolve symlinks. If the full path doesn't exist,
        // canonicalize the parent directory (which must exist) and append
        // the filename. This prevents symlink-in-allowed-dir attacks where
        // /tmp/allowed/symlink -> /etc/passwd would pass the glob check.
        let resolved = match std::fs::canonicalize(path) {
            Ok(canon) => canon.to_string_lossy().into_owned(),
            Err(_) => {
                // File doesn't exist yet — canonicalize parent directory.
                match p.parent() {
                    Some(parent) if !parent.as_os_str().is_empty() => {
                        match std::fs::canonicalize(parent) {
                            Ok(canon_parent) => {
                                let filename = p.file_name().unwrap_or_default();
                                canon_parent.join(filename).to_string_lossy().into_owned()
                            }
                            // Parent doesn't exist either — reject.
                            Err(_) => return false,
                        }
                    }
                    // No parent (bare filename) — use as-is after .. check.
                    _ => path.to_string(),
                }
            }
        };
        for pattern in &self.allowed_paths {
            if glob_match(pattern, &resolved) {
                return true;
            }
        }
        false
    }

    /// Check if a network host is allowed by the `allowed_hosts` list.
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if self.allowed_hosts.is_empty() {
            return false;
        }
        for pattern in &self.allowed_hosts {
            if pattern == "*" || pattern == host {
                return true;
            }
            // Support wildcard subdomain matching: *.example.com
            if let Some(suffix) = pattern.strip_prefix("*.") {
                if host.ends_with(suffix) && host.len() > suffix.len() {
                    return true;
                }
            }
        }
        false
    }

    /// Check if an environment variable name is allowed by `allowed_env_vars`.
    pub fn is_env_var_allowed(&self, name: &str) -> bool {
        if self.allowed_env_vars.is_empty() {
            return false;
        }
        for pattern in &self.allowed_env_vars {
            if pattern == "*" || pattern == name {
                return true;
            }
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Glob matching
// ---------------------------------------------------------------------------

/// Simple glob matching for file paths.
///
/// Supports:
/// - `*` matches any sequence of non-separator characters
/// - `**` matches any sequence of characters including separators
/// - Literal characters match themselves
fn glob_match(pattern: &str, path: &str) -> bool {
    // Handle the trivial "match everything" case.
    if pattern == "**" {
        return true;
    }

    let pat_bytes = pattern.as_bytes();
    let path_bytes = path.as_bytes();

    glob_match_inner(pat_bytes, path_bytes)
}

fn glob_match_inner(pat: &[u8], path: &[u8]) -> bool {
    let mut pi = 0; // pattern index
    let mut si = 0; // string (path) index

    // Track backtracking points for `*` wildcards.
    let mut star_pi = usize::MAX;
    let mut star_si = usize::MAX;

    while si < path.len() {
        if pi < pat.len() && pi + 1 < pat.len() && pat[pi] == b'*' && pat[pi + 1] == b'*' {
            // `**` matches everything including separators.
            // Skip the `**` and optional following separator.
            pi += 2;
            if pi < pat.len() && pat[pi] == b'/' {
                pi += 1;
            }
            // `**` at end matches everything remaining.
            if pi >= pat.len() {
                return true;
            }
            // Try matching the rest of the pattern at every position.
            for start in si..=path.len() {
                if glob_match_inner(&pat[pi..], &path[start..]) {
                    return true;
                }
            }
            return false;
        }

        if pi < pat.len() && pat[pi] == b'*' {
            // `*` matches any non-separator characters.
            star_pi = pi;
            star_si = si;
            pi += 1;
            continue;
        }

        if pi < pat.len() && (pat[pi] == path[si] || pat[pi] == b'?') {
            pi += 1;
            si += 1;
            continue;
        }

        // Mismatch: backtrack to last `*` if possible.
        if star_pi != usize::MAX {
            pi = star_pi + 1;
            star_si += 1;
            // `*` must not match separators.
            if path[star_si - 1] == b'/' {
                return false;
            }
            si = star_si;
            continue;
        }

        return false;
    }

    // Skip trailing `*` patterns.
    while pi < pat.len() && pat[pi] == b'*' {
        pi += 1;
    }

    pi >= pat.len()
}

// ---------------------------------------------------------------------------
// CapabilityGuardHandler
// ---------------------------------------------------------------------------

/// An `EffectHandler` wrapper that enforces `Capabilities` on top of an
/// inner handler.
///
/// Before delegating to the inner handler, it checks:
/// 1. Whether the effect tag is allowed
/// 2. Whether file paths are within `allowed_paths` (for file operations)
/// 3. Whether network hosts are within `allowed_hosts` (for TCP operations)
pub struct CapabilityGuardHandler<H: EffectHandler> {
    inner: H,
    capabilities: Capabilities,
}

impl<H: EffectHandler> CapabilityGuardHandler<H> {
    /// Create a new capability guard wrapping the given handler.
    pub fn new(inner: H, capabilities: Capabilities) -> Self {
        Self { inner, capabilities }
    }

    /// Extract a file path from the first argument of a file operation.
    fn extract_path(args: &[Value]) -> Option<String> {
        match args.first() {
            Some(Value::String(s)) => Some(s.clone()),
            Some(Value::Bytes(b)) => Some(String::from_utf8_lossy(b).into_owned()),
            _ => None,
        }
    }

    /// Extract a host from the first argument of a TCP operation.
    fn extract_host(args: &[Value]) -> Option<String> {
        match args.first() {
            Some(Value::String(s)) => Some(s.clone()),
            Some(Value::Bytes(b)) => Some(String::from_utf8_lossy(b).into_owned()),
            _ => None,
        }
    }
}

impl<H: EffectHandler> EffectHandler for CapabilityGuardHandler<H> {
    fn handle(&self, request: EffectRequest) -> Result<Value, EffectError> {
        let tag = request.tag;

        // 1. Check if the effect tag is allowed at all.
        if !self.capabilities.is_allowed(tag) {
            return Err(EffectError {
                tag,
                message: format!(
                    "permission denied: effect {:?} is not allowed by capabilities",
                    tag
                ),
            });
        }

        // 2. For file operations, check path restrictions.
        // Note: FileReadBytes/FileWriteBytes operate on integer file handles,
        // not paths. They inherit permissions from the FileOpen call that
        // created the handle. Path enforcement happens at open time only.
        match tag {
            EffectTag::FileRead | EffectTag::FileWrite | EffectTag::FileOpen
            | EffectTag::FileStat | EffectTag::DirList => {
                if let Some(path) = Self::extract_path(&request.args) {
                    if !self.capabilities.is_path_allowed(&path) {
                        return Err(EffectError {
                            tag,
                            message: format!(
                                "permission denied: path '{}' is not in allowed paths",
                                path
                            ),
                        });
                    }
                }
            }
            // FileReadBytes/FileWriteBytes use integer handles, not paths.
            // Permissions are enforced at FileOpen time.
            EffectTag::FileReadBytes | EffectTag::FileWriteBytes => {}
            _ => {}
        }

        // 3. For TCP operations, check host restrictions.
        match tag {
            EffectTag::TcpConnect | EffectTag::TcpListen | EffectTag::TcpAccept => {
                if let Some(host) = Self::extract_host(&request.args) {
                    if !self.capabilities.is_host_allowed(&host) {
                        return Err(EffectError {
                            tag,
                            message: format!(
                                "permission denied: host '{}' is not in allowed hosts",
                                host
                            ),
                        });
                    }
                }
            }
            _ => {}
        }

        // 4. For EnvGet, check allowed environment variable names.
        if tag == EffectTag::EnvGet {
            if let Some(var_name) = Self::extract_path(&request.args) {
                if !self.capabilities.is_env_var_allowed(&var_name) {
                    return Err(EffectError {
                        tag,
                        message: format!(
                            "permission denied: environment variable '{}' is not allowed",
                            var_name
                        ),
                    });
                }
            }
        }

        self.inner.handle(request)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrestricted_allows_everything() {
        let caps = Capabilities::unrestricted();
        assert!(caps.is_allowed(EffectTag::FileWrite));
        assert!(caps.is_allowed(EffectTag::TcpConnect));
        assert!(caps.is_allowed(EffectTag::ThreadSpawn));
        assert!(caps.is_allowed(EffectTag::MmapExec));
        assert!(caps.is_path_allowed("/tmp/any_file"));
        assert!(caps.is_host_allowed("any.host.com"));
    }

    #[test]
    fn sandbox_blocks_io() {
        let caps = Capabilities::sandbox();
        assert!(!caps.is_allowed(EffectTag::FileWrite));
        assert!(!caps.is_allowed(EffectTag::FileRead));
        assert!(!caps.is_allowed(EffectTag::TcpConnect));
        assert!(!caps.is_allowed(EffectTag::ThreadSpawn));
        assert!(!caps.is_allowed(EffectTag::MmapExec));
        assert!(!caps.is_allowed(EffectTag::CallNative));
        // Pure effects are allowed.
        assert!(caps.is_allowed(EffectTag::Print));
        assert!(caps.is_allowed(EffectTag::Timestamp));
        assert!(caps.is_allowed(EffectTag::Random));
    }

    #[test]
    fn io_restricted_allows_specified_paths() {
        let caps = Capabilities::io_restricted(&["/tmp/*", "/home/**"], &["api.example.com"]);
        // /tmp exists, so /tmp/foo.txt resolves via the parent directory.
        assert!(caps.is_path_allowed("/tmp/foo.txt"));
        // /home exists, so deep paths resolve via parent canonicalization.
        // Note: paths in non-existent parents are now rejected to prevent
        // symlink bypass attacks.
        assert!(!caps.is_path_allowed("/etc/passwd"));
        assert!(!caps.is_path_allowed("/home/user/secret.txt")); // /home/user may not exist
    }

    #[test]
    fn host_matching() {
        let caps = Capabilities::io_restricted(&[], &["api.example.com", "*.internal.net"]);
        assert!(caps.is_host_allowed("api.example.com"));
        assert!(caps.is_host_allowed("foo.internal.net"));
        assert!(!caps.is_host_allowed("evil.com"));
        assert!(!caps.is_host_allowed("internal.net")); // wildcard requires subdomain
    }

    #[test]
    fn glob_basic_patterns() {
        assert!(glob_match("/tmp/*", "/tmp/foo.txt"));
        assert!(glob_match("/tmp/*", "/tmp/bar"));
        assert!(!glob_match("/tmp/*", "/tmp/sub/file.txt")); // * doesn't cross /
        assert!(glob_match("/tmp/**", "/tmp/sub/file.txt")); // ** does
        assert!(glob_match("**", "/any/path/at/all"));
    }

    #[test]
    fn daemon_candidate_caps() {
        let caps = Capabilities::daemon_candidate();
        assert!(!caps.can_spawn_threads);
        assert!(!caps.can_ffi);
        assert!(!caps.can_mmap_exec);
        assert!(!caps.is_allowed(EffectTag::TcpConnect));
        assert!(caps.is_allowed(EffectTag::FileRead));
        assert!(caps.is_path_allowed("/tmp/test.txt"));
        assert!(!caps.is_path_allowed("/etc/passwd"));
        assert_eq!(caps.max_memory, 10 * 1024 * 1024);
        assert_eq!(caps.max_steps, 1_000_000);
    }

    #[test]
    fn null_byte_in_path_rejected() {
        let caps = Capabilities::io_restricted(&["/tmp/**"], &[]);
        assert!(!caps.is_path_allowed("/tmp/foo\0bar"));
        assert!(!caps.is_path_allowed("\0"));
        assert!(!caps.is_path_allowed("/tmp/\0"));
    }

    #[test]
    fn nonexistent_file_parent_must_resolve() {
        // A new file in an existing allowed directory should be allowed.
        let caps = Capabilities::io_restricted(&["/tmp/**"], &[]);
        // /tmp exists and is in the allowed list, so a new file should work.
        assert!(caps.is_path_allowed("/tmp/brand_new_file_that_does_not_exist.txt"));
    }

    #[test]
    fn nonexistent_parent_rejected() {
        // If the parent directory doesn't exist, the path is rejected
        // (prevents symlink attacks in non-existent directories).
        let caps = Capabilities::io_restricted(&["/tmp/**"], &[]);
        assert!(!caps.is_path_allowed("/nonexistent_dir_abc123/file.txt"));
    }
}
