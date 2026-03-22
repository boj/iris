use iris_evolve::instrumentation::{AuditTrail, AuditAction};

#[test]
fn print_audit_chain_example() {
    let mut trail = AuditTrail::new();

    // Entry 0: Deploy replace_prim (IRIS component passes performance gate)
    let before = blake3::hash(b"system_state_v1");
    let after = blake3::hash(b"system_state_v2_with_iris_replace_prim");
    trail.record_full(
        AuditAction::ComponentDeployed { name: "replace_prim".into(), slowdown: 1.3 },
        *before.as_bytes(), *after.as_bytes(), None, 0.23,
    );

    // Entry 1: Deploy insert_node
    let before2 = *after.as_bytes();
    let after2 = *blake3::hash(b"system_state_v3_with_iris_insert_node").as_bytes();
    trail.record_full(
        AuditAction::ComponentDeployed { name: "insert_node".into(), slowdown: 1.4 },
        before2, after2, None, 0.15,
    );

    // Entry 2: Regression detected, rollback insert_node
    let before3 = after2;
    let after3 = before2;
    trail.record_full(
        AuditAction::ComponentReverted { name: "insert_node".into(), reason: "p99 regression 3.2x".into() },
        before3, after3, None, -0.15,
    );

    let h = |b: &[u8; 32]| b.iter().map(|x| format!("{:02x}", x)).collect::<Vec<_>>().join("");

    println!("\n=== BLAKE3 Merkle Audit Chain ===\n");
    for entry in trail.entries() {
        println!("Entry #{}", entry.id);
        println!("  action:       {:?}", entry.action);
        println!("  perf_delta:   {:+.2}", entry.performance_delta);
        println!("  before_hash:  {}", h(&entry.before_hash));
        println!("  after_hash:   {}", h(&entry.after_hash));
        println!("  prev_hash:    {}", h(&entry.prev_hash));
        println!("  entry_hash:   {}", h(&entry.entry_hash));
        println!("  valid:        {}", entry.entry_hash == entry.compute_hash());
        println!();
    }

    println!("Chain integrity: {}", trail.verify_chain());
    println!("Merkle root:     {}", h(&trail.merkle_root()));
    println!("Entries:         {}", trail.len());

    println!("\n--- Tamper test ---");
    let mut tampered = trail.clone();
    tampered.entries_mut()[1].performance_delta = 99.0; // tamper with entry #1
    println!("After tampering entry #1: chain valid = {}", tampered.verify_chain());
}
