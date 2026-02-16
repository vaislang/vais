//! Standard library utility functions (atoi, rand, string ops, etc.)

use super::*;

impl TypeChecker {
    pub(super) fn register_stdlib_builtins(&mut self) {
        // ===== Standard library utility functions =====

        // atoi: (s: str) -> i32 - string to integer
        self.functions.insert(
            "atoi".to_string(),
            FunctionSig {
                name: "atoi".to_string(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // atol: (s: str) -> i64 - string to long integer
        self.functions.insert(
            "atol".to_string(),
            FunctionSig {
                name: "atol".to_string(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // atof: (s: str) -> f64 - string to double
        self.functions.insert(
            "atof".to_string(),
            FunctionSig {
                name: "atof".to_string(),
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::F64,
                ..Default::default()
            },
        );

        // fgets_ptr: (buffer: i64, size: i64, stream: i64) -> i64 - fgets with raw pointer params
        self.functions.insert(
            "fgets_ptr".to_string(),
            FunctionSig {
                name: "fgets_ptr".to_string(),
                params: vec![
                    ("buffer".to_string(), ResolvedType::I64, false),
                    ("size".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // atol_ptr: (s: i64) -> i64 - atol with raw pointer param
        self.functions.insert(
            "atol_ptr".to_string(),
            FunctionSig {
                name: "atol_ptr".to_string(),
                params: vec![("s".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // atof_ptr: (s: i64) -> f64 - atof with raw pointer param
        self.functions.insert(
            "atof_ptr".to_string(),
            FunctionSig {
                name: "atof_ptr".to_string(),
                params: vec![("s".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::F64,
                ..Default::default()
            },
        );

        // labs: (x: i64) -> i64 - absolute value (long integer)
        self.functions.insert(
            "labs".to_string(),
            FunctionSig {
                name: "labs".to_string(),
                params: vec![("x".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fabs: (x: f64) -> f64 - absolute value (double)
        self.functions.insert(
            "fabs".to_string(),
            FunctionSig {
                name: "fabs".to_string(),
                params: vec![("x".to_string(), ResolvedType::F64, false)],
                ret: ResolvedType::F64,
                ..Default::default()
            },
        );

        // sqrt: (x: f64) -> f64 - square root
        self.functions.insert(
            "sqrt".to_string(),
            FunctionSig {
                name: "sqrt".to_string(),
                params: vec![("x".to_string(), ResolvedType::F64, false)],
                ret: ResolvedType::F64,
                ..Default::default()
            },
        );

        // rand: () -> i32 - pseudo-random number
        self.functions.insert(
            "rand".to_string(),
            FunctionSig {
                name: "rand".to_string(),
                params: vec![],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // srand: (seed: i32) -> void - seed random number generator
        self.functions.insert(
            "srand".to_string(),
            FunctionSig {
                name: "srand".to_string(),
                params: vec![("seed".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::Unit,
                ..Default::default()
            },
        );

        // isdigit: (c: i32) -> i32 - test if digit
        self.functions.insert(
            "isdigit".to_string(),
            FunctionSig {
                name: "isdigit".to_string(),
                params: vec![("c".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // isalpha: (c: i32) -> i32 - test if alphabetic
        self.functions.insert(
            "isalpha".to_string(),
            FunctionSig {
                name: "isalpha".to_string(),
                params: vec![("c".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // toupper: (c: i32) -> i32 - convert to uppercase
        self.functions.insert(
            "toupper".to_string(),
            FunctionSig {
                name: "toupper".to_string(),
                params: vec![("c".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // tolower: (c: i32) -> i32 - convert to lowercase
        self.functions.insert(
            "tolower".to_string(),
            FunctionSig {
                name: "tolower".to_string(),
                params: vec![("c".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // strcpy: (dest: i64, src: str) -> i64 - copy string
        self.functions.insert(
            "strcpy".to_string(),
            FunctionSig {
                name: "strcpy".to_string(),
                params: vec![
                    ("dest".to_string(), ResolvedType::I64, false),
                    ("src".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // strcat: (dest: i64, src: str) -> i64 - concatenate string
        self.functions.insert(
            "strcat".to_string(),
            FunctionSig {
                name: "strcat".to_string(),
                params: vec![
                    ("dest".to_string(), ResolvedType::I64, false),
                    ("src".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );
    }
}
