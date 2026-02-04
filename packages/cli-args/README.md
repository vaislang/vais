# cli-args

Command-line argument parser for Vais programs.

## Features

- Parse command-line arguments
- Check for flags (`-v`, `--verbose`)
- Get named options (`--output=file`, `--output file`)
- Access positional arguments

## Usage

```vais
U cli-args

F main() -> i64 {
    args := parse_args()

    # Check argument count
    I args.count == 0 {
        puts_ptr("No arguments provided")
        R 0
    }

    # Check for help flag
    I args.has_flag("help") {
        puts_ptr("Usage: program [options] [args]")
        R 0
    }

    # Check for verbose flag
    verbose := args.has_flag("verbose")
    I verbose {
        puts_ptr("Verbose mode enabled")
    }

    # Get output file option
    output := args.get_option("output")
    I output != 0 {
        puts_ptr("Output file: ")
        puts_ptr(output)
    }

    # Get first positional argument
    first := args.get(0)
    I first != 0 {
        puts_ptr("First arg: ")
        puts_ptr(first)
    }

    0
}
```

## API

### Types

- `Args` - Parsed arguments structure with count and program name

### Functions

- `parse_args() -> Args` - Parse command-line arguments
- `Args.get(idx: i64) -> i64` - Get argument at index (0-based, excludes program name)
- `Args.has_flag(name: i64) -> i64` - Check if flag is present
- `Args.get_option(name: i64) -> i64` - Get value for named option
- `Args.program_name() -> i64` - Get program name

### Extern Functions

- `get_argc() -> i64` - Get argument count (requires C runtime)
- `get_argv(idx: i64) -> i64` - Get argument string at index (requires C runtime)

## Examples

```bash
# Check for help flag
./program --help

# Set output file
./program --output=result.txt
./program --output result.txt

# Verbose mode with positional args
./program -v input.txt output.txt
```

## License

MIT
