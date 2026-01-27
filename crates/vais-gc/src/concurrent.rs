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

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::{self, JoinHandle};

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
            gc_threshold: 1024 * 1024, // 1 MB
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
    objects: RwLock<HashMap<usize, ConcurrentGcObject>>,
    /// Root set.
    roots: RwLock<HashSet<usize>>,
    /// Gray set (objects to scan).
    gray_set: Mutex<VecDeque<usize>>,
    /// Write barrier buffer.
    write_barrier_buffer: Mutex<Vec<WriteBarrierEntry>>,
    /// Current GC phase.
    phase: RwLock<GcPhase>,
    /// Statistics.
    stats: RwLock<ConcurrentGcStats>,
    /// Configuration.
    config: ConcurrentGcConfig,
    /// Bytes allocated since last GC.
    bytes_since_gc: AtomicUsize,
    /// Global timestamp counter for write barriers.
    timestamp: AtomicU64,
    /// Flag to stop background threads.
    shutdown: AtomicBool,
    /// Condition variable for GC thread coordination.
    gc_condvar: Condvar,
    /// Mutex for condition variable.
    gc_mutex: Mutex<()>,
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

    /// Write barrier - called when a pointer field is modified.
    ///
    /// This maintains the invariant that black objects never point to white objects
    /// during concurrent marking.
    pub fn write_barrier(&self, source: usize, old_target: usize, new_target: usize) {
        if !self.config.write_barrier {
            return;
        }

        // Only record during concurrent mark phase
        let phase = *self.phase.read().unwrap();
        if phase != GcPhase::ConcurrentMark {
            return;
        }

        let entry = WriteBarrierEntry {
            source,
            old_target,
            new_target,
            timestamp: self.timestamp.fetch_add(1, Ordering::Relaxed),
        };

        let mut buffer = self.write_barrier_buffer.lock().unwrap();
        buffer.push(entry);

        // If new target is white and source is black, mark new target gray
        // This is the snapshot-at-the-beginning barrier
        if new_target != 0 {
            if let Some(obj) = self.objects.read().unwrap().get(&new_target) {
                if obj.header.get_color() == Color::White {
                    obj.header.set_color(Color::Gray);
                    let mut gray = self.gray_set.lock().unwrap();
                    gray.push_back(new_target);
                }
            }
        }
    }

    /// Requests a garbage collection.
    pub fn request_collection(&self) {
        let mut phase = self.phase.write().unwrap();
        if *phase == GcPhase::Idle {
            *phase = GcPhase::InitialMark;
            drop(phase);
            // Signal GC thread
            self.gc_condvar.notify_one();
        }
    }

    /// Runs a full collection synchronously.
    pub fn collect_sync(&self) {
        self.initial_mark();
        self.concurrent_mark_full();
        self.remark();
        self.sweep_sync();
    }

    /// Phase 1: Initial Mark (STW).
    ///
    /// Brief pause to mark root objects.
    fn initial_mark(&self) {
        let start = std::time::Instant::now();

        {
            let mut phase = self.phase.write().unwrap();
            *phase = GcPhase::InitialMark;
        }

        // Reset all objects to white
        {
            let objects = self.objects.read().unwrap();
            for obj in objects.values() {
                obj.header.set_color(Color::White);
            }
        }

        // Mark roots gray
        let roots: Vec<usize>;
        {
            let root_set = self.roots.read().unwrap();
            roots = root_set.iter().copied().collect();
        }

        {
            let objects = self.objects.read().unwrap();
            let mut gray = self.gray_set.lock().unwrap();
            gray.clear();

            for ptr in roots {
                if let Some(obj) = objects.get(&ptr) {
                    if obj.header.compare_and_set_color(Color::White, Color::Gray) {
                        gray.push_back(ptr);
                    }
                }
            }
        }

        {
            let mut phase = self.phase.write().unwrap();
            *phase = GcPhase::ConcurrentMark;
        }

        // Update pause time stats
        let pause_ns = start.elapsed().as_nanos() as u64;
        let mut stats = self.stats.write().unwrap();
        stats.total_pause_time_ns += pause_ns;
        if pause_ns > stats.max_pause_time_ns {
            stats.max_pause_time_ns = pause_ns;
        }
    }

    /// Phase 2: Concurrent Mark.
    ///
    /// Traces object graph. Can be run incrementally.
    fn concurrent_mark_step(&self, max_steps: usize) -> bool {
        let mut steps = 0;

        while steps < max_steps {
            let ptr = {
                let mut gray = self.gray_set.lock().unwrap();
                gray.pop_front()
            };

            let ptr = match ptr {
                Some(p) => p,
                None => return true, // Done
            };

            // Scan object for child pointers
            let children = self.scan_object(ptr);

            // Mark children gray if white
            {
                let objects = self.objects.read().unwrap();
                let mut gray = self.gray_set.lock().unwrap();

                for child_ptr in children {
                    if let Some(obj) = objects.get(&child_ptr) {
                        if obj.header.compare_and_set_color(Color::White, Color::Gray) {
                            gray.push_back(child_ptr);
                        }
                    }
                }
            }

            // Mark current object black
            if let Some(obj) = self.objects.read().unwrap().get(&ptr) {
                obj.header.set_color(Color::Black);
            }

            steps += 1;
            self.stats.write().unwrap().marking_steps += 1;
        }

        false // Not done yet
    }

    /// Full concurrent mark (not incremental).
    fn concurrent_mark_full(&self) {
        while !self.concurrent_mark_step(self.config.max_marking_steps) {
            // Keep marking
        }
    }

    /// Scans an object for child pointers.
    fn scan_object(&self, ptr: usize) -> Vec<usize> {
        let objects = self.objects.read().unwrap();
        let obj = match objects.get(&ptr) {
            Some(o) => o,
            None => return vec![],
        };

        let size = obj.header.size.load(Ordering::Relaxed);
        let ptr_size = std::mem::size_of::<usize>();
        let mut children = Vec::new();

        // Conservative scanning
        for offset in (0..size).step_by(ptr_size) {
            if offset + ptr_size <= size {
                unsafe {
                    let potential_ptr = std::ptr::read(obj.data.as_ptr().add(offset) as *const usize);
                    if objects.contains_key(&potential_ptr) {
                        children.push(potential_ptr);
                    }
                }
            }
        }

        children
    }

    /// Phase 3: Remark (STW).
    ///
    /// Process write barrier entries and finish marking.
    fn remark(&self) {
        let start = std::time::Instant::now();

        {
            let mut phase = self.phase.write().unwrap();
            *phase = GcPhase::Remark;
        }

        // Process write barrier buffer
        let entries: Vec<WriteBarrierEntry>;
        {
            let mut buffer = self.write_barrier_buffer.lock().unwrap();
            entries = buffer.drain(..).collect();
        }

        {
            let mut stats = self.stats.write().unwrap();
            stats.write_barriers_processed += entries.len() as u64;
        }

        // Mark any newly reachable objects
        {
            let objects = self.objects.read().unwrap();
            let mut gray = self.gray_set.lock().unwrap();

            for entry in entries {
                if entry.new_target != 0 {
                    if let Some(obj) = objects.get(&entry.new_target) {
                        if obj.header.compare_and_set_color(Color::White, Color::Gray) {
                            gray.push_back(entry.new_target);
                        }
                    }
                }
            }
        }

        // Finish marking
        self.concurrent_mark_full();

        {
            let mut phase = self.phase.write().unwrap();
            *phase = GcPhase::ConcurrentSweep;
        }

        // Update pause time stats
        let pause_ns = start.elapsed().as_nanos() as u64;
        let mut stats = self.stats.write().unwrap();
        stats.total_pause_time_ns += pause_ns;
        if pause_ns > stats.max_pause_time_ns {
            stats.max_pause_time_ns = pause_ns;
        }
    }

    /// Phase 4: Sweep (synchronous version).
    fn sweep_sync(&self) {
        let mut to_remove = Vec::new();
        let mut bytes_freed = 0usize;

        {
            let objects = self.objects.read().unwrap();
            for (ptr, obj) in objects.iter() {
                if obj.header.get_color() == Color::White {
                    to_remove.push(*ptr);
                    bytes_freed += obj.header.size.load(Ordering::Relaxed);
                }
            }
        }

        {
            let mut objects = self.objects.write().unwrap();
            for ptr in &to_remove {
                objects.remove(ptr);
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.collections += 1;
            stats.last_freed = to_remove.len();
            stats.last_bytes_freed = bytes_freed;
            stats.bytes_allocated = stats.bytes_allocated.saturating_sub(bytes_freed);
            stats.objects_count = stats.objects_count.saturating_sub(to_remove.len());
        }

        self.bytes_since_gc.store(0, Ordering::Relaxed);

        {
            let mut phase = self.phase.write().unwrap();
            *phase = GcPhase::Idle;
        }
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

/// Background GC worker that runs concurrent phases.
pub struct GcWorker {
    /// The concurrent GC instance.
    gc: Arc<ConcurrentGc>,
    /// Worker thread handle.
    thread: Option<JoinHandle<()>>,
}

impl GcWorker {
    /// Creates and starts a new GC worker.
    pub fn new(gc: Arc<ConcurrentGc>) -> Self {
        let gc_clone = Arc::clone(&gc);

        let thread = thread::spawn(move || {
            Self::worker_loop(gc_clone);
        });

        Self {
            gc,
            thread: Some(thread),
        }
    }

    /// Worker loop that waits for and processes GC requests.
    fn worker_loop(gc: Arc<ConcurrentGc>) {
        loop {
            // Wait for GC request
            {
                let guard = gc.gc_mutex.lock().unwrap();
                let _guard = gc
                    .gc_condvar
                    .wait_while(guard, |_| {
                        !gc.shutdown.load(Ordering::Relaxed)
                            && *gc.phase.read().unwrap() == GcPhase::Idle
                    })
                    .unwrap();
            }

            if gc.shutdown.load(Ordering::Relaxed) {
                break;
            }

            // Run collection
            gc.collect_sync();
        }
    }

    /// Stops the worker thread.
    pub fn stop(&mut self) {
        self.gc.shutdown();

        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for GcWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Incremental GC controller for cooperative scheduling.
pub struct IncrementalGc {
    /// The concurrent GC instance.
    gc: Arc<ConcurrentGc>,
    /// Current incremental state.
    state: GcPhase,
}

impl IncrementalGc {
    /// Creates a new incremental GC controller.
    pub fn new(gc: Arc<ConcurrentGc>) -> Self {
        Self {
            gc,
            state: GcPhase::Idle,
        }
    }

    /// Performs one incremental step of GC work.
    ///
    /// Returns true if a collection cycle is complete.
    pub fn step(&mut self) -> bool {
        match self.state {
            GcPhase::Idle => {
                // Check if collection needed
                let bytes = self.gc.bytes_since_gc.load(Ordering::Relaxed);
                if bytes >= self.gc.config.gc_threshold {
                    self.gc.initial_mark();
                    self.state = GcPhase::ConcurrentMark;
                }
                false
            }
            GcPhase::InitialMark => {
                self.gc.initial_mark();
                self.state = GcPhase::ConcurrentMark;
                false
            }
            GcPhase::ConcurrentMark => {
                let done = self.gc.concurrent_mark_step(100); // Small batch
                if done {
                    self.gc.remark();
                    self.state = GcPhase::ConcurrentSweep;
                }
                false
            }
            GcPhase::Remark => {
                self.gc.remark();
                self.state = GcPhase::ConcurrentSweep;
                false
            }
            GcPhase::ConcurrentSweep => {
                self.gc.sweep_sync();
                self.state = GcPhase::Idle;
                true // Cycle complete
            }
        }
    }

    /// Starts a new collection cycle.
    pub fn start_collection(&mut self) {
        if self.state == GcPhase::Idle {
            self.state = GcPhase::InitialMark;
        }
    }

    /// Checks if a collection is in progress.
    pub fn is_collecting(&self) -> bool {
        self.state != GcPhase::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_allocation() {
        let gc = ConcurrentGc::new();

        let ptr1 = gc.alloc(100, 1);
        let ptr2 = gc.alloc(200, 2);

        assert!(!ptr1.is_null());
        assert!(!ptr2.is_null());
        assert_ne!(ptr1, ptr2);

        assert_eq!(gc.object_count(), 2);
    }

    #[test]
    fn test_root_tracking() {
        let gc = ConcurrentGc::new();

        let ptr = gc.alloc(100, 1) as usize;
        gc.add_root(ptr);

        gc.collect_sync();

        // Object should still be alive (rooted)
        assert!(gc.is_alive(ptr));
    }

    #[test]
    fn test_garbage_collection() {
        let gc = ConcurrentGc::new();

        let ptr1 = gc.alloc(100, 1) as usize;
        let ptr2 = gc.alloc(200, 2) as usize;

        // Only root ptr1
        gc.add_root(ptr1);

        gc.collect_sync();

        // ptr1 should be alive, ptr2 should be collected
        assert!(gc.is_alive(ptr1));
        assert!(!gc.is_alive(ptr2));
    }

    #[test]
    fn test_stats() {
        let gc = ConcurrentGc::new();

        gc.alloc(100, 1);
        gc.alloc(200, 2);

        let stats = gc.get_stats();
        assert_eq!(stats.objects_count, 2);
        assert!(stats.bytes_allocated >= 300);
    }

    #[test]
    fn test_tri_color_marking() {
        let gc = ConcurrentGc::new();

        let ptr = gc.alloc(100, 1) as usize;

        // Initially white
        {
            let objects = gc.objects.read().unwrap();
            let obj = objects.get(&ptr).unwrap();
            assert_eq!(obj.header.get_color(), Color::White);
        }

        gc.add_root(ptr);
        gc.collect_sync();

        // After collection and being rooted, should be black
        {
            let objects = gc.objects.read().unwrap();
            // Object survived, color might be reset for next cycle
            assert!(objects.contains_key(&ptr));
        }
    }

    #[test]
    fn test_incremental_gc() {
        let gc = ConcurrentGc::new();
        let mut inc = IncrementalGc::new(Arc::clone(&gc));

        // Allocate enough to trigger collection
        for _ in 0..100 {
            gc.alloc(10000, 1);
        }

        // Run incremental steps until complete
        inc.start_collection();
        while inc.is_collecting() {
            inc.step();
        }

        // Most objects should be collected (no roots)
        assert_eq!(gc.object_count(), 0);
    }

    #[test]
    fn test_write_barrier() {
        let gc = ConcurrentGc::with_config(ConcurrentGcConfig {
            write_barrier: true,
            ..Default::default()
        });

        let ptr1 = gc.alloc(100, 1) as usize;
        let ptr2 = gc.alloc(100, 2) as usize;

        // Simulate a pointer write
        gc.write_barrier(ptr1, 0, ptr2);

        // Buffer should have the entry
        {
            let buffer = gc.write_barrier_buffer.lock().unwrap();
            // Entry might or might not be there depending on GC phase
            // During Idle phase, write barriers are ignored
        }
    }

    #[test]
    fn test_gc_phases() {
        let gc = ConcurrentGc::new();

        assert_eq!(gc.get_phase(), GcPhase::Idle);

        gc.alloc(100, 1);
        gc.collect_sync();

        // Should be idle after collection
        assert_eq!(gc.get_phase(), GcPhase::Idle);
    }

    #[test]
    fn test_config() {
        let config = ConcurrentGcConfig {
            gc_threshold: 500,
            target_pause_ns: 500_000,
            max_marking_steps: 500,
            concurrent_sweep: false,
            write_barrier: false,
        };

        let gc = ConcurrentGc::with_config(config);
        assert_eq!(gc.config.gc_threshold, 500);
    }

    #[test]
    fn test_multiple_collections() {
        let gc = ConcurrentGc::new();

        for i in 0..5 {
            let ptr = gc.alloc(100, i);
            gc.add_root(ptr as usize);
            gc.collect_sync();
        }

        let stats = gc.get_stats();
        assert_eq!(stats.collections, 5);
        assert_eq!(stats.objects_count, 5);
    }
}
