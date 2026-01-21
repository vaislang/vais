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
}
