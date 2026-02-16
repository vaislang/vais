//! Strength reduction - replace expensive operations with cheaper ones.

/// Strength reduction - replace expensive operations with cheaper ones
pub(crate) fn strength_reduction(ir: &str) -> String {
    let mut result = String::with_capacity(ir.len());

    for line in ir.lines() {
        let trimmed = line.trim();

        // Replace multiplication by power of 2 with shift
        if let Some(reduced) = try_strength_reduce_mul(trimmed) {
            result.push_str(&reduced);
            result.push('\n');
            continue;
        }

        // Replace division by power of 2 with shift
        if let Some(reduced) = try_strength_reduce_div(trimmed) {
            result.push_str(&reduced);
            result.push('\n');
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Try to reduce multiplication to shift
pub(super) fn try_strength_reduce_mul(line: &str) -> Option<String> {
    if !line.contains(" = mul i64 ") {
        return None;
    }

    let parts: Vec<&str> = line.split(" = mul i64 ").collect();
    if parts.len() != 2 {
        return None;
    }

    let dest = parts[0].trim();
    let operands: Vec<&str> = parts[1].split(',').map(|s| s.trim()).collect();
    if operands.len() != 2 {
        return None;
    }

    // Check if one operand is a power of 2
    let (var, shift) = if let Ok(n) = operands[1].parse::<i64>() {
        if is_power_of_2(n) {
            (operands[0], log2(n))
        } else {
            return None;
        }
    } else if let Ok(n) = operands[0].parse::<i64>() {
        if is_power_of_2(n) {
            (operands[1], log2(n))
        } else {
            return None;
        }
    } else {
        return None;
    };

    Some(format!(
        "  {} = shl i64 {}, {}  ; strength reduced from mul by {}",
        dest,
        var,
        shift,
        1i64 << shift
    ))
}

/// Try to reduce division to shift (only for unsigned or positive known values)
fn try_strength_reduce_div(line: &str) -> Option<String> {
    if !line.contains(" = sdiv i64 ") {
        return None;
    }

    let parts: Vec<&str> = line.split(" = sdiv i64 ").collect();
    if parts.len() != 2 {
        return None;
    }

    let dest = parts[0].trim();
    let operands: Vec<&str> = parts[1].split(',').map(|s| s.trim()).collect();
    if operands.len() != 2 {
        return None;
    }

    // Only reduce if divisor is power of 2
    if let Ok(n) = operands[1].parse::<i64>() {
        if is_power_of_2(n) && n > 0 {
            let shift = log2(n);
            return Some(format!(
                "  {} = ashr i64 {}, {}  ; strength reduced from div by {}",
                dest, operands[0], shift, n
            ));
        }
    }

    None
}

pub(super) fn is_power_of_2(n: i64) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

pub(super) fn log2(n: i64) -> u32 {
    63 - n.leading_zeros()
}
