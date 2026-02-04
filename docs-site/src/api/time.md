# Time API Reference

> Time measurement, sleep operations, and Duration type

## Import

```vais
U std/time
```

## Duration Struct

```vais
S Duration { secs: i64, nanos: i64 }
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(secs: i64, nanos: i64) -> Duration` | Create with normalization |
| `from_secs` | `F from_secs(secs: i64) -> Duration` | From seconds |
| `from_millis` | `F from_millis(millis: i64) -> Duration` | From milliseconds |
| `from_micros` | `F from_micros(micros: i64) -> Duration` | From microseconds |
| `from_nanos` | `F from_nanos(nanos: i64) -> Duration` | From nanoseconds |
| `as_secs` | `F as_secs(&self) -> i64` | Total seconds |
| `as_millis` | `F as_millis(&self) -> i64` | Total milliseconds |
| `as_micros` | `F as_micros(&self) -> i64` | Total microseconds |
| `as_nanos` | `F as_nanos(&self) -> i64` | Total nanoseconds |
| `subsec_nanos` | `F subsec_nanos(&self) -> i64` | Subsecond nanoseconds component |
| `subsec_millis` | `F subsec_millis(&self) -> i64` | Subsecond milliseconds component |
| `subsec_micros` | `F subsec_micros(&self) -> i64` | Subsecond microseconds component |
| `add` | `F add(&self, other: Duration) -> Duration` | Add durations |
| `sub` | `F sub(&self, other: Duration) -> Duration` | Subtract durations |

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `time_now` | `F time_now() -> i64` | Unix timestamp (seconds) |
| `time_millis` | `F time_millis() -> i64` | Milliseconds since epoch |
| `time_micros` | `F time_micros() -> i64` | Microseconds since epoch |
| `sleep_millis` | `F sleep_millis(millis: i64) -> i64` | Sleep milliseconds |
| `sleep_micros` | `F sleep_micros(micros: i64) -> i64` | Sleep microseconds |
| `sleep_secs` | `F sleep_secs(secs: i64) -> i64` | Sleep seconds |
| `sleep` | `F sleep(millis: i64) -> i64` | Sleep (alias) |
| `sleep_duration` | `F sleep_duration(dur: Duration) -> i64` | Sleep for Duration |
| `elapsed_millis` | `F elapsed_millis(start: i64) -> i64` | Elapsed since start (ms) |
| `elapsed_micros` | `F elapsed_micros(start: i64) -> i64` | Elapsed since start (us) |
| `now_duration` | `F now_duration() -> Duration` | Current time as Duration |

## Usage

```vais
U std/time

F main() -> i64 {
    start := time_millis()
    sleep_millis(100)
    elapsed := elapsed_millis(start)  # ~100
    0
}
```
