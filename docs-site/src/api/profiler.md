# Profiler API Reference

> Runtime performance profiling with timing, memory tracking, and sampling support

## Import

```vais
U std/profiler
```

## Constants

| Name | Value | Description |
|------|-------|-------------|
| `PROFILER_DISABLED` | 0 | Profiling disabled |
| `PROFILER_ENABLED` | 1 | Instrumentation mode enabled |
| `PROFILER_SAMPLING` | 2 | Sampling mode enabled |
| `MAX_PROFILE_ENTRIES` | 4096 | Maximum profile entries |
| `SAMPLE_INTERVAL_MS` | 10 | Default sampling interval |

## Structs

### Timer

High-resolution timer for measuring code execution time.

**Fields:**
- `start_ns: i64` - Start time in nanoseconds
- `end_ns: i64` - End time in nanoseconds
- `running: i64` - 1 if timer is running

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> Timer` | Create new timer |
| `start` | `F start(&self) -> Timer` | Start the timer |
| `stop` | `F stop(&self) -> Timer` | Stop the timer |
| `elapsed_ns` | `F elapsed_ns(&self) -> i64` | Get elapsed nanoseconds |
| `elapsed_us` | `F elapsed_us(&self) -> i64` | Get elapsed microseconds |
| `elapsed_ms` | `F elapsed_ms(&self) -> i64` | Get elapsed milliseconds |
| `reset` | `F reset(&self) -> Timer` | Reset timer to zero |

### ProfileEntry

Single profiling record for a named code region.

**Fields:**
- `name: str` - Profile region name
- `call_count: i64` - Number of times called
- `total_time_ns: i64` - Total time spent
- `min_time_ns: i64` - Minimum time per call
- `max_time_ns: i64` - Maximum time per call
- `start_time: i64` - Start time (for nested timing)
- `depth: i64` - Call depth (for recursion tracking)

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(name: str) -> ProfileEntry` | Create profile entry |
| `enter` | `F enter(&self) -> i64` | Enter profiled region |
| `exit` | `F exit(&self) -> i64` | Exit profiled region |
| `avg_time_ns` | `F avg_time_ns(&self) -> i64` | Get average time per call |
| `reset` | `F reset(&self) -> i64` | Reset entry statistics |

### Profiler

Main profiling interface with instrumentation support.

**Fields:**
- `entries: i64` - Pointer to ProfileEntry array
- `entry_count: i64` - Number of entries
- `capacity: i64` - Array capacity
- `enabled: i64` - Profiler mode (DISABLED/ENABLED/SAMPLING)
- `start_time: i64` - Global start time
- `sample_interval: i64` - Sampling interval in ms

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> Profiler` | Create profiler (default capacity 256) |
| `enable` | `F enable(&self) -> Profiler` | Enable instrumentation mode |
| `disable` | `F disable(&self) -> Profiler` | Disable profiling |
| `enable_sampling` | `F enable_sampling(&self, interval_ms: i64) -> Profiler` | Enable sampling mode |
| `is_enabled` | `F is_enabled(&self) -> i64` | Check if profiling enabled |
| `get_entry` | `F get_entry(&self, name: str) -> i64` | Find or create entry |
| `enter` | `F enter(&self, name: str) -> i64` | Enter profiled region |
| `exit` | `F exit(&self, name: str) -> i64` | Exit profiled region |
| `total_time_ns` | `F total_time_ns(&self) -> i64` | Get total elapsed time |
| `entry_count` | `F entry_count(&self) -> i64` | Get number of entries |
| `reset` | `F reset(&self) -> i64` | Reset all entries |
| `report` | `F report(&self) -> i64` | Print profiling report to stdout |

### MemoryProfiler

Track memory allocations and deallocations.

**Fields:**
- `allocations: i64` - Total allocations count
- `deallocations: i64` - Total deallocations count
- `bytes_allocated: i64` - Current allocated bytes
- `peak_bytes: i64` - Peak allocated bytes
- `total_allocated: i64` - Total bytes ever allocated
- `enabled: i64` - 1 if enabled

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> MemoryProfiler` | Create memory profiler |
| `enable` | `F enable(&self) -> MemoryProfiler` | Enable memory tracking |
| `disable` | `F disable(&self) -> MemoryProfiler` | Disable memory tracking |
| `track_alloc` | `F track_alloc(&self, size: i64) -> i64` | Track allocation |
| `track_dealloc` | `F track_dealloc(&self, size: i64) -> i64` | Track deallocation |
| `current_bytes` | `F current_bytes(&self) -> i64` | Get current allocated bytes |
| `peak_bytes` | `F peak_bytes(&self) -> i64` | Get peak allocated bytes |
| `report` | `F report(&self) -> i64` | Print memory report |
| `reset` | `F reset(&self) -> i64` | Reset all counters |

### SampleProfiler

Statistical profiler that samples call stacks at intervals.

**Fields:**
- `samples: i64` - Pointer to sample buffer
- `sample_count: i64` - Number of samples collected
- `capacity: i64` - Buffer capacity
- `interval_ms: i64` - Sampling interval
- `running: i64` - 1 if sampling active
- `thread_handle: i64` - Sampler thread handle

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(interval_ms: i64) -> SampleProfiler` | Create sampling profiler |
| `start` | `F start(&self) -> i64` | Start sampling thread |
| `stop` | `F stop(&self) -> i64` | Stop sampling thread |
| `record_sample` | `F record_sample(&self, func_ptr: i64) -> i64` | Record a sample |
| `count` | `F count(&self) -> i64` | Get sample count |
| `analyze` | `F analyze(&self) -> i64` | Analyze and print results |
| `reset` | `F reset(&self) -> i64` | Clear all samples |

### FlameGraphBuilder

Generate flame graph data from stack samples.

**Fields:**
- `stack_samples: i64` - Array of stack samples
- `sample_count: i64` - Number of samples
- `capacity: i64` - Buffer capacity

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> FlameGraphBuilder` | Create flame graph builder |
| `record_stack` | `F record_stack(&self, stack_ptr: i64, depth: i64) -> i64` | Record stack sample |
| `generate_folded` | `F generate_folded(&self, output_buffer: i64) -> i64` | Generate folded format |

## Global Functions

### Profiler Management

| Function | Signature | Description |
|----------|-----------|-------------|
| `profiler_init` | `F profiler_init() -> i64` | Initialize global profiler |
| `get_profiler` | `F get_profiler() -> &Profiler` | Get global profiler |
| `mem_profiler_init` | `F mem_profiler_init() -> i64` | Initialize memory profiler |
| `get_mem_profiler` | `F get_mem_profiler() -> &MemoryProfiler` | Get global memory profiler |

### Convenience Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `profile_begin` | `F profile_begin(name: str) -> i64` | Begin profiling region |
| `profile_end` | `F profile_end(name: str) -> i64` | End profiling region |
| `profile_fn` | `F profile_fn(name: str, fn_ptr: i64, arg: i64) -> i64` | Profile function call |
| `timer` | `F timer() -> Timer` | Create timer |
| `time_fn` | `F time_fn(fn_ptr: i64, arg: i64) -> (i64, i64)` | Time function, return (result, elapsed_ns) |

## Usage

### Basic Timing

```vais
U std/profiler

F main() -> i64 {
    t := timer()
    t.start()

    # ... code to measure ...
    do_work()

    t.stop()
    ms := t.elapsed_ms()
    us := t.elapsed_us()

    0
}
```

### Instrumentation Profiling

```vais
U std/profiler

F main() -> i64 {
    p := get_profiler()
    p.enable()

    # Profile regions
    p.enter("region1")
    do_work()
    p.exit("region1")

    p.enter("region2")
    do_more_work()
    p.exit("region2")

    # Print report
    p.report()

    0
}
```

### Convenience Functions

```vais
U std/profiler

F expensive_computation(n: i64) -> i64 {
    sum := 0
    i := 0
    L i < n {
        sum = sum + i
        i = i + 1
    }
    sum
}

F main() -> i64 {
    # Profile with begin/end
    profile_begin("compute")
    result := expensive_computation(1000000)
    profile_end("compute")

    # Or time a function call
    (result2, elapsed) := time_fn(expensive_computation_ptr, 1000000)

    get_profiler().report()

    0
}
```

### Memory Profiling

```vais
U std/profiler

F main() -> i64 {
    mem := get_mem_profiler()
    mem.enable()

    # Allocations are tracked automatically
    buffer := malloc(1024)
    mem.track_alloc(1024)

    # Do work...

    free(buffer)
    mem.track_dealloc(1024)

    # Print report
    mem.report()

    0
}
```

### Sampling Profiler

```vais
U std/profiler

F main() -> i64 {
    # Sample every 10ms
    sampler := SampleProfiler::new(10)
    sampler.start()

    # Run workload
    run_application()

    # Stop and analyze
    sampler.stop()
    sampler.analyze()

    0
}
```

### Nested Profiling

```vais
U std/profiler

F outer() -> i64 {
    profile_begin("outer")

    inner()
    inner()

    profile_end("outer")
    0
}

F inner() -> i64 {
    profile_begin("inner")

    # Do work...
    compute()

    profile_end("inner")
    0
}

F main() -> i64 {
    get_profiler().enable()

    outer()

    get_profiler().report()

    0
}
```

## Report Format

The `report()` method prints a table with the following columns:

- **Name**: Profile region name
- **Calls**: Number of times called
- **Total(ms)**: Total time spent in milliseconds
- **Avg(us)**: Average time per call in microseconds
- **Min(us)**: Minimum time in microseconds
- **Max(us)**: Maximum time in microseconds

Example output:

```
=== Profiling Report ===
Total time: 1543 ms

Name                          Calls      Total(ms)  Avg(us)   Min(us)   Max(us)
--------------------------------------------------------------------------------
compute                       1000000    1234       1         0         15
io_operation                  5000       309        61        50        150
```

## Overview

The profiler module provides multiple profiling approaches:

1. **Timer**: Manual high-resolution timing for specific code sections
2. **Profiler**: Automatic instrumentation with enter/exit markers
3. **MemoryProfiler**: Track allocations, deallocations, and memory usage
4. **SampleProfiler**: Statistical profiling with periodic stack sampling
5. **FlameGraphBuilder**: Generate flame graph visualization data

All profilers use nanosecond-resolution timing and support nested/recursive profiling. The global profiler instances allow easy integration without passing profiler objects through the call chain.
