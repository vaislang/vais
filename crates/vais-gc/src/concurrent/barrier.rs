//! Write barrier implementation for concurrent GC

use super::*;

impl ConcurrentGc {
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
}
