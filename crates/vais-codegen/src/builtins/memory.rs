use super::*;

impl CodeGenerator {
    pub(super) fn register_memory_functions(&mut self) {
        // malloc: (i64) -> i64 (pointer as integer)
        register_extern!(
            self,
            "malloc",
            vec![(String::from("size"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // free: (i64) -> void
        // Note: declare uses i64 param, but call sites special-case to i8*.
        // The mismatch is handled by generate_expr_call.rs which converts
        // the i64 arg to i8* via inttoptr before calling @free.
        register_extern!(
            self,
            "free",
            vec![(String::from("ptr"), ResolvedType::I64)],
            ResolvedType::Unit
        );

        // memcpy: (dest, src, n) -> dest
        register_extern!(
            self,
            "memcpy",
            vec![
                (String::from("dest"), ResolvedType::I64),
                (String::from("src"), ResolvedType::I64),
                (String::from("n"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // memcmp: (s1, s2, n) -> i64 (compare memory)
        register_extern!(
            self,
            "memcmp",
            vec![
                (String::from("s1"), ResolvedType::I64),
                (String::from("s2"), ResolvedType::I64),
                (String::from("n"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // load_byte: load single byte from memory (internal helper)
        register_helper!(self, "load_byte" => "__load_byte",
            vec![(String::from("ptr"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_byte: store single byte to memory (internal helper)
        register_helper!(self, "store_byte" => "__store_byte",
            vec![
                (String::from("ptr"), ResolvedType::I64),
                (String::from("val"), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_i64: load 64-bit integer from memory (internal helper)
        register_helper!(self, "load_i64" => "__load_i64",
            vec![(String::from("ptr"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_i64: store 64-bit integer to memory (internal helper)
        register_helper!(self, "store_i64" => "__store_i64",
            vec![
                (String::from("ptr"), ResolvedType::I64),
                (String::from("val"), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // swap: swap two i64 elements in array (internal helper)
        register_helper!(self, "swap" => "__swap",
            vec![
                (String::from("ptr"), ResolvedType::I64),
                (String::from("idx1"), ResolvedType::I64),
                (String::from("idx2"), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_f64: load 64-bit float from memory (internal helper)
        register_helper!(self, "load_f64" => "__load_f64",
            vec![(String::from("ptr"), ResolvedType::I64)],
            ResolvedType::F64
        );

        // store_f64: store 64-bit float to memory (internal helper)
        register_helper!(self, "store_f64" => "__store_f64",
            vec![
                (String::from("ptr"), ResolvedType::I64),
                (String::from("val"), ResolvedType::F64),
            ],
            ResolvedType::Unit
        );

        // load_i8: load 8-bit integer from memory (internal helper)
        register_helper!(self, "load_i8" => "__load_i8",
            vec![(String::from("ptr"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_i8: store 8-bit integer to memory (internal helper)
        register_helper!(self, "store_i8" => "__store_i8",
            vec![
                (String::from("ptr"), ResolvedType::I64),
                (String::from("val"), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_i16: load 16-bit integer from memory (internal helper)
        register_helper!(self, "load_i16" => "__load_i16",
            vec![(String::from("ptr"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_i16: store 16-bit integer to memory (internal helper)
        register_helper!(self, "store_i16" => "__store_i16",
            vec![
                (String::from("ptr"), ResolvedType::I64),
                (String::from("val"), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_i32: load 32-bit integer from memory (internal helper)
        register_helper!(self, "load_i32" => "__load_i32",
            vec![(String::from("ptr"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_i32: store 32-bit integer to memory (internal helper)
        register_helper!(self, "store_i32" => "__store_i32",
            vec![
                (String::from("ptr"), ResolvedType::I64),
                (String::from("val"), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_f32: load 32-bit float from memory (internal helper)
        register_helper!(self, "load_f32" => "__load_f32",
            vec![(String::from("ptr"), ResolvedType::I64)],
            ResolvedType::F64
        );

        // store_f32: store 32-bit float to memory (internal helper)
        register_helper!(self, "store_f32" => "__store_f32",
            vec![
                (String::from("ptr"), ResolvedType::I64),
                (String::from("val"), ResolvedType::F64),
            ],
            ResolvedType::Unit
        );

        // str_to_ptr: convert str (i8*) to i64 — IR is special-cased in generate_expr
        register_builtin!(
            self,
            "str_to_ptr",
            vec![(String::from("s"), ResolvedType::Str, false)],
            ResolvedType::I64
        );

        // ptr_to_str: convert i64 to str (i8*) — IR is special-cased in generate_expr
        register_builtin!(
            self,
            "ptr_to_str",
            vec![(String::from("ptr"), ResolvedType::I64, false)],
            ResolvedType::Str
        );
    }

    pub(super) fn register_string_functions(&mut self) {
        // strlen: (s) -> len (accepts str)
        register_extern!(
            self,
            "strlen",
            vec![(String::from("s"), ResolvedType::Str)],
            ResolvedType::I64
        );

        // strcmp: (s1, s2) -> int
        register_extern!(
            self,
            "strcmp",
            vec![
                (String::from("s1"), ResolvedType::Str),
                (String::from("s2"), ResolvedType::Str),
            ],
            ResolvedType::I32
        );

        // strncmp: (s1, s2, n) -> int
        register_extern!(
            self,
            "strncmp",
            vec![
                (String::from("s1"), ResolvedType::Str),
                (String::from("s2"), ResolvedType::Str),
                (String::from("n"), ResolvedType::I64),
            ],
            ResolvedType::I32
        );

        // memcpy_str: (dest: str, src: str, len: i64) -> str
        // Copies len bytes from str src to str dest pointer
        register_extern!(self, "memcpy_str" => "memcpy",
            vec![
                (String::from("dest"), ResolvedType::Str),
                (String::from("src"), ResolvedType::Str),
                (String::from("len"), ResolvedType::I64),
            ],
            ResolvedType::Str
        );
    }
}
