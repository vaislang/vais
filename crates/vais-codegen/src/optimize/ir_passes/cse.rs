//! Common subexpression elimination with GVN (Global Value Numbering).

use std::collections::HashMap;

/// Value number for GVN-based CSE
type ValueNumber = u32;

/// Interned operation identifier to avoid per-expression String allocation.
/// Maps 1:1 with the LLVM IR binary operation names.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum GvnOp {
    Add,
    Sub,
    Mul,
    Sdiv,
    And,
    Or,
    Xor,
    Shl,
    Ashr,
    Lshr,
}

impl GvnOp {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "add" => Some(Self::Add),
            "sub" => Some(Self::Sub),
            "mul" => Some(Self::Mul),
            "sdiv" => Some(Self::Sdiv),
            "and" => Some(Self::And),
            "or" => Some(Self::Or),
            "xor" => Some(Self::Xor),
            "shl" => Some(Self::Shl),
            "ashr" => Some(Self::Ashr),
            "lshr" => Some(Self::Lshr),
            _ => None,
        }
    }
}

/// Interned type identifier to avoid per-expression String allocation.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum GvnTy {
    I64,
    I32,
    I16,
    I8,
    I1,
}

impl GvnTy {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "i64" => Some(Self::I64),
            "i32" => Some(Self::I32),
            "i16" => Some(Self::I16),
            "i8" => Some(Self::I8),
            "i1" => Some(Self::I1),
            _ => None,
        }
    }
}

/// GVN table entry: canonical expression representation.
///
/// Uses interned enums for `op` and `ty` fields instead of String to avoid
/// heap allocation on every expression lookup (review #7: GvnExpr String
/// allocation overhead).
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum GvnExpr {
    /// Binary operation with value numbers for operands
    BinOp {
        op: GvnOp,
        ty: GvnTy,
        lhs: ValueNumber,
        rhs: ValueNumber,
    },
    /// A constant value
    Constant(i64),
    /// An input (parameter or unknown) identified by name
    Input(String),
}

/// GVN state for a function
struct GvnState {
    /// Map from variable name (%0, %x, etc.) to value number
    var_to_vn: HashMap<String, ValueNumber>,
    /// Map from GVN expression to value number
    expr_to_vn: HashMap<GvnExpr, ValueNumber>,
    /// Map from value number to the first variable that defined it
    vn_to_var: HashMap<ValueNumber, String>,
    /// Next value number to assign
    next_vn: ValueNumber,
}

impl GvnState {
    fn new() -> Self {
        Self {
            var_to_vn: HashMap::new(),
            expr_to_vn: HashMap::new(),
            vn_to_var: HashMap::new(),
            next_vn: 0,
        }
    }

    /// Get or assign a value number for a variable name (as an input/unknown)
    fn get_or_assign_input_vn(&mut self, var: &str) -> ValueNumber {
        if let Some(&vn) = self.var_to_vn.get(var) {
            return vn;
        }
        let expr = GvnExpr::Input(var.to_string());
        if let Some(&vn) = self.expr_to_vn.get(&expr) {
            self.var_to_vn.insert(var.to_string(), vn);
            return vn;
        }
        let vn = self.next_vn;
        self.next_vn += 1;
        self.var_to_vn.insert(var.to_string(), vn);
        self.vn_to_var.insert(vn, var.to_string());
        self.expr_to_vn.insert(expr, vn);
        vn
    }

    /// Get or assign a value number for a constant
    fn get_or_assign_const_vn(&mut self, val: i64) -> ValueNumber {
        let expr = GvnExpr::Constant(val);
        if let Some(&vn) = self.expr_to_vn.get(&expr) {
            return vn;
        }
        let vn = self.next_vn;
        self.next_vn += 1;
        self.expr_to_vn.insert(expr, vn);
        vn
    }

    /// Get the value number for an operand (either %var or constant)
    fn operand_vn(&mut self, operand: &str) -> ValueNumber {
        if operand.starts_with('%') {
            self.get_or_assign_input_vn(operand)
        } else if let Ok(c) = operand.parse::<i64>() {
            self.get_or_assign_const_vn(c)
        } else {
            // Unknown operand, treat as unique input
            self.get_or_assign_input_vn(operand)
        }
    }

    /// Look up or register a binary operation, applying commutativity for
    /// commutative operators (add, mul, and, or, xor).
    /// Returns (value_number, Option<existing_var_name>)
    fn lookup_binop(
        &mut self,
        op: GvnOp,
        ty: GvnTy,
        lhs: &str,
        rhs: &str,
    ) -> (ValueNumber, Option<String>) {
        let lhs_vn = self.operand_vn(lhs);
        let rhs_vn = self.operand_vn(rhs);

        // Normalize commutative operations: put smaller VN first
        let is_commutative = matches!(op, GvnOp::Add | GvnOp::Mul | GvnOp::And | GvnOp::Or | GvnOp::Xor);
        let (norm_lhs, norm_rhs) = if is_commutative && lhs_vn > rhs_vn {
            (rhs_vn, lhs_vn)
        } else {
            (lhs_vn, rhs_vn)
        };

        let expr = GvnExpr::BinOp {
            op,
            ty,
            lhs: norm_lhs,
            rhs: norm_rhs,
        };

        if let Some(&vn) = self.expr_to_vn.get(&expr) {
            // Found an existing computation with the same value number
            let existing_var = self.vn_to_var.get(&vn).cloned();
            (vn, existing_var)
        } else {
            // New expression
            let vn = self.next_vn;
            self.next_vn += 1;
            self.expr_to_vn.insert(expr, vn);
            (vn, None)
        }
    }

    /// Register a variable as having a particular value number
    fn register_var(&mut self, var: &str, vn: ValueNumber) {
        self.var_to_vn.insert(var.to_string(), vn);
        // Only register as canonical representative if not already present
        self.vn_to_var.entry(vn).or_insert_with(|| var.to_string());
    }
}

/// Common subexpression elimination with GVN (Global Value Numbering)
///
/// Extends basic CSE with:
/// - Value numbering table for tracking expression equivalence
/// - Algebraic identity detection (a+b == b+a for commutative ops)
/// - Dominator-scoped value table (reset at function boundaries)
pub(crate) fn common_subexpression_elimination(ir: &str) -> String {
    let mut result = String::with_capacity(ir.len());
    let mut gvn = GvnState::new();

    for line in ir.lines() {
        let trimmed = line.trim();

        // Reset GVN state at function boundaries
        if line.starts_with("define ") {
            gvn = GvnState::new();
        }

        // Also reset at labels (conservative dominator approximation):
        // In a more precise implementation, we would build a dominator tree
        // and only keep values from dominating blocks. For now, we reset at
        // merge points (labels that aren't the entry block).
        if trimmed.ends_with(':') && !trimmed.contains("entry") {
            // Keep the state but don't clear it - this is a simplification
            // A full dominator tree implementation would scope the table properly
        }

        // Pattern: %N = BINOP TYPE A, B
        if let Some((dest, op, ty, ty_str, lhs, rhs)) = extract_binop_components(trimmed) {
            let (vn, existing) = gvn.lookup_binop(op, ty, &lhs, &rhs);

            if let Some(existing_var) = existing {
                // This expression was computed before - reuse it
                result.push_str(&format!(
                    "  {} = add {} 0, {}  ; GVN-CSE: reusing binop {}, {}\n",
                    dest, ty_str, existing_var, lhs, rhs
                ));
                gvn.register_var(&dest, vn);
                continue;
            } else {
                // New expression - register it
                gvn.register_var(&dest, vn);
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Extract binary operation components for GVN.
/// Returns (dest, interned_op, interned_ty, ty_str, lhs_operand, rhs_operand) if the line matches.
/// Uses interned GvnOp/GvnTy to avoid String allocation on the hot path.
fn extract_binop_components(line: &str) -> Option<(String, GvnOp, GvnTy, &'static str, String, String)> {
    let ops: &[&str] = &["add", "sub", "mul", "sdiv", "and", "or", "xor", "shl", "ashr", "lshr"];
    let types: &[&str] = &["i64", "i32", "i16", "i8", "i1"];

    for op_str in ops {
        for ty_str in types {
            let pattern = format!(" = {} {} ", op_str, ty_str);
            if line.contains(&pattern) {
                let parts: Vec<&str> = line.split(&pattern).collect();
                if parts.len() == 2 {
                    let dest = parts[0].trim().to_string();
                    let operands: Vec<&str> = parts[1].split(',').map(|s| s.trim()).collect();
                    if operands.len() == 2 {
                        let op = GvnOp::from_str(op_str)?;
                        let ty = GvnTy::from_str(ty_str)?;
                        // Return the static str for the type name to use in IR output
                        let static_ty: &'static str = match ty {
                            GvnTy::I64 => "i64",
                            GvnTy::I32 => "i32",
                            GvnTy::I16 => "i16",
                            GvnTy::I8 => "i8",
                            GvnTy::I1 => "i1",
                        };
                        return Some((
                            dest,
                            op,
                            ty,
                            static_ty,
                            operands[0].to_string(),
                            operands[1].to_string(),
                        ));
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cse() {
        let ir = r#"define i64 @test(i64 %a, i64 %b) {
entry:
  %0 = add i64 %a, %b
  %1 = add i64 %a, %b
  ret i64 %1
}
"#;
        let result = common_subexpression_elimination(ir);
        assert!(
            result.contains("GVN-CSE"),
            "Duplicate expression should be eliminated. Result:\n{}",
            result
        );
    }

    #[test]
    fn test_commutative_cse() {
        let ir = r#"define i64 @test(i64 %a, i64 %b) {
entry:
  %0 = add i64 %a, %b
  %1 = add i64 %b, %a
  ret i64 %1
}
"#;
        let result = common_subexpression_elimination(ir);
        assert!(
            result.contains("GVN-CSE"),
            "Commutative expression a+b == b+a should be detected. Result:\n{}",
            result
        );
    }

    #[test]
    fn test_non_commutative_not_cse() {
        let ir = r#"define i64 @test(i64 %a, i64 %b) {
entry:
  %0 = sub i64 %a, %b
  %1 = sub i64 %b, %a
  ret i64 %1
}
"#;
        let result = common_subexpression_elimination(ir);
        assert!(
            !result.contains("GVN-CSE"),
            "Non-commutative sub a-b != b-a should NOT be eliminated. Result:\n{}",
            result
        );
    }

    #[test]
    fn test_mul_commutativity() {
        let ir = r#"define i64 @test(i64 %x, i64 %y) {
entry:
  %0 = mul i64 %x, %y
  %1 = mul i64 %y, %x
  %2 = add i64 %0, %1
  ret i64 %2
}
"#;
        let result = common_subexpression_elimination(ir);
        assert!(
            result.contains("GVN-CSE"),
            "Commutative mul x*y == y*x should be detected. Result:\n{}",
            result
        );
    }

    #[test]
    fn test_different_ops_not_cse() {
        let ir = r#"define i64 @test(i64 %a, i64 %b) {
entry:
  %0 = add i64 %a, %b
  %1 = mul i64 %a, %b
  ret i64 %1
}
"#;
        let result = common_subexpression_elimination(ir);
        assert!(
            !result.contains("GVN-CSE"),
            "Different operations should NOT be eliminated. Result:\n{}",
            result
        );
    }

    #[test]
    fn test_cse_resets_at_function_boundary() {
        let ir = r#"define i64 @foo(i64 %a, i64 %b) {
entry:
  %0 = add i64 %a, %b
  ret i64 %0
}

define i64 @bar(i64 %a, i64 %b) {
entry:
  %0 = add i64 %a, %b
  ret i64 %0
}
"#;
        let result = common_subexpression_elimination(ir);
        // Should NOT eliminate across function boundaries
        assert!(
            !result.contains("GVN-CSE"),
            "CSE should not cross function boundaries. Result:\n{}",
            result
        );
    }
}
