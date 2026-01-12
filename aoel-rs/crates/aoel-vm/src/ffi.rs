//! AOEL FFI - Dynamic Library Loading
//!
//! libloading을 사용하여 동적 라이브러리에서 함수를 로드하고 호출

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int, c_long};
use std::path::Path;
use std::sync::Arc;

use libloading::{Library, Symbol};

use aoel_ir::Value;
use crate::error::{RuntimeError, RuntimeResult};

/// FFI 함수 시그니처 타입
#[derive(Debug, Clone, PartialEq)]
pub enum FfiType {
    Void,
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Ptr,
    CStr,
}

impl FfiType {
    /// 문자열에서 FfiType 파싱
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "void" => Some(FfiType::Void),
            "bool" => Some(FfiType::Bool),
            "i8" => Some(FfiType::I8),
            "i16" => Some(FfiType::I16),
            "i32" | "int" => Some(FfiType::I32),
            "i64" | "long" => Some(FfiType::I64),
            "u8" => Some(FfiType::U8),
            "u16" => Some(FfiType::U16),
            "u32" | "uint" => Some(FfiType::U32),
            "u64" | "ulong" => Some(FfiType::U64),
            "f32" | "float" => Some(FfiType::F32),
            "f64" | "double" => Some(FfiType::F64),
            "ptr" | "pointer" => Some(FfiType::Ptr),
            "cstr" | "string" => Some(FfiType::CStr),
            _ => None,
        }
    }
}

/// FFI 함수 정보
#[derive(Debug, Clone)]
pub struct FfiFunctionInfo {
    pub name: String,
    pub params: Vec<FfiType>,
    pub return_type: FfiType,
}

/// 로드된 라이브러리와 함수들을 관리
pub struct FfiLoader {
    /// 로드된 라이브러리들 (lib_name -> Library)
    libraries: HashMap<String, Arc<Library>>,
    /// 등록된 함수 정보 (lib_name::fn_name -> FfiFunctionInfo)
    functions: HashMap<String, FfiFunctionInfo>,
    /// 라이브러리 검색 경로
    search_paths: Vec<String>,
}

impl FfiLoader {
    pub fn new() -> Self {
        Self {
            libraries: HashMap::new(),
            functions: HashMap::new(),
            search_paths: vec![
                ".".to_string(),
                "/usr/lib".to_string(),
                "/usr/local/lib".to_string(),
            ],
        }
    }

    /// 검색 경로 추가
    pub fn add_search_path(&mut self, path: &str) {
        self.search_paths.push(path.to_string());
    }

    /// 라이브러리 로드
    pub fn load_library(&mut self, lib_name: &str) -> RuntimeResult<()> {
        if self.libraries.contains_key(lib_name) {
            return Ok(()); // 이미 로드됨
        }

        // 라이브러리 파일 이름 결정
        let lib_filename = self.resolve_library_name(lib_name);

        // 라이브러리 로드 시도
        let library = unsafe {
            // 먼저 직접 이름으로 시도
            let mut last_error = None;
            let result = Library::new(&lib_filename);

            if let Ok(lib) = result {
                lib
            } else {
                // 검색 경로에서 찾기
                let mut found_lib = None;
                for path in &self.search_paths {
                    let full_path = Path::new(path).join(&lib_filename);
                    match Library::new(&full_path) {
                        Ok(lib) => {
                            found_lib = Some(lib);
                            break;
                        }
                        Err(e) => {
                            last_error = Some(e);
                        }
                    }
                }

                match found_lib {
                    Some(lib) => lib,
                    None => {
                        let err_msg = last_error
                            .map(|e| e.to_string())
                            .unwrap_or_else(|| "Library not found".to_string());
                        return Err(RuntimeError::FfiError(format!(
                            "Failed to load library '{}': {}", lib_name, err_msg
                        )));
                    }
                }
            }
        };

        self.libraries.insert(lib_name.to_string(), Arc::new(library));
        Ok(())
    }

    /// 플랫폼별 라이브러리 파일 이름 해석
    fn resolve_library_name(&self, lib_name: &str) -> String {
        // 특수 케이스: "c" 또는 "libc"는 시스템 C 라이브러리
        if lib_name == "c" || lib_name == "libc" {
            #[cfg(target_os = "macos")]
            return "libSystem.B.dylib".to_string();
            #[cfg(target_os = "linux")]
            return "libc.so.6".to_string();
            #[cfg(target_os = "windows")]
            return "msvcrt.dll".to_string();
            #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
            return format!("lib{}.so", lib_name);
        }

        // 특수 케이스: "libm"은 수학 라이브러리
        if lib_name == "m" || lib_name == "libm" {
            #[cfg(target_os = "macos")]
            return "libSystem.B.dylib".to_string(); // macOS에서 libm은 libSystem에 포함
            #[cfg(target_os = "linux")]
            return "libm.so.6".to_string();
            #[cfg(target_os = "windows")]
            return "msvcrt.dll".to_string();
            #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
            return "libm.so".to_string();
        }

        // 이미 확장자가 있는 경우 그대로 사용
        if lib_name.ends_with(".so") || lib_name.ends_with(".dylib") || lib_name.ends_with(".dll") {
            return lib_name.to_string();
        }

        // 플랫폼별 라이브러리 이름 생성
        #[cfg(target_os = "macos")]
        return format!("lib{}.dylib", lib_name);
        #[cfg(target_os = "linux")]
        return format!("lib{}.so", lib_name);
        #[cfg(target_os = "windows")]
        return format!("{}.dll", lib_name);
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        return format!("lib{}.so", lib_name);
    }

    /// FFI 함수 등록
    pub fn register_function(
        &mut self,
        lib_name: &str,
        fn_name: &str,
        params: Vec<FfiType>,
        return_type: FfiType,
    ) {
        let key = format!("{}::{}", lib_name, fn_name);
        self.functions.insert(key, FfiFunctionInfo {
            name: fn_name.to_string(),
            params,
            return_type,
        });
    }

    /// FFI 함수 호출
    pub fn call_function(
        &mut self,
        lib_name: &str,
        fn_name: &str,
        args: &[Value],
    ) -> RuntimeResult<Value> {
        // 라이브러리 로드 (아직 안됐으면)
        self.load_library(lib_name)?;

        let library = self.libraries.get(lib_name)
            .ok_or_else(|| RuntimeError::FfiError(
                format!("Library '{}' not loaded", lib_name)
            ))?
            .clone();

        // 함수 정보 조회
        let key = format!("{}::{}", lib_name, fn_name);
        let func_info = self.functions.get(&key).cloned();

        // 함수 호출 (시그니처에 따라 다르게 처리)
        self.call_dynamic(&library, fn_name, args, func_info.as_ref())
    }

    /// 동적 함수 호출 (타입에 따른 디스패치)
    fn call_dynamic(
        &self,
        library: &Library,
        fn_name: &str,
        args: &[Value],
        func_info: Option<&FfiFunctionInfo>,
    ) -> RuntimeResult<Value> {
        // 인자 개수와 타입에 따라 적절한 함수 시그니처로 호출
        // 일반적인 시그니처들을 처리

        let arg_count = args.len();

        // 반환 타입 결정
        let return_type = func_info
            .map(|f| f.return_type.clone())
            .unwrap_or(FfiType::I64); // 기본값은 i64

        match arg_count {
            0 => self.call_0(library, fn_name, &return_type),
            1 => self.call_1(library, fn_name, args, func_info, &return_type),
            2 => self.call_2(library, fn_name, args, func_info, &return_type),
            3 => self.call_3(library, fn_name, args, func_info, &return_type),
            _ => Err(RuntimeError::FfiError(
                format!("FFI call with {} arguments not supported", arg_count)
            )),
        }
    }

    /// 인자 없는 함수 호출
    fn call_0(&self, library: &Library, fn_name: &str, return_type: &FfiType) -> RuntimeResult<Value> {
        unsafe {
            match return_type {
                FfiType::Void => {
                    let func: Symbol<unsafe extern "C" fn()> = library.get(fn_name.as_bytes())
                        .map_err(|e| RuntimeError::FfiError(format!("Function '{}' not found: {}", fn_name, e)))?;
                    func();
                    Ok(Value::Void)
                }
                FfiType::I32 | FfiType::I64 => {
                    let func: Symbol<unsafe extern "C" fn() -> c_long> = library.get(fn_name.as_bytes())
                        .map_err(|e| RuntimeError::FfiError(format!("Function '{}' not found: {}", fn_name, e)))?;
                    Ok(Value::Int(func() as i64))
                }
                FfiType::F64 => {
                    let func: Symbol<unsafe extern "C" fn() -> c_double> = library.get(fn_name.as_bytes())
                        .map_err(|e| RuntimeError::FfiError(format!("Function '{}' not found: {}", fn_name, e)))?;
                    Ok(Value::Float(func()))
                }
                _ => Err(RuntimeError::FfiError("Unsupported return type".to_string())),
            }
        }
    }

    /// 인자 1개 함수 호출
    fn call_1(
        &self,
        library: &Library,
        fn_name: &str,
        args: &[Value],
        func_info: Option<&FfiFunctionInfo>,
        return_type: &FfiType,
    ) -> RuntimeResult<Value> {
        let arg = &args[0];
        let param_type = func_info
            .and_then(|f| f.params.first().cloned())
            .unwrap_or_else(|| self.infer_type(arg));

        unsafe {
            match (&param_type, return_type) {
                // int -> int (abs 등)
                (FfiType::I32 | FfiType::I64, FfiType::I32 | FfiType::I64) => {
                    let func: Symbol<unsafe extern "C" fn(c_int) -> c_int> = library.get(fn_name.as_bytes())
                        .map_err(|e| RuntimeError::FfiError(format!("Function '{}' not found: {}", fn_name, e)))?;
                    let val = self.value_to_int(arg)?;
                    Ok(Value::Int(func(val as c_int) as i64))
                }
                // double -> double (sqrt, sin, cos 등)
                (FfiType::F64, FfiType::F64) => {
                    let func: Symbol<unsafe extern "C" fn(c_double) -> c_double> = library.get(fn_name.as_bytes())
                        .map_err(|e| RuntimeError::FfiError(format!("Function '{}' not found: {}", fn_name, e)))?;
                    let val = self.value_to_float(arg)?;
                    Ok(Value::Float(func(val)))
                }
                // cstr -> int (strlen 등)
                (FfiType::CStr, FfiType::I32 | FfiType::I64 | FfiType::U64) => {
                    let func: Symbol<unsafe extern "C" fn(*const c_char) -> c_long> = library.get(fn_name.as_bytes())
                        .map_err(|e| RuntimeError::FfiError(format!("Function '{}' not found: {}", fn_name, e)))?;
                    let s = self.value_to_cstring(arg)?;
                    Ok(Value::Int(func(s.as_ptr()) as i64))
                }
                // cstr -> cstr (getenv 등)
                (FfiType::CStr, FfiType::CStr | FfiType::Ptr) => {
                    let func: Symbol<unsafe extern "C" fn(*const c_char) -> *const c_char> = library.get(fn_name.as_bytes())
                        .map_err(|e| RuntimeError::FfiError(format!("Function '{}' not found: {}", fn_name, e)))?;
                    let s = self.value_to_cstring(arg)?;
                    let result = func(s.as_ptr());
                    if result.is_null() {
                        Ok(Value::Void)
                    } else {
                        let result_str = CStr::from_ptr(result).to_string_lossy().into_owned();
                        Ok(Value::String(result_str))
                    }
                }
                // cstr -> double (atof 등)
                (FfiType::CStr, FfiType::F64) => {
                    let func: Symbol<unsafe extern "C" fn(*const c_char) -> c_double> = library.get(fn_name.as_bytes())
                        .map_err(|e| RuntimeError::FfiError(format!("Function '{}' not found: {}", fn_name, e)))?;
                    let s = self.value_to_cstring(arg)?;
                    Ok(Value::Float(func(s.as_ptr())))
                }
                _ => Err(RuntimeError::FfiError(format!(
                    "Unsupported function signature for {}: {:?} -> {:?}",
                    fn_name, param_type, return_type
                ))),
            }
        }
    }

    /// 인자 2개 함수 호출
    fn call_2(
        &self,
        library: &Library,
        fn_name: &str,
        args: &[Value],
        func_info: Option<&FfiFunctionInfo>,
        return_type: &FfiType,
    ) -> RuntimeResult<Value> {
        let (arg1, arg2) = (&args[0], &args[1]);

        let param_types = func_info
            .map(|f| (f.params.get(0).cloned(), f.params.get(1).cloned()))
            .unwrap_or_else(|| (Some(self.infer_type(arg1)), Some(self.infer_type(arg2))));

        unsafe {
            match (&param_types.0, &param_types.1, return_type) {
                // double, double -> double (pow 등)
                (Some(FfiType::F64), Some(FfiType::F64), FfiType::F64) => {
                    let func: Symbol<unsafe extern "C" fn(c_double, c_double) -> c_double> =
                        library.get(fn_name.as_bytes())
                            .map_err(|e| RuntimeError::FfiError(format!("Function '{}' not found: {}", fn_name, e)))?;
                    let v1 = self.value_to_float(arg1)?;
                    let v2 = self.value_to_float(arg2)?;
                    Ok(Value::Float(func(v1, v2)))
                }
                // int, int -> int
                (Some(FfiType::I32 | FfiType::I64), Some(FfiType::I32 | FfiType::I64), FfiType::I32 | FfiType::I64) => {
                    let func: Symbol<unsafe extern "C" fn(c_int, c_int) -> c_int> =
                        library.get(fn_name.as_bytes())
                            .map_err(|e| RuntimeError::FfiError(format!("Function '{}' not found: {}", fn_name, e)))?;
                    let v1 = self.value_to_int(arg1)?;
                    let v2 = self.value_to_int(arg2)?;
                    Ok(Value::Int(func(v1 as c_int, v2 as c_int) as i64))
                }
                _ => Err(RuntimeError::FfiError(format!(
                    "Unsupported 2-arg function signature for {}",
                    fn_name
                ))),
            }
        }
    }

    /// 인자 3개 함수 호출
    fn call_3(
        &self,
        library: &Library,
        fn_name: &str,
        args: &[Value],
        _func_info: Option<&FfiFunctionInfo>,
        return_type: &FfiType,
    ) -> RuntimeResult<Value> {
        let (arg1, arg2, arg3) = (&args[0], &args[1], &args[2]);

        unsafe {
            match return_type {
                // 3개 double -> double
                FfiType::F64 => {
                    let func: Symbol<unsafe extern "C" fn(c_double, c_double, c_double) -> c_double> =
                        library.get(fn_name.as_bytes())
                            .map_err(|e| RuntimeError::FfiError(format!("Function '{}' not found: {}", fn_name, e)))?;
                    let v1 = self.value_to_float(arg1)?;
                    let v2 = self.value_to_float(arg2)?;
                    let v3 = self.value_to_float(arg3)?;
                    Ok(Value::Float(func(v1, v2, v3)))
                }
                _ => Err(RuntimeError::FfiError(format!(
                    "Unsupported 3-arg function signature for {}",
                    fn_name
                ))),
            }
        }
    }

    /// Value에서 타입 추론
    fn infer_type(&self, value: &Value) -> FfiType {
        match value {
            Value::Int(_) => FfiType::I64,
            Value::Float(_) => FfiType::F64,
            Value::Bool(_) => FfiType::Bool,
            Value::String(_) => FfiType::CStr,
            _ => FfiType::Ptr,
        }
    }

    /// Value를 i64로 변환
    fn value_to_int(&self, value: &Value) -> RuntimeResult<i64> {
        match value {
            Value::Int(n) => Ok(*n),
            Value::Float(f) => Ok(*f as i64),
            Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err(RuntimeError::TypeError(format!("Cannot convert {:?} to int", value))),
        }
    }

    /// Value를 f64로 변환
    fn value_to_float(&self, value: &Value) -> RuntimeResult<f64> {
        match value {
            Value::Float(f) => Ok(*f),
            Value::Int(n) => Ok(*n as f64),
            _ => Err(RuntimeError::TypeError(format!("Cannot convert {:?} to float", value))),
        }
    }

    /// Value를 CString으로 변환
    fn value_to_cstring(&self, value: &Value) -> RuntimeResult<CString> {
        match value {
            Value::String(s) => CString::new(s.as_str())
                .map_err(|_| RuntimeError::TypeError("String contains null byte".to_string())),
            _ => Err(RuntimeError::TypeError(format!("Cannot convert {:?} to string", value))),
        }
    }

    /// 라이브러리 언로드
    pub fn unload_library(&mut self, lib_name: &str) {
        self.libraries.remove(lib_name);
    }

    /// 모든 라이브러리 언로드
    pub fn unload_all(&mut self) {
        self.libraries.clear();
    }

    /// 로드된 라이브러리 목록
    pub fn loaded_libraries(&self) -> Vec<&str> {
        self.libraries.keys().map(|s| s.as_str()).collect()
    }

    /// 등록된 함수 목록
    pub fn registered_functions(&self) -> Vec<&str> {
        self.functions.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for FfiLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_type_from_str() {
        assert_eq!(FfiType::from_str("i32"), Some(FfiType::I32));
        assert_eq!(FfiType::from_str("f64"), Some(FfiType::F64));
        assert_eq!(FfiType::from_str("cstr"), Some(FfiType::CStr));
        assert_eq!(FfiType::from_str("void"), Some(FfiType::Void));
    }

    #[test]
    fn test_ffi_loader_new() {
        let loader = FfiLoader::new();
        assert!(loader.libraries.is_empty());
        assert!(loader.functions.is_empty());
    }

    #[test]
    fn test_register_function() {
        let mut loader = FfiLoader::new();
        loader.register_function("c", "abs", vec![FfiType::I32], FfiType::I32);

        assert!(loader.functions.contains_key("c::abs"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_load_libc() {
        let mut loader = FfiLoader::new();
        // macOS에서 libc 로드 테스트
        let result = loader.load_library("c");
        assert!(result.is_ok(), "Failed to load libc: {:?}", result);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_call_abs() {
        let mut loader = FfiLoader::new();
        loader.register_function("c", "abs", vec![FfiType::I32], FfiType::I32);

        let result = loader.call_function("c", "abs", &[Value::Int(-42)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_call_sqrt() {
        let mut loader = FfiLoader::new();
        loader.register_function("m", "sqrt", vec![FfiType::F64], FfiType::F64);

        let result = loader.call_function("m", "sqrt", &[Value::Float(16.0)]);
        assert!(result.is_ok());
        if let Value::Float(f) = result.unwrap() {
            assert!((f - 4.0).abs() < 0.0001);
        } else {
            panic!("Expected float result");
        }
    }
}
