//! Math built-in functions

use super::*;

impl TypeChecker {
    pub(super) fn register_math_builtins(&mut self) {
        // ===== Math functions =====
        let math_f64_fns = vec![
            "sin", "cos", "tan", "asin", "acos", "atan", "exp", "log", "log2", "log10", "floor",
            "ceil", "round", "abs",
        ];
        for name in math_f64_fns {
            self.functions.insert(
                name.to_string(),
                FunctionSig {
                    name: name.to_string(),
                    params: vec![("x".to_string(), ResolvedType::F64, false)],
                    ret: ResolvedType::F64,
                    ..Default::default()
                },
            );
        }

        let math_f64_2_fns = vec!["pow", "atan2", "min", "max"];
        for name in math_f64_2_fns {
            self.functions.insert(
                name.to_string(),
                FunctionSig {
                    name: name.to_string(),
                    params: vec![
                        ("a".to_string(), ResolvedType::F64, false),
                        ("b".to_string(), ResolvedType::F64, false),
                    ],
                    ret: ResolvedType::F64,
                    ..Default::default()
                },
            );
        }

        // Integer math functions
        let math_i64_fns = vec!["abs_i64"];
        for name in math_i64_fns {
            self.functions.insert(
                name.to_string(),
                FunctionSig {
                    name: name.to_string(),
                    params: vec![("x".to_string(), ResolvedType::I64, false)],
                    ret: ResolvedType::I64,
                    ..Default::default()
                },
            );
        }

        let math_i64_2_fns = vec!["min_i64", "max_i64"];
        for name in math_i64_2_fns {
            self.functions.insert(
                name.to_string(),
                FunctionSig {
                    name: name.to_string(),
                    params: vec![
                        ("a".to_string(), ResolvedType::I64, false),
                        ("b".to_string(), ResolvedType::I64, false),
                    ],
                    ret: ResolvedType::I64,
                    ..Default::default()
                },
            );
        }

        let math_clamp_fns = vec![
            ("clamp", ResolvedType::F64),
            ("clamp_i64", ResolvedType::I64),
        ];
        for (name, ty) in math_clamp_fns {
            self.functions.insert(
                name.to_string(),
                FunctionSig {
                    name: name.to_string(),
                    params: vec![
                        ("x".to_string(), ty.clone(), false),
                        ("min".to_string(), ty.clone(), false),
                        ("max".to_string(), ty.clone(), false),
                    ],
                    ret: ty,
                    ..Default::default()
                },
            );
        }
    }
}
