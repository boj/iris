fn main() {
    if std::env::var("CARGO_FEATURE_LEAN_FFI").is_ok() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let lean_lib_dir = format!("{}/../../lean/.lake/build/lib", manifest_dir);

        let lean_prefix = std::process::Command::new("lean")
            .arg("--print-prefix")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        let lean_include = format!("{}/include", lean_prefix);
        let lean_lib = format!("{}/lib/lean", lean_prefix);

        // Compile shim FIRST (as object file, not archive)
        // so it can reference Lean symbols that come after in link order
        cc::Build::new()
            .file(format!("{}/lean_shim.c", manifest_dir))
            .include(&lean_include)
            .opt_level(2)
            .compile("iris_lean_shim");

        // Lean libraries with whole-archive to resolve cross-library dependencies
        println!("cargo:rustc-link-search=native={}", lean_lib_dir);
        println!("cargo:rustc-link-search=native={}", lean_lib);
        println!("cargo:rustc-link-arg=-Wl,--whole-archive");
        println!("cargo:rustc-link-lib=static=IrisKernel");
        println!("cargo:rustc-link-lib=static=Init");
        println!("cargo:rustc-link-lib=static=leanrt");
        println!("cargo:rustc-link-lib=static=leancpp");
        println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");

        // System deps
        println!("cargo:rustc-link-lib=dylib=gmp");
        println!("cargo:rustc-link-lib=dylib=uv");
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=dylib=dl");
    }
}
