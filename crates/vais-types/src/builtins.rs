//! Built-in function and type registration for the type checker.

use std::collections::HashMap;

use super::TypeChecker;
use crate::types::{EffectAnnotation, EnumDef, FunctionSig, ResolvedType, VariantFieldTypes};

impl TypeChecker {
    pub(crate) fn register_builtins(&mut self) {
        self.register_core_builtins();
        self.register_print_builtins();
        self.register_memory_builtins();
        self.register_stdlib_builtins();
        self.register_file_io_builtins();
        self.register_simd_builtins();
        self.register_helper_print_builtins();
        self.register_gc_builtins();
        self.register_system_builtins();
        self.register_io_builtins();
        self.register_math_builtins();
        self.register_enum_builtins();
    }

    fn register_core_builtins(&mut self) {
        // printf: (str, ...) -> i32
        self.functions.insert(
            "printf".to_string(),
            FunctionSig {
                name: "printf".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("format".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I32,
                is_async: false,
                is_vararg: true,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // puts: (str) -> i64
        self.functions.insert(
            "puts".to_string(),
            FunctionSig {
                name: "puts".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // putchar: (i32) -> i32
        self.functions.insert(
            "putchar".to_string(),
            FunctionSig {
                name: "putchar".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("c".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::I32,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // malloc: (size: i64) -> i64 (pointer as integer for simplicity)
        self.functions.insert(
            "malloc".to_string(),
            FunctionSig {
                name: "malloc".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("size".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // free: (ptr: i64) -> ()
        self.functions.insert(
            "free".to_string(),
            FunctionSig {
                name: "free".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("ptr".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::Unit,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // sizeof: (val: T) -> i64 — compile-time type size query
        self.functions.insert(
            "sizeof".to_string(),
            FunctionSig {
                name: "sizeof".to_string(),
                generics: vec!["T".to_string()],
                generic_bounds: HashMap::new(),
                params: vec![(
                    "val".to_string(),
                    ResolvedType::Generic("T".to_string()),
                    false,
                )],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // alignof: (val: T) -> i64 — compile-time type alignment query
        self.functions.insert(
            "alignof".to_string(),
            FunctionSig {
                name: "alignof".to_string(),
                generics: vec!["T".to_string()],
                generic_bounds: HashMap::new(),
                params: vec![(
                    "val".to_string(),
                    ResolvedType::Generic("T".to_string()),
                    false,
                )],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // type_size: () -> i64 — compile-time size of generic type T
        // Used in generic containers to get element size at monomorphization time
        self.functions.insert(
            "type_size".to_string(),
            FunctionSig {
                name: "type_size".to_string(),
                generics: vec!["T".to_string()],
                generic_bounds: HashMap::new(),
                params: vec![],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // load_typed: (ptr: i64) -> T — type-aware memory load
        // Dispatches to correct load instruction based on resolved type T
        self.functions.insert(
            "load_typed".to_string(),
            FunctionSig {
                name: "load_typed".to_string(),
                generics: vec!["T".to_string()],
                generic_bounds: HashMap::new(),
                params: vec![("ptr".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::Generic("T".to_string()),
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // store_typed: (ptr: i64, val: T) -> () — type-aware memory store
        // Dispatches to correct store instruction based on resolved type T
        self.functions.insert(
            "store_typed".to_string(),
            FunctionSig {
                name: "store_typed".to_string(),
                generics: vec!["T".to_string()],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    (
                        "val".to_string(),
                        ResolvedType::Generic("T".to_string()),
                        false,
                    ),
                ],
                ret: ResolvedType::Unit,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // exit: (code: i32) -> void (noreturn, but typed as Unit)
        self.functions.insert(
            "exit".to_string(),
            FunctionSig {
                name: "exit".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("code".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::Unit,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

    }

    fn register_print_builtins(&mut self) {
        // print: (format, ...) -> void - format string output (no newline)
        self.functions.insert(
            "print".to_string(),
            FunctionSig {
                name: "print".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("format".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::Unit,
                is_async: false,
                is_vararg: true,
                required_params: Some(1),
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // println: (format, ...) -> void - format string output (with newline)
        self.functions.insert(
            "println".to_string(),
            FunctionSig {
                name: "println".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("format".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::Unit,
                is_async: false,
                is_vararg: true,
                required_params: Some(1),
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // format: (format, ...) -> str - format string, returns allocated string
        self.functions.insert(
            "format".to_string(),
            FunctionSig {
                name: "format".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("format".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::Str,
                is_async: false,
                is_vararg: true,
                required_params: Some(1),
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

    }

    fn register_memory_builtins(&mut self) {
        // memcpy: (dest, src, n) -> i64
        self.functions.insert(
            "memcpy".to_string(),
            FunctionSig {
                name: "memcpy".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("dest".to_string(), ResolvedType::I64, false),
                    ("src".to_string(), ResolvedType::I64, false),
                    ("n".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // memcmp: (s1, s2, n) -> i64 (compare memory)
        self.functions.insert(
            "memcmp".to_string(),
            FunctionSig {
                name: "memcmp".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("s1".to_string(), ResolvedType::I64, false),
                    ("s2".to_string(), ResolvedType::I64, false),
                    ("n".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // memcpy_str: (dest, src_str, n) -> i64 (accepts str as src)
        self.functions.insert(
            "memcpy_str".to_string(),
            FunctionSig {
                name: "memcpy_str".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("dest".to_string(), ResolvedType::I64, false),
                    ("src".to_string(), ResolvedType::Str, false),
                    ("n".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // strlen: (s) -> i64 (accepts str)
        self.functions.insert(
            "strlen".to_string(),
            FunctionSig {
                name: "strlen".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // str_to_ptr: (s) -> i64 (convert str to raw pointer)
        self.functions.insert(
            "str_to_ptr".to_string(),
            FunctionSig {
                name: "str_to_ptr".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // ptr_to_str: (p) -> str (convert raw pointer to str)
        self.functions.insert(
            "ptr_to_str".to_string(),
            FunctionSig {
                name: "ptr_to_str".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("p".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::Str,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // puts_ptr: (s) -> i32
        self.functions.insert(
            "puts_ptr".to_string(),
            FunctionSig {
                name: "puts_ptr".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("s".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I32,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // load_byte: (ptr) -> i64
        self.functions.insert(
            "load_byte".to_string(),
            FunctionSig {
                name: "load_byte".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("ptr".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // store_byte: (ptr, val) -> ()
        self.functions.insert(
            "store_byte".to_string(),
            FunctionSig {
                name: "store_byte".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    ("val".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::Unit,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // load_i64: (ptr) -> i64
        self.functions.insert(
            "load_i64".to_string(),
            FunctionSig {
                name: "load_i64".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("ptr".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // store_i64: (ptr, val) -> ()
        self.functions.insert(
            "store_i64".to_string(),
            FunctionSig {
                name: "store_i64".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    ("val".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::Unit,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

    }

    fn register_stdlib_builtins(&mut self) {
        // ===== Standard library utility functions =====

        // atoi: (s: str) -> i32 - string to integer
        self.functions.insert(
            "atoi".to_string(),
            FunctionSig {
                name: "atoi".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I32,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // atol: (s: str) -> i64 - string to long integer
        self.functions.insert(
            "atol".to_string(),
            FunctionSig {
                name: "atol".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // atof: (s: str) -> f64 - string to double
        self.functions.insert(
            "atof".to_string(),
            FunctionSig {
                name: "atof".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::F64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fgets_ptr: (buffer: i64, size: i64, stream: i64) -> i64 - fgets with raw pointer params
        self.functions.insert(
            "fgets_ptr".to_string(),
            FunctionSig {
                name: "fgets_ptr".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("buffer".to_string(), ResolvedType::I64, false),
                    ("size".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // atol_ptr: (s: i64) -> i64 - atol with raw pointer param
        self.functions.insert(
            "atol_ptr".to_string(),
            FunctionSig {
                name: "atol_ptr".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("s".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // atof_ptr: (s: i64) -> f64 - atof with raw pointer param
        self.functions.insert(
            "atof_ptr".to_string(),
            FunctionSig {
                name: "atof_ptr".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("s".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::F64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // labs: (x: i64) -> i64 - absolute value (long integer)
        self.functions.insert(
            "labs".to_string(),
            FunctionSig {
                name: "labs".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("x".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fabs: (x: f64) -> f64 - absolute value (double)
        self.functions.insert(
            "fabs".to_string(),
            FunctionSig {
                name: "fabs".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("x".to_string(), ResolvedType::F64, false)],
                ret: ResolvedType::F64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // sqrt: (x: f64) -> f64 - square root
        self.functions.insert(
            "sqrt".to_string(),
            FunctionSig {
                name: "sqrt".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("x".to_string(), ResolvedType::F64, false)],
                ret: ResolvedType::F64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // rand: () -> i32 - pseudo-random number
        self.functions.insert(
            "rand".to_string(),
            FunctionSig {
                name: "rand".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![],
                ret: ResolvedType::I32,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // srand: (seed: i32) -> void - seed random number generator
        self.functions.insert(
            "srand".to_string(),
            FunctionSig {
                name: "srand".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("seed".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::Unit,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // isdigit: (c: i32) -> i32 - test if digit
        self.functions.insert(
            "isdigit".to_string(),
            FunctionSig {
                name: "isdigit".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("c".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::I32,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // isalpha: (c: i32) -> i32 - test if alphabetic
        self.functions.insert(
            "isalpha".to_string(),
            FunctionSig {
                name: "isalpha".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("c".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::I32,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // toupper: (c: i32) -> i32 - convert to uppercase
        self.functions.insert(
            "toupper".to_string(),
            FunctionSig {
                name: "toupper".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("c".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::I32,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // tolower: (c: i32) -> i32 - convert to lowercase
        self.functions.insert(
            "tolower".to_string(),
            FunctionSig {
                name: "tolower".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("c".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::I32,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // strcpy: (dest: i64, src: str) -> i64 - copy string
        self.functions.insert(
            "strcpy".to_string(),
            FunctionSig {
                name: "strcpy".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("dest".to_string(), ResolvedType::I64, false),
                    ("src".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // strcat: (dest: i64, src: str) -> i64 - concatenate string
        self.functions.insert(
            "strcat".to_string(),
            FunctionSig {
                name: "strcat".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("dest".to_string(), ResolvedType::I64, false),
                    ("src".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

    }

    fn register_file_io_builtins(&mut self) {
        // ===== File I/O functions =====

        // fopen: (path, mode) -> FILE* (as i64)
        self.functions.insert(
            "fopen".to_string(),
            FunctionSig {
                name: "fopen".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("path".to_string(), ResolvedType::Str, false),
                    ("mode".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fopen_ptr: same as fopen but accepts i64 pointer (for selfhost)
        self.functions.insert(
            "fopen_ptr".to_string(),
            FunctionSig {
                name: "fopen_ptr".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("path".to_string(), ResolvedType::I64, false),
                    ("mode".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fclose: (stream) -> i32
        self.functions.insert(
            "fclose".to_string(),
            FunctionSig {
                name: "fclose".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I32,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fread: (ptr, size, count, stream) -> i64
        self.functions.insert(
            "fread".to_string(),
            FunctionSig {
                name: "fread".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    ("size".to_string(), ResolvedType::I64, false),
                    ("count".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fwrite: (ptr, size, count, stream) -> i64
        self.functions.insert(
            "fwrite".to_string(),
            FunctionSig {
                name: "fwrite".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    ("size".to_string(), ResolvedType::I64, false),
                    ("count".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fgetc: (stream) -> i64 (returns -1 on EOF)
        self.functions.insert(
            "fgetc".to_string(),
            FunctionSig {
                name: "fgetc".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fputc: (c, stream) -> i64
        self.functions.insert(
            "fputc".to_string(),
            FunctionSig {
                name: "fputc".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("c".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fgets: (str, n, stream) -> i64 (char*)
        self.functions.insert(
            "fgets".to_string(),
            FunctionSig {
                name: "fgets".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("str".to_string(), ResolvedType::I64, false),
                    ("n".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fputs: (str, stream) -> i64
        self.functions.insert(
            "fputs".to_string(),
            FunctionSig {
                name: "fputs".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("str".to_string(), ResolvedType::Str, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fseek: (stream, offset, origin) -> i64
        self.functions.insert(
            "fseek".to_string(),
            FunctionSig {
                name: "fseek".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("stream".to_string(), ResolvedType::I64, false),
                    ("offset".to_string(), ResolvedType::I64, false),
                    ("origin".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // ftell: (stream) -> i64
        self.functions.insert(
            "ftell".to_string(),
            FunctionSig {
                name: "ftell".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fflush: (stream) -> i64
        self.functions.insert(
            "fflush".to_string(),
            FunctionSig {
                name: "fflush".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // feof: (stream) -> i64
        self.functions.insert(
            "feof".to_string(),
            FunctionSig {
                name: "feof".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fileno: (stream) -> i64
        self.functions.insert(
            "fileno".to_string(),
            FunctionSig {
                name: "fileno".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fsync: (fd) -> i64
        self.functions.insert(
            "fsync".to_string(),
            FunctionSig {
                name: "fsync".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("fd".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // fdatasync: (fd) -> i64
        self.functions.insert(
            "fdatasync".to_string(),
            FunctionSig {
                name: "fdatasync".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("fd".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // mmap: (addr, len, prot, flags, fd, offset) -> ptr
        self.functions.insert(
            "mmap".to_string(),
            FunctionSig {
                name: "mmap".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("addr".to_string(), ResolvedType::I64, false),
                    ("len".to_string(), ResolvedType::I64, false),
                    ("prot".to_string(), ResolvedType::I64, false),
                    ("flags".to_string(), ResolvedType::I64, false),
                    ("fd".to_string(), ResolvedType::I64, false),
                    ("offset".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // munmap: (addr, len) -> int
        self.functions.insert(
            "munmap".to_string(),
            FunctionSig {
                name: "munmap".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("addr".to_string(), ResolvedType::I64, false),
                    ("len".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // msync: (addr, len, flags) -> int
        self.functions.insert(
            "msync".to_string(),
            FunctionSig {
                name: "msync".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("addr".to_string(), ResolvedType::I64, false),
                    ("len".to_string(), ResolvedType::I64, false),
                    ("flags".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // madvise: (addr, len, advice) -> int
        self.functions.insert(
            "madvise".to_string(),
            FunctionSig {
                name: "madvise".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("addr".to_string(), ResolvedType::I64, false),
                    ("len".to_string(), ResolvedType::I64, false),
                    ("advice".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // posix_open: (path, flags, mode) -> fd
        self.functions.insert(
            "posix_open".to_string(),
            FunctionSig {
                name: "posix_open".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("path".to_string(), ResolvedType::Str, false),
                    ("flags".to_string(), ResolvedType::I64, false),
                    ("mode".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // posix_close: (fd) -> i64
        self.functions.insert(
            "posix_close".to_string(),
            FunctionSig {
                name: "posix_close".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("fd".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // remove: (path) -> i64
        self.functions.insert(
            "remove".to_string(),
            FunctionSig {
                name: "remove".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // flock: (fd, operation) -> i64 (advisory file locking)
        self.functions.insert(
            "flock".to_string(),
            FunctionSig {
                name: "flock".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("fd".to_string(), ResolvedType::I64, false),
                    ("operation".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // mkdir: (path, mode) -> i64 (0 on success, -1 on error)
        self.functions.insert(
            "mkdir".to_string(),
            FunctionSig {
                name: "mkdir".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("path".to_string(), ResolvedType::Str, false),
                    ("mode".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // rmdir: (path) -> i64 (0 on success, -1 on error)
        self.functions.insert(
            "rmdir".to_string(),
            FunctionSig {
                name: "rmdir".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // opendir: (path) -> i64 (DIR* as i64, 0 on error)
        self.functions.insert(
            "opendir".to_string(),
            FunctionSig {
                name: "opendir".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // readdir: (dirp) -> i64 (pointer to dirent name, 0 at end)
        self.functions.insert(
            "readdir".to_string(),
            FunctionSig {
                name: "readdir".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("dirp".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // closedir: (dirp) -> i64 (0 on success)
        self.functions.insert(
            "closedir".to_string(),
            FunctionSig {
                name: "closedir".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("dirp".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // rename_file: (old, new_path) -> i64 (0 on success)
        self.functions.insert(
            "rename_file".to_string(),
            FunctionSig {
                name: "rename_file".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("old".to_string(), ResolvedType::Str, false),
                    ("new_path".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // unlink: (path) -> i64 (0 on success)
        self.functions.insert(
            "unlink".to_string(),
            FunctionSig {
                name: "unlink".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // stat_size: (path) -> i64 (file size in bytes)
        self.functions.insert(
            "stat_size".to_string(),
            FunctionSig {
                name: "stat_size".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // stat_mtime: (path) -> i64 (modification time as unix timestamp)
        self.functions.insert(
            "stat_mtime".to_string(),
            FunctionSig {
                name: "stat_mtime".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // getcwd: (buf, size) -> i64 (pointer to buf on success, 0 on error)
        self.functions.insert(
            "getcwd".to_string(),
            FunctionSig {
                name: "getcwd".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("buf".to_string(), ResolvedType::I64, false),
                    ("size".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // chdir: (path) -> i64 (0 on success)
        self.functions.insert(
            "chdir".to_string(),
            FunctionSig {
                name: "chdir".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // access: (path, mode) -> i64 (0 on success, -1 on error)
        self.functions.insert(
            "access".to_string(),
            FunctionSig {
                name: "access".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("path".to_string(), ResolvedType::Str, false),
                    ("mode".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );
    }

    pub(crate) fn register_simd_builtins(&mut self) {
        // Helper to create vector types
        let vec2f32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F32),
            lanes: 2,
        };
        let vec4f32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F32),
            lanes: 4,
        };
        let vec8f32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F32),
            lanes: 8,
        };
        let vec2f64 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F64),
            lanes: 2,
        };
        let vec4f64 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F64),
            lanes: 4,
        };
        let vec4i32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::I32),
            lanes: 4,
        };
        let vec8i32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::I32),
            lanes: 8,
        };
        let vec2i64 = ResolvedType::Vector {
            element: Box::new(ResolvedType::I64),
            lanes: 2,
        };
        let vec4i64 = ResolvedType::Vector {
            element: Box::new(ResolvedType::I64),
            lanes: 4,
        };

        // === Vector Constructors ===
        // vec2f32(x, y) -> Vec2f32
        self.functions.insert(
            "vec2f32".to_string(),
            FunctionSig {
                name: "vec2f32".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("x".to_string(), ResolvedType::F32, false),
                    ("y".to_string(), ResolvedType::F32, false),
                ],
                ret: vec2f32.clone(),
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // vec4f32(x, y, z, w) -> Vec4f32
        self.functions.insert(
            "vec4f32".to_string(),
            FunctionSig {
                name: "vec4f32".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("x".to_string(), ResolvedType::F32, false),
                    ("y".to_string(), ResolvedType::F32, false),
                    ("z".to_string(), ResolvedType::F32, false),
                    ("w".to_string(), ResolvedType::F32, false),
                ],
                ret: vec4f32.clone(),
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // vec8f32(a, b, c, d, e, f, g, h) -> Vec8f32
        self.functions.insert(
            "vec8f32".to_string(),
            FunctionSig {
                name: "vec8f32".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("a".to_string(), ResolvedType::F32, false),
                    ("b".to_string(), ResolvedType::F32, false),
                    ("c".to_string(), ResolvedType::F32, false),
                    ("d".to_string(), ResolvedType::F32, false),
                    ("e".to_string(), ResolvedType::F32, false),
                    ("f".to_string(), ResolvedType::F32, false),
                    ("g".to_string(), ResolvedType::F32, false),
                    ("h".to_string(), ResolvedType::F32, false),
                ],
                ret: vec8f32.clone(),
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // vec2f64(x, y) -> Vec2f64
        self.functions.insert(
            "vec2f64".to_string(),
            FunctionSig {
                name: "vec2f64".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("x".to_string(), ResolvedType::F64, false),
                    ("y".to_string(), ResolvedType::F64, false),
                ],
                ret: vec2f64.clone(),
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // vec4f64(x, y, z, w) -> Vec4f64
        self.functions.insert(
            "vec4f64".to_string(),
            FunctionSig {
                name: "vec4f64".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("x".to_string(), ResolvedType::F64, false),
                    ("y".to_string(), ResolvedType::F64, false),
                    ("z".to_string(), ResolvedType::F64, false),
                    ("w".to_string(), ResolvedType::F64, false),
                ],
                ret: vec4f64.clone(),
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // vec4i32(x, y, z, w) -> Vec4i32
        self.functions.insert(
            "vec4i32".to_string(),
            FunctionSig {
                name: "vec4i32".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("x".to_string(), ResolvedType::I32, false),
                    ("y".to_string(), ResolvedType::I32, false),
                    ("z".to_string(), ResolvedType::I32, false),
                    ("w".to_string(), ResolvedType::I32, false),
                ],
                ret: vec4i32.clone(),
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // vec8i32(a, b, c, d, e, f, g, h) -> Vec8i32
        self.functions.insert(
            "vec8i32".to_string(),
            FunctionSig {
                name: "vec8i32".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("a".to_string(), ResolvedType::I32, false),
                    ("b".to_string(), ResolvedType::I32, false),
                    ("c".to_string(), ResolvedType::I32, false),
                    ("d".to_string(), ResolvedType::I32, false),
                    ("e".to_string(), ResolvedType::I32, false),
                    ("f".to_string(), ResolvedType::I32, false),
                    ("g".to_string(), ResolvedType::I32, false),
                    ("h".to_string(), ResolvedType::I32, false),
                ],
                ret: vec8i32.clone(),
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // vec2i64(x, y) -> Vec2i64
        self.functions.insert(
            "vec2i64".to_string(),
            FunctionSig {
                name: "vec2i64".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("x".to_string(), ResolvedType::I64, false),
                    ("y".to_string(), ResolvedType::I64, false),
                ],
                ret: vec2i64.clone(),
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // vec4i64(x, y, z, w) -> Vec4i64
        self.functions.insert(
            "vec4i64".to_string(),
            FunctionSig {
                name: "vec4i64".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![
                    ("x".to_string(), ResolvedType::I64, false),
                    ("y".to_string(), ResolvedType::I64, false),
                    ("z".to_string(), ResolvedType::I64, false),
                    ("w".to_string(), ResolvedType::I64, false),
                ],
                ret: vec4i64.clone(),
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // === SIMD Arithmetic Operations ===
        // Helper macro to register binary SIMD operations
        macro_rules! register_simd_binop {
            ($name:expr, $vec_ty:expr) => {
                self.functions.insert(
                    $name.to_string(),
                    FunctionSig {
                        name: $name.to_string(),
                        generics: vec![],
                        generic_bounds: HashMap::new(),
                        params: vec![
                            ("a".to_string(), $vec_ty.clone(), false),
                            ("b".to_string(), $vec_ty.clone(), false),
                        ],
                        ret: $vec_ty.clone(),
                        is_async: false,
                        is_vararg: false,
                        required_params: None,
                        contracts: None,
                        effect_annotation: EffectAnnotation::Infer,
                        inferred_effects: None,
                    },
                );
            };
        }

        // Vec4f32 operations
        register_simd_binop!("simd_add_vec4f32", vec4f32);
        register_simd_binop!("simd_sub_vec4f32", vec4f32);
        register_simd_binop!("simd_mul_vec4f32", vec4f32);
        register_simd_binop!("simd_div_vec4f32", vec4f32);

        // Vec8f32 operations
        register_simd_binop!("simd_add_vec8f32", vec8f32);
        register_simd_binop!("simd_sub_vec8f32", vec8f32);
        register_simd_binop!("simd_mul_vec8f32", vec8f32);
        register_simd_binop!("simd_div_vec8f32", vec8f32);

        // Vec2f64 operations
        register_simd_binop!("simd_add_vec2f64", vec2f64);
        register_simd_binop!("simd_sub_vec2f64", vec2f64);
        register_simd_binop!("simd_mul_vec2f64", vec2f64);
        register_simd_binop!("simd_div_vec2f64", vec2f64);

        // Vec4f64 operations
        register_simd_binop!("simd_add_vec4f64", vec4f64);
        register_simd_binop!("simd_sub_vec4f64", vec4f64);
        register_simd_binop!("simd_mul_vec4f64", vec4f64);
        register_simd_binop!("simd_div_vec4f64", vec4f64);

        // Vec4i32 operations
        register_simd_binop!("simd_add_vec4i32", vec4i32);
        register_simd_binop!("simd_sub_vec4i32", vec4i32);
        register_simd_binop!("simd_mul_vec4i32", vec4i32);

        // Vec8i32 operations
        register_simd_binop!("simd_add_vec8i32", vec8i32);
        register_simd_binop!("simd_sub_vec8i32", vec8i32);
        register_simd_binop!("simd_mul_vec8i32", vec8i32);

        // Vec2i64 operations
        register_simd_binop!("simd_add_vec2i64", vec2i64);
        register_simd_binop!("simd_sub_vec2i64", vec2i64);
        register_simd_binop!("simd_mul_vec2i64", vec2i64);

        // Vec4i64 operations
        register_simd_binop!("simd_add_vec4i64", vec4i64);
        register_simd_binop!("simd_sub_vec4i64", vec4i64);
        register_simd_binop!("simd_mul_vec4i64", vec4i64);

        // === Horizontal Reduction Operations ===
        // simd_reduce_add_vec4f32(v) -> f32
        self.functions.insert(
            "simd_reduce_add_vec4f32".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec4f32".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("v".to_string(), vec4f32.clone(), false)],
                ret: ResolvedType::F32,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // simd_reduce_add_vec8f32(v) -> f32
        self.functions.insert(
            "simd_reduce_add_vec8f32".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec8f32".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("v".to_string(), vec8f32, false)],
                ret: ResolvedType::F32,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // simd_reduce_add_vec2f64(v) -> f64
        self.functions.insert(
            "simd_reduce_add_vec2f64".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec2f64".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("v".to_string(), vec2f64, false)],
                ret: ResolvedType::F64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // simd_reduce_add_vec4f64(v) -> f64
        self.functions.insert(
            "simd_reduce_add_vec4f64".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec4f64".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("v".to_string(), vec4f64, false)],
                ret: ResolvedType::F64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // simd_reduce_add_vec4i32(v) -> i32
        self.functions.insert(
            "simd_reduce_add_vec4i32".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec4i32".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("v".to_string(), vec4i32, false)],
                ret: ResolvedType::I32,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // simd_reduce_add_vec8i32(v) -> i32
        self.functions.insert(
            "simd_reduce_add_vec8i32".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec8i32".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("v".to_string(), vec8i32, false)],
                ret: ResolvedType::I32,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // simd_reduce_add_vec2i64(v) -> i64
        self.functions.insert(
            "simd_reduce_add_vec2i64".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec2i64".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("v".to_string(), vec2i64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // simd_reduce_add_vec4i64(v) -> i64
        self.functions.insert(
            "simd_reduce_add_vec4i64".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec4i64".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("v".to_string(), vec4i64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );
    }

    fn register_helper_print_builtins(&mut self) {
        // ===== Helper print functions used by examples =====

        // print_i64: (n: i64) -> i64
        self.functions.insert(
            "print_i64".to_string(),
            FunctionSig {
                name: "print_i64".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("n".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // print_f64: (n: f64) -> i64
        self.functions.insert(
            "print_f64".to_string(),
            FunctionSig {
                name: "print_f64".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("n".to_string(), ResolvedType::F64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // puts_str: (s: str) -> i64
        self.functions.insert(
            "puts_str".to_string(),
            FunctionSig {
                name: "puts_str".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );

        // println_i64: (n: i64) -> i64
        self.functions.insert(
            "println_i64".to_string(),
            FunctionSig {
                name: "println_i64".to_string(),
                generics: vec![],
                generic_bounds: HashMap::new(),
                params: vec![("n".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
                is_vararg: false,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
            },
        );
    }

    fn register_gc_builtins(&mut self) {
        // ===== GC functions used by gc examples =====
        let gc_fns = vec![
            ("gc_init", vec![], ResolvedType::I64),
            (
                "gc_alloc",
                vec![
                    ("size".to_string(), ResolvedType::I64, false),
                    ("type_id".to_string(), ResolvedType::I64, false),
                ],
                ResolvedType::I64,
            ),
            ("gc_collect", vec![], ResolvedType::I64),
            (
                "gc_add_root",
                vec![("ptr".to_string(), ResolvedType::I64, false)],
                ResolvedType::I64,
            ),
            (
                "gc_remove_root",
                vec![("ptr".to_string(), ResolvedType::I64, false)],
                ResolvedType::I64,
            ),
            ("gc_bytes_allocated", vec![], ResolvedType::I64),
            ("gc_objects_count", vec![], ResolvedType::I64),
            ("gc_collections", vec![], ResolvedType::I64),
            (
                "gc_set_threshold",
                vec![("threshold".to_string(), ResolvedType::I64, false)],
                ResolvedType::I64,
            ),
            ("gc_print_stats", vec![], ResolvedType::I64),
        ];
        for (name, params, ret) in gc_fns {
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
                },
            );
        }
    }

    fn register_system_builtins(&mut self) {
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
                },
            );
        }
    }

    fn register_io_builtins(&mut self) {
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
                },
            );
        }
    }

    fn register_math_builtins(&mut self) {
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
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: vec![("x".to_string(), ResolvedType::F64, false)],
                    ret: ResolvedType::F64,
                    is_async: false,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                },
            );
        }

        let math_f64_2_fns = vec!["pow", "atan2", "min", "max"];
        for name in math_f64_2_fns {
            self.functions.insert(
                name.to_string(),
                FunctionSig {
                    name: name.to_string(),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: vec![
                        ("a".to_string(), ResolvedType::F64, false),
                        ("b".to_string(), ResolvedType::F64, false),
                    ],
                    ret: ResolvedType::F64,
                    is_async: false,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
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
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: vec![("x".to_string(), ResolvedType::I64, false)],
                    ret: ResolvedType::I64,
                    is_async: false,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                },
            );
        }

        let math_i64_2_fns = vec!["min_i64", "max_i64"];
        for name in math_i64_2_fns {
            self.functions.insert(
                name.to_string(),
                FunctionSig {
                    name: name.to_string(),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: vec![
                        ("a".to_string(), ResolvedType::I64, false),
                        ("b".to_string(), ResolvedType::I64, false),
                    ],
                    ret: ResolvedType::I64,
                    is_async: false,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
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
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: vec![
                        ("x".to_string(), ty.clone(), false),
                        ("min".to_string(), ty.clone(), false),
                        ("max".to_string(), ty.clone(), false),
                    ],
                    ret: ty,
                    is_async: false,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                },
            );
        }
    }

    fn register_enum_builtins(&mut self) {
        // Register built-in Result<T, E> enum
        {
            let mut variants = HashMap::new();
            variants.insert(
                "Ok".to_string(),
                VariantFieldTypes::Tuple(vec![ResolvedType::Generic("T".to_string())]),
            );
            variants.insert(
                "Err".to_string(),
                VariantFieldTypes::Tuple(vec![ResolvedType::Generic("E".to_string())]),
            );
            self.enums.insert(
                "Result".to_string(),
                EnumDef {
                    name: "Result".to_string(),
                    generics: vec!["T".to_string(), "E".to_string()],
                    variants,
                    methods: HashMap::new(),
                },
            );
            self.exhaustiveness_checker
                .register_enum("Result", vec!["Ok".to_string(), "Err".to_string()]);
        }

        // Register built-in Option<T> enum
        if !self.enums.contains_key("Option") {
            let mut variants = HashMap::new();
            variants.insert("None".to_string(), VariantFieldTypes::Unit);
            variants.insert(
                "Some".to_string(),
                VariantFieldTypes::Tuple(vec![ResolvedType::Generic("T".to_string())]),
            );
            self.enums.insert(
                "Option".to_string(),
                EnumDef {
                    name: "Option".to_string(),
                    generics: vec!["T".to_string()],
                    variants,
                    methods: HashMap::new(),
                },
            );
            self.exhaustiveness_checker
                .register_enum("Option", vec!["None".to_string(), "Some".to_string()]);
        }
    }
}
