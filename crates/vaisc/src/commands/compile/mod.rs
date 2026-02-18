//! Compilation functions for different targets.

use crate::incremental;
use crate::package;
use crate::runtime::{
    find_gc_library, find_http_runtime, find_runtime_file, find_sync_runtime, find_thread_runtime,
    get_runtime_for_module,
};
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use vais_codegen::TargetTriple;
use vais_types::TypeChecker;

// Submodules
mod per_module;
mod parallel;
mod pipeline;
mod native;
mod wasm;

// Re-export public functions
pub(crate) use per_module::compile_per_module;
pub(crate) use native::compile_to_native;
pub(crate) use wasm::{compile_to_wasm32, compile_to_wasi};

/// Route IR compilation to the appropriate backend based on target triple.
#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_ir_to_binary(
    ir_path: &Path,
    bin_path: &Path,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    target: &TargetTriple,
    hot: bool,
    lto_mode: &vais_codegen::optimize::LtoMode,
    pgo_mode: &vais_codegen::optimize::PgoMode,
    coverage_mode: &vais_codegen::optimize::CoverageMode,
    used_modules: &HashSet<String>,
    native_deps: &HashMap<String, package::NativeDependency>,
    obj_cache_dir: Option<&Path>,
) -> Result<(), String> {
    match target {
        TargetTriple::Wasm32Unknown => compile_to_wasm32(ir_path, bin_path, opt_level, verbose),
        TargetTriple::WasiPreview1 | TargetTriple::WasiPreview2 => {
            compile_to_wasi(ir_path, bin_path, opt_level, verbose)
        }
        _ => compile_to_native(
            ir_path,
            bin_path,
            opt_level,
            debug,
            verbose,
            hot,
            lto_mode,
            pgo_mode,
            coverage_mode,
            used_modules,
            native_deps,
            obj_cache_dir,
        ),
    }
}
