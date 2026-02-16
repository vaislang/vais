//! Core built-in functions (printf, malloc, sizeof, etc.)

use super::*;

impl TypeChecker {
    pub(super) fn register_core_builtins(&mut self) {
        // printf: (str, ...) -> i32
        self.functions.insert(
            "printf".to_string(),
            FunctionSig {
                name: "printf".to_string(),
                params: vec![("format".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I32,
                is_vararg: true,
                ..Default::default()
            },
        );

        // puts: (str) -> i64
        self.functions.insert(
            "puts".to_string(),
            FunctionSig {
                name: "puts".to_string(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // putchar: (i32) -> i32
        self.functions.insert(
            "putchar".to_string(),
            FunctionSig {
                name: "putchar".to_string(),
                params: vec![("c".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // malloc: (size: i64) -> i64 (pointer as integer for simplicity)
        self.functions.insert(
            "malloc".to_string(),
            FunctionSig {
                name: "malloc".to_string(),
                params: vec![("size".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // free: (ptr: i64) -> ()
        self.functions.insert(
            "free".to_string(),
            FunctionSig {
                name: "free".to_string(),
                params: vec![("ptr".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::Unit,
                ..Default::default()
            },
        );

        // sizeof: (val: T) -> i64 — compile-time type size query
        self.functions.insert(
            "sizeof".to_string(),
            FunctionSig {
                name: "sizeof".to_string(),
                generics: vec!["T".to_string()],
                params: vec![(
                    "val".to_string(),
                    ResolvedType::Generic("T".to_string()),
                    false,
                )],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // alignof: (val: T) -> i64 — compile-time type alignment query
        self.functions.insert(
            "alignof".to_string(),
            FunctionSig {
                name: "alignof".to_string(),
                generics: vec!["T".to_string()],
                params: vec![(
                    "val".to_string(),
                    ResolvedType::Generic("T".to_string()),
                    false,
                )],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // type_size: () -> i64 — compile-time size of generic type T
        // Used in generic containers to get element size at monomorphization time
        self.functions.insert(
            "type_size".to_string(),
            FunctionSig {
                name: "type_size".to_string(),
                generics: vec!["T".to_string()],
                params: vec![],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // load_typed: (ptr: i64) -> T — type-aware memory load
        // Dispatches to correct load instruction based on resolved type T
        self.functions.insert(
            "load_typed".to_string(),
            FunctionSig {
                name: "load_typed".to_string(),
                generics: vec!["T".to_string()],
                params: vec![("ptr".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::Generic("T".to_string()),
                ..Default::default()
            },
        );

        // store_typed: (ptr: i64, val: T) -> () — type-aware memory store
        // Dispatches to correct store instruction based on resolved type T
        self.functions.insert(
            "store_typed".to_string(),
            FunctionSig {
                name: "store_typed".to_string(),
                generics: vec!["T".to_string()],
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    (
                        "val".to_string(),
                        ResolvedType::Generic("T".to_string()),
                        false,
                    ),
                ],
                ret: ResolvedType::Unit,
                ..Default::default()
            },
        );

        // exit: (code: i32) -> void (noreturn, but typed as Unit)
        self.functions.insert(
            "exit".to_string(),
            FunctionSig {
                name: "exit".to_string(),
                params: vec![("code".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::Unit,
                ..Default::default()
            },
        );
    }
}
