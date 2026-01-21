//! Built-in function registration for Vais code generator
//!
//! Contains definitions for external C functions and helper functions.

use crate::{FunctionInfo, CodeGenerator};
use vais_types::ResolvedType;

/// Macro for registering extern functions with less boilerplate
macro_rules! register_extern {
    ($gen:expr, $name:expr, $params:expr, $ret:expr) => {
        $gen.functions.insert(
            $name.to_string(),
            FunctionInfo {
                name: $name.to_string(),
                params: $params,
                ret_type: $ret,
                is_extern: true,
            },
        );
    };
    ($gen:expr, $key:expr => $name:expr, $params:expr, $ret:expr) => {
        $gen.functions.insert(
            $key.to_string(),
            FunctionInfo {
                name: $name.to_string(),
                params: $params,
                ret_type: $ret,
                is_extern: true,
            },
        );
    };
}

/// Macro for registering internal helper functions
macro_rules! register_helper {
    ($gen:expr, $key:expr => $name:expr, $params:expr, $ret:expr) => {
        $gen.functions.insert(
            $key.to_string(),
            FunctionInfo {
                name: $name.to_string(),
                params: $params,
                ret_type: $ret,
                is_extern: false,
            },
        );
    };
}

impl CodeGenerator {
    /// Register all built-in external and helper functions
    pub(crate) fn register_builtin_functions(&mut self) {
        self.register_io_functions();
        self.register_memory_functions();
        self.register_file_functions();
        self.register_string_functions();
        self.register_async_functions();
        self.register_simd_functions();
    }

    fn register_io_functions(&mut self) {
        // printf for printing
        register_extern!(self, "printf",
            vec![("format".to_string(), ResolvedType::Str)],
            ResolvedType::I32
        );

        // putchar for single character output
        register_extern!(self, "putchar",
            vec![("c".to_string(), ResolvedType::I32)],
            ResolvedType::I32
        );

        // puts for simple string output
        register_extern!(self, "puts",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::I32
        );

        // puts_ptr: print string from pointer (maps to C puts)
        register_extern!(self, "puts_ptr" => "puts",
            vec![("s".to_string(), ResolvedType::I64)],
            ResolvedType::I32
        );

        // exit: (i32) -> void (noreturn)
        register_extern!(self, "exit",
            vec![("code".to_string(), ResolvedType::I32)],
            ResolvedType::Unit
        );
    }

    fn register_memory_functions(&mut self) {
        // malloc: (i64) -> i64 (pointer as integer)
        register_extern!(self, "malloc",
            vec![("size".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // free: (i64) -> void
        register_extern!(self, "free",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::Unit
        );

        // memcpy: (dest, src, n) -> dest
        register_extern!(self, "memcpy",
            vec![
                ("dest".to_string(), ResolvedType::I64),
                ("src".to_string(), ResolvedType::I64),
                ("n".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // memcpy_str: same as memcpy but accepts str as src
        register_extern!(self, "memcpy_str" => "memcpy",
            vec![
                ("dest".to_string(), ResolvedType::I64),
                ("src".to_string(), ResolvedType::Str),
                ("n".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // load_byte: load single byte from memory (internal helper)
        register_helper!(self, "load_byte" => "__load_byte",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_byte: store single byte to memory (internal helper)
        register_helper!(self, "store_byte" => "__store_byte",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_i64: load 64-bit integer from memory (internal helper)
        register_helper!(self, "load_i64" => "__load_i64",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_i64: store 64-bit integer to memory (internal helper)
        register_helper!(self, "store_i64" => "__store_i64",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );
    }

    fn register_file_functions(&mut self) {
        // fopen: (path, mode) -> FILE*
        register_extern!(self, "fopen",
            vec![
                ("path".to_string(), ResolvedType::Str),
                ("mode".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // fclose: (FILE*) -> int
        register_extern!(self, "fclose",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I32
        );

        // fread: (ptr, size, count, FILE*) -> size_t
        register_extern!(self, "fread",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("size".to_string(), ResolvedType::I64),
                ("count".to_string(), ResolvedType::I64),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fwrite: (ptr, size, count, FILE*) -> size_t
        register_extern!(self, "fwrite",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("size".to_string(), ResolvedType::I64),
                ("count".to_string(), ResolvedType::I64),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fgetc: (FILE*) -> int
        register_extern!(self, "fgetc",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fputc: (char, FILE*) -> int
        register_extern!(self, "fputc",
            vec![
                ("c".to_string(), ResolvedType::I64),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fgets: (str, n, FILE*) -> char*
        register_extern!(self, "fgets",
            vec![
                ("str".to_string(), ResolvedType::I64),
                ("n".to_string(), ResolvedType::I64),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fputs: (str, FILE*) -> int
        register_extern!(self, "fputs",
            vec![
                ("str".to_string(), ResolvedType::Str),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fseek: (FILE*, offset, origin) -> int
        register_extern!(self, "fseek",
            vec![
                ("stream".to_string(), ResolvedType::I64),
                ("offset".to_string(), ResolvedType::I64),
                ("origin".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // ftell: (FILE*) -> long
        register_extern!(self, "ftell",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fflush: (FILE*) -> int
        register_extern!(self, "fflush",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // feof: (FILE*) -> int
        register_extern!(self, "feof",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );
    }

    fn register_string_functions(&mut self) {
        // strlen: (s) -> len (accepts str)
        register_extern!(self, "strlen",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // strcmp: (s1, s2) -> int
        register_extern!(self, "strcmp",
            vec![
                ("s1".to_string(), ResolvedType::Str),
                ("s2".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I32
        );

        // strncmp: (s1, s2, n) -> int
        register_extern!(self, "strncmp",
            vec![
                ("s1".to_string(), ResolvedType::Str),
                ("s2".to_string(), ResolvedType::Str),
                ("n".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I32
        );
    }

    fn register_async_functions(&mut self) {
        // usleep: microsecond sleep for cooperative scheduling
        register_extern!(self, "usleep",
            vec![("usec".to_string(), ResolvedType::I64)],
            ResolvedType::I32
        );

        // sched_yield: yield CPU to other processes
        register_extern!(self, "sched_yield",
            vec![],
            ResolvedType::I32
        );
    }

    fn register_simd_functions(&mut self) {
        // Helper to create vector types
        let vec2f32 = ResolvedType::Vector { element: Box::new(ResolvedType::F32), lanes: 2 };
        let vec4f32 = ResolvedType::Vector { element: Box::new(ResolvedType::F32), lanes: 4 };
        let vec8f32 = ResolvedType::Vector { element: Box::new(ResolvedType::F32), lanes: 8 };
        let vec2f64 = ResolvedType::Vector { element: Box::new(ResolvedType::F64), lanes: 2 };
        let vec4f64 = ResolvedType::Vector { element: Box::new(ResolvedType::F64), lanes: 4 };
        let vec4i32 = ResolvedType::Vector { element: Box::new(ResolvedType::I32), lanes: 4 };
        let vec8i32 = ResolvedType::Vector { element: Box::new(ResolvedType::I32), lanes: 8 };
        let vec2i64 = ResolvedType::Vector { element: Box::new(ResolvedType::I64), lanes: 2 };
        let vec4i64 = ResolvedType::Vector { element: Box::new(ResolvedType::I64), lanes: 4 };

        // === Vector Constructors ===
        self.functions.insert(
            "vec2f32".to_string(),
            FunctionInfo {
                name: "vec2f32".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::F32),
                    ("y".to_string(), ResolvedType::F32),
                ],
                ret_type: vec2f32.clone(),
                is_extern: false,
            },
        );

        self.functions.insert(
            "vec4f32".to_string(),
            FunctionInfo {
                name: "vec4f32".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::F32),
                    ("y".to_string(), ResolvedType::F32),
                    ("z".to_string(), ResolvedType::F32),
                    ("w".to_string(), ResolvedType::F32),
                ],
                ret_type: vec4f32.clone(),
                is_extern: false,
            },
        );

        self.functions.insert(
            "vec8f32".to_string(),
            FunctionInfo {
                name: "vec8f32".to_string(),
                params: vec![
                    ("a".to_string(), ResolvedType::F32),
                    ("b".to_string(), ResolvedType::F32),
                    ("c".to_string(), ResolvedType::F32),
                    ("d".to_string(), ResolvedType::F32),
                    ("e".to_string(), ResolvedType::F32),
                    ("f".to_string(), ResolvedType::F32),
                    ("g".to_string(), ResolvedType::F32),
                    ("h".to_string(), ResolvedType::F32),
                ],
                ret_type: vec8f32.clone(),
                is_extern: false,
            },
        );

        self.functions.insert(
            "vec2f64".to_string(),
            FunctionInfo {
                name: "vec2f64".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::F64),
                    ("y".to_string(), ResolvedType::F64),
                ],
                ret_type: vec2f64.clone(),
                is_extern: false,
            },
        );

        self.functions.insert(
            "vec4f64".to_string(),
            FunctionInfo {
                name: "vec4f64".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::F64),
                    ("y".to_string(), ResolvedType::F64),
                    ("z".to_string(), ResolvedType::F64),
                    ("w".to_string(), ResolvedType::F64),
                ],
                ret_type: vec4f64.clone(),
                is_extern: false,
            },
        );

        self.functions.insert(
            "vec4i32".to_string(),
            FunctionInfo {
                name: "vec4i32".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::I32),
                    ("y".to_string(), ResolvedType::I32),
                    ("z".to_string(), ResolvedType::I32),
                    ("w".to_string(), ResolvedType::I32),
                ],
                ret_type: vec4i32.clone(),
                is_extern: false,
            },
        );

        self.functions.insert(
            "vec8i32".to_string(),
            FunctionInfo {
                name: "vec8i32".to_string(),
                params: vec![
                    ("a".to_string(), ResolvedType::I32),
                    ("b".to_string(), ResolvedType::I32),
                    ("c".to_string(), ResolvedType::I32),
                    ("d".to_string(), ResolvedType::I32),
                    ("e".to_string(), ResolvedType::I32),
                    ("f".to_string(), ResolvedType::I32),
                    ("g".to_string(), ResolvedType::I32),
                    ("h".to_string(), ResolvedType::I32),
                ],
                ret_type: vec8i32.clone(),
                is_extern: false,
            },
        );

        self.functions.insert(
            "vec2i64".to_string(),
            FunctionInfo {
                name: "vec2i64".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::I64),
                    ("y".to_string(), ResolvedType::I64),
                ],
                ret_type: vec2i64.clone(),
                is_extern: false,
            },
        );

        self.functions.insert(
            "vec4i64".to_string(),
            FunctionInfo {
                name: "vec4i64".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::I64),
                    ("y".to_string(), ResolvedType::I64),
                    ("z".to_string(), ResolvedType::I64),
                    ("w".to_string(), ResolvedType::I64),
                ],
                ret_type: vec4i64.clone(),
                is_extern: false,
            },
        );

        // === SIMD Arithmetic Operations ===
        macro_rules! register_simd_binop {
            ($name:expr, $vec_ty:expr) => {
                self.functions.insert(
                    $name.to_string(),
                    FunctionInfo {
                        name: $name.to_string(),
                        params: vec![
                            ("a".to_string(), $vec_ty.clone()),
                            ("b".to_string(), $vec_ty.clone()),
                        ],
                        ret_type: $vec_ty.clone(),
                        is_extern: false,
                    },
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
        self.functions.insert(
            "simd_reduce_add_vec4f32".to_string(),
            FunctionInfo {
                name: "simd_reduce_add_vec4f32".to_string(),
                params: vec![("v".to_string(), vec4f32)],
                ret_type: ResolvedType::F32,
                is_extern: false,
            },
        );

        self.functions.insert(
            "simd_reduce_add_vec8f32".to_string(),
            FunctionInfo {
                name: "simd_reduce_add_vec8f32".to_string(),
                params: vec![("v".to_string(), vec8f32)],
                ret_type: ResolvedType::F32,
                is_extern: false,
            },
        );

        self.functions.insert(
            "simd_reduce_add_vec2f64".to_string(),
            FunctionInfo {
                name: "simd_reduce_add_vec2f64".to_string(),
                params: vec![("v".to_string(), vec2f64)],
                ret_type: ResolvedType::F64,
                is_extern: false,
            },
        );

        self.functions.insert(
            "simd_reduce_add_vec4f64".to_string(),
            FunctionInfo {
                name: "simd_reduce_add_vec4f64".to_string(),
                params: vec![("v".to_string(), vec4f64)],
                ret_type: ResolvedType::F64,
                is_extern: false,
            },
        );

        self.functions.insert(
            "simd_reduce_add_vec4i32".to_string(),
            FunctionInfo {
                name: "simd_reduce_add_vec4i32".to_string(),
                params: vec![("v".to_string(), vec4i32)],
                ret_type: ResolvedType::I32,
                is_extern: false,
            },
        );

        self.functions.insert(
            "simd_reduce_add_vec8i32".to_string(),
            FunctionInfo {
                name: "simd_reduce_add_vec8i32".to_string(),
                params: vec![("v".to_string(), vec8i32)],
                ret_type: ResolvedType::I32,
                is_extern: false,
            },
        );

        self.functions.insert(
            "simd_reduce_add_vec2i64".to_string(),
            FunctionInfo {
                name: "simd_reduce_add_vec2i64".to_string(),
                params: vec![("v".to_string(), vec2i64)],
                ret_type: ResolvedType::I64,
                is_extern: false,
            },
        );

        self.functions.insert(
            "simd_reduce_add_vec4i64".to_string(),
            FunctionInfo {
                name: "simd_reduce_add_vec4i64".to_string(),
                params: vec![("v".to_string(), vec4i64)],
                ret_type: ResolvedType::I64,
                is_extern: false,
            },
        );
    }
}
