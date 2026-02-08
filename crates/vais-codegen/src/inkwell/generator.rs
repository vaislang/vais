//! Inkwell-based LLVM code generator.
//!
//! Provides type-safe LLVM IR generation using the inkwell crate.

use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::values::{BasicValueEnum, FunctionValue, GlobalValue, PointerValue};

use vais_ast::{self as ast, Expr, Module as VaisModule};
use vais_types::ResolvedType;

use super::builtins;
use super::types::TypeMapper;
use crate::{CodegenResult, TargetTriple};

/// Loop context for break/continue handling.
pub(super) struct LoopContext<'ctx> {
    /// Block to jump to on break
    pub(super) break_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// Block to jump to on continue
    pub(super) continue_block: inkwell::basic_block::BasicBlock<'ctx>,
}

/// Closure information for captured variables
#[derive(Clone)]
#[allow(dead_code)]
pub(super) struct ClosureInfo<'ctx> {
    /// The generated LLVM function
    pub(super) func: FunctionValue<'ctx>,
    /// Captured variable names and their values (for passing to the lambda)
    pub(super) captures: Vec<(String, BasicValueEnum<'ctx>)>,
}

/// LLVM code generator using inkwell.
///
/// This generator provides direct LLVM API access for type-safe
/// code generation, as opposed to text-based IR generation.
pub struct InkwellCodeGenerator<'ctx> {
    /// LLVM context - owns all LLVM objects
    pub(super) context: &'ctx Context,

    /// Current LLVM module being built
    pub(super) module: Module<'ctx>,

    /// IR builder for instruction generation
    pub(super) builder: Builder<'ctx>,

    /// Type mapper for Vais -> LLVM type conversion
    pub(super) type_mapper: TypeMapper<'ctx>,

    /// Target architecture
    #[allow(dead_code)]
    pub(super) target: TargetTriple,

    /// Registered functions by name
    pub(super) functions: HashMap<String, FunctionValue<'ctx>>,

    /// Local variables (alloca pointers and their types)
    pub(super) locals: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,

    /// String constants
    pub(super) string_constants: HashMap<String, GlobalValue<'ctx>>,
    pub(super) string_counter: usize,

    /// Loop stack for break/continue
    pub(super) loop_stack: Vec<LoopContext<'ctx>>,

    /// Label counter for unique block names
    pub(super) label_counter: usize,

    /// Current function being compiled
    pub(super) current_function: Option<FunctionValue<'ctx>>,

    /// Generic type substitutions
    pub(super) generic_substitutions: HashMap<String, ResolvedType>,

    /// Generated struct types (for deduplication)
    pub(super) generated_structs: HashMap<String, StructType<'ctx>>,

    /// Struct field names (struct name -> field names in order)
    pub(super) struct_fields: HashMap<String, Vec<String>>,

    /// Struct field type names (struct name -> [(field_name, type_name)])
    /// Used for nested field access to resolve intermediate struct types
    pub(super) struct_field_type_names: HashMap<String, Vec<(String, String)>>,

    /// Lambda function counter for unique naming
    pub(super) lambda_counter: usize,

    /// Lambda functions generated during expression compilation
    pub(super) lambda_functions: Vec<FunctionValue<'ctx>>,

    /// Enum variant tags: maps (enum_name, variant_name) -> tag
    pub(super) enum_variants: HashMap<(String, String), i32>,

    /// Variable name -> struct type name tracking (for method call resolution)
    pub(super) var_struct_types: HashMap<String, String>,

    /// Struct name -> generic parameter names (for method generic substitution)
    pub(super) struct_generic_params: HashMap<String, Vec<String>>,

    /// Lambda binding info: variable name -> (lambda function name, captured values)
    pub(super) lambda_bindings: HashMap<String, (String, Vec<(String, BasicValueEnum<'ctx>)>)>,

    /// Temporary storage for the last generated lambda (used by Stmt::Let to track bindings)
    pub(super) _last_lambda_info: Option<(String, Vec<(String, BasicValueEnum<'ctx>)>)>,

    /// Constants: name -> value (evaluated at compile time)
    pub(super) constants: HashMap<String, BasicValueEnum<'ctx>>,

    /// Function name -> return struct type name (for struct type inference)
    pub(super) function_return_structs: HashMap<String, String>,

    /// Defer stack: expressions to execute in LIFO order before function return
    pub(super) defer_stack: Vec<Expr>,

    /// TCO state: when generating a tail-recursive function as a loop,
    /// this holds the parameter allocas and the loop header block for jumping back.
    pub(super) tco_state: Option<TcoState<'ctx>>,

    /// Resolved function signatures from type checker (for return/param type inference)
    pub(super) resolved_function_sigs: HashMap<String, vais_types::FunctionSig>,
}

/// Tail Call Optimization state for loop-based tail recursion elimination.
pub(super) struct TcoState<'ctx> {
    /// Parameter allocas (name -> alloca pointer) for updating params before looping back
    pub(super) param_allocas: Vec<(String, PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,
    /// The loop header block to branch back to
    pub(super) loop_header: inkwell::basic_block::BasicBlock<'ctx>,
}

impl<'ctx> InkwellCodeGenerator<'ctx> {
    /// Creates a new inkwell code generator.
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        Self::new_with_target(context, module_name, TargetTriple::Native)
    }

    /// Creates a new inkwell code generator with specified target.
    pub fn new_with_target(
        context: &'ctx Context,
        module_name: &str,
        target: TargetTriple,
    ) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let type_mapper = TypeMapper::new(context);

        // Set target triple if not native
        if target != TargetTriple::Native {
            module.set_triple(&inkwell::targets::TargetTriple::create(target.triple_str()));
        }

        let gen = Self {
            context,
            module,
            builder,
            type_mapper,
            target,
            functions: HashMap::new(),
            locals: HashMap::new(),
            string_constants: HashMap::new(),
            string_counter: 0,
            loop_stack: Vec::new(),
            label_counter: 0,
            current_function: None,
            generic_substitutions: HashMap::new(),
            generated_structs: HashMap::new(),
            struct_fields: HashMap::new(),
            struct_field_type_names: HashMap::new(),
            lambda_counter: 0,
            lambda_functions: Vec::new(),
            enum_variants: HashMap::new(),
            var_struct_types: HashMap::new(),
            struct_generic_params: HashMap::new(),
            lambda_bindings: HashMap::new(),
            _last_lambda_info: None,
            constants: HashMap::new(),
            function_return_structs: HashMap::new(),
            defer_stack: Vec::new(),
            tco_state: None,
            resolved_function_sigs: HashMap::new(),
        };

        // Declare built-in functions
        builtins::declare_builtins(context, &gen.module);

        gen
    }

    /// Generates code for an entire Vais module.
    pub fn generate_module(&mut self, vais_module: &VaisModule) -> CodegenResult<()> {
        // First pass: collect all function signatures, struct definitions, enum definitions, and extern blocks
        for item in &vais_module.items {
            match &item.node {
                ast::Item::Function(func) => {
                    self.declare_function(func)?;
                }
                ast::Item::Struct(s) => {
                    self.define_struct(s)?;
                }
                ast::Item::Enum(e) => {
                    self.define_enum(e)?;
                }
                ast::Item::ExternBlock(extern_block) => {
                    self.declare_extern_block(extern_block)?;
                }
                ast::Item::Union(u) => {
                    self.define_union(u)?;
                }
                ast::Item::Const(const_def) => {
                    self.define_const(const_def)?;
                }
                _ => {}
            }
        }

        // Second pass: declare methods from Impl blocks and struct inline methods
        for item in &vais_module.items {
            match &item.node {
                ast::Item::Impl(impl_block) => {
                    if let Some(type_name) = Self::get_impl_type_name(&impl_block.target_type.node)
                    {
                        for method in &impl_block.methods {
                            self.declare_method(&type_name, &method.node)?;
                        }
                    }
                }
                ast::Item::Struct(s) => {
                    let type_name = s.name.node.clone();
                    for method in &s.methods {
                        self.declare_method(&type_name, &method.node)?;
                    }
                }
                _ => {}
            }
        }

        // Third pass: generate function bodies and method bodies
        for item in &vais_module.items {
            match &item.node {
                ast::Item::Function(func) => {
                    self.generate_function(func)?;
                }
                ast::Item::Impl(impl_block) => {
                    if let Some(type_name) = Self::get_impl_type_name(&impl_block.target_type.node)
                    {
                        for method in &impl_block.methods {
                            self.generate_method(&type_name, &method.node)?;
                        }
                    }
                }
                ast::Item::Struct(s) => {
                    let type_name = s.name.node.clone();
                    for method in &s.methods {
                        self.generate_method(&type_name, &method.node)?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Returns the generated LLVM module.
    pub fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }

    /// Set resolved function signatures from the type checker.
    pub fn set_resolved_functions(&mut self, resolved: HashMap<String, vais_types::FunctionSig>) {
        self.resolved_function_sigs = resolved;
    }

    /// Returns the LLVM IR as a string.
    pub fn get_ir_string(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Writes the LLVM IR to a file.
    pub fn write_to_file(&self, path: &std::path::Path) -> Result<(), String> {
        self.module.print_to_file(path).map_err(|e| e.to_string())
    }

    // ========== Declaration Phase ==========
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_ast::{Literal, Type};

    #[test]
    fn test_create_generator() {
        let context = Context::create();
        let gen = InkwellCodeGenerator::new(&context, "test_module");
        assert!(!gen.get_ir_string().is_empty());
    }

    #[test]
    fn test_generate_literal_int() {
        let context = Context::create();
        let mut gen = InkwellCodeGenerator::new(&context, "test");

        let lit = Literal::Int(42);
        let result = gen.generate_literal(&lit).unwrap();

        assert!(result.is_int_value());
        let int_val = result.into_int_value();
        assert_eq!(int_val.get_zero_extended_constant(), Some(42));
    }

    #[test]
    fn test_generate_literal_float() {
        let context = Context::create();
        let mut gen = InkwellCodeGenerator::new(&context, "test");

        let lit = Literal::Float(3.14);
        let result = gen.generate_literal(&lit).unwrap();

        assert!(result.is_float_value());
    }

    #[test]
    fn test_generate_literal_bool() {
        let context = Context::create();
        let mut gen = InkwellCodeGenerator::new(&context, "test");

        let lit = Literal::Bool(true);
        let result = gen.generate_literal(&lit).unwrap();

        assert!(result.is_int_value());
        let int_val = result.into_int_value();
        assert_eq!(int_val.get_zero_extended_constant(), Some(1));
    }

    #[test]
    fn test_generate_string_literal() {
        let context = Context::create();
        let mut gen = InkwellCodeGenerator::new(&context, "test");

        // Need a function context for builder position
        let fn_type = context.void_type().fn_type(&[], false);
        let func = gen.module.add_function("__test_str", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        gen.builder.position_at_end(entry);

        let result = gen.generate_string_literal("hello").unwrap();
        assert!(result.is_pointer_value());
    }

    #[test]
    fn test_ast_type_to_resolved() {
        let context = Context::create();
        let gen = InkwellCodeGenerator::new(&context, "test");

        // Test basic types
        let i64_type = Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        };
        let resolved = gen.ast_type_to_resolved(&i64_type);
        assert!(matches!(resolved, ResolvedType::I64));

        let bool_type = Type::Named {
            name: "bool".to_string(),
            generics: vec![],
        };
        let resolved = gen.ast_type_to_resolved(&bool_type);
        assert!(matches!(resolved, ResolvedType::Bool));

        let unit_type = Type::Unit;
        let resolved = gen.ast_type_to_resolved(&unit_type);
        assert!(matches!(resolved, ResolvedType::Unit));
    }

    #[test]
    fn test_lambda_counter() {
        let context = Context::create();
        let gen = InkwellCodeGenerator::new(&context, "test");

        // Lambda counter should start at 0
        assert_eq!(gen.lambda_counter, 0);
    }

    #[test]
    fn test_generic_substitutions() {
        let context = Context::create();
        let mut gen = InkwellCodeGenerator::new(&context, "test");

        // Initially empty
        assert!(gen.get_generic_substitution("T").is_none());

        // Set substitutions
        let mut subst = HashMap::new();
        subst.insert("T".to_string(), ResolvedType::I64);
        subst.insert("U".to_string(), ResolvedType::Bool);
        gen.set_generic_substitutions(subst);

        // Check substitutions
        assert!(matches!(
            gen.get_generic_substitution("T"),
            Some(ResolvedType::I64)
        ));
        assert!(matches!(
            gen.get_generic_substitution("U"),
            Some(ResolvedType::Bool)
        ));
        assert!(gen.get_generic_substitution("V").is_none());

        // Clear substitutions
        gen.clear_generic_substitutions();
        assert!(gen.get_generic_substitution("T").is_none());
    }

    #[test]
    fn test_mangle_names() {
        let context = Context::create();
        let gen = InkwellCodeGenerator::new(&context, "test");

        // Empty type args
        let name = gen.mangle_struct_name("Vec", &[]);
        assert_eq!(name, "Vec");

        // With type args
        let name = gen.mangle_struct_name("Vec", &[ResolvedType::I64]);
        assert_eq!(name, "Vec$i64");

        // Multiple type args
        let name = gen.mangle_struct_name("HashMap", &[ResolvedType::Str, ResolvedType::I64]);
        assert_eq!(name, "HashMap$str_i64");
    }

    #[test]
    fn test_substitute_type() {
        let context = Context::create();
        let mut gen = InkwellCodeGenerator::new(&context, "test");

        // Set substitutions
        let mut subst = HashMap::new();
        subst.insert("T".to_string(), ResolvedType::I64);
        gen.set_generic_substitutions(subst);

        // Substitute a generic type
        let generic_type = ResolvedType::Generic("T".to_string());
        let substituted = gen.substitute_type(&generic_type);
        assert!(matches!(substituted, ResolvedType::I64));

        // Non-generic type stays the same
        let concrete_type = ResolvedType::Bool;
        let substituted = gen.substitute_type(&concrete_type);
        assert!(matches!(substituted, ResolvedType::Bool));
    }
}
