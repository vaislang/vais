//! System functions (environment, process, signal)

use std::collections::HashMap;

use super::*;

impl TypeChecker {
    pub(super) fn register_system_builtins(&mut self) {
        // ===== System functions (env/process/signal) =====
        #[allow(clippy::type_complexity)]
        let sys_fns: Vec<(&str, Vec<(String, ResolvedType, bool)>, ResolvedType)> = vec![
            (
                "getenv",
                vec![("name".to_string(), ResolvedType::Str, false)],
                ResolvedType::I64,
            ),
            (
                "setenv",
                vec![
                    ("name".to_string(), ResolvedType::Str, false),
                    ("value".to_string(), ResolvedType::Str, false),
                    ("overwrite".to_string(), ResolvedType::I32, false),
                ],
                ResolvedType::I32,
            ),
            (
                "unsetenv",
                vec![("name".to_string(), ResolvedType::Str, false)],
                ResolvedType::I32,
            ),
            (
                "system",
                vec![("command".to_string(), ResolvedType::Str, false)],
                ResolvedType::I32,
            ),
            (
                "popen",
                vec![
                    ("command".to_string(), ResolvedType::Str, false),
                    ("mode".to_string(), ResolvedType::Str, false),
                ],
                ResolvedType::I64,
            ),
            (
                "pclose",
                vec![("stream".to_string(), ResolvedType::I64, false)],
                ResolvedType::I32,
            ),
            (
                "exit",
                vec![("status".to_string(), ResolvedType::I32, false)],
                ResolvedType::Unit,
            ),
            (
                "signal",
                vec![
                    ("signum".to_string(), ResolvedType::I32, false),
                    ("handler".to_string(), ResolvedType::I64, false),
                ],
                ResolvedType::I64,
            ),
            (
                "raise",
                vec![("signum".to_string(), ResolvedType::I32, false)],
                ResolvedType::I32,
            ),
        ];
        for (name, params, ret) in sys_fns {
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
