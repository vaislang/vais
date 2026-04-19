//! Built-in function declarations for inkwell code generator.

mod simd;

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;

/// Declares all built-in functions in the module.
pub fn declare_builtins<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> Result<(), String> {
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
    module.add_function(
        "memcpy",
        i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into(), i64_type.into()], false),
        None,
    );
    // memcpy_str: wrapper that calls memcpy (used by selfhost codegen.vais)
    {
        let memcpy_fn = module
            .get_function("memcpy")
            .ok_or("ICE: builtin memcpy must be declared before use")?;
        let fn_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into(), i64_type.into()], false);
        let func = module.add_function("memcpy_str", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let result = builder
            .build_call(
                memcpy_fn,
                &[
                    func.get_nth_param(0)
                        .ok_or("ICE: builtin function missing parameter")?
                        .into(),
                    func.get_nth_param(1)
                        .ok_or("ICE: builtin function missing parameter")?
                        .into(),
                    func.get_nth_param(2)
                        .ok_or("ICE: builtin function missing parameter")?
                        .into(),
                ],
                "result",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            .try_as_basic_value()
            .left()
            .ok_or("ICE: inkwell builtins: call returned void")?;
        builder
            .build_return(Some(&result))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }
    // realloc(ptr, size) -> ptr
    module.add_function(
        "realloc",
        i8_ptr.fn_type(&[i8_ptr.into(), i64_type.into()], false),
        None,
    );
    // memset(dest, val, n) -> ptr
    module.add_function(
        "memset",
        i8_ptr.fn_type(&[i8_ptr.into(), i32_type.into(), i64_type.into()], false),
        None,
    );
    // memcmp(s1, s2, n) -> i32
    module.add_function(
        "memcmp",
        i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into(), i64_type.into()], false),
        None,
    );

    // ===== String functions =====
    // strlen(s) -> i64
    module.add_function("strlen", i64_type.fn_type(&[i8_ptr.into()], false), None);
    // strcmp(s1, s2) -> i32
    module.add_function(
        "strcmp",
        i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false),
        None,
    );
    // strncmp(s1, s2, n) -> i32
    module.add_function(
        "strncmp",
        i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into(), i64_type.into()], false),
        None,
    );
    // strcpy(dest, src) -> ptr
    module.add_function(
        "strcpy",
        i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false),
        None,
    );
    // strcat(dest, src) -> ptr
    module.add_function(
        "strcat",
        i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false),
        None,
    );

    // ===== File I/O functions =====
    // fopen(path, mode) -> ptr (FILE*)
    module.add_function(
        "fopen",
        i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false),
        None,
    );
    // fopen_ptr: wrapper that calls fopen (for selfhost, accepts i64 path as ptr)
    {
        let fopen_fn = module
            .get_function("fopen")
            .ok_or("ICE: builtin fopen must be declared before use")?;
        let fn_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let func = module.add_function("fopen_ptr", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let result = builder
            .build_call(
                fopen_fn,
                &[
                    func.get_nth_param(0)
                        .ok_or("ICE: builtin function missing parameter")?
                        .into(),
                    func.get_nth_param(1)
                        .ok_or("ICE: builtin function missing parameter")?
                        .into(),
                ],
                "result",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            .try_as_basic_value()
            .left()
            .ok_or("ICE: inkwell builtins: call returned void")?;
        builder
            .build_return(Some(&result))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }
    // fclose(stream) -> i32
    module.add_function("fclose", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // fread(ptr, size, count, stream) -> i64
    module.add_function(
        "fread",
        i64_type.fn_type(
            &[
                i8_ptr.into(),
                i64_type.into(),
                i64_type.into(),
                i8_ptr.into(),
            ],
            false,
        ),
        None,
    );
    // fwrite(ptr, size, count, stream) -> i64
    module.add_function(
        "fwrite",
        i64_type.fn_type(
            &[
                i8_ptr.into(),
                i64_type.into(),
                i64_type.into(),
                i8_ptr.into(),
            ],
            false,
        ),
        None,
    );
    // fgetc(stream) -> i32
    module.add_function("fgetc", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // fputc(c, stream) -> i32
    module.add_function(
        "fputc",
        i32_type.fn_type(&[i32_type.into(), i8_ptr.into()], false),
        None,
    );
    // fgets(buf, n, stream) -> ptr
    module.add_function(
        "fgets",
        i8_ptr.fn_type(&[i8_ptr.into(), i32_type.into(), i8_ptr.into()], false),
        None,
    );
    // fputs(s, stream) -> i32
    module.add_function(
        "fputs",
        i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false),
        None,
    );
    // fseek(stream, offset, origin) -> i32
    module.add_function(
        "fseek",
        i32_type.fn_type(&[i8_ptr.into(), i64_type.into(), i32_type.into()], false),
        None,
    );
    // ftell(stream) -> i64
    module.add_function("ftell", i64_type.fn_type(&[i8_ptr.into()], false), None);
    // fflush(stream) -> i32
    module.add_function("fflush", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // feof(stream) -> i32
    module.add_function("feof", i32_type.fn_type(&[i8_ptr.into()], false), None);

    // get_stdin() -> FILE* (returns stdin stream)
    {
        let fn_type = i8_ptr.fn_type(&[], false);
        let func = module.add_function("get_stdin", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        // On macOS: __stdinp, on Linux: stdin
        let stdin_sym = if cfg!(target_os = "macos") {
            "__stdinp"
        } else {
            "stdin"
        };
        let stdin_global = module.add_global(i8_ptr, Some(AddressSpace::default()), stdin_sym);
        stdin_global.set_externally_initialized(true);
        let val = builder
            .build_load(i8_ptr, stdin_global.as_pointer_value(), "stdin_val")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        builder
            .build_return(Some(&val))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }
    // get_stdout() -> FILE* (returns stdout stream)
    {
        let fn_type = i8_ptr.fn_type(&[], false);
        let func = module.add_function("get_stdout", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let stdout_sym = if cfg!(target_os = "macos") {
            "__stdoutp"
        } else {
            "stdout"
        };
        let stdout_global = module.add_global(i8_ptr, Some(AddressSpace::default()), stdout_sym);
        stdout_global.set_externally_initialized(true);
        let val = builder
            .build_load(i8_ptr, stdout_global.as_pointer_value(), "stdout_val")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        builder
            .build_return(Some(&val))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }
    // fileno(stream) -> i32 (get fd from FILE*)
    module.add_function("fileno", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // fsync(fd) -> i32
    module.add_function("fsync", i32_type.fn_type(&[i32_type.into()], false), None);
    // fdatasync(fd) -> i32 (data-only sync)
    module.add_function(
        "fdatasync",
        i32_type.fn_type(&[i32_type.into()], false),
        None,
    );
    // open(path, flags, mode) -> fd
    module.add_function(
        "open",
        i32_type.fn_type(&[i8_ptr.into(), i32_type.into(), i32_type.into()], false),
        None,
    );
    // close(fd) -> i32
    module.add_function("close", i32_type.fn_type(&[i32_type.into()], false), None);
    // remove(path) -> i32
    module.add_function("remove", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // flock(fd, operation) -> i32 (advisory file locking)
    module.add_function(
        "flock",
        i32_type.fn_type(&[i32_type.into(), i32_type.into()], false),
        None,
    );

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
    // pipe(fds: ptr) -> i32
    module.add_function("pipe", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // kqueue() -> i32
    module.add_function("kqueue", i32_type.fn_type(&[], false), None);

    // ===== GC functions =====
    module.add_function("vais_gc_init", i64_type.fn_type(&[], false), None);
    module.add_function(
        "vais_gc_alloc",
        i64_type.fn_type(&[i64_type.into(), i32_type.into()], false),
        None,
    );
    module.add_function(
        "vais_gc_add_root",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function(
        "vais_gc_remove_root",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function("vais_gc_collect", i64_type.fn_type(&[], false), None);
    module.add_function(
        "vais_gc_bytes_allocated",
        i64_type.fn_type(&[], false),
        None,
    );
    module.add_function("vais_gc_objects_count", i64_type.fn_type(&[], false), None);
    module.add_function("vais_gc_collections", i64_type.fn_type(&[], false), None);
    module.add_function(
        "vais_gc_set_threshold",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function("vais_gc_print_stats", i64_type.fn_type(&[], false), None);

    // ===== Thread/Sync functions =====
    module.add_function("__cpu_count", i64_type.fn_type(&[], false), None);
    module.add_function("__mutex_create", i64_type.fn_type(&[], false), None);
    module.add_function(
        "__mutex_lock",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function(
        "__mutex_unlock",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function(
        "__mutex_destroy",
        void_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function(
        "__thread_create",
        i64_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false),
        None,
    );
    module.add_function(
        "__thread_join",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );

    // ===== Abort for panic =====
    module.add_function("abort", void_type.fn_type(&[], false), None);

    // ===== Additional Thread/Sync functions =====
    module.add_function(
        "__mutex_try_lock",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function(
        "__thread_sleep_ms",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function("__rwlock_create", i64_type.fn_type(&[], false), None);
    module.add_function(
        "__rwlock_read_lock",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function(
        "__rwlock_read_unlock",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function(
        "__rwlock_write_lock",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function(
        "__rwlock_write_unlock",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function(
        "__rwlock_destroy",
        void_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function("__cond_create", i64_type.fn_type(&[], false), None);
    module.add_function(
        "__cond_wait",
        i64_type.fn_type(&[i64_type.into(), i64_type.into()], false),
        None,
    );
    module.add_function(
        "__cond_signal",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function(
        "__cond_broadcast",
        i64_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function(
        "__cond_destroy",
        void_type.fn_type(&[i64_type.into()], false),
        None,
    );
    module.add_function(
        "__atomic_load",
        i64_type.fn_type(&[i8_ptr.into()], false),
        None,
    );
    module.add_function(
        "__atomic_store",
        void_type.fn_type(&[i8_ptr.into(), i64_type.into()], false),
        None,
    );
    module.add_function(
        "__atomic_add",
        i64_type.fn_type(&[i8_ptr.into(), i64_type.into()], false),
        None,
    );
    module.add_function(
        "__atomic_cas",
        i64_type.fn_type(&[i8_ptr.into(), i64_type.into(), i64_type.into()], false),
        None,
    );

    // ===== GC gen functions =====
    module.add_function("vais_gen_gc_init", void_type.fn_type(&[], false), None);

    // ===== SIMD functions (defined inline) =====
    simd::define_simd_builtins(context, module)?;

    // ===== String helper functions =====
    // snprintf(buf, size, fmt, ...) -> i32
    module.add_function(
        "snprintf",
        i32_type.fn_type(&[i8_ptr.into(), i64_type.into(), i8_ptr.into()], true),
        None,
    );
    // strtol(str, endptr, base) -> i64
    module.add_function(
        "strtol",
        i64_type.fn_type(&[i8_ptr.into(), i8_ptr.into(), i32_type.into()], false),
        None,
    );
    // strtod(str, endptr) -> f64
    module.add_function(
        "strtod",
        f64_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false),
        None,
    );

    // ===== Time functions =====
    module.add_function("time", i64_type.fn_type(&[i8_ptr.into()], false), None);
    module.add_function("clock", i64_type.fn_type(&[], false), None);

    // ===== System functions (env/process/signal) =====
    // getenv(name) -> ptr (NULL if not found)
    module.add_function("getenv", i8_ptr.fn_type(&[i8_ptr.into()], false), None);
    // setenv(name, value, overwrite) -> i32
    module.add_function(
        "setenv",
        i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into(), i32_type.into()], false),
        None,
    );
    // unsetenv(name) -> i32
    module.add_function("unsetenv", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // system(command) -> i32
    module.add_function("system", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // popen(command, mode) -> FILE*
    module.add_function(
        "popen",
        i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false),
        None,
    );
    // pclose(stream) -> i32
    module.add_function("pclose", i32_type.fn_type(&[i8_ptr.into()], false), None);
    // signal(signum, handler) -> ptr
    module.add_function(
        "signal",
        i8_ptr.fn_type(&[i32_type.into(), i8_ptr.into()], false),
        None,
    );
    // raise(signum) -> i32
    module.add_function("raise", i32_type.fn_type(&[i32_type.into()], false), None);

    // ===== Network functions =====
    module.add_function(
        "socket",
        i32_type.fn_type(&[i32_type.into(), i32_type.into(), i32_type.into()], false),
        None,
    );
    module.add_function(
        "bind",
        i32_type.fn_type(&[i32_type.into(), i8_ptr.into(), i32_type.into()], false),
        None,
    );
    module.add_function(
        "listen",
        i32_type.fn_type(&[i32_type.into(), i32_type.into()], false),
        None,
    );
    module.add_function(
        "accept",
        i32_type.fn_type(&[i32_type.into(), i8_ptr.into(), i8_ptr.into()], false),
        None,
    );
    module.add_function(
        "connect",
        i32_type.fn_type(&[i32_type.into(), i8_ptr.into(), i32_type.into()], false),
        None,
    );
    module.add_function(
        "send",
        i64_type.fn_type(
            &[
                i32_type.into(),
                i8_ptr.into(),
                i64_type.into(),
                i32_type.into(),
            ],
            false,
        ),
        None,
    );
    module.add_function(
        "recv",
        i64_type.fn_type(
            &[
                i32_type.into(),
                i8_ptr.into(),
                i64_type.into(),
                i32_type.into(),
            ],
            false,
        ),
        None,
    );
    module.add_function(
        "read",
        i64_type.fn_type(&[i32_type.into(), i8_ptr.into(), i64_type.into()], false),
        None,
    );
    module.add_function(
        "write",
        i64_type.fn_type(&[i32_type.into(), i8_ptr.into(), i64_type.into()], false),
        None,
    );

    // ===== Regex functions =====
    module.add_function(
        "regcomp",
        i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into(), i32_type.into()], false),
        None,
    );
    module.add_function(
        "regexec",
        i32_type.fn_type(
            &[
                i8_ptr.into(),
                i8_ptr.into(),
                i64_type.into(),
                i8_ptr.into(),
                i32_type.into(),
            ],
            false,
        ),
        None,
    );
    module.add_function("regfree", void_type.fn_type(&[i8_ptr.into()], false), None);

    // ===== Runtime intrinsic helpers (mirror text-mode function_gen/runtime.rs) =====
    declare_runtime_intrinsics(context, module)?;

    Ok(())
}

/// Runtime intrinsic helpers (callable from user code as `time_now_ms`, `load_i8`, etc.).
///
/// The Inkwell codegen path emits inline bodies here; the text-mode
/// function_gen/runtime.rs path emits equivalent bodies as raw IR. Keep both in
/// sync when adding new intrinsics.
fn declare_runtime_intrinsics<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let i8_ptr = context.i8_type().ptr_type(AddressSpace::default());
    let i16_ptr = context.i16_type().ptr_type(AddressSpace::default());
    let i32_ptr = context.i32_type().ptr_type(AddressSpace::default());
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let unit_type = i64_type; // Unit -> i64 (ignored return) for LLVM compat
    let void_type = context.void_type();

    // Helper to emit `declare`-only (no body, external runtime must provide)
    // We instead emit inline bodies for everything listed in RUNTIME_INTRINSIC_NAMES.

    // --- load_i8 / load_i16 / load_i32 (all return i64, signed-extend from narrow) ---
    for (name, bits) in [("load_i8", 8), ("load_i16", 16), ("load_i32", 32)] {
        let fn_type = i64_type.fn_type(&[i64_type.into()], false);
        let func = module.add_function(name, fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let ptr_i64 = func
            .get_nth_param(0)
            .ok_or("ICE: runtime intrinsic missing param")?
            .into_int_value();
        let narrow_ptr_ty = match bits {
            8 => i8_ptr,
            16 => i16_ptr,
            32 => i32_ptr,
            _ => unreachable!(),
        };
        let narrow_ty = match bits {
            8 => context.i8_type(),
            16 => context.i16_type(),
            32 => context.i32_type(),
            _ => unreachable!(),
        };
        let ptr = builder
            .build_int_to_ptr(ptr_i64, narrow_ptr_ty, "narrow_ptr")
            .map_err(|e| format!("ICE: {e}"))?;
        let loaded = builder
            .build_load(narrow_ty, ptr, "narrow_val")
            .map_err(|e| format!("ICE: {e}"))?;
        let widened = builder
            .build_int_s_extend(loaded.into_int_value(), i64_type, "wide_val")
            .map_err(|e| format!("ICE: {e}"))?;
        builder
            .build_return(Some(&widened))
            .map_err(|e| format!("ICE: {e}"))?;
    }

    // --- store_i8 / store_i16 / store_i32 (truncate i64 -> narrow, return unit-as-i64) ---
    for (name, bits) in [("store_i8", 8), ("store_i16", 16), ("store_i32", 32)] {
        let fn_type = unit_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let func = module.add_function(name, fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let ptr_i64 = func
            .get_nth_param(0)
            .ok_or("ICE: runtime intrinsic missing param")?
            .into_int_value();
        let val_i64 = func
            .get_nth_param(1)
            .ok_or("ICE: runtime intrinsic missing param")?
            .into_int_value();
        let narrow_ptr_ty = match bits {
            8 => i8_ptr,
            16 => i16_ptr,
            32 => i32_ptr,
            _ => unreachable!(),
        };
        let narrow_ty = match bits {
            8 => context.i8_type(),
            16 => context.i16_type(),
            32 => context.i32_type(),
            _ => unreachable!(),
        };
        let ptr = builder
            .build_int_to_ptr(ptr_i64, narrow_ptr_ty, "narrow_ptr")
            .map_err(|e| format!("ICE: {e}"))?;
        let narrow_val = builder
            .build_int_truncate(val_i64, narrow_ty, "narrow_val")
            .map_err(|e| format!("ICE: {e}"))?;
        builder
            .build_store(ptr, narrow_val)
            .map_err(|e| format!("ICE: {e}"))?;
        builder
            .build_return(Some(&i64_type.const_int(0, false)))
            .map_err(|e| format!("ICE: {e}"))?;
    }

    // --- time_now_ms: call gettimeofday, return seconds*1000 + usec/1000 ---
    // struct timeval { long tv_sec; long tv_usec; } — on 64-bit systems, both i64
    {
        let timeval_ty = context.struct_type(&[i64_type.into(), i64_type.into()], false);
        let timeval_ptr_ty = timeval_ty.ptr_type(AddressSpace::default());

        // declare i32 @gettimeofday(timeval*, i8*)
        let gettime_ty =
            i32_type.fn_type(&[timeval_ptr_ty.into(), i8_ptr.into()], false);
        let gettime_fn = module
            .get_function("gettimeofday")
            .unwrap_or_else(|| module.add_function("gettimeofday", gettime_ty, None));

        let fn_type = i64_type.fn_type(&[], false);
        let func = module.add_function("time_now_ms", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let tv_alloca = builder
            .build_alloca(timeval_ty, "tv")
            .map_err(|e| format!("ICE: {e}"))?;
        let null_tz = i8_ptr.const_null();
        builder
            .build_call(gettime_fn, &[tv_alloca.into(), null_tz.into()], "gtd")
            .map_err(|e| format!("ICE: {e}"))?;
        let sec_ptr = builder
            .build_struct_gep(timeval_ty, tv_alloca, 0, "sec_ptr")
            .map_err(|e| format!("ICE: {e}"))?;
        let usec_ptr = builder
            .build_struct_gep(timeval_ty, tv_alloca, 1, "usec_ptr")
            .map_err(|e| format!("ICE: {e}"))?;
        let sec = builder
            .build_load(i64_type, sec_ptr, "sec")
            .map_err(|e| format!("ICE: {e}"))?
            .into_int_value();
        let usec = builder
            .build_load(i64_type, usec_ptr, "usec")
            .map_err(|e| format!("ICE: {e}"))?
            .into_int_value();
        let sec_ms = builder
            .build_int_mul(sec, i64_type.const_int(1000, false), "sec_ms")
            .map_err(|e| format!("ICE: {e}"))?;
        let usec_ms = builder
            .build_int_signed_div(usec, i64_type.const_int(1000, false), "usec_ms")
            .map_err(|e| format!("ICE: {e}"))?;
        let total = builder
            .build_int_add(sec_ms, usec_ms, "total")
            .map_err(|e| format!("ICE: {e}"))?;
        builder
            .build_return(Some(&total))
            .map_err(|e| format!("ICE: {e}"))?;
    }

    // --- call_poll: indirect call through function pointer (i64 poll_fn, i64 future_ptr) -> i64 ---
    {
        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let func = module.add_function("call_poll", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let poll_fn_i64 = func
            .get_nth_param(0)
            .ok_or("ICE: call_poll missing param")?
            .into_int_value();
        let future_ptr = func
            .get_nth_param(1)
            .ok_or("ICE: call_poll missing param")?
            .into_int_value();
        // Cast i64 -> function pointer: i64 (*)(i64)
        let poll_fn_sig = i64_type.fn_type(&[i64_type.into()], false);
        let poll_fn_ptr_ty = poll_fn_sig.ptr_type(AddressSpace::default());
        let poll_fn_ptr = builder
            .build_int_to_ptr(poll_fn_i64, poll_fn_ptr_ty, "poll_fn_ptr")
            .map_err(|e| format!("ICE: {e}"))?;
        let call = builder
            .build_indirect_call(poll_fn_sig, poll_fn_ptr, &[future_ptr.into()], "poll_call")
            .map_err(|e| format!("ICE: {e}"))?;
        let result = call
            .try_as_basic_value()
            .left()
            .ok_or("ICE: call_poll returned void")?;
        builder
            .build_return(Some(&result))
            .map_err(|e| format!("ICE: {e}"))?;
    }

    // --- extract_poll_status: result & 1 (low bit = status: 0=Pending, 1=Ready) ---
    // --- extract_poll_value: result >> 1 (shift off status bit) ---
    for (name, is_value) in [("extract_poll_status", false), ("extract_poll_value", true)] {
        let fn_type = i64_type.fn_type(&[i64_type.into()], false);
        let func = module.add_function(name, fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let arg = func
            .get_nth_param(0)
            .ok_or("ICE: extract_poll_* missing param")?
            .into_int_value();
        let result = if is_value {
            builder
                .build_right_shift(arg, i64_type.const_int(1, false), true, "val")
                .map_err(|e| format!("ICE: {e}"))?
        } else {
            builder
                .build_and(arg, i64_type.const_int(1, false), "status")
                .map_err(|e| format!("ICE: {e}"))?
        };
        builder
            .build_return(Some(&result))
            .map_err(|e| format!("ICE: {e}"))?;
    }

    // --- async_platform: return platform ID (1=macOS, 2=Linux, 3=Windows) ---
    {
        let fn_type = i64_type.fn_type(&[], false);
        let func = module.add_function("async_platform", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let platform_id = if cfg!(target_os = "macos") {
            1
        } else if cfg!(target_os = "linux") {
            2
        } else if cfg!(target_os = "windows") {
            3
        } else {
            0
        };
        builder
            .build_return(Some(&i64_type.const_int(platform_id, false)))
            .map_err(|e| format!("ICE: {e}"))?;
    }

    // --- write_byte(fd, val) -> i64: write single byte via write() syscall ---
    {
        let write_fn = module
            .get_function("write")
            .ok_or("ICE: write must be declared")?;
        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let func = module.add_function("write_byte", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let fd_i64 = func
            .get_nth_param(0)
            .ok_or("ICE: write_byte missing param")?
            .into_int_value();
        let val_i64 = func
            .get_nth_param(1)
            .ok_or("ICE: write_byte missing param")?
            .into_int_value();
        let fd_i32 = builder
            .build_int_truncate(fd_i64, i32_type, "fd")
            .map_err(|e| format!("ICE: {e}"))?;
        let val_i8 = builder
            .build_int_truncate(val_i64, context.i8_type(), "byte")
            .map_err(|e| format!("ICE: {e}"))?;
        let buf = builder
            .build_alloca(context.i8_type(), "buf")
            .map_err(|e| format!("ICE: {e}"))?;
        builder
            .build_store(buf, val_i8)
            .map_err(|e| format!("ICE: {e}"))?;
        let call = builder
            .build_call(
                write_fn,
                &[fd_i32.into(), buf.into(), i64_type.const_int(1, false).into()],
                "w",
            )
            .map_err(|e| format!("ICE: {e}"))?;
        let result = call
            .try_as_basic_value()
            .left()
            .ok_or("ICE: write returned void")?;
        builder
            .build_return(Some(&result))
            .map_err(|e| format!("ICE: {e}"))?;
    }

    // --- read_byte(fd) -> i64: read single byte via read() syscall ---
    {
        let read_fn = module
            .get_function("read")
            .ok_or("ICE: read must be declared")?;
        let fn_type = i64_type.fn_type(&[i64_type.into()], false);
        let func = module.add_function("read_byte", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let fd_i64 = func
            .get_nth_param(0)
            .ok_or("ICE: read_byte missing param")?
            .into_int_value();
        let fd_i32 = builder
            .build_int_truncate(fd_i64, i32_type, "fd")
            .map_err(|e| format!("ICE: {e}"))?;
        let buf = builder
            .build_alloca(context.i8_type(), "buf")
            .map_err(|e| format!("ICE: {e}"))?;
        builder
            .build_call(
                read_fn,
                &[fd_i32.into(), buf.into(), i64_type.const_int(1, false).into()],
                "r",
            )
            .map_err(|e| format!("ICE: {e}"))?;
        let byte = builder
            .build_load(context.i8_type(), buf, "byte")
            .map_err(|e| format!("ICE: {e}"))?
            .into_int_value();
        let widened = builder
            .build_int_s_extend(byte, i64_type, "wide")
            .map_err(|e| format!("ICE: {e}"))?;
        builder
            .build_return(Some(&widened))
            .map_err(|e| format!("ICE: {e}"))?;
    }

    // --- epoll_set_timer_ms / iocp_set_timer_ms / kevent_* : stub returns 0 ---
    // These are platform-specific reactor hooks; the production runtime
    // provides full implementations. For Inkwell builds we emit stubs so the
    // module links — callers that require these to fire must use the text IR
    // backend or link against a full runtime library.
    for name in [
        "epoll_set_timer_ms",
        "iocp_set_timer_ms",
        "kevent_register",
        "kevent_wait",
        "kevent_get_fd",
        "kevent_get_filter",
    ] {
        let nparams = match name {
            "kevent_register" | "kevent_wait" => 4,
            "kevent_get_fd" | "kevent_get_filter" => 2,
            "epoll_set_timer_ms" | "iocp_set_timer_ms" => 3,
            _ => 0,
        };
        let param_tys: Vec<_> = (0..nparams).map(|_| i64_type.into()).collect();
        let fn_type = i64_type.fn_type(&param_tys, false);
        let func = module.add_function(name, fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        builder
            .build_return(Some(&i64_type.const_int(0, false)))
            .map_err(|e| format!("ICE: {e}"))?;
    }

    // Suppress unused warning for void_type
    let _ = void_type;

    Ok(())
}

fn declare_math_functions<'ctx>(context: &'ctx Context, module: &Module<'ctx>) {
    let f64_type = context.f64_type();

    // f64 -> f64 unary functions
    let unary_f64 = f64_type.fn_type(&[f64_type.into()], false);
    for name in &[
        "sqrt", "sin", "cos", "tan", "asin", "acos", "atan", "exp", "log", "log10", "log2",
        "floor", "ceil", "round", "fabs",
    ] {
        module.add_function(name, unary_f64, None);
    }

    // f64 x f64 -> f64 binary functions
    let binary_f64 = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
    for name in &["pow", "atan2", "fmod"] {
        module.add_function(name, binary_f64, None);
    }
}
