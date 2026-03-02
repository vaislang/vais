//! Constant folding - evaluate constant expressions at compile time
//!
//! Supports arithmetic (add/sub/mul/sdiv/srem), bitwise (and/or/xor),
//! shift (shl/ashr/lshr), and identity/absorbing element simplification.

/// Constant folding - evaluate constant expressions at compile time
pub(crate) fn constant_folding(ir: &str) -> String {
    let mut result = String::with_capacity(ir.len());

    for line in ir.lines() {
        let trimmed = line.trim();

        // Try full constant folding (both operands are constants)
        if let Some(folded) = try_fold_any_binop(trimmed) {
            result.push_str(&folded);
            result.push('\n');
            continue;
        }

        // Try identity/absorbing simplification (one operand is constant)
        if let Some(simplified) = try_simplify_identity(trimmed) {
            result.push_str(&simplified);
            result.push('\n');
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Try to fold any binary operation with constant operands
fn try_fold_any_binop(line: &str) -> Option<String> {
    // Arithmetic operations
    if let Some(folded) = try_fold_binary_op(line, "add", |a, b| a.wrapping_add(b)) {
        return Some(folded);
    }
    if let Some(folded) = try_fold_binary_op(line, "sub", |a, b| a.wrapping_sub(b)) {
        return Some(folded);
    }
    if let Some(folded) = try_fold_binary_op(line, "mul", |a, b| a.wrapping_mul(b)) {
        return Some(folded);
    }
    if let Some(folded) =
        try_fold_binary_op(line, "sdiv", |a, b| if b != 0 { a / b } else { 0 })
    {
        return Some(folded);
    }
    if let Some(folded) =
        try_fold_binary_op(line, "srem", |a, b| if b != 0 { a % b } else { 0 })
    {
        return Some(folded);
    }

    // Bitwise operations
    if let Some(folded) = try_fold_binary_op(line, "and", |a, b| a & b) {
        return Some(folded);
    }
    if let Some(folded) = try_fold_binary_op(line, "or", |a, b| a | b) {
        return Some(folded);
    }
    if let Some(folded) = try_fold_binary_op(line, "xor", |a, b| a ^ b) {
        return Some(folded);
    }

    // Shift operations (with shift amount bounds check)
    if let Some(folded) = try_fold_binary_op(line, "shl", |a, b| {
        if (0..64).contains(&b) {
            a.wrapping_shl(b as u32)
        } else {
            0
        }
    }) {
        return Some(folded);
    }
    if let Some(folded) = try_fold_binary_op(line, "ashr", |a, b| {
        if (0..64).contains(&b) {
            a.wrapping_shr(b as u32)
        } else if a < 0 {
            -1
        } else {
            0
        }
    }) {
        return Some(folded);
    }
    if let Some(folded) = try_fold_binary_op(line, "lshr", |a, b| {
        if (0..64).contains(&b) {
            ((a as u64).wrapping_shr(b as u32)) as i64
        } else {
            0
        }
    }) {
        return Some(folded);
    }

    None
}

/// Try to simplify identity and absorbing element patterns:
/// - add X, 0 => X
/// - mul X, 1 => X
/// - mul X, 0 => 0
/// - and X, 0 => 0
/// - and X, -1 => X
/// - or X, 0 => X
/// - shl X, 0 => X
/// - sub X, 0 => X
fn try_simplify_identity(line: &str) -> Option<String> {
    let ops_and_identities: &[(&str, &[(&str, IdentityAction)])] = &[
        (
            "add",
            &[
                ("0", IdentityAction::Other),
            ],
        ),
        (
            "sub",
            &[
                ("0", IdentityAction::First), // X - 0 => X (only right operand)
            ],
        ),
        (
            "mul",
            &[
                ("1", IdentityAction::Other),
                ("0", IdentityAction::Zero),
            ],
        ),
        (
            "and",
            &[
                ("0", IdentityAction::Zero),
                ("-1", IdentityAction::Other),
            ],
        ),
        (
            "or",
            &[
                ("0", IdentityAction::Other),
            ],
        ),
        (
            "xor",
            &[
                ("0", IdentityAction::Other),
            ],
        ),
        (
            "shl",
            &[
                ("0", IdentityAction::First),
            ],
        ),
        (
            "ashr",
            &[
                ("0", IdentityAction::First),
            ],
        ),
        (
            "lshr",
            &[
                ("0", IdentityAction::First),
            ],
        ),
    ];

    for (op, rules) in ops_and_identities {
        let pattern = format!(" = {} i64 ", op);
        if !line.contains(&pattern) {
            continue;
        }

        let parts: Vec<&str> = line.split(&pattern).collect();
        if parts.len() != 2 {
            continue;
        }

        let dest = parts[0].trim();
        let operands: Vec<&str> = parts[1].split(',').map(|s| s.trim()).collect();
        if operands.len() != 2 {
            continue;
        }

        for (identity_val, action) in *rules {
            match action {
                IdentityAction::Other => {
                    // Commutative: if either operand is the identity, return the other
                    if operands[0] == *identity_val {
                        return Some(format!(
                            "  {} = add i64 0, {}  ; identity: {} {} {}",
                            dest, operands[1], operands[0], op, operands[1]
                        ));
                    }
                    if operands[1] == *identity_val {
                        return Some(format!(
                            "  {} = add i64 0, {}  ; identity: {} {} {}",
                            dest, operands[0], operands[0], op, operands[1]
                        ));
                    }
                }
                IdentityAction::First => {
                    // Non-commutative: only right operand can be identity
                    if operands[1] == *identity_val {
                        return Some(format!(
                            "  {} = add i64 0, {}  ; identity: {} {} {}",
                            dest, operands[0], operands[0], op, operands[1]
                        ));
                    }
                }
                IdentityAction::Zero => {
                    if operands[0] == *identity_val || operands[1] == *identity_val {
                        return Some(format!(
                            "  {} = add i64 0, 0  ; absorbing: {} {} {}",
                            dest, operands[0], op, operands[1]
                        ));
                    }
                }
            }
        }
    }

    None
}

#[derive(Clone, Copy)]
enum IdentityAction {
    Other,  // Commutative identity: return the non-identity operand
    First,  // Non-commutative: only when right operand is the value
    Zero,   // Absorbing element: result is 0
}

/// Try to fold a binary operation with constant operands
fn try_fold_binary_op<F>(line: &str, op: &str, f: F) -> Option<String>
where
    F: Fn(i64, i64) -> i64,
{
    // Pattern: %N = add i64 X, Y
    let pattern = format!(" = {} i64 ", op);
    if !line.contains(&pattern) {
        return None;
    }

    let parts: Vec<&str> = line.split(&pattern).collect();
    if parts.len() != 2 {
        return None;
    }

    let dest = parts[0].trim();
    let operands: Vec<&str> = parts[1].split(',').map(|s| s.trim()).collect();
    if operands.len() != 2 {
        return None;
    }

    // Try to parse both operands as constants
    let a = operands[0].parse::<i64>().ok()?;
    let b = operands[1].parse::<i64>().ok()?;

    let result = f(a, b);
    Some(format!(
        "  {} = add i64 0, {}  ; folded from {} {} {}",
        dest, result, a, op, b
    ))
}
