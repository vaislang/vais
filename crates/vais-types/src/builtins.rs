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

    fn register_print_builtins(&mut self) {
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

    fn register_memory_builtins(&mut self) {
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

    fn register_stdlib_builtins(&mut self) {
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

    fn register_file_io_builtins(&mut self) {
        // ===== File I/O functions =====

        // fopen: (path, mode) -> FILE* (as i64)
        self.functions.insert(
            "fopen".to_string(),
            FunctionSig {
                name: "fopen".to_string(),
                params: vec![
                    ("path".to_string(), ResolvedType::Str, false),
                    ("mode".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fopen_ptr: same as fopen but accepts i64 pointer (for selfhost)
        self.functions.insert(
            "fopen_ptr".to_string(),
            FunctionSig {
                name: "fopen_ptr".to_string(),
                params: vec![
                    ("path".to_string(), ResolvedType::I64, false),
                    ("mode".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fclose: (stream) -> i32
        self.functions.insert(
            "fclose".to_string(),
            FunctionSig {
                name: "fclose".to_string(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // fread: (ptr, size, count, stream) -> i64
        self.functions.insert(
            "fread".to_string(),
            FunctionSig {
                name: "fread".to_string(),
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    ("size".to_string(), ResolvedType::I64, false),
                    ("count".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fwrite: (ptr, size, count, stream) -> i64
        self.functions.insert(
            "fwrite".to_string(),
            FunctionSig {
                name: "fwrite".to_string(),
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    ("size".to_string(), ResolvedType::I64, false),
                    ("count".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fgetc: (stream) -> i64 (returns -1 on EOF)
        self.functions.insert(
            "fgetc".to_string(),
            FunctionSig {
                name: "fgetc".to_string(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fputc: (c, stream) -> i64
        self.functions.insert(
            "fputc".to_string(),
            FunctionSig {
                name: "fputc".to_string(),
                params: vec![
                    ("c".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fgets: (str, n, stream) -> i64 (char*)
        self.functions.insert(
            "fgets".to_string(),
            FunctionSig {
                name: "fgets".to_string(),
                params: vec![
                    ("str".to_string(), ResolvedType::I64, false),
                    ("n".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fputs: (str, stream) -> i64
        self.functions.insert(
            "fputs".to_string(),
            FunctionSig {
                name: "fputs".to_string(),
                params: vec![
                    ("str".to_string(), ResolvedType::Str, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fseek: (stream, offset, origin) -> i64
        self.functions.insert(
            "fseek".to_string(),
            FunctionSig {
                name: "fseek".to_string(),
                params: vec![
                    ("stream".to_string(), ResolvedType::I64, false),
                    ("offset".to_string(), ResolvedType::I64, false),
                    ("origin".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // ftell: (stream) -> i64
        self.functions.insert(
            "ftell".to_string(),
            FunctionSig {
                name: "ftell".to_string(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fflush: (stream) -> i64
        self.functions.insert(
            "fflush".to_string(),
            FunctionSig {
                name: "fflush".to_string(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // feof: (stream) -> i64
        self.functions.insert(
            "feof".to_string(),
            FunctionSig {
                name: "feof".to_string(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fileno: (stream) -> i64
        self.functions.insert(
            "fileno".to_string(),
            FunctionSig {
                name: "fileno".to_string(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fsync: (fd) -> i64
        self.functions.insert(
            "fsync".to_string(),
            FunctionSig {
                name: "fsync".to_string(),
                params: vec![("fd".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fdatasync: (fd) -> i64
        self.functions.insert(
            "fdatasync".to_string(),
            FunctionSig {
                name: "fdatasync".to_string(),
                params: vec![("fd".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // mmap: (addr, len, prot, flags, fd, offset) -> ptr
        self.functions.insert(
            "mmap".to_string(),
            FunctionSig {
                name: "mmap".to_string(),
                params: vec![
                    ("addr".to_string(), ResolvedType::I64, false),
                    ("len".to_string(), ResolvedType::I64, false),
                    ("prot".to_string(), ResolvedType::I64, false),
                    ("flags".to_string(), ResolvedType::I64, false),
                    ("fd".to_string(), ResolvedType::I64, false),
                    ("offset".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // munmap: (addr, len) -> int
        self.functions.insert(
            "munmap".to_string(),
            FunctionSig {
                name: "munmap".to_string(),
                params: vec![
                    ("addr".to_string(), ResolvedType::I64, false),
                    ("len".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // msync: (addr, len, flags) -> int
        self.functions.insert(
            "msync".to_string(),
            FunctionSig {
                name: "msync".to_string(),
                params: vec![
                    ("addr".to_string(), ResolvedType::I64, false),
                    ("len".to_string(), ResolvedType::I64, false),
                    ("flags".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // madvise: (addr, len, advice) -> int
        self.functions.insert(
            "madvise".to_string(),
            FunctionSig {
                name: "madvise".to_string(),
                params: vec![
                    ("addr".to_string(), ResolvedType::I64, false),
                    ("len".to_string(), ResolvedType::I64, false),
                    ("advice".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // posix_open: (path, flags, mode) -> fd
        self.functions.insert(
            "posix_open".to_string(),
            FunctionSig {
                name: "posix_open".to_string(),
                params: vec![
                    ("path".to_string(), ResolvedType::Str, false),
                    ("flags".to_string(), ResolvedType::I64, false),
                    ("mode".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // posix_close: (fd) -> i64
        self.functions.insert(
            "posix_close".to_string(),
            FunctionSig {
                name: "posix_close".to_string(),
                params: vec![("fd".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // remove: (path) -> i64
        self.functions.insert(
            "remove".to_string(),
            FunctionSig {
                name: "remove".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // flock: (fd, operation) -> i64 (advisory file locking)
        self.functions.insert(
            "flock".to_string(),
            FunctionSig {
                name: "flock".to_string(),
                params: vec![
                    ("fd".to_string(), ResolvedType::I64, false),
                    ("operation".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // mkdir: (path, mode) -> i64 (0 on success, -1 on error)
        self.functions.insert(
            "mkdir".to_string(),
            FunctionSig {
                name: "mkdir".to_string(),
                params: vec![
                    ("path".to_string(), ResolvedType::Str, false),
                    ("mode".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // rmdir: (path) -> i64 (0 on success, -1 on error)
        self.functions.insert(
            "rmdir".to_string(),
            FunctionSig {
                name: "rmdir".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // opendir: (path) -> i64 (DIR* as i64, 0 on error)
        self.functions.insert(
            "opendir".to_string(),
            FunctionSig {
                name: "opendir".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // readdir: (dirp) -> i64 (pointer to dirent name, 0 at end)
        self.functions.insert(
            "readdir".to_string(),
            FunctionSig {
                name: "readdir".to_string(),
                params: vec![("dirp".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // closedir: (dirp) -> i64 (0 on success)
        self.functions.insert(
            "closedir".to_string(),
            FunctionSig {
                name: "closedir".to_string(),
                params: vec![("dirp".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // rename_file: (old, new_path) -> i64 (0 on success)
        self.functions.insert(
            "rename_file".to_string(),
            FunctionSig {
                name: "rename_file".to_string(),
                params: vec![
                    ("old".to_string(), ResolvedType::Str, false),
                    ("new_path".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // unlink: (path) -> i64 (0 on success)
        self.functions.insert(
            "unlink".to_string(),
            FunctionSig {
                name: "unlink".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // stat_size: (path) -> i64 (file size in bytes)
        self.functions.insert(
            "stat_size".to_string(),
            FunctionSig {
                name: "stat_size".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // stat_mtime: (path) -> i64 (modification time as unix timestamp)
        self.functions.insert(
            "stat_mtime".to_string(),
            FunctionSig {
                name: "stat_mtime".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // getcwd: (buf, size) -> i64 (pointer to buf on success, 0 on error)
        self.functions.insert(
            "getcwd".to_string(),
            FunctionSig {
                name: "getcwd".to_string(),
                params: vec![
                    ("buf".to_string(), ResolvedType::I64, false),
                    ("size".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // chdir: (path) -> i64 (0 on success)
        self.functions.insert(
            "chdir".to_string(),
            FunctionSig {
                name: "chdir".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // access: (path, mode) -> i64 (0 on success, -1 on error)
        self.functions.insert(
            "access".to_string(),
            FunctionSig {
                name: "access".to_string(),
                params: vec![
                    ("path".to_string(), ResolvedType::Str, false),
                    ("mode".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
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
                params: vec![
                    ("x".to_string(), ResolvedType::F32, false),
                    ("y".to_string(), ResolvedType::F32, false),
                ],
                ret: vec2f32.clone(),
                ..Default::default()
            },
        );

        // vec4f32(x, y, z, w) -> Vec4f32
        self.functions.insert(
            "vec4f32".to_string(),
            FunctionSig {
                name: "vec4f32".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::F32, false),
                    ("y".to_string(), ResolvedType::F32, false),
                    ("z".to_string(), ResolvedType::F32, false),
                    ("w".to_string(), ResolvedType::F32, false),
                ],
                ret: vec4f32.clone(),
                ..Default::default()
            },
        );

        // vec8f32(a, b, c, d, e, f, g, h) -> Vec8f32
        self.functions.insert(
            "vec8f32".to_string(),
            FunctionSig {
                name: "vec8f32".to_string(),
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
                ..Default::default()
            },
        );

        // vec2f64(x, y) -> Vec2f64
        self.functions.insert(
            "vec2f64".to_string(),
            FunctionSig {
                name: "vec2f64".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::F64, false),
                    ("y".to_string(), ResolvedType::F64, false),
                ],
                ret: vec2f64.clone(),
                ..Default::default()
            },
        );

        // vec4f64(x, y, z, w) -> Vec4f64
        self.functions.insert(
            "vec4f64".to_string(),
            FunctionSig {
                name: "vec4f64".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::F64, false),
                    ("y".to_string(), ResolvedType::F64, false),
                    ("z".to_string(), ResolvedType::F64, false),
                    ("w".to_string(), ResolvedType::F64, false),
                ],
                ret: vec4f64.clone(),
                ..Default::default()
            },
        );

        // vec4i32(x, y, z, w) -> Vec4i32
        self.functions.insert(
            "vec4i32".to_string(),
            FunctionSig {
                name: "vec4i32".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::I32, false),
                    ("y".to_string(), ResolvedType::I32, false),
                    ("z".to_string(), ResolvedType::I32, false),
                    ("w".to_string(), ResolvedType::I32, false),
                ],
                ret: vec4i32.clone(),
                ..Default::default()
            },
        );

        // vec8i32(a, b, c, d, e, f, g, h) -> Vec8i32
        self.functions.insert(
            "vec8i32".to_string(),
            FunctionSig {
                name: "vec8i32".to_string(),
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
                ..Default::default()
            },
        );

        // vec2i64(x, y) -> Vec2i64
        self.functions.insert(
            "vec2i64".to_string(),
            FunctionSig {
                name: "vec2i64".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::I64, false),
                    ("y".to_string(), ResolvedType::I64, false),
                ],
                ret: vec2i64.clone(),
                ..Default::default()
            },
        );

        // vec4i64(x, y, z, w) -> Vec4i64
        self.functions.insert(
            "vec4i64".to_string(),
            FunctionSig {
                name: "vec4i64".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::I64, false),
                    ("y".to_string(), ResolvedType::I64, false),
                    ("z".to_string(), ResolvedType::I64, false),
                    ("w".to_string(), ResolvedType::I64, false),
                ],
                ret: vec4i64.clone(),
                ..Default::default()
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
                        params: vec![
                            ("a".to_string(), $vec_ty.clone(), false),
                            ("b".to_string(), $vec_ty.clone(), false),
                        ],
                        ret: $vec_ty.clone(),
                        ..Default::default()
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
                params: vec![("v".to_string(), vec4f32.clone(), false)],
                ret: ResolvedType::F32,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec8f32(v) -> f32
        self.functions.insert(
            "simd_reduce_add_vec8f32".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec8f32".to_string(),
                params: vec![("v".to_string(), vec8f32, false)],
                ret: ResolvedType::F32,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec2f64(v) -> f64
        self.functions.insert(
            "simd_reduce_add_vec2f64".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec2f64".to_string(),
                params: vec![("v".to_string(), vec2f64, false)],
                ret: ResolvedType::F64,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec4f64(v) -> f64
        self.functions.insert(
            "simd_reduce_add_vec4f64".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec4f64".to_string(),
                params: vec![("v".to_string(), vec4f64, false)],
                ret: ResolvedType::F64,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec4i32(v) -> i32
        self.functions.insert(
            "simd_reduce_add_vec4i32".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec4i32".to_string(),
                params: vec![("v".to_string(), vec4i32, false)],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec8i32(v) -> i32
        self.functions.insert(
            "simd_reduce_add_vec8i32".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec8i32".to_string(),
                params: vec![("v".to_string(), vec8i32, false)],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec2i64(v) -> i64
        self.functions.insert(
            "simd_reduce_add_vec2i64".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec2i64".to_string(),
                params: vec![("v".to_string(), vec2i64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec4i64(v) -> i64
        self.functions.insert(
            "simd_reduce_add_vec4i64".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec4i64".to_string(),
                params: vec![("v".to_string(), vec4i64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
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
                    params,
                    ret,
                    ..Default::default()
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
                    hkt_params: HashMap::new(),
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
                    hkt_params: HashMap::new(),
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
