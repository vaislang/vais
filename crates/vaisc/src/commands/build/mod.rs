//! Build commands and GPU compilation.

// Submodules
mod backend;
mod core;
mod gpu;
mod utils;

// Re-export public functions
pub(crate) use backend::generate_with_text_backend;
pub(crate) use core::{cmd_build, cmd_build_with_timing};
pub(crate) use gpu::cmd_build_gpu;
pub(crate) use utils::find_std_dir;
