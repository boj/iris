use iris_evolve::self_improving_daemon::*;
use iris_evolve::auto_improve::AutoImproveConfig;
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    println!("=== IRIS Self-Improving Daemon: Live Test ===\n");
    
    let dir = std::env::temp_dir().join("iris_live_test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    
    let config = SelfImprovingConfig {
        cycle_time_ms: 10, // fast cycles for testing
        max_cycles: Some(100),
        improve_interval: 5,
        inspect_interval: 3,
        auto_improve: AutoImproveConfig {
            cycle_interval_secs: 0,
            max_slowdown: 2.0,
            test_cases_per_component: 5,
            evolution_generations: 20,
            evolution_pop_size: 16,
            gate_runs: 3,
            explore_problems: 3,
        },
        state_dir: Some(dir.clone()),
        memory_limit: 50 * 1024 * 1024,
        seed: Some(42),
        max_improve_threads: 2,
        max_stagnant: 5,
        min_improvement: 0.05,
        exec_mode: ExecMode::FixedInterval(Duration::from_millis(10)),
        trigger_check_interval: 10,
    };
    
    println!("Config: {} max cycles, {}ms interval, improve every {} cycles",
        config.max_cycles.unwrap_or(0), config.cycle_time_ms, config.improve_interval);
    println!("Gate: {:.1}x max slowdown, {} test cases, {} gens × {} pop",
        config.auto_improve.max_slowdown,
        config.auto_improve.test_cases_per_component,
        config.auto_improve.evolution_generations,
        config.auto_improve.evolution_pop_size);
    println!();
    
    let mut daemon = ThreadedDaemon::new(config);
    let result = daemon.run();
    
    println!("\n=== Results ===");
    println!("Cycles completed:    {}", result.cycles_completed);
    println!("Improvement cycles:  {}", result.improvement_cycles);
    println!("Components deployed: {}", result.components_deployed);
    println!("Audit entries:       {}", result.audit_entries);
    println!("Converged components:{}", result.converged_components);
    println!("Fully converged:     {}", result.fully_converged);
    println!("Recursive depth:     {}", result.recursive_depth);
    println!("Wall time:           {:.2}s", result.total_time.as_secs_f64());
    
    // Check persistence
    let state_path = std::env::temp_dir().join("iris_live_test/daemon_state.json");
    let audit_path = std::env::temp_dir().join("iris_live_test/audit_trail.json");
    println!("\nPersistence:");
    println!("  State file: {} ({})", 
        state_path.display(),
        if state_path.exists() { format!("{} bytes", std::fs::metadata(&state_path).unwrap().len()) } else { "missing".to_string() });
    println!("  Audit file: {} ({})",
        audit_path.display(),
        if audit_path.exists() { format!("{} bytes", std::fs::metadata(&audit_path).unwrap().len()) } else { "missing".to_string() });
    
    println!("\n=== IRIS daemon test complete ===");
}
