//! Concurrent/Incremental Garbage Collector
//!
//! Implements a concurrent mark-sweep garbage collector that minimizes
//! pause times by performing most GC work concurrently with mutator threads.
//!
//! # Design
//!
//! - **Tri-color marking**: White (unvisited), Gray (to scan), Black (scanned)
//! - **Write barrier**: Detects pointer modifications during concurrent marking
//! - **Incremental collection**: Work can be divided into smaller steps
//! - **Concurrent sweep**: Sweep phase runs in background thread
//!
//! # Phases
//!
//! 1. **Initial Mark** (STW): Mark roots, very brief pause
//! 2. **Concurrent Mark**: Trace object graph concurrently
//! 3. **Remark** (STW): Process write barrier, brief pause
//! 4. **Concurrent Sweep**: Free unmarked objects in background

mod barrier;
mod mark;
mod sweep;
mod worker;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};

/// Tri-color marking states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Not yet visited by marking.
    White,
    /// Marked but children not yet scanned.
    Gray,
    /// Fully scanned, all children marked.
    Black,
}

/// Object header for concurrent GC.
#[repr(C)]
#[derive(Debug)]
pub struct ConcurrentGcHeader {
    /// Size of object data in bytes.
    pub size: AtomicUsize,
    /// Current color in tri-color marking.
    pub color: RwLock<Color>,
    /// Type ID for debugging.
    pub type_id: u32,
    /// Generation (for generational GC extension).
    pub generation: u8,
    /// Is this object pinned (cannot be moved)?
    pub pinned: AtomicBool,
}

impl ConcurrentGcHeader {
    pub fn new(size: usize, type_id: u32) -> Self {
        Self {
            size: AtomicUsize::new(size),
            color: RwLock::new(Color::White),
            type_id,
            generation: 0,
            pinned: AtomicBool::new(false),
        }
    }

    pub fn get_color(&self) -> Color {
        *self.color.read().unwrap()
    }

    pub fn set_color(&self, color: Color) {
        *self.color.write().unwrap() = color;
    }

    pub fn compare_and_set_color(&self, expected: Color, new: Color) -> bool {
        let mut guard = self.color.write().unwrap();
        if *guard == expected {
            *guard = new;
            true
        } else {
            false
        }
    }
}

/// Concurrent GC object.
pub struct ConcurrentGcObject {
    pub header: ConcurrentGcHeader,
    pub data: Vec<u8>,
}

impl ConcurrentGcObject {
    pub fn new(size: usize, type_id: u32) -> Self {
        Self {
            header: ConcurrentGcHeader::new(size, type_id),
            data: vec![0u8; size],
        }
    }

    pub fn data_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn data_ptr_mut(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }
}

/// Write barrier entry for tracking pointer modifications.
#[derive(Debug, Clone)]
pub struct WriteBarrierEntry {
    /// Source object that contains the modified pointer.
    pub source: usize,
    /// Old pointer value before modification.
    pub old_target: usize,
    /// New pointer value after modification.
    pub new_target: usize,
    /// Timestamp of the modification.
    pub timestamp: u64,
}

/// GC phase for concurrent collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcPhase {
    /// Idle, no collection in progress.
    Idle,
    /// Initial mark (stop-the-world).
    InitialMark,
    /// Concurrent marking.
    ConcurrentMark,
    /// Remark phase (stop-the-world).
    Remark,
    /// Concurrent sweep.
    ConcurrentSweep,
}

/// Statistics for concurrent GC.
#[derive(Debug, Clone, Default)]
pub struct ConcurrentGcStats {
    /// Total collections performed.
    pub collections: u64,
    /// Total bytes allocated.
    pub bytes_allocated: usize,
    /// Total objects alive.
    pub objects_count: usize,
    /// Objects freed in last collection.
    pub last_freed: usize,
    /// Bytes freed in last collection.
    pub last_bytes_freed: usize,
    /// Total STW pause time (nanoseconds).
    pub total_pause_time_ns: u64,
    /// Max STW pause time (nanoseconds).
    pub max_pause_time_ns: u64,
    /// Write barrier entries processed.
    pub write_barriers_processed: u64,
    /// Number of marking steps performed.
    pub marking_steps: u64,
}

/// Configuration for concurrent GC.
#[derive(Debug, Clone)]
pub struct ConcurrentGcConfig {
    /// Threshold for triggering collection (bytes).
    pub gc_threshold: usize,
    /// Target pause time (nanoseconds).
    pub target_pause_ns: u64,
    /// Maximum marking steps per incremental batch.
    pub max_marking_steps: usize,
    /// Enable concurrent sweep.
    pub concurrent_sweep: bool,
    /// Enable write barrier.
    pub write_barrier: bool,
}

impl Default for ConcurrentGcConfig {
    fn default() -> Self {
        Self {
            gc_threshold: 1024 * 1024,  // 1 MB
            target_pause_ns: 1_000_000, // 1 ms
            max_marking_steps: 1000,
            concurrent_sweep: true,
            write_barrier: true,
        }
    }
}

/// Concurrent Garbage Collector.
pub struct ConcurrentGc {
    /// All allocated objects (ptr -> object).
    pub(crate) objects: RwLock<HashMap<usize, ConcurrentGcObject>>,
    /// Root set.
    pub(crate) roots: RwLock<HashSet<usize>>,
    /// Gray set (objects to scan).
    pub(crate) gray_set: Mutex<VecDeque<usize>>,
    /// Write barrier buffer.
    pub(crate) write_barrier_buffer: Mutex<Vec<WriteBarrierEntry>>,
    /// Current GC phase.
    pub(crate) phase: RwLock<GcPhase>,
    /// Statistics.
    pub(crate) stats: RwLock<ConcurrentGcStats>,
    /// Configuration.
    pub(crate) config: ConcurrentGcConfig,
    /// Bytes allocated since last GC.
    pub(crate) bytes_since_gc: AtomicUsize,
    /// Global timestamp counter for write barriers.
    pub(crate) timestamp: AtomicU64,
    /// Flag to stop background threads.
    pub(crate) shutdown: AtomicBool,
    /// Condition variable for GC thread coordination.
    pub(crate) gc_condvar: Condvar,
    /// Mutex for condition variable.
    pub(crate) gc_mutex: Mutex<()>,
}

impl ConcurrentGc {
    /// Creates a new concurrent GC with default configuration.
    pub fn new() -> Arc<Self> {
        Self::with_config(ConcurrentGcConfig::default())
    }

    /// Creates a new concurrent GC with custom configuration.
    pub fn with_config(config: ConcurrentGcConfig) -> Arc<Self> {
        Arc::new(Self {
            objects: RwLock::new(HashMap::new()),
            roots: RwLock::new(HashSet::new()),
            gray_set: Mutex::new(VecDeque::new()),
            write_barrier_buffer: Mutex::new(Vec::new()),
            phase: RwLock::new(GcPhase::Idle),
            stats: RwLock::new(ConcurrentGcStats::default()),
            config,
            bytes_since_gc: AtomicUsize::new(0),
            timestamp: AtomicU64::new(0),
            shutdown: AtomicBool::new(false),
            gc_condvar: Condvar::new(),
            gc_mutex: Mutex::new(()),
        })
    }

    /// Allocates a new object.
    pub fn alloc(&self, size: usize, type_id: u32) -> *mut u8 {
        // Check if we should trigger GC
        let bytes = self.bytes_since_gc.fetch_add(size, Ordering::Relaxed) + size;
        if bytes >= self.config.gc_threshold {
            self.request_collection();
        }

        let mut obj = ConcurrentGcObject::new(size, type_id);
        let ptr = obj.data_ptr_mut() as usize;

        {
            let mut objects = self.objects.write().unwrap();
            objects.insert(ptr, obj);
        }

        {
            let mut stats = self.stats.write().unwrap();
            stats.bytes_allocated += size;
            stats.objects_count += 1;
        }

        ptr as *mut u8
    }

    /// Adds a root pointer.
    pub fn add_root(&self, ptr: usize) {
        if ptr != 0 {
            let mut roots = self.roots.write().unwrap();
            roots.insert(ptr);
        }
    }

    /// Removes a root pointer.
    pub fn remove_root(&self, ptr: usize) {
        let mut roots = self.roots.write().unwrap();
        roots.remove(&ptr);
    }

    /// Gets the current GC phase.
    pub fn get_phase(&self) -> GcPhase {
        *self.phase.read().unwrap()
    }

    /// Gets GC statistics.
    pub fn get_stats(&self) -> ConcurrentGcStats {
        self.stats.read().unwrap().clone()
    }

    /// Shuts down the GC.
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.gc_condvar.notify_all();
    }

    /// Checks if an object is alive (for debugging).
    pub fn is_alive(&self, ptr: usize) -> bool {
        let objects = self.objects.read().unwrap();
        objects.contains_key(&ptr)
    }

    /// Gets the number of live objects.
    pub fn object_count(&self) -> usize {
        self.objects.read().unwrap().len()
    }
}

/// Creates a new default `Arc<ConcurrentGc>`.
pub fn default_concurrent_gc() -> Arc<ConcurrentGc> {
    ConcurrentGc::new()
}

impl Drop for ConcurrentGc {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

// Re-export worker types
pub use worker::{GcWorker, IncrementalGc};
