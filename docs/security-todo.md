# Security TODO

Status of vulnerabilities from the 2026-03-24 security audits (GPT, Opus, comprehensive).

## All Issues Resolved

### ~~HIGH: MmapExec + CallNative shellcode path~~ - RESOLVED (2026-03-25)
- **Resolution:** `effects.rs` deleted in crate consolidation. No MmapExec/CallNative implementation exists. Effects dispatch through EffectHandler trait; any future handler implementing these must validate function pointers via a handle registry.

### ~~MEDIUM: Symlink bypass for new files~~ - FIXED (2026-03-25)
- **Location:** `src/iris-exec/src/capabilities.rs`
- **Fix:** `is_path_allowed()` now canonicalizes the parent directory when the full path doesn't exist. If the parent can't be canonicalized, the path is rejected. This prevents symlink chains inside allowed directories from escaping.

### ~~MEDIUM: FileReadBytes/FileWriteBytes path check bypass~~ - FIXED (2026-03-25)
- **Location:** `src/iris-exec/src/capabilities.rs`
- **Fix:** Removed `FileReadBytes`/`FileWriteBytes` from path-check match arm. These operations use integer file handles, not paths. Path enforcement is performed at `FileOpen` time; byte operations inherit the handle's permissions.

### ~~MEDIUM: JIT compiled code has no step limits~~ - RESOLVED (2026-03-25)
- **Resolution:** `jit.rs` deleted in crate consolidation. No JIT compiler exists. All execution goes through bootstrap's step-counted tree-walking evaluator.

### ~~LOW: No explicit null-byte check in paths~~ - FIXED (2026-03-25)
- **Location:** `src/iris-exec/src/capabilities.rs`
- **Fix:** Added `if path.contains('\0') { return false; }` at the top of `is_path_allowed`. Prevents C-string truncation attacks.

### ~~LOW: VM bytecode cache key collision~~ - RESOLVED (2026-03-25)
- **Resolution:** `vm.rs` deleted in crate consolidation. No VM bytecode cache exists.

## Previously Fixed (2026-03-24)

- Capability propagation to sub-contexts (4 sandbox escape paths: graph_eval, eval_ref, par_eval, spawn)
- FfiCall arbitrary dlopen/dlsym → hardcoded allowlist (libm/libc math only)
- `can_ffi` check in `is_allowed()`
- `unrestricted()` range covers FfiCall (0x2B)
- `TcpListen`/`TcpAccept` host checking
- Path traversal `..` bypass → rejected via `Component::ParentDir` + `canonicalize`
- `let rec` parser gap

## Architectural Note

`CapabilityGuardHandler` in `capabilities.rs` is well-implemented but currently not wired into the default execution path. The bootstrap evaluator dispatches effects to an optional `EffectHandler`; callers must wrap their handler with `CapabilityGuardHandler::new(inner, caps)` to enforce sandboxing. The daemon (`self_improving_daemon.rs`) and any production effect handler should use this wrapper.
