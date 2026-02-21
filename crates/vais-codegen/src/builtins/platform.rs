use super::*;

impl CodeGenerator {
    pub(super) fn register_stdlib_functions(&mut self) {
        // --- Number conversion functions ---

        // atoi: (s: str) -> i32 - string to integer
        register_extern!(
            self,
            "atoi",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::I32
        );

        // atol: (s: str) -> i64 - string to long integer
        register_extern!(
            self,
            "atol",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // atol_ptr: (s: str) -> i64 - atol with pointer param
        register_extern!(self, "atol_ptr" => "atol",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // atof: (s: str) -> f64 - string to double
        register_extern!(
            self,
            "atof",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::F64
        );

        // atof_ptr: (s: str) -> f64 - atof with pointer param
        register_extern!(self, "atof_ptr" => "atof",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::F64
        );

        // --- Math functions ---

        // labs: (x: i64) -> i64 - absolute value (long integer)
        register_extern!(
            self,
            "labs",
            vec![("x".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fabs: (x: f64) -> f64 - absolute value (double)
        register_extern!(
            self,
            "fabs",
            vec![("x".to_string(), ResolvedType::F64)],
            ResolvedType::F64
        );

        // sqrt: (x: f64) -> f64 - square root
        register_extern!(
            self,
            "sqrt",
            vec![("x".to_string(), ResolvedType::F64)],
            ResolvedType::F64
        );

        // sin: (x: f64) -> f64 - sine
        register_extern!(
            self,
            "sin",
            vec![("x".to_string(), ResolvedType::F64)],
            ResolvedType::F64
        );

        // cos: (x: f64) -> f64 - cosine
        register_extern!(
            self,
            "cos",
            vec![("x".to_string(), ResolvedType::F64)],
            ResolvedType::F64
        );

        // exp: (x: f64) -> f64 - exponential
        register_extern!(
            self,
            "exp",
            vec![("x".to_string(), ResolvedType::F64)],
            ResolvedType::F64
        );

        // log: (x: f64) -> f64 - natural logarithm
        register_extern!(
            self,
            "log",
            vec![("x".to_string(), ResolvedType::F64)],
            ResolvedType::F64
        );

        // rand: () -> i32 - pseudo-random number
        register_extern!(self, "rand", vec![], ResolvedType::I32);

        // srand: (seed: i32) -> void - seed random number generator
        register_extern!(
            self,
            "srand",
            vec![("seed".to_string(), ResolvedType::I32)],
            ResolvedType::Unit
        );

        // --- Character classification functions ---

        // isdigit: (c: i32) -> i32 - test if digit
        register_extern!(
            self,
            "isdigit",
            vec![("c".to_string(), ResolvedType::I32)],
            ResolvedType::I32
        );

        // isalpha: (c: i32) -> i32 - test if alphabetic
        register_extern!(
            self,
            "isalpha",
            vec![("c".to_string(), ResolvedType::I32)],
            ResolvedType::I32
        );

        // toupper: (c: i32) -> i32 - convert to uppercase
        register_extern!(
            self,
            "toupper",
            vec![("c".to_string(), ResolvedType::I32)],
            ResolvedType::I32
        );

        // tolower: (c: i32) -> i32 - convert to lowercase
        register_extern!(
            self,
            "tolower",
            vec![("c".to_string(), ResolvedType::I32)],
            ResolvedType::I32
        );

        // --- String manipulation functions ---

        // strcpy: (dest: i64, src: str) -> i64 - copy string
        register_extern!(
            self,
            "strcpy",
            vec![
                ("dest".to_string(), ResolvedType::I64),
                ("src".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // strcat: (dest: i64, src: str) -> i64 - concatenate string
        register_extern!(
            self,
            "strcat",
            vec![
                ("dest".to_string(), ResolvedType::I64),
                ("src".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );
    }

    pub(super) fn register_async_functions(&mut self) {
        // usleep: microsecond sleep for cooperative scheduling
        register_extern!(
            self,
            "usleep",
            vec![("usec".to_string(), ResolvedType::I64)],
            ResolvedType::I32
        );

        // sched_yield: yield CPU to other processes
        register_extern!(self, "sched_yield", vec![], ResolvedType::I32);

        // call_poll: call an indirect poll function pointer with a future pointer
        // Returns an i64 encoding {status, value} as a packed struct
        register_helper!(self, "call_poll" => "__call_poll",
            vec![
                ("poll_fn".to_string(), ResolvedType::I64),
                ("future_ptr".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // extract_poll_status: extract status (0=Pending, 1=Ready) from poll result
        register_helper!(self, "extract_poll_status" => "__extract_poll_status",
            vec![("poll_result".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // extract_poll_value: extract value from poll result
        register_helper!(self, "extract_poll_value" => "__extract_poll_value",
            vec![("poll_result".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // time_now_ms: current time in milliseconds
        register_helper!(self, "time_now_ms" => "__time_now_ms",
            vec![],
            ResolvedType::I64
        );

        // === Platform I/O syscalls for async reactor ===

        // kqueue helpers: only on macOS (uses kevent syscall)
        #[cfg(target_os = "macos")]
        {
            // kqueue: create kqueue instance (macOS)
            register_extern!(self, "kqueue", vec![], ResolvedType::I64);

            // kevent_register: register event with kqueue
            register_helper!(self, "kevent_register" => "__kevent_register",
                vec![
                    ("kq".to_string(), ResolvedType::I64),
                    ("fd".to_string(), ResolvedType::I64),
                    ("filter".to_string(), ResolvedType::I64),
                    ("flags".to_string(), ResolvedType::I64),
                ],
                ResolvedType::I64
            );

            // kevent_wait: wait for events
            register_helper!(self, "kevent_wait" => "__kevent_wait",
                vec![
                    ("kq".to_string(), ResolvedType::I64),
                    ("events_buf".to_string(), ResolvedType::I64),
                    ("max_events".to_string(), ResolvedType::I64),
                    ("timeout_ms".to_string(), ResolvedType::I64),
                ],
                ResolvedType::I64
            );

            // kevent_get_fd: get fd from event at index
            register_helper!(self, "kevent_get_fd" => "__kevent_get_fd",
                vec![
                    ("events_buf".to_string(), ResolvedType::I64),
                    ("index".to_string(), ResolvedType::I64),
                ],
                ResolvedType::I64
            );

            // kevent_get_filter: get filter from event at index
            register_helper!(self, "kevent_get_filter" => "__kevent_get_filter",
                vec![
                    ("events_buf".to_string(), ResolvedType::I64),
                    ("index".to_string(), ResolvedType::I64),
                ],
                ResolvedType::I64
            );
        }

        // close: close file descriptor
        register_extern!(
            self,
            "close",
            vec![("fd".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // pipe: create pipe (takes buffer pointer for two fds)
        register_extern!(self, "pipe" => "pipe",
            vec![("fds_buf".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // write_byte: write a single byte to fd
        register_helper!(self, "write_byte" => "__write_byte",
            vec![
                ("fd".to_string(), ResolvedType::I64),
                ("value".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // read_byte: read a single byte from fd
        register_helper!(self, "read_byte" => "__read_byte",
            vec![("fd".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // === Cross-platform async reactor support ===

        // async_platform: returns platform ID (1=macOS, 2=Linux, 3=Windows)
        register_helper!(self, "async_platform" => "__async_platform",
            vec![],
            ResolvedType::I64
        );

        // epoll_set_timer_ms: configure timerfd delay (Linux epoll backend)
        register_helper!(self, "epoll_set_timer_ms" => "__epoll_set_timer_ms",
            vec![
                ("kq".to_string(), ResolvedType::I64),
                ("timer_id".to_string(), ResolvedType::I64),
                ("delay_ms".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // iocp_set_timer_ms: configure timer delay (Windows IOCP backend)
        register_helper!(self, "iocp_set_timer_ms" => "__iocp_set_timer_ms",
            vec![
                ("kq".to_string(), ResolvedType::I64),
                ("timer_id".to_string(), ResolvedType::I64),
                ("delay_ms".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );
    }

    pub(super) fn register_simd_functions(&mut self) {
        // Helper to create vector types
        let vec2f32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F32),
            lanes: 2,
        };
        let vec4f32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F32),
            lanes: 4,
        };
        let vec8f32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F32),
            lanes: 8,
        };
        let vec2f64 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F64),
            lanes: 2,
        };
        let vec4f64 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F64),
            lanes: 4,
        };
        let vec4i32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::I32),
            lanes: 4,
        };
        let vec8i32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::I32),
            lanes: 8,
        };
        let vec2i64 = ResolvedType::Vector {
            element: Box::new(ResolvedType::I64),
            lanes: 2,
        };
        let vec4i64 = ResolvedType::Vector {
            element: Box::new(ResolvedType::I64),
            lanes: 4,
        };

        // === Vector Constructors ===
        register_helper!(self, "vec2f32" => "vec2f32",
            vec![("x".to_string(), ResolvedType::F32), ("y".to_string(), ResolvedType::F32)],
            vec2f32.clone()
        );

        register_helper!(self, "vec4f32" => "vec4f32",
            vec![("x".to_string(), ResolvedType::F32), ("y".to_string(), ResolvedType::F32),
                 ("z".to_string(), ResolvedType::F32), ("w".to_string(), ResolvedType::F32)],
            vec4f32.clone()
        );

        register_helper!(self, "vec8f32" => "vec8f32",
            vec![("a".to_string(), ResolvedType::F32), ("b".to_string(), ResolvedType::F32),
                 ("c".to_string(), ResolvedType::F32), ("d".to_string(), ResolvedType::F32),
                 ("e".to_string(), ResolvedType::F32), ("f".to_string(), ResolvedType::F32),
                 ("g".to_string(), ResolvedType::F32), ("h".to_string(), ResolvedType::F32)],
            vec8f32.clone()
        );

        register_helper!(self, "vec2f64" => "vec2f64",
            vec![("x".to_string(), ResolvedType::F64), ("y".to_string(), ResolvedType::F64)],
            vec2f64.clone()
        );

        register_helper!(self, "vec4f64" => "vec4f64",
            vec![("x".to_string(), ResolvedType::F64), ("y".to_string(), ResolvedType::F64),
                 ("z".to_string(), ResolvedType::F64), ("w".to_string(), ResolvedType::F64)],
            vec4f64.clone()
        );

        register_helper!(self, "vec4i32" => "vec4i32",
            vec![("x".to_string(), ResolvedType::I32), ("y".to_string(), ResolvedType::I32),
                 ("z".to_string(), ResolvedType::I32), ("w".to_string(), ResolvedType::I32)],
            vec4i32.clone()
        );

        register_helper!(self, "vec8i32" => "vec8i32",
            vec![("a".to_string(), ResolvedType::I32), ("b".to_string(), ResolvedType::I32),
                 ("c".to_string(), ResolvedType::I32), ("d".to_string(), ResolvedType::I32),
                 ("e".to_string(), ResolvedType::I32), ("f".to_string(), ResolvedType::I32),
                 ("g".to_string(), ResolvedType::I32), ("h".to_string(), ResolvedType::I32)],
            vec8i32.clone()
        );

        register_helper!(self, "vec2i64" => "vec2i64",
            vec![("x".to_string(), ResolvedType::I64), ("y".to_string(), ResolvedType::I64)],
            vec2i64.clone()
        );

        register_helper!(self, "vec4i64" => "vec4i64",
            vec![("x".to_string(), ResolvedType::I64), ("y".to_string(), ResolvedType::I64),
                 ("z".to_string(), ResolvedType::I64), ("w".to_string(), ResolvedType::I64)],
            vec4i64.clone()
        );

        // === SIMD Arithmetic Operations ===
        macro_rules! register_simd_binop {
            ($name:expr, $vec_ty:expr) => {
                register_helper!(self, $name => $name,
                    vec![("a".to_string(), $vec_ty.clone()), ("b".to_string(), $vec_ty.clone())],
                    $vec_ty.clone()
                );
            };
        }

        // Vec4f32 operations
        register_simd_binop!("simd_add_vec4f32", vec4f32);
        register_simd_binop!("simd_sub_vec4f32", vec4f32);
        register_simd_binop!("simd_mul_vec4f32", vec4f32);
        register_simd_binop!("simd_div_vec4f32", vec4f32);

        // Vec8f32 operations
        register_simd_binop!("simd_add_vec8f32", vec8f32);
        register_simd_binop!("simd_sub_vec8f32", vec8f32);
        register_simd_binop!("simd_mul_vec8f32", vec8f32);
        register_simd_binop!("simd_div_vec8f32", vec8f32);

        // Vec2f64 operations
        register_simd_binop!("simd_add_vec2f64", vec2f64);
        register_simd_binop!("simd_sub_vec2f64", vec2f64);
        register_simd_binop!("simd_mul_vec2f64", vec2f64);
        register_simd_binop!("simd_div_vec2f64", vec2f64);

        // Vec4f64 operations
        register_simd_binop!("simd_add_vec4f64", vec4f64);
        register_simd_binop!("simd_sub_vec4f64", vec4f64);
        register_simd_binop!("simd_mul_vec4f64", vec4f64);
        register_simd_binop!("simd_div_vec4f64", vec4f64);

        // Vec4i32 operations
        register_simd_binop!("simd_add_vec4i32", vec4i32);
        register_simd_binop!("simd_sub_vec4i32", vec4i32);
        register_simd_binop!("simd_mul_vec4i32", vec4i32);

        // Vec8i32 operations
        register_simd_binop!("simd_add_vec8i32", vec8i32);
        register_simd_binop!("simd_sub_vec8i32", vec8i32);
        register_simd_binop!("simd_mul_vec8i32", vec8i32);

        // Vec2i64 operations
        register_simd_binop!("simd_add_vec2i64", vec2i64);
        register_simd_binop!("simd_sub_vec2i64", vec2i64);
        register_simd_binop!("simd_mul_vec2i64", vec2i64);

        // Vec4i64 operations
        register_simd_binop!("simd_add_vec4i64", vec4i64);
        register_simd_binop!("simd_sub_vec4i64", vec4i64);
        register_simd_binop!("simd_mul_vec4i64", vec4i64);

        // === Horizontal Reduction Operations ===
        register_helper!(self, "simd_reduce_add_vec4f32" => "simd_reduce_add_vec4f32",
            vec![("v".to_string(), vec4f32)], ResolvedType::F32);
        register_helper!(self, "simd_reduce_add_vec8f32" => "simd_reduce_add_vec8f32",
            vec![("v".to_string(), vec8f32)], ResolvedType::F32);
        register_helper!(self, "simd_reduce_add_vec2f64" => "simd_reduce_add_vec2f64",
            vec![("v".to_string(), vec2f64)], ResolvedType::F64);
        register_helper!(self, "simd_reduce_add_vec4f64" => "simd_reduce_add_vec4f64",
            vec![("v".to_string(), vec4f64)], ResolvedType::F64);
        register_helper!(self, "simd_reduce_add_vec4i32" => "simd_reduce_add_vec4i32",
            vec![("v".to_string(), vec4i32)], ResolvedType::I32);
        register_helper!(self, "simd_reduce_add_vec8i32" => "simd_reduce_add_vec8i32",
            vec![("v".to_string(), vec8i32)], ResolvedType::I32);
        register_helper!(self, "simd_reduce_add_vec2i64" => "simd_reduce_add_vec2i64",
            vec![("v".to_string(), vec2i64)], ResolvedType::I64);
        register_helper!(self, "simd_reduce_add_vec4i64" => "simd_reduce_add_vec4i64",
            vec![("v".to_string(), vec4i64)], ResolvedType::I64);
    }

    pub(super) fn register_gc_functions(&mut self) {
        // GC runtime functions
        register_extern!(self, "vais_gc_init", vec![], ResolvedType::I64);

        register_extern!(
            self,
            "vais_gc_alloc",
            vec![
                ("size".to_string(), ResolvedType::I64),
                ("type_id".to_string(), ResolvedType::I32),
            ],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "vais_gc_add_root",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "vais_gc_remove_root",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        register_extern!(self, "vais_gc_collect", vec![], ResolvedType::I64);

        register_extern!(self, "vais_gc_bytes_allocated", vec![], ResolvedType::I64);

        register_extern!(self, "vais_gc_objects_count", vec![], ResolvedType::I64);

        register_extern!(self, "vais_gc_collections", vec![], ResolvedType::I64);

        register_extern!(
            self,
            "vais_gc_set_threshold",
            vec![("threshold".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        register_extern!(self, "vais_gc_print_stats", vec![], ResolvedType::I64);
    }

    pub(super) fn register_system_functions(&mut self) {
        // Environment variable functions
        register_extern!(
            self,
            "getenv",
            vec![("name".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "setenv",
            vec![
                ("name".to_string(), ResolvedType::Str),
                ("value".to_string(), ResolvedType::Str),
                ("overwrite".to_string(), ResolvedType::I32),
            ],
            ResolvedType::I32
        );

        register_extern!(
            self,
            "unsetenv",
            vec![("name".to_string(), ResolvedType::Str)],
            ResolvedType::I32
        );

        // Process execution functions
        register_extern!(
            self,
            "system",
            vec![("command".to_string(), ResolvedType::Str)],
            ResolvedType::I32
        );

        register_extern!(
            self,
            "popen",
            vec![
                ("command".to_string(), ResolvedType::Str),
                ("mode".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "pclose",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I32
        );

        // Exit
        register_extern!(
            self,
            "exit",
            vec![("status".to_string(), ResolvedType::I32)],
            ResolvedType::Unit
        );

        // Signal handling
        register_extern!(
            self,
            "signal",
            vec![
                ("signum".to_string(), ResolvedType::I32),
                ("handler".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "raise",
            vec![("signum".to_string(), ResolvedType::I32)],
            ResolvedType::I32
        );
    }
}
