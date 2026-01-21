//! Built-in function declarations for inkwell code generator.

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;

/// Declares all built-in functions in the module.
pub fn declare_builtins<'ctx>(context: &'ctx Context, module: &Module<'ctx>) {
    // I/O functions
    declare_puts(context, module);
    declare_printf(context, module);
    declare_getchar(context, module);

    // Memory functions
    declare_malloc(context, module);
    declare_free(context, module);
    declare_memcpy(context, module);
    declare_memset(context, module);
    declare_strlen(context, module);

    // Math functions
    declare_math_functions(context, module);

    // Abort for panic
    declare_abort(context, module);
}

fn declare_puts<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let ptr_type = context.ptr_type(AddressSpace::default());
    let fn_type = i32_type.fn_type(&[ptr_type.into()], false);
    module.add_function("puts", fn_type, None)
}

fn declare_printf<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let ptr_type = context.ptr_type(AddressSpace::default());
    let fn_type = i32_type.fn_type(&[ptr_type.into()], true); // variadic
    module.add_function("printf", fn_type, None)
}

fn declare_getchar<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);
    module.add_function("getchar", fn_type, None)
}

fn declare_malloc<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let ptr_type = context.ptr_type(AddressSpace::default());
    let i64_type = context.i64_type();
    let fn_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("malloc", fn_type, None)
}

fn declare_free<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let void_type = context.void_type();
    let ptr_type = context.ptr_type(AddressSpace::default());
    let fn_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("free", fn_type, None)
}

fn declare_memcpy<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let ptr_type = context.ptr_type(AddressSpace::default());
    let i64_type = context.i64_type();
    let fn_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into()], false);
    module.add_function("memcpy", fn_type, None)
}

fn declare_memset<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let ptr_type = context.ptr_type(AddressSpace::default());
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let fn_type = ptr_type.fn_type(&[ptr_type.into(), i32_type.into(), i64_type.into()], false);
    module.add_function("memset", fn_type, None)
}

fn declare_strlen<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let i64_type = context.i64_type();
    let ptr_type = context.ptr_type(AddressSpace::default());
    let fn_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("strlen", fn_type, None)
}

fn declare_math_functions<'ctx>(context: &'ctx Context, module: &Module<'ctx>) {
    let f64_type = context.f64_type();

    // sqrt(f64) -> f64
    let fn_type = f64_type.fn_type(&[f64_type.into()], false);
    module.add_function("sqrt", fn_type, None);
    module.add_function("sin", fn_type, None);
    module.add_function("cos", fn_type, None);
    module.add_function("tan", fn_type, None);
    module.add_function("asin", fn_type, None);
    module.add_function("acos", fn_type, None);
    module.add_function("atan", fn_type, None);
    module.add_function("exp", fn_type, None);
    module.add_function("log", fn_type, None);
    module.add_function("log10", fn_type, None);
    module.add_function("log2", fn_type, None);
    module.add_function("floor", fn_type, None);
    module.add_function("ceil", fn_type, None);
    module.add_function("round", fn_type, None);
    module.add_function("fabs", fn_type, None);

    // pow(f64, f64) -> f64
    let pow_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
    module.add_function("pow", pow_type, None);

    // atan2(f64, f64) -> f64
    module.add_function("atan2", pow_type, None);

    // fmod(f64, f64) -> f64
    module.add_function("fmod", pow_type, None);
}

fn declare_abort<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    module.add_function("abort", fn_type, None)
}
