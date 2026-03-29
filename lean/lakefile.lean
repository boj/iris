import Lake
open Lake DSL

package «iris-kernel» where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]
  -- Prefer static linking for FFI integration with Rust
  preferReleaseBuild := true

@[default_target]
lean_lib «IrisKernel» where
  srcDir := "."
  roots := #[`IrisKernel]

-- Build a static library containing the @[export]-ed FFI functions.
-- After `lake build`, the .a file will be in .lake/build/lib/.
-- Rust links against this via its build.rs.
lean_exe «iris-kernel-ffi-check» where
  root := `IrisKernel
  -- This exe target forces all @[export] symbols into the link.
  -- We don't actually run it — we just need the compiled objects.

-- IPC server binary: reads kernel requests from stdin, writes results to stdout.
-- Used by the Rust IPC bridge (lean_bridge.rs) instead of linking the Lean runtime.
lean_exe «iris-kernel-server» where
  root := `IrisKernelServer
