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
        $gen.functions.insert(
            $name.to_string(),
            FunctionInfo {
                signature: FunctionSig {
                    name: $name.to_string(),
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
                },
                is_extern: true,
                _extern_abi: Some("C".to_string()),
            },
        );
    };
    ($gen:expr, $key:expr => $name:expr, $params:expr, $ret:expr) => {
        $gen.functions.insert(
            $key.to_string(),
            FunctionInfo {
                signature: FunctionSig {
                    name: $name.to_string(),
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
                },
                is_extern: true,
                _extern_abi: Some("C".to_string()),
            },
        );
    };
}

/// Macro for registering internal helper functions
macro_rules! register_helper {
    ($gen:expr, $key:expr => $name:expr, $params:expr, $ret:expr) => {
        $gen.functions.insert(
            $key.to_string(),
            FunctionInfo {
                signature: FunctionSig {
                    name: $name.to_string(),
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
                },
                is_extern: false,
                _extern_abi: None,
            },
        );
    };
}

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

    fn register_io_functions(&mut self) {
        // printf for printing (variadic)
        self.functions.insert(
            "printf".to_string(),
            FunctionInfo {
                signature: FunctionSig {
                    name: "printf".to_string(),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: convert_params(vec![("format".to_string(), ResolvedType::Str)]),
                    ret: ResolvedType::I32,
                    is_async: false,
                    is_vararg: true,
                    required_params: Some(1),
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                },
                is_extern: true,
                _extern_abi: Some("C".to_string()),
            },
        );

        // putchar for single character output
        register_extern!(
            self,
            "putchar",
            vec![("c".to_string(), ResolvedType::I32)],
            ResolvedType::I32
        );

        // puts for simple string output
        register_extern!(
            self,
            "puts",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::I32
        );

        // puts_ptr: print string from pointer (maps to C puts)
        register_extern!(self, "puts_ptr" => "puts",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::I32
        );

        // print: format string output (no newline)
        // Registered as vararg; first arg is format string, rest are values
        self.functions.insert(
            "print".to_string(),
            FunctionInfo {
                signature: FunctionSig {
                    name: "print".to_string(),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: convert_params(vec![("format".to_string(), ResolvedType::Str)]),
                    ret: ResolvedType::Unit,
                    is_async: false,
                    is_vararg: true,
                    required_params: Some(1),
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                },
                is_extern: false,
                _extern_abi: None,
            },
        );

        // println: format string output (with newline)
        self.functions.insert(
            "println".to_string(),
            FunctionInfo {
                signature: FunctionSig {
                    name: "println".to_string(),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: convert_params(vec![("format".to_string(), ResolvedType::Str)]),
                    ret: ResolvedType::Unit,
                    is_async: false,
                    is_vararg: true,
                    required_params: Some(1),
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                },
                is_extern: false,
                _extern_abi: None,
            },
        );

        // format: format string output, returns allocated string
        self.functions.insert(
            "format".to_string(),
            FunctionInfo {
                signature: FunctionSig {
                    name: "format".to_string(),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: convert_params(vec![("format".to_string(), ResolvedType::Str)]),
                    ret: ResolvedType::Str,
                    is_async: false,
                    is_vararg: true,
                    required_params: Some(1),
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                },
                is_extern: false,
                _extern_abi: None,
            },
        );

        // exit: (i32) -> void (noreturn)
        register_extern!(
            self,
            "exit",
            vec![("code".to_string(), ResolvedType::I32)],
            ResolvedType::Unit
        );
    }

    fn register_memory_functions(&mut self) {
        // malloc: (i64) -> i64 (pointer as integer)
        register_extern!(
            self,
            "malloc",
            vec![("size".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // free: (i64) -> void
        register_extern!(
            self,
            "free",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::Unit
        );

        // memcpy: (dest, src, n) -> dest
        register_extern!(
            self,
            "memcpy",
            vec![
                ("dest".to_string(), ResolvedType::I64),
                ("src".to_string(), ResolvedType::I64),
                ("n".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // memcmp: (s1, s2, n) -> i64 (compare memory)
        register_extern!(
            self,
            "memcmp",
            vec![
                ("s1".to_string(), ResolvedType::I64),
                ("s2".to_string(), ResolvedType::I64),
                ("n".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // load_byte: load single byte from memory (internal helper)
        register_helper!(self, "load_byte" => "__load_byte",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_byte: store single byte to memory (internal helper)
        register_helper!(self, "store_byte" => "__store_byte",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_i64: load 64-bit integer from memory (internal helper)
        register_helper!(self, "load_i64" => "__load_i64",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_i64: store 64-bit integer to memory (internal helper)
        register_helper!(self, "store_i64" => "__store_i64",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // swap: swap two i64 elements in array (internal helper)
        register_helper!(self, "swap" => "__swap",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("idx1".to_string(), ResolvedType::I64),
                ("idx2".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_f64: load 64-bit float from memory (internal helper)
        register_helper!(self, "load_f64" => "__load_f64",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::F64
        );

        // store_f64: store 64-bit float to memory (internal helper)
        register_helper!(self, "store_f64" => "__store_f64",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::F64),
            ],
            ResolvedType::Unit
        );

        // load_i8: load 8-bit integer from memory (internal helper)
        register_helper!(self, "load_i8" => "__load_i8",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_i8: store 8-bit integer to memory (internal helper)
        register_helper!(self, "store_i8" => "__store_i8",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_i16: load 16-bit integer from memory (internal helper)
        register_helper!(self, "load_i16" => "__load_i16",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_i16: store 16-bit integer to memory (internal helper)
        register_helper!(self, "store_i16" => "__store_i16",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_i32: load 32-bit integer from memory (internal helper)
        register_helper!(self, "load_i32" => "__load_i32",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // store_i32: store 32-bit integer to memory (internal helper)
        register_helper!(self, "store_i32" => "__store_i32",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Unit
        );

        // load_f32: load 32-bit float from memory (internal helper)
        register_helper!(self, "load_f32" => "__load_f32",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::F64
        );

        // store_f32: store 32-bit float to memory (internal helper)
        register_helper!(self, "store_f32" => "__store_f32",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("val".to_string(), ResolvedType::F64),
            ],
            ResolvedType::Unit
        );

        // str_to_ptr: convert str (i8*) to i64 — IR is special-cased in generate_expr
        // Register so infer_expr_type returns the correct type
        self.functions.insert(
            "str_to_ptr".to_string(),
            FunctionInfo {
                signature: FunctionSig {
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
                is_extern: false,
                _extern_abi: None,
            },
        );

        // ptr_to_str: convert i64 to str (i8*) — IR is special-cased in generate_expr
        self.functions.insert(
            "ptr_to_str".to_string(),
            FunctionInfo {
                signature: FunctionSig {
                    name: "ptr_to_str".to_string(),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params: vec![("ptr".to_string(), ResolvedType::I64, false)],
                    ret: ResolvedType::Str,
                    is_async: false,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                },
                is_extern: false,
                _extern_abi: None,
            },
        );
    }

    fn register_file_functions(&mut self) {
        // fopen: (path, mode) -> FILE*
        register_extern!(
            self,
            "fopen",
            vec![
                ("path".to_string(), ResolvedType::Str),
                ("mode".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // fopen_ptr: same as fopen but accepts i64 pointers (for selfhost)
        register_extern!(
            self,
            "fopen_ptr",
            vec![
                ("path".to_string(), ResolvedType::I64),
                ("mode".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // fclose: (FILE*) -> int
        register_extern!(
            self,
            "fclose",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I32
        );

        // fread: (ptr, size, count, FILE*) -> size_t
        register_extern!(
            self,
            "fread",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("size".to_string(), ResolvedType::I64),
                ("count".to_string(), ResolvedType::I64),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fwrite: (ptr, size, count, FILE*) -> size_t
        register_extern!(
            self,
            "fwrite",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("size".to_string(), ResolvedType::I64),
                ("count".to_string(), ResolvedType::I64),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fgetc: (FILE*) -> int
        register_extern!(
            self,
            "fgetc",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fputc: (char, FILE*) -> int
        register_extern!(
            self,
            "fputc",
            vec![
                ("c".to_string(), ResolvedType::I64),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fgets_ptr: (str, i32, str) -> i64 - fgets with correct pointer types
        register_extern!(self, "fgets_ptr" => "fgets",
            vec![
                ("buffer".to_string(), ResolvedType::Str),
                ("n".to_string(), ResolvedType::I32),
                ("stream".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // fgets: (str, n, FILE*) -> char*
        register_extern!(
            self,
            "fgets",
            vec![
                ("str".to_string(), ResolvedType::I64),
                ("n".to_string(), ResolvedType::I64),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fputs: (str, FILE*) -> int
        register_extern!(
            self,
            "fputs",
            vec![
                ("str".to_string(), ResolvedType::Str),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fseek: (FILE*, offset, origin) -> int
        register_extern!(
            self,
            "fseek",
            vec![
                ("stream".to_string(), ResolvedType::I64),
                ("offset".to_string(), ResolvedType::I64),
                ("origin".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // ftell: (FILE*) -> long
        register_extern!(
            self,
            "ftell",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fflush: (FILE*) -> int
        register_extern!(
            self,
            "fflush",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // feof: (FILE*) -> int
        register_extern!(
            self,
            "feof",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fileno: (FILE*) -> int (get file descriptor from FILE*)
        register_extern!(
            self,
            "fileno",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fsync: (fd) -> int (flush to disk)
        register_extern!(
            self,
            "fsync",
            vec![("fd".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fdatasync: (fd) -> int (flush data only, no metadata)
        // On macOS, mapped to fcntl F_FULLFSYNC or fsync fallback
        register_extern!(
            self,
            "fdatasync",
            vec![("fd".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // mmap: (addr, len, prot, flags, fd, offset) -> void* (as i64)
        register_extern!(
            self,
            "mmap",
            vec![
                ("addr".to_string(), ResolvedType::I64),
                ("len".to_string(), ResolvedType::I64),
                ("prot".to_string(), ResolvedType::I64),
                ("flags".to_string(), ResolvedType::I64),
                ("fd".to_string(), ResolvedType::I64),
                ("offset".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // munmap: (addr, len) -> int
        register_extern!(
            self,
            "munmap",
            vec![
                ("addr".to_string(), ResolvedType::I64),
                ("len".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // msync: (addr, len, flags) -> int
        register_extern!(
            self,
            "msync",
            vec![
                ("addr".to_string(), ResolvedType::I64),
                ("len".to_string(), ResolvedType::I64),
                ("flags".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // madvise: (addr, len, advice) -> int
        register_extern!(
            self,
            "madvise",
            vec![
                ("addr".to_string(), ResolvedType::I64),
                ("len".to_string(), ResolvedType::I64),
                ("advice".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // POSIX open: (path, flags, mode) -> fd
        register_extern!(self, "posix_open" => "open",
            vec![
                ("path".to_string(), ResolvedType::Str),
                ("flags".to_string(), ResolvedType::I64),
                ("mode".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // POSIX close: (fd) -> int
        register_extern!(self, "posix_close" => "close",
            vec![("fd".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // remove: (path) -> int (delete file)
        register_extern!(
            self,
            "remove",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // flock: (fd, operation) -> int (advisory file locking)
        register_extern!(
            self,
            "flock",
            vec![
                ("fd".to_string(), ResolvedType::I64),
                ("operation".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // mkdir: (path, mode) -> int (0 on success, -1 on error)
        register_extern!(
            self,
            "mkdir",
            vec![
                ("path".to_string(), ResolvedType::Str),
                ("mode".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // rmdir: (path) -> int (0 on success, -1 on error)
        register_extern!(
            self,
            "rmdir",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // opendir: (path) -> DIR* (as i64, 0 on error)
        register_extern!(
            self,
            "opendir",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // readdir: (dirp) -> dirent* (pointer to name, 0 at end)
        register_helper!(self, "readdir" => "__readdir_wrapper",
            vec![("dirp".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // closedir: (dirp) -> int (0 on success)
        register_extern!(
            self,
            "closedir",
            vec![("dirp".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // rename_file: (old, new) -> int (0 on success) - maps to C rename()
        register_extern!(self, "rename_file" => "rename",
            vec![
                ("old".to_string(), ResolvedType::Str),
                ("new_path".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // unlink: (path) -> int (0 on success)
        register_extern!(
            self,
            "unlink",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // stat_size: (path) -> i64 (file size in bytes)
        register_helper!(self, "stat_size" => "__stat_size",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // stat_mtime: (path) -> i64 (modification time as unix timestamp)
        register_helper!(self, "stat_mtime" => "__stat_mtime",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // getcwd: (buf, size) -> i64 (pointer as i64, 0 on error) — needs wrapper for ptr→i64 conversion
        register_helper!(self, "getcwd" => "__getcwd_wrapper",
            vec![
                ("buf".to_string(), ResolvedType::I64),
                ("size".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // chdir: (path) -> int (0 on success)
        register_extern!(
            self,
            "chdir",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // access: (path, mode) -> int (0 on success, -1 on error)
        register_extern!(
            self,
            "access",
            vec![
                ("path".to_string(), ResolvedType::Str),
                ("mode".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );
    }

    fn register_string_functions(&mut self) {
        // strlen: (s) -> len (accepts str)
        register_extern!(
            self,
            "strlen",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // strcmp: (s1, s2) -> int
        register_extern!(
            self,
            "strcmp",
            vec![
                ("s1".to_string(), ResolvedType::Str),
                ("s2".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I32
        );

        // strncmp: (s1, s2, n) -> int
        register_extern!(
            self,
            "strncmp",
            vec![
                ("s1".to_string(), ResolvedType::Str),
                ("s2".to_string(), ResolvedType::Str),
                ("n".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I32
        );

        // memcpy_str: (dest: str, src: str, len: i64) -> str
        // Copies len bytes from str src to str dest pointer
        register_extern!(self, "memcpy_str" => "memcpy",
            vec![
                ("dest".to_string(), ResolvedType::Str),
                ("src".to_string(), ResolvedType::Str),
                ("len".to_string(), ResolvedType::I64),
            ],
            ResolvedType::Str
        );
    }

    fn register_stdlib_functions(&mut self) {
        // --- Number conversion functions ---

        // atoi: (s: str) -> i32 - string to integer
        register_extern!(
            self,
            "atoi",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::I32
        );

        // atol: (s: str) -> i64 - string to long integer
        register_extern!(
            self,
            "atol",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // atol_ptr: (s: str) -> i64 - atol with pointer param
        register_extern!(self, "atol_ptr" => "atol",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // atof: (s: str) -> f64 - string to double
        register_extern!(
            self,
            "atof",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::F64
        );

        // atof_ptr: (s: str) -> f64 - atof with pointer param
        register_extern!(self, "atof_ptr" => "atof",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::F64
        );

        // --- Math functions ---

        // labs: (x: i64) -> i64 - absolute value (long integer)
        register_extern!(
            self,
            "labs",
            vec![("x".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fabs: (x: f64) -> f64 - absolute value (double)
        register_extern!(
            self,
            "fabs",
            vec![("x".to_string(), ResolvedType::F64)],
            ResolvedType::F64
        );

        // sqrt: (x: f64) -> f64 - square root
        register_extern!(
            self,
            "sqrt",
            vec![("x".to_string(), ResolvedType::F64)],
            ResolvedType::F64
        );

        // sin: (x: f64) -> f64 - sine
        register_extern!(
            self,
            "sin",
            vec![("x".to_string(), ResolvedType::F64)],
            ResolvedType::F64
        );

        // cos: (x: f64) -> f64 - cosine
        register_extern!(
            self,
            "cos",
            vec![("x".to_string(), ResolvedType::F64)],
            ResolvedType::F64
        );

        // exp: (x: f64) -> f64 - exponential
        register_extern!(
            self,
            "exp",
            vec![("x".to_string(), ResolvedType::F64)],
            ResolvedType::F64
        );

        // log: (x: f64) -> f64 - natural logarithm
        register_extern!(
            self,
            "log",
            vec![("x".to_string(), ResolvedType::F64)],
            ResolvedType::F64
        );

        // rand: () -> i32 - pseudo-random number
        register_extern!(self, "rand", vec![], ResolvedType::I32);

        // srand: (seed: i32) -> void - seed random number generator
        register_extern!(
            self,
            "srand",
            vec![("seed".to_string(), ResolvedType::I32)],
            ResolvedType::Unit
        );

        // --- Character classification functions ---

        // isdigit: (c: i32) -> i32 - test if digit
        register_extern!(
            self,
            "isdigit",
            vec![("c".to_string(), ResolvedType::I32)],
            ResolvedType::I32
        );

        // isalpha: (c: i32) -> i32 - test if alphabetic
        register_extern!(
            self,
            "isalpha",
            vec![("c".to_string(), ResolvedType::I32)],
            ResolvedType::I32
        );

        // toupper: (c: i32) -> i32 - convert to uppercase
        register_extern!(
            self,
            "toupper",
            vec![("c".to_string(), ResolvedType::I32)],
            ResolvedType::I32
        );

        // tolower: (c: i32) -> i32 - convert to lowercase
        register_extern!(
            self,
            "tolower",
            vec![("c".to_string(), ResolvedType::I32)],
            ResolvedType::I32
        );

        // --- String manipulation functions ---

        // strcpy: (dest: i64, src: str) -> i64 - copy string
        register_extern!(
            self,
            "strcpy",
            vec![
                ("dest".to_string(), ResolvedType::I64),
                ("src".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // strcat: (dest: i64, src: str) -> i64 - concatenate string
        register_extern!(
            self,
            "strcat",
            vec![
                ("dest".to_string(), ResolvedType::I64),
                ("src".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );
    }

    fn register_async_functions(&mut self) {
        // usleep: microsecond sleep for cooperative scheduling
        register_extern!(
            self,
            "usleep",
            vec![("usec".to_string(), ResolvedType::I64)],
            ResolvedType::I32
        );

        // sched_yield: yield CPU to other processes
        register_extern!(self, "sched_yield", vec![], ResolvedType::I32);

        // call_poll: call an indirect poll function pointer with a future pointer
        // Returns an i64 encoding {status, value} as a packed struct
        register_helper!(self, "call_poll" => "__call_poll",
            vec![
                ("poll_fn".to_string(), ResolvedType::I64),
                ("future_ptr".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // extract_poll_status: extract status (0=Pending, 1=Ready) from poll result
        register_helper!(self, "extract_poll_status" => "__extract_poll_status",
            vec![("poll_result".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // extract_poll_value: extract value from poll result
        register_helper!(self, "extract_poll_value" => "__extract_poll_value",
            vec![("poll_result".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // time_now_ms: current time in milliseconds
        register_helper!(self, "time_now_ms" => "__time_now_ms",
            vec![],
            ResolvedType::I64
        );

        // === Platform I/O syscalls for async reactor ===

        // kqueue helpers: only on macOS (uses kevent syscall)
        #[cfg(target_os = "macos")]
        {
            // kqueue: create kqueue instance (macOS)
            register_extern!(self, "kqueue", vec![], ResolvedType::I64);

            // kevent_register: register event with kqueue
            register_helper!(self, "kevent_register" => "__kevent_register",
                vec![
                    ("kq".to_string(), ResolvedType::I64),
                    ("fd".to_string(), ResolvedType::I64),
                    ("filter".to_string(), ResolvedType::I64),
                    ("flags".to_string(), ResolvedType::I64),
                ],
                ResolvedType::I64
            );

            // kevent_wait: wait for events
            register_helper!(self, "kevent_wait" => "__kevent_wait",
                vec![
                    ("kq".to_string(), ResolvedType::I64),
                    ("events_buf".to_string(), ResolvedType::I64),
                    ("max_events".to_string(), ResolvedType::I64),
                    ("timeout_ms".to_string(), ResolvedType::I64),
                ],
                ResolvedType::I64
            );

            // kevent_get_fd: get fd from event at index
            register_helper!(self, "kevent_get_fd" => "__kevent_get_fd",
                vec![
                    ("events_buf".to_string(), ResolvedType::I64),
                    ("index".to_string(), ResolvedType::I64),
                ],
                ResolvedType::I64
            );

            // kevent_get_filter: get filter from event at index
            register_helper!(self, "kevent_get_filter" => "__kevent_get_filter",
                vec![
                    ("events_buf".to_string(), ResolvedType::I64),
                    ("index".to_string(), ResolvedType::I64),
                ],
                ResolvedType::I64
            );
        }

        // close: close file descriptor
        register_extern!(
            self,
            "close",
            vec![("fd".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // pipe: create pipe (takes buffer pointer for two fds)
        register_extern!(self, "pipe" => "pipe",
            vec![("fds_buf".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // write_byte: write a single byte to fd
        register_helper!(self, "write_byte" => "__write_byte",
            vec![
                ("fd".to_string(), ResolvedType::I64),
                ("value".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // read_byte: read a single byte from fd
        register_helper!(self, "read_byte" => "__read_byte",
            vec![("fd".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // === Cross-platform async reactor support ===

        // async_platform: returns platform ID (1=macOS, 2=Linux, 3=Windows)
        register_helper!(self, "async_platform" => "__async_platform",
            vec![],
            ResolvedType::I64
        );

        // epoll_set_timer_ms: configure timerfd delay (Linux epoll backend)
        register_helper!(self, "epoll_set_timer_ms" => "__epoll_set_timer_ms",
            vec![
                ("kq".to_string(), ResolvedType::I64),
                ("timer_id".to_string(), ResolvedType::I64),
                ("delay_ms".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // iocp_set_timer_ms: configure timer delay (Windows IOCP backend)
        register_helper!(self, "iocp_set_timer_ms" => "__iocp_set_timer_ms",
            vec![
                ("kq".to_string(), ResolvedType::I64),
                ("timer_id".to_string(), ResolvedType::I64),
                ("delay_ms".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );
    }

    fn register_simd_functions(&mut self) {
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
        register_helper!(self, "vec2f32" => "vec2f32",
            vec![("x".to_string(), ResolvedType::F32), ("y".to_string(), ResolvedType::F32)],
            vec2f32.clone()
        );

        register_helper!(self, "vec4f32" => "vec4f32",
            vec![("x".to_string(), ResolvedType::F32), ("y".to_string(), ResolvedType::F32),
                 ("z".to_string(), ResolvedType::F32), ("w".to_string(), ResolvedType::F32)],
            vec4f32.clone()
        );

        register_helper!(self, "vec8f32" => "vec8f32",
            vec![("a".to_string(), ResolvedType::F32), ("b".to_string(), ResolvedType::F32),
                 ("c".to_string(), ResolvedType::F32), ("d".to_string(), ResolvedType::F32),
                 ("e".to_string(), ResolvedType::F32), ("f".to_string(), ResolvedType::F32),
                 ("g".to_string(), ResolvedType::F32), ("h".to_string(), ResolvedType::F32)],
            vec8f32.clone()
        );

        register_helper!(self, "vec2f64" => "vec2f64",
            vec![("x".to_string(), ResolvedType::F64), ("y".to_string(), ResolvedType::F64)],
            vec2f64.clone()
        );

        register_helper!(self, "vec4f64" => "vec4f64",
            vec![("x".to_string(), ResolvedType::F64), ("y".to_string(), ResolvedType::F64),
                 ("z".to_string(), ResolvedType::F64), ("w".to_string(), ResolvedType::F64)],
            vec4f64.clone()
        );

        register_helper!(self, "vec4i32" => "vec4i32",
            vec![("x".to_string(), ResolvedType::I32), ("y".to_string(), ResolvedType::I32),
                 ("z".to_string(), ResolvedType::I32), ("w".to_string(), ResolvedType::I32)],
            vec4i32.clone()
        );

        register_helper!(self, "vec8i32" => "vec8i32",
            vec![("a".to_string(), ResolvedType::I32), ("b".to_string(), ResolvedType::I32),
                 ("c".to_string(), ResolvedType::I32), ("d".to_string(), ResolvedType::I32),
                 ("e".to_string(), ResolvedType::I32), ("f".to_string(), ResolvedType::I32),
                 ("g".to_string(), ResolvedType::I32), ("h".to_string(), ResolvedType::I32)],
            vec8i32.clone()
        );

        register_helper!(self, "vec2i64" => "vec2i64",
            vec![("x".to_string(), ResolvedType::I64), ("y".to_string(), ResolvedType::I64)],
            vec2i64.clone()
        );

        register_helper!(self, "vec4i64" => "vec4i64",
            vec![("x".to_string(), ResolvedType::I64), ("y".to_string(), ResolvedType::I64),
                 ("z".to_string(), ResolvedType::I64), ("w".to_string(), ResolvedType::I64)],
            vec4i64.clone()
        );

        // === SIMD Arithmetic Operations ===
        macro_rules! register_simd_binop {
            ($name:expr, $vec_ty:expr) => {
                register_helper!(self, $name => $name,
                    vec![("a".to_string(), $vec_ty.clone()), ("b".to_string(), $vec_ty.clone())],
                    $vec_ty.clone()
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
        register_helper!(self, "simd_reduce_add_vec4f32" => "simd_reduce_add_vec4f32",
            vec![("v".to_string(), vec4f32)], ResolvedType::F32);
        register_helper!(self, "simd_reduce_add_vec8f32" => "simd_reduce_add_vec8f32",
            vec![("v".to_string(), vec8f32)], ResolvedType::F32);
        register_helper!(self, "simd_reduce_add_vec2f64" => "simd_reduce_add_vec2f64",
            vec![("v".to_string(), vec2f64)], ResolvedType::F64);
        register_helper!(self, "simd_reduce_add_vec4f64" => "simd_reduce_add_vec4f64",
            vec![("v".to_string(), vec4f64)], ResolvedType::F64);
        register_helper!(self, "simd_reduce_add_vec4i32" => "simd_reduce_add_vec4i32",
            vec![("v".to_string(), vec4i32)], ResolvedType::I32);
        register_helper!(self, "simd_reduce_add_vec8i32" => "simd_reduce_add_vec8i32",
            vec![("v".to_string(), vec8i32)], ResolvedType::I32);
        register_helper!(self, "simd_reduce_add_vec2i64" => "simd_reduce_add_vec2i64",
            vec![("v".to_string(), vec2i64)], ResolvedType::I64);
        register_helper!(self, "simd_reduce_add_vec4i64" => "simd_reduce_add_vec4i64",
            vec![("v".to_string(), vec4i64)], ResolvedType::I64);
    }

    fn register_gc_functions(&mut self) {
        // GC runtime functions
        register_extern!(self, "vais_gc_init", vec![], ResolvedType::I64);

        register_extern!(
            self,
            "vais_gc_alloc",
            vec![
                ("size".to_string(), ResolvedType::I64),
                ("type_id".to_string(), ResolvedType::I32),
            ],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "vais_gc_add_root",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "vais_gc_remove_root",
            vec![("ptr".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        register_extern!(self, "vais_gc_collect", vec![], ResolvedType::I64);

        register_extern!(self, "vais_gc_bytes_allocated", vec![], ResolvedType::I64);

        register_extern!(self, "vais_gc_objects_count", vec![], ResolvedType::I64);

        register_extern!(self, "vais_gc_collections", vec![], ResolvedType::I64);

        register_extern!(
            self,
            "vais_gc_set_threshold",
            vec![("threshold".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        register_extern!(self, "vais_gc_print_stats", vec![], ResolvedType::I64);
    }

    fn register_system_functions(&mut self) {
        // Environment variable functions
        register_extern!(
            self,
            "getenv",
            vec![("name".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "setenv",
            vec![
                ("name".to_string(), ResolvedType::Str),
                ("value".to_string(), ResolvedType::Str),
                ("overwrite".to_string(), ResolvedType::I32),
            ],
            ResolvedType::I32
        );

        register_extern!(
            self,
            "unsetenv",
            vec![("name".to_string(), ResolvedType::Str)],
            ResolvedType::I32
        );

        // Process execution functions
        register_extern!(
            self,
            "system",
            vec![("command".to_string(), ResolvedType::Str)],
            ResolvedType::I32
        );

        register_extern!(
            self,
            "popen",
            vec![
                ("command".to_string(), ResolvedType::Str),
                ("mode".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "pclose",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I32
        );

        // Exit
        register_extern!(
            self,
            "exit",
            vec![("status".to_string(), ResolvedType::I32)],
            ResolvedType::Unit
        );

        // Signal handling
        register_extern!(
            self,
            "signal",
            vec![
                ("signum".to_string(), ResolvedType::I32),
                ("handler".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        register_extern!(
            self,
            "raise",
            vec![("signum".to_string(), ResolvedType::I32)],
            ResolvedType::I32
        );
    }
}
