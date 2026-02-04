# toml-parser

TOML configuration file parser for Vais programs.

## Features

- Parse TOML key-value pairs
- Support for strings, integers, and booleans
- Handle comments (#)
- Table sections ([section])
- Simple and lightweight

## Usage

```vais
U toml-parser

F main() -> i64 {
    toml_content := "
# Configuration file
name = \"my-app\"
version = 1
debug = true

[database]
host = \"localhost\"
port = 5432
"

    table := toml_parse(toml_content)

    # Get string value
    name := table.get_str("name")
    I name != 0 {
        puts_ptr("Name: ")
        puts_ptr(name)
    }

    # Get integer value
    version := table.get_int("version")
    printf("Version: %d\n", version)

    # Get boolean value
    debug := table.get_bool("debug")
    I debug {
        puts_ptr("Debug mode enabled")
    }

    # Check if key exists
    I table.has_key("version") {
        puts_ptr("Version key exists")
    }

    # Free table
    table.free()

    0
}
```

## API

### Types

- `TomlValue` - Represents a TOML value (string, integer, or boolean)
- `TomlTable` - Represents a TOML table with key-value pairs

### TomlValue Methods

- `TomlValue.from_str(s: i64) -> TomlValue` - Create string value
- `TomlValue.from_int(i: i64) -> TomlValue` - Create integer value
- `TomlValue.from_bool(b: i64) -> TomlValue` - Create boolean value
- `value.free() -> i64` - Free value memory

### TomlTable Methods

- `TomlTable.new() -> TomlTable` - Create new empty table
- `table.get_str(key: i64) -> i64` - Get string value by key
- `table.get_int(key: i64) -> i64` - Get integer value by key
- `table.get_bool(key: i64) -> i64` - Get boolean value by key
- `table.has_key(key: i64) -> i64` - Check if key exists
- `table.set(key: i64, value: TomlValue) -> i64` - Add key-value pair
- `table.free() -> i64` - Free table memory

### Functions

- `toml_parse(input: i64) -> TomlTable` - Parse TOML string

## Supported TOML Features

### Key-Value Pairs
```toml
key = "value"
number = 42
flag = true
```

### Comments
```toml
# This is a comment
key = "value"  # Inline comments not supported
```

### Tables (sections)
```toml
[section]
key = "value"
```

## Limitations

- No arrays
- No inline tables
- No multi-line strings
- No dotted keys
- No nested tables
- No date/time types
- Basic value types only (string, integer, boolean)

## Examples

```vais
# Parse configuration file
content := "
app_name = \"MyApp\"
port = 8080
debug = false
"

table := toml_parse(content)

app_name := table.get_str("app_name")
port := table.get_int("port")
debug := table.get_bool("debug")

printf("%s running on port %d\n", app_name, port)

table.free()
```

## License

MIT
