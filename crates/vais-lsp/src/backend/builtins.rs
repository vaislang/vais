//! Builtin function information and utility functions for the LSP backend

use tower_lsp::lsp_types::*;

/// Check if a position is within a range
pub(crate) fn position_in_range(position: &Position, range: &Range) -> bool {
    if position.line < range.start.line || position.line > range.end.line {
        return false;
    }
    if position.line == range.start.line && position.character < range.start.character {
        return false;
    }
    if position.line == range.end.line && position.character > range.end.character {
        return false;
    }
    true
}

/// Get hover information for builtin functions
pub(crate) fn get_builtin_hover(name: &str) -> Option<Hover> {
    let info = match name {
        "puts" => Some(("fn(str) -> i64", "Print a string to stdout with newline")),
        "putchar" => Some(("fn(i64) -> i64", "Print a single character (ASCII value)")),
        "print_i64" => Some(("fn(i64) -> i64", "Print a 64-bit signed integer")),
        "print_f64" => Some(("fn(f64) -> i64", "Print a 64-bit floating point number")),
        "malloc" => Some((
            "fn(i64) -> i64",
            "Allocate `size` bytes of heap memory, returns pointer",
        )),
        "free" => Some(("fn(i64) -> i64", "Free heap memory at pointer")),
        "memcpy" => Some((
            "fn(i64, i64, i64) -> i64",
            "Copy `n` bytes from `src` to `dst`",
        )),
        "strlen" => Some(("fn(i64) -> i64", "Get length of null-terminated string")),
        "load_i64" => Some((
            "fn(i64) -> i64",
            "Load a 64-bit integer from memory address",
        )),
        "store_i64" => Some((
            "fn(i64, i64) -> i64",
            "Store a 64-bit integer to memory address",
        )),
        "load_byte" => Some(("fn(i64) -> i64", "Load a single byte from memory address")),
        "store_byte" => Some((
            "fn(i64, i64) -> i64",
            "Store a single byte to memory address",
        )),
        "sqrt" => Some(("fn(f64) -> f64", "Square root (from std/math)")),
        "sin" => Some(("fn(f64) -> f64", "Sine function (from std/math)")),
        "cos" => Some(("fn(f64) -> f64", "Cosine function (from std/math)")),
        "tan" => Some(("fn(f64) -> f64", "Tangent function (from std/math)")),
        "pow" => Some(("fn(f64, f64) -> f64", "Power function x^y (from std/math)")),
        "log" => Some(("fn(f64) -> f64", "Natural logarithm (from std/math)")),
        "exp" => Some(("fn(f64) -> f64", "Exponential e^x (from std/math)")),
        "floor" => Some(("fn(f64) -> f64", "Round down to integer (from std/math)")),
        "ceil" => Some(("fn(f64) -> f64", "Round up to integer (from std/math)")),
        "round" => Some(("fn(f64) -> f64", "Round to nearest integer (from std/math)")),
        "abs" => Some(("fn(f64) -> f64", "Absolute value for f64 (from std/math)")),
        "abs_i64" => Some(("fn(i64) -> i64", "Absolute value for i64 (from std/math)")),
        "min" => Some((
            "fn(f64, f64) -> f64",
            "Minimum of two f64 values (from std/math)",
        )),
        "max" => Some((
            "fn(f64, f64) -> f64",
            "Maximum of two f64 values (from std/math)",
        )),
        "PI" => Some((
            "const f64 = 3.14159...",
            "Mathematical constant \u{03C0} (from std/math)",
        )),
        "TAU" => Some((
            "const f64 = 6.28318...",
            "Mathematical constant \u{03C4} = 2\u{03C0} (from std/math)",
        )),
        "read_i64" => Some(("fn() -> i64", "Read integer from stdin (from std/io)")),
        "read_f64" => Some(("fn() -> f64", "Read float from stdin (from std/io)")),
        "read_line" => Some(("fn(i64, i64) -> i64", "Read line into buffer (from std/io)")),
        "read_char" => Some(("fn() -> i64", "Read single character (from std/io)")),
        _ => None,
    };

    info.map(|(sig, doc)| Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("```vais\n{}\n```\n\n{}\n\n*Built-in function*", sig, doc),
        }),
        range: None,
    })
}
