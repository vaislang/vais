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
    module.add_function(
        "memcpy",
        i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into(), i64_type.into()], false),
        None,
    );
    // memcpy_str: wrapper that calls memcpy (used by selfhost codegen.vais)
    {
        let memcpy_fn = module.get_function("memcpy").unwrap();
        let fn_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into(), i64_type.into()], false);
        let func = module.add_function("memcpy_str", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let result = builder
            .build_call(
                memcpy_fn,
                &[
                    func.get_nth_param(0).unwrap().into(),
                    func.get_nth_param(1).unwrap().into(),
                    func.get_nth_param(2).unwrap().into(),
                ],
                "result",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap();
        builder.build_return(Some(&result)).unwrap();
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
        let fopen_fn = module.get_function("fopen").unwrap();
        let fn_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let func = module.add_function("fopen_ptr", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let result = builder
            .build_call(
                fopen_fn,
                &[
                    func.get_nth_param(0).unwrap().into(),
                    func.get_nth_param(1).unwrap().into(),
                ],
                "result",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap();
        builder.build_return(Some(&result)).unwrap();
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
        let stdin_global = module.add_global(i8_ptr, Some(AddressSpace::default()), "__stdinp");
        stdin_global.set_externally_initialized(true);
        let val = builder
            .build_load(i8_ptr, stdin_global.as_pointer_value(), "stdin_val")
            .unwrap();
        builder.build_return(Some(&val)).unwrap();
    }
    // get_stdout() -> FILE* (returns stdout stream)
    {
        let fn_type = i8_ptr.fn_type(&[], false);
        let func = module.add_function("get_stdout", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);
        let stdout_global =
            module.add_global(i8_ptr, Some(AddressSpace::default()), "__stdoutp");
        stdout_global.set_externally_initialized(true);
        let val = builder
            .build_load(i8_ptr, stdout_global.as_pointer_value(), "stdout_val")
            .unwrap();
        builder.build_return(Some(&val)).unwrap();
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
    // close(fd: i32) -> i32
    module.add_function("close", i32_type.fn_type(&[i32_type.into()], false), None);
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
    define_simd_builtins(context, module);

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
}

/// Define SIMD built-in functions with actual implementations.
/// Uses heap-allocated arrays behind i8* pointers to represent SIMD vectors,
/// matching the text backend's approach.
fn define_simd_builtins<'ctx>(context: &'ctx Context, module: &Module<'ctx>) {
    let builder = context.create_builder();
    let i8_ptr = context.i8_type().ptr_type(AddressSpace::default());
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();

    let malloc_fn = module.get_function("malloc").unwrap_or_else(|| {
        module.add_function("malloc", i8_ptr.fn_type(&[i64_type.into()], false), None)
    });

    // Helper: define a vec constructor that allocates N elements and stores args
    // vec4i32(a, b, c, d) -> i8* (heap-allocated [i32 x 4])
    {
        let fn_type = i8_ptr.fn_type(
            &[
                i32_type.into(),
                i32_type.into(),
                i32_type.into(),
                i32_type.into(),
            ],
            false,
        );
        let func = module.add_function("vec4i32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        // malloc(16) for 4 x i32
        let ptr = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "ptr")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let i32_ptr = builder
            .build_pointer_cast(ptr, i32_type.ptr_type(AddressSpace::default()), "i32ptr")
            .unwrap();
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        i32_type,
                        i32_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .unwrap()
            };
            builder
                .build_store(gep, func.get_nth_param(i).unwrap().into_int_value())
                .unwrap();
        }
        builder.build_return(Some(&ptr)).unwrap();
    }

    // vec4f32(a, b, c, d) -> i8* (heap-allocated [float x 4])
    {
        let fn_type = i8_ptr.fn_type(
            &[
                f64_type.into(),
                f64_type.into(),
                f64_type.into(),
                f64_type.into(),
            ],
            false,
        );
        let func = module.add_function("vec4f32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let ptr = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "ptr")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let f32_ptr = builder
            .build_pointer_cast(ptr, f32_type.ptr_type(AddressSpace::default()), "f32ptr")
            .unwrap();
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        f32_type,
                        f32_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .unwrap()
            };
            // Truncate f64 param to f32
            let param = func.get_nth_param(i).unwrap().into_float_value();
            let f32_val = builder.build_float_trunc(param, f32_type, "trunc").unwrap();
            builder.build_store(gep, f32_val).unwrap();
        }
        builder.build_return(Some(&ptr)).unwrap();
    }

    // vec2i64(a, b) -> i8*
    {
        let fn_type = i8_ptr.fn_type(&[i64_type.into(), i64_type.into()], false);
        let func = module.add_function("vec2i64", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let ptr = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "ptr")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let i64_ptr = builder
            .build_pointer_cast(ptr, i64_type.ptr_type(AddressSpace::default()), "i64ptr")
            .unwrap();
        for i in 0..2u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        i64_type,
                        i64_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .unwrap()
            };
            builder
                .build_store(gep, func.get_nth_param(i).unwrap().into_int_value())
                .unwrap();
        }
        builder.build_return(Some(&ptr)).unwrap();
    }

    // Helper macro for SIMD binary operations on i32x4
    // pattern: load 4 elements from each, do op, store into new alloc, return
    for (name, op) in &[
        ("simd_add_vec4i32", "add"),
        ("simd_sub_vec4i32", "sub"),
        ("simd_mul_vec4i32", "mul"),
    ] {
        let fn_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let func = module.add_function(name, fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);

        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0).unwrap().into_pointer_value(),
                i32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .unwrap();
        let b_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(1).unwrap().into_pointer_value(),
                i32_type.ptr_type(AddressSpace::default()),
                "b",
            )
            .unwrap();
        let out = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "out")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let out_i32 = builder
            .build_pointer_cast(out, i32_type.ptr_type(AddressSpace::default()), "out_i32")
            .unwrap();

        for i in 0..4u32 {
            let idx = i32_type.const_int(i as u64, false);
            let a_gep = unsafe { builder.build_gep(i32_type, a_ptr, &[idx], "a_gep").unwrap() };
            let b_gep = unsafe { builder.build_gep(i32_type, b_ptr, &[idx], "b_gep").unwrap() };
            let a_val = builder
                .build_load(i32_type, a_gep, "a_val")
                .unwrap()
                .into_int_value();
            let b_val = builder
                .build_load(i32_type, b_gep, "b_val")
                .unwrap()
                .into_int_value();
            let result = match *op {
                "add" => builder.build_int_add(a_val, b_val, "r").unwrap(),
                "sub" => builder.build_int_sub(a_val, b_val, "r").unwrap(),
                "mul" => builder.build_int_mul(a_val, b_val, "r").unwrap(),
                _ => unreachable!(),
            };
            let o_gep = unsafe {
                builder
                    .build_gep(i32_type, out_i32, &[idx], "o_gep")
                    .unwrap()
            };
            builder.build_store(o_gep, result).unwrap();
        }
        builder.build_return(Some(&out)).unwrap();
    }

    // simd_reduce_add_vec4i32: sum all 4 i32 elements -> i64
    {
        let fn_type = i64_type.fn_type(&[i8_ptr.into()], false);
        let func = module.add_function("simd_reduce_add_vec4i32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0).unwrap().into_pointer_value(),
                i32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .unwrap();
        let mut sum = i64_type.const_int(0, false);
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        i32_type,
                        a_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .unwrap()
            };
            let val = builder
                .build_load(i32_type, gep, "val")
                .unwrap()
                .into_int_value();
            let ext = builder.build_int_s_extend(val, i64_type, "ext").unwrap();
            sum = builder.build_int_add(sum, ext, "sum").unwrap();
        }
        builder.build_return(Some(&sum)).unwrap();
    }

    // simd_reduce_mul_vec4i32
    {
        let fn_type = i64_type.fn_type(&[i8_ptr.into()], false);
        let func = module.add_function("simd_reduce_mul_vec4i32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0).unwrap().into_pointer_value(),
                i32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .unwrap();
        let mut prod = i64_type.const_int(1, false);
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        i32_type,
                        a_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .unwrap()
            };
            let val = builder
                .build_load(i32_type, gep, "val")
                .unwrap()
                .into_int_value();
            let ext = builder.build_int_s_extend(val, i64_type, "ext").unwrap();
            prod = builder.build_int_mul(prod, ext, "prod").unwrap();
        }
        builder.build_return(Some(&prod)).unwrap();
    }

    // SIMD f32x4 binary ops
    for (name, op) in &[
        ("simd_add_vec4f32", "fadd"),
        ("simd_sub_vec4f32", "fsub"),
        ("simd_mul_vec4f32", "fmul"),
        ("simd_div_vec4f32", "fdiv"),
    ] {
        let fn_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let func = module.add_function(name, fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);

        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0).unwrap().into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .unwrap();
        let b_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(1).unwrap().into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "b",
            )
            .unwrap();
        let out = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "out")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let out_f32 = builder
            .build_pointer_cast(out, f32_type.ptr_type(AddressSpace::default()), "out_f32")
            .unwrap();

        for i in 0..4u32 {
            let idx = i32_type.const_int(i as u64, false);
            let a_gep = unsafe { builder.build_gep(f32_type, a_ptr, &[idx], "a_gep").unwrap() };
            let b_gep = unsafe { builder.build_gep(f32_type, b_ptr, &[idx], "b_gep").unwrap() };
            let a_val = builder
                .build_load(f32_type, a_gep, "a_val")
                .unwrap()
                .into_float_value();
            let b_val = builder
                .build_load(f32_type, b_gep, "b_val")
                .unwrap()
                .into_float_value();
            let result = match *op {
                "fadd" => builder.build_float_add(a_val, b_val, "r").unwrap(),
                "fsub" => builder.build_float_sub(a_val, b_val, "r").unwrap(),
                "fmul" => builder.build_float_mul(a_val, b_val, "r").unwrap(),
                "fdiv" => builder.build_float_div(a_val, b_val, "r").unwrap(),
                _ => unreachable!(),
            };
            let o_gep = unsafe {
                builder
                    .build_gep(f32_type, out_f32, &[idx], "o_gep")
                    .unwrap()
            };
            builder.build_store(o_gep, result).unwrap();
        }
        builder.build_return(Some(&out)).unwrap();
    }

    // simd_reduce_add_vec4f32: sum all 4 f32 elements -> f64
    {
        let fn_type = f64_type.fn_type(&[i8_ptr.into()], false);
        let func = module.add_function("simd_reduce_add_vec4f32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0).unwrap().into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .unwrap();
        let mut sum = f64_type.const_float(0.0);
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        f32_type,
                        a_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .unwrap()
            };
            let val = builder
                .build_load(f32_type, gep, "val")
                .unwrap()
                .into_float_value();
            let ext = builder.build_float_ext(val, f64_type, "ext").unwrap();
            sum = builder.build_float_add(sum, ext, "sum").unwrap();
        }
        builder.build_return(Some(&sum)).unwrap();
    }

    // simd_reduce_mul_vec4f32
    {
        let fn_type = f64_type.fn_type(&[i8_ptr.into()], false);
        let func = module.add_function("simd_reduce_mul_vec4f32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0).unwrap().into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .unwrap();
        let mut prod = f64_type.const_float(1.0);
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        f32_type,
                        a_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .unwrap()
            };
            let val = builder
                .build_load(f32_type, gep, "val")
                .unwrap()
                .into_float_value();
            let ext = builder.build_float_ext(val, f64_type, "ext").unwrap();
            prod = builder.build_float_mul(prod, ext, "prod").unwrap();
        }
        builder.build_return(Some(&prod)).unwrap();
    }

    // simd_dot_vec4f32(a, b) -> f64: sum of element-wise products
    {
        let fn_type = f64_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let func = module.add_function("simd_dot_vec4f32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0).unwrap().into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .unwrap();
        let b_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(1).unwrap().into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "b",
            )
            .unwrap();
        let mut sum = f64_type.const_float(0.0);
        for i in 0..4u32 {
            let idx = i32_type.const_int(i as u64, false);
            let a_gep = unsafe { builder.build_gep(f32_type, a_ptr, &[idx], "a_gep").unwrap() };
            let b_gep = unsafe { builder.build_gep(f32_type, b_ptr, &[idx], "b_gep").unwrap() };
            let a_val = builder
                .build_load(f32_type, a_gep, "a_val")
                .unwrap()
                .into_float_value();
            let b_val = builder
                .build_load(f32_type, b_gep, "b_val")
                .unwrap()
                .into_float_value();
            let prod = builder.build_float_mul(a_val, b_val, "prod").unwrap();
            let ext = builder.build_float_ext(prod, f64_type, "ext").unwrap();
            sum = builder.build_float_add(sum, ext, "sum").unwrap();
        }
        builder.build_return(Some(&sum)).unwrap();
    }

    // simd_create_vec4f32 (same as vec4f32)
    {
        let fn_type = i8_ptr.fn_type(
            &[
                f64_type.into(),
                f64_type.into(),
                f64_type.into(),
                f64_type.into(),
            ],
            false,
        );
        let func = module.add_function("simd_create_vec4f32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let ptr = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "ptr")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let f32_ptr = builder
            .build_pointer_cast(ptr, f32_type.ptr_type(AddressSpace::default()), "f32ptr")
            .unwrap();
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        f32_type,
                        f32_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .unwrap()
            };
            let param = func.get_nth_param(i).unwrap().into_float_value();
            let f32_val = builder.build_float_trunc(param, f32_type, "trunc").unwrap();
            builder.build_store(gep, f32_val).unwrap();
        }
        builder.build_return(Some(&ptr)).unwrap();
    }

    // simd_get_vec4f32(ptr, idx) -> f64
    {
        let fn_type = f64_type.fn_type(&[i8_ptr.into(), i32_type.into()], false);
        let func = module.add_function("simd_get_vec4f32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0).unwrap().into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .unwrap();
        let idx = func.get_nth_param(1).unwrap().into_int_value();
        let gep = unsafe { builder.build_gep(f32_type, a_ptr, &[idx], "gep").unwrap() };
        let val = builder
            .build_load(f32_type, gep, "val")
            .unwrap()
            .into_float_value();
        let ext = builder.build_float_ext(val, f64_type, "ext").unwrap();
        builder.build_return(Some(&ext)).unwrap();
    }

    // Vec2i64 binary ops
    {
        let fn_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let func = module.add_function("simd_add_vec2i64", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0).unwrap().into_pointer_value(),
                i64_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .unwrap();
        let b_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(1).unwrap().into_pointer_value(),
                i64_type.ptr_type(AddressSpace::default()),
                "b",
            )
            .unwrap();
        let out = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "out")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let out_i64 = builder
            .build_pointer_cast(out, i64_type.ptr_type(AddressSpace::default()), "out_i64")
            .unwrap();
        for i in 0..2u32 {
            let idx = i32_type.const_int(i as u64, false);
            let a_gep = unsafe { builder.build_gep(i64_type, a_ptr, &[idx], "a_gep").unwrap() };
            let b_gep = unsafe { builder.build_gep(i64_type, b_ptr, &[idx], "b_gep").unwrap() };
            let a_val = builder
                .build_load(i64_type, a_gep, "a_val")
                .unwrap()
                .into_int_value();
            let b_val = builder
                .build_load(i64_type, b_gep, "b_val")
                .unwrap()
                .into_int_value();
            let result = builder.build_int_add(a_val, b_val, "r").unwrap();
            let o_gep = unsafe {
                builder
                    .build_gep(i64_type, out_i64, &[idx], "o_gep")
                    .unwrap()
            };
            builder.build_store(o_gep, result).unwrap();
        }
        builder.build_return(Some(&out)).unwrap();
    }

    // simd_reduce_add_vec2i64
    {
        let fn_type = i64_type.fn_type(&[i8_ptr.into()], false);
        let func = module.add_function("simd_reduce_add_vec2i64", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0).unwrap().into_pointer_value(),
                i64_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .unwrap();
        let mut sum = i64_type.const_int(0, false);
        for i in 0..2u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        i64_type,
                        a_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .unwrap()
            };
            let val = builder
                .build_load(i64_type, gep, "val")
                .unwrap()
                .into_int_value();
            sum = builder.build_int_add(sum, val, "sum").unwrap();
        }
        builder.build_return(Some(&sum)).unwrap();
    }
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
