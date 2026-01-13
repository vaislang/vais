//! Vais AST to IR Lowering
//!
//! Vais AST를 IR 명령어로 변환

use vais_ir::{Instruction, OpCode, ReduceOp as IrReduceOp, Value};
use vais_ast::{
    BinaryOp, Expr, FfiBlock, FunctionDef, ImplDef, IndexKind, Item, Pattern, Program, ReduceKind, TraitDef, UnaryOp,
};
use std::collections::HashMap;
use thiserror::Error;

/// Lowering 에러
#[derive(Debug, Error)]
pub enum LowerError {
    #[error("Unsupported expression: {0}")]
    UnsupportedExpr(String),

    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
}

pub type LowerResult<T> = Result<T, LowerError>;

/// 컴파일된 함수
#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub name: String,
    pub params: Vec<String>,
    pub instructions: Vec<Instruction>,
    /// 로컬 변수 슬롯 개수 (params + locals)
    pub local_count: u16,
}

/// FFI 함수 정보
#[derive(Debug, Clone)]
pub struct FfiFnInfo {
    pub lib_name: String,
    pub extern_name: String,
    pub param_count: usize,
}

/// 로컬 변수 스코프 (변수명 -> 인덱스)
#[derive(Debug, Clone, Default)]
struct LocalScope {
    /// 변수명 -> 로컬 인덱스 매핑
    locals: HashMap<String, u16>,
    /// 다음 할당할 인덱스
    next_index: u16,
}

impl LocalScope {
    fn new() -> Self {
        Self::default()
    }

    /// 변수 인덱스 조회 (없으면 None)
    fn get(&self, name: &str) -> Option<u16> {
        self.locals.get(name).copied()
    }

    /// 새 변수 등록하고 인덱스 반환
    fn declare(&mut self, name: &str) -> u16 {
        if let Some(&idx) = self.locals.get(name) {
            idx // 이미 선언된 변수는 기존 인덱스 반환
        } else {
            let idx = self.next_index;
            self.locals.insert(name.to_string(), idx);
            self.next_index += 1;
            idx
        }
    }

    /// 현재까지 할당된 로컬 변수 개수
    fn count(&self) -> u16 {
        self.next_index
    }
}

/// Vais Lowerer
pub struct Lowerer {
    /// 현재 함수 이름 (재귀 호출용)
    current_function: Option<String>,
    /// 컴파일된 함수들
    functions: Vec<CompiledFunction>,
    /// FFI 함수 레지스트리: vais_name -> (lib_name, extern_name, param_count)
    ffi_functions: HashMap<String, FfiFnInfo>,
    /// 현재 함수의 로컬 스코프
    scope: LocalScope,
}

impl Lowerer {
    pub fn new() -> Self {
        Self {
            current_function: None,
            functions: Vec::new(),
            ffi_functions: HashMap::new(),
            scope: LocalScope::new(),
        }
    }

    /// FFI 함수 레지스트리 반환
    pub fn ffi_functions(&self) -> &HashMap<String, FfiFnInfo> {
        &self.ffi_functions
    }

    /// 프로그램을 컴파일
    pub fn lower_program(&mut self, program: &Program) -> LowerResult<Vec<CompiledFunction>> {
        for item in &program.items {
            match item {
                Item::Function(f) => {
                    let compiled = self.lower_function(f)?;
                    self.functions.push(compiled);
                }
                Item::Expr(e) => {
                    // REPL용: main 함수로 래핑
                    self.scope = LocalScope::new();
                    let instructions = self.lower_expr(e)?;
                    self.functions.push(CompiledFunction {
                        name: "__main__".to_string(),
                        params: Vec::new(),
                        instructions,
                        local_count: self.scope.count(),
                    });
                }
                Item::Ffi(ffi_block) => {
                    // FFI 블록 처리: 함수 레지스트리에 등록
                    self.register_ffi_block(ffi_block);
                }
                Item::Trait(trait_def) => {
                    // Trait 기본 구현 컴파일
                    self.lower_trait(trait_def)?;
                }
                Item::Impl(impl_def) => {
                    // Impl 메서드 컴파일
                    self.lower_impl(impl_def)?;
                }
                _ => {
                    // TypeDef, Module, Use, Enum은 런타임에 직접 영향 없음
                }
            }
        }

        Ok(std::mem::take(&mut self.functions))
    }

    /// 함수 정의를 컴파일
    fn lower_function(&mut self, func: &FunctionDef) -> LowerResult<CompiledFunction> {
        self.current_function = Some(func.name.clone());

        // 새 스코프 생성
        self.scope = LocalScope::new();

        let params: Vec<String> = func.params.iter().map(|p| p.name.clone()).collect();

        // 매개변수를 로컬 스코프에 등록 (인덱스 0부터 시작)
        for param in &params {
            self.scope.declare(param);
        }

        let mut instructions = Vec::new();

        // 매개변수는 VM이 직접 locals에 저장해주므로 Store 불필요

        // 함수 본문 컴파일
        let body_instrs = self.lower_expr(&func.body)?;
        instructions.extend(body_instrs);

        // Return 추가
        instructions.push(Instruction::new(OpCode::Return));

        let local_count = self.scope.count();
        self.current_function = None;

        Ok(CompiledFunction {
            name: func.name.clone(),
            params,
            instructions,
            local_count,
        })
    }

    /// Trait 정의를 컴파일 (기본 구현이 있는 메서드만)
    fn lower_trait(&mut self, trait_def: &TraitDef) -> LowerResult<()> {
        for method in &trait_def.methods {
            // 기본 구현이 있는 경우에만 컴파일
            if let Some(ref default_body) = method.default_impl {
                self.current_function = Some(format!("{}::{}", trait_def.name, method.name));

                // 새 스코프 생성
                self.scope = LocalScope::new();

                let params: Vec<String> = method.params.iter().map(|p| p.name.clone()).collect();

                // 매개변수를 로컬 스코프에 등록
                for param in &params {
                    self.scope.declare(param);
                }

                let mut instructions = Vec::new();
                let body_instrs = self.lower_expr(default_body)?;
                instructions.extend(body_instrs);
                instructions.push(Instruction::new(OpCode::Return));

                let local_count = self.scope.count();
                self.current_function = None;

                self.functions.push(CompiledFunction {
                    name: format!("{}::{}", trait_def.name, method.name),
                    params,
                    instructions,
                    local_count,
                });
            }
        }
        Ok(())
    }

    /// Impl 블록을 컴파일
    fn lower_impl(&mut self, impl_def: &ImplDef) -> LowerResult<()> {
        // 타입 이름 추출 (간단한 버전)
        let type_name = self.type_expr_to_string(&impl_def.target_type);

        for method in &impl_def.methods {
            // 메서드 이름: Type::method 형식
            let method_name = format!("{}::{}", type_name, method.name);

            self.current_function = Some(method_name.clone());

            // 새 스코프 생성
            self.scope = LocalScope::new();

            let params: Vec<String> = method.params.iter().map(|p| p.name.clone()).collect();

            // 매개변수를 로컬 스코프에 등록
            for param in &params {
                self.scope.declare(param);
            }

            let mut instructions = Vec::new();
            let body_instrs = self.lower_expr(&method.body)?;
            instructions.extend(body_instrs);
            instructions.push(Instruction::new(OpCode::Return));

            let local_count = self.scope.count();
            self.current_function = None;

            self.functions.push(CompiledFunction {
                name: method_name,
                params,
                instructions,
                local_count,
            });
        }
        Ok(())
    }

    /// TypeExpr를 문자열로 변환 (간단한 버전)
    fn type_expr_to_string(&self, type_expr: &vais_ast::TypeExpr) -> String {
        match type_expr {
            vais_ast::TypeExpr::Simple(name) => name.clone(),
            vais_ast::TypeExpr::TypeVar(name) => name.clone(),
            vais_ast::TypeExpr::Generic(name, _) => name.clone(),
            vais_ast::TypeExpr::Array(_) => "Array".to_string(),
            vais_ast::TypeExpr::Set(_) => "Set".to_string(),
            vais_ast::TypeExpr::Map(_, _) => "Map".to_string(),
            vais_ast::TypeExpr::Tuple(_) => "Tuple".to_string(),
            vais_ast::TypeExpr::Optional(_) => "Optional".to_string(),
            vais_ast::TypeExpr::Result(_) => "Result".to_string(),
            vais_ast::TypeExpr::Function(_, _) => "Function".to_string(),
            vais_ast::TypeExpr::Struct(_) => "Struct".to_string(),
            vais_ast::TypeExpr::Future(_) => "Future".to_string(),
            vais_ast::TypeExpr::Channel(_) => "Channel".to_string(),
        }
    }

    /// 표현식을 IR로 변환
    fn lower_expr(&mut self, expr: &Expr) -> LowerResult<Vec<Instruction>> {
        let mut instrs = Vec::new();

        match expr {
            // === Literals ===
            Expr::Integer(n, _) => {
                instrs.push(Instruction::new(OpCode::Const(Value::Int(*n))));
            }

            Expr::Float(f, _) => {
                instrs.push(Instruction::new(OpCode::Const(Value::Float(*f))));
            }

            Expr::String(s, _) => {
                instrs.push(Instruction::new(OpCode::Const(Value::String(s.clone()))));
            }

            Expr::Bool(b, _) => {
                instrs.push(Instruction::new(OpCode::Const(Value::Bool(*b))));
            }

            Expr::Nil(_) => {
                instrs.push(Instruction::new(OpCode::Const(Value::Void)));
            }

            // === Identifiers ===
            Expr::Ident(name, _) => {
                // 로컬 변수인지 확인
                if let Some(idx) = self.scope.get(name) {
                    instrs.push(Instruction::new(OpCode::LoadLocal(idx)));
                } else {
                    // 글로벌/클로저 변수
                    instrs.push(Instruction::new(OpCode::Load(name.clone())));
                }
            }

            Expr::LambdaParam(_) => {
                // 람다 내 _ 참조: 특수 변수 로드
                if let Some(idx) = self.scope.get("_") {
                    instrs.push(Instruction::new(OpCode::LoadLocal(idx)));
                } else {
                    instrs.push(Instruction::new(OpCode::Load("_".to_string())));
                }
            }

            // === Collections ===
            Expr::Array(elements, _) => {
                for elem in elements {
                    instrs.extend(self.lower_expr(elem)?);
                }
                instrs.push(Instruction::new(OpCode::MakeArray(elements.len())));
            }

            Expr::Set(elements, _) => {
                for elem in elements {
                    instrs.extend(self.lower_expr(elem)?);
                }
                instrs.push(Instruction::new(OpCode::MakeSet(elements.len())));
            }

            Expr::Map(fields, _) => {
                let field_names: Vec<String> = fields.iter().map(|(k, _)| k.clone()).collect();
                for (_, v) in fields {
                    instrs.extend(self.lower_expr(v)?);
                }
                instrs.push(Instruction::new(OpCode::MakeStruct(field_names)));
            }

            Expr::Tuple(elements, _) => {
                // 튜플은 배열로 처리
                for elem in elements {
                    instrs.extend(self.lower_expr(elem)?);
                }
                instrs.push(Instruction::new(OpCode::MakeArray(elements.len())));
            }

            // === Binary Operations ===
            Expr::Binary(left, op, right, _) => {
                instrs.extend(self.lower_expr(left)?);
                instrs.extend(self.lower_expr(right)?);

                let opcode = match op {
                    BinaryOp::Add => OpCode::Add,
                    BinaryOp::Sub => OpCode::Sub,
                    BinaryOp::Mul => OpCode::Mul,
                    BinaryOp::Div => OpCode::Div,
                    BinaryOp::Mod => OpCode::Mod,
                    BinaryOp::Eq => OpCode::Eq,
                    BinaryOp::NotEq => OpCode::Neq,
                    BinaryOp::Lt => OpCode::Lt,
                    BinaryOp::Gt => OpCode::Gt,
                    BinaryOp::LtEq => OpCode::Lte,
                    BinaryOp::GtEq => OpCode::Gte,
                    BinaryOp::And => OpCode::And,
                    BinaryOp::Or => OpCode::Or,
                    BinaryOp::Concat => OpCode::Concat,
                };
                instrs.push(Instruction::new(opcode));
            }

            // === Unary Operations ===
            Expr::Unary(op, inner, _) => {
                instrs.extend(self.lower_expr(inner)?);

                let opcode = match op {
                    UnaryOp::Neg => OpCode::Neg,
                    UnaryOp::Not => OpCode::Not,
                    UnaryOp::Len => OpCode::Len,
                };
                instrs.push(Instruction::new(opcode));
            }

            // === Vais Collection Operations ===
            Expr::MapOp(arr, transform, _) => {
                instrs.extend(self.lower_expr(arr)?);

                // 단순 패턴 최적화: _ * const, _ + const 등
                if let Some(opcode) = self.try_optimize_map(transform) {
                    instrs.push(Instruction::new(opcode));
                } else {
                    let transform_instrs = self.lower_expr(transform)?;
                    instrs.push(Instruction::new(OpCode::Map(Box::new(transform_instrs))));
                }
            }

            Expr::FilterOp(arr, predicate, _) => {
                instrs.extend(self.lower_expr(arr)?);

                // 단순 패턴 최적화: _ > const, _ < const 등
                if let Some(opcode) = self.try_optimize_filter(predicate) {
                    instrs.push(Instruction::new(opcode));
                } else {
                    let pred_instrs = self.lower_expr(predicate)?;
                    instrs.push(Instruction::new(OpCode::Filter(Box::new(pred_instrs))));
                }
            }

            Expr::ReduceOp(arr, kind, _) => {
                instrs.extend(self.lower_expr(arr)?);

                let (reduce_op, init_value) = match kind {
                    ReduceKind::Sum => (IrReduceOp::Sum, Value::Int(0)),
                    ReduceKind::Product => (IrReduceOp::Product, Value::Int(1)),
                    ReduceKind::Min => (IrReduceOp::Min, Value::Void),
                    ReduceKind::Max => (IrReduceOp::Max, Value::Void),
                    ReduceKind::And => (IrReduceOp::All, Value::Bool(true)),
                    ReduceKind::Or => (IrReduceOp::Any, Value::Bool(false)),
                    ReduceKind::Custom(init, func) => {
                        let init_instrs = self.lower_expr(init)?;
                        let func_instrs = self.lower_expr(func)?;
                        // 복잡한 케이스: 일단 Sum으로 대체
                        instrs.extend(init_instrs);
                        (IrReduceOp::Custom(Box::new(func_instrs)), Value::Void)
                    }
                };

                instrs.push(Instruction::new(OpCode::Reduce(reduce_op, init_value)));
            }

            // === Access ===
            Expr::Field(obj, field, _) => {
                instrs.extend(self.lower_expr(obj)?);
                instrs.push(Instruction::new(OpCode::GetField(field.clone())));
            }

            Expr::Index(arr, kind, _) => {
                instrs.extend(self.lower_expr(arr)?);

                match kind.as_ref() {
                    IndexKind::Single(idx) => {
                        instrs.extend(self.lower_expr(idx)?);
                        instrs.push(Instruction::new(OpCode::Index));
                    }
                    IndexKind::Slice(start, end) => {
                        // start
                        if let Some(s) = start {
                            instrs.extend(self.lower_expr(s)?);
                        } else {
                            instrs.push(Instruction::new(OpCode::Const(Value::Int(0))));
                        }
                        // end
                        if let Some(e) = end {
                            instrs.extend(self.lower_expr(e)?);
                        } else {
                            // -1은 끝까지를 의미
                            instrs.push(Instruction::new(OpCode::Const(Value::Int(-1))));
                        }
                        instrs.push(Instruction::new(OpCode::Slice));
                    }
                }
            }

            Expr::MethodCall(obj, method, args, _) => {
                // obj.method(args) -> method(obj, args)
                instrs.extend(self.lower_expr(obj)?);
                for arg in args {
                    instrs.extend(self.lower_expr(arg)?);
                }
                instrs.push(Instruction::new(OpCode::CallBuiltin(
                    method.clone(),
                    args.len() + 1,
                )));
            }

            // === Control Flow ===
            Expr::Ternary(cond, then_expr, else_expr, _) => {
                // cond ? then : else
                instrs.extend(self.lower_expr(cond)?);

                let then_instrs = self.lower_expr(then_expr)?;
                let else_instrs = self.lower_expr(else_expr)?;

                // JumpIfNot to else
                let then_len = then_instrs.len() as i32;
                let else_len = else_instrs.len() as i32;

                instrs.push(Instruction::new(OpCode::JumpIfNot(then_len + 1)));
                instrs.extend(then_instrs);
                instrs.push(Instruction::new(OpCode::Jump(else_len)));
                instrs.extend(else_instrs);
            }

            Expr::If(cond, then_expr, else_expr, _) => {
                instrs.extend(self.lower_expr(cond)?);

                let then_instrs = self.lower_expr(then_expr)?;
                let else_instrs = if let Some(e) = else_expr {
                    self.lower_expr(e)?
                } else {
                    vec![Instruction::new(OpCode::Const(Value::Void))]
                };

                let then_len = then_instrs.len() as i32;
                let else_len = else_instrs.len() as i32;

                instrs.push(Instruction::new(OpCode::JumpIfNot(then_len + 1)));
                instrs.extend(then_instrs);
                instrs.push(Instruction::new(OpCode::Jump(else_len)));
                instrs.extend(else_instrs);
            }

            Expr::Match(scrutinee, arms, _) => {
                // scrutinee를 평가하고 임시 변수에 저장
                instrs.extend(self.lower_expr(scrutinee)?);
                let match_idx = self.scope.declare("__match_val__");
                instrs.push(Instruction::new(OpCode::StoreLocal(match_idx)));

                if arms.is_empty() {
                    // arm이 없으면 Void 반환
                    instrs.push(Instruction::new(OpCode::Const(Value::Void)));
                } else {
                    // 각 arm의 (조건 코드, 본문 코드)를 준비
                    let mut compiled_arms: Vec<(Vec<Instruction>, Vec<Instruction>)> = Vec::new();

                    for arm in arms {
                        // 조건 생성: 패턴 매칭 + guard
                        let mut cond_instrs = Vec::new();
                        cond_instrs.push(Instruction::new(OpCode::LoadLocal(match_idx)));
                        cond_instrs.extend(self.lower_pattern(&arm.pattern)?);

                        // guard 조건이 있으면 AND
                        if let Some(guard) = &arm.guard {
                            cond_instrs.extend(self.lower_expr(guard)?);
                            cond_instrs.push(Instruction::new(OpCode::And));
                        }

                        // 본문 코드
                        let body_instrs = self.lower_expr(&arm.body)?;

                        compiled_arms.push((cond_instrs, body_instrs));
                    }

                    // if-else 체인으로 변환 (역순으로 처리)
                    // 마지막 arm부터 시작하여 else 체인 구축
                    let result_instrs = self.build_match_chain(&compiled_arms)?;
                    instrs.extend(result_instrs);
                }
            }

            Expr::Block(exprs, _) => {
                for (i, e) in exprs.iter().enumerate() {
                    instrs.extend(self.lower_expr(e)?);
                    // 마지막이 아니면 Pop
                    if i < exprs.len() - 1 {
                        instrs.push(Instruction::new(OpCode::Pop));
                    }
                }
            }

            // === Binding ===
            Expr::Let(bindings, body, _) => {
                // 각 바인딩 처리
                for (name, value, _is_mut) in bindings {
                    instrs.extend(self.lower_expr(value)?);
                    // 로컬 변수로 등록하고 StoreLocal 사용
                    // (is_mut는 런타임에서는 특별히 처리하지 않음, 타입 체커에서 검증)
                    let idx = self.scope.declare(name);
                    instrs.push(Instruction::new(OpCode::StoreLocal(idx)));
                }
                // 본문 실행
                instrs.extend(self.lower_expr(body)?);
            }

            // 재할당
            Expr::Assign(name, value, _) => {
                instrs.extend(self.lower_expr(value)?);
                // 기존 변수에 재할당 (StoreLocal 사용)
                if let Some(idx) = self.scope.get(name) {
                    instrs.push(Instruction::new(OpCode::StoreLocal(idx)));
                } else {
                    // 글로벌 변수로 Store (fallback)
                    instrs.push(Instruction::new(OpCode::Store(name.clone())));
                }
                // 할당 후 값 반환 (스택에 다시 로드)
                if let Some(idx) = self.scope.get(name) {
                    instrs.push(Instruction::new(OpCode::LoadLocal(idx)));
                } else {
                    instrs.push(Instruction::new(OpCode::Load(name.clone())));
                }
            }

            // === Function ===
            Expr::Call(func, args, _) => {
                // 인자 먼저 푸시
                for arg in args {
                    instrs.extend(self.lower_expr(arg)?);
                }

                // 함수 호출
                if let Expr::Ident(name, _) = func.as_ref() {
                    // FFI 함수 체크
                    if let Some(ffi_info) = self.is_ffi_function(name).cloned() {
                        instrs.push(Instruction::new(OpCode::CallFfi(
                            ffi_info.lib_name,
                            ffi_info.extern_name,
                            args.len(),
                        )));
                    // 빌트인 함수 체크
                    } else if Self::is_builtin(name) {
                        instrs.push(Instruction::new(OpCode::CallBuiltin(name.clone(), args.len())));
                    } else {
                        instrs.push(Instruction::new(OpCode::Call(name.clone(), args.len())));
                    }
                } else {
                    // 함수 표현식 (람다 등)
                    instrs.extend(self.lower_expr(func)?);
                    instrs.push(Instruction::new(OpCode::CallBuiltin(
                        "__call__".to_string(),
                        args.len() + 1,
                    )));
                }
            }

            Expr::SelfCall(args, _) => {
                // 재귀 호출 $()
                for arg in args {
                    instrs.extend(self.lower_expr(arg)?);
                }
                instrs.push(Instruction::new(OpCode::SelfCall(args.len())));
            }

            Expr::Lambda(params, body, _) => {
                // 람다를 클로저로 생성
                let body_instrs = self.lower_expr(body)?;
                instrs.push(Instruction::new(OpCode::MakeClosure(
                    params.clone(),
                    Box::new(body_instrs),
                )));
            }

            // === Special ===
            Expr::Range(start, end, _) => {
                instrs.extend(self.lower_expr(start)?);
                instrs.extend(self.lower_expr(end)?);
                instrs.push(Instruction::new(OpCode::Range));
            }

            Expr::Contains(elem, arr, _) => {
                instrs.extend(self.lower_expr(elem)?);
                instrs.extend(self.lower_expr(arr)?);
                instrs.push(Instruction::new(OpCode::Contains));
            }

            Expr::Error(msg, _) => {
                if let Some(m) = msg {
                    instrs.extend(self.lower_expr(m)?);
                    // 스택에서 값을 가져와 에러 생성
                    instrs.push(Instruction::new(OpCode::Error("".to_string())));
                } else {
                    instrs.push(Instruction::new(OpCode::Error("error".to_string())));
                }
            }

            Expr::Try(inner, _) => {
                instrs.extend(self.lower_expr(inner)?);
                instrs.push(Instruction::new(OpCode::Try));
            }

            Expr::Coalesce(value, default, _) => {
                instrs.extend(self.lower_expr(value)?);
                instrs.extend(self.lower_expr(default)?);
                instrs.push(Instruction::new(OpCode::Coalesce));
            }

            Expr::TryCatch { body, error_name, handler, .. } => {
                // try-catch 구현 (상대 점프 사용):
                // [SetCatch relative_offset]  - body_len + 2 (ClearCatch + Jump)
                // [body 명령어들]
                // [ClearCatch]
                // [Jump relative_offset]      - handler_len + 1 (Store 포함)
                // [Store error_name]
                // [handler 명령어들]

                // 먼저 body와 handler를 컴파일
                let body_instrs = self.lower_expr(body)?;
                let handler_instrs = self.lower_expr(handler)?;

                let body_len = body_instrs.len();
                let handler_len = handler_instrs.len();

                // SetCatch: body 다음의 Store 위치로 (body_len + 2: ClearCatch, Jump)
                instrs.push(Instruction::new(OpCode::SetCatch(body_len + 2)));

                // body 실행
                instrs.extend(body_instrs);

                // 핸들러 제거
                instrs.push(Instruction::new(OpCode::ClearCatch));

                // 성공 시 handler 건너뜀 (handler_len + 1: Store 포함)
                instrs.push(Instruction::new(OpCode::Jump((handler_len + 1) as i32)));

                // 에러 저장 (에러 핸들러 시작점)
                let error_idx = self.scope.declare(error_name);
                instrs.push(Instruction::new(OpCode::StoreLocal(error_idx)));

                // handler 실행
                instrs.extend(handler_instrs);
            }

            Expr::Struct(type_name, fields, _) => {
                // Struct 리터럴은 Map처럼 처리 (MakeStruct 사용)
                // __type__ 필드를 추가하여 타입 정보 저장

                // __type__ 값 먼저 push
                instrs.push(Instruction::new(OpCode::Const(Value::String(
                    type_name.clone(),
                ))));

                // 나머지 필드 값 push
                for (_, value) in fields {
                    instrs.extend(self.lower_expr(value)?);
                }

                // 필드 이름 목록 생성 (__type__ 포함)
                let mut field_names = vec!["__type__".to_string()];
                field_names.extend(fields.iter().map(|(k, _)| k.clone()));

                instrs.push(Instruction::new(OpCode::MakeStruct(field_names)));
            }

            // List comprehension: [expr for var in iter if cond]
            // 이를 iter.@(x => cond ? expr : []).flatten() 형태로 변환
            // 또는 간단히 filter와 map 조합으로
            Expr::ListComprehension { expr, var, iter, cond, .. } => {
                // 변수를 로컬 스코프에 등록
                let var_idx = self.scope.declare(var);

                // iter 로드
                instrs.extend(self.lower_expr(iter)?);

                // filter (조건이 있는 경우)
                if let Some(condition) = cond {
                    let filter_body = {
                        let mut body = Vec::new();
                        body.push(Instruction::new(OpCode::StoreLocal(var_idx)));
                        body.extend(self.lower_expr(condition)?);
                        body
                    };
                    instrs.push(Instruction::new(OpCode::Filter(Box::new(filter_body))));
                }

                // map
                let map_body = {
                    let mut body = Vec::new();
                    body.push(Instruction::new(OpCode::StoreLocal(var_idx)));
                    body.extend(self.lower_expr(expr)?);
                    body
                };
                instrs.push(Instruction::new(OpCode::Map(Box::new(map_body))));
            }

            // Set comprehension: #{expr for var in iter if cond}
            Expr::SetComprehension { expr, var, iter, cond, .. } => {
                // 변수를 로컬 스코프에 등록
                let var_idx = self.scope.declare(var);

                // iter 로드
                instrs.extend(self.lower_expr(iter)?);

                // filter (조건이 있는 경우)
                if let Some(condition) = cond {
                    let filter_body = {
                        let mut body = Vec::new();
                        body.push(Instruction::new(OpCode::StoreLocal(var_idx)));
                        body.extend(self.lower_expr(condition)?);
                        body
                    };
                    instrs.push(Instruction::new(OpCode::Filter(Box::new(filter_body))));
                }

                // map
                let map_body = {
                    let mut body = Vec::new();
                    body.push(Instruction::new(OpCode::StoreLocal(var_idx)));
                    body.extend(self.lower_expr(expr)?);
                    body
                };
                instrs.push(Instruction::new(OpCode::Map(Box::new(map_body))));

                // 배열을 세트로 변환 (MakeSet은 스택의 배열을 변환)
                // 여기서는 결과를 배열로 남기고 런타임에서 세트로 변환하도록 함
                // TODO: ArrayToSet opcode 추가 필요
            }

            // Await 표현식
            Expr::Await(inner, _) => {
                instrs.extend(self.lower_expr(inner)?);
                instrs.push(Instruction::new(OpCode::Await));
            }

            // Spawn 표현식 (태스크 생성)
            Expr::Spawn(inner, _) => {
                instrs.extend(self.lower_expr(inner)?);
                instrs.push(Instruction::new(OpCode::Spawn));
            }

            // Channel send: chan <- value
            Expr::Send(chan, value, _) => {
                instrs.extend(self.lower_expr(chan)?);
                instrs.extend(self.lower_expr(value)?);
                instrs.push(Instruction::new(OpCode::Send));
            }

            // Channel receive: <- chan
            Expr::Recv(chan, _) => {
                instrs.extend(self.lower_expr(chan)?);
                instrs.push(Instruction::new(OpCode::Recv));
            }

            // Parallel Map: arr.||@(f)
            Expr::ParallelMap(arr, transform, _) => {
                instrs.extend(self.lower_expr(arr)?);
                let transform_instrs = self.lower_expr(transform)?;
                instrs.push(Instruction::new(OpCode::ParallelMap(Box::new(transform_instrs))));
            }

            // Parallel Filter: arr.||?(p)
            Expr::ParallelFilter(arr, predicate, _) => {
                instrs.extend(self.lower_expr(arr)?);
                let pred_instrs = self.lower_expr(predicate)?;
                instrs.push(Instruction::new(OpCode::ParallelFilter(Box::new(pred_instrs))));
            }

            // Parallel Reduce: arr.||/+
            Expr::ParallelReduce(arr, kind, _) => {
                instrs.extend(self.lower_expr(arr)?);

                let (reduce_op, init_value) = match kind {
                    ReduceKind::Sum => (IrReduceOp::Sum, Value::Int(0)),
                    ReduceKind::Product => (IrReduceOp::Product, Value::Int(1)),
                    ReduceKind::Min => (IrReduceOp::Min, Value::Void),
                    ReduceKind::Max => (IrReduceOp::Max, Value::Void),
                    ReduceKind::And => (IrReduceOp::All, Value::Bool(true)),
                    ReduceKind::Or => (IrReduceOp::Any, Value::Bool(false)),
                    ReduceKind::Custom(init, func) => {
                        let init_instrs = self.lower_expr(init)?;
                        let func_instrs = self.lower_expr(func)?;
                        instrs.extend(init_instrs);
                        (IrReduceOp::Custom(Box::new(func_instrs)), Value::Void)
                    }
                };

                instrs.push(Instruction::new(OpCode::ParallelReduce(reduce_op, init_value)));
            }
        }

        Ok(instrs)
    }

    /// Map 연산 최적화: 단순 패턴 감지
    /// `_ * const`, `_ + const`, `_ - const`, `_ / const` 패턴을 네이티브 opcode로 변환
    fn try_optimize_map(&self, transform: &Expr) -> Option<OpCode> {
        // 패턴: _ op const 또는 const op _
        if let Expr::Binary(left, op, right, _) = transform {
            // _ * const
            if matches!(left.as_ref(), Expr::LambdaParam(_)) {
                if let Expr::Integer(n, _) = right.as_ref() {
                    return match op {
                        BinaryOp::Mul => Some(OpCode::MapMulConst(*n)),
                        BinaryOp::Add => Some(OpCode::MapAddConst(*n)),
                        BinaryOp::Sub => Some(OpCode::MapSubConst(*n)),
                        BinaryOp::Div => Some(OpCode::MapDivConst(*n)),
                        _ => None,
                    };
                }
            }
            // const * _ (교환 가능한 연산만)
            if matches!(right.as_ref(), Expr::LambdaParam(_)) {
                if let Expr::Integer(n, _) = left.as_ref() {
                    return match op {
                        BinaryOp::Mul => Some(OpCode::MapMulConst(*n)),
                        BinaryOp::Add => Some(OpCode::MapAddConst(*n)),
                        _ => None,
                    };
                }
            }
        }
        None
    }

    /// Filter 연산 최적화: 단순 패턴 감지
    /// `_ > const`, `_ < const`, `_ % 2 == 0` 등 패턴을 네이티브 opcode로 변환
    fn try_optimize_filter(&self, predicate: &Expr) -> Option<OpCode> {
        // 패턴: _ op const
        if let Expr::Binary(left, op, right, _) = predicate {
            // _ > const, _ < const 등
            if matches!(left.as_ref(), Expr::LambdaParam(_)) {
                if let Expr::Integer(n, _) = right.as_ref() {
                    return match op {
                        BinaryOp::Gt => Some(OpCode::FilterGtConst(*n)),
                        BinaryOp::Lt => Some(OpCode::FilterLtConst(*n)),
                        BinaryOp::GtEq => Some(OpCode::FilterGteConst(*n)),
                        BinaryOp::LtEq => Some(OpCode::FilterLteConst(*n)),
                        BinaryOp::Eq => Some(OpCode::FilterEqConst(*n)),
                        BinaryOp::NotEq => Some(OpCode::FilterNeqConst(*n)),
                        _ => None,
                    };
                }
            }
            // _ % 2 == 0 (짝수), _ % 2 != 0 (홀수)
            if let Expr::Binary(mod_left, mod_op, mod_right, _) = left.as_ref() {
                if *mod_op == BinaryOp::Mod
                    && matches!(mod_left.as_ref(), Expr::LambdaParam(_))
                {
                    if let Expr::Integer(2, _) = mod_right.as_ref() {
                        if let Expr::Integer(0, _) = right.as_ref() {
                            return match op {
                                BinaryOp::Eq => Some(OpCode::FilterEven),
                                BinaryOp::NotEq => Some(OpCode::FilterOdd),
                                _ => None,
                            };
                        }
                    }
                }
            }
        }
        None
    }

    /// 패턴을 IR로 변환 (매칭 조건 생성)
    /// 스택에 매칭 대상 값이 있고, 결과로 Bool을 푸시
    fn lower_pattern(&mut self, pattern: &Pattern) -> LowerResult<Vec<Instruction>> {
        let mut instrs = Vec::new();

        match pattern {
            Pattern::Wildcard(_) => {
                // 항상 매칭 - 값 버리고 true 반환
                instrs.push(Instruction::new(OpCode::Pop));
                instrs.push(Instruction::new(OpCode::Const(Value::Bool(true))));
            }

            Pattern::Literal(expr) => {
                // 값 비교
                instrs.extend(self.lower_expr(expr)?);
                instrs.push(Instruction::new(OpCode::Eq));
            }

            Pattern::Binding(name, _) => {
                // 값을 변수에 바인딩하고 true 반환
                instrs.push(Instruction::new(OpCode::Dup));
                let idx = self.scope.declare(name);
                instrs.push(Instruction::new(OpCode::StoreLocal(idx)));
                instrs.push(Instruction::new(OpCode::Pop));
                instrs.push(Instruction::new(OpCode::Const(Value::Bool(true))));
            }

            Pattern::Tuple(patterns, _) | Pattern::Array(patterns, _) => {
                // 튜플/배열 패턴: 길이 확인 후 각 요소 매칭
                // 스택: [value]

                // 임시 변수 인덱스 미리 할당
                let pat_acc_idx = self.scope.declare("__pat_acc__");

                // 1. 길이 확인
                instrs.push(Instruction::new(OpCode::Dup));
                instrs.push(Instruction::new(OpCode::Len));
                instrs.push(Instruction::new(OpCode::Const(Value::Int(patterns.len() as i64))));
                instrs.push(Instruction::new(OpCode::Eq));
                // 스택: [value, len_ok]

                // 2. 각 요소에 대해 패턴 매칭
                for (i, pat) in patterns.iter().enumerate() {
                    // 현재 스택: [value, prev_result]
                    // 이전 결과가 false면 스킵하지 않고 계속 검사 (short-circuit 생략)

                    // value를 스택 맨 위로 복사 (Dup은 맨 위만 복사)
                    // 임시 변수에 저장하고 로드하는 방식 사용
                    instrs.push(Instruction::new(OpCode::StoreLocal(pat_acc_idx)));
                    instrs.push(Instruction::new(OpCode::Dup)); // value 복사
                    instrs.push(Instruction::new(OpCode::Const(Value::Int(i as i64))));
                    instrs.push(Instruction::new(OpCode::Index));
                    // 스택: [value, element]

                    // 요소에 대해 재귀적 패턴 매칭
                    instrs.extend(self.lower_pattern(pat)?);
                    // 스택: [value, element_matched]

                    // 이전 결과와 AND
                    instrs.push(Instruction::new(OpCode::LoadLocal(pat_acc_idx)));
                    instrs.push(Instruction::new(OpCode::And));
                    // 스택: [value, combined_result]
                }

                // 최종 결과 저장, value 제거
                instrs.push(Instruction::new(OpCode::StoreLocal(pat_acc_idx)));
                instrs.push(Instruction::new(OpCode::Pop)); // value 제거
                instrs.push(Instruction::new(OpCode::LoadLocal(pat_acc_idx)));
            }

            Pattern::Struct(fields, _) => {
                // 구조체 패턴: 각 필드 매칭
                // 스택: [value]

                if fields.is_empty() {
                    // 빈 구조체 패턴 - 항상 매칭
                    instrs.push(Instruction::new(OpCode::Pop));
                    instrs.push(Instruction::new(OpCode::Const(Value::Bool(true))));
                } else {
                    // 임시 변수 인덱스 미리 할당
                    let pat_acc_idx = self.scope.declare("__pat_acc__");

                    // 첫 번째 필드 처리
                    let (first_name, first_pat) = &fields[0];
                    instrs.push(Instruction::new(OpCode::Dup));
                    instrs.push(Instruction::new(OpCode::GetField(first_name.clone())));

                    if let Some(pat) = first_pat {
                        instrs.extend(self.lower_pattern(pat)?);
                    } else {
                        // 바인딩만: field_name을 변수로 저장
                        instrs.push(Instruction::new(OpCode::Dup));
                        let field_idx = self.scope.declare(first_name);
                        instrs.push(Instruction::new(OpCode::StoreLocal(field_idx)));
                        instrs.push(Instruction::new(OpCode::Pop));
                        instrs.push(Instruction::new(OpCode::Const(Value::Bool(true))));
                    }

                    // 나머지 필드 처리
                    for (field_name, sub_pat) in fields.iter().skip(1) {
                        instrs.push(Instruction::new(OpCode::StoreLocal(pat_acc_idx)));
                        instrs.push(Instruction::new(OpCode::Dup));
                        instrs.push(Instruction::new(OpCode::GetField(field_name.clone())));

                        if let Some(pat) = sub_pat {
                            instrs.extend(self.lower_pattern(pat)?);
                        } else {
                            instrs.push(Instruction::new(OpCode::Dup));
                            let field_idx = self.scope.declare(field_name);
                            instrs.push(Instruction::new(OpCode::StoreLocal(field_idx)));
                            instrs.push(Instruction::new(OpCode::Pop));
                            instrs.push(Instruction::new(OpCode::Const(Value::Bool(true))));
                        }

                        instrs.push(Instruction::new(OpCode::LoadLocal(pat_acc_idx)));
                        instrs.push(Instruction::new(OpCode::And));
                    }

                    // 최종 결과 저장, value 제거
                    instrs.push(Instruction::new(OpCode::StoreLocal(pat_acc_idx)));
                    instrs.push(Instruction::new(OpCode::Pop));
                    instrs.push(Instruction::new(OpCode::LoadLocal(pat_acc_idx)));
                }
            }

            Pattern::Variant(name, inner, _) => {
                // Enum variant 패턴
                // 현재는 간단히 이름 비교만 (태그 확인)
                // TODO: 실제 Enum 타입 시스템 필요
                let pat_acc_idx = self.scope.declare("__pat_acc__");

                instrs.push(Instruction::new(OpCode::Dup));
                instrs.push(Instruction::new(OpCode::GetField("__variant__".to_string())));
                instrs.push(Instruction::new(OpCode::Const(Value::String(name.clone()))));
                instrs.push(Instruction::new(OpCode::Eq));

                if let Some(inner_pat) = inner {
                    // 내부 패턴 매칭
                    instrs.push(Instruction::new(OpCode::StoreLocal(pat_acc_idx)));
                    instrs.push(Instruction::new(OpCode::Dup));
                    instrs.push(Instruction::new(OpCode::GetField("__value__".to_string())));
                    instrs.extend(self.lower_pattern(inner_pat)?);
                    instrs.push(Instruction::new(OpCode::LoadLocal(pat_acc_idx)));
                    instrs.push(Instruction::new(OpCode::And));
                }

                // value 제거
                instrs.push(Instruction::new(OpCode::StoreLocal(pat_acc_idx)));
                instrs.push(Instruction::new(OpCode::Pop));
                instrs.push(Instruction::new(OpCode::LoadLocal(pat_acc_idx)));
            }

            Pattern::Range(start, end, _) => {
                // 범위 패턴: start <= value <= end
                // 스택: [value]
                let pat_acc_idx = self.scope.declare("__pat_acc__");

                instrs.push(Instruction::new(OpCode::Dup));
                instrs.extend(self.lower_expr(start)?);
                instrs.push(Instruction::new(OpCode::Gte)); // value >= start
                // 스택: [value, gte_start]

                instrs.push(Instruction::new(OpCode::StoreLocal(pat_acc_idx)));
                // 스택: [value]

                instrs.extend(self.lower_expr(end)?);
                instrs.push(Instruction::new(OpCode::Lte)); // value <= end
                // 스택: [lte_end]

                instrs.push(Instruction::new(OpCode::LoadLocal(pat_acc_idx)));
                instrs.push(Instruction::new(OpCode::And));
                // 스택: [result]
            }

            Pattern::Or(patterns, _) => {
                // Or 패턴: 하나라도 매칭되면 true
                if patterns.is_empty() {
                    instrs.push(Instruction::new(OpCode::Pop));
                    instrs.push(Instruction::new(OpCode::Const(Value::Bool(false))));
                } else {
                    // 임시 변수 인덱스 할당
                    let or_val_idx = self.scope.declare("__match_or_val__");
                    let pat_acc_idx = self.scope.declare("__pat_acc__");

                    // 값을 임시 저장
                    instrs.push(Instruction::new(OpCode::StoreLocal(or_val_idx)));
                    instrs.push(Instruction::new(OpCode::Const(Value::Bool(false)))); // 초기값

                    for pat in patterns {
                        instrs.push(Instruction::new(OpCode::StoreLocal(pat_acc_idx)));
                        instrs.push(Instruction::new(OpCode::LoadLocal(or_val_idx)));
                        instrs.extend(self.lower_pattern(pat)?);
                        instrs.push(Instruction::new(OpCode::LoadLocal(pat_acc_idx)));
                        instrs.push(Instruction::new(OpCode::Or));
                    }
                }
            }
        }

        Ok(instrs)
    }

    /// Match 표현식을 if-else 체인으로 변환
    /// compiled_arms: Vec<(조건 코드, 본문 코드)>
    fn build_match_chain(
        &self,
        compiled_arms: &[(Vec<Instruction>, Vec<Instruction>)],
    ) -> LowerResult<Vec<Instruction>> {
        if compiled_arms.is_empty() {
            return Ok(vec![Instruction::new(OpCode::Const(Value::Void))]);
        }

        if compiled_arms.len() == 1 {
            // 단일 arm: 조건 체크 후 본문 실행, 실패 시 Void
            let (cond, body) = &compiled_arms[0];
            let mut instrs = Vec::new();

            instrs.extend(cond.clone());
            let body_len = body.len() as i32;
            instrs.push(Instruction::new(OpCode::JumpIfNot(body_len + 1)));
            instrs.extend(body.clone());
            instrs.push(Instruction::new(OpCode::Jump(1))); // else 부분 건너뛰기
            instrs.push(Instruction::new(OpCode::Const(Value::Void))); // 매칭 실패 시

            return Ok(instrs);
        }

        // 여러 arm: 재귀적으로 if-else 체인 구축
        let (cond, body) = &compiled_arms[0];
        let rest = &compiled_arms[1..];

        // 나머지 arm들의 코드 먼저 생성
        let else_instrs = self.build_match_chain(rest)?;

        let mut instrs = Vec::new();
        instrs.extend(cond.clone());

        let body_len = body.len() as i32;
        let else_len = else_instrs.len() as i32;

        instrs.push(Instruction::new(OpCode::JumpIfNot(body_len + 1)));
        instrs.extend(body.clone());
        instrs.push(Instruction::new(OpCode::Jump(else_len)));
        instrs.extend(else_instrs);

        Ok(instrs)
    }

    /// FFI 블록 등록
    fn register_ffi_block(&mut self, ffi_block: &FfiBlock) {
        for ffi_fn in &ffi_block.functions {
            let extern_name = ffi_fn.extern_name.clone()
                .unwrap_or_else(|| ffi_fn.name.clone());

            self.ffi_functions.insert(ffi_fn.name.clone(), FfiFnInfo {
                lib_name: ffi_block.lib_name.clone(),
                extern_name,
                param_count: ffi_fn.params.len(),
            });
        }
    }

    /// FFI 함수 여부 확인
    fn is_ffi_function(&self, name: &str) -> Option<&FfiFnInfo> {
        self.ffi_functions.get(name)
    }

    /// 빌트인 함수 여부 확인
    fn is_builtin(name: &str) -> bool {
        const BUILTINS: &[&str] = &[
            // Math
            "sqrt", "abs", "pow", "sin", "cos", "tan", "log", "log10",
            "floor", "ceil", "round", "min", "max",
            // Collection
            "len", "first", "last", "reverse", "concat", "range",
            // String
            "upper", "lower", "trim", "split", "join", "substr", "replace",
            "contains", "starts_with", "ends_with",
            // Type conversion
            "int", "float", "str", "string", "bool",
            // I/O
            "print", "println",
        ];
        BUILTINS.contains(&name.to_lowercase().as_str())
    }
}

impl Default for Lowerer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_ast::{FunctionDef, MatchArm, Param, Program};
    use vais_lexer::Span;

    fn dummy_span() -> Span {
        Span::new(0, 0)
    }

    fn make_func(name: &str, params: Vec<&str>, body: Expr) -> FunctionDef {
        FunctionDef {
            name: name.to_string(),
            type_params: vec![],
            params: params.into_iter().map(|p| Param {
                name: p.to_string(),
                ty: None,
                default: None,
                span: dummy_span(),
            }).collect(),
            return_type: None,
            body,
            is_pub: false,
            is_async: false,
            is_test: false,
            span: dummy_span(),
        }
    }

    fn int(n: i64) -> Expr {
        Expr::Integer(n, dummy_span())
    }

    fn float(f: f64) -> Expr {
        Expr::Float(f, dummy_span())
    }

    fn string(s: &str) -> Expr {
        Expr::String(s.to_string(), dummy_span())
    }

    fn bool_expr(b: bool) -> Expr {
        Expr::Bool(b, dummy_span())
    }

    fn ident(name: &str) -> Expr {
        Expr::Ident(name.to_string(), dummy_span())
    }

    fn binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
        Expr::Binary(Box::new(left), op, Box::new(right), dummy_span())
    }

    fn unary(op: UnaryOp, expr: Expr) -> Expr {
        Expr::Unary(op, Box::new(expr), dummy_span())
    }

    // === Literal Tests ===

    #[test]
    fn test_literal_integer() {
        let mut lowerer = Lowerer::new();
        let instrs = lowerer.lower_expr(&int(42)).unwrap();
        assert_eq!(instrs.len(), 1);
        assert!(matches!(instrs[0].opcode, OpCode::Const(Value::Int(42))));
    }

    #[test]
    fn test_literal_float() {
        let mut lowerer = Lowerer::new();
        let instrs = lowerer.lower_expr(&float(3.15)).unwrap();
        assert_eq!(instrs.len(), 1);
        if let OpCode::Const(Value::Float(f)) = &instrs[0].opcode {
            assert!((*f - 3.15).abs() < f64::EPSILON);
        } else {
            panic!("Expected float constant");
        }
    }

    #[test]
    fn test_literal_string() {
        let mut lowerer = Lowerer::new();
        let instrs = lowerer.lower_expr(&string("hello")).unwrap();
        assert_eq!(instrs.len(), 1);
        assert!(matches!(instrs[0].opcode, OpCode::Const(Value::String(ref s)) if s == "hello"));
    }

    #[test]
    fn test_literal_bool() {
        let mut lowerer = Lowerer::new();

        let instrs = lowerer.lower_expr(&bool_expr(true)).unwrap();
        assert!(matches!(instrs[0].opcode, OpCode::Const(Value::Bool(true))));

        let instrs = lowerer.lower_expr(&bool_expr(false)).unwrap();
        assert!(matches!(instrs[0].opcode, OpCode::Const(Value::Bool(false))));
    }

    #[test]
    fn test_literal_nil() {
        let mut lowerer = Lowerer::new();
        let instrs = lowerer.lower_expr(&Expr::Nil(dummy_span())).unwrap();
        assert_eq!(instrs.len(), 1);
        assert!(matches!(instrs[0].opcode, OpCode::Const(Value::Void)));
    }

    // === Identifier Tests ===

    #[test]
    fn test_identifier() {
        let mut lowerer = Lowerer::new();
        let instrs = lowerer.lower_expr(&ident("x")).unwrap();
        assert_eq!(instrs.len(), 1);
        assert!(matches!(&instrs[0].opcode, OpCode::Load(name) if name == "x"));
    }

    #[test]
    fn test_lambda_param() {
        let mut lowerer = Lowerer::new();
        let instrs = lowerer.lower_expr(&Expr::LambdaParam(dummy_span())).unwrap();
        assert_eq!(instrs.len(), 1);
        assert!(matches!(&instrs[0].opcode, OpCode::Load(name) if name == "_"));
    }

    // === Binary Operation Tests ===

    #[test]
    fn test_binary_arithmetic() {
        let mut lowerer = Lowerer::new();

        // Addition
        let instrs = lowerer.lower_expr(&binary(int(1), BinaryOp::Add, int(2))).unwrap();
        assert_eq!(instrs.len(), 3);
        assert!(matches!(instrs[2].opcode, OpCode::Add));

        // Subtraction
        let instrs = lowerer.lower_expr(&binary(int(5), BinaryOp::Sub, int(3))).unwrap();
        assert!(matches!(instrs[2].opcode, OpCode::Sub));

        // Multiplication
        let instrs = lowerer.lower_expr(&binary(int(4), BinaryOp::Mul, int(2))).unwrap();
        assert!(matches!(instrs[2].opcode, OpCode::Mul));

        // Division
        let instrs = lowerer.lower_expr(&binary(int(10), BinaryOp::Div, int(2))).unwrap();
        assert!(matches!(instrs[2].opcode, OpCode::Div));

        // Modulo
        let instrs = lowerer.lower_expr(&binary(int(10), BinaryOp::Mod, int(3))).unwrap();
        assert!(matches!(instrs[2].opcode, OpCode::Mod));
    }

    #[test]
    fn test_binary_comparison() {
        let mut lowerer = Lowerer::new();

        let instrs = lowerer.lower_expr(&binary(int(1), BinaryOp::Eq, int(1))).unwrap();
        assert!(matches!(instrs[2].opcode, OpCode::Eq));

        let instrs = lowerer.lower_expr(&binary(int(1), BinaryOp::NotEq, int(2))).unwrap();
        assert!(matches!(instrs[2].opcode, OpCode::Neq));

        let instrs = lowerer.lower_expr(&binary(int(1), BinaryOp::Lt, int(2))).unwrap();
        assert!(matches!(instrs[2].opcode, OpCode::Lt));

        let instrs = lowerer.lower_expr(&binary(int(2), BinaryOp::Gt, int(1))).unwrap();
        assert!(matches!(instrs[2].opcode, OpCode::Gt));

        let instrs = lowerer.lower_expr(&binary(int(1), BinaryOp::LtEq, int(1))).unwrap();
        assert!(matches!(instrs[2].opcode, OpCode::Lte));

        let instrs = lowerer.lower_expr(&binary(int(2), BinaryOp::GtEq, int(2))).unwrap();
        assert!(matches!(instrs[2].opcode, OpCode::Gte));
    }

    #[test]
    fn test_binary_logical() {
        let mut lowerer = Lowerer::new();

        let instrs = lowerer.lower_expr(&binary(bool_expr(true), BinaryOp::And, bool_expr(false))).unwrap();
        assert!(matches!(instrs[2].opcode, OpCode::And));

        let instrs = lowerer.lower_expr(&binary(bool_expr(true), BinaryOp::Or, bool_expr(false))).unwrap();
        assert!(matches!(instrs[2].opcode, OpCode::Or));
    }

    #[test]
    fn test_binary_concat() {
        let mut lowerer = Lowerer::new();
        let instrs = lowerer.lower_expr(&binary(string("a"), BinaryOp::Concat, string("b"))).unwrap();
        assert!(matches!(instrs[2].opcode, OpCode::Concat));
    }

    // === Unary Operation Tests ===

    #[test]
    fn test_unary_neg() {
        let mut lowerer = Lowerer::new();
        let instrs = lowerer.lower_expr(&unary(UnaryOp::Neg, int(5))).unwrap();
        assert_eq!(instrs.len(), 2);
        assert!(matches!(instrs[1].opcode, OpCode::Neg));
    }

    #[test]
    fn test_unary_not() {
        let mut lowerer = Lowerer::new();
        let instrs = lowerer.lower_expr(&unary(UnaryOp::Not, bool_expr(true))).unwrap();
        assert!(matches!(instrs[1].opcode, OpCode::Not));
    }

    #[test]
    fn test_unary_len() {
        let mut lowerer = Lowerer::new();
        let instrs = lowerer.lower_expr(&unary(UnaryOp::Len, ident("arr"))).unwrap();
        assert!(matches!(instrs[1].opcode, OpCode::Len));
    }

    // === Collection Tests ===

    #[test]
    fn test_array_literal() {
        let mut lowerer = Lowerer::new();
        let arr = Expr::Array(vec![int(1), int(2), int(3)], dummy_span());
        let instrs = lowerer.lower_expr(&arr).unwrap();

        // 3 consts + 1 MakeArray
        assert_eq!(instrs.len(), 4);
        assert!(matches!(instrs[3].opcode, OpCode::MakeArray(3)));
    }

    #[test]
    fn test_empty_array() {
        let mut lowerer = Lowerer::new();
        let arr = Expr::Array(vec![], dummy_span());
        let instrs = lowerer.lower_expr(&arr).unwrap();

        assert_eq!(instrs.len(), 1);
        assert!(matches!(instrs[0].opcode, OpCode::MakeArray(0)));
    }

    #[test]
    fn test_map_literal() {
        let mut lowerer = Lowerer::new();
        let map = Expr::Map(
            vec![
                ("x".to_string(), int(1)),
                ("y".to_string(), int(2)),
            ],
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&map).unwrap();

        // 2 consts + 1 MakeStruct
        assert_eq!(instrs.len(), 3);
        if let OpCode::MakeStruct(fields) = &instrs[2].opcode {
            assert_eq!(fields, &vec!["x".to_string(), "y".to_string()]);
        } else {
            panic!("Expected MakeStruct");
        }
    }

    #[test]
    fn test_tuple_literal() {
        let mut lowerer = Lowerer::new();
        let tuple = Expr::Tuple(vec![int(1), string("a")], dummy_span());
        let instrs = lowerer.lower_expr(&tuple).unwrap();

        assert_eq!(instrs.len(), 3);
        assert!(matches!(instrs[2].opcode, OpCode::MakeArray(2)));
    }

    // === Collection Operation Tests ===

    #[test]
    fn test_map_op() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::MapOp(
            Box::new(ident("arr")),
            Box::new(binary(Expr::LambdaParam(dummy_span()), BinaryOp::Mul, int(2))),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        // Load arr + Map (either optimized MapMulConst or generic Map)
        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::Map(_) | OpCode::MapMulConst(_))));
    }

    #[test]
    fn test_filter_op() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::FilterOp(
            Box::new(ident("arr")),
            Box::new(binary(Expr::LambdaParam(dummy_span()), BinaryOp::Gt, int(0))),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        // Either optimized FilterGtConst or generic Filter
        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::Filter(_) | OpCode::FilterGtConst(_))));
    }

    #[test]
    fn test_reduce_op_sum() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::ReduceOp(
            Box::new(ident("arr")),
            ReduceKind::Sum,
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::Reduce(IrReduceOp::Sum, _))));
    }

    #[test]
    fn test_reduce_op_product() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::ReduceOp(
            Box::new(ident("arr")),
            ReduceKind::Product,
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::Reduce(IrReduceOp::Product, _))));
    }

    #[test]
    fn test_reduce_op_min_max() {
        let mut lowerer = Lowerer::new();

        let expr = Expr::ReduceOp(Box::new(ident("arr")), ReduceKind::Min, dummy_span());
        let instrs = lowerer.lower_expr(&expr).unwrap();
        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::Reduce(IrReduceOp::Min, _))));

        let expr = Expr::ReduceOp(Box::new(ident("arr")), ReduceKind::Max, dummy_span());
        let instrs = lowerer.lower_expr(&expr).unwrap();
        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::Reduce(IrReduceOp::Max, _))));
    }

    #[test]
    fn test_reduce_op_all_any() {
        let mut lowerer = Lowerer::new();

        let expr = Expr::ReduceOp(Box::new(ident("arr")), ReduceKind::And, dummy_span());
        let instrs = lowerer.lower_expr(&expr).unwrap();
        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::Reduce(IrReduceOp::All, _))));

        let expr = Expr::ReduceOp(Box::new(ident("arr")), ReduceKind::Or, dummy_span());
        let instrs = lowerer.lower_expr(&expr).unwrap();
        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::Reduce(IrReduceOp::Any, _))));
    }

    // === Access Tests ===

    #[test]
    fn test_field_access() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Field(
            Box::new(ident("obj")),
            "name".to_string(),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert_eq!(instrs.len(), 2);
        assert!(matches!(&instrs[1].opcode, OpCode::GetField(f) if f == "name"));
    }

    #[test]
    fn test_index_single() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Index(
            Box::new(ident("arr")),
            Box::new(IndexKind::Single(int(0))),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Index)));
    }

    #[test]
    fn test_index_slice() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Index(
            Box::new(ident("arr")),
            Box::new(IndexKind::Slice(Some(int(1)), Some(int(3)))),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Slice)));
    }

    #[test]
    fn test_index_slice_defaults() {
        let mut lowerer = Lowerer::new();

        // [..3] - start defaults to 0
        let expr = Expr::Index(
            Box::new(ident("arr")),
            Box::new(IndexKind::Slice(None, Some(int(3)))),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();
        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Const(Value::Int(0)))));

        // [1..] - end defaults to -1
        let expr = Expr::Index(
            Box::new(ident("arr")),
            Box::new(IndexKind::Slice(Some(int(1)), None)),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();
        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Const(Value::Int(-1)))));
    }

    #[test]
    fn test_method_call() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::MethodCall(
            Box::new(string("hello")),
            "upper".to_string(),
            vec![],
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::CallBuiltin(name, 1) if name == "upper")));
    }

    // === Control Flow Tests ===

    #[test]
    fn test_ternary() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Ternary(
            Box::new(bool_expr(true)),
            Box::new(int(1)),
            Box::new(int(0)),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        // Should have JumpIfNot and Jump
        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::JumpIfNot(_))));
        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Jump(_))));
    }

    #[test]
    fn test_if_then_else() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::If(
            Box::new(bool_expr(true)),
            Box::new(int(1)),
            Some(Box::new(int(0))),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::JumpIfNot(_))));
    }

    #[test]
    fn test_if_without_else() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::If(
            Box::new(bool_expr(true)),
            Box::new(int(1)),
            None,  // no else
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        // Should produce Void for else branch
        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Const(Value::Void))));
    }

    #[test]
    fn test_match_single_arm() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Match(
            Box::new(int(1)),
            vec![MatchArm {
                pattern: Pattern::Wildcard(dummy_span()),
                guard: None,
                body: int(42),
                span: dummy_span(),
            }],
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        // Should store match value using StoreLocal
        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::StoreLocal(_))));
    }

    #[test]
    fn test_match_multiple_arms() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Match(
            Box::new(ident("x")),
            vec![
                MatchArm {
                    pattern: Pattern::Literal(int(1)),
                    guard: None,
                    body: string("one"),
                    span: dummy_span(),
                },
                MatchArm {
                    pattern: Pattern::Literal(int(2)),
                    guard: None,
                    body: string("two"),
                    span: dummy_span(),
                },
                MatchArm {
                    pattern: Pattern::Wildcard(dummy_span()),
                    guard: None,
                    body: string("other"),
                    span: dummy_span(),
                },
            ],
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        // Multiple JumpIfNot for each arm
        let jump_count = instrs.iter().filter(|i| matches!(i.opcode, OpCode::JumpIfNot(_))).count();
        assert!(jump_count >= 2);
    }

    #[test]
    fn test_match_with_guard() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Match(
            Box::new(ident("x")),
            vec![MatchArm {
                pattern: Pattern::Binding("n".to_string(), dummy_span()),
                guard: Some(binary(ident("n"), BinaryOp::Gt, int(0))),
                body: string("positive"),
                span: dummy_span(),
            }],
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        // Should have And for pattern + guard
        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::And)));
    }

    #[test]
    fn test_match_empty() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Match(
            Box::new(int(1)),
            vec![],  // empty arms
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        // Should return Void
        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Const(Value::Void))));
    }

    #[test]
    fn test_block() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Block(
            vec![int(1), int(2), int(3)],
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        // Should have Pop between expressions
        let pop_count = instrs.iter().filter(|i| matches!(i.opcode, OpCode::Pop)).count();
        assert_eq!(pop_count, 2);  // 2 pops for 3 expressions
    }

    // === Binding Tests ===

    #[test]
    fn test_let_binding() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Let(
            vec![("x".to_string(), int(42), false)],
            Box::new(ident("x")),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        // x is stored using StoreLocal (index 0) and loaded using LoadLocal
        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::StoreLocal(0))));
        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::LoadLocal(0))));
    }

    #[test]
    fn test_let_multiple_bindings() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Let(
            vec![
                ("x".to_string(), int(1), false),
                ("y".to_string(), int(2), false),
            ],
            Box::new(binary(ident("x"), BinaryOp::Add, ident("y"))),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        // x is at index 0, y is at index 1
        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::StoreLocal(0))));
        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::StoreLocal(1))));
    }

    // === Function Call Tests ===

    #[test]
    fn test_call_user_function() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Call(
            Box::new(ident("my_func")),
            vec![int(1), int(2)],
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::Call(name, 2) if name == "my_func")));
    }

    #[test]
    fn test_call_builtin_function() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Call(
            Box::new(ident("sqrt")),
            vec![int(4)],
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::CallBuiltin(name, 1) if name == "sqrt")));
    }

    #[test]
    fn test_self_call() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::SelfCall(
            vec![int(1), int(2)],
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::SelfCall(2))));
    }

    #[test]
    fn test_lambda() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Lambda(
            vec!["x".to_string()],
            Box::new(binary(ident("x"), BinaryOp::Mul, int(2))),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::MakeClosure(params, _) if params == &vec!["x".to_string()])));
    }

    // === Special Expression Tests ===

    #[test]
    fn test_range() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Range(
            Box::new(int(1)),
            Box::new(int(10)),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Range)));
    }

    #[test]
    fn test_contains() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Contains(
            Box::new(int(5)),
            Box::new(ident("arr")),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Contains)));
    }

    #[test]
    fn test_error_with_message() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Error(
            Some(Box::new(string("oops"))),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::Error(_))));
    }

    #[test]
    fn test_error_without_message() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Error(None, dummy_span());
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::Error(msg) if msg == "error")));
    }

    #[test]
    fn test_try() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Try(
            Box::new(ident("maybe_error")),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Try)));
    }

    #[test]
    fn test_coalesce() {
        let mut lowerer = Lowerer::new();
        let expr = Expr::Coalesce(
            Box::new(ident("maybe_nil")),
            Box::new(int(0)),
            dummy_span(),
        );
        let instrs = lowerer.lower_expr(&expr).unwrap();

        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Coalesce)));
    }

    // === Pattern Tests ===

    #[test]
    fn test_pattern_wildcard() {
        let mut lowerer = Lowerer::new();
        let instrs = lowerer.lower_pattern(&Pattern::Wildcard(dummy_span())).unwrap();

        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Pop)));
        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Const(Value::Bool(true)))));
    }

    #[test]
    fn test_pattern_literal() {
        let mut lowerer = Lowerer::new();
        let instrs = lowerer.lower_pattern(&Pattern::Literal(int(42))).unwrap();

        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Eq)));
    }

    #[test]
    fn test_pattern_binding() {
        let mut lowerer = Lowerer::new();
        let instrs = lowerer.lower_pattern(&Pattern::Binding("x".to_string(), dummy_span())).unwrap();

        assert!(instrs.iter().any(|i| matches!(i.opcode, OpCode::Dup)));
        // Binding pattern stores to local variable using StoreLocal
        assert!(instrs.iter().any(|i| matches!(&i.opcode, OpCode::StoreLocal(_))));
    }

    // === Function Definition Tests ===

    #[test]
    fn test_simple_function() {
        let func = make_func("add", vec!["a", "b"],
            binary(ident("a"), BinaryOp::Add, ident("b")));

        let program = Program {
            items: vec![Item::Function(func)],
            span: dummy_span(),
        };

        let mut lowerer = Lowerer::new();
        let result = lowerer.lower_program(&program);
        assert!(result.is_ok());

        let functions = result.unwrap();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "add");
        assert!(functions[0].instructions.iter().any(|i| matches!(i.opcode, OpCode::Return)));
    }

    #[test]
    fn test_self_recursion() {
        // fib(n) = n < 2 ? n : $(n-1) + $(n-2)
        let func = make_func("fib", vec!["n"],
            Expr::Ternary(
                Box::new(binary(ident("n"), BinaryOp::Lt, int(2))),
                Box::new(ident("n")),
                Box::new(binary(
                    Expr::SelfCall(vec![binary(ident("n"), BinaryOp::Sub, int(1))], dummy_span()),
                    BinaryOp::Add,
                    Expr::SelfCall(vec![binary(ident("n"), BinaryOp::Sub, int(2))], dummy_span()),
                )),
                dummy_span(),
            )
        );

        let program = Program {
            items: vec![Item::Function(func)],
            span: dummy_span(),
        };

        let mut lowerer = Lowerer::new();
        let result = lowerer.lower_program(&program);
        assert!(result.is_ok());

        let functions = result.unwrap();
        assert_eq!(functions[0].name, "fib");

        let has_self_call = functions[0]
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, OpCode::SelfCall(_)));
        assert!(has_self_call);
    }

    #[test]
    fn test_collection_ops() {
        // arr.@(_*2)
        let expr = Expr::MapOp(
            Box::new(ident("arr")),
            Box::new(binary(Expr::LambdaParam(dummy_span()), BinaryOp::Mul, int(2))),
            dummy_span(),
        );

        let program = Program {
            items: vec![Item::Expr(expr)],
            span: dummy_span(),
        };

        let mut lowerer = Lowerer::new();
        let result = lowerer.lower_program(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expression_as_main() {
        // 표현식만 있는 프로그램 -> __main__ 함수로 래핑
        let program = Program {
            items: vec![Item::Expr(binary(int(1), BinaryOp::Add, int(2)))],
            span: dummy_span(),
        };

        let mut lowerer = Lowerer::new();
        let result = lowerer.lower_program(&program).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "__main__");
    }

    // === FFI Tests ===

    #[test]
    fn test_ffi_registration() {
        use vais_ast::{FfiBlock, FfiFn, FfiType};

        let ffi_block = FfiBlock {
            lib_name: "libc".to_string(),
            abi: "C".to_string(),
            functions: vec![
                FfiFn {
                    name: "my_abs".to_string(),
                    extern_name: Some("abs".to_string()),
                    params: vec![("x".to_string(), FfiType::Int(32))],
                    return_type: FfiType::Int(32),
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };

        let program = Program {
            items: vec![Item::Ffi(ffi_block)],
            span: dummy_span(),
        };

        let mut lowerer = Lowerer::new();
        lowerer.lower_program(&program).unwrap();

        let ffi_funcs = lowerer.ffi_functions();
        assert!(ffi_funcs.contains_key("my_abs"));

        let info = ffi_funcs.get("my_abs").unwrap();
        assert_eq!(info.lib_name, "libc");
        assert_eq!(info.extern_name, "abs");
        assert_eq!(info.param_count, 1);
    }

    #[test]
    fn test_ffi_call() {
        use vais_ast::{FfiBlock, FfiFn, FfiType};

        let ffi_block = FfiBlock {
            lib_name: "libc".to_string(),
            abi: "C".to_string(),
            functions: vec![
                FfiFn {
                    name: "my_abs".to_string(),
                    extern_name: Some("abs".to_string()),
                    params: vec![("x".to_string(), FfiType::Int(32))],
                    return_type: FfiType::Int(32),
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };

        // FFI 호출 표현식
        let call_expr = Expr::Call(
            Box::new(ident("my_abs")),
            vec![int(-5)],
            dummy_span(),
        );

        let program = Program {
            items: vec![
                Item::Ffi(ffi_block),
                Item::Expr(call_expr),
            ],
            span: dummy_span(),
        };

        let mut lowerer = Lowerer::new();
        let funcs = lowerer.lower_program(&program).unwrap();

        // __main__ 함수에서 CallFfi가 사용되어야 함
        let main_func = &funcs[0];
        assert!(main_func.instructions.iter().any(|i| matches!(&i.opcode, OpCode::CallFfi(lib, name, 1) if lib == "libc" && name == "abs")));
    }

    // === Builtin Detection Tests ===

    #[test]
    fn test_is_builtin() {
        assert!(Lowerer::is_builtin("sqrt"));
        assert!(Lowerer::is_builtin("SQRT"));  // case insensitive
        assert!(Lowerer::is_builtin("print"));
        assert!(Lowerer::is_builtin("len"));
        assert!(!Lowerer::is_builtin("my_custom_func"));
    }

    // === Default Implementation ===

    #[test]
    fn test_lowerer_default() {
        let lowerer1 = Lowerer::new();
        let lowerer2 = Lowerer::default();

        assert!(lowerer1.ffi_functions().is_empty());
        assert!(lowerer2.ffi_functions().is_empty());
    }

    // === Pattern Matching Lowering Tests ===

    #[test]
    fn test_lower_match_literal_pattern() {
        // match x { 0 => "zero", 1 => "one", _ => "other" }
        let source = r#"classify(x) = match x { 0 => "zero", 1 => "one", _ => "other" }"#;
        let program = vais_parser::parse(source).unwrap();
        let functions = Lowerer::new().lower_program(&program).unwrap();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "classify");
        // Verify the function has instructions for the match expression
        assert!(!functions[0].instructions.is_empty());
    }

    #[test]
    fn test_lower_match_binding_pattern() {
        // match x { n => n * 2 }
        let source = r#"double(x) = match x { n => n * 2 }"#;
        let program = vais_parser::parse(source).unwrap();
        let functions = Lowerer::new().lower_program(&program).unwrap();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "double");
        // Should contain StoreLocal for binding 'n' and LoadLocal to use it
        let has_store = functions[0].instructions.iter().any(|i| {
            matches!(&i.opcode, OpCode::StoreLocal(_))
        });
        let has_load = functions[0].instructions.iter().any(|i| {
            matches!(&i.opcode, OpCode::LoadLocal(_))
        });
        assert!(has_store, "Should have StoreLocal for binding 'n'");
        assert!(has_load, "Should have LoadLocal to use 'n'");
    }

    #[test]
    fn test_lower_match_with_guard() {
        // match x { n if n > 0 => "positive", _ => "other" }
        let source = r#"sign(x) = match x { n if n > 0 => "positive", _ => "other" }"#;
        let program = vais_parser::parse(source).unwrap();
        let functions = Lowerer::new().lower_program(&program).unwrap();

        assert_eq!(functions.len(), 1);
        // Should have jump instructions for the if-else chain
        let has_jump = functions[0].instructions.iter().any(|i| {
            matches!(&i.opcode, OpCode::Jump(_) | OpCode::JumpIfNot(_))
        });
        assert!(has_jump, "Match should generate jump instructions");
    }

    #[test]
    fn test_lower_match_range_pattern() {
        // match x { 1..10 => "small", _ => "other" }
        let source = r#"size(x) = match x { 1..10 => "small", _ => "other" }"#;
        let program = vais_parser::parse(source).unwrap();
        let functions = Lowerer::new().lower_program(&program).unwrap();

        assert_eq!(functions.len(), 1);
        // Range pattern should generate >= and <= comparisons
        let has_gte = functions[0].instructions.iter().any(|i| {
            matches!(&i.opcode, OpCode::Gte)
        });
        let has_lte = functions[0].instructions.iter().any(|i| {
            matches!(&i.opcode, OpCode::Lte)
        });
        assert!(has_gte, "Range pattern should generate >= comparison");
        assert!(has_lte, "Range pattern should generate <= comparison");
    }

    #[test]
    fn test_lower_match_wildcard() {
        let source = r#"always_one(x) = match x { _ => 1 }"#;
        let program = vais_parser::parse(source).unwrap();
        let functions = Lowerer::new().lower_program(&program).unwrap();

        assert_eq!(functions.len(), 1);
        // Wildcard should always match (push true)
        let has_const_true = functions[0].instructions.iter().any(|i| {
            matches!(&i.opcode, OpCode::Const(Value::Bool(true)))
        });
        assert!(has_const_true, "Wildcard should push true");
    }

    #[test]
    fn test_lower_multiple_patterns() {
        // Test that multiple patterns generate correct if-else chain
        let source = r#"test(x) = match x { 0 => "a", 1 => "b", 2 => "c", _ => "d" }"#;
        let program = vais_parser::parse(source).unwrap();
        let functions = Lowerer::new().lower_program(&program).unwrap();

        assert_eq!(functions.len(), 1);
        // Count jump instructions - should have multiple for the chain
        let jump_count = functions[0].instructions.iter().filter(|i| {
            matches!(&i.opcode, OpCode::Jump(_) | OpCode::JumpIfNot(_))
        }).count();
        assert!(jump_count >= 3, "Should have multiple jumps for pattern chain");
    }
}
