//! v6b AST to IR Lowering
//!
//! v6b AST를 IR 명령어로 변환

use aoel_ir::{Instruction, OpCode, ReduceOp as IrReduceOp, Value};
use aoel_v6b_ast::{
    BinaryOp, Expr, FunctionDef, IndexKind, Item, Pattern, Program, ReduceKind, UnaryOp,
};
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
}

/// v6b Lowerer
pub struct Lowerer {
    /// 현재 함수 이름 (재귀 호출용)
    current_function: Option<String>,
    /// 컴파일된 함수들
    functions: Vec<CompiledFunction>,
}

impl Lowerer {
    pub fn new() -> Self {
        Self {
            current_function: None,
            functions: Vec::new(),
        }
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
                    let instructions = self.lower_expr(e)?;
                    self.functions.push(CompiledFunction {
                        name: "__main__".to_string(),
                        params: Vec::new(),
                        instructions,
                    });
                }
                _ => {
                    // TypeDef, Module, Use는 런타임에 직접 영향 없음
                }
            }
        }

        Ok(std::mem::take(&mut self.functions))
    }

    /// 함수 정의를 컴파일
    fn lower_function(&mut self, func: &FunctionDef) -> LowerResult<CompiledFunction> {
        self.current_function = Some(func.name.clone());

        let params: Vec<String> = func.params.iter().map(|p| p.name.clone()).collect();

        let mut instructions = Vec::new();

        // 매개변수를 로컬 변수로 저장
        for param in &params {
            instructions.push(Instruction::new(OpCode::Store(param.clone())));
        }

        // 함수 본문 컴파일
        let body_instrs = self.lower_expr(&func.body)?;
        instructions.extend(body_instrs);

        // Return 추가
        instructions.push(Instruction::new(OpCode::Return));

        self.current_function = None;

        Ok(CompiledFunction {
            name: func.name.clone(),
            params,
            instructions,
        })
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
                instrs.push(Instruction::new(OpCode::Load(name.clone())));
            }

            Expr::LambdaParam(_) => {
                // 람다 내 _ 참조: 특수 변수 로드
                instrs.push(Instruction::new(OpCode::Load("_".to_string())));
            }

            // === Collections ===
            Expr::Array(elements, _) => {
                for elem in elements {
                    instrs.extend(self.lower_expr(elem)?);
                }
                instrs.push(Instruction::new(OpCode::MakeArray(elements.len())));
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

            // === v6b Collection Operations ===
            Expr::MapOp(arr, transform, _) => {
                instrs.extend(self.lower_expr(arr)?);
                let transform_instrs = self.lower_expr(transform)?;
                instrs.push(Instruction::new(OpCode::Map(Box::new(transform_instrs))));
            }

            Expr::FilterOp(arr, predicate, _) => {
                instrs.extend(self.lower_expr(arr)?);
                let pred_instrs = self.lower_expr(predicate)?;
                instrs.push(Instruction::new(OpCode::Filter(Box::new(pred_instrs))));
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
                instrs.extend(self.lower_expr(scrutinee)?);
                instrs.push(Instruction::new(OpCode::Store("__match_val__".to_string())));

                // 각 arm을 if-else 체인으로 변환
                let mut arm_instrs: Vec<Vec<Instruction>> = Vec::new();

                for arm in arms {
                    let mut arm_code = Vec::new();

                    // 패턴 매칭 조건 생성
                    arm_code.push(Instruction::new(OpCode::Load("__match_val__".to_string())));
                    arm_code.extend(self.lower_pattern(&arm.pattern)?);

                    // guard 조건
                    if let Some(guard) = &arm.guard {
                        arm_code.extend(self.lower_expr(guard)?);
                        arm_code.push(Instruction::new(OpCode::And));
                    }

                    // body
                    let body_instrs = self.lower_expr(&arm.body)?;
                    arm_code.extend(body_instrs);

                    arm_instrs.push(arm_code);
                }

                // 첫 번째 arm만 일단 실행 (간소화)
                if let Some(first) = arm_instrs.into_iter().next() {
                    instrs.extend(first);
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
                for (name, value) in bindings {
                    instrs.extend(self.lower_expr(value)?);
                    instrs.push(Instruction::new(OpCode::Store(name.clone())));
                }
                // 본문 실행
                instrs.extend(self.lower_expr(body)?);
            }

            // === Function ===
            Expr::Call(func, args, _) => {
                // 인자 먼저 푸시
                for arg in args {
                    instrs.extend(self.lower_expr(arg)?);
                }

                // 함수 호출
                if let Expr::Ident(name, _) = func.as_ref() {
                    instrs.push(Instruction::new(OpCode::Call(name.clone(), args.len())));
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
                // 람다는 클로저로 캡처 (간소화: 인라인 처리)
                // TODO: 실제 클로저 구현
                let body_instrs = self.lower_expr(body)?;
                instrs.push(Instruction::new(OpCode::Const(Value::String(format!(
                    "lambda[{}]",
                    params.join(",")
                )))));
                // 람다 본문은 호출 시점에 실행됨
                for i in body_instrs {
                    instrs.push(i);
                }
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
        }

        Ok(instrs)
    }

    /// 패턴을 IR로 변환 (매칭 조건 생성)
    fn lower_pattern(&mut self, pattern: &Pattern) -> LowerResult<Vec<Instruction>> {
        let mut instrs = Vec::new();

        match pattern {
            Pattern::Wildcard(_) => {
                // 항상 매칭
                instrs.push(Instruction::new(OpCode::Pop));
                instrs.push(Instruction::new(OpCode::Const(Value::Bool(true))));
            }

            Pattern::Literal(expr) => {
                // 값 비교
                instrs.extend(self.lower_expr(expr)?);
                instrs.push(Instruction::new(OpCode::Eq));
            }

            Pattern::Binding(name, _) => {
                // 값을 변수에 바인딩
                instrs.push(Instruction::new(OpCode::Dup));
                instrs.push(Instruction::new(OpCode::Store(name.clone())));
                instrs.push(Instruction::new(OpCode::Pop));
                instrs.push(Instruction::new(OpCode::Const(Value::Bool(true))));
            }

            _ => {
                // 복잡한 패턴은 나중에 구현
                instrs.push(Instruction::new(OpCode::Pop));
                instrs.push(Instruction::new(OpCode::Const(Value::Bool(true))));
            }
        }

        Ok(instrs)
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
    use aoel_v6b_ast::{FunctionDef, Param, Program};
    use aoel_v6b_lexer::Span;

    fn dummy_span() -> Span {
        Span::new(0, 0)
    }

    #[test]
    fn test_simple_function() {
        // add(a,b) = a + b
        let func = FunctionDef {
            name: "add".to_string(),
            params: vec![
                Param {
                    name: "a".to_string(),
                    ty: None,
                    default: None,
                    span: dummy_span(),
                },
                Param {
                    name: "b".to_string(),
                    ty: None,
                    default: None,
                    span: dummy_span(),
                },
            ],
            return_type: None,
            body: Expr::Binary(
                Box::new(Expr::Ident("a".to_string(), dummy_span())),
                BinaryOp::Add,
                Box::new(Expr::Ident("b".to_string(), dummy_span())),
                dummy_span(),
            ),
            is_pub: false,
            span: dummy_span(),
        };

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
    }

    #[test]
    fn test_self_recursion() {
        // fib(n) = n < 2 ? n : $(n-1) + $(n-2)
        let func = FunctionDef {
            name: "fib".to_string(),
            params: vec![Param {
                name: "n".to_string(),
                ty: None,
                default: None,
                span: dummy_span(),
            }],
            return_type: None,
            body: Expr::Ternary(
                Box::new(Expr::Binary(
                    Box::new(Expr::Ident("n".to_string(), dummy_span())),
                    BinaryOp::Lt,
                    Box::new(Expr::Integer(2, dummy_span())),
                    dummy_span(),
                )),
                Box::new(Expr::Ident("n".to_string(), dummy_span())),
                Box::new(Expr::Binary(
                    Box::new(Expr::SelfCall(
                        vec![Expr::Binary(
                            Box::new(Expr::Ident("n".to_string(), dummy_span())),
                            BinaryOp::Sub,
                            Box::new(Expr::Integer(1, dummy_span())),
                            dummy_span(),
                        )],
                        dummy_span(),
                    )),
                    BinaryOp::Add,
                    Box::new(Expr::SelfCall(
                        vec![Expr::Binary(
                            Box::new(Expr::Ident("n".to_string(), dummy_span())),
                            BinaryOp::Sub,
                            Box::new(Expr::Integer(2, dummy_span())),
                            dummy_span(),
                        )],
                        dummy_span(),
                    )),
                    dummy_span(),
                )),
                dummy_span(),
            ),
            is_pub: false,
            span: dummy_span(),
        };

        let program = Program {
            items: vec![Item::Function(func)],
            span: dummy_span(),
        };

        let mut lowerer = Lowerer::new();
        let result = lowerer.lower_program(&program);
        assert!(result.is_ok());

        let functions = result.unwrap();
        assert_eq!(functions[0].name, "fib");

        // SelfCall이 IR에 있는지 확인
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
            Box::new(Expr::Ident("arr".to_string(), dummy_span())),
            Box::new(Expr::Binary(
                Box::new(Expr::LambdaParam(dummy_span())),
                BinaryOp::Mul,
                Box::new(Expr::Integer(2, dummy_span())),
                dummy_span(),
            )),
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
}
