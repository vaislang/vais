//! Contract helper utilities.

use std::fmt::Write;
use crate::CodeGenerator;

impl CodeGenerator {
    pub(super) fn get_or_create_contract_string(&mut self, s: &str) -> String {
        // Check if we already have this string
        if let Some(name) = self.contracts.contract_constants.get(s) {
            return format!(
                "getelementptr inbounds ([{} x i8], [{} x i8]* {}, i64 0, i64 0)",
                s.len() + 1,
                s.len() + 1,
                name
            );
        }

        // Create a new string constant
        let const_name = format!("@.str.contract.{}", self.contracts.contract_counter);
        self.contracts.contract_counter += 1;

        let gep_expr = format!(
            "getelementptr inbounds ([{} x i8], [{} x i8]* {}, i64 0, i64 0)",
            s.len() + 1,
            s.len() + 1,
            const_name
        );

        self.contracts
            .contract_constants
            .insert(s.to_string(), const_name);

        gep_expr
    }

}

/// Escape a string for LLVM IR constant
pub(super) fn escape_string_for_llvm(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\5C"),
            '"' => result.push_str("\\22"),
            '\n' => result.push_str("\\0A"),
            '\r' => result.push_str("\\0D"),
            '\t' => result.push_str("\\09"),
            c if c.is_ascii_graphic() || c == ' ' => result.push(c),
            c => {
                // Escape non-printable characters as hex
                for byte in c.to_string().as_bytes() {
                    write!(result, "\\{:02X}", byte).unwrap();
                }
            }
        }
    }
    result
}
