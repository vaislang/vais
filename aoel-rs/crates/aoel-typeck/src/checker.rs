//! AOEL Type Checker

use aoel_ast::*;
use aoel_lexer::Span;

use crate::error::{TypeError, TypeResult};
use crate::infer::TypeEnv;
use crate::types::Type;

/// 프로그램 타입 체크
pub fn check(program: &Program) -> TypeResult<()> {
    let mut checker = TypeChecker::new();
    checker.check_program(program)?;
    Ok(())
}

/// 타입 체커
pub struct TypeChecker {
    env: TypeEnv,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new(),
        }
    }

    /// 프로그램 타입 체크
    pub fn check_program(&mut self, program: &Program) -> TypeResult<()> {
        // 1단계: 함수 시그니처 수집
        for item in &program.items {
            if let Item::Function(func) = item {
                self.register_function(func)?;
            }
        }

        // 2단계: 함수 본문 체크
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    self.check_function(func)?;
                }
                Item::Expr(expr) => {
                    self.infer_expr(expr)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// 함수 시그니처 등록
    fn register_function(&mut self, func: &FunctionDef) -> TypeResult<()> {
        let param_types: Vec<Type> = func
            .params
            .iter()
            .map(|p| {
                p.ty.as_ref()
                    .map(|t| self.convert_type_expr(t))
                    .unwrap_or_else(|| self.env.fresh_var())
            })
            .collect();

        let return_type = func
            .return_type
            .as_ref()
            .map(|t| self.convert_type_expr(t))
            .unwrap_or_else(|| self.env.fresh_var());

        let func_type = Type::Function(param_types, Box::new(return_type));
        self.env.register_function(func.name.clone(), func_type);

        Ok(())
    }

    /// 함수 타입 체크
    fn check_function(&mut self, func: &FunctionDef) -> TypeResult<()> {
        // 현재 함수 설정 (재귀용)
        if let Some(func_type) = self.env.lookup_function(&func.name).cloned() {
            self.env.current_function = Some((func.name.clone(), func_type.clone()));

            // 매개변수 바인딩
            if let Type::Function(param_types, _) = &func_type {
                for (param, ty) in func.params.iter().zip(param_types.iter()) {
                    self.env.bind_var(param.name.clone(), ty.clone());
                }
            }
        }

        // 본문 타입 추론
        let body_type = self.infer_expr(&func.body)?;

        // 반환 타입과 통일
        if let Some(Type::Function(_, ret)) = self.env.lookup_function(&func.name).cloned() {
            self.env.unify(&body_type, &ret, func.span)?;
        }

        self.env.current_function = None;
        Ok(())
    }

    /// 표현식 타입 추론
    fn infer_expr(&mut self, expr: &Expr) -> TypeResult<Type> {
        match expr {
            // 리터럴
            Expr::Integer(_, _) => Ok(Type::Int),
            Expr::Float(_, _) => Ok(Type::Float),
            Expr::String(_, _) => Ok(Type::String),
            Expr::Bool(_, _) => Ok(Type::Bool),
            Expr::Nil(_) => Ok(Type::Unit),

            // 식별자
            Expr::Ident(name, span) => {
                if let Some(ty) = self.env.lookup_var(name) {
                    Ok(ty.clone())
                } else if let Some(ty) = self.env.lookup_function(name) {
                    Ok(ty.clone())
                } else {
                    Err(TypeError::UndefinedVariable {
                        name: name.clone(),
                        span: *span,
                    })
                }
            }

            // 람다 파라미터 (_)
            Expr::LambdaParam(span) => {
                if let Some(ty) = self.env.lookup_var("_") {
                    Ok(ty.clone())
                } else {
                    // 람다 컨텍스트 외부에서 사용
                    Err(TypeError::UndefinedVariable {
                        name: "_".to_string(),
                        span: *span,
                    })
                }
            }

            // 단항 연산자
            Expr::Unary(op, operand, span) => {
                let operand_type = self.infer_expr(operand)?;
                self.check_unary_op(*op, &operand_type, *span)
            }

            // 이항 연산자
            Expr::Binary(left, op, right, span) => {
                let left_type = self.infer_expr(left)?;
                let right_type = self.infer_expr(right)?;
                self.check_binary_op(&left_type, *op, &right_type, *span)
            }

            // 삼항 연산자
            Expr::Ternary(cond, then_expr, else_expr, span) => {
                let cond_type = self.infer_expr(cond)?;
                self.env.unify(&cond_type, &Type::Bool, *span)?;

                let then_type = self.infer_expr(then_expr)?;
                let else_type = self.infer_expr(else_expr)?;
                self.env.unify(&then_type, &else_type, *span)?;

                Ok(then_type)
            }

            // 함수 호출
            Expr::Call(callee, args, span) => {
                let callee_type = self.infer_expr(callee)?;
                self.check_function_call(&callee_type, args, *span)
            }

            // 재귀 호출
            Expr::SelfCall(args, span) => {
                if let Some((_, func_type)) = &self.env.current_function.clone() {
                    self.check_function_call(func_type, args, *span)
                } else {
                    Err(TypeError::RecursiveInference { span: *span })
                }
            }

            // 배열
            Expr::Array(elements, span) => {
                if elements.is_empty() {
                    let elem_type = self.env.fresh_var();
                    Ok(Type::Array(Box::new(elem_type)))
                } else {
                    let first_type = self.infer_expr(&elements[0])?;
                    for elem in elements.iter().skip(1) {
                        let elem_type = self.infer_expr(elem)?;
                        self.env.unify(&first_type, &elem_type, *span)?;
                    }
                    Ok(Type::Array(Box::new(first_type)))
                }
            }

            // 튜플
            Expr::Tuple(elements, _) => {
                let types: Vec<Type> = elements
                    .iter()
                    .map(|e| self.infer_expr(e))
                    .collect::<TypeResult<Vec<_>>>()?;
                Ok(Type::Tuple(types))
            }

            // 맵
            Expr::Map(entries, _) => {
                if entries.is_empty() {
                    let key_type = self.env.fresh_var();
                    let val_type = self.env.fresh_var();
                    Ok(Type::Map(Box::new(key_type), Box::new(val_type)))
                } else {
                    let val_types: Vec<Type> = entries
                        .iter()
                        .map(|(_, v)| self.infer_expr(v))
                        .collect::<TypeResult<Vec<_>>>()?;

                    // 모든 값 타입 통일
                    let first_val = &val_types[0];
                    for ty in val_types.iter().skip(1) {
                        self.env.unify(first_val, ty, Span::default())?;
                    }

                    Ok(Type::Map(
                        Box::new(Type::String),
                        Box::new(first_val.clone()),
                    ))
                }
            }

            // 인덱스 접근
            Expr::Index(base, index_kind, span) => {
                let base_type = self.infer_expr(base)?;
                self.check_index(&base_type, index_kind, *span)
            }

            // 필드 접근
            Expr::Field(base, field, span) => {
                let base_type = self.infer_expr(base)?;
                self.check_field_access(&base_type, field, *span)
            }

            // 범위
            Expr::Range(start, end, span) => {
                let start_type = self.infer_expr(start)?;
                let end_type = self.infer_expr(end)?;
                self.env.unify(&start_type, &Type::Int, *span)?;
                self.env.unify(&end_type, &Type::Int, *span)?;
                Ok(Type::Array(Box::new(Type::Int)))
            }

            // Map 연산
            Expr::MapOp(base, mapper, span) => {
                let base_type = self.infer_expr(base)?;
                let resolved = self.env.resolve(&base_type);

                // 타입 변수면 배열로 통일
                if let Type::Var(_) = resolved {
                    let elem_type = self.env.fresh_var();
                    self.env.unify(&base_type, &Type::Array(Box::new(elem_type.clone())), *span)?;
                    self.env.bind_var("_".to_string(), elem_type);
                    let result_type = self.infer_expr(mapper)?;
                    Ok(Type::Array(Box::new(result_type)))
                } else if let Type::Array(elem_type) = resolved {
                    // _ 변수를 요소 타입으로 바인딩
                    self.env.bind_var("_".to_string(), (*elem_type).clone());
                    let result_type = self.infer_expr(mapper)?;
                    Ok(Type::Array(Box::new(result_type)))
                } else {
                    Err(TypeError::InvalidOperator {
                        op: "map (.@)".to_string(),
                        ty: base_type.to_string(),
                        span: *span,
                    })
                }
            }

            // Filter 연산
            Expr::FilterOp(base, predicate, span) => {
                let base_type = self.infer_expr(base)?;
                let resolved = self.env.resolve(&base_type);

                // 타입 변수면 배열로 통일
                if let Type::Var(_) = resolved {
                    let elem_type = self.env.fresh_var();
                    self.env.unify(&base_type, &Type::Array(Box::new(elem_type.clone())), *span)?;
                    self.env.bind_var("_".to_string(), elem_type.clone());
                    let pred_type = self.infer_expr(predicate)?;
                    self.env.unify(&pred_type, &Type::Bool, *span)?;
                    Ok(Type::Array(Box::new(elem_type)))
                } else if let Type::Array(elem_type) = resolved {
                    self.env.bind_var("_".to_string(), (*elem_type).clone());
                    let pred_type = self.infer_expr(predicate)?;
                    self.env.unify(&pred_type, &Type::Bool, *span)?;
                    Ok(Type::Array(elem_type))
                } else {
                    Err(TypeError::InvalidOperator {
                        op: "filter (.?)".to_string(),
                        ty: base_type.to_string(),
                        span: *span,
                    })
                }
            }

            // Reduce 연산
            Expr::ReduceOp(base, kind, span) => {
                let base_type = self.infer_expr(base)?;
                let resolved = self.env.resolve(&base_type);

                // 타입 변수면 배열로 통일
                let elem_type = if let Type::Var(_) = resolved {
                    let elem = self.env.fresh_var();
                    self.env.unify(&base_type, &Type::Array(Box::new(elem.clone())), *span)?;
                    elem
                } else if let Type::Array(elem) = resolved {
                    (*elem).clone()
                } else {
                    return Err(TypeError::InvalidOperator {
                        op: "reduce (./)".to_string(),
                        ty: base_type.to_string(),
                        span: *span,
                    });
                };

                match kind {
                    ReduceKind::Sum | ReduceKind::Product => {
                        if elem_type.is_numeric() {
                            Ok(elem_type)
                        } else {
                            self.env.unify(&elem_type, &Type::Int, *span)?;
                            Ok(Type::Int)
                        }
                    }
                    ReduceKind::Min | ReduceKind::Max => Ok(elem_type),
                    ReduceKind::And | ReduceKind::Or => Ok(Type::Bool),
                    ReduceKind::Custom(_, _) => {
                        // 커스텀 리듀스의 경우 Any 반환
                        Ok(Type::Any)
                    }
                }
            }

            // Let 바인딩
            Expr::Let(bindings, body, _) => {
                for (name, value) in bindings {
                    let value_type = self.infer_expr(value)?;
                    self.env.bind_var(name.clone(), value_type);
                }
                self.infer_expr(body)
            }

            // If 표현식
            Expr::If(cond, then_expr, else_expr, span) => {
                let cond_type = self.infer_expr(cond)?;
                self.env.unify(&cond_type, &Type::Bool, *span)?;

                let then_type = self.infer_expr(then_expr)?;

                if let Some(else_e) = else_expr {
                    let else_type = self.infer_expr(else_e)?;
                    self.env.unify(&then_type, &else_type, *span)?;
                    Ok(then_type)
                } else {
                    // else 없으면 Optional
                    Ok(Type::Optional(Box::new(then_type)))
                }
            }

            // 블록
            Expr::Block(exprs, _) => {
                let mut last_type = Type::Unit;
                for e in exprs {
                    last_type = self.infer_expr(e)?;
                }
                Ok(last_type)
            }

            // Try
            Expr::Try(inner, span) => {
                let inner_type = self.infer_expr(inner)?;
                match self.env.resolve(&inner_type) {
                    Type::Optional(t) => Ok((*t).clone()),
                    Type::Result(t) => Ok((*t).clone()),
                    _ => Err(TypeError::InvalidOperator {
                        op: "try (?)".to_string(),
                        ty: inner_type.to_string(),
                        span: *span,
                    }),
                }
            }

            // Contains
            Expr::Contains(elem, container, span) => {
                let elem_type = self.infer_expr(elem)?;
                let container_type = self.infer_expr(container)?;

                match self.env.resolve(&container_type) {
                    Type::Array(inner) => {
                        self.env.unify(&elem_type, &inner, *span)?;
                        Ok(Type::Bool)
                    }
                    Type::String => {
                        self.env.unify(&elem_type, &Type::String, *span)?;
                        Ok(Type::Bool)
                    }
                    _ => Err(TypeError::InvalidOperator {
                        op: "contains (@)".to_string(),
                        ty: container_type.to_string(),
                        span: *span,
                    }),
                }
            }

            // 메서드 호출
            Expr::MethodCall(base, method, args, span) => {
                let base_type = self.infer_expr(base)?;
                self.check_method_call(&base_type, method, args, *span)
            }

            // Error
            Expr::Error(_, _) => Ok(Type::Never),

            // Match 표현식
            Expr::Match(scrutinee, arms, span) => {
                // TODO: 향후 패턴 타입 검증에 사용
                let _scrutinee_type = self.infer_expr(scrutinee)?;
                if arms.is_empty() {
                    return Ok(Type::Never);
                }

                // 첫 번째 arm의 결과 타입
                let first_result = self.infer_expr(&arms[0].body)?;

                // 모든 arm의 결과 타입이 동일한지 확인
                for arm in arms.iter().skip(1) {
                    let result_type = self.infer_expr(&arm.body)?;
                    self.env.unify(&first_result, &result_type, *span)?;
                }

                Ok(first_result)
            }

            // Lambda 표현식
            Expr::Lambda(params, body, _) => {
                // 파라미터 타입 생성
                let param_types: Vec<Type> = params.iter().map(|_| self.env.fresh_var()).collect();

                // 파라미터 바인딩
                for (name, ty) in params.iter().zip(param_types.iter()) {
                    self.env.bind_var(name.clone(), ty.clone());
                }

                // 본문 타입 추론
                let return_type = self.infer_expr(body)?;

                Ok(Type::Function(param_types, Box::new(return_type)))
            }

            // Coalesce 표현식 (a ?? b)
            Expr::Coalesce(left, right, span) => {
                let left_type = self.infer_expr(left)?;
                let right_type = self.infer_expr(right)?;

                // left가 Optional이면 inner와 right 통일
                if let Type::Optional(inner) = self.env.resolve(&left_type) {
                    self.env.unify(&inner, &right_type, *span)?;
                    Ok((*inner).clone())
                } else {
                    Ok(left_type)
                }
            }
        }
    }

    /// 단항 연산자 타입 체크
    fn check_unary_op(&mut self, op: UnaryOp, operand: &Type, span: Span) -> TypeResult<Type> {
        let resolved = self.env.resolve(operand);
        match op {
            UnaryOp::Neg => {
                if resolved.is_numeric() {
                    Ok(resolved)
                } else {
                    Err(TypeError::InvalidOperator {
                        op: "-".to_string(),
                        ty: resolved.to_string(),
                        span,
                    })
                }
            }
            UnaryOp::Not => {
                self.env.unify(&resolved, &Type::Bool, span)?;
                Ok(Type::Bool)
            }
            UnaryOp::Len => Ok(Type::Int),
        }
    }

    /// 이항 연산자 타입 체크
    fn check_binary_op(
        &mut self,
        left: &Type,
        op: BinaryOp,
        right: &Type,
        span: Span,
    ) -> TypeResult<Type> {
        let left_resolved = self.env.resolve(left);
        let right_resolved = self.env.resolve(right);

        match op {
            // 산술 연산자
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                // Add 연산은 배열/문자열 연결도 가능하므로 특별 처리
                if op == BinaryOp::Add {
                    // 한쪽이 배열이면 다른 쪽도 배열로 통일
                    if matches!(left_resolved, Type::Array(_)) {
                        self.env.unify(left, right, span)?;
                        return Ok(self.env.resolve(left));
                    }
                    if matches!(right_resolved, Type::Array(_)) {
                        self.env.unify(left, right, span)?;
                        return Ok(self.env.resolve(right));
                    }
                    // 한쪽이 문자열이면 문자열 연결
                    if matches!(left_resolved, Type::String) || matches!(right_resolved, Type::String) {
                        self.env.unify(left, &Type::String, span)?;
                        self.env.unify(right, &Type::String, span)?;
                        return Ok(Type::String);
                    }
                }

                // 타입 변수가 있으면 숫자 타입으로 통일 (배열/문자열이 아닌 경우)
                if matches!(left_resolved, Type::Var(_)) {
                    self.env.unify(left, &Type::Int, span)?;
                }
                if matches!(right_resolved, Type::Var(_)) {
                    self.env.unify(right, &Type::Int, span)?;
                }

                // 다시 resolve해서 통일 결과 확인
                let left_final = self.env.resolve(left);
                let right_final = self.env.resolve(right);

                self.env.unify(left, right, span)?;

                if left_final.is_numeric() || right_final.is_numeric() {
                    // Float가 있으면 Float, 아니면 Int
                    if matches!(left_final, Type::Float) || matches!(right_final, Type::Float)
                    {
                        Ok(Type::Float)
                    } else {
                        Ok(Type::Int)
                    }
                } else if matches!(left_final, Type::String) {
                    // 문자열 연결
                    Ok(Type::String)
                } else if matches!(left_final, Type::Array(_)) {
                    // 배열 연결
                    Ok(left_final)
                } else {
                    Err(TypeError::InvalidOperator {
                        op: format!("{:?}", op),
                        ty: left_final.to_string(),
                        span,
                    })
                }
            }

            // 비교 연산자
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => {
                // 타입 변수가 있으면 숫자 타입으로 통일
                if matches!(left_resolved, Type::Var(_)) {
                    self.env.unify(left, &Type::Int, span)?;
                }
                if matches!(right_resolved, Type::Var(_)) {
                    self.env.unify(right, &Type::Int, span)?;
                }
                self.env.unify(left, right, span)?;
                Ok(Type::Bool)
            }

            // 등호 연산자
            BinaryOp::Eq | BinaryOp::NotEq => {
                self.env.unify(left, right, span)?;
                Ok(Type::Bool)
            }

            // 논리 연산자
            BinaryOp::And | BinaryOp::Or => {
                self.env.unify(left, &Type::Bool, span)?;
                self.env.unify(right, &Type::Bool, span)?;
                Ok(Type::Bool)
            }

            // Concat
            BinaryOp::Concat => {
                self.env.unify(left, right, span)?;
                match &left_resolved {
                    Type::String => Ok(Type::String),
                    Type::Array(_) => Ok(left_resolved),
                    _ => Err(TypeError::InvalidOperator {
                        op: "++".to_string(),
                        ty: left_resolved.to_string(),
                        span,
                    }),
                }
            }
        }
    }

    /// 함수 호출 타입 체크
    fn check_function_call(
        &mut self,
        func_type: &Type,
        args: &[Expr],
        span: Span,
    ) -> TypeResult<Type> {
        let resolved = self.env.resolve(func_type);

        if let Type::Function(param_types, return_type) = resolved {
            if args.len() != param_types.len() {
                return Err(TypeError::ArgumentCount {
                    expected: param_types.len(),
                    found: args.len(),
                    span,
                });
            }

            for (arg, param_type) in args.iter().zip(param_types.iter()) {
                let arg_type = self.infer_expr(arg)?;
                self.env.unify(&arg_type, param_type, span)?;
            }

            Ok((*return_type).clone())
        } else if let Type::Var(_) = resolved {
            // 타입 변수인 경우: 함수 타입으로 통일 시도
            // 파라미터 타입은 인자로부터 추론, 반환 타입은 새 변수
            let param_types: Vec<Type> = args
                .iter()
                .map(|arg| self.infer_expr(arg))
                .collect::<TypeResult<Vec<_>>>()?;
            let return_type = self.env.fresh_var();
            let inferred_func_type = Type::Function(param_types, Box::new(return_type.clone()));
            self.env.unify(func_type, &inferred_func_type, span)?;
            Ok(return_type)
        } else {
            Err(TypeError::NotAFunction {
                ty: resolved.to_string(),
                span,
            })
        }
    }

    /// 인덱스 접근 타입 체크
    fn check_index(&mut self, base: &Type, index_kind: &IndexKind, span: Span) -> TypeResult<Type> {
        let resolved = self.env.resolve(base);

        match index_kind {
            IndexKind::Single(index_expr) => {
                let index_type = self.infer_expr(index_expr)?;

                match &resolved {
                    Type::Array(elem) => {
                        self.env.unify(&index_type, &Type::Int, span)?;
                        Ok((**elem).clone())
                    }
                    Type::String => {
                        self.env.unify(&index_type, &Type::Int, span)?;
                        Ok(Type::String)
                    }
                    Type::Map(_, v) => Ok((**v).clone()),
                    Type::Tuple(_types) => {
                        // 튜플 인덱싱은 상수 인덱스만 지원
                        self.env.unify(&index_type, &Type::Int, span)?;
                        // 반환 타입은 Any (정적으로 결정 불가)
                        Ok(Type::Any)
                    }
                    // 타입 변수인 경우: 배열로 통일
                    Type::Var(_) => {
                        self.env.unify(&index_type, &Type::Int, span)?;
                        let elem_type = self.env.fresh_var();
                        self.env.unify(base, &Type::Array(Box::new(elem_type.clone())), span)?;
                        Ok(elem_type)
                    }
                    _ => Err(TypeError::InvalidIndex {
                        base: resolved.to_string(),
                        index: index_type.to_string(),
                        span,
                    }),
                }
            }
            IndexKind::Slice(start, end) => {
                if let Some(s) = start {
                    let s_type = self.infer_expr(s)?;
                    self.env.unify(&s_type, &Type::Int, span)?;
                }
                if let Some(e) = end {
                    let e_type = self.infer_expr(e)?;
                    self.env.unify(&e_type, &Type::Int, span)?;
                }

                match &resolved {
                    Type::Array(_) => Ok(resolved.clone()),
                    Type::String => Ok(Type::String),
                    // 타입 변수인 경우: 배열로 통일
                    Type::Var(_) => {
                        let elem_type = self.env.fresh_var();
                        let arr_type = Type::Array(Box::new(elem_type));
                        self.env.unify(base, &arr_type, span)?;
                        Ok(arr_type)
                    }
                    _ => Err(TypeError::InvalidIndex {
                        base: resolved.to_string(),
                        index: "slice".to_string(),
                        span,
                    }),
                }
            }
        }
    }

    /// 필드 접근 타입 체크
    fn check_field_access(&mut self, base: &Type, field: &str, span: Span) -> TypeResult<Type> {
        let resolved = self.env.resolve(base);

        match &resolved {
            Type::Struct(fields) => {
                if let Some(ty) = fields.get(field) {
                    Ok(ty.clone())
                } else {
                    Err(TypeError::InvalidField {
                        field: field.to_string(),
                        ty: resolved.to_string(),
                        span,
                    })
                }
            }
            Type::Map(_, v) => Ok((**v).clone()),
            _ => Err(TypeError::InvalidField {
                field: field.to_string(),
                ty: resolved.to_string(),
                span,
            }),
        }
    }

    /// 메서드 호출 타입 체크
    fn check_method_call(
        &mut self,
        base: &Type,
        method: &str,
        args: &[Expr],
        span: Span,
    ) -> TypeResult<Type> {
        let resolved = self.env.resolve(base);

        // 빌트인 메서드
        match method {
            "len" => {
                match &resolved {
                    Type::Array(_) | Type::String | Type::Map(_, _) => Ok(Type::Int),
                    _ => Err(TypeError::InvalidField {
                        field: method.to_string(),
                        ty: resolved.to_string(),
                        span,
                    }),
                }
            }
            "push" | "pop" | "first" | "last" => {
                if let Type::Array(elem) = &resolved {
                    match method {
                        "push" => {
                            if args.len() != 1 {
                                return Err(TypeError::ArgumentCount {
                                    expected: 1,
                                    found: args.len(),
                                    span,
                                });
                            }
                            let arg_type = self.infer_expr(&args[0])?;
                            self.env.unify(&arg_type, elem, span)?;
                            Ok(Type::Unit)
                        }
                        "pop" | "first" | "last" => Ok(Type::Optional(elem.clone())),
                        _ => unreachable!(),
                    }
                } else {
                    Err(TypeError::InvalidField {
                        field: method.to_string(),
                        ty: resolved.to_string(),
                        span,
                    })
                }
            }
            _ => {
                // 사용자 정의 메서드는 지원하지 않음
                Err(TypeError::InvalidField {
                    field: method.to_string(),
                    ty: resolved.to_string(),
                    span,
                })
            }
        }
    }

    /// TypeExpr을 Type으로 변환
    fn convert_type_expr(&self, type_expr: &TypeExpr) -> Type {
        match type_expr {
            TypeExpr::Simple(name) => match name.as_str() {
                "Int" | "int" => Type::Int,
                "Float" | "float" => Type::Float,
                "String" | "string" | "str" => Type::String,
                "Bool" | "bool" => Type::Bool,
                "Unit" | "()" => Type::Unit,
                "Any" | "any" => Type::Any,
                _ => Type::Any, // 사용자 정의 타입은 Any로 처리
            },
            TypeExpr::Array(inner) => Type::Array(Box::new(self.convert_type_expr(inner))),
            TypeExpr::Tuple(types) => {
                Type::Tuple(types.iter().map(|t| self.convert_type_expr(t)).collect())
            }
            TypeExpr::Optional(inner) => Type::Optional(Box::new(self.convert_type_expr(inner))),
            TypeExpr::Result(inner) => Type::Result(Box::new(self.convert_type_expr(inner))),
            TypeExpr::Function(params, ret) => Type::Function(
                params.iter().map(|t| self.convert_type_expr(t)).collect(),
                Box::new(self.convert_type_expr(ret)),
            ),
            TypeExpr::Map(k, v) => Type::Map(
                Box::new(self.convert_type_expr(k)),
                Box::new(self.convert_type_expr(v)),
            ),
            TypeExpr::Struct(fields) => Type::Struct(
                fields
                    .iter()
                    .map(|(k, v)| (k.clone(), self.convert_type_expr(v)))
                    .collect(),
            ),
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_source(source: &str) -> TypeResult<()> {
        let program = aoel_parser::parse(source).unwrap();
        check(&program)
    }

    #[test]
    fn test_simple_function() {
        let result = check_source("add(a: Int, b: Int) -> Int = a + b");
        assert!(result.is_ok());
    }

    #[test]
    fn test_inferred_types() {
        let result = check_source("double(x) = x * 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_map() {
        let result = check_source("double_all(arr: [Int]) = arr.@(_ * 2)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_recursion() {
        let result = check_source("fact(n: Int) -> Int = n < 2 ? 1 : n * $(n - 1)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_fibonacci() {
        // Tests multiple recursive calls with arithmetic
        let result = check_source("fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_function_calls() {
        // Tests deeply nested function calls
        let result = check_source(
            "double(x) = x * 2
             triple(x) = x * 3
             compose(x) = double(triple(x))"
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_mutual_recursion() {
        // Tests mutual recursion between two functions
        let result = check_source(
            "is_even(n) = n == 0 ? true : is_odd(n - 1)
             is_odd(n) = n == 0 ? false : is_even(n - 1)"
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_higher_order_function() {
        // Tests passing function as parameter
        let result = check_source("apply_twice(f, x) = f(f(x))");
        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_arithmetic() {
        // Tests multiple parameters with arithmetic
        let result = check_source("quadratic(a, b, c, x) = a * x * x + b * x + c");
        assert!(result.is_ok());
    }

    #[test]
    fn test_conditional_type_inference() {
        // Tests conditional expressions with negation
        let result = check_source("abs(x) = x < 0 ? -x : x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tree_recursion() {
        // Tests tree-shaped recursive calls
        let result = check_source("tree_sum(n) = n <= 0 ? 0 : n + tree_sum(n - 1) + tree_sum(n - 2)");
        assert!(result.is_ok());
    }
}
