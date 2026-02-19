use super::*;
use std::sync::atomic::Ordering;
use vais_ast::*;
use vais_parser::parse;

#[test]
fn test_interpreter_simple() {
    let source = "F main()->i64{42}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::new();
    interp.load_module(&ast);

    let result = interp.run_main().unwrap();
    assert_eq!(result.as_i64().unwrap(), 42);
}

#[test]
fn test_interpreter_arithmetic() {
    let source = "F main()->i64{1+2*3}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::new();
    interp.load_module(&ast);

    let result = interp.run_main().unwrap();
    assert_eq!(result.as_i64().unwrap(), 7);
}

#[test]
fn test_interpreter_function_call() {
    let source = "F add(a:i64,b:i64)->i64{a+b} F main()->i64{add(3,4)}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::new();
    interp.load_module(&ast);

    let result = interp.run_main().unwrap();
    assert_eq!(result.as_i64().unwrap(), 7);
}

#[test]
fn test_interpreter_if_else() {
    let source = "F main()->i64{I true{1}E{0}}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::new();
    interp.load_module(&ast);

    let result = interp.run_main().unwrap();
    assert_eq!(result.as_i64().unwrap(), 1);
}

#[test]
fn test_interpreter_local_variable() {
    let source = "F main()->i64{x:=10;x+5}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::new();
    interp.load_module(&ast);

    let result = interp.run_main().unwrap();
    assert_eq!(result.as_i64().unwrap(), 15);
}

#[test]
fn test_profiling_execution_count() {
    let source = "F foo()->i64{1} F main()->i64{foo();foo();foo();0}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::new();
    interp.load_module(&ast);
    interp.run_main().unwrap();

    let profile = interp.get_profile("foo").unwrap();
    assert_eq!(profile.execution_count.load(Ordering::Relaxed), 3);
}

#[test]
fn test_tier_promotion_detection() {
    let thresholds = TierThresholds {
        interpreter_to_baseline: 2,
        baseline_to_optimizing: 10,
    };

    let source = "F hot()->i64{1} F main()->i64{hot();hot();hot();0}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::with_thresholds(thresholds);
    interp.load_module(&ast);
    interp.run_main().unwrap();

    assert!(interp.should_promote("hot").is_some());
    assert_eq!(interp.should_promote("hot"), Some(Tier::Baseline));
}

#[test]
fn test_tiered_jit_basic() {
    let source = "F main()->i64{42}";
    let ast = parse(source).unwrap();

    let mut jit = TieredJit::new().unwrap();
    let result = jit.run_main(&ast).unwrap();

    assert_eq!(result, 42);
}

#[test]
fn test_function_stats() {
    let source = "F foo()->i64{1} F main()->i64{foo();0}";
    let ast = parse(source).unwrap();

    let mut jit = TieredJit::new().unwrap();
    jit.run_main(&ast).unwrap();

    let stats = jit.get_function_stats("foo").unwrap();
    assert_eq!(stats.execution_count, 1);
    assert_eq!(stats.current_tier, Tier::Interpreter);
}

#[test]
fn test_tier_names() {
    assert_eq!(Tier::Interpreter.name(), "Interpreter");
    assert_eq!(Tier::Baseline.name(), "Baseline JIT");
    assert_eq!(Tier::Optimizing.name(), "Optimizing JIT");
}

#[test]
fn test_tier_ordering() {
    assert!(Tier::Interpreter < Tier::Baseline);
    assert!(Tier::Baseline < Tier::Optimizing);
}

#[test]
fn test_hot_function_detection() {
    let thresholds = TierThresholds {
        interpreter_to_baseline: 5,
        baseline_to_optimizing: 50,
    };

    let source = "F loop_func()->i64{x:=0;L{I x>=10{R x} x:=x+1}0} F main()->i64{loop_func()}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::with_thresholds(thresholds.clone());
    interp.load_module(&ast);

    // Execute multiple times to trigger promotion detection
    for _ in 0..6 {
        let _ = interp.call_function("loop_func", &[]);
    }

    // Should suggest promotion to baseline
    assert_eq!(interp.should_promote("loop_func"), Some(Tier::Baseline));
}

#[test]
fn test_tier_promotion_baseline_to_optimizing() {
    let thresholds = TierThresholds {
        interpreter_to_baseline: 2,
        baseline_to_optimizing: 5,
    };

    let source = "F hot()->i64{42} F main()->i64{hot()}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::with_thresholds(thresholds);
    interp.load_module(&ast);

    // Execute 6 times (exceeds baseline threshold)
    for _ in 0..6 {
        let _ = interp.call_function("hot", &[]);
    }

    // After 6 executions, should suggest promotion
    let promotion = interp.should_promote("hot");
    assert!(promotion.is_some());

    // Manually promote to baseline
    if let Some(profile) = interp.get_profile("hot") {
        let mut tier = profile
            .current_tier
            .write()
            .expect("current_tier lock poisoned");
        *tier = Tier::Baseline;
    }

    // Continue execution to hit optimizing threshold
    for _ in 0..50 {
        let _ = interp.call_function("hot", &[]);
    }

    // Now should suggest optimizing tier
    assert_eq!(interp.should_promote("hot"), Some(Tier::Optimizing));
}

#[test]
fn test_branch_profiling() {
    let source = "F branch_test(x:i64)->i64{I x>5{1}E{0}} F main()->i64{branch_test(10)}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::new();
    interp.load_module(&ast);

    // Execute with different values
    let _ = interp.call_function("branch_test", &[Value::I64(10)]);
    let _ = interp.call_function("branch_test", &[Value::I64(3)]);
    let _ = interp.call_function("branch_test", &[Value::I64(7)]);

    let profile = interp.get_profile("branch_test").unwrap();

    // Should have recorded branch outcomes
    let branch_counts = profile
        .branch_counts
        .read()
        .expect("branch_counts lock poisoned");
    assert!(!branch_counts.is_empty());
}

#[test]
fn test_loop_profiling() {
    let source = "F loop_test()->i64{i:=0;L{I i>=5{R i} i:=i+1}0} F main()->i64{loop_test()}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::new();
    interp.load_module(&ast);

    let _ = interp.run_main();

    let profile = interp.get_profile("loop_test").unwrap();

    // Should have recorded loop iterations
    let loop_counts = profile
        .loop_counts
        .read()
        .expect("loop_counts lock poisoned");
    assert!(!loop_counts.is_empty());
}

#[test]
fn test_tiered_jit_with_custom_thresholds() {
    let thresholds = TierThresholds {
        interpreter_to_baseline: 100,
        baseline_to_optimizing: 500,
    };

    let source = "F compute()->i64{10+20+30} F main()->i64{compute()}";
    let ast = parse(source).unwrap();

    let mut jit = TieredJit::with_thresholds(thresholds).unwrap();
    let result = jit.run_main(&ast).unwrap();

    assert_eq!(result, 60);

    // With high thresholds and single execution, should stay in interpreter tier
    let tier = jit.get_function_tier("main");
    assert!(tier == Tier::Interpreter || tier == Tier::Baseline);
}

#[test]
fn test_hot_path_score_calculation() {
    let source =
        "F loop_heavy()->i64{x:=0;L{I x>=100{R x} x:=x+1}0} F main()->i64{loop_heavy()}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::new();
    interp.load_module(&ast);

    // Execute function
    let _ = interp.run_main();

    let profile = interp.get_profile("loop_heavy").unwrap();
    profile.update_hot_path_score();

    let score = *profile
        .hot_path_score
        .read()
        .expect("hot_path_score lock poisoned");

    // Score should be high due to many loop iterations
    assert!(
        score > 50.0,
        "Hot path score should be > 50.0, got {}",
        score
    );
}

#[test]
fn test_deoptimization() {
    let source = "F hot()->i64{42} F main()->i64{hot()}";
    let ast = parse(source).unwrap();

    let mut jit = TieredJit::new().unwrap();
    jit.load_module(&ast);

    // Get profile and manually promote to Baseline
    if let Some(profile) = jit.interpreter.get_profile("hot") {
        let mut tier = profile
            .current_tier
            .write()
            .expect("current_tier lock poisoned");
        *tier = Tier::Baseline;
    }

    // Verify tier
    assert_eq!(jit.get_function_tier("hot"), Tier::Baseline);

    // Deoptimize
    jit.deoptimize("hot").unwrap();

    // Should be back to Interpreter
    assert_eq!(jit.get_function_tier("hot"), Tier::Interpreter);

    // Deopt count should be 1
    let stats = jit.get_function_stats("hot").unwrap();
    assert_eq!(stats.deopt_count, 1);
}

#[test]
fn test_deopt_blacklist() {
    let source = "F unstable()->i64{1} F main()->i64{unstable()}";
    let ast = parse(source).unwrap();

    let mut jit = TieredJit::new().unwrap();
    jit.load_module(&ast);

    let profile = jit.interpreter.get_profile("unstable").unwrap();

    // Manually promote and deopt 3 times
    for _ in 0..3 {
        {
            let mut tier = profile
                .current_tier
                .write()
                .expect("current_tier lock poisoned");
            *tier = Tier::Baseline;
        }
        jit.deoptimize("unstable").unwrap();
    }

    // Should be blacklisted
    assert!(profile.is_blacklisted());

    // should_promote should return None for blacklisted functions
    assert!(jit.interpreter.should_promote("unstable").is_none());
}

#[test]
fn test_osr_point_registration() {
    let jit = TieredJit::new().unwrap();

    let osr_point = OsrPoint {
        function: "hot_loop".to_string(),
        loop_id: 12345,
        target_tier: Tier::Baseline,
        iteration_threshold: 1000,
    };

    jit.register_osr_point(osr_point.clone());

    let points = jit.osr_points.read().expect("osr_points lock poisoned");
    assert_eq!(points.len(), 1);
    assert_eq!(points[0].function, "hot_loop");
    assert_eq!(points[0].loop_id, 12345);
}

#[test]
fn test_osr_check_threshold() {
    let jit = TieredJit::new().unwrap();

    let osr_point = OsrPoint {
        function: "compute".to_string(),
        loop_id: 99,
        target_tier: Tier::Optimizing,
        iteration_threshold: 500,
    };

    jit.register_osr_point(osr_point);

    // Below threshold - no promotion
    assert!(jit.check_osr("compute", 99, 300).is_none());

    // At threshold - should promote
    let tier = jit.check_osr("compute", 99, 500);
    assert_eq!(tier, Some(Tier::Optimizing));

    // Above threshold - should promote
    let tier = jit.check_osr("compute", 99, 1000);
    assert_eq!(tier, Some(Tier::Optimizing));
}

#[test]
fn test_dynamic_tier_promotion_with_score() {
    let thresholds = TierThresholds {
        interpreter_to_baseline: 50,
        baseline_to_optimizing: 200,
    };

    let source = "F work()->i64{x:=0;L{I x>=20{R x} x:=x+1}0} F main()->i64{work()}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::with_thresholds(thresholds);
    interp.load_module(&ast);

    // Execute multiple times to build up score
    for _ in 0..10 {
        let _ = interp.call_function("work", &[]);
    }

    let profile = interp.get_profile("work").unwrap();
    profile.update_hot_path_score();

    let score = *profile
        .hot_path_score
        .read()
        .expect("hot_path_score lock poisoned");

    // With 10 executions and ~20 loop iterations each, score should be high
    assert!(score > 50.0, "Score should exceed baseline threshold");

    // Should suggest promotion
    let promotion = interp.should_promote("work");
    assert_eq!(promotion, Some(Tier::Baseline));
}

#[test]
fn test_total_loop_iterations() {
    let source = "F looper()->i64{x:=0;L{I x>=10{R x} x:=x+1}0} F main()->i64{looper()}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::new();
    interp.load_module(&ast);

    let _ = interp.run_main();

    let profile = interp.get_profile("looper").unwrap();
    let total = profile.total_loop_iterations.load(Ordering::Relaxed);

    // Should have accumulated loop iterations
    assert!(
        total >= 10,
        "Expected at least 10 loop iterations, got {}",
        total
    );
}

#[test]
fn test_get_all_stats() {
    let source = "F foo()->i64{1} F bar()->i64{2} F main()->i64{foo()+bar()}";
    let ast = parse(source).unwrap();

    let mut jit = TieredJit::new().unwrap();
    jit.load_module(&ast);
    let _ = jit.interpreter.run_main();

    let all_stats = jit.get_all_stats();

    // Should have stats for all functions
    assert!(all_stats.contains_key("foo"));
    assert!(all_stats.contains_key("bar"));
    assert!(all_stats.contains_key("main"));
}

#[test]
fn test_branch_bias_score() {
    let source = "F biased(x:i64)->i64{I x>0{1}E{0}} F main()->i64{biased(10)}";
    let ast = parse(source).unwrap();

    let mut interp = Interpreter::new();
    interp.load_module(&ast);

    // Execute many times with same bias (always true)
    for _ in 0..20 {
        let _ = interp.call_function("biased", &[Value::I64(10)]);
    }

    let profile = interp.get_profile("biased").unwrap();
    profile.update_hot_path_score();

    let score = *profile
        .hot_path_score
        .read()
        .expect("hot_path_score lock poisoned");

    // Score should include branch bias component
    assert!(score > 0.0, "Score should be > 0 with branch bias");
}

#[test]
fn test_last_promoted_at() {
    let source = "F func()->i64{1} F main()->i64{func()}";
    let ast = parse(source).unwrap();

    let mut jit = TieredJit::new().unwrap();
    jit.load_module(&ast);

    let profile = jit.interpreter.get_profile("func").unwrap();

    // Execute a few times
    for _ in 0..5 {
        let _ = jit.interpreter.call_function("func", &[]);
    }

    let count_before = profile.execution_count.load(Ordering::Relaxed);

    // Mark as promoted
    profile.mark_promoted();

    let last_promoted = *profile
        .last_promoted_at
        .read()
        .expect("last_promoted_at lock poisoned");
    assert_eq!(last_promoted, count_before);
}
