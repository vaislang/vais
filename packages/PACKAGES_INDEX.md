# Vais Packages Index

Quick reference guide for all available Vais packages.

## Available Packages (10)

| Package | Version | Description | Key Features |
|---------|---------|-------------|--------------|
| [cli-args](#cli-args) | 1.0.0 | Command-line argument parser | Flags, named options, positional args |
| [env](#env) | 1.0.0 | Environment variable utilities | Get/set vars, common accessors, CI detection |
| [color](#color) | 1.0.0 | Terminal color output | ANSI colors, styling, semantic colors |
| [csv](#csv) | 1.0.0 | CSV parser and generator | Parse/write CSV, quoted fields, escaping |
| [toml-parser](#toml-parser) | 1.0.0 | TOML configuration parser | Parse TOML, key-value pairs, tables |
| [dotenv](#dotenv) | 1.0.0 | .env file loader | Load .env files, environment setup |
| [retry](#retry) | 1.0.0 | Retry logic with backoff | Exponential/linear backoff, configurable |
| [validate](#validate) | 1.0.0 | Input validation utilities | Email, URL, numeric, range, length checks |
| [cache](#cache) | 1.0.0 | In-memory LRU cache | LRU eviction, key-value storage, fixed capacity |
| [math-ext](#math-ext) | 1.0.0 | Extended math functions | GCD, LCM, primes, fibonacci, combinatorics |

## Quick Start

### Installation
```bash
# Install a package
vaisc pkg install cli-args

# Install multiple packages
vaisc pkg install env color validate
```

### Usage in Code
```vais
# Import package
U cli-args
U env
U color

F main() -> i64 {
    args := parse_args()
    home := env_home_dir()

    msg := green("Success!")
    puts_ptr(msg)
    free(msg)

    0
}
```

## Package Details

### cli-args
**Command-line argument parser**

```vais
U cli-args

args := parse_args()
I args.has_flag("help") {
    puts_ptr("Help message")
}
filename := args.get(0)
output := args.get_option("output")
```

**Main API:**
- `parse_args() -> Args`
- `Args.get(idx) -> i64`
- `Args.has_flag(name) -> i64`
- `Args.get_option(name) -> i64`

---

### env
**Environment variable utilities**

```vais
U env

home := env_home_dir()
env_set("MY_VAR", "value")

I env_is_ci() {
    puts_ptr("Running in CI")
}

port := env_get_i64("PORT", 8080)
```

**Main API:**
- `env_get(key) -> i64`
- `env_set(key, value) -> i64`
- `env_home_dir() -> i64`
- `env_is_ci() -> i64`

---

### color
**Terminal color output**

```vais
U color

msg := red("Error!")
puts_ptr(msg)
free(msg)

success_msg := success("Done!")
puts_ptr(success_msg)
free(success_msg)

I supports_color() {
    puts_ptr("Colors enabled")
}
```

**Main API:**
- `red(text) -> i64`, `green(text)`, `yellow(text)`, `blue(text)`
- `bold(text) -> i64`, `underline(text)`
- `success(text)`, `error(text)`, `warning(text)`, `info(text)`
- `supports_color() -> i64`

---

### csv
**CSV parser and generator**

```vais
U csv

line := "name,age,city"
row := csv_parse_line(line)

i := 0
L i < row.field_count {
    field := row.get(i)
    puts_ptr(field)
    i = i + 1
}

row.free()
```

**Main API:**
- `csv_parse_line(line) -> CsvRow`
- `csv_write_row(fields, count) -> i64`
- `CsvRow.get(idx) -> i64`
- `CsvReader.open(path) -> CsvReader`

---

### toml-parser
**TOML configuration parser**

```vais
U toml-parser

content := "name = \"app\"\nport = 8080\n"
config := toml_parse(content)

name := config.get_str("name")
port := config.get_int("port")

config.free()
```

**Main API:**
- `toml_parse(input) -> TomlTable`
- `TomlTable.get_str(key) -> i64`
- `TomlTable.get_int(key) -> i64`
- `TomlTable.get_bool(key) -> i64`

---

### dotenv
**.env file loader**

```vais
U dotenv

result := dotenv_load_default()

I result == 0 {
    api_key := dotenv_get("API_KEY")
    I api_key != 0 {
        puts_ptr("API key loaded")
    }
}
```

**Main API:**
- `dotenv_load(path) -> i64`
- `dotenv_load_default() -> i64`
- `dotenv_get(key) -> i64`
- `dotenv_has(key) -> i64`

---

### retry
**Retry logic with backoff**

```vais
U retry

config := RetryConfig.new(3, 100)
attempt := 0

L retry_should_continue(config, attempt) {
    success := do_operation()

    I success { B }

    delay := retry_delay_for(config, attempt)
    retry_sleep_ms(delay)
    attempt = attempt + 1
}
```

**Main API:**
- `RetryConfig.new(max, delay) -> RetryConfig`
- `retry_should_continue(config, attempt) -> i64`
- `retry_delay_for(config, attempt) -> i64`
- `retry_sleep_ms(ms) -> i64`

---

### validate
**Input validation utilities**

```vais
U validate

email := "user@example.com"
I is_email(email) {
    puts_ptr("Valid email")
}

I in_range(age, 0, 120) {
    puts_ptr("Valid age")
}

I min_length(password, 8) {
    puts_ptr("Password OK")
}
```

**Main API:**
- `is_email(s) -> i64`, `is_url(s)`
- `is_numeric(s) -> i64`, `is_alpha(s)`, `is_alphanumeric(s)`
- `in_range(value, min, max) -> i64`
- `min_length(s, min) -> i64`, `max_length(s, max)`

---

### cache
**In-memory LRU cache**

```vais
U cache

cache := Cache.new(10)

cache.put("key", "value")
value := cache.get("key")

I cache.has("key") {
    puts_ptr("Key exists")
}

cache.remove("key")
cache.clear()
cache.free()
```

**Main API:**
- `Cache.new(capacity) -> Cache`
- `cache.get(key) -> i64`
- `cache.put(key, value) -> i64`
- `cache.has(key) -> i64`
- `cache.remove(key) -> i64`

---

### math-ext
**Extended math functions**

```vais
U math-ext

a := abs(-42)           # 42
g := gcd(48, 18)        # 6
l := lcm(12, 18)        # 36
p := pow(2, 10)         # 1024

I is_prime(17) {
    puts_ptr("17 is prime")
}

fib := fibonacci(10)    # 55
fact := factorial(5)    # 120
```

**Main API:**
- `abs(x) -> i64`, `min(a, b)`, `max(a, b)`, `clamp(x, lo, hi)`
- `gcd(a, b) -> i64`, `lcm(a, b)`
- `pow(base, exp) -> i64`, `pow_fast(base, exp)`
- `is_prime(n) -> i64`, `fibonacci(n)`, `factorial(n)`

---

## Common Patterns

### CLI Application
```vais
U cli-args
U env
U color
U validate

F main() -> i64 {
    args := parse_args()

    # Help flag
    I args.has_flag("help") {
        show_help()
        R 0
    }

    # Verbose flag
    verbose := args.has_flag("verbose")

    # Get required argument
    input := args.get(0)
    I validate.is_not_empty(input) == 0 {
        err := color.error("Error: input file required")
        puts_ptr(err)
        free(err)
        R 1
    }

    # Get optional output file
    output := args.get_option("output")
    I output == 0 {
        output = "output.txt"
    }

    0
}
```

### Configuration Loading
```vais
U dotenv
U toml-parser
U validate

F load_config() -> TomlTable {
    # Load .env first
    dotenv_load(".env")

    # Read config file
    content := read_file("config.toml")
    config := toml_parse(content)

    # Validate
    port := config.get_int("port")
    I validate.in_range(port, 1, 65535) == 0 {
        puts_ptr("Invalid port")
    }

    config
}
```

### API Client with Retry
```vais
U retry
U cache
U validate

F fetch_data(url: i64) -> i64 {
    # Validate URL
    I validate.is_url(url) == 0 {
        R 0
    }

    # Check cache
    cache := Cache.new(100)
    cached := cache.get(url)
    I cached != 0 {
        R cached
    }

    # Fetch with retry
    config := RetryConfig.new(3, 1000)
    attempt := 0

    L retry_should_continue(config, attempt) {
        result := http_get(url)

        I result != 0 {
            cache.put(url, result)
            R result
        }

        delay := retry_delay_for(config, attempt)
        retry_sleep_ms(delay)
        attempt = attempt + 1
    }

    0
}
```

### Data Processing
```vais
U csv
U validate
U math-ext

F process_csv(path: i64) -> i64 {
    reader := CsvReader.open(path)
    sum := 0
    count := 0

    L 1 {
        row := reader.next_row()
        I row.field_count == 0 { B }

        value_str := row.get(1)
        I validate.is_numeric(value_str) {
            value := atol_ptr(value_str)
            sum = sum + value
            count = count + 1
        }

        row.free()
    }

    reader.close()

    # Calculate average
    I count > 0 {
        avg := sum / count
        printf("Average: %d\n", avg)
    }

    0
}
```

## Package Development

### Creating a Package

1. Create directory structure:
```bash
mkdir -p packages/my-package/src
```

2. Create `vais.toml`:
```toml
[package]
name = "my-package"
version = "1.0.0"
authors = ["Your Name"]
description = "Package description"
license = "MIT"

[dependencies]

[dev-dependencies]

[build]
```

3. Write `src/lib.vais`:
```vais
# My Package
# Description

F my_function(arg: i64) -> i64 {
    # Implementation
    arg
}
```

4. Create `README.md` with documentation

### Publishing a Package

```bash
# Build and test
vaisc build packages/my-package

# Publish to registry
vaisc pkg publish packages/my-package
```

## Resources

- [Package Registry](https://registry.vais.dev) - Browse and search packages
- [Package Development Guide](./docs/package-development.md) - How to create packages
- [Contributing](./CONTRIBUTING.md) - Contribution guidelines
- [License](./LICENSE) - MIT License

## Support

- Issues: [GitHub Issues](https://github.com/vais-lang/vais/issues)
- Discussions: [GitHub Discussions](https://github.com/vais-lang/vais/discussions)
- Chat: [Discord](https://discord.gg/vais)

---

**Last Updated**: 2026-02-04
**Total Packages**: 10
**Vais Version**: 0.1.0
