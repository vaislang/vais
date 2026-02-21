use super::*;

impl CodeGenerator {
    pub(super) fn register_memory_functions(&mut self) {
        // malloc: (i64) -> i64 (pointer as integer)
        register_extern!(
            self,
            "malloc",
            vec![("size".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // free: (i64) -> void
        register_extern!(
            self,
            "free",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::Unit
        );

        // memcpy: (dest, src, n) -> dest
        register_extern!(
            self,
            "memcpy",
            vec![
                ("dest".to_string(), ResolvedType::I64),
                ("src".to_string(), ResolvedType::I64),
                ("n".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // memcmp: (s1, s2, n) -> i64 (compare memory)
        register_extern!(
            self,
            "memcmp",
            vec![
                ("s1".to_string(), ResolvedType::I64),
                ("s2".to_string(), ResolvedType::I64),
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

        // swap: swap two i64 elements in array (internal helper)
        register_helper!(self, "swap" => "__swap",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("idx1".to_string(), ResolvedType::I64),
                ("idx2".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_f64: load 64-bit float from memory (internal helper)
        register_helper!(self, "load_f64" => "__load_f64",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::F64
        );

        // store_f64: store 64-bit float to memory (internal helper)
        register_helper!(self, "store_f64" => "__store_f64",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::F64),
            ],
            ResolvedType::Unit
        );

        // load_i8: load 8-bit integer from memory (internal helper)
        register_helper!(self, "load_i8" => "__load_i8",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_i8: store 8-bit integer to memory (internal helper)
        register_helper!(self, "store_i8" => "__store_i8",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_i16: load 16-bit integer from memory (internal helper)
        register_helper!(self, "load_i16" => "__load_i16",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_i16: store 16-bit integer to memory (internal helper)
        register_helper!(self, "store_i16" => "__store_i16",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_i32: load 32-bit integer from memory (internal helper)
        register_helper!(self, "load_i32" => "__load_i32",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_i32: store 32-bit integer to memory (internal helper)
        register_helper!(self, "store_i32" => "__store_i32",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_f32: load 32-bit float from memory (internal helper)
        register_helper!(self, "load_f32" => "__load_f32",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::F64
        );

        // store_f32: store 32-bit float to memory (internal helper)
        register_helper!(self, "store_f32" => "__store_f32",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::F64),
            ],
            ResolvedType::Unit
        );

        // str_to_ptr: convert str (i8*) to i64 — IR is special-cased in generate_expr
        register_builtin!(
            self,
            "str_to_ptr",
            vec![("s".to_string(), ResolvedType::Str, false)],
            ResolvedType::I64
        );

        // ptr_to_str: convert i64 to str (i8*) — IR is special-cased in generate_expr
        register_builtin!(
            self,
            "ptr_to_str",
            vec![("ptr".to_string(), ResolvedType::I64, false)],
            ResolvedType::Str
        );
    }

    pub(super) fn register_string_functions(&mut self) {
        // strlen: (s) -> len (accepts str)
        register_extern!(
            self,
            "strlen",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // strcmp: (s1, s2) -> int
        register_extern!(
            self,
            "strcmp",
            vec![
                ("s1".to_string(), ResolvedType::Str),
                ("s2".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I32
        );

        // strncmp: (s1, s2, n) -> int
        register_extern!(
            self,
            "strncmp",
            vec![
                ("s1".to_string(), ResolvedType::Str),
                ("s2".to_string(), ResolvedType::Str),
                ("n".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I32
        );

        // memcpy_str: (dest: str, src: str, len: i64) -> str
        // Copies len bytes from str src to str dest pointer
        register_extern!(self, "memcpy_str" => "memcpy",
            vec![
                ("dest".to_string(), ResolvedType::Str),
                ("src".to_string(), ResolvedType::Str),
                ("len".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Str
        );
    }
}
