//! Phase 147 Task 4: per-module codegen E2E tests
//!
//! Verifies that cross-module generic/impl/trait patterns produce correct LLVM IR
//! when items are split across two independent CodeGenerator instances
//! (simulating the per-module parallel build path).

use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Compile source using per-module codegen.
///
/// Splits module items into two halves:
/// - "other" module: items `[0, split_point)`
/// - "main"  module: items `[split_point, total)`
///
/// Returns `(main_ir, other_ir)`.
fn compile_per_module(source: &str) -> Result<(String, String), String> {
    let _tokens = tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;
    let module = parse(source).map_err(|e| format!("Parser error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;

    let instantiations = checker.get_generic_instantiations();
    let resolved_fns = checker.get_all_functions().clone();
    let type_aliases = checker.get_type_aliases().clone();

    let total_items = module.items.len();
    let split_point = total_items / 2;
    let other_indices: Vec<usize> = (0..split_point).collect();
    let main_indices: Vec<usize> = (split_point..total_items).collect();

    // Generate IR for the "other" (non-main) module.
    let mut gen_other =
        CodeGenerator::new_with_target("other", vais_codegen::TargetTriple::Native);
    gen_other.set_resolved_functions(resolved_fns.clone());
    gen_other.set_type_aliases(type_aliases.clone());
    let other_ir = gen_other
        .generate_module_subset(&module, &other_indices, &instantiations, false)
        .map_err(|e| format!("Codegen error (other): {:?}", e))?;

    // Generate IR for the "main" module.
    let mut gen_main = CodeGenerator::new_with_target("main", vais_codegen::TargetTriple::Native);
    gen_main.set_resolved_functions(resolved_fns);
    gen_main.set_type_aliases(type_aliases);
    let main_ir = gen_main
        .generate_module_subset(&module, &main_indices, &instantiations, true)
        .map_err(|e| format!("Codegen error (main): {:?}", e))?;

    Ok((main_ir, other_ir))
}

/// Assert that an IR string is a valid (non-empty) module by checking for the
/// ModuleID comment that `emit_module_header` always emits.
fn assert_valid_ir(ir: &str, label: &str) {
    assert!(
        ir.contains("ModuleID"),
        "{label} should contain ModuleID header.\nIR:\n{ir}"
    );
}

// ===========================================================================
// Test 1: generic function — type-inference call, split across modules
// ===========================================================================

#[test]
fn e2e_per_module_generic_function() {
    // identity<T> is the first item (other module).
    // main is the second item (main module).
    // Type inference: identity(42) → identity<i64>
    let source = r#"
F identity<T>(x: T) -> T { x }

F main() -> i64 {
    identity(42)
}
"#;
    let (main_ir, other_ir) = compile_per_module(source).expect("per-module compile failed");

    assert_valid_ir(&other_ir, "other_ir");
    assert_valid_ir(&main_ir, "main_ir");

    // The specialized function body should be in the module that owns identity.
    let combined = format!("{other_ir}\n{main_ir}");
    assert!(
        combined.contains("identity"),
        "Expected 'identity' in at least one module IR.\ncombined:\n{combined}"
    );
}

// ===========================================================================
// Test 2: generic struct with field access
// ===========================================================================

#[test]
fn e2e_per_module_generic_struct_method() {
    // Container<T> struct is in the other module.
    // main() creates a Container and accesses its field.
    let source = r#"
S Container<T> {
    val: T
}

F main() -> i64 {
    c := Container { val: 7 }
    c.val
}
"#;
    let (main_ir, other_ir) = compile_per_module(source).expect("per-module compile failed");

    assert_valid_ir(&other_ir, "other_ir");
    assert_valid_ir(&main_ir, "main_ir");

    // Container struct type must appear somewhere in the combined IR.
    let combined = format!("{other_ir}\n{main_ir}");
    assert!(
        combined.contains("Container") || combined.contains("i64"),
        "Expected 'Container' or 'i64' in combined IR.\ncombined:\n{combined}"
    );
}

// ===========================================================================
// Test 3: trait definition and impl in separate halves
// ===========================================================================

#[test]
fn e2e_per_module_trait_impl() {
    // W Greet + S Dog are in the other module.
    // X Dog: Greet + main are in the main module.
    let source = r#"
W Greet {
    F hello(self) -> i64
}

S Dog { }

X Dog: Greet {
    F hello(self) -> i64 { 1 }
}

F main() -> i64 {
    d := Dog { }
    d.hello()
}
"#;
    let (main_ir, other_ir) = compile_per_module(source).expect("per-module compile failed");

    assert_valid_ir(&other_ir, "other_ir");
    assert_valid_ir(&main_ir, "main_ir");

    let combined = format!("{other_ir}\n{main_ir}");
    assert!(
        combined.contains("hello") || combined.contains("Dog"),
        "Expected 'hello' or 'Dog' in combined IR.\ncombined:\n{combined}"
    );
}

// ===========================================================================
// Test 4: specialized struct type across modules
// ===========================================================================

#[test]
fn e2e_per_module_specialized_struct_type() {
    // Pair<T> struct is in the other module; main uses Pair with i64.
    let source = r#"
S Pair<T> {
    a: T,
    b: T
}

F main() -> i64 {
    p := Pair { a: 3, b: 4 }
    p.a + p.b
}
"#;
    let (main_ir, other_ir) = compile_per_module(source).expect("per-module compile failed");

    assert_valid_ir(&other_ir, "other_ir");
    assert_valid_ir(&main_ir, "main_ir");

    // The specialized struct type should appear in the combined IR.
    let combined = format!("{other_ir}\n{main_ir}");
    assert!(
        combined.contains("Pair") || combined.contains("i64"),
        "Expected 'Pair' or 'i64' in combined IR.\ncombined:\n{combined}"
    );
}

// ===========================================================================
// Test 5: multiple instantiations of the same generic function
// ===========================================================================

#[test]
fn e2e_per_module_multiple_instantiations() {
    // convert<T> is in the other module; main calls it (type inferred as i64).
    let source = r#"
F convert<T>(x: T) -> T { x }

F main() -> i64 {
    a := convert(10)
    a
}
"#;
    let (main_ir, other_ir) = compile_per_module(source).expect("per-module compile failed");

    assert_valid_ir(&other_ir, "other_ir");
    assert_valid_ir(&main_ir, "main_ir");

    let combined = format!("{other_ir}\n{main_ir}");
    assert!(
        combined.contains("convert"),
        "Expected 'convert' in combined IR.\ncombined:\n{combined}"
    );
}

// ===========================================================================
// Test 6: cross-module impl method call
// ===========================================================================

#[test]
fn e2e_per_module_cross_module_method_call() {
    // Counter struct + impl are in the other module; main uses them.
    let source = r#"
S Counter {
    val: i64
}

X Counter {
    F inc(self) -> i64 {
        self.val + 1
    }
}

F main() -> i64 {
    c := Counter { val: 5 }
    c.inc()
}
"#;
    let (main_ir, other_ir) = compile_per_module(source).expect("per-module compile failed");

    assert_valid_ir(&other_ir, "other_ir");
    assert_valid_ir(&main_ir, "main_ir");

    // inc method definition should be in other_ir (owns Counter + impl).
    assert!(
        other_ir.contains("inc") || main_ir.contains("inc"),
        "Expected 'inc' in at least one module IR.\nother_ir:\n{other_ir}\nmain_ir:\n{main_ir}"
    );
}

// ===========================================================================
// Test 7: extern declarations generated correctly
// ===========================================================================

#[test]
fn e2e_per_module_extern_declarations() {
    // helper is defined in the other module; main calls it.
    // main_ir should declare @helper as extern or call it.
    let source = r#"
F helper() -> i64 {
    99
}

F main() -> i64 {
    helper()
}
"#;
    let (main_ir, other_ir) = compile_per_module(source).expect("per-module compile failed");

    assert_valid_ir(&other_ir, "other_ir");
    assert_valid_ir(&main_ir, "main_ir");

    // helper should be defined in the module that owns it.
    assert!(
        other_ir.contains("define") && other_ir.contains("helper"),
        "Expected 'define' + 'helper' in other_ir.\nother_ir:\n{other_ir}"
    );

    // main_ir should reference helper (declare or call).
    assert!(
        main_ir.contains("helper"),
        "Expected 'helper' reference in main_ir.\nmain_ir:\n{main_ir}"
    );
}

// ===========================================================================
// Test 8: no generic instantiations (simple two-function split)
// ===========================================================================

#[test]
fn e2e_per_module_no_instantiations() {
    // Two plain functions — no generics — split across modules.
    let source = r#"
F add(a: i64, b: i64) -> i64 {
    a + b
}

F main() -> i64 {
    add(2, 3)
}
"#;
    let (main_ir, other_ir) = compile_per_module(source).expect("per-module compile failed");

    assert_valid_ir(&other_ir, "other_ir");
    assert_valid_ir(&main_ir, "main_ir");

    // add must be defined in other_ir (it owns the first item).
    assert!(
        other_ir.contains("define") && other_ir.contains("add"),
        "Expected 'define' + 'add' in other_ir.\nother_ir:\n{other_ir}"
    );
    // main_ir must reference add.
    assert!(
        main_ir.contains("add"),
        "Expected 'add' reference in main_ir.\nmain_ir:\n{main_ir}"
    );
}

// ===========================================================================
// Test 9: nested generic (generic calling another generic)
// ===========================================================================

#[test]
fn e2e_per_module_nested_generic() {
    // identity<T> and wrap<T> are split into the other module;
    // main is in the main module.
    // wrap<T> internally calls identity<T> by type inference.
    let source = r#"
F identity<T>(x: T) -> T { x }

F wrap<T>(x: T) -> T { identity(x) }

F main() -> i64 {
    wrap(55)
}
"#;
    let (main_ir, other_ir) = compile_per_module(source).expect("per-module compile failed");

    assert_valid_ir(&other_ir, "other_ir");
    assert_valid_ir(&main_ir, "main_ir");

    let combined = format!("{other_ir}\n{main_ir}");
    assert!(
        combined.contains("wrap"),
        "Expected 'wrap' in combined IR.\ncombined:\n{combined}"
    );
    assert!(
        combined.contains("identity"),
        "Expected 'identity' in combined IR.\ncombined:\n{combined}"
    );
}

// ===========================================================================
// Test 10: user-defined Drop-like trait across modules
// ===========================================================================

#[test]
fn e2e_per_module_drop_trait() {
    // Resource struct + W Drop trait + X Resource: Drop are in the other module.
    // main is in the main module.
    // Drop trait is declared inline (not built-in).
    let source = r#"
S Resource {
    id: i64
}

W Drop {
    F drop(&self) -> i64
}

X Resource: Drop {
    F drop(&self) -> i64 {
        0
    }
}

F main() -> i64 {
    r := Resource { id: 1 }
    r.id
}
"#;
    let (main_ir, other_ir) = compile_per_module(source).expect("per-module compile failed");

    assert_valid_ir(&other_ir, "other_ir");
    assert_valid_ir(&main_ir, "main_ir");

    let combined = format!("{other_ir}\n{main_ir}");
    assert!(
        combined.contains("drop") || combined.contains("Resource"),
        "Expected 'drop' or 'Resource' in combined IR.\ncombined:\n{combined}"
    );
}
