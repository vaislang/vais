# color

Terminal color output utilities using ANSI escape codes.

## Features

- Colorize text output (red, green, yellow, blue, magenta, cyan, white, black)
- Text styling (bold, dim, italic, underline)
- Background colors
- Semantic colors (success, error, warning, info)
- Color support detection
- Strip ANSI codes from strings

## Usage

```vais
U color

F main() -> i64 {
    # Basic colors
    msg := red("Error!")
    puts_ptr(msg)
    free(msg)

    msg = green("Success!")
    puts_ptr(msg)
    free(msg)

    # Text styling
    msg = bold("Important")
    puts_ptr(msg)
    free(msg)

    # Semantic colors
    msg = error("Failed to open file")
    puts_ptr(msg)
    free(msg)

    msg = success("Task completed")
    puts_ptr(msg)
    free(msg)

    # Check color support
    I supports_color() {
        puts_ptr("Terminal supports colors")
    } E {
        puts_ptr("Terminal may not support colors")
    }

    0
}
```

## API

### Foreground Colors

- `red(text: i64) -> i64` - Red text
- `green(text: i64) -> i64` - Green text
- `yellow(text: i64) -> i64` - Yellow text
- `blue(text: i64) -> i64` - Blue text
- `magenta(text: i64) -> i64` - Magenta text
- `cyan(text: i64) -> i64` - Cyan text
- `white(text: i64) -> i64` - White text
- `black(text: i64) -> i64` - Black text

### Text Styling

- `bold(text: i64) -> i64` - Bold text
- `dim(text: i64) -> i64` - Dimmed text
- `italic(text: i64) -> i64` - Italic text
- `underline(text: i64) -> i64` - Underlined text

### Background Colors

- `bg_red(text: i64) -> i64` - Red background
- `bg_green(text: i64) -> i64` - Green background
- `bg_yellow(text: i64) -> i64` - Yellow background
- `bg_blue(text: i64) -> i64` - Blue background
- `bg_magenta(text: i64) -> i64` - Magenta background
- `bg_cyan(text: i64) -> i64` - Cyan background
- `bg_white(text: i64) -> i64` - White background
- `bg_black(text: i64) -> i64` - Black background

### Semantic Colors

- `success(text: i64) -> i64` - Success message (green)
- `error(text: i64) -> i64` - Error message (red)
- `warning(text: i64) -> i64` - Warning message (yellow)
- `info(text: i64) -> i64` - Info message (cyan)

### Utilities

- `supports_color() -> i64` - Check if terminal supports colors
- `strip_ansi(text: i64) -> i64` - Remove ANSI codes from string
- `ansi_format(code: i64, text: i64) -> i64` - Custom ANSI formatting

## Notes

- All color functions return allocated strings that must be freed by the caller
- Color support is detected via the `TERM` environment variable
- Set `NO_COLOR` environment variable to disable colors
- ANSI codes work on most modern terminals (Unix, macOS, Windows 10+)

## Examples

```vais
# Error messages
err_msg := error("Connection failed")
puts_ptr(err_msg)
free(err_msg)

# Success with bold
msg := bold(success("Build completed"))
puts_ptr(msg)
free(msg)

# Custom formatting
msg = ansi_format(31, "Custom red")  # 31 = red
puts_ptr(msg)
free(msg)
```

## License

MIT
