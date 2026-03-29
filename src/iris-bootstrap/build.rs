/// Find Lean 4 prefix (tries PATH first, then nix-shell).
fn lean_prefix() -> String {
    if let Ok(output) = std::process::Command::new("lean")
        .arg("--print-prefix")
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    if let Ok(output) = std::process::Command::new("nix-shell")
        .args(["-p", "lean4", "--run", "lean --print-prefix"])
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    panic!("Cannot find Lean 4 — install it or add it to PATH");
}

/// On NixOS, find a nix package's lib dir via `nix eval`.
fn nix_lib_path(pkg: &str) -> Option<String> {
    let expr = format!("nixpkgs#{}", pkg);
    let output = std::process::Command::new("nix")
        .args(["eval", "--raw", &expr])
        .env("NIX_PATH", std::env::var("NIX_PATH").unwrap_or_default())
        .output()
        .ok()?;
    if output.status.success() {
        let store_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let lib_dir = format!("{}/lib", store_path);
        if std::path::Path::new(&lib_dir).exists() {
            return Some(lib_dir);
        }
    }
    None
}

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let lean_dir = format!("{}/../../lean", manifest_dir);
    let lean_lib_dir = format!("{}/.lake/build/lib", lean_dir);
    let iris_kernel_a = format!("{}/libIrisKernel.a", lean_lib_dir);

    // Build the Lean kernel static library if it doesn't exist.
    // Uses build_ffi.sh which does: lake build → leanc -c → ar rcs
    if !std::path::Path::new(&iris_kernel_a).exists() {
        eprintln!("cargo:warning=libIrisKernel.a not found, running build_ffi.sh...");
        let build_script = format!("{}/build_ffi.sh", lean_dir);

        // Try directly first
        let status = std::process::Command::new("bash")
            .arg(&build_script)
            .current_dir(&lean_dir)
            .status()
            .unwrap_or_else(|_| {
                // Fall back to nix-shell with Lean
                std::process::Command::new("nix-shell")
                    .args(["-p", "lean4", "--run", &format!("bash {}", build_script)])
                    .current_dir(&lean_dir)
                    .status()
                    .expect("failed to run build_ffi.sh — is Lean 4 installed?")
            });
        assert!(status.success(), "build_ffi.sh failed");
    }

    let prefix = lean_prefix();
    let lean_include = format!("{}/include", prefix);
    let lean_lib = format!("{}/lib/lean", prefix);

    // Compile C shim
    cc::Build::new()
        .file(format!("{}/lean_shim.c", manifest_dir))
        .include(&lean_include)
        .opt_level(2)
        .compile("iris_lean_shim");

    // Lean libraries
    println!("cargo:rustc-link-search=native={}", lean_lib_dir);
    println!("cargo:rustc-link-search=native={}", lean_lib);
    println!("cargo:rustc-link-arg=-Wl,--whole-archive");
    println!("cargo:rustc-link-lib=static=IrisKernel");
    println!("cargo:rustc-link-lib=static=Init");
    println!("cargo:rustc-link-lib=static=leanrt");
    println!("cargo:rustc-link-lib=static=leancpp");
    println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");

    // NixOS: add library search paths for system deps
    if let Some(gmp_lib) = nix_lib_path("gmp") {
        println!("cargo:rustc-link-search=native={}", gmp_lib);
    }
    if let Some(uv_lib) = nix_lib_path("libuv") {
        println!("cargo:rustc-link-search=native={}", uv_lib);
    }

    // System deps required by Lean runtime
    println!("cargo:rustc-link-lib=dylib=gmp");
    println!("cargo:rustc-link-lib=dylib=uv");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=dl");
}
