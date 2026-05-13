//! Print and formatting built-in functions

use super::*;

impl TypeChecker {
    pub(super) fn register_print_builtins(&mut self) {
        // print: (format, ...) -> void - format string output (no newline)
        self.functions.insert(
            "print".to_string(),
            FunctionSig {
                name: "print".to_string(),
                params: vec![("format".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::Unit,
                is_vararg: true,
                required_params: Some(1),
                ..Default::default()
            },
        );

        // println: (format, ...) -> void - format string output (with newline)
        self.functions.insert(
            "println".to_string(),
            FunctionSig {
                name: "println".to_string(),
                params: vec![("format".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::Unit,
                is_vararg: true,
                required_params: Some(1),
                ..Default::default()
            },
        );

        // format: (format, ...) -> str - format string, returns allocated string
        self.functions.insert(
            "format".to_string(),
            FunctionSig {
                name: "format".to_string(),
                params: vec![("format".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::Str,
                is_vararg: true,
                required_params: Some(1),
                ..Default::default()
            },
        );
    }

    pub(super) fn register_helper_print_builtins(&mut self) {
        // ===== Helper print functions used by examples =====

        // print_i64: (n: i64) -> i64
        self.functions.insert(
            "print_i64".to_string(),
            FunctionSig {
                name: "print_i64".to_string(),
                params: vec![("n".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // print_f64: (n: f64) -> i64
        self.functions.insert(
            "print_f64".to_string(),
            FunctionSig {
                name: "print_f64".to_string(),
                params: vec![("n".to_string(), ResolvedType::F64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // puts_str: (s: str) -> i64
        self.functions.insert(
            "puts_str".to_string(),
            FunctionSig {
                name: "puts_str".to_string(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // println_i64: (n: i64) -> i64
        self.functions.insert(
            "println_i64".to_string(),
            FunctionSig {
                name: "println_i64".to_string(),
                params: vec![("n".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );
    }
}
