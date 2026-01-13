//! Vais Web Playground
//!
//! WASM으로 컴파일되어 브라우저에서 Vais 코드를 실행

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

mod wasm_vm;

use wasm_vm::WasmVm;

/// 초기화 - 패닉 훅 설정
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// 실행 결과
#[derive(Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub execution_time_ms: f64,
}

/// Vais 코드 실행
#[wasm_bindgen]
pub fn execute(source: &str) -> String {
    let start = js_sys_now();

    let result = execute_internal(source);

    let elapsed = js_sys_now() - start;

    let exec_result = match result {
        Ok(output) => ExecutionResult {
            success: true,
            output,
            error: None,
            execution_time_ms: elapsed,
        },
        Err(err) => ExecutionResult {
            success: false,
            output: String::new(),
            error: Some(err),
            execution_time_ms: elapsed,
        },
    };

    serde_json::to_string(&exec_result).unwrap_or_else(|_| {
        r#"{"success":false,"output":"","error":"Serialization error"}"#.to_string()
    })
}

/// 내부 실행 함수
fn execute_internal(source: &str) -> Result<String, String> {
    // 1. 파싱
    let program = vais_parser::parse(source)
        .map_err(|e| format!("Parse error: {:?}", e))?;

    // 2. 타입 체크 (선택적)
    if let Err(e) = vais_typeck::check(&program) {
        // 타입 에러는 경고로만 표시
        // return Err(format!("Type error: {}", e));
        let _ = e; // 무시
    }

    // 3. IR로 변환
    let mut lowerer = vais_lowering::Lowerer::new();
    let functions = lowerer.lower_program(&program)
        .map_err(|e| format!("Lowering error: {:?}", e))?;

    // 4. WASM VM으로 실행
    let mut vm = WasmVm::new();
    vm.load_functions(functions);

    // __main__ 또는 첫 번째 함수 실행
    let target = if vm.has_function("__main__") {
        "__main__".to_string()
    } else if let Some(name) = vm.first_function_name() {
        name.to_string()
    } else {
        return Err("No function to execute".to_string());
    };

    let result = vm.call_function(&target, vec![])
        .map_err(|e| format!("Runtime error: {}", e))?;

    // 출력 수집
    let mut output = vm.get_output();
    if !output.is_empty() && !matches!(result, vais_ir::Value::Void) {
        output.push_str(&format!("\n=> {}", result));
    } else if output.is_empty() {
        output = format!("{}", result);
    }

    Ok(output)
}

/// 코드 검사 (파싱 + 타입체크)
#[wasm_bindgen]
pub fn check(source: &str) -> String {
    let result = check_internal(source);

    let check_result = match result {
        Ok(info) => serde_json::json!({
            "valid": true,
            "functions": info.functions,
            "errors": [],
        }),
        Err(errors) => serde_json::json!({
            "valid": false,
            "functions": [],
            "errors": errors,
        }),
    };

    serde_json::to_string(&check_result).unwrap_or_else(|_| {
        r#"{"valid":false,"errors":["Serialization error"]}"#.to_string()
    })
}

#[derive(Default)]
struct CheckInfo {
    functions: Vec<String>,
}

fn check_internal(source: &str) -> Result<CheckInfo, Vec<String>> {
    let mut errors = Vec::new();
    let mut info = CheckInfo::default();

    // 파싱
    let program = match vais_parser::parse(source) {
        Ok(p) => p,
        Err(e) => {
            errors.push(format!("Parse error: {:?}", e));
            return Err(errors);
        }
    };

    // 함수 목록 수집
    for item in &program.items {
        if let vais_ast::Item::Function(f) = item {
            info.functions.push(f.name.clone());
        }
    }

    // 타입 체크
    if let Err(e) = vais_typeck::check(&program) {
        errors.push(format!("Type warning: {}", e));
    }

    if errors.is_empty() {
        Ok(info)
    } else {
        // 경고만 있으면 성공으로 처리
        if errors.iter().all(|e| e.starts_with("Type warning")) {
            Ok(info)
        } else {
            Err(errors)
        }
    }
}

/// AST 출력 (디버깅용)
#[wasm_bindgen]
pub fn get_ast(source: &str) -> String {
    match vais_parser::parse(source) {
        Ok(program) => format!("{:#?}", program),
        Err(e) => format!("Parse error: {:?}", e),
    }
}

/// 토큰 목록 (디버깅용)
#[wasm_bindgen]
pub fn get_tokens(source: &str) -> String {
    match vais_lexer::tokenize(source) {
        Ok(tokens) => {
            let token_strs: Vec<String> = tokens.iter()
                .map(|t| format!("{:?}: {:?}", t.kind, t.text))
                .collect();
            token_strs.join("\n")
        }
        Err(e) => format!("Lex error: {:?}", e),
    }
}

/// 코드 포맷팅
#[wasm_bindgen]
pub fn format_code(source: &str) -> String {
    match vais_parser::parse(source) {
        Ok(program) => {
            // 간단한 포맷팅 (vais-tools 없이)
            format_program(&program)
        }
        Err(e) => format!("// Parse error: {:?}\n{}", e, source),
    }
}

/// 간단한 프로그램 포맷터
fn format_program(program: &vais_ast::Program) -> String {
    let mut output = String::new();

    for (i, item) in program.items.iter().enumerate() {
        if i > 0 {
            output.push_str("\n\n");
        }

        match item {
            vais_ast::Item::Function(f) => {
                if f.is_pub {
                    output.push_str("pub ");
                }
                output.push_str(&f.name);
                output.push('(');
                for (j, param) in f.params.iter().enumerate() {
                    if j > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&param.name);
                    if let Some(ty) = &param.ty {
                        output.push_str(": ");
                        output.push_str(&format_type(ty));
                    }
                }
                output.push_str(") = ");
                output.push_str(&format_expr(&f.body));
            }
            vais_ast::Item::Expr(e) => {
                output.push_str(&format_expr(e));
            }
            _ => {
                output.push_str("// unsupported item");
            }
        }
    }

    output
}

fn format_type(ty: &vais_ast::TypeExpr) -> String {
    match ty {
        vais_ast::TypeExpr::Simple(name) => name.clone(),
        vais_ast::TypeExpr::Array(inner) => format!("[{}]", format_type(inner)),
        vais_ast::TypeExpr::Optional(inner) => format!("?{}", format_type(inner)),
        _ => "Any".to_string(),
    }
}

fn format_expr(expr: &vais_ast::Expr) -> String {
    match expr {
        vais_ast::Expr::Integer(n, _) => n.to_string(),
        vais_ast::Expr::Float(f, _) => {
            let s = f.to_string();
            if s.contains('.') { s } else { format!("{}.0", s) }
        }
        vais_ast::Expr::String(s, _) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        vais_ast::Expr::Bool(b, _) => b.to_string(),
        vais_ast::Expr::Nil(_) => "nil".to_string(),
        vais_ast::Expr::Ident(name, _) => name.clone(),
        vais_ast::Expr::LambdaParam(_) => "_".to_string(),
        vais_ast::Expr::Array(elems, _) => {
            let parts: Vec<String> = elems.iter().map(format_expr).collect();
            format!("[{}]", parts.join(", "))
        }
        vais_ast::Expr::Binary(left, op, right, _) => {
            format!("{} {} {}", format_expr(left), format_binop(op), format_expr(right))
        }
        vais_ast::Expr::Unary(op, inner, _) => {
            format!("{}{}", format_unop(op), format_expr(inner))
        }
        vais_ast::Expr::Call(func, args, _) => {
            let arg_strs: Vec<String> = args.iter().map(format_expr).collect();
            format!("{}({})", format_expr(func), arg_strs.join(", "))
        }
        vais_ast::Expr::SelfCall(args, _) => {
            let arg_strs: Vec<String> = args.iter().map(format_expr).collect();
            format!("$({})", arg_strs.join(", "))
        }
        vais_ast::Expr::Ternary(cond, then_e, else_e, _) => {
            format!("{} ? {} : {}", format_expr(cond), format_expr(then_e), format_expr(else_e))
        }
        vais_ast::Expr::If(cond, then_e, else_e, _) => {
            let mut s = format!("if {} then {}", format_expr(cond), format_expr(then_e));
            if let Some(e) = else_e {
                s.push_str(&format!(" else {}", format_expr(e)));
            }
            s
        }
        vais_ast::Expr::MapOp(arr, transform, _) => {
            format!("{}.@({})", format_expr(arr), format_expr(transform))
        }
        vais_ast::Expr::FilterOp(arr, pred, _) => {
            format!("{}.?({})", format_expr(arr), format_expr(pred))
        }
        vais_ast::Expr::Range(start, end, _) => {
            format!("{}..{}", format_expr(start), format_expr(end))
        }
        _ => "/* expr */".to_string(),
    }
}

fn format_binop(op: &vais_ast::BinaryOp) -> &'static str {
    match op {
        vais_ast::BinaryOp::Add => "+",
        vais_ast::BinaryOp::Sub => "-",
        vais_ast::BinaryOp::Mul => "*",
        vais_ast::BinaryOp::Div => "/",
        vais_ast::BinaryOp::Mod => "%",
        vais_ast::BinaryOp::Eq => "==",
        vais_ast::BinaryOp::NotEq => "!=",
        vais_ast::BinaryOp::Lt => "<",
        vais_ast::BinaryOp::Gt => ">",
        vais_ast::BinaryOp::LtEq => "<=",
        vais_ast::BinaryOp::GtEq => ">=",
        vais_ast::BinaryOp::And => "&&",
        vais_ast::BinaryOp::Or => "||",
        vais_ast::BinaryOp::Concat => "++",
    }
}

fn format_unop(op: &vais_ast::UnaryOp) -> &'static str {
    match op {
        vais_ast::UnaryOp::Neg => "-",
        vais_ast::UnaryOp::Not => "!",
        vais_ast::UnaryOp::Len => "#",
    }
}

/// JavaScript의 Date.now() 대용
fn js_sys_now() -> f64 {
    // WASM에서는 performance.now() 사용
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0.0
    }
}

#[cfg(target_arch = "wasm32")]
mod js_sys {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        pub type Date;

        #[wasm_bindgen(static_method_of = Date)]
        pub fn now() -> f64;
    }
}
