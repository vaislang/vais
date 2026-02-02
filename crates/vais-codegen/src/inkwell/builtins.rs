//! Built-in function declarations for inkwell code generator.

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;

/// Declares all built-in functions in the module.
pub fn declare_builtins<'ctx>(context: &'ctx Context, module: &Module<'ctx>) {
    let i8_ptr = context.i8_type().ptr_type(AddressSpace::default());
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let f64_type = context.f64_type();
    let void_type = context.void_type();

    // ===== I/O functions =====
    // puts(str) -> i32
    module.add_function("puts", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // printf(fmt, ...) -> i32  (variadic)
    module.add_function("printf", i32_type.fn_type(&[i8_ptr.into()], true), None);
    // getchar() -> i32
    module.add_function("getchar", i32_type.fn_type(&[], false), None);
    // putchar(c: i32) -> i32
    module.add_function("putchar", i32_type.fn_type(&[i32_type.into()], false), None);
    // exit(code: i32) -> void
    module.add_function("exit", void_type.fn_type(&[i32_type.into()], false), None);

    // ===== Memory functions =====
    // malloc(size: i64) -> ptr
    module.add_function("malloc", i8_ptr.fn_type(&[i64_type.into()], false), None);
    // free(ptr) -> void
    module.add_function("free", void_type.fn_type(&[i8_ptr.into()], false), None);
    // memcpy(dest, src, n) -> ptr
    module.add_function("memcpy", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into(), i64_type.into()], false), None);
    // memset(dest, val, n) -> ptr
    module.add_function("memset", i8_ptr.fn_type(&[i8_ptr.into(), i32_type.into(), i64_type.into()], false), None);

    // ===== String functions =====
    // strlen(s) -> i64
    module.add_function("strlen", i64_type.fn_type(&[i8_ptr.into()], false), None);
    // strcmp(s1, s2) -> i32
    module.add_function("strcmp", i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false), None);
    // strncmp(s1, s2, n) -> i32
    module.add_function("strncmp", i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into(), i64_type.into()], false), None);
    // strcpy(dest, src) -> ptr
    module.add_function("strcpy", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false), None);
    // strcat(dest, src) -> ptr
    module.add_function("strcat", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false), None);

    // ===== File I/O functions =====
    // fopen(path, mode) -> ptr (FILE*)
    module.add_function("fopen", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false), None);
    // fclose(stream) -> i32
    module.add_function("fclose", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // fread(ptr, size, count, stream) -> i64
    module.add_function("fread", i64_type.fn_type(&[i8_ptr.into(), i64_type.into(), i64_type.into(), i8_ptr.into()], false), None);
    // fwrite(ptr, size, count, stream) -> i64
    module.add_function("fwrite", i64_type.fn_type(&[i8_ptr.into(), i64_type.into(), i64_type.into(), i8_ptr.into()], false), None);
    // fgetc(stream) -> i32
    module.add_function("fgetc", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // fputc(c, stream) -> i32
    module.add_function("fputc", i32_type.fn_type(&[i32_type.into(), i8_ptr.into()], false), None);
    // fgets(buf, n, stream) -> ptr
    module.add_function("fgets", i8_ptr.fn_type(&[i8_ptr.into(), i32_type.into(), i8_ptr.into()], false), None);
    // fputs(s, stream) -> i32
    module.add_function("fputs", i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false), None);
    // fseek(stream, offset, origin) -> i32
    module.add_function("fseek", i32_type.fn_type(&[i8_ptr.into(), i64_type.into(), i32_type.into()], false), None);
    // ftell(stream) -> i64
    module.add_function("ftell", i64_type.fn_type(&[i8_ptr.into()], false), None);
    // fflush(stream) -> i32
    module.add_function("fflush", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // feof(stream) -> i32
    module.add_function("feof", i32_type.fn_type(&[i8_ptr.into()], false), None);

    // ===== Stdlib functions =====
    // atoi(s) -> i32
    module.add_function("atoi", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // atol(s) -> i64
    module.add_function("atol", i64_type.fn_type(&[i8_ptr.into()], false), None);
    // atof(s) -> f64
    module.add_function("atof", f64_type.fn_type(&[i8_ptr.into()], false), None);
    // rand() -> i32
    module.add_function("rand", i32_type.fn_type(&[], false), None);
    // srand(seed: i32) -> void
    module.add_function("srand", void_type.fn_type(&[i32_type.into()], false), None);
    // labs(x: i64) -> i64
    module.add_function("labs", i64_type.fn_type(&[i64_type.into()], false), None);
    // isdigit(c: i32) -> i32
    module.add_function("isdigit", i32_type.fn_type(&[i32_type.into()], false), None);
    // isalpha(c: i32) -> i32
    module.add_function("isalpha", i32_type.fn_type(&[i32_type.into()], false), None);
    // toupper(c: i32) -> i32
    module.add_function("toupper", i32_type.fn_type(&[i32_type.into()], false), None);
    // tolower(c: i32) -> i32
    module.add_function("tolower", i32_type.fn_type(&[i32_type.into()], false), None);

    // ===== Math functions =====
    declare_math_functions(context, module);

    // ===== Async/scheduling functions =====
    // usleep(usec: i64) -> i32
    module.add_function("usleep", i32_type.fn_type(&[i64_type.into()], false), None);
    // sched_yield() -> i32
    module.add_function("sched_yield", i32_type.fn_type(&[], false), None);
    // close(fd: i32) -> i32
    module.add_function("close", i32_type.fn_type(&[i32_type.into()], false), None);
    // pipe(fds: ptr) -> i32
    module.add_function("pipe", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // kqueue() -> i32
    module.add_function("kqueue", i32_type.fn_type(&[], false), None);

    // ===== GC functions =====
    module.add_function("vais_gc_init", i64_type.fn_type(&[], false), None);
    module.add_function("vais_gc_alloc", i64_type.fn_type(&[i64_type.into(), i32_type.into()], false), None);
    module.add_function("vais_gc_add_root", i64_type.fn_type(&[i64_type.into()], false), None);
    module.add_function("vais_gc_remove_root", i64_type.fn_type(&[i64_type.into()], false), None);
    module.add_function("vais_gc_collect", i64_type.fn_type(&[], false), None);
    module.add_function("vais_gc_bytes_allocated", i64_type.fn_type(&[], false), None);
    module.add_function("vais_gc_objects_count", i64_type.fn_type(&[], false), None);
    module.add_function("vais_gc_collections", i64_type.fn_type(&[], false), None);
    module.add_function("vais_gc_set_threshold", i64_type.fn_type(&[i64_type.into()], false), None);
    module.add_function("vais_gc_print_stats", i64_type.fn_type(&[], false), None);

    // ===== Thread/Sync functions =====
    module.add_function("__cpu_count", i64_type.fn_type(&[], false), None);
    module.add_function("__mutex_create", i64_type.fn_type(&[], false), None);
    module.add_function("__mutex_lock", i64_type.fn_type(&[i64_type.into()], false), None);
    module.add_function("__mutex_unlock", i64_type.fn_type(&[i64_type.into()], false), None);
    module.add_function("__mutex_destroy", void_type.fn_type(&[i64_type.into()], false), None);
    module.add_function("__thread_create", i64_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false), None);
    module.add_function("__thread_join", i64_type.fn_type(&[i64_type.into()], false), None);

    // ===== Abort for panic =====
    module.add_function("abort", void_type.fn_type(&[], false), None);

    // ===== Internal helper functions =====
    // These get generated as inline LLVM IR, not declared as extern
    // but the text backend generates them inline. For inkwell, we declare them
    // so they can be resolved; actual implementation is done via helper generation.
}

fn declare_math_functions<'ctx>(context: &'ctx Context, module: &Module<'ctx>) {
    let f64_type = context.f64_type();

    // f64 -> f64 unary functions
    let unary_f64 = f64_type.fn_type(&[f64_type.into()], false);
    for name in &["sqrt", "sin", "cos", "tan", "asin", "acos", "atan",
                   "exp", "log", "log10", "log2", "floor", "ceil", "round", "fabs"] {
        module.add_function(name, unary_f64, None);
    }

    // f64 x f64 -> f64 binary functions
    let binary_f64 = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
    for name in &["pow", "atan2", "fmod"] {
        module.add_function(name, binary_f64, None);
    }
}
