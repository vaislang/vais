//! VAIS Code Formatter
//!
//! Pretty-prints VAIS AST to formatted source code.

use crate::*;

mod declarations;
mod expressions;
mod macros;
mod patterns;
mod statements;
mod types;

/// Formatter configuration
#[derive(Clone)]
pub struct FormatConfig {
    /// Indentation size in spaces
    pub indent_size: usize,
    /// Maximum line length
    pub max_line_length: usize,
    /// Use tabs instead of spaces
    pub use_tabs: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_size: 4,
            max_line_length: 100,
            use_tabs: false,
        }
    }
}

/// Code formatter that converts AST to formatted source code
pub struct Formatter {
    config: FormatConfig,
    output: String,
    indent_level: usize,
}

impl Formatter {
    pub fn new(config: FormatConfig) -> Self {
        Self {
            config,
            output: String::with_capacity(4096),
            indent_level: 0,
        }
    }

    /// Format a module
    pub fn format_module(&mut self, module: &Module) -> String {
        self.output.clear();
        self.indent_level = 0;

        let mut first = true;
        for item in &module.items {
            if !first {
                self.output.push('\n');
            }
            first = false;
            self.format_item(&item.node);
        }

        self.output.clone()
    }

    /// Get indentation string
    pub(crate) fn indent(&self) -> String {
        if self.config.use_tabs {
            "\t".repeat(self.indent_level)
        } else {
            " ".repeat(self.indent_level * self.config.indent_size)
        }
    }

    /// Push indentation
    pub(crate) fn push_indent(&mut self) {
        self.indent_level += 1;
    }

    /// Pop indentation
    pub(crate) fn pop_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
}
