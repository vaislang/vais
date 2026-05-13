//! Built-in function and type registration for the type checker.

mod core;
mod enum_builtins;
mod file_io;
mod gc;
mod io;
mod math;
mod memory;
mod print;
mod simd;
mod stdlib;
mod system;

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
}
