//! Vais LLVM Code Generator
//!
//! Generates LLVM IR from typed AST for native code generation.
//!
//! # Backends
//!
//! This crate supports two code generation backends:
//!
//! - **text-codegen** (default): Generates LLVM IR as text, then compiles via clang.
//!   Does not require LLVM installation.
//!
//! - **inkwell-codegen**: Uses inkwell bindings for direct LLVM API access.
//!   Provides better type safety and performance. Requires LLVM 17+.
//!
//! # Feature Flags
//!
//! - `text-codegen` (default): Enable text-based IR generation
//! - `inkwell-codegen`: Enable inkwell-based generation (requires LLVM 17+)

pub mod abi;
#[cfg(test)]
mod abi_tests;
pub mod advanced_opt;
mod builtins;
#[cfg(test)]
mod cache_tests;
mod contracts;
mod control_flow;
pub mod cross_compile;
pub mod debug;
mod diagnostics;
mod emit;
mod error;
mod expr;
mod expr_helpers;
mod expr_helpers_call;
mod expr_helpers_control;
mod expr_helpers_data;
mod expr_helpers_misc;
mod expr_visitor;
mod ffi;
mod function_gen;
mod generate_expr;
mod generate_expr_call;
mod generate_expr_loop;
mod generate_expr_struct;
mod generics_helpers;
mod helpers;
mod init;
mod lambda_closure;
mod module_gen;
#[cfg(test)]
mod nested_field_tests;
pub mod optimize;
pub mod parallel;
mod registration;
mod state;
mod stmt;
mod stmt_visitor;
mod string_ops;
#[cfg(test)]
mod struct_param_tests;
mod target;
mod trait_dispatch;
mod type_inference;
mod types;
pub mod visitor;
pub mod vtable;
#[cfg(test)]
mod vtable_tests;
pub mod wasm_component;
mod wasm_helpers;

// Inkwell-based code generator (optional)
#[cfg(feature = "inkwell-codegen")]
pub mod inkwell;

#[cfg(feature = "inkwell-codegen")]
pub use inkwell::InkwellCodeGenerator;

pub use visitor::{ExprVisitor, ItemVisitor, StmtVisitor};

pub use debug::{DebugConfig, DebugInfoBuilder};

// Re-export error types
pub use error::{CodegenError, CodegenResult};

// Re-export state types
pub use state::DecreasesInfo;
pub(crate) use state::{
    ContractState, FunctionContext, GenericState, LambdaState, StringPool, TypeRegistry,
};

use std::collections::HashMap;
use vais_ast::*;
use vais_types::ResolvedType;

/// Maximum recursion depth for type resolution to prevent stack overflow
/// This limit protects against infinite recursive types like: type A = B; type B = A;
const MAX_TYPE_RECURSION_DEPTH: usize = 128;

/// Escape a string for use in LLVM IR string constants.
///
/// Handles all control characters (0x00-0x1F, 0x7F) and special characters
/// that need escaping in LLVM IR constant strings.
pub(crate) fn escape_llvm_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'\\' => result.push_str("\\5C"),
            b'"' => result.push_str("\\22"),
            b'\n' => result.push_str("\\0A"),
            b'\r' => result.push_str("\\0D"),
            b'\t' => result.push_str("\\09"),
            b'\0' => result.push_str("\\00"),
            0x01..=0x08 | 0x0B..=0x0C | 0x0E..=0x1F | 0x7F => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                result.push('\\');
                result.push(HEX[(byte >> 4) as usize] as char);
                result.push(HEX[(byte & 0x0F) as usize] as char);
            }
            _ => result.push(byte as char),
        }
    }
    result
}

#[cfg(test)]
use diagnostics::edit_distance;
pub(crate) use diagnostics::{format_did_you_mean, suggest_similar};
#[cfg(test)]
pub(crate) use diagnostics::suggest_type_conversion;
pub use target::TargetTriple;
// Re-export type structs from types module
pub(crate) use types::*;

/// Result of generating a block of statements
/// (value, ir_code, is_terminated)
/// is_terminated is true if the block ends with break, continue, or return
type _BlockResult = (String, String, bool);

/// LLVM IR Code Generator for Vais 0.0.1
///
/// Generates LLVM IR text from typed AST for native code generation via clang.
pub struct CodeGenerator {
    // Type definitions registry
    pub(crate) types: TypeRegistry,

    // Generic type system state
    pub(crate) generics: GenericState,

    // Current function compilation context
    pub(crate) fn_ctx: FunctionContext,

    // String constant pool
    pub(crate) strings: StringPool,

    // Lambda/closure/async state
    pub(crate) lambdas: LambdaState,

    // Module name
    module_name: String,

    // Target architecture
    target: TargetTriple,

    // Flag to emit unwrap panic message and abort declaration
    needs_unwrap_panic: bool,

    // Flag to emit string helper functions
    needs_string_helpers: bool,

    // Debug info builder for DWARF metadata generation
    debug_info: DebugInfoBuilder,

    // Cache for type_to_llvm conversions to avoid repeated computations
    // Uses interior mutability to allow caching through immutable references
    type_to_llvm_cache: std::cell::RefCell<HashMap<String, String>>,

    // GC mode configuration
    gc_enabled: bool,
    gc_threshold: usize,

    // VTable generator for trait objects (dyn Trait)
    // Uses trait definitions from `self.types.trait_defs` (TypeRegistry) for vtable layout
    vtable_generator: vtable::VtableGenerator,

    // Release mode flag (disables contract checks)
    release_mode: bool,

    // Contract verification state (old() snapshots, decreases, contract strings)
    contracts: ContractState,

    // Type recursion depth tracking (prevents infinite recursion)
    type_recursion_depth: std::cell::Cell<usize>,

    // WASM import metadata: function_name -> (module_name, import_name)
    pub(crate) wasm_imports: HashMap<String, (String, String)>,

    // WASM export metadata: function_name -> export_name
    pub(crate) wasm_exports: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_parser::parse;

    #[test]
    fn test_simple_function() {
        let source = "F add(a:i64,b:i64)->i64=a+b";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @add"));
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_fibonacci() {
        let source = "F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @fib"));
        assert!(ir.contains("call i64 @fib"));
    }

    #[test]
    fn test_if_else() {
        // I cond { then } E { else }
        let source = "F max(a:i64,b:i64)->i64{I a>b{R a}E{R b}}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @max"));
        assert!(ir.contains("br i1"));
        assert!(ir.contains("then"));
        assert!(ir.contains("else"));
    }

    #[test]
    fn test_loop_with_condition() {
        // L pattern:iter { body } - `L _:condition{body}` for while loop
        let source = "F countdown(n:i64)->i64{x:=n;L _:x>0{x=x-1};x}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @countdown"));
        assert!(ir.contains("loop.start"));
        assert!(ir.contains("loop.body"));
        assert!(ir.contains("loop.end"));
    }

    #[test]
    fn test_array_literal() {
        let source = "F get_arr()->*i64=[1,2,3]";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("alloca [3  x i64]"));
        assert!(ir.contains("getelementptr"));
        assert!(ir.contains("store i64"));
    }

    #[test]
    fn test_array_index() {
        let source = "F get_elem(arr:*i64, idx:i64)->i64=arr[idx]";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("getelementptr i64, i64*"));
        assert!(ir.contains("load i64, i64*"));
    }

    #[test]
    fn test_struct_codegen() {
        let source = "S Point{x:i64,y:i64}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("%Point = type { i64, i64 }"));
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_module() {
        let source = "";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Should generate valid LLVM IR even with empty module
        assert!(ir.contains("source_filename"));
    }

    #[test]
    fn test_minimal_function() {
        let source = "F f()->()=()";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define void @f"));
        assert!(ir.contains("ret void"));
    }

    #[test]
    fn test_function_returning_unit() {
        let source = "F void_fn()->(){}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define void @void_fn"));
    }

    #[test]
    fn test_empty_struct() {
        let source = "S Empty{}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Empty struct should still generate a type
        assert!(ir.contains("%Empty = type"));
    }

    #[test]
    fn test_single_field_struct() {
        let source = "S Single{x:i64}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("%Single = type { i64 }"));
    }

    #[test]
    fn test_enum_with_variants() {
        let source = "E Color{Red,Green,Blue}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Enum should generate a type
        assert!(ir.contains("%Color = type"));
    }

    #[test]
    fn test_i64_max_value() {
        let source = "F max()->i64=9223372036854775807";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("9223372036854775807"));
    }

    #[test]
    fn test_negative_number() {
        let source = "F neg()->i64=-42";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Negative numbers involve subtraction from 0
        assert!(ir.contains("sub i64 0, 42"));
    }

    #[test]
    fn test_zero_value() {
        let source = "F zero()->i64=0";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("ret i64 0"));
    }

    #[test]
    fn test_float_values() {
        let source = "F pi()->f64=3.141592653589793";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("double"));
    }

    #[test]
    fn test_boolean_true() {
        let source = "F yes()->bool=true";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("ret i1 true") || ir.contains("ret i1 1"));
    }

    #[test]
    fn test_boolean_false() {
        let source = "F no()->bool=false";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("ret i1 false") || ir.contains("ret i1 0"));
    }

    #[test]
    fn test_empty_string() {
        let source = r#"F empty()->str="""#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Should handle empty string
        assert!(ir.contains("@str") || ir.contains("i8*"));
    }

    #[test]
    fn test_string_with_escape() {
        let source = r#"F escaped()->str="hello\nworld""#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Should handle escape sequences
        assert!(ir.contains("@str"));
    }

    #[test]
    fn test_empty_array() {
        let source = "F empty_arr()->*i64=[]";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Empty array should still work
        assert!(ir.contains("define"));
    }

    #[test]
    fn test_nested_if_else() {
        let source = r#"
            F classify(x:i64)->i64{
                I x>0{
                    I x>100{2}E{1}
                }E{
                    I x<-100{-2}E{-1}
                }
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @classify"));
        // Should have multiple branches
        assert!(ir.contains("br i1"));
    }

    #[test]
    fn test_simple_match() {
        let source = "F digit(n:i64)->str=M n{0=>\"zero\",1=>\"one\",_=>\"other\"}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define"));
    }

    #[test]
    fn test_for_loop() {
        let source = "F sum_to(n:i64)->i64{s:=0;L i:0..n{s+=i};s}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @sum_to"));
        assert!(ir.contains("for.cond"));
        assert!(ir.contains("for.body"));
        assert!(ir.contains("for.inc"));
    }

    #[test]
    fn test_while_loop() {
        let source = "F count_down(n:i64)->i64{x:=n;L _:x>0{x-=1};x}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @count_down"));
    }

    #[test]
    fn test_infinite_loop_with_break() {
        let source = "F find()->i64{x:=0;L{I x>10{B x};x+=1};0}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @find"));
    }

    #[test]
    fn test_arithmetic_operations() {
        let source = "F math(a:i64,b:i64)->i64=a+b-a*b/a%b";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("add i64"));
        assert!(ir.contains("sub i64"));
        assert!(ir.contains("mul i64"));
        assert!(ir.contains("sdiv i64"));
        assert!(ir.contains("srem i64"));
    }

    #[test]
    fn test_comparison_operations() {
        let source = r#"
            F compare(a:i64,b:i64)->bool{
                x:=a<b;
                y:=a<=b;
                z:=a>b;
                w:=a>=b;
                u:=a==b;
                v:=a!=b;
                x
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("icmp slt"));
        assert!(ir.contains("icmp sle"));
        assert!(ir.contains("icmp sgt"));
        assert!(ir.contains("icmp sge"));
        assert!(ir.contains("icmp eq"));
        assert!(ir.contains("icmp ne"));
    }

    #[test]
    fn test_bitwise_operations() {
        let source = "F bits(a:i64,b:i64)->i64{x:=a&b;y:=a|b;z:=a^b;w:=a<<2;v:=a>>1;x}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("and i64"));
        assert!(ir.contains("or i64"));
        assert!(ir.contains("xor i64"));
        assert!(ir.contains("shl i64"));
        assert!(ir.contains("ashr i64"));
    }

    #[test]
    fn test_logical_operations() {
        let source = "F logic(a:bool,b:bool)->bool=a&&b||!a";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i1 @logic"));
    }

    #[test]
    fn test_unary_minus() {
        let source = "F negate(x:i64)->i64=-x";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("sub i64 0"));
    }

    #[test]
    fn test_bitwise_not() {
        let source = "F complement(x:i64)->i64=~x";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("xor i64") && ir.contains("-1"));
    }

    #[test]
    fn test_ternary_expression() {
        let source = "F abs(x:i64)->i64=x<0?-x:x";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @abs"));
        assert!(ir.contains("br i1"));
    }

    #[test]
    fn test_compound_assignment() {
        // In Vais, mutable variables use := for declaration
        let source = r#"
            F compound(x:i64)->i64{
                y:=x;
                y+=1;
                y-=2;
                y*=3;
                y/=4;
                y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @compound"));
    }

    #[test]
    fn test_struct_literal() {
        let source = r#"
            S Point{x:i64,y:i64}
            F origin()->Point=Point{x:0,y:0}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("%Point = type { i64, i64 }"));
        assert!(ir.contains("define %Point"));
    }

    #[test]
    fn test_struct_field_access() {
        let source = r#"
            S Point{x:i64,y:i64}
            F get_x(p:Point)->i64=p.x
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("getelementptr"));
    }

    #[test]
    fn test_lambda_simple() {
        let source = "F f()->i64{add:=|a:i64,b:i64|a+b;add(1,2)}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @f"));
    }

    #[test]
    fn test_recursive_factorial() {
        let source = "F factorial(n:i64)->i64=n<=1?1:n*@(n-1)";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @factorial"));
        assert!(ir.contains("call i64 @factorial"));
    }

    #[test]
    fn test_multiple_functions() {
        let source = r#"
            F add(a:i64,b:i64)->i64=a+b
            F sub(a:i64,b:i64)->i64=a-b
            F mul(a:i64,b:i64)->i64=a*b
            F test()->i64=mul(add(1,2),sub(5,2))
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @add"));
        assert!(ir.contains("define i64 @sub"));
        assert!(ir.contains("define i64 @mul"));
        assert!(ir.contains("define i64 @test"));
    }

    #[test]
    fn test_function_with_many_params() {
        let source = "F many(a:i64,b:i64,c:i64,d:i64,e:i64,f:i64,g:i64,h:i64)->i64=a+b+c+d+e+f+g+h";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // LLVM IR uses %a, %b etc, and the define line may not have spaces
        assert!(ir.contains("define i64 @many"));
        assert!(ir.contains("i64 %a"));
        assert!(ir.contains("i64 %h"));
    }

    #[test]
    fn test_all_integer_types() {
        let source = r#"
            F test_i8(x:i8)->i8=x
            F test_i16(x:i16)->i16=x
            F test_i32(x:i32)->i32=x
            F test_i64(x:i64)->i64=x
            F test_u8(x:u8)->u8=x
            F test_u16(x:u16)->u16=x
            F test_u32(x:u32)->u32=x
            F test_u64(x:u64)->u64=x
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i8 @test_i8"));
        assert!(ir.contains("define i16 @test_i16"));
        assert!(ir.contains("define i32 @test_i32"));
        assert!(ir.contains("define i64 @test_i64"));
        assert!(ir.contains("define i8 @test_u8"));
        assert!(ir.contains("define i16 @test_u16"));
        assert!(ir.contains("define i32 @test_u32"));
        assert!(ir.contains("define i64 @test_u64"));
    }

    #[test]
    fn test_float_types() {
        let source = r#"
            F test_f32(x:f32)->f32=x
            F test_f64(x:f64)->f64=x
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define float @test_f32"));
        assert!(ir.contains("define double @test_f64"));
    }

    #[test]
    fn test_deeply_nested_expression() {
        let source = "F deep(a:i64)->i64=((((a+1)+2)+3)+4)+5";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @deep"));
    }

    #[test]
    fn test_mixed_arithmetic_precedence() {
        let source = "F prec(a:i64,b:i64,c:i64)->i64=a+b*c";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Should multiply first then add (precedence)
        assert!(ir.contains("mul i64"));
        assert!(ir.contains("add i64"));
    }

    // ==================== Generic Instantiation Tests ====================

    #[test]
    fn test_generate_specialized_function() {
        use vais_types::TypeChecker;

        let source = r#"
            F identity<T>(x:T)->T=x
            F main()->i64=identity(42)
        "#;
        let module = parse(source).unwrap();

        // First, type check to get instantiations
        let mut checker = TypeChecker::new();
        checker.check_module(&module).unwrap();
        let instantiations = checker.get_generic_instantiations();

        // Generate code with instantiations
        let mut gen = CodeGenerator::new("test");
        let ir = gen
            .generate_module_with_instantiations(&module, instantiations)
            .unwrap();

        // Should contain specialized function identity$i64
        assert!(
            ir.contains("define i64 @identity$i64"),
            "Expected identity$i64 in IR: {}",
            ir
        );
        assert!(ir.contains("ret i64 %x"), "Expected return in identity$i64");
    }

    #[test]
    fn test_generate_specialized_struct_type() {
        use vais_types::TypeChecker;

        // Test that generic struct type definition is specialized
        // Note: Full struct literal code generation with generics requires additional work
        // This test verifies the type definition is generated correctly
        let source = r#"
            S Pair<T>{first:T,second:T}
            F main()->i64{
                p:=Pair{first:1,second:2};
                p.first
            }
        "#;
        let module = parse(source).unwrap();

        // Type check to get instantiations
        let mut checker = TypeChecker::new();
        checker.check_module(&module).unwrap();
        let instantiations = checker.get_generic_instantiations();

        // Verify instantiation was recorded
        let pair_inst = instantiations.iter().find(|i| i.base_name == "Pair");
        assert!(
            pair_inst.is_some(),
            "Expected Pair instantiation to be recorded"
        );

        // Verify mangled name
        let inst = pair_inst.unwrap();
        assert_eq!(
            inst.mangled_name, "Pair$i64",
            "Expected mangled name Pair$i64, got {}",
            inst.mangled_name
        );
    }

    #[test]
    fn test_multiple_instantiations() {
        use vais_types::TypeChecker;

        let source = r#"
            F identity<T>(x:T)->T=x
            F main()->f64{
                a:=identity(42);
                b:=identity(3.14);
                b
            }
        "#;
        let module = parse(source).unwrap();

        // Type check to get instantiations
        let mut checker = TypeChecker::new();
        checker.check_module(&module).unwrap();
        let instantiations = checker.get_generic_instantiations();

        // Should have at least 2 instantiations
        assert!(
            instantiations.len() >= 2,
            "Expected at least 2 instantiations, got {}",
            instantiations.len()
        );

        // Generate code with instantiations
        let mut gen = CodeGenerator::new("test");
        let ir = gen
            .generate_module_with_instantiations(&module, instantiations)
            .unwrap();

        // Should contain both specialized functions
        assert!(ir.contains("@identity$i64"), "Expected identity$i64 in IR");
        assert!(ir.contains("@identity$f64"), "Expected identity$f64 in IR");
    }

    #[test]
    fn test_no_code_for_generic_template() {
        use vais_types::TypeChecker;

        let source = r#"
            F identity<T>(x:T)->T=x
        "#;
        let module = parse(source).unwrap();

        // Type check (no instantiations since function isn't called)
        let mut checker = TypeChecker::new();
        checker.check_module(&module).unwrap();
        let instantiations = checker.get_generic_instantiations();

        // No instantiations
        assert!(instantiations.is_empty());

        // Generate code with empty instantiations
        let mut gen = CodeGenerator::new("test");
        let ir = gen
            .generate_module_with_instantiations(&module, instantiations)
            .unwrap();

        // Should NOT contain any identity function definition
        assert!(
            !ir.contains("define i64 @identity"),
            "Generic template should not generate code"
        );
        assert!(
            !ir.contains("define double @identity"),
            "Generic template should not generate code"
        );
    }

    // ==================== Advanced Edge Case Tests ====================

    #[test]
    fn test_i8_boundary_values() {
        // Test i8 min (-128) and max (127)
        let source = r#"
            F i8_bounds()->(i8,i8){
                min:i8=-128;
                max:i8=127;
                (min,max)
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Check for i8 type usage
        assert!(ir.contains("i8"));
    }

    #[test]
    fn test_i8_overflow_value() {
        // Test arithmetic that could overflow (using i64 as i8 not fully supported)
        let source = r#"
            F add_large()->i64{
                x:=9000000000000000000;
                y:=1000000000000000000;
                x+y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should generate code (overflow behavior is runtime)
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_i8_underflow_value() {
        // Test arithmetic that could underflow (using i64)
        let source = r#"
            F sub_large()->i64{
                x:=-9000000000000000000;
                y:=1000000000000000000;
                x-y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("sub i64"));
    }

    #[test]
    fn test_i64_max_value_codegen() {
        // Test i64 max: 9223372036854775807
        let source = r#"
            F i64_max()->i64=9223372036854775807
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("9223372036854775807"));
    }

    #[test]
    fn test_i64_min_value_codegen() {
        // Test i64 min (approximately): -9223372036854775808
        let source = r#"
            F i64_near_min()->i64=-9223372036854775807
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("sub i64 0, 9223372036854775807"));
    }

    #[test]
    fn test_integer_overflow_addition() {
        // Test potential overflow in addition
        let source = r#"
            F add_overflow(a:i64,b:i64)->i64=a+b
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should generate regular add (overflow is runtime behavior)
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_integer_overflow_multiplication() {
        // Test potential overflow in multiplication
        let source = r#"
            F mul_large()->i64{
                a:i64=1000000000;
                b:i64=1000000000;
                a*b
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("mul i64"));
    }

    #[test]
    fn test_division_by_zero() {
        // Test division by zero (runtime error, should compile)
        let source = r#"
            F div_zero()->i64{
                x:=10;
                y:=0;
                x/y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("sdiv i64"));
    }

    #[test]
    fn test_modulo_by_zero() {
        // Test modulo by zero
        let source = r#"
            F mod_zero()->i64{
                x:=10;
                y:=0;
                x%y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("srem i64"));
    }

    #[test]
    fn test_all_integer_type_boundaries() {
        // Test boundary values for all integer types
        // Note: Variables must be used for type information to appear in IR
        // Vais primarily uses i64 for integer arithmetic, but stores typed values
        // Test that integer literals with annotations generate valid IR
        let source = r#"
            F get_i8()->i8{
                a:i8=127;
                a
            }
            F get_i32()->i32{
                e:i32=2147483647;
                e
            }
            F get_i64()->i64{
                f:i64=9223372036854775807;
                f
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Check that the IR contains function definitions with correct return types
        assert!(ir.contains("i8"), "IR should contain i8 type");
        assert!(ir.contains("i32"), "IR should contain i32 type");
        assert!(ir.contains("i64"), "IR should contain i64 type");
    }

    #[test]
    fn test_signed_integer_wraparound() {
        // Test signed integer wraparound behavior (using i64)
        let source = r#"
            F wraparound()->i64{
                max:=9223372036854775806;
                max+1
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_nested_generic_codegen() {
        // Simplified generic struct test
        let source = r#"
            S Container<T>{data:T}
            F empty()->Container<i64> =Container{data:0}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("%Container"));
    }

    #[test]
    fn test_pattern_match_with_guard_codegen() {
        // Test pattern match with guard generates correct branches (fix escaping)
        let source = r#"
            F classify(x:i64)->str=M x{
                n I n>0=>"pos",
                n I n<0=>"neg",
                _=>"zero"
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have branches for guards
        assert!(ir.contains("br i1"));
    }

    #[test]
    fn test_mutual_recursion_codegen() {
        // Test mutual recursion generates correct calls
        let source = r#"
            F is_even(n:i64)->bool=n==0?true:is_odd(n-1)
            F is_odd(n:i64)->bool=n==0?false:is_even(n-1)
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i1 @is_even"));
        assert!(ir.contains("define i1 @is_odd"));
        assert!(ir.contains("call i1 @is_odd"));
        assert!(ir.contains("call i1 @is_even"));
    }

    #[test]
    fn test_deeply_nested_if_codegen() {
        // Test deeply nested if-else generates correct basic blocks
        let source = r#"
            F deep(x:i64)->i64{
                I x>100{
                    I x>1000{1}E{2}
                }E{
                    I x>10{
                        I x>50{3}E{4}
                    }E{5}
                }
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have multiple branches
        let br_count = ir.matches("br i1").count();
        assert!(br_count >= 4, "Expected at least 4 branches");
    }

    #[test]
    fn test_large_number_of_parameters() {
        // Test function with many parameters
        let source = r#"
            F many_params(
                a:i64,b:i64,c:i64,d:i64,e:i64,
                f:i64,g:i64,h:i64,i:i64,j:i64,
                k:i64,l:i64,m:i64,n:i64,o:i64
            )->i64=a+b+c+d+e+f+g+h+i+j+k+l+m+n+o
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @many_params"));
        // Check for parameter usage
        assert!(ir.contains("%a"));
        assert!(ir.contains("%o"));
    }

    #[test]
    fn test_zero_return_optimization() {
        // Test that returning 0 is optimized
        let source = r#"
            F zero()->i64=0
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("ret i64 0"));
    }

    #[test]
    fn test_constant_folding_candidate() {
        // Test expressions that could be constant folded
        let source = r#"
            F const_expr()->i64=2+3*4-1
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should generate arithmetic operations
        assert!(ir.contains("add i64") || ir.contains("ret i64 13"));
        assert!(ir.contains("mul i64") || ir.contains("ret i64 13"));
    }

    #[test]
    fn test_boolean_short_circuit() {
        // Test boolean short-circuit evaluation
        let source = r#"
            F short_circuit(a:bool,b:bool)->bool=a&&b||!a
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i1 @short_circuit"));
    }

    #[test]
    fn test_comparison_chain_codegen() {
        // Test comparison chains: a < b < c
        let source = r#"
            F compare_chain(a:i64,b:i64,c:i64)->bool{
                x:=a<b;
                y:=b<c;
                x&&y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("icmp slt"));
    }

    #[test]
    fn test_bitwise_operations_all_types() {
        // Test bitwise operations (i8 not fully supported, use i64)
        let source = r#"
            F bitwise_i64(a:i64,b:i64)->i64=a&b|a^b
            F bitwise_test()->i64=bitwise_i64(5,3)
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("and i64"));
        assert!(ir.contains("or i64"));
        assert!(ir.contains("xor i64"));
    }

    #[test]
    fn test_shift_operations_boundaries() {
        // Test shift operations at boundaries
        let source = r#"
            F shift_max(x:i64)->i64{
                a:=x<<63;
                b:=x>>63;
                a+b
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("shl i64"));
        assert!(ir.contains("ashr i64"));
    }

    #[test]
    fn test_negative_shift_amount() {
        // Test negative shift (undefined behavior, should compile)
        let source = r#"
            F neg_shift(x:i64)->i64{
                shift:=-1;
                x<<shift
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("shl i64"));
    }

    #[test]
    fn test_all_unary_operators() {
        // Test all unary operators
        let source = r#"
            F unary_ops(x:i64,b:bool)->(i64,i64,bool){
                neg:=-x;
                bit_not:=(~x);
                log_not:=!b;
                (neg,bit_not,log_not)
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("sub i64 0")); // negation
        assert!(ir.contains("xor i64") && ir.contains("-1")); // bitwise not
    }

    #[test]
    fn test_float_division_by_zero() {
        // Test float division (check IR has float division instruction)
        let source = r#"
            F fdiv_test(x:f64,y:f64)->f64=x/y
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Check that float division is generated
        assert!(ir.contains("fdiv") || ir.contains("define double"));
    }

    #[test]
    fn test_recursive_depth() {
        // Test deep recursion (should compile, runtime stack depth)
        let source = r#"
            F deep_recursion(n:i64)->i64=n<1?0:@(n-1)+1
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("call i64 @deep_recursion"));
    }

    // ==================== Decreases Termination Tests ====================

    #[test]
    fn test_decreases_basic() {
        // Test basic decreases clause for termination proof
        let source = r#"
            #[requires(n >= 0)]
            #[decreases(n)]
            F factorial(n:i64)->i64{I n<=1{R 1}R n*factorial(n-1)}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have initial decreases storage
        assert!(
            ir.contains("__decreases_factorial"),
            "Expected decreases storage variable"
        );
        // Should have non-negative check
        assert!(
            ir.contains("decreases_nonneg"),
            "Expected non-negative check"
        );
        // Should have strict decrease check before recursive call
        assert!(
            ir.contains("decreases_check"),
            "Expected decrease check before recursive call"
        );
        // Should have panic call for failed check
        assert!(
            ir.contains("@__panic"),
            "Expected panic call for failed check"
        );
    }

    #[test]
    fn test_decreases_strict_decrease_check() {
        // Test that the strict decrease check (new < old) is generated
        let source = r#"
            #[decreases(n)]
            F count_down(n:i64)->i64{I n<=0{R 0}R count_down(n-1)}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have icmp slt (strictly less than) check
        assert!(
            ir.contains("icmp slt i64"),
            "Expected strict less-than comparison for decreases"
        );
        // Should have both decreases labels
        assert!(ir.contains("decreases_check_ok"), "Expected success label");
        assert!(
            ir.contains("decreases_check_fail"),
            "Expected failure label"
        );
    }

    #[test]
    fn test_decreases_nonneg_check() {
        // Test that non-negative check is generated for decreases expression
        let source = r#"
            #[decreases(x)]
            F process(x:i64)->i64{I x<=0{R 0}R process(x-1)+1}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have icmp sge (signed greater-or-equal) for non-negative check
        assert!(
            ir.contains("icmp sge i64"),
            "Expected non-negative check (sge 0)"
        );
        assert!(
            ir.contains("decreases_nonneg_ok"),
            "Expected success label for non-negative"
        );
        assert!(
            ir.contains("decreases_nonneg_fail"),
            "Expected failure label for non-negative"
        );
    }

    #[test]
    fn test_decreases_release_mode() {
        // Test that decreases checks are skipped in release mode
        let source = r#"
            #[decreases(n)]
            F fib(n:i64)->i64{I n<2{R n}R fib(n-1)+fib(n-2)}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        gen.enable_release_mode();
        let ir = gen.generate_module(&module).unwrap();

        // Should NOT have decreases checks in release mode
        assert!(
            !ir.contains("__decreases_fib"),
            "Should skip decreases in release mode"
        );
        assert!(
            !ir.contains("decreases_nonneg"),
            "Should skip non-negative check in release mode"
        );
        assert!(
            !ir.contains("decreases_check"),
            "Should skip decrease check in release mode"
        );
    }

    #[test]
    fn test_decreases_with_selfcall() {
        // Test decreases with @ self-call operator
        let source = r#"
            #[decreases(n)]
            F sum_to(n:i64)->i64{I n<=0{R 0}R n+@(n-1)}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have decreases check before the self-call
        assert!(
            ir.contains("__decreases_sum_to"),
            "Expected decreases storage"
        );
        assert!(
            ir.contains("decreases_check"),
            "Expected decrease check before self-call"
        );
    }

    #[test]
    fn test_type_recursion_depth_limit() {
        // Test that deeply nested types work within the limit
        use vais_types::ResolvedType;

        let gen = CodeGenerator::new("test");

        // Create a deeply nested pointer type (should work)
        let mut nested_type = ResolvedType::I32;
        for _ in 0..50 {
            nested_type = ResolvedType::Pointer(Box::new(nested_type));
        }

        // This should work fine (well within the 128 limit)
        let llvm_type = gen.type_to_llvm(&nested_type);
        assert!(llvm_type.ends_with('*'), "Should generate nested pointers");

        // Create an extremely deeply nested type (exceeds limit of 128)
        let mut extremely_nested = ResolvedType::I32;
        for _ in 0..150 {
            extremely_nested = ResolvedType::Pointer(Box::new(extremely_nested));
        }

        // This should hit the recursion limit and fall back to i64
        // (The error is logged but doesn't fail - returns fallback type)
        let llvm_type_over_limit = gen.type_to_llvm(&extremely_nested);
        // Should still return a valid type (either i64 fallback or truncated)
        assert!(
            !llvm_type_over_limit.is_empty(),
            "Should return a fallback type on recursion limit"
        );
    }

    #[test]
    fn test_type_recursion_reset_between_calls() {
        // Test that recursion depth is properly reset between calls
        use vais_types::ResolvedType;

        let gen = CodeGenerator::new("test");

        // First call with nested types
        let mut nested1 = ResolvedType::I32;
        for _ in 0..30 {
            nested1 = ResolvedType::Pointer(Box::new(nested1));
        }
        let _ = gen.type_to_llvm(&nested1);

        // Second call should work independently (depth should be reset)
        let mut nested2 = ResolvedType::I64;
        for _ in 0..30 {
            nested2 = ResolvedType::Pointer(Box::new(nested2));
        }
        let llvm_type = gen.type_to_llvm(&nested2);
        assert!(
            llvm_type.ends_with('*'),
            "Second call should work independently"
        );
    }

    #[test]
    fn test_ast_type_recursion_limit() {
        // Test that ast_type_to_resolved also respects recursion limits
        use vais_ast::{Span, Type};

        let gen = CodeGenerator::new("test");

        // Create deeply nested AST type
        let mut nested = Type::Named {
            name: "i32".to_string(),
            generics: vec![],
        };
        for _ in 0..50 {
            nested = Type::Pointer(Box::new(Spanned::new(nested, Span { start: 0, end: 0 })));
        }

        // Should work within limit
        let resolved = gen.ast_type_to_resolved(&nested);
        assert!(
            matches!(resolved, ResolvedType::Pointer(_)),
            "Should resolve nested pointers"
        );

        // Create extremely nested type (exceeds limit)
        let mut extremely_nested = Type::Named {
            name: "i32".to_string(),
            generics: vec![],
        };
        for _ in 0..150 {
            extremely_nested = Type::Pointer(Box::new(Spanned::new(
                extremely_nested,
                Span { start: 0, end: 0 },
            )));
        }

        // Should hit limit and return fallback
        let resolved_over = gen.ast_type_to_resolved(&extremely_nested);
        // Should still return a valid type (Unknown as fallback)
        assert!(
            matches!(
                resolved_over,
                ResolvedType::Unknown | ResolvedType::Pointer(_)
            ),
            "Should return a fallback or truncated type on recursion limit"
        );
    }

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("hello", "hello"), 0);
        assert_eq!(edit_distance("hello", "hallo"), 1);
        assert_eq!(edit_distance("hello", "hell"), 1);
        assert_eq!(edit_distance("hello", "helloo"), 1);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("saturday", "sunday"), 3);
    }

    #[test]
    fn test_suggest_similar() {
        let candidates = vec!["count", "counter", "account", "mount", "county"];

        // Exact case-insensitive match should be prioritized
        let suggestions = suggest_similar("COUNT", &candidates, 3);
        assert_eq!(suggestions[0], "count");

        // Close matches
        let suggestions = suggest_similar("countr", &candidates, 3);
        assert!(suggestions.contains(&"counter".to_string()));
        assert!(suggestions.contains(&"count".to_string()));

        // Should limit to max_suggestions
        let suggestions = suggest_similar("cont", &candidates, 2);
        assert!(suggestions.len() <= 2);

        // No matches if too far
        let suggestions = suggest_similar("xyz", &candidates, 3);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_format_did_you_mean() {
        assert_eq!(format_did_you_mean(&[]), "");
        assert_eq!(
            format_did_you_mean(&["foo".to_string()]),
            ". Did you mean `foo`?"
        );
        assert_eq!(
            format_did_you_mean(&["foo".to_string(), "bar".to_string()]),
            ". Did you mean `foo` or `bar`?"
        );
        assert_eq!(
            format_did_you_mean(&["foo".to_string(), "bar".to_string(), "baz".to_string()]),
            ". Did you mean `foo`, `bar`, or `baz`?"
        );
    }

    #[test]
    fn test_suggest_type_conversion() {
        // Numeric conversions
        assert!(suggest_type_conversion("i64", "f64").contains("as i64"));
        assert!(suggest_type_conversion("f64", "i64").contains("as f64"));
        assert!(suggest_type_conversion("i32", "i64").contains("as i32"));

        // String conversions
        assert!(suggest_type_conversion("String", "&str").contains(".to_string()"));
        assert!(suggest_type_conversion("&str", "String").contains(".as_str()"));

        // Bool to int
        assert!(suggest_type_conversion("i64", "bool").contains("as i64"));

        // No suggestion for unrelated types
        assert_eq!(suggest_type_conversion("Vec", "HashMap"), "");
    }
}
