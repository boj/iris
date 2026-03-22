use std::env;
use std::path::PathBuf;

/// Check if the current CPU supports AVX-512F by reading cpuid.
///
/// For cross-compilation scenarios or when detection is unreliable, the
/// `IRIS_NO_AVX512` env var can be set to force the scalar fallback.
fn cpu_supports_avx512() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        // Check CPUID leaf 7, subleaf 0: EBX bit 16 = AVX-512F.
        if is_x86_feature_detected!("avx512f") {
            return true;
        }
    }
    false
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let c_src_dir = PathBuf::from(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("iris-clcu")
        .join("src");

    let c_sources = [
        c_src_dir.join("arena.c"),
        c_src_dir.join("interpreter.c"),
        c_src_dir.join("tlb_map.c"),
        c_src_dir.join("tso_ring.c"),
    ];

    // Only compile files that exist (tlb_map and tso_ring may have
    // unresolved dependencies; we only strictly need arena + interpreter).
    let available_sources: Vec<_> = c_sources
        .iter()
        .filter(|p| p.exists())
        .cloned()
        .collect();

    let mut build = cc::Build::new();
    build
        .include(&c_src_dir)
        .warnings(false) // C code uses __attribute__ extensions
        .opt_level(2);

    for src in &available_sources {
        build.file(src);
    }

    // Only enable AVX-512 if:
    //   1. IRIS_NO_AVX512 is not set
    //   2. The build-time CPU supports AVX-512F (runtime check)
    //   3. The compiler supports -mavx512f
    //
    // This prevents SIGILL crashes when the compiler can handle AVX-512
    // flags but the runtime CPU does not support the instructions.
    let force_no_avx512 = env::var("IRIS_NO_AVX512").is_ok();
    let use_avx512 = !force_no_avx512 && cpu_supports_avx512();

    if use_avx512 {
        // Verify the compiler can actually produce AVX-512 code.
        let probe_ok = cc::Build::new()
            .include(&c_src_dir)
            .warnings(false)
            .opt_level(0)
            .flag("-mavx512f")
            .file(&c_sources[1]) // interpreter.c — the file that uses AVX-512
            .try_compile("iris_clcu_avx_probe")
            .is_ok();

        if probe_ok {
            build.flag("-mavx512f");
            println!("cargo::rustc-cfg=iris_avx512");
        }
    }

    build.compile("iris_clcu");

    println!("cargo::rerun-if-changed={}", c_src_dir.display());
    for src in &available_sources {
        println!("cargo::rerun-if-changed={}", src.display());
    }
}
