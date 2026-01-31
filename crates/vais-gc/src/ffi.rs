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
    let mut heap = gc.lock().expect("GC heap mutex poisoned in vais_gc_alloc");
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
    let mut heap = gc.lock().expect("GC heap mutex poisoned in vais_gc_add_root");
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
    let mut heap = gc.lock().expect("GC heap mutex poisoned in vais_gc_remove_root");
    heap.remove_root(ptr);
}

/// Force garbage collection
#[no_mangle]
pub extern "C" fn vais_gc_collect() {
    let gc = get_gc();
    let mut heap = gc.lock().expect("GC heap mutex poisoned in vais_gc_collect");
    heap.collect();
}

/// Get bytes currently allocated
#[no_mangle]
pub extern "C" fn vais_gc_bytes_allocated() -> usize {
    let gc = get_gc();
    let heap = gc.lock().expect("GC heap mutex poisoned in vais_gc_bytes_allocated");
    heap.stats().bytes_allocated
}

/// Get number of GC objects
#[no_mangle]
pub extern "C" fn vais_gc_objects_count() -> usize {
    let gc = get_gc();
    let heap = gc.lock().expect("GC heap mutex poisoned in vais_gc_objects_count");
    heap.stats().objects_count
}

/// Get number of collections performed
#[no_mangle]
pub extern "C" fn vais_gc_collections() -> usize {
    let gc = get_gc();
    let heap = gc.lock().expect("GC heap mutex poisoned in vais_gc_collections");
    heap.stats().collections
}

/// Set GC threshold (bytes allocated before triggering collection)
#[no_mangle]
pub extern "C" fn vais_gc_set_threshold(threshold: usize) {
    let gc = get_gc();
    let mut heap = gc.lock().expect("GC heap mutex poisoned in vais_gc_set_threshold");
    heap.set_threshold(threshold);
}

/// Print GC statistics (for debugging)
#[no_mangle]
pub extern "C" fn vais_gc_print_stats() {
    let gc = get_gc();
    let heap = gc.lock().expect("GC heap mutex poisoned in vais_gc_print_stats");
    let stats = heap.stats();

    println!("GC Statistics:");
    println!("  Bytes allocated: {} bytes", stats.bytes_allocated);
    println!("  Objects count: {}", stats.objects_count);
    println!("  Collections: {}", stats.collections);
    println!("  Objects freed: {}", stats.objects_freed);
    println!("  GC threshold: {} bytes", stats.gc_threshold);
}

// ============================================
// Generational GC FFI
// ============================================

use crate::generational::GenerationalGc;
use std::sync::OnceLock;

static GLOBAL_GEN_GC: OnceLock<std::sync::Arc<std::sync::Mutex<GenerationalGc>>> = OnceLock::new();

fn get_gen_gc() -> std::sync::Arc<std::sync::Mutex<GenerationalGc>> {
    GLOBAL_GEN_GC
        .get_or_init(|| std::sync::Arc::new(std::sync::Mutex::new(GenerationalGc::new())))
        .clone()
}

/// Initialize generational GC
#[no_mangle]
pub extern "C" fn vais_gen_gc_init() {
    let _ = get_gen_gc();
}

/// Allocate memory via generational GC (new objects go to young generation)
#[no_mangle]
pub extern "C" fn vais_gen_gc_alloc(size: usize, type_id: u32) -> *mut c_void {
    let gc = get_gen_gc();
    let mut heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.alloc(size, type_id) as *mut c_void
}

/// Register a root pointer
#[no_mangle]
pub extern "C" fn vais_gen_gc_add_root(ptr: usize) {
    if ptr == 0 {
        return;
    }
    let gc = get_gen_gc();
    let mut heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.add_root(ptr);
}

/// Unregister a root pointer
#[no_mangle]
pub extern "C" fn vais_gen_gc_remove_root(ptr: usize) {
    if ptr == 0 {
        return;
    }
    let gc = get_gen_gc();
    let mut heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.remove_root(ptr);
}

/// Write barrier for generational GC
#[no_mangle]
pub extern "C" fn vais_gen_gc_write_barrier(source: usize, old_target: usize, new_target: usize) {
    let gc = get_gen_gc();
    let mut heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.write_barrier(source, old_target, new_target);
}

/// Force minor GC (young generation only)
#[no_mangle]
pub extern "C" fn vais_gen_gc_collect_minor() {
    let gc = get_gen_gc();
    let mut heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.collect_minor();
}

/// Force major GC (both generations)
#[no_mangle]
pub extern "C" fn vais_gen_gc_collect_major() {
    let gc = get_gen_gc();
    let mut heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.collect_major();
}

/// Force full GC (minor + major)
#[no_mangle]
pub extern "C" fn vais_gen_gc_collect_full() {
    let gc = get_gen_gc();
    let mut heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.collect_full();
}

/// Get number of young generation objects
#[no_mangle]
pub extern "C" fn vais_gen_gc_young_objects() -> usize {
    let gc = get_gen_gc();
    let heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.get_stats().young_objects
}

/// Get number of old generation objects
#[no_mangle]
pub extern "C" fn vais_gen_gc_old_objects() -> usize {
    let gc = get_gen_gc();
    let heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.get_stats().old_objects
}

/// Get number of minor collections
#[no_mangle]
pub extern "C" fn vais_gen_gc_minor_collections() -> u64 {
    let gc = get_gen_gc();
    let heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.get_stats().minor_collections
}

/// Get number of major collections
#[no_mangle]
pub extern "C" fn vais_gen_gc_major_collections() -> u64 {
    let gc = get_gen_gc();
    let heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.get_stats().major_collections
}

/// Get total number of promoted objects
#[no_mangle]
pub extern "C" fn vais_gen_gc_total_promoted() -> u64 {
    let gc = get_gen_gc();
    let heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.get_stats().total_promoted
}

/// Set young generation threshold
#[no_mangle]
pub extern "C" fn vais_gen_gc_set_young_threshold(threshold: usize) {
    let gc = get_gen_gc();
    let mut heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.set_young_threshold(threshold);
}

/// Set old generation threshold
#[no_mangle]
pub extern "C" fn vais_gen_gc_set_old_threshold(threshold: usize) {
    let gc = get_gen_gc();
    let mut heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.set_old_threshold(threshold);
}

/// Set promotion age (number of minor GCs before promotion)
#[no_mangle]
pub extern "C" fn vais_gen_gc_set_promotion_age(age: u8) {
    let gc = get_gen_gc();
    let mut heap = gc.lock().expect("Gen GC mutex poisoned");
    heap.set_promotion_age(age);
}

/// Print generational GC statistics
#[no_mangle]
pub extern "C" fn vais_gen_gc_print_stats() {
    let gc = get_gen_gc();
    let heap = gc.lock().expect("Gen GC mutex poisoned");
    let stats = heap.get_stats();

    println!("Generational GC Statistics:");
    println!("  Young generation:");
    println!("    Objects: {}", stats.young_objects);
    println!("    Bytes: {} bytes", stats.young_bytes);
    println!("  Old generation:");
    println!("    Objects: {}", stats.old_objects);
    println!("    Bytes: {} bytes", stats.old_bytes);
    println!("  Collections:");
    println!("    Minor: {}", stats.minor_collections);
    println!("    Major: {}", stats.major_collections);
    println!("  Promoted: {}", stats.total_promoted);
    println!("  Remembered set size: {}", stats.remembered_set_size);
}
