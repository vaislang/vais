//! Resource limits for sandboxed execution
//!
//! Defines constraints for memory, CPU time, and other resources
//! to prevent runaway plugins from consuming system resources.

use std::time::Duration;

/// Memory limit configuration
#[derive(Debug, Clone)]
pub struct MemoryLimit {
    /// Maximum memory in bytes (default: 64 MB)
    pub max_bytes: u64,
    /// Initial memory in bytes (default: 1 MB)
    pub initial_bytes: u64,
    /// Maximum linear memory pages for WASM (1 page = 64KB)
    pub max_wasm_pages: u32,
    /// Enable memory tracking
    pub track_usage: bool,
}

impl Default for MemoryLimit {
    fn default() -> Self {
        Self {
            max_bytes: 64 * 1024 * 1024,      // 64 MB
            initial_bytes: 1 * 1024 * 1024,    // 1 MB
            max_wasm_pages: 1024,              // 64 MB in WASM pages
            track_usage: true,
        }
    }
}

impl MemoryLimit {
    /// Create a new memory limit with the specified maximum bytes
    pub fn new(max_bytes: u64) -> Self {
        let max_wasm_pages = (max_bytes / (64 * 1024)) as u32;
        Self {
            max_bytes,
            initial_bytes: max_bytes.min(1024 * 1024),
            max_wasm_pages,
            track_usage: true,
        }
    }

    /// Create memory limit in megabytes
    pub fn megabytes(mb: u64) -> Self {
        Self::new(mb * 1024 * 1024)
    }

    /// Create an unlimited memory limit (use with caution)
    pub fn unlimited() -> Self {
        Self {
            max_bytes: u64::MAX,
            initial_bytes: 1024 * 1024,
            max_wasm_pages: u32::MAX,
            track_usage: false,
        }
    }

    /// Set initial memory
    pub fn with_initial(mut self, initial_bytes: u64) -> Self {
        self.initial_bytes = initial_bytes;
        self
    }

    /// Enable or disable usage tracking
    pub fn with_tracking(mut self, track: bool) -> Self {
        self.track_usage = track;
        self
    }
}

/// Time limit configuration
#[derive(Debug, Clone)]
pub struct TimeLimit {
    /// Maximum execution time in milliseconds (default: 5000ms = 5s)
    pub max_duration_ms: u64,
    /// Fuel limit for WASM execution (None = unlimited)
    pub fuel_limit: Option<u64>,
    /// Check interval for timeout detection
    pub check_interval_ms: u64,
    /// Enable epoch-based interruption for WASM
    pub use_epoch_interruption: bool,
}

impl Default for TimeLimit {
    fn default() -> Self {
        Self {
            max_duration_ms: 5000,           // 5 seconds
            fuel_limit: Some(10_000_000),    // 10 million fuel units
            check_interval_ms: 100,
            use_epoch_interruption: true,
        }
    }
}

impl TimeLimit {
    /// Create a new time limit with the specified duration in milliseconds
    pub fn new(max_duration_ms: u64) -> Self {
        Self {
            max_duration_ms,
            fuel_limit: Some(max_duration_ms * 2000), // Rough estimate
            check_interval_ms: (max_duration_ms / 50).max(10),
            use_epoch_interruption: true,
        }
    }

    /// Create time limit from Duration
    pub fn from_duration(duration: Duration) -> Self {
        Self::new(duration.as_millis() as u64)
    }

    /// Create time limit in seconds
    pub fn seconds(secs: u64) -> Self {
        Self::new(secs * 1000)
    }

    /// Create an unlimited time limit (use with caution)
    pub fn unlimited() -> Self {
        Self {
            max_duration_ms: u64::MAX,
            fuel_limit: None,
            check_interval_ms: u64::MAX,
            use_epoch_interruption: false,
        }
    }

    /// Set a specific fuel limit
    pub fn with_fuel(mut self, fuel: u64) -> Self {
        self.fuel_limit = Some(fuel);
        self
    }

    /// Disable fuel limiting
    pub fn without_fuel(mut self) -> Self {
        self.fuel_limit = None;
        self
    }

    /// Get as Duration
    pub fn as_duration(&self) -> Duration {
        Duration::from_millis(self.max_duration_ms)
    }
}

/// Stack limit configuration
#[derive(Debug, Clone)]
pub struct StackLimit {
    /// Maximum stack size in bytes (default: 1 MB)
    pub max_bytes: u64,
    /// Maximum call depth (default: 1000)
    pub max_call_depth: u32,
}

impl Default for StackLimit {
    fn default() -> Self {
        Self {
            max_bytes: 1024 * 1024,  // 1 MB
            max_call_depth: 1000,
        }
    }
}

impl StackLimit {
    /// Create a new stack limit
    pub fn new(max_bytes: u64, max_call_depth: u32) -> Self {
        Self {
            max_bytes,
            max_call_depth,
        }
    }
}

/// Combined resource limits
#[derive(Debug, Clone, Default)]
pub struct ResourceLimits {
    /// Memory limits
    pub memory: MemoryLimit,
    /// Time/execution limits
    pub time: TimeLimit,
    /// Stack limits
    pub stack: StackLimit,
    /// Maximum number of tables in WASM
    pub max_tables: u32,
    /// Maximum number of memories in WASM
    pub max_memories: u32,
    /// Maximum number of instances in WASM
    pub max_instances: u32,
}

impl ResourceLimits {
    /// Create new resource limits with all defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create restrictive limits for untrusted plugins
    pub fn restrictive() -> Self {
        Self {
            memory: MemoryLimit::megabytes(16),
            time: TimeLimit::seconds(1),
            stack: StackLimit::new(256 * 1024, 500),
            max_tables: 1,
            max_memories: 1,
            max_instances: 1,
        }
    }

    /// Create permissive limits for trusted plugins
    pub fn permissive() -> Self {
        Self {
            memory: MemoryLimit::megabytes(256),
            time: TimeLimit::unlimited(),
            stack: StackLimit::new(4 * 1024 * 1024, 5000),
            max_tables: 10,
            max_memories: 1,
            max_instances: 10,
        }
    }

    /// Set memory limits
    pub fn with_memory(mut self, memory: MemoryLimit) -> Self {
        self.memory = memory;
        self
    }

    /// Set time limits
    pub fn with_time(mut self, time: TimeLimit) -> Self {
        self.time = time;
        self
    }

    /// Set stack limits
    pub fn with_stack(mut self, stack: StackLimit) -> Self {
        self.stack = stack;
        self
    }

    /// Validate that limits are reasonable
    pub fn validate(&self) -> Result<(), String> {
        if self.memory.max_bytes < self.memory.initial_bytes {
            return Err("Max memory must be >= initial memory".to_string());
        }

        if self.time.max_duration_ms == 0 {
            return Err("Execution time must be > 0".to_string());
        }

        if self.stack.max_call_depth == 0 {
            return Err("Call depth must be > 0".to_string());
        }

        Ok(())
    }
}

/// Resource usage tracker
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    /// Current memory usage in bytes
    pub memory_bytes: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Fuel consumed (if tracking)
    pub fuel_consumed: Option<u64>,
    /// Number of function calls
    pub call_count: u64,
}

impl ResourceUsage {
    /// Create new empty usage tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Update memory usage
    pub fn update_memory(&mut self, bytes: u64) {
        self.memory_bytes = bytes;
        if bytes > self.peak_memory_bytes {
            self.peak_memory_bytes = bytes;
        }
    }

    /// Add execution time
    pub fn add_time(&mut self, ms: u64) {
        self.execution_time_ms += ms;
    }

    /// Add fuel consumption
    pub fn add_fuel(&mut self, fuel: u64) {
        *self.fuel_consumed.get_or_insert(0) += fuel;
    }

    /// Increment call count
    pub fn increment_calls(&mut self) {
        self.call_count += 1;
    }

    /// Check if memory limit is exceeded
    pub fn exceeds_memory(&self, limit: &MemoryLimit) -> bool {
        self.memory_bytes > limit.max_bytes
    }

    /// Check if time limit is exceeded
    pub fn exceeds_time(&self, limit: &TimeLimit) -> bool {
        self.execution_time_ms > limit.max_duration_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_limit_default() {
        let limit = MemoryLimit::default();
        assert_eq!(limit.max_bytes, 64 * 1024 * 1024);
        assert_eq!(limit.initial_bytes, 1024 * 1024);
    }

    #[test]
    fn test_memory_limit_megabytes() {
        let limit = MemoryLimit::megabytes(32);
        assert_eq!(limit.max_bytes, 32 * 1024 * 1024);
    }

    #[test]
    fn test_time_limit_default() {
        let limit = TimeLimit::default();
        assert_eq!(limit.max_duration_ms, 5000);
        assert!(limit.fuel_limit.is_some());
    }

    #[test]
    fn test_time_limit_seconds() {
        let limit = TimeLimit::seconds(10);
        assert_eq!(limit.max_duration_ms, 10000);
    }

    #[test]
    fn test_resource_limits_restrictive() {
        let limits = ResourceLimits::restrictive();
        assert_eq!(limits.memory.max_bytes, 16 * 1024 * 1024);
        assert_eq!(limits.time.max_duration_ms, 1000);
    }

    #[test]
    fn test_resource_limits_validate() {
        let valid = ResourceLimits::default();
        assert!(valid.validate().is_ok());

        let mut invalid = ResourceLimits::default();
        invalid.memory.initial_bytes = invalid.memory.max_bytes + 1;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_resource_usage_tracking() {
        let mut usage = ResourceUsage::new();

        usage.update_memory(1000);
        assert_eq!(usage.memory_bytes, 1000);
        assert_eq!(usage.peak_memory_bytes, 1000);

        usage.update_memory(500);
        assert_eq!(usage.memory_bytes, 500);
        assert_eq!(usage.peak_memory_bytes, 1000);

        usage.add_time(100);
        usage.add_time(50);
        assert_eq!(usage.execution_time_ms, 150);

        usage.increment_calls();
        usage.increment_calls();
        assert_eq!(usage.call_count, 2);
    }

    #[test]
    fn test_resource_usage_exceeds() {
        let mut usage = ResourceUsage::new();
        let memory_limit = MemoryLimit::megabytes(1);
        let time_limit = TimeLimit::seconds(1);

        usage.update_memory(512 * 1024); // 512 KB
        assert!(!usage.exceeds_memory(&memory_limit));

        usage.update_memory(2 * 1024 * 1024); // 2 MB
        assert!(usage.exceeds_memory(&memory_limit));

        usage.add_time(500);
        assert!(!usage.exceeds_time(&time_limit));

        usage.add_time(600);
        assert!(usage.exceeds_time(&time_limit));
    }
}
