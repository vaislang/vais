//! Input/output built-in functions

use std::collections::HashMap;

use super::*;

impl TypeChecker {
    pub(super) fn register_io_builtins(&mut self) {
        // ===== IO functions =====
        let io_fns = vec![
            ("read_i64", vec![], ResolvedType::I64),
            ("read_f64", vec![], ResolvedType::F64),
            (
                "prompt_i64",
                vec![("prompt".to_string(), ResolvedType::Str, false)],
                ResolvedType::I64,
            ),
            (
                "prompt_f64",
                vec![("prompt".to_string(), ResolvedType::Str, false)],
                ResolvedType::F64,
            ),
            (
                "fgets",
                vec![
                    ("buf".to_string(), ResolvedType::I64, false),
                    ("n".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ResolvedType::I64,
            ),
            ("get_stdin", vec![], ResolvedType::I64),
            ("get_stdout", vec![], ResolvedType::I64),
            ("get_stderr", vec![], ResolvedType::I64),
        ];
        for (name, params, ret) in io_fns {
            self.functions.insert(
                name.to_string(),
                FunctionSig {
                    name: name.to_string(),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params,
                    ret,
                    is_async: false,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                    hkt_params: HashMap::new(),
                    generic_callees: vec![],
                },
            );
        }
    }
}
