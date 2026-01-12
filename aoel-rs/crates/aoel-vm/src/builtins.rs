//! Built-in functions

use aoel_ir::Value;
use crate::error::{RuntimeError, RuntimeResult};

/// Execute a built-in function
pub fn call_builtin(name: &str, args: Vec<Value>) -> RuntimeResult<Value> {
    match name.to_uppercase().as_str() {
        // String functions
        "LEN" => builtin_len(args),
        "UPPER" => builtin_upper(args),
        "LOWER" => builtin_lower(args),
        "TRIM" => builtin_trim(args),
        "CONTAINS" => builtin_contains(args),
        "STARTS_WITH" => builtin_starts_with(args),
        "ENDS_WITH" => builtin_ends_with(args),
        "CONCAT" => builtin_concat(args),

        // Math functions
        "ABS" => builtin_abs(args),
        "MIN" => builtin_min(args),
        "MAX" => builtin_max(args),
        "SUM" => builtin_sum(args),
        "AVG" => builtin_avg(args),
        "COUNT" => builtin_count(args),

        // Type conversion
        "TO_STRING" => builtin_to_string(args),
        "TO_INT" => builtin_to_int(args),
        "TO_FLOAT" => builtin_to_float(args),

        // Array functions
        "FIRST" => builtin_first(args),
        "LAST" => builtin_last(args),
        "REVERSE" => builtin_reverse(args),
        "FLATTEN" => builtin_flatten(args),

        // Logical functions
        "IN" => builtin_in(args),
        "MATCH" => builtin_match(args),
        "XOR" => builtin_xor(args),
        "IMPLIES" => builtin_implies(args),

        _ => Err(RuntimeError::UnknownBuiltin(name.to_string())),
    }
}

// ==================== String Functions ====================

fn builtin_len(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 1, "LEN")?;
    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::Array(a) => Ok(Value::Int(a.len() as i64)),
        Value::Map(m) => Ok(Value::Int(m.len() as i64)),
        Value::Bytes(b) => Ok(Value::Int(b.len() as i64)),
        _ => Ok(Value::Int(0)),
    }
}

fn builtin_upper(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 1, "UPPER")?;
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        _ => Ok(args[0].clone()),
    }
}

fn builtin_lower(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 1, "LOWER")?;
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        _ => Ok(args[0].clone()),
    }
}

fn builtin_trim(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 1, "TRIM")?;
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim().to_string())),
        _ => Ok(args[0].clone()),
    }
}

fn builtin_contains(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 2, "CONTAINS")?;
    match (&args[0], &args[1]) {
        (Value::String(haystack), Value::String(needle)) => {
            Ok(Value::Bool(haystack.contains(needle.as_str())))
        }
        (Value::Array(arr), value) => Ok(Value::Bool(arr.contains(value))),
        _ => Ok(Value::Bool(false)),
    }
}

fn builtin_starts_with(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 2, "STARTS_WITH")?;
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(prefix)) => {
            Ok(Value::Bool(s.starts_with(prefix.as_str())))
        }
        _ => Ok(Value::Bool(false)),
    }
}

fn builtin_ends_with(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 2, "ENDS_WITH")?;
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(suffix)) => {
            Ok(Value::Bool(s.ends_with(suffix.as_str())))
        }
        _ => Ok(Value::Bool(false)),
    }
}

fn builtin_concat(args: Vec<Value>) -> RuntimeResult<Value> {
    let mut result = String::new();
    for arg in args {
        match arg {
            Value::String(s) => result.push_str(&s),
            other => result.push_str(&format!("{}", other)),
        }
    }
    Ok(Value::String(result))
}

// ==================== Math Functions ====================

fn builtin_abs(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 1, "ABS")?;
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Ok(args[0].clone()),
    }
}

fn builtin_min(args: Vec<Value>) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Ok(Value::Void);
    }

    // Handle array input
    if args.len() == 1 {
        if let Value::Array(arr) = &args[0] {
            return array_min(arr);
        }
    }

    // Handle multiple arguments
    array_min(&args)
}

fn builtin_max(args: Vec<Value>) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Ok(Value::Void);
    }

    // Handle array input
    if args.len() == 1 {
        if let Value::Array(arr) = &args[0] {
            return array_max(arr);
        }
    }

    // Handle multiple arguments
    array_max(&args)
}

fn builtin_sum(args: Vec<Value>) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Ok(Value::Int(0));
    }

    // Handle array input
    if args.len() == 1 {
        if let Value::Array(arr) = &args[0] {
            return array_sum(arr);
        }
    }

    array_sum(&args)
}

fn builtin_avg(args: Vec<Value>) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Ok(Value::Float(0.0));
    }

    // Handle array input
    let values = if args.len() == 1 {
        if let Value::Array(arr) = &args[0] {
            arr.clone()
        } else {
            args
        }
    } else {
        args
    };

    if values.is_empty() {
        return Ok(Value::Float(0.0));
    }

    let sum = array_sum(&values)?;
    let count = values.len() as f64;

    match sum {
        Value::Int(n) => Ok(Value::Float(n as f64 / count)),
        Value::Float(f) => Ok(Value::Float(f / count)),
        _ => Ok(Value::Float(0.0)),
    }
}

fn builtin_count(args: Vec<Value>) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Ok(Value::Int(0));
    }

    match &args[0] {
        Value::Array(arr) => Ok(Value::Int(arr.len() as i64)),
        Value::Map(m) => Ok(Value::Int(m.len() as i64)),
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        _ => Ok(Value::Int(1)),
    }
}

// ==================== Type Conversion ====================

fn builtin_to_string(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 1, "TO_STRING")?;
    Ok(Value::String(format!("{}", args[0])))
}

fn builtin_to_int(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 1, "TO_INT")?;
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::String(s) => Ok(Value::Int(s.parse().unwrap_or(0))),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        _ => Ok(Value::Int(0)),
    }
}

fn builtin_to_float(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 1, "TO_FLOAT")?;
    match &args[0] {
        Value::Int(n) => Ok(Value::Float(*n as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::String(s) => Ok(Value::Float(s.parse().unwrap_or(0.0))),
        Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        _ => Ok(Value::Float(0.0)),
    }
}

// ==================== Array Functions ====================

fn builtin_first(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 1, "FIRST")?;
    match &args[0] {
        Value::Array(arr) => Ok(arr.first().cloned().unwrap_or(Value::Void)),
        _ => Ok(Value::Void),
    }
}

fn builtin_last(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 1, "LAST")?;
    match &args[0] {
        Value::Array(arr) => Ok(arr.last().cloned().unwrap_or(Value::Void)),
        _ => Ok(Value::Void),
    }
}

fn builtin_reverse(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 1, "REVERSE")?;
    match &args[0] {
        Value::Array(arr) => {
            let mut reversed = arr.clone();
            reversed.reverse();
            Ok(Value::Array(reversed))
        }
        Value::String(s) => Ok(Value::String(s.chars().rev().collect())),
        _ => Ok(args[0].clone()),
    }
}

fn builtin_flatten(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 1, "FLATTEN")?;
    match &args[0] {
        Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                match item {
                    Value::Array(inner) => result.extend(inner.clone()),
                    other => result.push(other.clone()),
                }
            }
            Ok(Value::Array(result))
        }
        _ => Ok(args[0].clone()),
    }
}

// ==================== Logical Functions ====================

fn builtin_in(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 2, "IN")?;
    let needle = &args[0];
    let haystack = &args[1];

    match haystack {
        Value::Array(arr) => Ok(Value::Bool(arr.contains(needle))),
        Value::String(s) => {
            if let Value::String(n) = needle {
                Ok(Value::Bool(s.contains(n.as_str())))
            } else {
                Ok(Value::Bool(false))
            }
        }
        Value::Map(m) => {
            if let Value::String(key) = needle {
                Ok(Value::Bool(m.contains_key(key)))
            } else {
                Ok(Value::Bool(false))
            }
        }
        _ => Ok(Value::Bool(false)),
    }
}

fn builtin_match(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 2, "MATCH")?;
    // Simple pattern matching (not regex for now)
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(pattern)) => {
            // Simple glob-like matching: * matches anything
            if pattern == "*" {
                return Ok(Value::Bool(true));
            }
            if pattern.starts_with('*') && pattern.ends_with('*') {
                let inner = &pattern[1..pattern.len() - 1];
                return Ok(Value::Bool(s.contains(inner)));
            }
            if pattern.starts_with('*') {
                let suffix = &pattern[1..];
                return Ok(Value::Bool(s.ends_with(suffix)));
            }
            if pattern.ends_with('*') {
                let prefix = &pattern[..pattern.len() - 1];
                return Ok(Value::Bool(s.starts_with(prefix)));
            }
            Ok(Value::Bool(s == pattern))
        }
        _ => Ok(Value::Bool(false)),
    }
}

fn builtin_xor(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 2, "XOR")?;
    let a = args[0].is_truthy();
    let b = args[1].is_truthy();
    Ok(Value::Bool(a ^ b))
}

fn builtin_implies(args: Vec<Value>) -> RuntimeResult<Value> {
    require_args(&args, 2, "IMPLIES")?;
    let a = args[0].is_truthy();
    let b = args[1].is_truthy();
    // a IMPLIES b = NOT a OR b
    Ok(Value::Bool(!a || b))
}

// ==================== Helper Functions ====================

fn require_args(args: &[Value], count: usize, name: &str) -> RuntimeResult<()> {
    if args.len() != count {
        Err(RuntimeError::InvalidArgCount {
            expected: count,
            actual: args.len(),
        })
    } else {
        Ok(())
    }
}

fn array_sum(values: &[Value]) -> RuntimeResult<Value> {
    let mut int_sum: i64 = 0;
    let mut float_sum: f64 = 0.0;
    let mut has_float = false;

    for val in values {
        match val {
            Value::Int(n) => int_sum += n,
            Value::Float(f) => {
                float_sum += f;
                has_float = true;
            }
            _ => {}
        }
    }

    if has_float {
        Ok(Value::Float(int_sum as f64 + float_sum))
    } else {
        Ok(Value::Int(int_sum))
    }
}

fn array_min(values: &[Value]) -> RuntimeResult<Value> {
    let mut min: Option<Value> = None;

    for val in values {
        match val {
            Value::Int(n) => {
                min = Some(match min {
                    Some(Value::Int(m)) if m < *n => Value::Int(m),
                    Some(Value::Float(m)) if m < *n as f64 => Value::Float(m),
                    _ => Value::Int(*n),
                });
            }
            Value::Float(f) => {
                min = Some(match min {
                    Some(Value::Int(m)) if (m as f64) < *f => Value::Int(m),
                    Some(Value::Float(m)) if m < *f => Value::Float(m),
                    _ => Value::Float(*f),
                });
            }
            _ => {}
        }
    }

    Ok(min.unwrap_or(Value::Void))
}

fn array_max(values: &[Value]) -> RuntimeResult<Value> {
    let mut max: Option<Value> = None;

    for val in values {
        match val {
            Value::Int(n) => {
                max = Some(match max {
                    Some(Value::Int(m)) if m > *n => Value::Int(m),
                    Some(Value::Float(m)) if m > *n as f64 => Value::Float(m),
                    _ => Value::Int(*n),
                });
            }
            Value::Float(f) => {
                max = Some(match max {
                    Some(Value::Int(m)) if (m as f64) > *f => Value::Int(m),
                    Some(Value::Float(m)) if m > *f => Value::Float(m),
                    _ => Value::Float(*f),
                });
            }
            _ => {}
        }
    }

    Ok(max.unwrap_or(Value::Void))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len() {
        assert_eq!(
            call_builtin("LEN", vec![Value::String("hello".to_string())]).unwrap(),
            Value::Int(5)
        );
        assert_eq!(
            call_builtin("LEN", vec![Value::Array(vec![Value::Int(1), Value::Int(2)])]).unwrap(),
            Value::Int(2)
        );
    }

    #[test]
    fn test_upper_lower() {
        assert_eq!(
            call_builtin("UPPER", vec![Value::String("hello".to_string())]).unwrap(),
            Value::String("HELLO".to_string())
        );
        assert_eq!(
            call_builtin("LOWER", vec![Value::String("HELLO".to_string())]).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_sum() {
        assert_eq!(
            call_builtin(
                "SUM",
                vec![Value::Array(vec![
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3)
                ])]
            )
            .unwrap(),
            Value::Int(6)
        );
    }

    #[test]
    fn test_contains() {
        assert_eq!(
            call_builtin(
                "CONTAINS",
                vec![
                    Value::String("hello world".to_string()),
                    Value::String("world".to_string())
                ]
            )
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_xor() {
        assert_eq!(
            call_builtin("XOR", vec![Value::Bool(true), Value::Bool(false)]).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            call_builtin("XOR", vec![Value::Bool(true), Value::Bool(true)]).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_implies() {
        // true IMPLIES true = true
        assert_eq!(
            call_builtin("IMPLIES", vec![Value::Bool(true), Value::Bool(true)]).unwrap(),
            Value::Bool(true)
        );
        // true IMPLIES false = false
        assert_eq!(
            call_builtin("IMPLIES", vec![Value::Bool(true), Value::Bool(false)]).unwrap(),
            Value::Bool(false)
        );
        // false IMPLIES anything = true
        assert_eq!(
            call_builtin("IMPLIES", vec![Value::Bool(false), Value::Bool(false)]).unwrap(),
            Value::Bool(true)
        );
    }
}
