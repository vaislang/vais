//! JIT runtime management for external function resolution.

use std::collections::HashMap;

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
}
