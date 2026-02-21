//! Built-in function registration for Vais code generator
//!
//! Contains definitions for external C functions and helper functions.

use crate::{CodeGenerator, FunctionInfo};
use std::collections::HashMap;
use vais_types::{EffectAnnotation, FunctionSig, ResolvedType};

/// Convert simple params (name, type) to full params (name, type, is_mut=false)
fn convert_params(params: Vec<(String, ResolvedType)>) -> Vec<(String, ResolvedType, bool)> {
    params.into_iter().map(|(n, t)| (n, t, false)).collect()
}

/// Macro for registering extern functions with less boilerplate
macro_rules! register_extern {
    ($gen:expr, $name:expr, $params:expr, $ret:expr) => {
        $gen.types.functions.insert(
            String::from($name),
            FunctionInfo {
                signature: FunctionSig {
                    name: String::from($name),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: convert_params($params),
                    ret: $ret,
                    is_async: false,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                    hkt_params: HashMap::new(),
                },
                is_extern: true,
                _extern_abi: Some(String::from("C")),
            },
        );
    };
    ($gen:expr, $key:expr => $name:expr, $params:expr, $ret:expr) => {
        $gen.types.functions.insert(
            String::from($key),
            FunctionInfo {
                signature: FunctionSig {
                    name: String::from($name),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: convert_params($params),
                    ret: $ret,
                    is_async: false,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                    hkt_params: HashMap::new(),
                },
                is_extern: true,
                _extern_abi: Some(String::from("C")),
            },
        );
    };
}

/// Macro for registering internal helper functions
macro_rules! register_helper {
    ($gen:expr, $key:expr => $name:expr, $params:expr, $ret:expr) => {
        $gen.types.functions.insert(
            String::from($key),
            FunctionInfo {
                signature: FunctionSig {
                    name: String::from($name),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: convert_params($params),
                    ret: $ret,
                    is_async: false,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                    hkt_params: HashMap::new(),
                },
                is_extern: false,
                _extern_abi: None,
            },
        );
    };
}

/// Macro for registering variadic functions (is_vararg = true, required_params = param count)
macro_rules! register_vararg {
    ($gen:expr, $name:expr, $params:expr, $ret:expr, extern) => {{
        let p: Vec<(String, ResolvedType)> = $params;
        let req = p.len();
        $gen.types.functions.insert(
            String::from($name),
            FunctionInfo {
                signature: FunctionSig {
                    name: String::from($name),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: convert_params(p),
                    ret: $ret,
                    is_async: false,
                    is_vararg: true,
                    required_params: Some(req),
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                    hkt_params: HashMap::new(),
                },
                is_extern: true,
                _extern_abi: Some(String::from("C")),
            },
        );
    }};
    ($gen:expr, $name:expr, $params:expr, $ret:expr) => {{
        let p: Vec<(String, ResolvedType)> = $params;
        let req = p.len();
        $gen.types.functions.insert(
            String::from($name),
            FunctionInfo {
                signature: FunctionSig {
                    name: String::from($name),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: convert_params(p),
                    ret: $ret,
                    is_async: false,
                    is_vararg: true,
                    required_params: Some(req),
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                    hkt_params: HashMap::new(),
                },
                is_extern: false,
                _extern_abi: None,
            },
        );
    }};
}

/// Macro for registering non-extern builtin functions (params already include mutability)
macro_rules! register_builtin {
    ($gen:expr, $name:expr, $params:expr, $ret:expr) => {
        $gen.types.functions.insert(
            String::from($name),
            FunctionInfo {
                signature: FunctionSig {
                    name: String::from($name),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: $params,
                    ret: $ret,
                    is_async: false,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                    hkt_params: HashMap::new(),
                },
                is_extern: false,
                _extern_abi: None,
            },
        );
    };
}

mod file_io;
mod io;
mod memory;
mod platform;

impl CodeGenerator {
    /// Register all built-in external and helper functions
    pub(crate) fn register_builtin_functions(&mut self) {
        self.register_io_functions();
        self.register_memory_functions();
        self.register_file_functions();
        self.register_string_functions();
        self.register_stdlib_functions();
        self.register_async_functions();
        self.register_simd_functions();
        self.register_gc_functions();
        self.register_system_functions();
    }
}
