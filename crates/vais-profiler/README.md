# vais-profiler

Performance profiling tool for the Vais compiler.

## Features

- **CPU Profiling**: Sampling-based profiling with hot function detection
- **Memory Tracking**: Allocation/deallocation tracking with peak memory monitoring
- **Call Graph**: Function call relationship tracking and analysis
- **Multiple Report Formats**: Text, JSON, and Flamegraph support
- **C FFI**: Direct integration with LLVM IR generated code
- **Thread-Safe**: Concurrent profiling support with minimal overhead

## Usage

### Rust API

```rust
use vais_profiler::{Profiler, ProfilerConfig, ProfilerMode};
use vais_profiler::reporter::{TextReport, ProfileStats};

// Create profiler with default config
let profiler = Profiler::default();

// Start profiling
profiler.start().unwrap();

// Record samples
profiler.record_sample("main", 0x1000);
profiler.record_sample("foo", 0x2000);

// Track memory
profiler.record_allocation(1024, 0x10000);
profiler.record_deallocation(0x10000);

// Track calls
profiler.record_call("main", "foo");

// Stop profiling
profiler.stop().unwrap();

// Generate reports
let snapshot = profiler.snapshot();
let report = TextReport::new(snapshot.clone());
println!("{}", report);

let stats = ProfileStats::from_snapshot(&snapshot);
println!("{}", stats.to_json().unwrap());
```

### C FFI API

```c
#include "vais_profiler.h"

// Create profiler
void* profiler = vais_profiler_create(NULL);

// Start profiling
vais_profiler_start(profiler);

// Record events
vais_profiler_record_sample(profiler, "main", 0x1000);
vais_profiler_record_allocation(profiler, 1024, 0x10000);
vais_profiler_record_call(profiler, "main", "foo");

// Get statistics
VaisProfileStats stats = vais_profiler_get_stats(profiler);

// Stop and cleanup
vais_profiler_stop(profiler);
vais_profiler_destroy(profiler);
```

### Global Profiler

```c
// Initialize global profiler
vais_profiler_global_init(NULL);
vais_profiler_global_start();

// Use from anywhere
vais_profiler_global_record_sample("function", 0x1000);

// Stop and cleanup
vais_profiler_global_stop();
vais_profiler_global_destroy();
```

## Configuration

```rust
use vais_profiler::{ProfilerConfig, ProfilerMode};
use std::time::Duration;

let config = ProfilerConfig {
    mode: ProfilerMode::All,
    sample_interval: Duration::from_millis(1),
    track_memory: true,
    build_call_graph: true,
    max_samples: 1_000_000,
};

let profiler = Profiler::new(config);
```

## Report Types

### Text Report

Human-readable console output with CPU, memory, and call graph sections.

### Profile Stats (JSON)

Structured statistics suitable for programmatic analysis and visualization.

### Flamegraph

Stack-based visualization of sample data (requires `flamegraph` feature).

```bash
cargo build --features flamegraph
```

## Testing

```bash
cargo test
cargo test --features flamegraph
```

## License

MIT
