//! Built-in function and type registration for the type checker.

mod core;
mod print;
mod memory;
mod stdlib;
mod file_io;
mod simd;
mod gc;
mod system;
mod io;
mod math;
mod enum_builtins;

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
