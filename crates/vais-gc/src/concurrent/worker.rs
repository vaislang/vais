//! Background GC worker and incremental GC controller

use super::*;
use std::thread::{self, JoinHandle};

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
