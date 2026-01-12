//! v6b Type System

use std::collections::HashMap;
use std::fmt;

/// v6b 타입
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// 정수
    Int,
    /// 실수
    Float,
    /// 문자열
    String,
    /// 불리언
    Bool,
    /// Unit/Void
    Unit,
    /// 배열 타입
    Array(Box<Type>),
    /// 튜플 타입
    Tuple(Vec<Type>),
    /// 맵/딕셔너리 타입
    Map(Box<Type>, Box<Type>),
    /// 구조체 타입
    Struct(HashMap<String, Type>),
    /// 함수 타입
    Function(Vec<Type>, Box<Type>),
    /// Optional 타입
    Optional(Box<Type>),
    /// Result 타입
    Result(Box<Type>),
    /// 타입 변수 (추론용)
    Var(usize),
    /// Any 타입 (동적 타입)
    Any,
    /// Never 타입 (에러/발산)
    Never,
}

impl Type {
    /// 정수 타입인지 확인
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }

    /// 비교 가능한지 확인
    pub fn is_comparable(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::String | Type::Bool)
    }

    /// 배열 요소 타입 반환
    pub fn element_type(&self) -> Option<&Type> {
        match self {
            Type::Array(t) => Some(t),
            _ => None,
        }
    }

    /// 함수 반환 타입
    pub fn return_type(&self) -> Option<&Type> {
        match self {
            Type::Function(_, ret) => Some(ret),
            _ => None,
        }
    }

    /// 타입 변수 치환
    pub fn substitute(&self, var: usize, replacement: &Type) -> Type {
        match self {
            Type::Var(v) if *v == var => replacement.clone(),
            Type::Array(t) => Type::Array(Box::new(t.substitute(var, replacement))),
            Type::Tuple(ts) => Type::Tuple(
                ts.iter()
                    .map(|t| t.substitute(var, replacement))
                    .collect(),
            ),
            Type::Map(k, v) => Type::Map(
                Box::new(k.substitute(var, replacement)),
                Box::new(v.substitute(var, replacement)),
            ),
            Type::Struct(fields) => Type::Struct(
                fields
                    .iter()
                    .map(|(k, v)| (k.clone(), v.substitute(var, replacement)))
                    .collect(),
            ),
            Type::Function(params, ret) => Type::Function(
                params
                    .iter()
                    .map(|t| t.substitute(var, replacement))
                    .collect(),
                Box::new(ret.substitute(var, replacement)),
            ),
            Type::Optional(t) => Type::Optional(Box::new(t.substitute(var, replacement))),
            Type::Result(t) => Type::Result(Box::new(t.substitute(var, replacement))),
            _ => self.clone(),
        }
    }

    /// 타입 변수 포함 여부
    pub fn contains_var(&self, var: usize) -> bool {
        match self {
            Type::Var(v) => *v == var,
            Type::Array(t) => t.contains_var(var),
            Type::Tuple(ts) => ts.iter().any(|t| t.contains_var(var)),
            Type::Map(k, v) => k.contains_var(var) || v.contains_var(var),
            Type::Struct(fields) => fields.values().any(|t| t.contains_var(var)),
            Type::Function(params, ret) => {
                params.iter().any(|t| t.contains_var(var)) || ret.contains_var(var)
            }
            Type::Optional(t) | Type::Result(t) => t.contains_var(var),
            _ => false,
        }
    }

    /// 자유 타입 변수 수집
    pub fn free_vars(&self) -> Vec<usize> {
        let mut vars = Vec::new();
        self.collect_vars(&mut vars);
        vars.sort();
        vars.dedup();
        vars
    }

    fn collect_vars(&self, vars: &mut Vec<usize>) {
        match self {
            Type::Var(v) => vars.push(*v),
            Type::Array(t) => t.collect_vars(vars),
            Type::Tuple(ts) => {
                for t in ts {
                    t.collect_vars(vars);
                }
            }
            Type::Map(k, v) => {
                k.collect_vars(vars);
                v.collect_vars(vars);
            }
            Type::Struct(fields) => {
                for t in fields.values() {
                    t.collect_vars(vars);
                }
            }
            Type::Function(params, ret) => {
                for t in params {
                    t.collect_vars(vars);
                }
                ret.collect_vars(vars);
            }
            Type::Optional(t) | Type::Result(t) => t.collect_vars(vars),
            _ => {}
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::String => write!(f, "String"),
            Type::Bool => write!(f, "Bool"),
            Type::Unit => write!(f, "()"),
            Type::Array(t) => write!(f, "[{}]", t),
            Type::Tuple(ts) => {
                write!(f, "(")?;
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            Type::Map(k, v) => write!(f, "{{{}: {}}}", k, v),
            Type::Struct(fields) => {
                write!(f, "{{ ")?;
                for (i, (name, ty)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, ty)?;
                }
                write!(f, " }}")
            }
            Type::Function(params, ret) => {
                write!(f, "(")?;
                for (i, t) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Optional(t) => write!(f, "?{}", t),
            Type::Result(t) => write!(f, "!{}", t),
            Type::Var(v) => write!(f, "T{}", v),
            Type::Any => write!(f, "Any"),
            Type::Never => write!(f, "Never"),
        }
    }
}
