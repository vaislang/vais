//! v6b Type Inference
//!
//! Hindley-Milner 스타일 타입 추론

use std::collections::HashMap;

use aoel_lexer::Span;

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
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut env = Self {
            vars: HashMap::new(),
            functions: HashMap::new(),
            current_function: None,
            next_var: 0,
            substitution: HashMap::new(),
        };
        env.register_builtins();
        env
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
