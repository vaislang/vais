use crate::MemoryStats;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SampleCollector {
    samples: HashMap<String, Vec<usize>>,
    max_samples: usize,
    total_samples: usize,
}

impl SampleCollector {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: HashMap::new(),
            max_samples,
            total_samples: 0,
        }
    }

    pub fn add_sample(&mut self, function_name: &str, instruction_pointer: usize) {
        if self.total_samples >= self.max_samples {
            return;
        }

        self.samples
            .entry(function_name.to_string())
            .or_default()
            .push(instruction_pointer);
        self.total_samples += 1;
    }

    pub fn sample_count(&self) -> usize {
        self.total_samples
    }

    pub fn get_function_samples(&self) -> Vec<(String, usize)> {
        let mut result: Vec<_> = self
            .samples
            .iter()
            .map(|(name, samples)| (name.clone(), samples.len()))
            .collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    pub fn get_hot_functions(&self, top_n: usize) -> Vec<(String, usize, f64)> {
        let total = self.total_samples as f64;
        let mut functions = self.get_function_samples();
        functions.truncate(top_n);

        functions
            .into_iter()
            .map(|(name, count)| {
                let percentage = (count as f64 / total) * 100.0;
                (name, count, percentage)
            })
            .collect()
    }

    pub fn clear(&mut self) {
        self.samples.clear();
        self.total_samples = 0;
    }
}

#[derive(Debug)]
struct AllocationInfo {
    size: usize,
}

#[derive(Debug)]
pub struct MemoryTracker {
    allocations: HashMap<usize, AllocationInfo>,
    total_allocations: usize,
    total_deallocations: usize,
    total_allocated_bytes: usize,
    current_allocated_bytes: usize,
    peak_allocated_bytes: usize,
    allocation_counter: usize,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            total_allocations: 0,
            total_deallocations: 0,
            total_allocated_bytes: 0,
            current_allocated_bytes: 0,
            peak_allocated_bytes: 0,
            allocation_counter: 0,
        }
    }

    pub fn record_allocation(&mut self, size: usize, address: usize) {
        self.allocations.insert(address, AllocationInfo { size });

        self.total_allocations += 1;
        self.total_allocated_bytes += size;
        self.current_allocated_bytes += size;
        self.allocation_counter += 1;

        if self.current_allocated_bytes > self.peak_allocated_bytes {
            self.peak_allocated_bytes = self.current_allocated_bytes;
        }
    }

    pub fn record_deallocation(&mut self, address: usize) {
        if let Some(info) = self.allocations.remove(&address) {
            self.total_deallocations += 1;
            self.current_allocated_bytes -= info.size;
        }
    }

    pub fn total_allocations(&self) -> usize {
        self.total_allocations
    }

    pub fn total_deallocations(&self) -> usize {
        self.total_deallocations
    }

    pub fn total_allocated_bytes(&self) -> usize {
        self.total_allocated_bytes
    }

    pub fn current_allocated_bytes(&self) -> usize {
        self.current_allocated_bytes
    }

    pub fn peak_allocated_bytes(&self) -> usize {
        self.peak_allocated_bytes
    }

    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            total_allocations: self.total_allocations,
            total_deallocations: self.total_deallocations,
            total_allocated_bytes: self.total_allocated_bytes,
            current_allocated_bytes: self.current_allocated_bytes,
            peak_allocated_bytes: self.peak_allocated_bytes,
        }
    }

    pub fn get_live_allocations(&self) -> Vec<(usize, usize)> {
        let mut allocations: Vec<_> = self
            .allocations
            .iter()
            .map(|(addr, info)| (*addr, info.size))
            .collect();
        allocations.sort_by(|a, b| b.1.cmp(&a.1));
        allocations
    }

    pub fn clear(&mut self) {
        self.allocations.clear();
        self.total_allocations = 0;
        self.total_deallocations = 0;
        self.total_allocated_bytes = 0;
        self.current_allocated_bytes = 0;
        self.peak_allocated_bytes = 0;
        self.allocation_counter = 0;
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct CallGraph {
    edges: HashMap<(String, String), usize>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    pub fn record_call(&mut self, caller: &str, callee: &str) {
        let key = (caller.to_string(), callee.to_string());
        *self.edges.entry(key).or_insert(0) += 1;
    }

    pub fn get_call_count(&self, caller: &str, callee: &str) -> usize {
        self.edges
            .get(&(caller.to_string(), callee.to_string()))
            .copied()
            .unwrap_or(0)
    }

    pub fn get_edges(&self) -> Vec<(String, String, usize)> {
        self.edges
            .iter()
            .map(|((caller, callee), count)| (caller.clone(), callee.clone(), *count))
            .collect()
    }

    pub fn get_callers(&self, function: &str) -> Vec<(String, usize)> {
        self.edges
            .iter()
            .filter_map(|((caller, callee), count)| {
                if callee == function {
                    Some((caller.clone(), *count))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_callees(&self, function: &str) -> Vec<(String, usize)> {
        self.edges
            .iter()
            .filter_map(|((caller, callee), count)| {
                if caller == function {
                    Some((callee.clone(), *count))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_hot_edges(&self, top_n: usize) -> Vec<(String, String, usize)> {
        let mut edges = self.get_edges();
        edges.sort_by(|a, b| b.2.cmp(&a.2));
        edges.truncate(top_n);
        edges
    }

    pub fn clear(&mut self) {
        self.edges.clear();
    }
}

impl Default for CallGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_collector() {
        let mut collector = SampleCollector::new(1000);

        collector.add_sample("main", 0x1000);
        collector.add_sample("foo", 0x2000);
        collector.add_sample("main", 0x1100);
        collector.add_sample("bar", 0x3000);
        collector.add_sample("main", 0x1200);

        assert_eq!(collector.sample_count(), 5);

        let samples = collector.get_function_samples();
        assert_eq!(samples.len(), 3);
        assert_eq!(samples[0], ("main".to_string(), 3));
        // Order of foo and bar is not guaranteed (both have 1 sample)
        assert!(samples
            .iter()
            .any(|(name, count)| name == "foo" && *count == 1));
        assert!(samples
            .iter()
            .any(|(name, count)| name == "bar" && *count == 1));
    }

    #[test]
    fn test_sample_collector_max_samples() {
        let mut collector = SampleCollector::new(3);

        collector.add_sample("main", 0x1000);
        collector.add_sample("foo", 0x2000);
        collector.add_sample("bar", 0x3000);
        collector.add_sample("baz", 0x4000);

        assert_eq!(collector.sample_count(), 3);
    }

    #[test]
    fn test_hot_functions() {
        let mut collector = SampleCollector::new(1000);

        for _ in 0..50 {
            collector.add_sample("main", 0x1000);
        }
        for _ in 0..30 {
            collector.add_sample("foo", 0x2000);
        }
        for _ in 0..20 {
            collector.add_sample("bar", 0x3000);
        }

        let hot = collector.get_hot_functions(2);
        assert_eq!(hot.len(), 2);
        assert_eq!(hot[0].0, "main");
        assert_eq!(hot[0].1, 50);
        assert!((hot[0].2 - 50.0).abs() < 0.01);
        assert_eq!(hot[1].0, "foo");
        assert_eq!(hot[1].1, 30);
        assert!((hot[1].2 - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new();

        tracker.record_allocation(100, 0x1000);
        tracker.record_allocation(200, 0x2000);

        assert_eq!(tracker.total_allocations(), 2);
        assert_eq!(tracker.total_allocated_bytes(), 300);
        assert_eq!(tracker.current_allocated_bytes(), 300);
        assert_eq!(tracker.peak_allocated_bytes(), 300);

        tracker.record_deallocation(0x1000);

        assert_eq!(tracker.total_deallocations(), 1);
        assert_eq!(tracker.current_allocated_bytes(), 200);
        assert_eq!(tracker.peak_allocated_bytes(), 300);
    }

    #[test]
    fn test_memory_tracker_peak() {
        let mut tracker = MemoryTracker::new();

        tracker.record_allocation(100, 0x1000);
        tracker.record_allocation(200, 0x2000);
        tracker.record_allocation(300, 0x3000);
        assert_eq!(tracker.peak_allocated_bytes(), 600);

        tracker.record_deallocation(0x2000);
        assert_eq!(tracker.peak_allocated_bytes(), 600);
        assert_eq!(tracker.current_allocated_bytes(), 400);

        tracker.record_allocation(50, 0x4000);
        assert_eq!(tracker.peak_allocated_bytes(), 600);
    }

    #[test]
    fn test_live_allocations() {
        let mut tracker = MemoryTracker::new();

        tracker.record_allocation(100, 0x1000);
        tracker.record_allocation(200, 0x2000);
        tracker.record_allocation(50, 0x3000);

        let live = tracker.get_live_allocations();
        assert_eq!(live.len(), 3);
        assert_eq!(live[0].1, 200);
        assert_eq!(live[1].1, 100);
        assert_eq!(live[2].1, 50);
    }

    #[test]
    fn test_call_graph() {
        let mut graph = CallGraph::new();

        graph.record_call("main", "foo");
        graph.record_call("main", "bar");
        graph.record_call("foo", "baz");
        graph.record_call("main", "foo");

        assert_eq!(graph.get_call_count("main", "foo"), 2);
        assert_eq!(graph.get_call_count("main", "bar"), 1);
        assert_eq!(graph.get_call_count("foo", "baz"), 1);
        assert_eq!(graph.get_call_count("bar", "foo"), 0);
    }

    #[test]
    fn test_call_graph_callers() {
        let mut graph = CallGraph::new();

        graph.record_call("main", "foo");
        graph.record_call("bar", "foo");
        graph.record_call("baz", "foo");

        let callers = graph.get_callers("foo");
        assert_eq!(callers.len(), 3);
    }

    #[test]
    fn test_call_graph_callees() {
        let mut graph = CallGraph::new();

        graph.record_call("main", "foo");
        graph.record_call("main", "bar");
        graph.record_call("main", "baz");

        let callees = graph.get_callees("main");
        assert_eq!(callees.len(), 3);
    }

    #[test]
    fn test_hot_edges() {
        let mut graph = CallGraph::new();

        for _ in 0..10 {
            graph.record_call("main", "foo");
        }
        for _ in 0..5 {
            graph.record_call("main", "bar");
        }
        for _ in 0..3 {
            graph.record_call("foo", "baz");
        }

        let hot = graph.get_hot_edges(2);
        assert_eq!(hot.len(), 2);
        assert_eq!(hot[0], ("main".to_string(), "foo".to_string(), 10));
        assert_eq!(hot[1], ("main".to_string(), "bar".to_string(), 5));
    }
}
