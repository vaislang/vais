# Args

Command-line argument parsing.

**Module:** `std/args.vais`

## Types

### `ArgParser`

Builder-style argument parser.

```vais
S ArgParser {
    # internal implementation
}
```

## Methods

### `add_flag(name: str, short: str, help: str)`

Adds a boolean flag.

```vais
parser := ArgParser {}
parser.add_flag("verbose", "v", "Enable verbose output")
```

### `add_option(name: str, short: str, help: str, default: str)`

Adds a named option with a default value.

```vais
parser.add_option("output", "o", "Output file path", "out.txt")
```

### `add_positional(name: str, help: str)`

Adds a positional argument.

```vais
parser.add_positional("input", "Input file")
```

### `parse()`

Parses command-line arguments. Call after adding all flags/options.

```vais
parser.parse()
```

### `get_flag(name: str) -> bool`

Returns whether a flag was set.

```vais
I parser.get_flag("verbose") {
    puts("Verbose mode enabled")
}
```

### `get_option(name: str) -> str`

Returns the value of a named option.

```vais
output := parser.get_option("output")
```

### `print_help()`

Prints the help message with all registered flags and options.

## Example

```vais
F main() -> i64 {
    parser := ArgParser {}
    parser.add_flag("verbose", "v", "Enable verbose output")
    parser.add_option("output", "o", "Output file", "a.out")
    parser.add_positional("input", "Source file")
    parser.parse()

    I parser.get_flag("verbose") {
        puts("Compiling with verbose output...")
    }

    output := parser.get_option("output")
    puts("Output: {output}")
    0
}
```

## See Also

- [IO](./io.md) — standard I/O functions
