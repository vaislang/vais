# retry

Retry logic with exponential backoff for Vais programs.

## Features

- Configurable max retries
- Exponential backoff with configurable factor
- Linear backoff option
- Immediate retries (no delay)
- Delay calculation and capping
- Simple decision helpers

## Usage

```vais
U retry

F main() -> i64 {
    # Create retry configuration (3 retries, 100ms initial delay)
    config := RetryConfig.new(3, 100)

    attempt := 0
    L retry_should_continue(config, attempt) {
        puts_ptr("Attempting operation...")

        # Try operation
        success := do_operation()

        I success {
            puts_ptr("Success!")
            B
        }

        # Failed, calculate backoff delay
        delay := retry_delay_for(config, attempt)
        printf("Failed, retrying in %dms...\n", delay)

        retry_sleep_ms(delay)
        attempt = attempt + 1
    }

    I attempt >= config.max_retries {
        puts_ptr("All retries exhausted")
        R 1
    }

    0
}

F do_operation() -> i64 {
    # Simulated operation that might fail
    0  # Returns 0 for failure, 1 for success
}
```

## API

### Types

- `RetryConfig` - Retry configuration with max retries, delays, and backoff

### RetryConfig Methods

- `RetryConfig.new(max: i64, delay: i64) -> RetryConfig` - Create config with max retries and initial delay
- `RetryConfig.default() -> RetryConfig` - Default config (3 retries, 100ms delay)
- `RetryConfig.with_backoff(max: i64, delay: i64, backoff: i64) -> RetryConfig` - Config with custom backoff
- `config.with_max_delay(max_delay: i64) -> RetryConfig` - Set max delay cap

### Core Functions

- `retry_should_continue(config: RetryConfig, attempt: i64) -> i64` - Check if should retry
- `retry_delay_for(config: RetryConfig, attempt: i64) -> i64` - Calculate delay for attempt
- `retry_sleep_ms(ms: i64) -> i64` - Sleep for milliseconds
- `retry_execute(config: RetryConfig) -> i64` - Execute retry loop (simplified)

### Helper Functions

- `retry_linear(max_retries: i64, delay_ms: i64) -> RetryConfig` - Linear backoff (no exponential)
- `retry_immediate(max_retries: i64) -> RetryConfig` - No delay between retries
- `retry_next_delay(current: i64, backoff: i64) -> i64` - Calculate next delay
- `retry_total_time(config: RetryConfig) -> i64` - Total time for all retries
- `retry_is_first_attempt(attempt: i64) -> i64` - Check if first attempt
- `retry_is_last_attempt(config: RetryConfig, attempt: i64) -> i64` - Check if last attempt
- `retry_progress(config: RetryConfig, attempt: i64) -> i64` - Get progress (0-100)

## Examples

### Exponential Backoff

```vais
# Default: 3 retries with 100ms, 200ms, 400ms delays
config := RetryConfig.default()

# Attempt 0: 100ms delay
# Attempt 1: 200ms delay
# Attempt 2: 400ms delay
```

### Custom Backoff

```vais
# 5 retries, 50ms initial, 3x backoff
config := RetryConfig.with_backoff(5, 50, 3)

# Delays: 50ms, 150ms, 450ms, 1350ms, 4050ms
```

### Linear Backoff

```vais
# 4 retries with fixed 500ms delay
config := retry_linear(4, 500)

# Delays: 500ms, 500ms, 500ms, 500ms
```

### Immediate Retries

```vais
# 10 retries with no delay
config := retry_immediate(10)
```

### Network Request Example

```vais
F fetch_data() -> i64 {
    config := RetryConfig.new(3, 1000)  # 3 retries, 1s initial

    attempt := 0
    L retry_should_continue(config, attempt) {
        result := http_get("https://api.example.com/data")

        I result != 0 {
            R result  # Success
        }

        I retry_is_last_attempt(config, attempt) {
            puts_ptr("Final attempt failed")
            R 0
        }

        delay := retry_delay_for(config, attempt)
        printf("Retry in %dms...\n", delay)
        retry_sleep_ms(delay)

        attempt = attempt + 1
    }

    0  # Failed
}
```

## Constants

- `DEFAULT_MAX_RETRIES = 3`
- `DEFAULT_DELAY_MS = 100`
- `DEFAULT_BACKOFF_FACTOR = 2`
- `MAX_DELAY_MS = 30000` (30 seconds)

## Notes

- Delays are capped at `MAX_DELAY_MS` (30 seconds)
- Attempt numbers are 0-based
- Exponential backoff: `delay = initial * (factor ^ attempt)`
- `retry_sleep_ms()` requires `usleep()` system call

## License

MIT
