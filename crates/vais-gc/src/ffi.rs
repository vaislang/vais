//! C FFI for Vais integration
//!
//! Exports GC functions for use from Vais-generated LLVM code

use crate::{get_gc, init_gc};
use std::os::raw::c_void;

/// Initialize GC (call once at program start)
#[no_mangle]
pub extern "C" fn vais_gc_init() {
    init_gc();
}

/// Allocate GC-managed memory
///
/// # Arguments
/// * `size` - Size in bytes to allocate
/// * `type_id` - Type identifier for debugging (0 for unknown)
///
/// # Returns
/// Pointer to allocated memory (never null - panics on OOM)
#[no_mangle]
pub extern "C" fn vais_gc_alloc(size: usize, type_id: u32) -> *mut c_void {
    let gc = get_gc();
    let mut heap = gc.lock().unwrap();
    heap.alloc(size, type_id) as *mut c_void
}

/// Register a root pointer (e.g., local variable)
///
/// Call this when a GC pointer is stored on the stack or in global data
#[no_mangle]
pub extern "C" fn vais_gc_add_root(ptr: usize) {
    if ptr == 0 {
        return;
    }
    let gc = get_gc();
    let mut heap = gc.lock().unwrap();
    heap.add_root(ptr);
}

/// Unregister a root pointer
///
/// Call this when a local variable goes out of scope
#[no_mangle]
pub extern "C" fn vais_gc_remove_root(ptr: usize) {
    if ptr == 0 {
        return;
    }
    let gc = get_gc();
    let mut heap = gc.lock().unwrap();
    heap.remove_root(ptr);
}

/// Force garbage collection
#[no_mangle]
pub extern "C" fn vais_gc_collect() {
    let gc = get_gc();
    let mut heap = gc.lock().unwrap();
    heap.collect();
}

/// Get bytes currently allocated
#[no_mangle]
pub extern "C" fn vais_gc_bytes_allocated() -> usize {
    let gc = get_gc();
    let heap = gc.lock().unwrap();
    heap.stats().bytes_allocated
}

/// Get number of GC objects
#[no_mangle]
pub extern "C" fn vais_gc_objects_count() -> usize {
    let gc = get_gc();
    let heap = gc.lock().unwrap();
    heap.stats().objects_count
}

/// Get number of collections performed
#[no_mangle]
pub extern "C" fn vais_gc_collections() -> usize {
    let gc = get_gc();
    let heap = gc.lock().unwrap();
    heap.stats().collections
}

/// Set GC threshold (bytes allocated before triggering collection)
#[no_mangle]
pub extern "C" fn vais_gc_set_threshold(threshold: usize) {
    let gc = get_gc();
    let mut heap = gc.lock().unwrap();
    heap.set_threshold(threshold);
}

/// Print GC statistics (for debugging)
#[no_mangle]
pub extern "C" fn vais_gc_print_stats() {
    let gc = get_gc();
    let heap = gc.lock().unwrap();
    let stats = heap.stats();

    println!("GC Statistics:");
    println!("  Bytes allocated: {} bytes", stats.bytes_allocated);
    println!("  Objects count: {}", stats.objects_count);
    println!("  Collections: {}", stats.collections);
    println!("  Objects freed: {}", stats.objects_freed);
    println!("  GC threshold: {} bytes", stats.gc_threshold);
}
