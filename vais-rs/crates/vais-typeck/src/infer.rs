//! Vais Type Inference
//!
//! Hindley-Milner 스타일 타입 추론

use std::collections::HashMap;

use vais_lexer::Span;

use crate::error::{TypeError, TypeResult};
use crate::types::Type;

/// 타입 환경
#[derive(Debug, Clone)]
pub struct TypeEnv {
    /// 변수 -> 타입 매핑
    vars: HashMap<String, Type>,
    /// 함수 -> 타입 매핑
    functions: HashMap<String, Type>,
    /// 현재 함수 (재귀 호출용)
    pub current_function: Option<(String, Type)>,
    /// 다음 타입 변수 ID
    next_var: usize,
    /// 타입 변수 치환 (Union-Find)
    substitution: HashMap<usize, Type>,
    /// 타입 파라미터 -> 타입 변수 매핑 (제네릭용)
    type_params: HashMap<String, Type>,
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut env = Self {
            vars: HashMap::new(),
            functions: HashMap::new(),
            current_function: None,
            next_var: 0,
            substitution: HashMap::new(),
            type_params: HashMap::new(),
        };
        env.register_builtins();
        env
    }

    /// 타입 파라미터 바인딩 (제네릭 함수용)
    pub fn bind_type_param(&mut self, name: String) -> Type {
        let var = self.fresh_var();
        self.type_params.insert(name, var.clone());
        var
    }

    /// 타입 파라미터 조회
    pub fn lookup_type_param(&self, name: &str) -> Option<Type> {
        self.type_params.get(name).cloned()
    }

    /// 타입 파라미터 스코프 클리어
    pub fn clear_type_params(&mut self) {
        self.type_params.clear();
    }

    /// 빌트인 함수 등록
    fn register_builtins(&mut self) {
        // Math functions (Float -> Float) - 숫자 타입은 자동 변환됨
        let float_to_float = Type::Function(vec![Type::Float], Box::new(Type::Float));
        self.functions.insert("sqrt".to_string(), float_to_float.clone());
        self.functions.insert("sin".to_string(), float_to_float.clone());
        self.functions.insert("cos".to_string(), float_to_float.clone());
        self.functions.insert("tan".to_string(), float_to_float.clone());
        self.functions.insert("log".to_string(), float_to_float.clone());
        self.functions.insert("log10".to_string(), float_to_float.clone());
        self.functions.insert("exp".to_string(), float_to_float.clone());
        self.functions.insert("asin".to_string(), float_to_float.clone());
        self.functions.insert("acos".to_string(), float_to_float.clone());
        self.functions.insert("atan".to_string(), float_to_float.clone());

        // Math functions (Float -> Float, but conceptually rounding)
        self.functions.insert("floor".to_string(), float_to_float.clone());
        self.functions.insert("ceil".to_string(), float_to_float.clone());
        self.functions.insert("round".to_string(), float_to_float.clone());
        self.functions.insert("abs".to_string(), float_to_float.clone());

        // Math functions (Float, Float -> Float)
        let float2_to_float = Type::Function(vec![Type::Float, Type::Float], Box::new(Type::Float));
        self.functions.insert("pow".to_string(), float2_to_float.clone());
        self.functions.insert("atan2".to_string(), float2_to_float.clone());
        self.functions.insert("min".to_string(), float2_to_float.clone());
        self.functions.insert("max".to_string(), float2_to_float.clone());

        // Collection functions
        self.functions.insert("len".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Int)));
        self.functions.insert("first".to_string(), Type::Function(vec![Type::Array(Box::new(Type::Any))], Box::new(Type::Any)));
        self.functions.insert("last".to_string(), Type::Function(vec![Type::Array(Box::new(Type::Any))], Box::new(Type::Any)));
        self.functions.insert("reverse".to_string(), Type::Function(vec![Type::Array(Box::new(Type::Any))], Box::new(Type::Array(Box::new(Type::Any)))));

        // String functions
        self.functions.insert("upper".to_string(), Type::Function(vec![Type::String], Box::new(Type::String)));
        self.functions.insert("lower".to_string(), Type::Function(vec![Type::String], Box::new(Type::String)));
        self.functions.insert("trim".to_string(), Type::Function(vec![Type::String], Box::new(Type::String)));
        self.functions.insert("split".to_string(), Type::Function(vec![Type::String, Type::String], Box::new(Type::Array(Box::new(Type::String)))));
        self.functions.insert("join".to_string(), Type::Function(vec![Type::Array(Box::new(Type::String)), Type::String], Box::new(Type::String)));
        self.functions.insert("replace".to_string(), Type::Function(vec![Type::String, Type::String, Type::String], Box::new(Type::String)));
        self.functions.insert("substr".to_string(), Type::Function(vec![Type::String, Type::Int, Type::Int], Box::new(Type::String)));
        self.functions.insert("contains".to_string(), Type::Function(vec![Type::String, Type::String], Box::new(Type::Bool)));
        self.functions.insert("starts_with".to_string(), Type::Function(vec![Type::String, Type::String], Box::new(Type::Bool)));
        self.functions.insert("ends_with".to_string(), Type::Function(vec![Type::String, Type::String], Box::new(Type::Bool)));

        // Type conversion
        self.functions.insert("int".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Int)));
        self.functions.insert("float".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Float)));
        self.functions.insert("str".to_string(), Type::Function(vec![Type::Any], Box::new(Type::String)));
        self.functions.insert("bool".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Bool)));

        // I/O functions
        self.functions.insert("print".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Unit)));
        self.functions.insert("println".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Unit)));

        // Range
        self.functions.insert("range".to_string(), Type::Function(vec![Type::Int], Box::new(Type::Array(Box::new(Type::Int)))));

        // Extended array functions
        self.functions.insert("push".to_string(), Type::Function(vec![Type::Array(Box::new(Type::Any)), Type::Any], Box::new(Type::Array(Box::new(Type::Any)))));
        self.functions.insert("pop".to_string(), Type::Function(vec![Type::Array(Box::new(Type::Any))], Box::new(Type::Array(Box::new(Type::Any)))));
        self.functions.insert("take".to_string(), Type::Function(vec![Type::Array(Box::new(Type::Any)), Type::Int], Box::new(Type::Array(Box::new(Type::Any)))));
        self.functions.insert("drop".to_string(), Type::Function(vec![Type::Array(Box::new(Type::Any)), Type::Int], Box::new(Type::Array(Box::new(Type::Any)))));
        self.functions.insert("zip".to_string(), Type::Function(vec![Type::Array(Box::new(Type::Any)), Type::Array(Box::new(Type::Any))], Box::new(Type::Array(Box::new(Type::Any)))));
        self.functions.insert("flatten".to_string(), Type::Function(vec![Type::Array(Box::new(Type::Any))], Box::new(Type::Array(Box::new(Type::Any)))));
        self.functions.insert("sort".to_string(), Type::Function(vec![Type::Array(Box::new(Type::Any))], Box::new(Type::Array(Box::new(Type::Any)))));
        self.functions.insert("unique".to_string(), Type::Function(vec![Type::Array(Box::new(Type::Any))], Box::new(Type::Array(Box::new(Type::Any)))));
        self.functions.insert("index_of".to_string(), Type::Function(vec![Type::Array(Box::new(Type::Any)), Type::Any], Box::new(Type::Int)));
        self.functions.insert("concat".to_string(), Type::Function(vec![Type::Array(Box::new(Type::Any)), Type::Array(Box::new(Type::Any))], Box::new(Type::Array(Box::new(Type::Any)))));

        // Extended math functions
        self.functions.insert("log2".to_string(), float_to_float.clone());
        self.functions.insert("clamp".to_string(), Type::Function(vec![Type::Float, Type::Float, Type::Float], Box::new(Type::Float)));

        // Extended string functions
        self.functions.insert("chars".to_string(), Type::Function(vec![Type::String], Box::new(Type::Array(Box::new(Type::String)))));
        self.functions.insert("pad_left".to_string(), Type::Function(vec![Type::String, Type::Int, Type::String], Box::new(Type::String)));
        self.functions.insert("pad_right".to_string(), Type::Function(vec![Type::String, Type::Int, Type::String], Box::new(Type::String)));
        self.functions.insert("repeat".to_string(), Type::Function(vec![Type::String, Type::Int], Box::new(Type::String)));

        // Type checking functions
        self.functions.insert("type".to_string(), Type::Function(vec![Type::Any], Box::new(Type::String)));
        self.functions.insert("is_int".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Bool)));
        self.functions.insert("is_float".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Bool)));
        self.functions.insert("is_string".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Bool)));
        self.functions.insert("is_bool".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Bool)));
        self.functions.insert("is_array".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Bool)));
        self.functions.insert("is_map".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Bool)));
    }

    /// 새 타입 변수 생성
    pub fn fresh_var(&mut self) -> Type {
        let var = self.next_var;
        self.next_var += 1;
        Type::Var(var)
    }

    /// 변수 바인딩
    pub fn bind_var(&mut self, name: String, ty: Type) {
        self.vars.insert(name, ty);
    }

    /// 변수 조회
    pub fn lookup_var(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }

    /// 함수 등록
    pub fn register_function(&mut self, name: String, ty: Type) {
        self.functions.insert(name, ty);
    }

    /// 함수 조회
    pub fn lookup_function(&self, name: &str) -> Option<&Type> {
        self.functions.get(name)
    }

    /// 타입 변수 해결
    pub fn resolve(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(v) => {
                if let Some(resolved) = self.substitution.get(v) {
                    self.resolve(resolved)
                } else {
                    ty.clone()
                }
            }
            Type::Array(t) => Type::Array(Box::new(self.resolve(t))),
            Type::Tuple(ts) => Type::Tuple(ts.iter().map(|t| self.resolve(t)).collect()),
            Type::Map(k, v) => {
                Type::Map(Box::new(self.resolve(k)), Box::new(self.resolve(v)))
            }
            Type::Struct(fields) => {
                Type::Struct(fields.iter().map(|(k, v)| (k.clone(), self.resolve(v))).collect())
            }
            Type::Function(params, ret) => Type::Function(
                params.iter().map(|t| self.resolve(t)).collect(),
                Box::new(self.resolve(ret)),
            ),
            Type::Optional(t) => Type::Optional(Box::new(self.resolve(t))),
            Type::Result(t) => Type::Result(Box::new(self.resolve(t))),
            _ => ty.clone(),
        }
    }

    /// 타입 통일 (Unification)
    pub fn unify(&mut self, t1: &Type, t2: &Type, span: Span) -> TypeResult<()> {
        let t1 = self.resolve(t1);
        let t2 = self.resolve(t2);

        match (&t1, &t2) {
            // 같은 타입
            (a, b) if a == b => Ok(()),

            // Any는 모든 것과 통일
            (Type::Any, _) | (_, Type::Any) => Ok(()),

            // 타입 변수 바인딩
            (Type::Var(v), t) | (t, Type::Var(v)) => {
                // Occurs check
                if t.contains_var(*v) {
                    return Err(TypeError::InfiniteType(format!("T{} = {}", v, t)));
                }
                self.substitution.insert(*v, t.clone());
                Ok(())
            }

            // 배열 통일
            (Type::Array(a), Type::Array(b)) => self.unify(a, b, span),

            // 튜플 통일
            (Type::Tuple(a), Type::Tuple(b)) if a.len() == b.len() => {
                for (x, y) in a.iter().zip(b.iter()) {
                    self.unify(x, y, span)?;
                }
                Ok(())
            }

            // 함수 통일
            (Type::Function(p1, r1), Type::Function(p2, r2)) if p1.len() == p2.len() => {
                for (x, y) in p1.iter().zip(p2.iter()) {
                    self.unify(x, y, span)?;
                }
                self.unify(r1, r2, span)
            }

            // Optional 통일
            (Type::Optional(a), Type::Optional(b)) => self.unify(a, b, span),

            // Result 통일
            (Type::Result(a), Type::Result(b)) => self.unify(a, b, span),

            // Map 통일
            (Type::Map(k1, v1), Type::Map(k2, v2)) => {
                self.unify(k1, k2, span)?;
                self.unify(v1, v2, span)
            }

            // Struct 통일 (필드 이름과 타입이 모두 일치해야 함)
            (Type::Struct(f1), Type::Struct(f2)) if f1.len() == f2.len() => {
                for (name, ty1) in f1.iter() {
                    if let Some(ty2) = f2.iter().find(|(n, _)| *n == name).map(|(_, t)| t) {
                        self.unify(ty1, ty2, span)?;
                    } else {
                        return Err(TypeError::Mismatch {
                            expected: format!("struct with field '{}'", name),
                            found: "struct without that field".to_string(),
                            span,
                        });
                    }
                }
                Ok(())
            }

            // 숫자 타입 변환 (Int -> Float)
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(()),

            // 매칭 실패
            _ => Err(TypeError::Mismatch {
                expected: t1.to_string(),
                found: t2.to_string(),
                span,
            }),
        }
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_span() -> Span {
        Span::new(0, 0)
    }

    // ==================== TypeEnv Creation Tests ====================

    #[test]
    fn test_type_env_new() {
        let env = TypeEnv::new();
        assert!(env.vars.is_empty());
        assert!(!env.functions.is_empty()); // builtins registered
        assert!(env.current_function.is_none());
        assert_eq!(env.next_var, 0);
        assert!(env.substitution.is_empty());
    }

    #[test]
    fn test_type_env_default() {
        let env1 = TypeEnv::new();
        let env2 = TypeEnv::default();
        assert_eq!(env1.vars.len(), env2.vars.len());
        assert_eq!(env1.functions.len(), env2.functions.len());
    }

    // ==================== Builtin Registration Tests ====================

    #[test]
    fn test_builtins_math_functions() {
        let env = TypeEnv::new();

        // Float -> Float
        for name in ["sqrt", "sin", "cos", "tan", "log", "log10", "exp", "asin", "acos", "atan", "floor", "ceil", "round", "abs", "log2"] {
            let ty = env.lookup_function(name);
            assert!(ty.is_some(), "builtin function {} should exist", name);
            match ty.unwrap() {
                Type::Function(params, ret) => {
                    assert_eq!(params.len(), 1);
                    assert_eq!(**ret, Type::Float);
                }
                _ => panic!("{} should be a function type", name),
            }
        }
    }

    #[test]
    fn test_builtins_math_binary_functions() {
        let env = TypeEnv::new();

        // Float, Float -> Float
        for name in ["pow", "atan2", "min", "max"] {
            let ty = env.lookup_function(name);
            assert!(ty.is_some(), "builtin function {} should exist", name);
            match ty.unwrap() {
                Type::Function(params, ret) => {
                    assert_eq!(params.len(), 2);
                    assert_eq!(**ret, Type::Float);
                }
                _ => panic!("{} should be a function type", name),
            }
        }
    }

    #[test]
    fn test_builtins_string_functions() {
        let env = TypeEnv::new();

        // String -> String
        for name in ["upper", "lower", "trim"] {
            let ty = env.lookup_function(name);
            assert!(ty.is_some(), "builtin function {} should exist", name);
            match ty.unwrap() {
                Type::Function(params, ret) => {
                    assert_eq!(params, &vec![Type::String]);
                    assert_eq!(**ret, Type::String);
                }
                _ => panic!("{} should be a function type", name),
            }
        }
    }

    #[test]
    fn test_builtins_collection_functions() {
        let env = TypeEnv::new();

        // len: Any -> Int
        let len_ty = env.lookup_function("len").unwrap();
        match len_ty {
            Type::Function(params, ret) => {
                assert_eq!(params, &vec![Type::Any]);
                assert_eq!(**ret, Type::Int);
            }
            _ => panic!("len should be a function type"),
        }

        // range: Int -> Array<Int>
        let range_ty = env.lookup_function("range").unwrap();
        match range_ty {
            Type::Function(params, ret) => {
                assert_eq!(params, &vec![Type::Int]);
                assert_eq!(**ret, Type::Array(Box::new(Type::Int)));
            }
            _ => panic!("range should be a function type"),
        }
    }

    #[test]
    fn test_builtins_io_functions() {
        let env = TypeEnv::new();

        for name in ["print", "println"] {
            let ty = env.lookup_function(name).unwrap();
            match ty {
                Type::Function(params, ret) => {
                    assert_eq!(params, &vec![Type::Any]);
                    assert_eq!(**ret, Type::Unit);
                }
                _ => panic!("{} should be a function type", name),
            }
        }
    }

    #[test]
    fn test_builtins_type_conversion() {
        let env = TypeEnv::new();

        let int_ty = env.lookup_function("int").unwrap();
        assert_eq!(int_ty.return_type(), Some(&Type::Int));

        let float_ty = env.lookup_function("float").unwrap();
        assert_eq!(float_ty.return_type(), Some(&Type::Float));

        let str_ty = env.lookup_function("str").unwrap();
        assert_eq!(str_ty.return_type(), Some(&Type::String));

        let bool_ty = env.lookup_function("bool").unwrap();
        assert_eq!(bool_ty.return_type(), Some(&Type::Bool));
    }

    #[test]
    fn test_builtins_type_checking() {
        let env = TypeEnv::new();

        for name in ["is_int", "is_float", "is_string", "is_bool", "is_array", "is_map"] {
            let ty = env.lookup_function(name).unwrap();
            match ty {
                Type::Function(params, ret) => {
                    assert_eq!(params, &vec![Type::Any]);
                    assert_eq!(**ret, Type::Bool);
                }
                _ => panic!("{} should be a function type", name),
            }
        }
    }

    // ==================== fresh_var Tests ====================

    #[test]
    fn test_fresh_var() {
        let mut env = TypeEnv::new();

        let v0 = env.fresh_var();
        let v1 = env.fresh_var();
        let v2 = env.fresh_var();

        assert_eq!(v0, Type::Var(0));
        assert_eq!(v1, Type::Var(1));
        assert_eq!(v2, Type::Var(2));
    }

    #[test]
    fn test_fresh_var_sequential() {
        let mut env = TypeEnv::new();

        for i in 0..100 {
            let v = env.fresh_var();
            assert_eq!(v, Type::Var(i));
        }
    }

    // ==================== Variable Binding Tests ====================

    #[test]
    fn test_bind_var() {
        let mut env = TypeEnv::new();

        env.bind_var("x".to_string(), Type::Int);
        assert_eq!(env.lookup_var("x"), Some(&Type::Int));
    }

    #[test]
    fn test_bind_var_overwrite() {
        let mut env = TypeEnv::new();

        env.bind_var("x".to_string(), Type::Int);
        env.bind_var("x".to_string(), Type::String);
        assert_eq!(env.lookup_var("x"), Some(&Type::String));
    }

    #[test]
    fn test_lookup_var_not_found() {
        let env = TypeEnv::new();
        assert_eq!(env.lookup_var("nonexistent"), None);
    }

    #[test]
    fn test_bind_multiple_vars() {
        let mut env = TypeEnv::new();

        env.bind_var("a".to_string(), Type::Int);
        env.bind_var("b".to_string(), Type::String);
        env.bind_var("c".to_string(), Type::Bool);

        assert_eq!(env.lookup_var("a"), Some(&Type::Int));
        assert_eq!(env.lookup_var("b"), Some(&Type::String));
        assert_eq!(env.lookup_var("c"), Some(&Type::Bool));
    }

    // ==================== Function Registration Tests ====================

    #[test]
    fn test_register_function() {
        let mut env = TypeEnv::new();

        let func_type = Type::Function(vec![Type::Int], Box::new(Type::Int));
        env.register_function("myFunc".to_string(), func_type.clone());

        assert_eq!(env.lookup_function("myFunc"), Some(&func_type));
    }

    #[test]
    fn test_lookup_function_not_found() {
        let env = TypeEnv::new();
        assert_eq!(env.lookup_function("nonexistent_func"), None);
    }

    // ==================== resolve Tests ====================

    #[test]
    fn test_resolve_primitive() {
        let env = TypeEnv::new();

        assert_eq!(env.resolve(&Type::Int), Type::Int);
        assert_eq!(env.resolve(&Type::String), Type::String);
        assert_eq!(env.resolve(&Type::Bool), Type::Bool);
    }

    #[test]
    fn test_resolve_unbound_var() {
        let env = TypeEnv::new();

        assert_eq!(env.resolve(&Type::Var(0)), Type::Var(0));
    }

    #[test]
    fn test_resolve_bound_var() {
        let mut env = TypeEnv::new();
        env.substitution.insert(0, Type::Int);

        assert_eq!(env.resolve(&Type::Var(0)), Type::Int);
    }

    #[test]
    fn test_resolve_chain() {
        let mut env = TypeEnv::new();
        env.substitution.insert(0, Type::Var(1));
        env.substitution.insert(1, Type::Int);

        // T0 -> T1 -> Int
        assert_eq!(env.resolve(&Type::Var(0)), Type::Int);
    }

    #[test]
    fn test_resolve_array() {
        let mut env = TypeEnv::new();
        env.substitution.insert(0, Type::Int);

        let arr = Type::Array(Box::new(Type::Var(0)));
        assert_eq!(env.resolve(&arr), Type::Array(Box::new(Type::Int)));
    }

    #[test]
    fn test_resolve_tuple() {
        let mut env = TypeEnv::new();
        env.substitution.insert(0, Type::Int);
        env.substitution.insert(1, Type::String);

        let tuple = Type::Tuple(vec![Type::Var(0), Type::Var(1)]);
        assert_eq!(env.resolve(&tuple), Type::Tuple(vec![Type::Int, Type::String]));
    }

    #[test]
    fn test_resolve_map() {
        let mut env = TypeEnv::new();
        env.substitution.insert(0, Type::String);
        env.substitution.insert(1, Type::Int);

        let map = Type::Map(Box::new(Type::Var(0)), Box::new(Type::Var(1)));
        assert_eq!(env.resolve(&map), Type::Map(Box::new(Type::String), Box::new(Type::Int)));
    }

    #[test]
    fn test_resolve_struct() {
        let mut env = TypeEnv::new();
        env.substitution.insert(0, Type::Int);

        let mut fields = HashMap::new();
        fields.insert("field".to_string(), Type::Var(0));
        let struct_type = Type::Struct(fields);

        let resolved = env.resolve(&struct_type);
        match resolved {
            Type::Struct(f) => {
                assert_eq!(f.get("field"), Some(&Type::Int));
            }
            _ => panic!("should resolve to struct"),
        }
    }

    #[test]
    fn test_resolve_function() {
        let mut env = TypeEnv::new();
        env.substitution.insert(0, Type::Int);
        env.substitution.insert(1, Type::String);

        let func = Type::Function(vec![Type::Var(0)], Box::new(Type::Var(1)));
        assert_eq!(
            env.resolve(&func),
            Type::Function(vec![Type::Int], Box::new(Type::String))
        );
    }

    #[test]
    fn test_resolve_optional() {
        let mut env = TypeEnv::new();
        env.substitution.insert(0, Type::Int);

        let opt = Type::Optional(Box::new(Type::Var(0)));
        assert_eq!(env.resolve(&opt), Type::Optional(Box::new(Type::Int)));
    }

    #[test]
    fn test_resolve_result() {
        let mut env = TypeEnv::new();
        env.substitution.insert(0, Type::String);

        let res = Type::Result(Box::new(Type::Var(0)));
        assert_eq!(env.resolve(&res), Type::Result(Box::new(Type::String)));
    }

    // ==================== unify Tests ====================

    #[test]
    fn test_unify_same_type() {
        let mut env = TypeEnv::new();

        assert!(env.unify(&Type::Int, &Type::Int, dummy_span()).is_ok());
        assert!(env.unify(&Type::String, &Type::String, dummy_span()).is_ok());
        assert!(env.unify(&Type::Bool, &Type::Bool, dummy_span()).is_ok());
    }

    #[test]
    fn test_unify_with_any() {
        let mut env = TypeEnv::new();

        assert!(env.unify(&Type::Any, &Type::Int, dummy_span()).is_ok());
        assert!(env.unify(&Type::String, &Type::Any, dummy_span()).is_ok());
        assert!(env.unify(&Type::Any, &Type::Any, dummy_span()).is_ok());
    }

    #[test]
    fn test_unify_type_var_left() {
        let mut env = TypeEnv::new();
        let var = env.fresh_var();

        assert!(env.unify(&var, &Type::Int, dummy_span()).is_ok());
        assert_eq!(env.resolve(&var), Type::Int);
    }

    #[test]
    fn test_unify_type_var_right() {
        let mut env = TypeEnv::new();
        let var = env.fresh_var();

        assert!(env.unify(&Type::String, &var, dummy_span()).is_ok());
        assert_eq!(env.resolve(&var), Type::String);
    }

    #[test]
    fn test_unify_two_vars() {
        let mut env = TypeEnv::new();
        let v0 = env.fresh_var();
        let v1 = env.fresh_var();

        assert!(env.unify(&v0, &v1, dummy_span()).is_ok());

        // After unifying with Int, both should resolve to Int
        assert!(env.unify(&v0, &Type::Int, dummy_span()).is_ok());
        assert_eq!(env.resolve(&v0), Type::Int);
        assert_eq!(env.resolve(&v1), Type::Int);
    }

    #[test]
    fn test_unify_array() {
        let mut env = TypeEnv::new();

        let arr1 = Type::Array(Box::new(Type::Int));
        let arr2 = Type::Array(Box::new(Type::Int));
        assert!(env.unify(&arr1, &arr2, dummy_span()).is_ok());
    }

    #[test]
    fn test_unify_array_with_var() {
        let mut env = TypeEnv::new();
        let var = env.fresh_var();

        let arr1 = Type::Array(Box::new(var.clone()));
        let arr2 = Type::Array(Box::new(Type::Int));

        assert!(env.unify(&arr1, &arr2, dummy_span()).is_ok());
        assert_eq!(env.resolve(&var), Type::Int);
    }

    #[test]
    fn test_unify_tuple() {
        let mut env = TypeEnv::new();

        let t1 = Type::Tuple(vec![Type::Int, Type::String]);
        let t2 = Type::Tuple(vec![Type::Int, Type::String]);
        assert!(env.unify(&t1, &t2, dummy_span()).is_ok());
    }

    #[test]
    fn test_unify_tuple_with_vars() {
        let mut env = TypeEnv::new();
        let v0 = env.fresh_var();
        let v1 = env.fresh_var();

        let t1 = Type::Tuple(vec![v0.clone(), v1.clone()]);
        let t2 = Type::Tuple(vec![Type::Int, Type::String]);

        assert!(env.unify(&t1, &t2, dummy_span()).is_ok());
        assert_eq!(env.resolve(&v0), Type::Int);
        assert_eq!(env.resolve(&v1), Type::String);
    }

    #[test]
    fn test_unify_tuple_length_mismatch() {
        let mut env = TypeEnv::new();

        let t1 = Type::Tuple(vec![Type::Int]);
        let t2 = Type::Tuple(vec![Type::Int, Type::String]);

        assert!(env.unify(&t1, &t2, dummy_span()).is_err());
    }

    #[test]
    fn test_unify_function() {
        let mut env = TypeEnv::new();

        let f1 = Type::Function(vec![Type::Int], Box::new(Type::String));
        let f2 = Type::Function(vec![Type::Int], Box::new(Type::String));
        assert!(env.unify(&f1, &f2, dummy_span()).is_ok());
    }

    #[test]
    fn test_unify_function_with_vars() {
        let mut env = TypeEnv::new();
        let v0 = env.fresh_var();
        let v1 = env.fresh_var();

        let f1 = Type::Function(vec![v0.clone()], Box::new(v1.clone()));
        let f2 = Type::Function(vec![Type::Int], Box::new(Type::Bool));

        assert!(env.unify(&f1, &f2, dummy_span()).is_ok());
        assert_eq!(env.resolve(&v0), Type::Int);
        assert_eq!(env.resolve(&v1), Type::Bool);
    }

    #[test]
    fn test_unify_function_param_count_mismatch() {
        let mut env = TypeEnv::new();

        let f1 = Type::Function(vec![Type::Int], Box::new(Type::Int));
        let f2 = Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Int));

        assert!(env.unify(&f1, &f2, dummy_span()).is_err());
    }

    #[test]
    fn test_unify_optional() {
        let mut env = TypeEnv::new();
        let var = env.fresh_var();

        let o1 = Type::Optional(Box::new(var.clone()));
        let o2 = Type::Optional(Box::new(Type::Int));

        assert!(env.unify(&o1, &o2, dummy_span()).is_ok());
        assert_eq!(env.resolve(&var), Type::Int);
    }

    #[test]
    fn test_unify_result() {
        let mut env = TypeEnv::new();
        let var = env.fresh_var();

        let r1 = Type::Result(Box::new(var.clone()));
        let r2 = Type::Result(Box::new(Type::String));

        assert!(env.unify(&r1, &r2, dummy_span()).is_ok());
        assert_eq!(env.resolve(&var), Type::String);
    }

    #[test]
    fn test_unify_map() {
        let mut env = TypeEnv::new();
        let k_var = env.fresh_var();
        let v_var = env.fresh_var();

        let m1 = Type::Map(Box::new(k_var.clone()), Box::new(v_var.clone()));
        let m2 = Type::Map(Box::new(Type::String), Box::new(Type::Int));

        assert!(env.unify(&m1, &m2, dummy_span()).is_ok());
        assert_eq!(env.resolve(&k_var), Type::String);
        assert_eq!(env.resolve(&v_var), Type::Int);
    }

    #[test]
    fn test_unify_struct() {
        let mut env = TypeEnv::new();
        let var = env.fresh_var();

        let mut f1 = HashMap::new();
        f1.insert("x".to_string(), var.clone());
        let s1 = Type::Struct(f1);

        let mut f2 = HashMap::new();
        f2.insert("x".to_string(), Type::Int);
        let s2 = Type::Struct(f2);

        assert!(env.unify(&s1, &s2, dummy_span()).is_ok());
        assert_eq!(env.resolve(&var), Type::Int);
    }

    #[test]
    fn test_unify_struct_missing_field() {
        let mut env = TypeEnv::new();

        let mut f1 = HashMap::new();
        f1.insert("x".to_string(), Type::Int);
        f1.insert("y".to_string(), Type::Int);
        let s1 = Type::Struct(f1);

        let mut f2 = HashMap::new();
        f2.insert("x".to_string(), Type::Int);
        let s2 = Type::Struct(f2);

        // Different field counts
        assert!(env.unify(&s1, &s2, dummy_span()).is_err());
    }

    #[test]
    fn test_unify_numeric_conversion() {
        let mut env = TypeEnv::new();

        // Int and Float can unify
        assert!(env.unify(&Type::Int, &Type::Float, dummy_span()).is_ok());
        assert!(env.unify(&Type::Float, &Type::Int, dummy_span()).is_ok());
    }

    #[test]
    fn test_unify_mismatch() {
        let mut env = TypeEnv::new();

        assert!(env.unify(&Type::Int, &Type::String, dummy_span()).is_err());
        assert!(env.unify(&Type::Bool, &Type::Int, dummy_span()).is_err());
        assert!(env.unify(&Type::Array(Box::new(Type::Int)), &Type::Int, dummy_span()).is_err());
    }

    // ==================== Occurs Check Tests ====================

    #[test]
    fn test_occurs_check() {
        let mut env = TypeEnv::new();
        let var = env.fresh_var();

        // T0 = Array<T0> should fail (infinite type)
        let infinite = Type::Array(Box::new(var.clone()));
        let result = env.unify(&var, &infinite, dummy_span());

        assert!(result.is_err());
        match result.unwrap_err() {
            TypeError::InfiniteType(_) => {}
            e => panic!("Expected InfiniteType error, got {:?}", e),
        }
    }

    #[test]
    fn test_occurs_check_nested() {
        let mut env = TypeEnv::new();
        let var = env.fresh_var();

        // T0 = Function<Int, Tuple<T0, String>> should fail
        let infinite = Type::Function(
            vec![Type::Int],
            Box::new(Type::Tuple(vec![var.clone(), Type::String]))
        );
        let result = env.unify(&var, &infinite, dummy_span());

        assert!(result.is_err());
    }

    // ==================== Complex Unification Tests ====================

    #[test]
    fn test_unify_complex_function_type() {
        let mut env = TypeEnv::new();
        let a = env.fresh_var();
        let b = env.fresh_var();

        // (T0 -> T1) with (Int -> String)
        let f1 = Type::Function(vec![a.clone()], Box::new(b.clone()));
        let f2 = Type::Function(vec![Type::Int], Box::new(Type::String));

        assert!(env.unify(&f1, &f2, dummy_span()).is_ok());
        assert_eq!(env.resolve(&a), Type::Int);
        assert_eq!(env.resolve(&b), Type::String);
    }

    #[test]
    fn test_unify_nested_containers() {
        let mut env = TypeEnv::new();
        let elem_var = env.fresh_var();

        // Array<Optional<T0>> with Array<Optional<Int>>
        let t1 = Type::Array(Box::new(Type::Optional(Box::new(elem_var.clone()))));
        let t2 = Type::Array(Box::new(Type::Optional(Box::new(Type::Int))));

        assert!(env.unify(&t1, &t2, dummy_span()).is_ok());
        assert_eq!(env.resolve(&elem_var), Type::Int);
    }

    #[test]
    fn test_unify_preserves_resolved_vars() {
        let mut env = TypeEnv::new();
        let v0 = env.fresh_var();
        let v1 = env.fresh_var();

        // First unify v0 = Int
        assert!(env.unify(&v0, &Type::Int, dummy_span()).is_ok());

        // Then unify v1 = v0 (should make v1 = Int too)
        assert!(env.unify(&v1, &v0, dummy_span()).is_ok());
        assert_eq!(env.resolve(&v1), Type::Int);
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_type_env_clone() {
        let mut env = TypeEnv::new();
        env.bind_var("x".to_string(), Type::Int);
        env.register_function("f".to_string(), Type::Function(vec![], Box::new(Type::Unit)));

        let cloned = env.clone();
        assert_eq!(cloned.lookup_var("x"), Some(&Type::Int));
        assert!(cloned.lookup_function("f").is_some());
    }

    // ==================== Current Function Tests ====================

    #[test]
    fn test_current_function() {
        let mut env = TypeEnv::new();
        assert!(env.current_function.is_none());

        let func_type = Type::Function(vec![Type::Int], Box::new(Type::Int));
        env.current_function = Some(("myFunc".to_string(), func_type.clone()));

        assert!(env.current_function.is_some());
        let (name, ty) = env.current_function.as_ref().unwrap();
        assert_eq!(name, "myFunc");
        assert_eq!(*ty, func_type);
    }
}
