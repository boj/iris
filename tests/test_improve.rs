//! Integration tests for observation-driven improvement.
//!
//! Tests the trace collection, test case building, and the full
//! evaluate_and_trace workflow.

use std::collections::BTreeMap;

use iris_types::eval::Value;
use iris_types::trace::{TraceCollector, TraceEntry, TraceBuffer, FunctionId, ImprovementConfig};

// ---------------------------------------------------------------------------
// TraceBuffer tests
// ---------------------------------------------------------------------------

#[test]
fn test_trace_buffer_push_and_snapshot() {
    let mut buf = TraceBuffer::new(5);
    for i in 0..3 {
        buf.push(TraceEntry {
            inputs: vec![Value::Int(i)],
            output: Value::Int(i * 2),
            latency_ns: 1000,
        });
    }
    assert_eq!(buf.len(), 3);
    let snap = buf.snapshot();
    assert_eq!(snap.len(), 3);
    assert_eq!(snap[0].inputs, vec![Value::Int(0)]);
    assert_eq!(snap[2].output, Value::Int(4));
}

#[test]
fn test_trace_buffer_ring_overflow() {
    let mut buf = TraceBuffer::new(3);
    for i in 0..5 {
        buf.push(TraceEntry {
            inputs: vec![Value::Int(i)],
            output: Value::Int(i),
            latency_ns: 100,
        });
    }
    assert_eq!(buf.len(), 5); // total pushes
    let snap = buf.snapshot();
    assert_eq!(snap.len(), 3); // capacity
    // Oldest entries (0, 1) evicted; should have 2, 3, 4
    assert_eq!(snap[0].inputs, vec![Value::Int(2)]);
    assert_eq!(snap[1].inputs, vec![Value::Int(3)]);
    assert_eq!(snap[2].inputs, vec![Value::Int(4)]);
}

#[test]
fn test_trace_buffer_avg_latency() {
    let mut buf = TraceBuffer::new(10);
    buf.push(TraceEntry { inputs: vec![], output: Value::Int(0), latency_ns: 100 });
    buf.push(TraceEntry { inputs: vec![], output: Value::Int(0), latency_ns: 200 });
    buf.push(TraceEntry { inputs: vec![], output: Value::Int(0), latency_ns: 300 });
    assert_eq!(buf.avg_latency_ns(), 200);
}

// ---------------------------------------------------------------------------
// TraceCollector tests
// ---------------------------------------------------------------------------

#[test]
fn test_collector_sampling() {
    // 100% sampling rate
    let collector = TraceCollector::new(1.0, 100);
    assert!(collector.should_sample());
    assert!(collector.should_sample());
}

#[test]
fn test_collector_low_sampling() {
    // 10% sampling rate → every 10th call
    let collector = TraceCollector::new(0.1, 100);
    let mut sampled = 0;
    for _ in 0..100 {
        if collector.should_sample() {
            sampled += 1;
        }
    }
    assert_eq!(sampled, 10); // Exactly 10% with deterministic counter
}

#[test]
fn test_collector_record_and_ready() {
    let collector = TraceCollector::new(1.0, 100);
    let fn_id = FunctionId("my_func".into());

    for i in 0..5 {
        collector.record(fn_id.clone(), TraceEntry {
            inputs: vec![Value::Int(i)],
            output: Value::Int(i * 2),
            latency_ns: 1000,
        });
    }

    // Not ready yet (need 50 by default)
    assert!(collector.ready_functions(50).is_empty());
    assert_eq!(collector.ready_functions(5), vec![fn_id.clone()]);
}

#[test]
fn test_collector_build_test_cases() {
    let collector = TraceCollector::new(1.0, 100);
    let fn_id = FunctionId("double".into());

    for i in 0..10 {
        collector.record(fn_id.clone(), TraceEntry {
            inputs: vec![Value::Int(i)],
            output: Value::Int(i * 2),
            latency_ns: 500,
        });
    }

    let cases = collector.build_test_cases(&fn_id, 5);
    assert!(cases.len() <= 5);
    // Each test case should have the correct expected output
    for tc in &cases {
        assert!(tc.expected_output.is_some());
        let input = match &tc.inputs[0] { Value::Int(v) => *v, _ => panic!() };
        let expected = match &tc.expected_output.as_ref().unwrap()[0] {
            Value::Int(v) => *v,
            _ => panic!(),
        };
        assert_eq!(expected, input * 2);
    }
}

#[test]
fn test_collector_deduplication() {
    let collector = TraceCollector::new(1.0, 100);
    let fn_id = FunctionId("inc".into());

    // Record the same input multiple times
    for _ in 0..10 {
        collector.record(fn_id.clone(), TraceEntry {
            inputs: vec![Value::Int(5)],
            output: Value::Int(6),
            latency_ns: 100,
        });
    }

    let cases = collector.build_test_cases(&fn_id, 20);
    assert_eq!(cases.len(), 1); // Deduplicated to 1
}

#[test]
fn test_collector_disable_enable() {
    let collector = TraceCollector::new(1.0, 100);
    assert!(collector.should_sample());

    collector.disable();
    assert!(!collector.should_sample());
    assert!(!collector.should_sample());

    collector.enable();
    // Counter continued incrementing, but next 1-in-1 will hit
    assert!(collector.should_sample());
}

#[test]
fn test_collector_stats() {
    let collector = TraceCollector::new(1.0, 100);
    collector.record(FunctionId("a".into()), TraceEntry {
        inputs: vec![], output: Value::Int(0), latency_ns: 0,
    });
    collector.record(FunctionId("a".into()), TraceEntry {
        inputs: vec![], output: Value::Int(1), latency_ns: 0,
    });
    collector.record(FunctionId("b".into()), TraceEntry {
        inputs: vec![], output: Value::Int(0), latency_ns: 0,
    });

    let stats = collector.stats();
    assert_eq!(stats.get(&FunctionId("a".into())), Some(&2));
    assert_eq!(stats.get(&FunctionId("b".into())), Some(&1));
}

// ---------------------------------------------------------------------------
// ImprovementConfig tests
// ---------------------------------------------------------------------------

#[test]
fn test_improvement_config_defaults() {
    let config = ImprovementConfig::default();
    assert_eq!(config.min_traces, 50);
    assert!((config.max_slowdown - 2.0).abs() < f64::EPSILON);
    assert_eq!(config.evolution_budget_secs, 5);
    assert_eq!(config.population_size, 32);
    assert_eq!(config.max_generations, 200);
    assert_eq!(config.max_test_cases, 20);
}

// ---------------------------------------------------------------------------
// LiveRegistry tests
// ---------------------------------------------------------------------------

#[test]
fn test_live_registry_get_and_swap() {
    use iris_exec::improve::LiveRegistry;
    use iris_exec::registry::FragmentRegistry;
    use iris_types::trace::ImprovementResult;

    // Compile a real program to get a valid graph
    let src = "let f : Int = 1";
    let module = iris_bootstrap::syntax::parse(src).unwrap();
    let result = iris_bootstrap::syntax::lower::compile_module(&module);
    let (_, frag, _) = &result.fragments[0];

    let mut graphs = BTreeMap::new();
    graphs.insert("f".to_string(), frag.graph.clone());

    let reg = LiveRegistry::new(graphs, FragmentRegistry::new());

    assert!(reg.get_graph("f").is_some());
    assert!(reg.get_graph("nonexistent").is_none());
    assert_eq!(reg.function_names(), vec!["f".to_string()]);

    // Swap with the same graph (just testing the mechanism)
    reg.swap("f", frag.graph.clone(), ImprovementResult {
        fn_id: FunctionId("f".into()),
        success: true,
        old_latency_ns: 1000,
        new_latency_ns: Some(500),
        test_cases_used: 5,
        generations_run: 100,
    });

    assert_eq!(reg.improvements().len(), 1);
    assert!(reg.improvements()[0].success);
}

// ---------------------------------------------------------------------------
// evaluate_and_trace integration test
// ---------------------------------------------------------------------------

#[test]
fn test_evaluate_and_trace_records() {
    use iris_exec::improve::evaluate_and_trace;
    use iris_exec::registry::FragmentRegistry;

    // Compile a simple program: let answer = 42
    let src = "let answer : Int = 42";
    let module = iris_bootstrap::syntax::parse(src).unwrap();
    let result = iris_bootstrap::syntax::lower::compile_module(&module);
    assert!(result.errors.is_empty());

    let (name, frag, _) = &result.fragments[0];
    let mut registry = FragmentRegistry::new();
    registry.register(frag.clone());

    // 100% sampling
    let collector = TraceCollector::new(1.0, 100);

    let (outputs, _) = evaluate_and_trace(name, &frag.graph, &[], &registry, &collector).unwrap();
    assert_eq!(outputs, vec![Value::Int(42)]);

    // Trace should have been recorded
    let stats = collector.stats();
    assert_eq!(stats.get(&FunctionId("answer".into())), Some(&1));

    // The recorded test case should have empty inputs and output=42
    let cases = collector.build_test_cases(&FunctionId("answer".into()), 10);
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].inputs, Vec::<Value>::new());
    assert_eq!(cases[0].expected_output, Some(vec![Value::Int(42)]));
}
