fn main() {
    if std::env::var("CARGO_FEATURE_LEAN_FFI").is_ok() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let lean_dir = format!("{}/../../lean", manifest_dir);
        let server_bin = format!("{}/.lake/build/bin/iris-kernel-server", lean_dir);

        // Build the Lean kernel server binary via lake
        let status = std::process::Command::new("lake")
            .arg("build")
            .arg("iris-kernel-server")
            .current_dir(&lean_dir)
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("cargo:warning=Built Lean kernel server at {}", server_bin);
            }
            Ok(s) => {
                println!(
                    "cargo:warning=lake build iris-kernel-server failed with status {}. \
                     Lean kernel IPC will fall back to Rust implementation at runtime.",
                    s
                );
            }
            Err(e) => {
                println!(
                    "cargo:warning=Could not run lake: {}. \
                     Lean kernel IPC will fall back to Rust implementation at runtime.",
                    e
                );
            }
        }

        // No native linking needed — communication is via IPC (stdin/stdout pipes).
        // The server binary is found at runtime by lean_bridge.rs.

        // Rerun if the Lean sources change
        println!("cargo:rerun-if-changed=../../lean/IrisKernelServer.lean");
        println!("cargo:rerun-if-changed=../../lean/IrisKernel/FFI.lean");
        println!("cargo:rerun-if-changed=../../lean/IrisKernel/Kernel.lean");
        println!("cargo:rerun-if-changed=../../lean/IrisKernel/Types.lean");
        println!("cargo:rerun-if-changed=../../lean/lakefile.lean");
    }
}
