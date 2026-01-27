pub mod collector;
pub mod reporter;
pub mod ffi;

use collector::{CallGraph, MemoryTracker, SampleCollector};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProfilerError {
    #[error("Profiler already running")]
    AlreadyRunning,
    #[error("Profiler not running")]
    NotRunning,
    #[error("Sample collection error: {0}")]
    CollectionError(String),
    #[error("Report generation error: {0}")]
    ReportError(String),
}

pub type Result<T> = std::result::Result<T, ProfilerError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfilerMode {
    Sampling,
    Instrumentation,
    Memory,
    All,
}

#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    pub mode: ProfilerMode,
    pub sample_interval: Duration,
    pub track_memory: bool,
    pub build_call_graph: bool,
    pub max_samples: usize,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            mode: ProfilerMode::All,
            sample_interval: Duration::from_millis(1),
            track_memory: true,
            build_call_graph: true,
            max_samples: 1_000_000,
        }
    }
}

#[derive(Debug)]
struct ProfilerState {
    running: bool,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
    sample_collector: SampleCollector,
    memory_tracker: MemoryTracker,
    call_graph: CallGraph,
}

impl ProfilerState {
    fn new(config: &ProfilerConfig) -> Self {
        Self {
            running: false,
            start_time: None,
            end_time: None,
            sample_collector: SampleCollector::new(config.max_samples),
            memory_tracker: MemoryTracker::new(),
            call_graph: CallGraph::new(),
        }
    }

    fn reset(&mut self) {
        self.running = false;
        self.start_time = None;
        self.end_time = None;
        self.sample_collector.clear();
        self.memory_tracker.clear();
        self.call_graph.clear();
    }
}

pub struct Profiler {
    config: ProfilerConfig,
    state: Arc<RwLock<ProfilerState>>,
}

impl Profiler {
    pub fn new(config: ProfilerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(ProfilerState::new(&config))),
            config,
        }
    }

    pub fn start(&self) -> Result<()> {
        let mut state = self.state.write();
        if state.running {
            return Err(ProfilerError::AlreadyRunning);
        }

        state.reset();
        state.running = true;
        state.start_time = Some(Instant::now());
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        let mut state = self.state.write();
        if !state.running {
            return Err(ProfilerError::NotRunning);
        }

        state.running = false;
        state.end_time = Some(Instant::now());
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.state.read().running
    }

    pub fn record_sample(&self, function_name: &str, instruction_pointer: usize) {
        if !self.is_running() {
            return;
        }

        let mut state = self.state.write();
        state.sample_collector.add_sample(function_name, instruction_pointer);
    }

    pub fn record_allocation(&self, size: usize, address: usize) {
        if !self.is_running() || !self.config.track_memory {
            return;
        }

        let mut state = self.state.write();
        state.memory_tracker.record_allocation(size, address);
    }

    pub fn record_deallocation(&self, address: usize) {
        if !self.is_running() || !self.config.track_memory {
            return;
        }

        let mut state = self.state.write();
        state.memory_tracker.record_deallocation(address);
    }

    pub fn record_call(&self, caller: &str, callee: &str) {
        if !self.is_running() || !self.config.build_call_graph {
            return;
        }

        let mut state = self.state.write();
        state.call_graph.record_call(caller, callee);
    }

    pub fn get_duration(&self) -> Option<Duration> {
        let state = self.state.read();
        match (state.start_time, state.end_time) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            (Some(start), None) if state.running => Some(Instant::now().duration_since(start)),
            _ => None,
        }
    }

    pub fn get_sample_count(&self) -> usize {
        self.state.read().sample_collector.sample_count()
    }

    pub fn get_total_allocations(&self) -> usize {
        self.state.read().memory_tracker.total_allocations()
    }

    pub fn get_total_allocated_bytes(&self) -> usize {
        self.state.read().memory_tracker.total_allocated_bytes()
    }

    pub fn get_current_allocated_bytes(&self) -> usize {
        self.state.read().memory_tracker.current_allocated_bytes()
    }

    pub fn get_peak_allocated_bytes(&self) -> usize {
        self.state.read().memory_tracker.peak_allocated_bytes()
    }

    pub fn snapshot(&self) -> ProfileSnapshot {
        let state = self.state.read();
        ProfileSnapshot {
            duration: self.get_duration(),
            samples: state.sample_collector.get_function_samples(),
            memory_stats: state.memory_tracker.get_stats(),
            call_graph: state.call_graph.get_edges(),
        }
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new(ProfilerConfig::default())
    }
}

#[derive(Debug, Clone)]
pub struct ProfileSnapshot {
    pub duration: Option<Duration>,
    pub samples: Vec<(String, usize)>,
    pub memory_stats: MemoryStats,
    pub call_graph: Vec<(String, String, usize)>,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub total_allocated_bytes: usize,
    pub current_allocated_bytes: usize,
    pub peak_allocated_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_lifecycle() {
        let profiler = Profiler::default();
        assert!(!profiler.is_running());

        profiler.start().unwrap();
        assert!(profiler.is_running());

        profiler.stop().unwrap();
        assert!(!profiler.is_running());
    }

    #[test]
    fn test_profiler_double_start() {
        let profiler = Profiler::default();
        profiler.start().unwrap();
        assert!(profiler.start().is_err());
    }

    #[test]
    fn test_profiler_stop_without_start() {
        let profiler = Profiler::default();
        assert!(profiler.stop().is_err());
    }

    #[test]
    fn test_record_samples() {
        let profiler = Profiler::default();
        profiler.start().unwrap();

        profiler.record_sample("main", 0x1000);
        profiler.record_sample("foo", 0x2000);
        profiler.record_sample("main", 0x1000);

        assert_eq!(profiler.get_sample_count(), 3);
        profiler.stop().unwrap();

        let snapshot = profiler.snapshot();
        assert_eq!(snapshot.samples.len(), 2);
    }

    #[test]
    fn test_memory_tracking() {
        let profiler = Profiler::default();
        profiler.start().unwrap();

        profiler.record_allocation(100, 0x1000);
        profiler.record_allocation(200, 0x2000);

        assert_eq!(profiler.get_total_allocations(), 2);
        assert_eq!(profiler.get_total_allocated_bytes(), 300);
        assert_eq!(profiler.get_current_allocated_bytes(), 300);

        profiler.record_deallocation(0x1000);
        assert_eq!(profiler.get_current_allocated_bytes(), 200);

        profiler.stop().unwrap();
    }

    #[test]
    fn test_peak_memory() {
        let profiler = Profiler::default();
        profiler.start().unwrap();

        profiler.record_allocation(100, 0x1000);
        profiler.record_allocation(200, 0x2000);
        assert_eq!(profiler.get_peak_allocated_bytes(), 300);

        profiler.record_deallocation(0x1000);
        assert_eq!(profiler.get_peak_allocated_bytes(), 300);
        assert_eq!(profiler.get_current_allocated_bytes(), 200);

        profiler.stop().unwrap();
    }

    #[test]
    fn test_call_graph() {
        let profiler = Profiler::default();
        profiler.start().unwrap();

        profiler.record_call("main", "foo");
        profiler.record_call("main", "bar");
        profiler.record_call("foo", "baz");
        profiler.record_call("main", "foo");

        profiler.stop().unwrap();

        let snapshot = profiler.snapshot();
        assert_eq!(snapshot.call_graph.len(), 3);

        let main_to_foo = snapshot
            .call_graph
            .iter()
            .find(|(caller, callee, _)| caller == "main" && callee == "foo")
            .unwrap();
        assert_eq!(main_to_foo.2, 2);
    }

    #[test]
    fn test_snapshot() {
        let profiler = Profiler::default();
        profiler.start().unwrap();

        profiler.record_sample("main", 0x1000);
        profiler.record_allocation(100, 0x1000);
        profiler.record_call("main", "foo");

        std::thread::sleep(Duration::from_millis(10));
        profiler.stop().unwrap();

        let snapshot = profiler.snapshot();
        assert!(snapshot.duration.is_some());
        assert!(snapshot.duration.unwrap() >= Duration::from_millis(10));
        assert_eq!(snapshot.samples.len(), 1);
        assert_eq!(snapshot.memory_stats.total_allocations, 1);
        assert_eq!(snapshot.call_graph.len(), 1);
    }
}
