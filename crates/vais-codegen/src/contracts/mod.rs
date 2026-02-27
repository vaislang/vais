//! Contract code generation for Design by Contract
//!
//! Generates LLVM IR for requires (preconditions) and ensures (postconditions).
//! Contract checks are only generated in debug builds.

mod assert_assume;
mod auto_checks;
mod decreases;
mod ensures;
mod helpers;
mod invariants;
mod requires;

use crate::CodeGenerator;
use helpers::escape_string_for_llvm;

impl CodeGenerator {
    /// Generate declarations for contract runtime functions
    pub(crate) fn generate_contract_declarations(&self) -> String {
        // Only generate if we have any contracts
        if self.contracts.contract_constants.is_empty() && self.release_mode {
            return String::new();
        }

        let mut ir = String::new();
        ir.push_str("; Contract runtime declarations\n");
        // Note: __contract_fail and __panic are now defined in generate_helper_functions()
        // LLVM assume intrinsic for optimization hints (used by assume() in release mode)
        ir.push_str("declare void @llvm.assume(i1)\n");
        ir.push('\n');

        ir
    }

    /// Generate string constants for contract messages
    pub(crate) fn generate_contract_string_constants(&self) -> String {
        let mut ir = String::new();

        for (s, name) in &self.contracts.contract_constants {
            let escaped = escape_string_for_llvm(s);
            use std::fmt::Write;
            writeln!(
                ir,
                "{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"",
                name,
                s.len() + 1,
                escaped
            )
            .unwrap();
        }

        ir
    }
}
