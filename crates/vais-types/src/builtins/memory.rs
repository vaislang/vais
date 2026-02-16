//! Memory manipulation built-in functions

use super::*;

impl TypeChecker {
    pub(super) fn register_memory_builtins(&mut self) {
        // memcpy: (dest, src, n) -> i64
        self.functions.insert(
            "memcpy".to_string(),
            FunctionSig {
                name: "memcpy".to_string(),
                params: vec![
                    ("dest".to_string(), ResolvedType::I64, false),
                    ("src".to_string(), ResolvedType::I64, false),
                    ("n".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // memcmp: (s1, s2, n) -> i64 (compare memory)
        self.functions.insert(
            "memcmp".to_string(),
            FunctionSig {
                name: "memcmp".to_string(),
                params: vec![
                    ("s1".to_string(), ResolvedType::I64, false),
                    ("s2".to_string(), ResolvedType::I64, false),
                    ("n".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // memcpy_str: (dest, src_str, n) -> i64 (accepts str as src)
        self.functions.insert(
            "memcpy_str".to_string(),
            FunctionSig {
                name: "memcpy_str".to_string(),
                params: vec![
                    ("dest".to_string(), ResolvedType::I64, false),
                    ("src".to_string(), ResolvedType::Str, false),
                    ("n".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // strlen: (s) -> i64 (accepts str)
        self.functions.insert(
            "strlen".to_string(),
            FunctionSig {
                name: "strlen".to_string(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // str_to_ptr: (s) -> i64 (convert str to raw pointer)
        self.functions.insert(
            "str_to_ptr".to_string(),
            FunctionSig {
                name: "str_to_ptr".to_string(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // ptr_to_str: (p) -> str (convert raw pointer to str)
        self.functions.insert(
            "ptr_to_str".to_string(),
            FunctionSig {
                name: "ptr_to_str".to_string(),
                params: vec![("p".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::Str,
                ..Default::default()
            },
        );

        // puts_ptr: (s) -> i32
        self.functions.insert(
            "puts_ptr".to_string(),
            FunctionSig {
                name: "puts_ptr".to_string(),
                params: vec![("s".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // load_byte: (ptr) -> i64
        self.functions.insert(
            "load_byte".to_string(),
            FunctionSig {
                name: "load_byte".to_string(),
                params: vec![("ptr".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // store_byte: (ptr, val) -> ()
        self.functions.insert(
            "store_byte".to_string(),
            FunctionSig {
                name: "store_byte".to_string(),
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    ("val".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::Unit,
                ..Default::default()
            },
        );

        // load_i64: (ptr) -> i64
        self.functions.insert(
            "load_i64".to_string(),
            FunctionSig {
                name: "load_i64".to_string(),
                params: vec![("ptr".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // store_i64: (ptr, val) -> ()
        self.functions.insert(
            "store_i64".to_string(),
            FunctionSig {
                name: "store_i64".to_string(),
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    ("val".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::Unit,
                ..Default::default()
            },
        );

        // swap: (ptr, idx1, idx2) -> () — swap two i64 elements in array
        self.functions.insert(
            "swap".to_string(),
            FunctionSig {
                name: "swap".to_string(),
                params: vec![
                    (
                        "ptr".to_string(),
                        ResolvedType::Pointer(Box::new(ResolvedType::I64)),
                        false,
                    ),
                    ("idx1".to_string(), ResolvedType::I64, false),
                    ("idx2".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::Unit,
                ..Default::default()
            },
        );
    }
}
