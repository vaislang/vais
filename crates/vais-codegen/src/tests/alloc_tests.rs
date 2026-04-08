use crate::CodeGenerator;
use vais_parser::parse;

#[test]
fn test_alloc_tracker_string_concat() {
    // Verify that track_alloc registers allocations and returns store IR.
    // generate_alloc_cleanup is currently disabled (returns empty) to avoid
    // use-after-free — so we only test the tracking + store IR path.
    let mut gen = CodeGenerator::new("test");
    let store_ir1 = gen.track_alloc("%ptr1".to_string());
    let store_ir2 = gen.track_alloc("%ptr2".to_string());
    assert!(
        store_ir1.contains("store i8* %ptr1"),
        "Expected store for ptr1, got: {}",
        store_ir1
    );
    assert!(
        store_ir2.contains("store i8* %ptr2"),
        "Expected store for ptr2, got: {}",
        store_ir2
    );
    assert_eq!(gen.fn_ctx.alloc_tracker.len(), 2);
    // Cleanup is disabled — should return empty
    let cleanup_ir = gen.generate_alloc_cleanup();
    assert!(
        cleanup_ir.is_empty(),
        "Expected empty cleanup (disabled), got: {}",
        cleanup_ir
    );
}

#[test]
fn test_alloc_tracker_clear() {
    let mut gen = CodeGenerator::new("test");
    let _store_ir = gen.track_alloc("%ptr1".to_string());
    assert!(!gen.fn_ctx.alloc_tracker.is_empty());
    gen.clear_alloc_tracker();
    assert!(gen.fn_ctx.alloc_tracker.is_empty());
    // After clearing, cleanup should produce empty string
    let cleanup_ir = gen.generate_alloc_cleanup();
    assert!(cleanup_ir.is_empty(), "Expected empty cleanup after clear");
}

#[test]
fn test_alloc_tracker_no_alloc() {
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(1, 2)
"#;
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    let ir = gen.generate_module(&module).unwrap();
    // Pure arithmetic should NOT have auto-free
    let add_fn = ir.find("define i64 @add").unwrap();
    let add_end = ir[add_fn..].find("\n}\n").unwrap() + add_fn;
    let add_ir = &ir[add_fn..add_end];
    assert!(
        !add_ir.contains("auto-free"),
        "Pure function should not have auto-free cleanup"
    );
}

#[test]
fn test_drop_function_level_no_recursion() {
    // Regression: Counter_drop must NOT recursively call itself.
    // The `self` param is a reference; only alloca-declared locals are auto-dropped.
    let source = r#"
S Counter { n: i64 }
W Drop { F drop(&self) -> i64 }
X Counter: Drop { F drop(&self) -> i64 { 0 } }
F count_to(limit: i64) -> i64 {
    c := Counter { n: limit }
    c.n
}
F main() -> i64 { count_to(42) }
"#;
    let module = parse(source).unwrap();
    let mut checker = vais_types::TypeChecker::new();
    checker.check_module(&module).unwrap();
    let mut gen = CodeGenerator::new("test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    let insts = checker.get_generic_instantiations();
    let ir = if insts.is_empty() {
        gen.generate_module(&module)
    } else {
        gen.generate_module_with_instantiations(&module, &insts)
    }
    .unwrap();
    // Counter_drop should be defined (not just called)
    assert!(
        ir.contains("define i64 @Counter_drop"),
        "Counter_drop should be defined"
    );
    // Counter_drop body should NOT contain a recursive call to itself
    let drop_body: String = ir
        .lines()
        .skip_while(|l| !l.contains("define i64 @Counter_drop"))
        .skip(1)
        .take_while(|l| !l.starts_with("define "))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !drop_body.contains("call i64 @Counter_drop"),
        "Counter_drop must not recursively call itself, got:\n{}",
        drop_body
    );
}

#[test]
fn test_block_scope_drop_ir_correct() {
    // Block-scoped named locals should have drop call in that block's IR,
    // using the single-pointer alloca directly.
    let source = r#"
S Token { value: i64 }
W Drop { F drop(&self) -> i64 }
X Token: Drop { F drop(&self) -> i64 { 0 } }
F process(flag: bool) -> i64 {
    result := mut 0
    I flag {
        t := Token { value: 42 }
        result = t.value
    }
    result
}
F main() -> i64 { process(true) }
"#;
    let module = parse(source).unwrap();
    let mut checker = vais_types::TypeChecker::new();
    checker.check_module(&module).unwrap();
    let mut gen = CodeGenerator::new("test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    let insts = checker.get_generic_instantiations();
    let ir = if insts.is_empty() {
        gen.generate_module(&module)
    } else {
        gen.generate_module_with_instantiations(&module, &insts)
    }
    .unwrap();
    // IR must contain Token_drop call in the then-block (block-scope drop)
    assert!(ir.contains("Token_drop"), "IR must contain Token_drop call");
    // With single-pointer layout, drop is called directly with %var (no load needed)
    assert!(
        ir.contains("__drop_ret_"),
        "Drop must call Token_drop with direct pointer"
    );
}
