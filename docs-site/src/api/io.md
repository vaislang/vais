# IO API Reference

> Standard input/output operations for reading from stdin

## Import

```vais
U std/io
```

## Constants

| Name | Value | Description |
|------|-------|-------------|
| `INPUT_BUFFER_SIZE` | 1024 | Default buffer size for input |

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `read_line` | `F read_line(buffer: i64, max_len: i64) -> i64` | Read line from stdin into buffer |
| `read_i64` | `F read_i64() -> i64` | Read integer from stdin |
| `read_f64` | `F read_f64() -> f64` | Read float from stdin |
| `read_char` | `F read_char() -> i64` | Read single character from stdin |
| `prompt_line` | `F prompt_line(prompt: i64, buffer: i64, max_len: i64) -> i64` | Print prompt, read line |
| `prompt_i64` | `F prompt_i64(prompt: i64) -> i64` | Print prompt, read integer |
| `prompt_f64` | `F prompt_f64(prompt: i64) -> f64` | Print prompt, read float |

## Usage

```vais
U std/io

F main() -> i64 {
    age := prompt_i64("Enter your age: ")
    name_buf := malloc(256)
    prompt_line("Enter name: ", name_buf, 256)
    free(name_buf)
    0
}
```
