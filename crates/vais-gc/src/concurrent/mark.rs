//! Tri-color marking phases for concurrent GC

use super::*;

impl ConcurrentGc {
    /// Phase 1: Initial Mark (STW).
    ///
    /// Brief pause to mark root objects.
    pub(crate) fn initial_mark(&self) {
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
    pub(crate) fn concurrent_mark_step(&self, max_steps: usize) -> bool {
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
    pub(crate) fn concurrent_mark_full(&self) {
        while !self.concurrent_mark_step(self.config.max_marking_steps) {
            // Keep marking
        }
    }

    /// Scans an object for child pointers.
    pub(crate) fn scan_object(&self, ptr: usize) -> Vec<usize> {
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
                // SAFETY: `offset + ptr_size <= size` is checked above, and `obj.data`
                // has at least `size` bytes. The read is aligned to `usize` boundaries
                // via `step_by(ptr_size)`. Conservative pointer scanning: the value is
                // only used if it matches a known GC object address in the objects map.
                unsafe {
                    let potential_ptr =
                        std::ptr::read(obj.data.as_ptr().add(offset) as *const usize);
                    if objects.contains_key(&potential_ptr) {
                        children.push(potential_ptr);
                    }
                }
            }
        }

        children
    }
}
