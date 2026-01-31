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
//! # Concurrent GC
//!
//! The concurrent garbage collector minimizes pause times by performing
//! most GC work concurrently with mutator threads:
//!
//! - **Tri-color marking**: White (unvisited), Gray (to scan), Black (scanned)
//! - **Write barrier**: Maintains invariant during concurrent marking
//! - **Incremental collection**: Work can be divided into smaller steps
//! - **Concurrent sweep**: Sweep phase runs in background thread
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
mod concurrent;
mod ffi;
mod gc;
mod generational;

pub use allocator::{GcAllocator, GcStats};
pub use concurrent::{
    default_concurrent_gc, Color, ConcurrentGc, ConcurrentGcConfig, ConcurrentGcHeader,
    ConcurrentGcObject, ConcurrentGcStats, GcPhase, GcWorker, IncrementalGc, WriteBarrierEntry,
};
pub use ffi::*;
pub use gc::{GcHeap, GcObject, GcRoot};
pub use generational::{
    CardTable, CollectionType, GenGcConfig, GenGcObject, GenGcObjectHeader, GenGcStats,
    Generation, GenerationalGc, RememberedSet,
};

use std::sync::{Arc, Mutex, OnceLock};

/// Global GC instance (thread-safe lazy initialization using OnceLock)
static GLOBAL_GC: OnceLock<Arc<Mutex<GcHeap>>> = OnceLock::new();

/// Initialize the global GC
///
/// This function is thread-safe and idempotent - it can be called multiple times
/// but the GC will only be initialized once.
pub fn init_gc() {
    // OnceLock::get_or_init is thread-safe and ensures single initialization
    GLOBAL_GC.get_or_init(|| Arc::new(Mutex::new(GcHeap::new())));
}

/// Get the global GC instance
///
/// This function will initialize the GC if it hasn't been initialized yet.
/// It is thread-safe and will never panic.
pub fn get_gc() -> Arc<Mutex<GcHeap>> {
    GLOBAL_GC
        .get_or_init(|| Arc::new(Mutex::new(GcHeap::new())))
        .clone()
}
