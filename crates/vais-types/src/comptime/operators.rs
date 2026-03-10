//! Binary and unary operator evaluation for compile-time expressions

use super::*;

impl ComptimeEvaluator {
    pub(crate) fn eval_binary_op(
        &self,
        op: BinOp,
        left: ComptimeValue,
        right: ComptimeValue,
        span: Span,
    ) -> TypeResult<ComptimeValue> {
        match (op, &left, &right) {
            // String concatenation
            (BinOp::Add, ComptimeValue::String(l), ComptimeValue::String(r)) => {
                Ok(ComptimeValue::String(format!("{}{}", l, r)))
            }

            // Integer arithmetic
            (BinOp::Add, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Int(l.checked_add(*r).ok_or_else(|| {
                    TypeError::Mismatch {
                        expected: "no overflow".to_string(),
                        found: "integer overflow".to_string(),
                        span: Some(span),
                    }
                })?))
            }
            (BinOp::Sub, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Int(l.checked_sub(*r).ok_or_else(|| {
                    TypeError::Mismatch {
                        expected: "no overflow".to_string(),
                        found: "integer overflow".to_string(),
                        span: Some(span),
                    }
                })?))
            }
            (BinOp::Mul, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Int(l.checked_mul(*r).ok_or_else(|| {
                    TypeError::Mismatch {
                        expected: "no overflow".to_string(),
                        found: "integer overflow".to_string(),
                        span: Some(span),
                    }
                })?))
            }
            (BinOp::Div, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                if *r == 0 {
                    return Err(TypeError::Mismatch {
                        expected: "non-zero divisor".to_string(),
                        found: "division by zero".to_string(),
                        span: Some(span),
                    });
                }
                Ok(ComptimeValue::Int(l / r))
            }
            (BinOp::Mod, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                if *r == 0 {
                    return Err(TypeError::Mismatch {
                        expected: "non-zero divisor".to_string(),
                        found: "modulo by zero".to_string(),
                        span: Some(span),
                    });
                }
                Ok(ComptimeValue::Int(l % r))
            }

            // Float arithmetic
            (BinOp::Add, ComptimeValue::Float(l), ComptimeValue::Float(r)) => {
                Ok(ComptimeValue::Float(l + r))
            }
            (BinOp::Sub, ComptimeValue::Float(l), ComptimeValue::Float(r)) => {
                Ok(ComptimeValue::Float(l - r))
            }
            (BinOp::Mul, ComptimeValue::Float(l), ComptimeValue::Float(r)) => {
                Ok(ComptimeValue::Float(l * r))
            }
            (BinOp::Div, ComptimeValue::Float(l), ComptimeValue::Float(r)) => {
                Ok(ComptimeValue::Float(l / r))
            }

            // Comparison operators (integers)
            (BinOp::Lt, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Bool(l < r))
            }
            (BinOp::Lte, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Bool(l <= r))
            }
            (BinOp::Gt, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Bool(l > r))
            }
            (BinOp::Gte, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Bool(l >= r))
            }
            (BinOp::Eq, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Bool(l == r))
            }
            (BinOp::Neq, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Bool(l != r))
            }

            // String comparison
            (BinOp::Eq, ComptimeValue::String(l), ComptimeValue::String(r)) => {
                Ok(ComptimeValue::Bool(l == r))
            }
            (BinOp::Neq, ComptimeValue::String(l), ComptimeValue::String(r)) => {
                Ok(ComptimeValue::Bool(l != r))
            }

            // Logical operators
            (BinOp::And, ComptimeValue::Bool(l), ComptimeValue::Bool(r)) => {
                Ok(ComptimeValue::Bool(*l && *r))
            }
            (BinOp::Or, ComptimeValue::Bool(l), ComptimeValue::Bool(r)) => {
                Ok(ComptimeValue::Bool(*l || *r))
            }

            // Bitwise operators
            (BinOp::BitAnd, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Int(l & r))
            }
            (BinOp::BitOr, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Int(l | r))
            }
            (BinOp::BitXor, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Int(l ^ r))
            }
            (BinOp::Shl, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Int(l << r))
            }
            (BinOp::Shr, ComptimeValue::Int(l), ComptimeValue::Int(r)) => {
                Ok(ComptimeValue::Int(l >> r))
            }

            _ => Err(TypeError::Mismatch {
                expected: format!("compatible operands for {:?}", op),
                found: format!("{:?} and {:?}", left, right),
                span: Some(span),
            }),
        }
    }

    pub(crate) fn eval_unary_op(
        &self,
        op: UnaryOp,
        val: ComptimeValue,
        span: Span,
    ) -> TypeResult<ComptimeValue> {
        match (op, &val) {
            (UnaryOp::Neg, ComptimeValue::Int(n)) => Ok(ComptimeValue::Int(-n)),
            (UnaryOp::Neg, ComptimeValue::Float(f)) => Ok(ComptimeValue::Float(-f)),
            (UnaryOp::Not, ComptimeValue::Bool(b)) => Ok(ComptimeValue::Bool(!b)),
            (UnaryOp::BitNot, ComptimeValue::Int(n)) => Ok(ComptimeValue::Int(!n)),
            _ => Err(TypeError::Mismatch {
                expected: format!("compatible operand for {:?}", op),
                found: format!("{:?}", val),
                span: Some(span),
            }),
        }
    }
}
