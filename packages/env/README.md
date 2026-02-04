# env

Environment variable utilities for Vais programs.

## Features

- Get, set, and remove environment variables
- Query common environment variables (HOME, PATH, USER, etc.)
- Detect CI environments
- Parse integer values from environment

## Usage

```vais
U env

F main() -> i64 {
    # Get environment variable
    home := env_home_dir()
    I home != 0 {
        puts_ptr("HOME: ")
        puts_ptr(home)
    }

    # Set environment variable
    success := env_set("MY_VAR", "my_value")
    I success {
        puts_ptr("Variable set successfully")
    }

    # Check if variable exists
    I env_exists("PATH") {
        path := env_path()
        puts_ptr("PATH: ")
        puts_ptr(path)
    }

    # Get integer from environment
    port := env_get_i64("PORT", 8080)
    printf("Using port: %d\n", port)

    # Check if running in CI
    I env_is_ci() {
        puts_ptr("Running in CI environment")
    }

    0
}
```

## API

### Core Functions

- `env_get(key: i64) -> i64` - Get environment variable value
- `env_set(key: i64, value: i64) -> i64` - Set environment variable
- `env_set_if_not_exists(key: i64, value: i64) -> i64` - Set only if not exists
- `env_remove(key: i64) -> i64` - Remove environment variable
- `env_exists(key: i64) -> i64` - Check if variable exists

### Common Variables

- `env_home_dir() -> i64` - Get HOME directory
- `env_user() -> i64` - Get USER name
- `env_path() -> i64` - Get PATH
- `env_shell() -> i64` - Get SHELL
- `env_pwd() -> i64` - Get current working directory
- `env_temp_dir() -> i64` - Get temp directory (TMPDIR/TMP)
- `env_os_name() -> i64` - Get OS name
- `env_lang() -> i64` - Get LANG locale
- `env_editor() -> i64` - Get EDITOR (defaults to "vi")
- `env_term() -> i64` - Get TERM type

### Utilities

- `env_is_ci() -> i64` - Check if running in CI environment
- `env_get_i64(key: i64, default_val: i64) -> i64` - Get integer value

### Extern Functions

- `getenv(name: i64) -> i64` - C library getenv
- `setenv(name: i64, value: i64, overwrite: i64) -> i64` - C library setenv
- `unsetenv(name: i64) -> i64` - C library unsetenv

## Examples

```vais
# Get user's home directory
home := env_home_dir()

# Set custom variable
env_set("APP_MODE", "production")

# Get with default
timeout := env_get_i64("TIMEOUT", 30)

# Check CI environment
I env_is_ci() {
    puts_ptr("Running CI tests...")
}
```

## License

MIT
