//! JIT runtime management for external function resolution.
//!
//! # Stage 5b-impl (DEFERRED #16, Step 17)
//!
//! This module hosts the JIT-side stdout capture infrastructure required by
//! the MIR/native asymmetry guard in `compiler/fuzz/src/lib.rs`. The
//! reconnaissance pass (commit 4fc8f937) established that JIT didn't
//! register a Vais print builtin at all — the asymmetry guard was dead.
//!
//! 5b-impl restores semantics to that guard via three layers:
//!   - `STDOUT_SINK`: thread-local `RefCell<String>` (this file, 5b-1)
//!   - `vais_runtime_print(ptr, len)` extern "C" thunk (this file, 5b-1)
//!   - `register_stdlib` registers the thunk under the same symbol the
//!     JIT codegen emits (5b-2)
//!   - JIT codegen lowers Vais print/println/print_int builtin calls to
//!     indirect calls into the thunk (5b-3, separate file)
//!   - `compiler/fuzz/src/lib.rs::run_native_path` reads `STDOUT_SINK::take()`
//!     to populate `RunOutput.stdout`; asymmetry guard removed (5b-4)

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// JIT-execution stdout capture buffer. Each thread running JIT-emitted
    /// code accumulates print output here; tests / fuzz drivers call
    /// `take_stdout_sink()` after the run to retrieve and reset.
    ///
    /// 5b-1: dead until 5b-3 wires `vais_runtime_print` calls into emitted
    /// JIT code.
    static STDOUT_SINK: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Take the accumulated JIT-execution stdout buffer for the current thread,
/// resetting it to empty. Used by fuzz drivers and unit tests that compare
/// MIR-interpreted vs JIT-compiled execution side-by-side.
///
/// 5b-1: dead until 5b-4 wires this into the fuzz path.
#[allow(dead_code)]
pub fn take_stdout_sink() -> String {
    STDOUT_SINK.with(|sink| std::mem::take(&mut *sink.borrow_mut()))
}

/// `vais_runtime_print(ptr, len)` — JIT-callable thunk that appends a UTF-8
/// byte slice to the thread-local STDOUT_SINK. Matches the C ABI the JIT
/// codegen emits for Vais print/println/print_int builtins.
///
/// 5b-1: dead until 5b-2 registers it in `JitRuntime::register_stdlib` and
/// 5b-3 emits indirect calls.
///
/// # Safety
///
/// `ptr` must be a valid pointer to `len` bytes of UTF-8-encoded data.
/// `len` must be non-negative; behavior is undefined on negative inputs.
/// Caller (JIT codegen) is responsible for ensuring `ptr`+`len` lifetime
/// covers the call.
#[allow(dead_code)]
#[no_mangle]
pub unsafe extern "C" fn vais_runtime_print(ptr: *const u8, len: i64) {
    if ptr.is_null() || len <= 0 {
        return;
    }
    // SAFETY: caller asserts `ptr`..`ptr+len` is a valid UTF-8 byte slice
    // for the duration of this call.
    let slice = std::slice::from_raw_parts(ptr, len as usize);
    let s = match std::str::from_utf8(slice) {
        Ok(s) => s,
        Err(_) => return, // silently drop non-UTF-8 (matches Vais spec contract)
    };
    STDOUT_SINK.with(|sink| sink.borrow_mut().push_str(s));
}

/// JIT runtime for managing external function pointers.
pub struct JitRuntime {
    /// Map of function name to function pointer.
    functions: HashMap<String, *const u8>,
}

impl JitRuntime {
    /// Creates a new JIT runtime with standard library functions.
    pub fn new() -> Self {
        let mut runtime = Self {
            functions: HashMap::new(),
        };
        runtime.register_stdlib();
        runtime
    }

    /// Registers standard library functions.
    fn register_stdlib(&mut self) {
        // I/O functions
        self.register("puts", libc::puts as *const u8);
        self.register("printf", libc::printf as *const u8);
        self.register("putchar", libc::putchar as *const u8);
        self.register("getchar", libc::getchar as *const u8);

        // Memory functions
        self.register("malloc", libc::malloc as *const u8);
        self.register("free", libc::free as *const u8);
        self.register("memcpy", libc::memcpy as *const u8);
        self.register("memset", libc::memset as *const u8);
        self.register("strlen", libc::strlen as *const u8);

        // Process functions
        self.register("exit", libc::exit as *const u8);
        self.register("abort", libc::abort as *const u8);

        // Math functions (from libm)
        self.register("sqrt", libm::sqrt as *const u8);
        self.register("sin", libm::sin as *const u8);
        self.register("cos", libm::cos as *const u8);
        self.register("tan", libm::tan as *const u8);
        self.register("floor", libm::floor as *const u8);
        self.register("ceil", libm::ceil as *const u8);
        self.register("round", libm::round as *const u8);
        self.register("fabs", libm::fabs as *const u8);
        self.register("pow", libm::pow as *const u8);
        self.register("log", libm::log as *const u8);
        self.register("log10", libm::log10 as *const u8);
        self.register("log2", libm::log2 as *const u8);
        self.register("exp", libm::exp as *const u8);
    }

    /// Registers a function pointer by name.
    pub fn register(&mut self, name: &str, ptr: *const u8) {
        self.functions.insert(name.to_string(), ptr);
    }

    /// Looks up a function pointer by name.
    pub fn lookup(&self, name: &str) -> Option<*const u8> {
        self.functions.get(name).copied()
    }

    /// Returns all registered function names.
    pub fn registered_functions(&self) -> Vec<&str> {
        self.functions.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for JitRuntime {
    fn default() -> Self {
        Self::new()
    }
}

// Libm bindings for math functions
mod libm {
    extern "C" {
        pub fn sqrt(x: f64) -> f64;
        pub fn sin(x: f64) -> f64;
        pub fn cos(x: f64) -> f64;
        pub fn tan(x: f64) -> f64;
        pub fn floor(x: f64) -> f64;
        pub fn ceil(x: f64) -> f64;
        pub fn round(x: f64) -> f64;
        pub fn fabs(x: f64) -> f64;
        pub fn pow(x: f64, y: f64) -> f64;
        pub fn log(x: f64) -> f64;
        pub fn log10(x: f64) -> f64;
        pub fn log2(x: f64) -> f64;
        pub fn exp(x: f64) -> f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = JitRuntime::new();
        assert!(runtime.lookup("puts").is_some());
        assert!(runtime.lookup("malloc").is_some());
        assert!(runtime.lookup("sqrt").is_some());
    }

    #[test]
    fn test_custom_registration() {
        extern "C" fn dummy_func() {}

        let mut runtime = JitRuntime::new();
        runtime.register("my_func", dummy_func as *const u8);
        assert!(runtime.lookup("my_func").is_some());
    }

    #[test]
    fn test_unknown_function() {
        let runtime = JitRuntime::new();
        assert!(runtime.lookup("unknown_function_xyz").is_none());
    }

    #[test]
    fn test_registered_functions_list() {
        let runtime = JitRuntime::new();
        let funcs = runtime.registered_functions();

        // Should contain I/O functions
        assert!(funcs.contains(&"puts"));
        assert!(funcs.contains(&"printf"));

        // Should contain memory functions
        assert!(funcs.contains(&"malloc"));
        assert!(funcs.contains(&"free"));

        // Should contain math functions
        assert!(funcs.contains(&"sqrt"));
        assert!(funcs.contains(&"sin"));
    }

    #[test]
    fn test_multiple_custom_registrations() {
        extern "C" fn func_a() {}
        extern "C" fn func_b() {}

        let mut runtime = JitRuntime::new();
        runtime.register("func_a", func_a as *const u8);
        runtime.register("func_b", func_b as *const u8);

        assert!(runtime.lookup("func_a").is_some());
        assert!(runtime.lookup("func_b").is_some());
        assert_ne!(runtime.lookup("func_a"), runtime.lookup("func_b"));
    }

    #[test]
    fn test_overwrite_registration() {
        extern "C" fn original() {}
        extern "C" fn replacement() {}

        let mut runtime = JitRuntime::new();
        runtime.register("test_func", original as *const u8);
        let first_ptr = runtime.lookup("test_func").unwrap();

        runtime.register("test_func", replacement as *const u8);
        let second_ptr = runtime.lookup("test_func").unwrap();

        assert_ne!(first_ptr, second_ptr);
    }

    #[test]
    fn test_default_runtime() {
        let runtime = JitRuntime::default();
        assert!(runtime.lookup("malloc").is_some());
        assert!(runtime.lookup("sqrt").is_some());
    }
}
