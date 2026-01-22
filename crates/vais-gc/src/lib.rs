//! Vais Garbage Collector
//!
//! Optional Mark-and-Sweep garbage collector for Vais language.
//! Provides automatic memory management for REPL and scripting environments.
//!
//! # Design
//!
//! - **Mark-and-Sweep** algorithm for simplicity
//! - **Conservative scanning** for roots (scans stack)
//! - **Reference counting** for cycle detection (optional)
//! - **C FFI** for integration with Vais-generated LLVM code
//!
//! # Usage
//!
//! ```vais
//! #[gc]
//! F main() -> i64 {
//!     list := Vec.new()
//!     list.push(1)
//!     # No free() needed - GC handles it
//!     0
//! }
//! ```

mod allocator;
mod ffi;
mod gc;

pub use allocator::{GcAllocator, GcStats};
pub use gc::{GcObject, GcHeap, GcRoot};
pub use ffi::*;

use std::sync::{Arc, Mutex};

/// Global GC instance (lazy initialization)
static mut GLOBAL_GC: Option<Arc<Mutex<GcHeap>>> = None;

/// Initialize the global GC
pub fn init_gc() {
    unsafe {
        if GLOBAL_GC.is_none() {
            GLOBAL_GC = Some(Arc::new(Mutex::new(GcHeap::new())));
        }
    }
}

/// Get the global GC instance
pub fn get_gc() -> Arc<Mutex<GcHeap>> {
    unsafe {
        if GLOBAL_GC.is_none() {
            init_gc();
        }
        GLOBAL_GC.as_ref().unwrap().clone()
    }
}
