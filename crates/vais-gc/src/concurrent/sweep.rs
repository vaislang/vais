//! Sweep and remark phases for concurrent GC

use super::*;

impl ConcurrentGc {
    /// Runs a full collection synchronously.
    pub fn collect_sync(&self) {
        self.initial_mark();
        self.concurrent_mark_full();
        self.remark();
        self.sweep_sync();
    }

    /// Phase 3: Remark (STW).
    ///
    /// Process write barrier entries and finish marking.
    pub(crate) fn remark(&self) {
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
    pub(crate) fn sweep_sync(&self) {
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
}
