use super::*;

impl CodeGenerator {
    pub(super) fn register_stdlib_functions(&mut self) {
        // --- Number conversion functions ---

        // atoi: (s: str) -> i32 - string to integer
        register_extern!(
            self,
            "atoi",
            vec![(String::from("s"), ResolvedType::Str)],
            ResolvedType::I32
        );

        // atol: (s: str) -> i64 - string to long integer
        register_extern!(
            self,
            "atol",
            vec![(String::from("s"), ResolvedType::Str)],
            ResolvedType::I64
        );

        // atol_ptr: (s: str) -> i64 - atol with pointer param
        register_extern!(self, "atol_ptr" => "atol",
            vec![(String::from("s"), ResolvedType::Str)],
            ResolvedType::I64
        );

        // atof: (s: str) -> f64 - string to double
        register_extern!(
            self,
            "atof",
            vec![(String::from("s"), ResolvedType::Str)],
            ResolvedType::F64
        );

        // atof_ptr: (s: str) -> f64 - atof with pointer param
        register_extern!(self, "atof_ptr" => "atof",
            vec![(String::from("s"), ResolvedType::Str)],
            ResolvedType::F64
        );

        // --- Math functions ---

        // labs: (x: i64) -> i64 - absolute value (long integer)
        register_extern!(
            self,
            "labs",
            vec![(String::from("x"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fabs: (x: f64) -> f64 - absolute value (double)
        register_extern!(
            self,
            "fabs",
            vec![(String::from("x"), ResolvedType::F64)],
            ResolvedType::F64
        );

        // sqrt: (x: f64) -> f64 - square root
        register_extern!(
            self,
            "sqrt",
            vec![(String::from("x"), ResolvedType::F64)],
            ResolvedType::F64
        );

        // sin: (x: f64) -> f64 - sine
        register_extern!(
            self,
            "sin",
            vec![(String::from("x"), ResolvedType::F64)],
            ResolvedType::F64
        );

        // cos: (x: f64) -> f64 - cosine
        register_extern!(
            self,
            "cos",
            vec![(String::from("x"), ResolvedType::F64)],
            ResolvedType::F64
        );

        // exp: (x: f64) -> f64 - exponential
        register_extern!(
            self,
            "exp",
            vec![(String::from("x"), ResolvedType::F64)],
            ResolvedType::F64
        );

        // log: (x: f64) -> f64 - natural logarithm
        register_extern!(
            self,
            "log",
            vec![(String::from("x"), ResolvedType::F64)],
            ResolvedType::F64
        );

        // rand: () -> i32 - pseudo-random number
        register_extern!(self, "rand", vec![], ResolvedType::I32);

        // srand: (seed: i32) -> void - seed random number generator
        register_extern!(
            self,
            "srand",
            vec![(String::from("seed"), ResolvedType::I32)],
            ResolvedType::Unit
        );

        // --- Character classification functions ---

        // isdigit: (c: i32) -> i32 - test if digit
        register_extern!(
            self,
            "isdigit",
            vec![(String::from("c"), ResolvedType::I32)],
            ResolvedType::I32
        );

        // isalpha: (c: i32) -> i32 - test if alphabetic
        register_extern!(
            self,
            "isalpha",
            vec![(String::from("c"), ResolvedType::I32)],
            ResolvedType::I32
        );

        // toupper: (c: i32) -> i32 - convert to uppercase
        register_extern!(
            self,
            "toupper",
            vec![(String::from("c"), ResolvedType::I32)],
            ResolvedType::I32
        );

        // tolower: (c: i32) -> i32 - convert to lowercase
        register_extern!(
            self,
            "tolower",
            vec![(String::from("c"), ResolvedType::I32)],
            ResolvedType::I32
        );

        // --- String manipulation functions ---

        // strcpy: (dest: i64, src: str) -> i64 - copy string
        register_extern!(
            self,
            "strcpy",
            vec![
                (String::from("dest"), ResolvedType::I64),
                (String::from("src"), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // strcat: (dest: i64, src: str) -> i64 - concatenate string
        register_extern!(
            self,
            "strcat",
            vec![
                (String::from("dest"), ResolvedType::I64),
                (String::from("src"), ResolvedType::Str),
            ],
            ResolvedType::I64
        );
    }

    pub(super) fn register_async_functions(&mut self) {
        // usleep: microsecond sleep for cooperative scheduling
        register_extern!(
            self,
            "usleep",
            vec![(String::from("usec"), ResolvedType::I64)],
            ResolvedType::I32
        );

        // sched_yield: yield CPU to other processes
        register_extern!(self, "sched_yield", vec![], ResolvedType::I32);

        // call_poll: call an indirect poll function pointer with a future pointer
        // Returns an i64 encoding {status, value} as a packed struct
        register_helper!(self, "call_poll" => "__call_poll",
            vec![
                (String::from("poll_fn"), ResolvedType::I64),
                (String::from("future_ptr"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // extract_poll_status: extract status (0=Pending, 1=Ready) from poll result
        register_helper!(self, "extract_poll_status" => "__extract_poll_status",
            vec![(String::from("poll_result"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // extract_poll_value: extract value from poll result
        register_helper!(self, "extract_poll_value" => "__extract_poll_value",
            vec![(String::from("poll_result"), ResolvedType::I64)],
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
                    (String::from("kq"), ResolvedType::I64),
                    (String::from("fd"), ResolvedType::I64),
                    (String::from("filter"), ResolvedType::I64),
                    (String::from("flags"), ResolvedType::I64),
                ],
                ResolvedType::I64
            );

            // kevent_wait: wait for events
            register_helper!(self, "kevent_wait" => "__kevent_wait",
                vec![
                    (String::from("kq"), ResolvedType::I64),
                    (String::from("events_buf"), ResolvedType::I64),
                    (String::from("max_events"), ResolvedType::I64),
                    (String::from("timeout_ms"), ResolvedType::I64),
                ],
                ResolvedType::I64
            );

            // kevent_get_fd: get fd from event at index
            register_helper!(self, "kevent_get_fd" => "__kevent_get_fd",
                vec![
                    (String::from("events_buf"), ResolvedType::I64),
                    (String::from("index"), ResolvedType::I64),
                ],
                ResolvedType::I64
            );

            // kevent_get_filter: get filter from event at index
            register_helper!(self, "kevent_get_filter" => "__kevent_get_filter",
                vec![
                    (String::from("events_buf"), ResolvedType::I64),
                    (String::from("index"), ResolvedType::I64),
                ],
                ResolvedType::I64
            );
        }

        // close: close file descriptor
        register_extern!(
            self,
            "close",
            vec![(String::from("fd"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // pipe: create pipe (takes buffer pointer for two fds)
        register_extern!(self, "pipe" => "pipe",
            vec![(String::from("fds_buf"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // write_byte: write a single byte to fd
        register_helper!(self, "write_byte" => "__write_byte",
            vec![
                (String::from("fd"), ResolvedType::I64),
                (String::from("value"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // read_byte: read a single byte from fd
        register_helper!(self, "read_byte" => "__read_byte",
            vec![(String::from("fd"), ResolvedType::I64)],
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
                (String::from("kq"), ResolvedType::I64),
                (String::from("timer_id"), ResolvedType::I64),
                (String::from("delay_ms"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // iocp_set_timer_ms: configure timer delay (Windows IOCP backend)
        register_helper!(self, "iocp_set_timer_ms" => "__iocp_set_timer_ms",
            vec![
                (String::from("kq"), ResolvedType::I64),
                (String::from("timer_id"), ResolvedType::I64),
                (String::from("delay_ms"), ResolvedType::I64),
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
            vec![(String::from("x"), ResolvedType::F32), (String::from("y"), ResolvedType::F32)],
            vec2f32.clone()
        );

        register_helper!(self, "vec4f32" => "vec4f32",
            vec![(String::from("x"), ResolvedType::F32), (String::from("y"), ResolvedType::F32),
                 (String::from("z"), ResolvedType::F32), (String::from("w"), ResolvedType::F32)],
            vec4f32.clone()
        );

        register_helper!(self, "vec8f32" => "vec8f32",
            vec![(String::from("a"), ResolvedType::F32), (String::from("b"), ResolvedType::F32),
                 (String::from("c"), ResolvedType::F32), (String::from("d"), ResolvedType::F32),
                 (String::from("e"), ResolvedType::F32), (String::from("f"), ResolvedType::F32),
                 (String::from("g"), ResolvedType::F32), (String::from("h"), ResolvedType::F32)],
            vec8f32.clone()
        );

        register_helper!(self, "vec2f64" => "vec2f64",
            vec![(String::from("x"), ResolvedType::F64), (String::from("y"), ResolvedType::F64)],
            vec2f64.clone()
        );

        register_helper!(self, "vec4f64" => "vec4f64",
            vec![(String::from("x"), ResolvedType::F64), (String::from("y"), ResolvedType::F64),
                 (String::from("z"), ResolvedType::F64), (String::from("w"), ResolvedType::F64)],
            vec4f64.clone()
        );

        register_helper!(self, "vec4i32" => "vec4i32",
            vec![(String::from("x"), ResolvedType::I32), (String::from("y"), ResolvedType::I32),
                 (String::from("z"), ResolvedType::I32), (String::from("w"), ResolvedType::I32)],
            vec4i32.clone()
        );

        register_helper!(self, "vec8i32" => "vec8i32",
            vec![(String::from("a"), ResolvedType::I32), (String::from("b"), ResolvedType::I32),
                 (String::from("c"), ResolvedType::I32), (String::from("d"), ResolvedType::I32),
                 (String::from("e"), ResolvedType::I32), (String::from("f"), ResolvedType::I32),
                 (String::from("g"), ResolvedType::I32), (String::from("h"), ResolvedType::I32)],
            vec8i32.clone()
        );

        register_helper!(self, "vec2i64" => "vec2i64",
            vec![(String::from("x"), ResolvedType::I64), (String::from("y"), ResolvedType::I64)],
            vec2i64.clone()
        );

        register_helper!(self, "vec4i64" => "vec4i64",
            vec![(String::from("x"), ResolvedType::I64), (String::from("y"), ResolvedType::I64),
                 (String::from("z"), ResolvedType::I64), (String::from("w"), ResolvedType::I64)],
            vec4i64.clone()
        );

        // === SIMD Arithmetic Operations ===
        macro_rules! register_simd_binop {
            ($name:expr, $vec_ty:expr) => {
                register_helper!(self, $name => $name,
                    vec![(String::from("a"), $vec_ty.clone()), (String::from("b"), $vec_ty.clone())],
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
            vec![(String::from("v"), vec4f32)], ResolvedType::F32);
        register_helper!(self, "simd_reduce_add_vec8f32" => "simd_reduce_add_vec8f32",
            vec![(String::from("v"), vec8f32)], ResolvedType::F32);
        register_helper!(self, "simd_reduce_add_vec2f64" => "simd_reduce_add_vec2f64",
            vec![(String::from("v"), vec2f64)], ResolvedType::F64);
        register_helper!(self, "simd_reduce_add_vec4f64" => "simd_reduce_add_vec4f64",
            vec![(String::from("v"), vec4f64)], ResolvedType::F64);
        register_helper!(self, "simd_reduce_add_vec4i32" => "simd_reduce_add_vec4i32",
            vec![(String::from("v"), vec4i32)], ResolvedType::I32);
        register_helper!(self, "simd_reduce_add_vec8i32" => "simd_reduce_add_vec8i32",
            vec![(String::from("v"), vec8i32)], ResolvedType::I32);
        register_helper!(self, "simd_reduce_add_vec2i64" => "simd_reduce_add_vec2i64",
            vec![(String::from("v"), vec2i64)], ResolvedType::I64);
        register_helper!(self, "simd_reduce_add_vec4i64" => "simd_reduce_add_vec4i64",
            vec![(String::from("v"), vec4i64)], ResolvedType::I64);
    }

    pub(super) fn register_gc_functions(&mut self) {
        // GC runtime functions
        register_extern!(self, "vais_gc_init", vec![], ResolvedType::I64);

        register_extern!(
            self,
            "vais_gc_alloc",
            vec![
                (String::from("size"), ResolvedType::I64),
                (String::from("type_id"), ResolvedType::I32),
            ],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "vais_gc_add_root",
            vec![(String::from("ptr"), ResolvedType::I64)],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "vais_gc_remove_root",
            vec![(String::from("ptr"), ResolvedType::I64)],
            ResolvedType::I64
        );

        register_extern!(self, "vais_gc_collect", vec![], ResolvedType::I64);

        register_extern!(self, "vais_gc_bytes_allocated", vec![], ResolvedType::I64);

        register_extern!(self, "vais_gc_objects_count", vec![], ResolvedType::I64);

        register_extern!(self, "vais_gc_collections", vec![], ResolvedType::I64);

        register_extern!(
            self,
            "vais_gc_set_threshold",
            vec![(String::from("threshold"), ResolvedType::I64)],
            ResolvedType::I64
        );

        register_extern!(self, "vais_gc_print_stats", vec![], ResolvedType::I64);
    }

    pub(super) fn register_system_functions(&mut self) {
        // Environment variable functions
        register_extern!(
            self,
            "getenv",
            vec![(String::from("name"), ResolvedType::Str)],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "setenv",
            vec![
                (String::from("name"), ResolvedType::Str),
                (String::from("value"), ResolvedType::Str),
                (String::from("overwrite"), ResolvedType::I32),
            ],
            ResolvedType::I32
        );

        register_extern!(
            self,
            "unsetenv",
            vec![(String::from("name"), ResolvedType::Str)],
            ResolvedType::I32
        );

        // Process execution functions
        register_extern!(
            self,
            "system",
            vec![(String::from("command"), ResolvedType::Str)],
            ResolvedType::I32
        );

        register_extern!(
            self,
            "popen",
            vec![
                (String::from("command"), ResolvedType::Str),
                (String::from("mode"), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "pclose",
            vec![(String::from("stream"), ResolvedType::I64)],
            ResolvedType::I32
        );

        // Exit
        register_extern!(
            self,
            "exit",
            vec![(String::from("status"), ResolvedType::I32)],
            ResolvedType::Unit
        );

        // Signal handling
        register_extern!(
            self,
            "signal",
            vec![
                (String::from("signum"), ResolvedType::I32),
                (String::from("handler"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "raise",
            vec![(String::from("signum"), ResolvedType::I32)],
            ResolvedType::I32
        );
    }
}
