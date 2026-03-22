//! Tests for the analytic decomposition module (analyzer).
//!
//! Verifies pattern detection and skeleton construction across all
//! supported pattern types.

use iris_types::eval::*;
use iris_evolve::analyzer::*;
use iris_exec::interpreter;

fn tc(inputs: Vec<Value>, expected: Value) -> TestCase {
    TestCase {
        inputs,
        expected_output: Some(vec![expected]),
        initial_state: None,
        expected_state: None,
    }
}

// ---------------------------------------------------------------------------
// Pattern detection tests
// ---------------------------------------------------------------------------

#[test]
fn analyzer_detect_constant() {
    let cases = vec![
        tc(vec![Value::Int(0)], Value::Int(42)),
        tc(vec![Value::Int(99)], Value::Int(42)),
        tc(vec![Value::Int(-5)], Value::Int(42)),
    ];
    let pattern = analyze_test_cases(&cases);
    assert_eq!(pattern, DetectedPattern::Constant(Value::Int(42)));
}

#[test]
fn analyzer_detect_identity() {
    let cases = vec![
        tc(vec![Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(7)], Value::Int(7)),
        tc(vec![Value::Int(-3)], Value::Int(-3)),
        tc(vec![Value::Int(100)], Value::Int(100)),
    ];
    let pattern = analyze_test_cases(&cases);
    assert_eq!(pattern, DetectedPattern::Identity { input_index: 0 });
}

#[test]
fn analyzer_detect_binary_add() {
    let cases = vec![
        tc(vec![Value::Int(1), Value::Int(2)], Value::Int(3)),
        tc(vec![Value::Int(0), Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(10), Value::Int(-3)], Value::Int(7)),
        tc(vec![Value::Int(-5), Value::Int(-5)], Value::Int(-10)),
    ];
    let pattern = analyze_test_cases(&cases);
    assert_eq!(
        pattern,
        DetectedPattern::BinaryArithmetic {
            op: 0x00,
            left_input: 0,
            right_input: 1,
        }
    );
}

#[test]
fn analyzer_detect_binary_sub() {
    let cases = vec![
        tc(vec![Value::Int(10), Value::Int(3)], Value::Int(7)),
        tc(vec![Value::Int(5), Value::Int(5)], Value::Int(0)),
        tc(vec![Value::Int(0), Value::Int(1)], Value::Int(-1)),
    ];
    let pattern = analyze_test_cases(&cases);
    assert_eq!(
        pattern,
        DetectedPattern::BinaryArithmetic {
            op: 0x01,
            left_input: 0,
            right_input: 1,
        }
    );
}

#[test]
fn analyzer_detect_binary_mul() {
    let cases = vec![
        tc(vec![Value::Int(3), Value::Int(4)], Value::Int(12)),
        tc(vec![Value::Int(0), Value::Int(99)], Value::Int(0)),
        tc(vec![Value::Int(-2), Value::Int(5)], Value::Int(-10)),
    ];
    let pattern = analyze_test_cases(&cases);
    assert_eq!(
        pattern,
        DetectedPattern::BinaryArithmetic {
            op: 0x02,
            left_input: 0,
            right_input: 1,
        }
    );
}

#[test]
fn analyzer_detect_calculator_add_sub() {
    // Calculator: input[1] dispatches between add(input[0], input[2])
    // and sub(input[0], input[2]).
    let cases = vec![
        tc(
            vec![Value::Int(3), Value::Int(0), Value::Int(5)],
            Value::Int(8),
        ), // 3+5
        tc(
            vec![Value::Int(10), Value::Int(1), Value::Int(3)],
            Value::Int(7),
        ), // 10-3
        tc(
            vec![Value::Int(0), Value::Int(0), Value::Int(0)],
            Value::Int(0),
        ), // 0+0
        tc(
            vec![Value::Int(7), Value::Int(1), Value::Int(7)],
            Value::Int(0),
        ), // 7-7
        tc(
            vec![Value::Int(1), Value::Int(0), Value::Int(1)],
            Value::Int(2),
        ), // 1+1
        tc(
            vec![Value::Int(5), Value::Int(1), Value::Int(2)],
            Value::Int(3),
        ), // 5-2
    ];
    let pattern = analyze_test_cases(&cases);
    match &pattern {
        DetectedPattern::ConditionalDispatch {
            dispatch_input,
            branches,
        } => {
            assert_eq!(*dispatch_input, 1, "dispatch input should be index 1");
            assert!(branches.len() >= 2, "should have at least 2 branches");

            // Branch 0: add(input[0], input[2])
            let b0 = branches.iter().find(|(k, _)| *k == 0);
            assert!(b0.is_some(), "should have branch for dispatch value 0");
            match &b0.unwrap().1 {
                DetectedPattern::BinaryArithmetic {
                    op,
                    left_input,
                    right_input,
                } => {
                    assert_eq!(*op, 0x00, "branch 0 should be add");
                    assert_eq!(*left_input, 0);
                    assert_eq!(*right_input, 2);
                }
                other => panic!("Expected BinaryArithmetic for branch 0, got {:?}", other),
            }

            // Branch 1: sub(input[0], input[2])
            let b1 = branches.iter().find(|(k, _)| *k == 1);
            assert!(b1.is_some(), "should have branch for dispatch value 1");
            match &b1.unwrap().1 {
                DetectedPattern::BinaryArithmetic {
                    op,
                    left_input,
                    right_input,
                } => {
                    assert_eq!(*op, 0x01, "branch 1 should be sub");
                    assert_eq!(*left_input, 0);
                    assert_eq!(*right_input, 2);
                }
                other => panic!("Expected BinaryArithmetic for branch 1, got {:?}", other),
            }
        }
        other => panic!("Expected ConditionalDispatch, got {:?}", other),
    }
}

#[test]
fn analyzer_detect_fold_sum() {
    let cases = vec![
        tc(
            vec![Value::tuple(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
            Value::Int(6),
        ),
        tc(vec![Value::tuple(vec![Value::Int(10)])], Value::Int(10)),
        tc(
            vec![Value::tuple(vec![Value::Int(0), Value::Int(0)])],
            Value::Int(0),
        ),
    ];
    let pattern = analyze_test_cases(&cases);
    assert_eq!(
        pattern,
        DetectedPattern::FoldReduction {
            base: 0,
            op: 0x00,
            input_index: 0,
        }
    );
}

#[test]
fn analyzer_detect_fold_product() {
    let cases = vec![
        tc(
            vec![Value::tuple(vec![
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
            ])],
            Value::Int(24),
        ),
        tc(vec![Value::tuple(vec![Value::Int(5)])], Value::Int(5)),
        tc(
            vec![Value::tuple(vec![Value::Int(1), Value::Int(1)])],
            Value::Int(1),
        ),
    ];
    let pattern = analyze_test_cases(&cases);
    assert_eq!(
        pattern,
        DetectedPattern::FoldReduction {
            base: 1,
            op: 0x02,
            input_index: 0,
        }
    );
}

#[test]
fn analyzer_detect_map_double() {
    let cases = vec![
        tc(
            vec![Value::tuple(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
            Value::tuple(vec![Value::Int(2), Value::Int(4), Value::Int(6)]),
        ),
        tc(
            vec![Value::tuple(vec![Value::Int(0)])],
            Value::tuple(vec![Value::Int(0)]),
        ),
        tc(
            vec![Value::tuple(vec![Value::Int(5), Value::Int(10)])],
            Value::tuple(vec![Value::Int(10), Value::Int(20)]),
        ),
    ];
    let pattern = analyze_test_cases(&cases);
    assert_eq!(
        pattern,
        DetectedPattern::MapTransform {
            op: 0x00,
            input_index: 0,
        }
    );
}

#[test]
fn analyzer_detect_map_square() {
    let cases = vec![
        tc(
            vec![Value::tuple(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
            Value::tuple(vec![Value::Int(1), Value::Int(4), Value::Int(9)]),
        ),
        tc(
            vec![Value::tuple(vec![Value::Int(0)])],
            Value::tuple(vec![Value::Int(0)]),
        ),
    ];
    let pattern = analyze_test_cases(&cases);
    assert_eq!(
        pattern,
        DetectedPattern::MapTransform {
            op: 0x02,
            input_index: 0,
        }
    );
}

#[test]
fn analyzer_detect_map_fold_sum_of_squares() {
    let cases = vec![
        tc(
            vec![Value::tuple(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
            Value::Int(14), // 1 + 4 + 9
        ),
        tc(vec![Value::tuple(vec![Value::Int(0)])], Value::Int(0)),
        tc(
            vec![Value::tuple(vec![Value::Int(4), Value::Int(3)])],
            Value::Int(25), // 16 + 9
        ),
    ];
    let pattern = analyze_test_cases(&cases);
    assert_eq!(
        pattern,
        DetectedPattern::MapFoldComposition {
            map_op: 0x02,
            fold_op: 0x00,
            fold_base: 0,
            input_index: 0,
        }
    );
}

#[test]
fn analyzer_detect_unknown() {
    let cases = vec![
        tc(vec![Value::Int(1)], Value::Int(17)),
        tc(vec![Value::Int(2)], Value::Int(42)),
        tc(vec![Value::Int(3)], Value::Int(-5)),
    ];
    let pattern = analyze_test_cases(&cases);
    assert_eq!(pattern, DetectedPattern::Unknown);
}

// ---------------------------------------------------------------------------
// Skeleton construction tests
// ---------------------------------------------------------------------------

#[test]
fn analyzer_build_constant_skeleton() {
    let pattern = DetectedPattern::Constant(Value::Int(42));
    let fragment = build_skeleton(&pattern);
    assert!(fragment.is_some(), "should build constant skeleton");
}

#[test]
fn analyzer_build_identity_skeleton() {
    let pattern = DetectedPattern::Identity { input_index: 0 };
    let fragment = build_skeleton(&pattern);
    assert!(fragment.is_some(), "should build identity skeleton");
}

#[test]
fn analyzer_build_binary_arithmetic_skeleton() {
    let pattern = DetectedPattern::BinaryArithmetic {
        op: 0x00,
        left_input: 0,
        right_input: 1,
    };
    let fragment = build_skeleton(&pattern);
    assert!(fragment.is_some(), "should build binary arithmetic skeleton");
}

#[test]
fn analyzer_build_conditional_dispatch_skeleton() {
    let pattern = DetectedPattern::ConditionalDispatch {
        dispatch_input: 1,
        branches: vec![
            (
                0,
                DetectedPattern::BinaryArithmetic {
                    op: 0x00,
                    left_input: 0,
                    right_input: 2,
                },
            ),
            (
                1,
                DetectedPattern::BinaryArithmetic {
                    op: 0x01,
                    left_input: 0,
                    right_input: 2,
                },
            ),
        ],
    };
    let fragment = build_skeleton(&pattern);
    assert!(
        fragment.is_some(),
        "should build conditional dispatch skeleton"
    );
}

#[test]
fn analyzer_build_fold_skeleton() {
    let pattern = DetectedPattern::FoldReduction {
        base: 0,
        op: 0x00,
        input_index: 0,
    };
    let fragment = build_skeleton(&pattern);
    assert!(fragment.is_some(), "should build fold skeleton");
}

#[test]
fn analyzer_build_map_skeleton() {
    let pattern = DetectedPattern::MapTransform {
        op: 0x00,
        input_index: 0,
    };
    let fragment = build_skeleton(&pattern);
    assert!(fragment.is_some(), "should build map skeleton");
}

#[test]
fn analyzer_build_map_fold_skeleton() {
    let pattern = DetectedPattern::MapFoldComposition {
        map_op: 0x02,
        fold_op: 0x00,
        fold_base: 0,
        input_index: 0,
    };
    let fragment = build_skeleton(&pattern);
    assert!(fragment.is_some(), "should build map+fold skeleton");
}

#[test]
fn analyzer_build_unknown_returns_none() {
    let pattern = DetectedPattern::Unknown;
    let fragment = build_skeleton(&pattern);
    assert!(fragment.is_none(), "unknown pattern should return None");
}

// ---------------------------------------------------------------------------
// End-to-end: detect + build
// ---------------------------------------------------------------------------

#[test]
fn analyzer_end_to_end_constant() {
    let cases = vec![
        tc(vec![Value::Int(0)], Value::Int(42)),
        tc(vec![Value::Int(99)], Value::Int(42)),
    ];
    let pattern = analyze_test_cases(&cases);
    assert!(matches!(pattern, DetectedPattern::Constant(_)));
    let skeleton = build_skeleton(&pattern);
    assert!(skeleton.is_some());
}

#[test]
fn analyzer_end_to_end_fold_sum() {
    let cases = vec![
        tc(
            vec![Value::tuple(vec![Value::Int(1), Value::Int(2)])],
            Value::Int(3),
        ),
        tc(
            vec![Value::tuple(vec![
                Value::Int(10),
                Value::Int(20),
                Value::Int(30),
            ])],
            Value::Int(60),
        ),
    ];
    let pattern = analyze_test_cases(&cases);
    assert!(matches!(pattern, DetectedPattern::FoldReduction { .. }));
    let skeleton = build_skeleton(&pattern);
    assert!(skeleton.is_some());
}

#[test]
fn analyzer_end_to_end_map_double() {
    let cases = vec![
        tc(
            vec![Value::tuple(vec![Value::Int(1), Value::Int(2)])],
            Value::tuple(vec![Value::Int(2), Value::Int(4)]),
        ),
        tc(
            vec![Value::tuple(vec![Value::Int(5)])],
            Value::tuple(vec![Value::Int(10)]),
        ),
    ];
    let pattern = analyze_test_cases(&cases);
    assert!(matches!(pattern, DetectedPattern::MapTransform { .. }));
    let skeleton = build_skeleton(&pattern);
    assert!(skeleton.is_some());
}

#[test]
fn analyzer_empty_test_cases() {
    let pattern = analyze_test_cases(&[]);
    assert_eq!(pattern, DetectedPattern::Unknown);
}

#[test]
fn analyzer_no_expected_output() {
    let cases = vec![TestCase {
        inputs: vec![Value::Int(1)],
        expected_output: None,
        initial_state: None,
        expected_state: None,
    }];
    let pattern = analyze_test_cases(&cases);
    assert_eq!(pattern, DetectedPattern::Unknown);
}

// ---------------------------------------------------------------------------
// Interpreter evaluation tests: verify skeletons produce correct outputs
// ---------------------------------------------------------------------------

#[test]
fn analyzer_skeleton_constant_evaluates_correctly() {
    let cases = vec![
        tc(vec![Value::Int(0)], Value::Int(42)),
        tc(vec![Value::Int(99)], Value::Int(42)),
    ];
    let pattern = analyze_test_cases(&cases);
    let fragment = build_skeleton(&pattern).unwrap();

    for case in &cases {
        let (outputs, _state) =
            interpreter::interpret(&fragment.graph, &case.inputs, None).unwrap();
        assert_eq!(
            outputs,
            case.expected_output.clone().unwrap(),
            "constant skeleton should return 42"
        );
    }
}

#[test]
fn analyzer_skeleton_identity_evaluates_correctly() {
    let cases = vec![
        tc(vec![Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(7)], Value::Int(7)),
        tc(vec![Value::Int(-3)], Value::Int(-3)),
    ];
    let pattern = analyze_test_cases(&cases);
    let fragment = build_skeleton(&pattern).unwrap();

    for case in &cases {
        let (outputs, _state) =
            interpreter::interpret(&fragment.graph, &case.inputs, None).unwrap();
        assert_eq!(
            outputs,
            case.expected_output.clone().unwrap(),
            "identity skeleton should return input unchanged"
        );
    }
}

#[test]
fn analyzer_skeleton_fold_sum_evaluates_correctly() {
    let cases = vec![
        tc(
            vec![Value::tuple(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
            Value::Int(6),
        ),
        tc(vec![Value::tuple(vec![Value::Int(10)])], Value::Int(10)),
    ];
    let pattern = analyze_test_cases(&cases);
    let fragment = build_skeleton(&pattern).unwrap();

    for case in &cases {
        let (outputs, _state) =
            interpreter::interpret(&fragment.graph, &case.inputs, None).unwrap();
        assert_eq!(
            outputs,
            case.expected_output.clone().unwrap(),
            "fold sum skeleton should sum list elements"
        );
    }
}

#[test]
fn analyzer_skeleton_map_double_evaluates_correctly() {
    let cases = vec![
        tc(
            vec![Value::tuple(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
            Value::tuple(vec![Value::Int(2), Value::Int(4), Value::Int(6)]),
        ),
        tc(
            vec![Value::tuple(vec![Value::Int(0)])],
            Value::tuple(vec![Value::Int(0)]),
        ),
        tc(
            vec![Value::tuple(vec![Value::Int(5), Value::Int(10)])],
            Value::tuple(vec![Value::Int(10), Value::Int(20)]),
        ),
    ];
    let pattern = analyze_test_cases(&cases);
    let fragment = build_skeleton(&pattern).unwrap();

    for case in &cases {
        let result = interpreter::interpret(&fragment.graph, &case.inputs, None);
        match result {
            Ok((outputs, _state)) => {
                assert_eq!(
                    outputs,
                    case.expected_output.clone().unwrap(),
                    "map double skeleton should double each element"
                );
            }
            Err(e) => {
                panic!("map double skeleton evaluation failed: {:?}", e);
            }
        }
    }
}

#[test]
fn analyzer_detect_clamp() {
    let cases = vec![
        tc(vec![Value::Int(5)], Value::Int(5)),
        tc(vec![Value::Int(-3)], Value::Int(0)),
        tc(vec![Value::Int(15)], Value::Int(10)),
        tc(vec![Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(10)], Value::Int(10)),
    ];
    let pattern = analyze_test_cases(&cases);
    assert_eq!(pattern, DetectedPattern::Clamp { lo: 0, hi: 10 });
}

#[test]
fn analyzer_skeleton_clamp_evaluates_correctly() {
    let cases = vec![
        tc(vec![Value::Int(5)], Value::Int(5)),
        tc(vec![Value::Int(-3)], Value::Int(0)),
        tc(vec![Value::Int(15)], Value::Int(10)),
        tc(vec![Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(10)], Value::Int(10)),
    ];
    let pattern = analyze_test_cases(&cases);
    let fragment = build_skeleton(&pattern).unwrap();

    for case in &cases {
        let (outputs, _state) =
            interpreter::interpret(&fragment.graph, &case.inputs, None).unwrap();
        assert_eq!(
            outputs,
            case.expected_output.clone().unwrap(),
            "clamp skeleton should clamp input to [0, 10] for input {:?}",
            case.inputs,
        );
    }
}

#[test]
fn analyzer_detect_fibonacci() {
    let cases = vec![
        tc(vec![Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(1)], Value::Int(1)),
        tc(vec![Value::Int(5)], Value::Int(5)),
        tc(vec![Value::Int(7)], Value::Int(13)),
    ];
    let patterns = analyze_all_patterns(&cases);
    assert!(
        patterns.iter().any(|p| matches!(p, DetectedPattern::Fibonacci)),
        "should detect Fibonacci pattern"
    );
}

#[test]
fn analyzer_skeleton_fibonacci_evaluates_correctly() {
    let cases = vec![
        tc(vec![Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(1)], Value::Int(1)),
        tc(vec![Value::Int(5)], Value::Int(5)),
        tc(vec![Value::Int(7)], Value::Int(13)),
    ];
    let pattern = DetectedPattern::Fibonacci;
    let fragment = build_skeleton(&pattern).unwrap();

    for case in &cases {
        let (outputs, _state) =
            interpreter::interpret(&fragment.graph, &case.inputs, None).unwrap();
        assert_eq!(
            outputs,
            case.expected_output.clone().unwrap(),
            "fibonacci skeleton should return fib(n) for input {:?}",
            case.inputs,
        );
    }
}

#[test]
fn analyzer_detect_factorial() {
    let cases = vec![
        tc(vec![Value::Int(0)], Value::Int(1)),
        tc(vec![Value::Int(1)], Value::Int(1)),
        tc(vec![Value::Int(5)], Value::Int(120)),
        tc(vec![Value::Int(3)], Value::Int(6)),
    ];
    let patterns = analyze_all_patterns(&cases);
    assert!(
        patterns.iter().any(|p| matches!(p, DetectedPattern::Factorial)),
        "should detect Factorial pattern"
    );
}

#[test]
fn analyzer_detect_power() {
    let cases = vec![
        tc(vec![Value::Int(2), Value::Int(0)], Value::Int(1)),
        tc(vec![Value::Int(2), Value::Int(3)], Value::Int(8)),
        tc(vec![Value::Int(3), Value::Int(2)], Value::Int(9)),
        tc(vec![Value::Int(5), Value::Int(3)], Value::Int(125)),
    ];
    let patterns = analyze_all_patterns(&cases);
    assert!(
        patterns.iter().any(|p| matches!(p, DetectedPattern::Power)),
        "should detect Power pattern"
    );
}

#[test]
fn analyzer_skeleton_factorial_evaluates_correctly() {
    let cases = vec![
        tc(vec![Value::Int(0)], Value::Int(1)),
        tc(vec![Value::Int(1)], Value::Int(1)),
        tc(vec![Value::Int(5)], Value::Int(120)),
        tc(vec![Value::Int(3)], Value::Int(6)),
    ];
    let pattern = DetectedPattern::Factorial;
    let fragment = build_skeleton(&pattern).unwrap();

    for case in &cases {
        let (outputs, _state) =
            interpreter::interpret(&fragment.graph, &case.inputs, None).unwrap();
        assert_eq!(
            outputs,
            case.expected_output.clone().unwrap(),
            "factorial skeleton should return n! for input {:?}",
            case.inputs,
        );
    }
}

#[test]
fn analyzer_detect_manhattan() {
    let cases = vec![
        tc(
            vec![
                Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
                Value::tuple(vec![Value::Int(4), Value::Int(2), Value::Int(1)]),
            ],
            Value::Int(5),
        ),
        tc(
            vec![
                Value::tuple(vec![Value::Int(0), Value::Int(0)]),
                Value::tuple(vec![Value::Int(3), Value::Int(4)]),
            ],
            Value::Int(7),
        ),
        tc(
            vec![
                Value::tuple(vec![Value::Int(5)]),
                Value::tuple(vec![Value::Int(5)]),
            ],
            Value::Int(0),
        ),
    ];
    let patterns = analyze_all_patterns(&cases);
    assert!(
        patterns.iter().any(|p| matches!(p, DetectedPattern::ZipMapFold { .. })),
        "should detect ZipMapFold pattern for manhattan distance"
    );
}

#[test]
fn analyzer_detect_polynomial() {
    let cases = vec![
        tc(
            vec![
                Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
                Value::Int(2),
            ],
            Value::Int(17), // 1 + 2*2 + 3*4 = 1 + 4 + 12 = 17
        ),
        tc(
            vec![
                Value::tuple(vec![Value::Int(5), Value::Int(0), Value::Int(1)]),
                Value::Int(3),
            ],
            Value::Int(14), // 5 + 0*3 + 1*9 = 14
        ),
        tc(
            vec![
                Value::tuple(vec![Value::Int(1)]),
                Value::Int(99),
            ],
            Value::Int(1), // just the constant term
        ),
    ];
    let patterns = analyze_all_patterns(&cases);
    assert!(
        patterns.iter().any(|p| matches!(p, DetectedPattern::Polynomial)),
        "should detect Polynomial pattern"
    );
}

#[test]
fn analyzer_detect_linear_search() {
    let cases = vec![
        tc(
            vec![
                Value::tuple(vec![Value::Int(1), Value::Int(3), Value::Int(5), Value::Int(7), Value::Int(9)]),
                Value::Int(5),
            ],
            Value::Int(2),
        ),
        tc(
            vec![
                Value::tuple(vec![Value::Int(10), Value::Int(20), Value::Int(30)]),
                Value::Int(30),
            ],
            Value::Int(2),
        ),
        tc(
            vec![
                Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
                Value::Int(4),
            ],
            Value::Int(-1),
        ),
    ];
    let patterns = analyze_all_patterns(&cases);
    assert!(
        patterns.iter().any(|p| matches!(p, DetectedPattern::LinearSearch)),
        "should detect LinearSearch pattern"
    );
}
