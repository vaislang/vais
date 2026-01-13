//! AOEL Type System

use std::collections::HashMap;
use std::fmt;

/// AOEL 타입
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
    /// 세트 타입
    Set(Box<Type>),
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
            Type::Set(t) => write!(f, "#{{{}}}", t),
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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Type Construction Tests ====================

    #[test]
    fn test_primitive_types() {
        assert_eq!(Type::Int, Type::Int);
        assert_eq!(Type::Float, Type::Float);
        assert_eq!(Type::String, Type::String);
        assert_eq!(Type::Bool, Type::Bool);
        assert_eq!(Type::Unit, Type::Unit);
        assert_eq!(Type::Any, Type::Any);
        assert_eq!(Type::Never, Type::Never);
    }

    #[test]
    fn test_array_type() {
        let arr = Type::Array(Box::new(Type::Int));
        assert_eq!(arr, Type::Array(Box::new(Type::Int)));

        // Nested array
        let nested = Type::Array(Box::new(Type::Array(Box::new(Type::String))));
        assert_eq!(nested, Type::Array(Box::new(Type::Array(Box::new(Type::String)))));
    }

    #[test]
    fn test_tuple_type() {
        let tuple = Type::Tuple(vec![Type::Int, Type::String, Type::Bool]);
        assert_eq!(tuple, Type::Tuple(vec![Type::Int, Type::String, Type::Bool]));

        // Empty tuple
        let empty = Type::Tuple(vec![]);
        assert_eq!(empty, Type::Tuple(vec![]));
    }

    #[test]
    fn test_map_type() {
        let map = Type::Map(Box::new(Type::String), Box::new(Type::Int));
        assert_eq!(map, Type::Map(Box::new(Type::String), Box::new(Type::Int)));
    }

    #[test]
    fn test_struct_type() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Type::String);
        fields.insert("age".to_string(), Type::Int);

        let struct_type = Type::Struct(fields.clone());
        assert_eq!(struct_type, Type::Struct(fields));
    }

    #[test]
    fn test_function_type() {
        let func = Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Int));
        assert_eq!(func, Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Int)));

        // No args function
        let no_args = Type::Function(vec![], Box::new(Type::Unit));
        assert_eq!(no_args, Type::Function(vec![], Box::new(Type::Unit)));
    }

    #[test]
    fn test_optional_type() {
        let opt = Type::Optional(Box::new(Type::String));
        assert_eq!(opt, Type::Optional(Box::new(Type::String)));
    }

    #[test]
    fn test_result_type() {
        let res = Type::Result(Box::new(Type::Int));
        assert_eq!(res, Type::Result(Box::new(Type::Int)));
    }

    #[test]
    fn test_type_var() {
        let var0 = Type::Var(0);
        let var1 = Type::Var(1);
        assert_ne!(var0, var1);
        assert_eq!(var0, Type::Var(0));
    }

    // ==================== is_numeric Tests ====================

    #[test]
    fn test_is_numeric_int() {
        assert!(Type::Int.is_numeric());
    }

    #[test]
    fn test_is_numeric_float() {
        assert!(Type::Float.is_numeric());
    }

    #[test]
    fn test_is_numeric_non_numeric() {
        assert!(!Type::String.is_numeric());
        assert!(!Type::Bool.is_numeric());
        assert!(!Type::Unit.is_numeric());
        assert!(!Type::Any.is_numeric());
        assert!(!Type::Never.is_numeric());
        assert!(!Type::Array(Box::new(Type::Int)).is_numeric());
        assert!(!Type::Var(0).is_numeric());
    }

    // ==================== is_comparable Tests ====================

    #[test]
    fn test_is_comparable_primitives() {
        assert!(Type::Int.is_comparable());
        assert!(Type::Float.is_comparable());
        assert!(Type::String.is_comparable());
        assert!(Type::Bool.is_comparable());
    }

    #[test]
    fn test_is_comparable_non_comparable() {
        assert!(!Type::Unit.is_comparable());
        assert!(!Type::Any.is_comparable());
        assert!(!Type::Never.is_comparable());
        assert!(!Type::Array(Box::new(Type::Int)).is_comparable());
        assert!(!Type::Tuple(vec![Type::Int]).is_comparable());
        assert!(!Type::Map(Box::new(Type::String), Box::new(Type::Int)).is_comparable());
        assert!(!Type::Function(vec![], Box::new(Type::Unit)).is_comparable());
    }

    // ==================== element_type Tests ====================

    #[test]
    fn test_element_type_array() {
        let arr = Type::Array(Box::new(Type::Int));
        assert_eq!(arr.element_type(), Some(&Type::Int));

        let nested = Type::Array(Box::new(Type::Array(Box::new(Type::String))));
        assert_eq!(nested.element_type(), Some(&Type::Array(Box::new(Type::String))));
    }

    #[test]
    fn test_element_type_non_array() {
        assert_eq!(Type::Int.element_type(), None);
        assert_eq!(Type::String.element_type(), None);
        assert_eq!(Type::Tuple(vec![Type::Int]).element_type(), None);
        assert_eq!(Type::Map(Box::new(Type::String), Box::new(Type::Int)).element_type(), None);
    }

    // ==================== return_type Tests ====================

    #[test]
    fn test_return_type_function() {
        let func = Type::Function(vec![Type::Int], Box::new(Type::String));
        assert_eq!(func.return_type(), Some(&Type::String));

        let no_args = Type::Function(vec![], Box::new(Type::Unit));
        assert_eq!(no_args.return_type(), Some(&Type::Unit));
    }

    #[test]
    fn test_return_type_non_function() {
        assert_eq!(Type::Int.return_type(), None);
        assert_eq!(Type::String.return_type(), None);
        assert_eq!(Type::Array(Box::new(Type::Int)).return_type(), None);
    }

    // ==================== substitute Tests ====================

    #[test]
    fn test_substitute_type_var() {
        let var = Type::Var(0);
        let result = var.substitute(0, &Type::Int);
        assert_eq!(result, Type::Int);
    }

    #[test]
    fn test_substitute_different_var() {
        let var = Type::Var(0);
        let result = var.substitute(1, &Type::Int);
        assert_eq!(result, Type::Var(0));
    }

    #[test]
    fn test_substitute_primitive() {
        let int_type = Type::Int;
        let result = int_type.substitute(0, &Type::String);
        assert_eq!(result, Type::Int);
    }

    #[test]
    fn test_substitute_in_array() {
        let arr = Type::Array(Box::new(Type::Var(0)));
        let result = arr.substitute(0, &Type::Int);
        assert_eq!(result, Type::Array(Box::new(Type::Int)));
    }

    #[test]
    fn test_substitute_in_tuple() {
        let tuple = Type::Tuple(vec![Type::Var(0), Type::Var(1), Type::Int]);
        let result = tuple.substitute(0, &Type::String);
        assert_eq!(result, Type::Tuple(vec![Type::String, Type::Var(1), Type::Int]));
    }

    #[test]
    fn test_substitute_in_map() {
        let map = Type::Map(Box::new(Type::Var(0)), Box::new(Type::Var(1)));
        let result = map.substitute(0, &Type::String);
        assert_eq!(result, Type::Map(Box::new(Type::String), Box::new(Type::Var(1))));
    }

    #[test]
    fn test_substitute_in_struct() {
        let mut fields = HashMap::new();
        fields.insert("field".to_string(), Type::Var(0));
        let struct_type = Type::Struct(fields);

        let result = struct_type.substitute(0, &Type::Int);
        let mut expected_fields = HashMap::new();
        expected_fields.insert("field".to_string(), Type::Int);
        assert_eq!(result, Type::Struct(expected_fields));
    }

    #[test]
    fn test_substitute_in_function() {
        let func = Type::Function(vec![Type::Var(0), Type::Var(1)], Box::new(Type::Var(0)));
        let result = func.substitute(0, &Type::Int);
        assert_eq!(result, Type::Function(vec![Type::Int, Type::Var(1)], Box::new(Type::Int)));
    }

    #[test]
    fn test_substitute_in_optional() {
        let opt = Type::Optional(Box::new(Type::Var(0)));
        let result = opt.substitute(0, &Type::String);
        assert_eq!(result, Type::Optional(Box::new(Type::String)));
    }

    #[test]
    fn test_substitute_in_result() {
        let res = Type::Result(Box::new(Type::Var(0)));
        let result = res.substitute(0, &Type::Int);
        assert_eq!(result, Type::Result(Box::new(Type::Int)));
    }

    #[test]
    fn test_substitute_nested() {
        // Array<Function<T0, T1>>
        let nested = Type::Array(Box::new(Type::Function(
            vec![Type::Var(0)],
            Box::new(Type::Var(1))
        )));
        let result = nested.substitute(0, &Type::Int);
        assert_eq!(result, Type::Array(Box::new(Type::Function(
            vec![Type::Int],
            Box::new(Type::Var(1))
        ))));
    }

    // ==================== contains_var Tests ====================

    #[test]
    fn test_contains_var_direct() {
        let var = Type::Var(0);
        assert!(var.contains_var(0));
        assert!(!var.contains_var(1));
    }

    #[test]
    fn test_contains_var_primitive() {
        assert!(!Type::Int.contains_var(0));
        assert!(!Type::String.contains_var(0));
    }

    #[test]
    fn test_contains_var_in_array() {
        let arr = Type::Array(Box::new(Type::Var(0)));
        assert!(arr.contains_var(0));
        assert!(!arr.contains_var(1));
    }

    #[test]
    fn test_contains_var_in_tuple() {
        let tuple = Type::Tuple(vec![Type::Int, Type::Var(1), Type::String]);
        assert!(!tuple.contains_var(0));
        assert!(tuple.contains_var(1));
    }

    #[test]
    fn test_contains_var_in_map() {
        let map = Type::Map(Box::new(Type::Var(0)), Box::new(Type::Int));
        assert!(map.contains_var(0));
        assert!(!map.contains_var(1));

        let map2 = Type::Map(Box::new(Type::String), Box::new(Type::Var(1)));
        assert!(map2.contains_var(1));
    }

    #[test]
    fn test_contains_var_in_struct() {
        let mut fields = HashMap::new();
        fields.insert("a".to_string(), Type::Int);
        fields.insert("b".to_string(), Type::Var(0));
        let struct_type = Type::Struct(fields);
        assert!(struct_type.contains_var(0));
        assert!(!struct_type.contains_var(1));
    }

    #[test]
    fn test_contains_var_in_function() {
        let func = Type::Function(vec![Type::Var(0)], Box::new(Type::Int));
        assert!(func.contains_var(0));

        let func2 = Type::Function(vec![Type::Int], Box::new(Type::Var(1)));
        assert!(func2.contains_var(1));
        assert!(!func2.contains_var(0));
    }

    #[test]
    fn test_contains_var_in_optional() {
        let opt = Type::Optional(Box::new(Type::Var(0)));
        assert!(opt.contains_var(0));
        assert!(!opt.contains_var(1));
    }

    #[test]
    fn test_contains_var_in_result() {
        let res = Type::Result(Box::new(Type::Var(0)));
        assert!(res.contains_var(0));
    }

    // ==================== free_vars Tests ====================

    #[test]
    fn test_free_vars_primitive() {
        assert_eq!(Type::Int.free_vars(), vec![]);
        assert_eq!(Type::String.free_vars(), vec![]);
    }

    #[test]
    fn test_free_vars_single_var() {
        assert_eq!(Type::Var(0).free_vars(), vec![0]);
        assert_eq!(Type::Var(5).free_vars(), vec![5]);
    }

    #[test]
    fn test_free_vars_multiple_vars() {
        let func = Type::Function(vec![Type::Var(2), Type::Var(0)], Box::new(Type::Var(1)));
        let vars = func.free_vars();
        assert_eq!(vars, vec![0, 1, 2]); // sorted
    }

    #[test]
    fn test_free_vars_dedup() {
        // Same variable appears multiple times
        let func = Type::Function(vec![Type::Var(0), Type::Var(0)], Box::new(Type::Var(0)));
        let vars = func.free_vars();
        assert_eq!(vars, vec![0]); // deduped
    }

    #[test]
    fn test_free_vars_nested() {
        let nested = Type::Array(Box::new(Type::Tuple(vec![
            Type::Var(0),
            Type::Map(Box::new(Type::Var(1)), Box::new(Type::Var(2)))
        ])));
        let vars = nested.free_vars();
        assert_eq!(vars, vec![0, 1, 2]);
    }

    #[test]
    fn test_free_vars_optional_result() {
        let opt = Type::Optional(Box::new(Type::Var(0)));
        let res = Type::Result(Box::new(Type::Var(1)));
        assert_eq!(opt.free_vars(), vec![0]);
        assert_eq!(res.free_vars(), vec![1]);
    }

    // ==================== Display Tests ====================

    #[test]
    fn test_display_primitives() {
        assert_eq!(format!("{}", Type::Int), "Int");
        assert_eq!(format!("{}", Type::Float), "Float");
        assert_eq!(format!("{}", Type::String), "String");
        assert_eq!(format!("{}", Type::Bool), "Bool");
        assert_eq!(format!("{}", Type::Unit), "()");
        assert_eq!(format!("{}", Type::Any), "Any");
        assert_eq!(format!("{}", Type::Never), "Never");
    }

    #[test]
    fn test_display_array() {
        assert_eq!(format!("{}", Type::Array(Box::new(Type::Int))), "[Int]");
        assert_eq!(
            format!("{}", Type::Array(Box::new(Type::Array(Box::new(Type::String))))),
            "[[String]]"
        );
    }

    #[test]
    fn test_display_tuple() {
        assert_eq!(format!("{}", Type::Tuple(vec![])), "()");
        assert_eq!(format!("{}", Type::Tuple(vec![Type::Int])), "(Int)");
        assert_eq!(
            format!("{}", Type::Tuple(vec![Type::Int, Type::String])),
            "(Int, String)"
        );
    }

    #[test]
    fn test_display_map() {
        assert_eq!(
            format!("{}", Type::Map(Box::new(Type::String), Box::new(Type::Int))),
            "{String: Int}"
        );
    }

    #[test]
    fn test_display_function() {
        assert_eq!(
            format!("{}", Type::Function(vec![], Box::new(Type::Unit))),
            "() -> ()"
        );
        assert_eq!(
            format!("{}", Type::Function(vec![Type::Int], Box::new(Type::String))),
            "(Int) -> String"
        );
        assert_eq!(
            format!("{}", Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Int))),
            "(Int, Int) -> Int"
        );
    }

    #[test]
    fn test_display_optional() {
        assert_eq!(format!("{}", Type::Optional(Box::new(Type::Int))), "?Int");
    }

    #[test]
    fn test_display_result() {
        assert_eq!(format!("{}", Type::Result(Box::new(Type::String))), "!String");
    }

    #[test]
    fn test_display_var() {
        assert_eq!(format!("{}", Type::Var(0)), "T0");
        assert_eq!(format!("{}", Type::Var(42)), "T42");
    }

    // ==================== Clone and PartialEq Tests ====================

    #[test]
    fn test_type_clone() {
        let func = Type::Function(vec![Type::Var(0)], Box::new(Type::Int));
        let cloned = func.clone();
        assert_eq!(func, cloned);
    }

    #[test]
    fn test_type_inequality() {
        assert_ne!(Type::Int, Type::Float);
        assert_ne!(Type::Array(Box::new(Type::Int)), Type::Array(Box::new(Type::Float)));
        assert_ne!(
            Type::Function(vec![Type::Int], Box::new(Type::Int)),
            Type::Function(vec![Type::Float], Box::new(Type::Int))
        );
    }

    // ==================== Complex Type Tests ====================

    #[test]
    fn test_complex_type_construction() {
        // (String, Int) -> Optional<Array<Result<Bool>>>
        let complex = Type::Function(
            vec![Type::String, Type::Int],
            Box::new(Type::Optional(Box::new(Type::Array(Box::new(Type::Result(Box::new(Type::Bool)))))))
        );

        let display = format!("{}", complex);
        assert_eq!(display, "(String, Int) -> ?[!Bool]");
    }

    #[test]
    fn test_deeply_nested_type() {
        // Array<Map<String, Tuple<Int, Function<Bool, Float>>>>
        let deep = Type::Array(Box::new(Type::Map(
            Box::new(Type::String),
            Box::new(Type::Tuple(vec![
                Type::Int,
                Type::Function(vec![Type::Bool], Box::new(Type::Float))
            ]))
        )));

        let display = format!("{}", deep);
        assert!(display.contains("[{"));
        assert!(display.contains("(Int, (Bool) -> Float)"));
    }

    #[test]
    fn test_substitute_chain() {
        // T0 -> (T1, T2)
        let func = Type::Function(
            vec![Type::Var(0)],
            Box::new(Type::Tuple(vec![Type::Var(1), Type::Var(2)]))
        );

        // Substitute T0 -> Int, T1 -> String, T2 -> Bool
        let result = func
            .substitute(0, &Type::Int)
            .substitute(1, &Type::String)
            .substitute(2, &Type::Bool);

        assert_eq!(result, Type::Function(
            vec![Type::Int],
            Box::new(Type::Tuple(vec![Type::String, Type::Bool]))
        ));
    }
}
