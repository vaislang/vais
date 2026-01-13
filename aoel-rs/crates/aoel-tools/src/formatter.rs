//! AOEL Code Formatter
//!
//! AST를 기반으로 일관된 코드 스타일로 포맷팅

use aoel_ast::*;

/// Formatter 설정
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// 들여쓰기 문자 (spaces or tab)
    pub indent: String,
    /// 들여쓰기 너비 (CLI에서 설정)
    pub indent_width: usize,
    /// 연산자 주위 공백
    pub space_around_operators: bool,
    /// 쉼표 뒤 공백
    pub space_after_comma: bool,
    /// 콜론 뒤 공백
    pub space_after_colon: bool,
    /// 최대 줄 길이 (0 = 무제한)
    pub max_line_length: usize,
    /// 최대 줄 너비 (CLI에서 설정, max_line_length와 동일)
    pub max_line_width: usize,
    /// 빈 줄로 함수 구분
    pub blank_lines_between_functions: usize,
    /// 배열 요소를 여러 줄로 분리하는 임계값
    pub array_wrap_threshold: usize,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent: "    ".to_string(), // 4 spaces
            indent_width: 4,
            space_around_operators: true,
            space_after_comma: true,
            space_after_colon: true,
            max_line_length: 100,
            max_line_width: 100,
            blank_lines_between_functions: 1,
            array_wrap_threshold: 5,
        }
    }
}

/// AOEL Code Formatter
pub struct Formatter {
    config: FormatConfig,
    indent_level: usize,
    output: String,
}

impl Formatter {
    pub fn new() -> Self {
        Self::with_config(FormatConfig::default())
    }

    pub fn with_config(config: FormatConfig) -> Self {
        Self {
            config,
            indent_level: 0,
            output: String::new(),
        }
    }

    /// 프로그램 전체 포맷
    pub fn format(&mut self, program: &Program) -> String {
        self.output.clear();
        self.indent_level = 0;

        let mut first = true;
        for item in &program.items {
            if !first {
                // 함수 사이 빈 줄
                if matches!(item, Item::Function(_)) {
                    for _ in 0..self.config.blank_lines_between_functions {
                        self.output.push('\n');
                    }
                }
            }
            first = false;
            self.format_item(item);
            self.output.push('\n');
        }

        self.output.trim_end().to_string() + "\n"
    }

    /// 소스 코드 문자열 포맷
    pub fn format_source(&mut self, source: &str) -> Result<String, String> {
        let program = aoel_parser::parse(source)
            .map_err(|e| format!("Parse error: {:?}", e))?;
        Ok(self.format(&program))
    }

    fn format_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => self.format_function(f),
            Item::TypeDef(t) => self.format_typedef(t),
            Item::Enum(e) => self.format_enum(e),
            Item::Module(m) => self.format_module(m),
            Item::Use(u) => self.format_use(u),
            Item::Ffi(f) => self.format_ffi(f),
            Item::Expr(e) => {
                self.write_indent();
                self.format_expr(e);
            }
        }
    }

    fn format_function(&mut self, func: &FunctionDef) {
        self.write_indent();

        // pub prefix
        if func.is_pub {
            self.output.push_str("pub ");
        }

        // name(params)
        self.output.push_str(&func.name);
        self.output.push('(');

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.output.push(',');
                if self.config.space_after_comma {
                    self.output.push(' ');
                }
            }
            self.output.push_str(&param.name);

            // 타입 어노테이션
            if let Some(ty) = &param.ty {
                self.output.push(':');
                if self.config.space_after_colon {
                    self.output.push(' ');
                }
                self.format_type(ty);
            }

            // 기본값
            if let Some(default) = &param.default {
                if self.config.space_around_operators {
                    self.output.push_str(" = ");
                } else {
                    self.output.push('=');
                }
                self.format_expr(default);
            }
        }

        self.output.push(')');

        // 반환 타입
        if let Some(ret_type) = &func.return_type {
            self.output.push_str(" -> ");
            self.format_type(ret_type);
        }

        // = body
        if self.config.space_around_operators {
            self.output.push_str(" = ");
        } else {
            self.output.push('=');
        }

        self.format_expr(&func.body);
    }

    fn format_typedef(&mut self, typedef: &TypeDef) {
        self.write_indent();
        if typedef.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str("type ");
        self.output.push_str(&typedef.name);
        if self.config.space_around_operators {
            self.output.push_str(" = ");
        } else {
            self.output.push('=');
        }
        self.format_type(&typedef.ty);
    }

    fn format_enum(&mut self, enum_def: &EnumDef) {
        self.write_indent();
        if enum_def.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str("enum ");
        self.output.push_str(&enum_def.name);

        // 타입 파라미터
        if !enum_def.type_params.is_empty() {
            self.output.push('<');
            for (i, param) in enum_def.type_params.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.output.push_str(&param.name);
            }
            self.output.push('>');
        }

        self.output.push_str(" {\n");
        self.indent_level += 1;

        for (i, variant) in enum_def.variants.iter().enumerate() {
            self.write_indent();
            self.output.push_str(&variant.name);
            if !variant.fields.is_empty() {
                self.output.push('(');
                for (j, field) in variant.fields.iter().enumerate() {
                    if j > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_type(field);
                }
                self.output.push(')');
            }
            if i < enum_def.variants.len() - 1 {
                self.output.push(',');
            }
            self.output.push('\n');
        }

        self.indent_level -= 1;
        self.write_indent();
        self.output.push('}');
    }

    fn format_module(&mut self, module: &ModuleDef) {
        self.write_indent();
        self.output.push_str("mod ");
        self.output.push_str(&module.name);
    }

    fn format_use(&mut self, use_def: &UseDef) {
        self.write_indent();
        self.output.push_str("use ");
        self.output.push_str(&use_def.path.join("."));

        if let Some(items) = &use_def.items {
            self.output.push_str(".{");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    self.output.push(',');
                    if self.config.space_after_comma {
                        self.output.push(' ');
                    }
                }
                self.output.push_str(item);
            }
            self.output.push('}');
        }

        if let Some(alias) = &use_def.alias {
            self.output.push_str(" as ");
            self.output.push_str(alias);
        }
    }

    fn format_ffi(&mut self, ffi: &FfiBlock) {
        self.write_indent();
        self.output.push_str("ffi \"");
        self.output.push_str(&ffi.lib_name);
        self.output.push_str("\" {\n");

        self.indent_level += 1;
        for func in &ffi.functions {
            self.write_indent();
            self.output.push_str("fn ");
            self.output.push_str(&func.name);

            if let Some(ext_name) = &func.extern_name {
                if self.config.space_around_operators {
                    self.output.push_str(" = ");
                } else {
                    self.output.push('=');
                }
                self.output.push('"');
                self.output.push_str(ext_name);
                self.output.push('"');
            }

            self.output.push('(');
            for (i, (name, ty)) in func.params.iter().enumerate() {
                if i > 0 {
                    self.output.push(',');
                    if self.config.space_after_comma {
                        self.output.push(' ');
                    }
                }
                self.output.push_str(name);
                self.output.push(':');
                if self.config.space_after_colon {
                    self.output.push(' ');
                }
                self.format_ffi_type(ty);
            }
            self.output.push(')');

            if func.return_type != FfiType::Void {
                self.output.push_str(" -> ");
                self.format_ffi_type(&func.return_type);
            }
            self.output.push('\n');
        }
        self.indent_level -= 1;

        self.write_indent();
        self.output.push('}');
    }

    fn format_ffi_type(&mut self, ty: &FfiType) {
        match ty {
            FfiType::Void => self.output.push_str("void"),
            FfiType::Int(8) => self.output.push_str("i8"),
            FfiType::Int(16) => self.output.push_str("i16"),
            FfiType::Int(32) => self.output.push_str("i32"),
            FfiType::Int(64) => self.output.push_str("i64"),
            FfiType::Int(n) => {
                self.output.push('i');
                self.output.push_str(&n.to_string());
            }
            FfiType::Uint(8) => self.output.push_str("u8"),
            FfiType::Uint(16) => self.output.push_str("u16"),
            FfiType::Uint(32) => self.output.push_str("u32"),
            FfiType::Uint(64) => self.output.push_str("u64"),
            FfiType::Uint(n) => {
                self.output.push('u');
                self.output.push_str(&n.to_string());
            }
            FfiType::F32 => self.output.push_str("f32"),
            FfiType::F64 => self.output.push_str("f64"),
            FfiType::Bool => self.output.push_str("bool"),
            FfiType::CStr => self.output.push_str("cstr"),
            FfiType::Opaque => self.output.push_str("ptr"),
            FfiType::Ptr(inner) => {
                self.output.push('*');
                self.format_ffi_type(inner);
            }
            FfiType::MutPtr(inner) => {
                self.output.push_str("*mut ");
                self.format_ffi_type(inner);
            }
        }
    }

    fn format_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Integer(n, _) => {
                self.output.push_str(&n.to_string());
            }
            Expr::Float(f, _) => {
                let s = format!("{}", f);
                self.output.push_str(&s);
                // 정수처럼 보이면 .0 추가
                if !s.contains('.') && !s.contains('e') {
                    self.output.push_str(".0");
                }
            }
            Expr::String(s, _) => {
                self.output.push('"');
                self.output.push_str(&escape_string(s));
                self.output.push('"');
            }
            Expr::Bool(b, _) => {
                self.output.push_str(if *b { "true" } else { "false" });
            }
            Expr::Nil(_) => {
                self.output.push_str("nil");
            }
            Expr::Ident(name, _) => {
                self.output.push_str(name);
            }
            Expr::LambdaParam(_) => {
                self.output.push('_');
            }
            Expr::Array(elements, _) => {
                self.output.push('[');
                let wrap = elements.len() > self.config.array_wrap_threshold;
                if wrap {
                    self.output.push('\n');
                    self.indent_level += 1;
                }
                for (i, elem) in elements.iter().enumerate() {
                    if wrap {
                        self.write_indent();
                    }
                    if i > 0 && !wrap {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.format_expr(elem);
                    if wrap {
                        self.output.push(',');
                        self.output.push('\n');
                    }
                }
                if wrap {
                    self.indent_level -= 1;
                    self.write_indent();
                }
                self.output.push(']');
            }
            Expr::Set(elements, _) => {
                self.output.push_str("#{");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expr(elem);
                }
                self.output.push('}');
            }
            Expr::Map(fields, _) => {
                self.output.push('{');
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.output.push_str(key);
                    self.output.push(':');
                    if self.config.space_after_colon {
                        self.output.push(' ');
                    }
                    self.format_expr(value);
                }
                self.output.push('}');
            }
            Expr::Tuple(elements, _) => {
                self.output.push('(');
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.format_expr(elem);
                }
                if elements.len() == 1 {
                    self.output.push(',');
                }
                self.output.push(')');
            }
            Expr::Binary(left, op, right, _) => {
                self.format_expr(left);
                if self.config.space_around_operators {
                    self.output.push(' ');
                }
                self.output.push_str(binary_op_str(op));
                if self.config.space_around_operators {
                    self.output.push(' ');
                }
                self.format_expr(right);
            }
            Expr::Unary(op, inner, _) => {
                self.output.push_str(unary_op_str(op));
                self.format_expr(inner);
            }
            Expr::Ternary(cond, then_expr, else_expr, _) => {
                self.format_expr(cond);
                self.output.push_str(" ? ");
                self.format_expr(then_expr);
                self.output.push_str(" : ");
                self.format_expr(else_expr);
            }
            Expr::If(cond, then_expr, else_expr, _) => {
                self.output.push_str("if ");
                self.format_expr(cond);
                self.output.push_str(" then ");
                self.format_expr(then_expr);
                if let Some(else_e) = else_expr {
                    self.output.push_str(" else ");
                    self.format_expr(else_e);
                }
            }
            Expr::Match(scrutinee, arms, _) => {
                self.output.push_str("match ");
                self.format_expr(scrutinee);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for arm in arms {
                    self.write_indent();
                    self.format_pattern(&arm.pattern);
                    if let Some(guard) = &arm.guard {
                        self.output.push_str(" if ");
                        self.format_expr(guard);
                    }
                    self.output.push_str(" => ");
                    self.format_expr(&arm.body);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');
            }
            Expr::Block(exprs, _) => {
                if exprs.len() == 1 {
                    self.format_expr(&exprs[0]);
                } else {
                    self.output.push_str("{\n");
                    self.indent_level += 1;
                    for expr in exprs {
                        self.write_indent();
                        self.format_expr(expr);
                        self.output.push('\n');
                    }
                    self.indent_level -= 1;
                    self.write_indent();
                    self.output.push('}');
                }
            }
            Expr::Let(bindings, body, _) => {
                self.output.push_str("let ");
                for (i, (name, value)) in bindings.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.output.push_str(name);
                    if self.config.space_around_operators {
                        self.output.push_str(" = ");
                    } else {
                        self.output.push('=');
                    }
                    self.format_expr(value);
                }
                self.output.push_str(" in ");
                self.format_expr(body);
            }
            Expr::Call(func, args, _) => {
                self.format_expr(func);
                self.output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.format_expr(arg);
                }
                self.output.push(')');
            }
            Expr::SelfCall(args, _) => {
                self.output.push_str("$(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.format_expr(arg);
                }
                self.output.push(')');
            }
            Expr::Lambda(params, body, _) => {
                self.output.push('|');
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.output.push_str(param);
                }
                self.output.push_str("| ");
                self.format_expr(body);
            }
            Expr::Field(obj, field, _) => {
                self.format_expr(obj);
                self.output.push('.');
                self.output.push_str(field);
            }
            Expr::Index(arr, kind, _) => {
                self.format_expr(arr);
                self.output.push('[');
                match kind.as_ref() {
                    IndexKind::Single(idx) => {
                        self.format_expr(idx);
                    }
                    IndexKind::Slice(start, end) => {
                        if let Some(s) = start {
                            self.format_expr(s);
                        }
                        self.output.push(':');
                        if let Some(e) = end {
                            self.format_expr(e);
                        }
                    }
                }
                self.output.push(']');
            }
            Expr::MethodCall(obj, method, args, _) => {
                self.format_expr(obj);
                self.output.push('.');
                self.output.push_str(method);
                self.output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.format_expr(arg);
                }
                self.output.push(')');
            }
            Expr::MapOp(arr, transform, _) => {
                self.format_expr(arr);
                self.output.push_str(".@(");
                self.format_expr(transform);
                self.output.push(')');
            }
            Expr::FilterOp(arr, predicate, _) => {
                self.format_expr(arr);
                self.output.push_str(".?(");
                self.format_expr(predicate);
                self.output.push(')');
            }
            Expr::ReduceOp(arr, kind, _) => {
                self.format_expr(arr);
                self.output.push_str("./");
                match kind {
                    ReduceKind::Sum => self.output.push('+'),
                    ReduceKind::Product => self.output.push('*'),
                    ReduceKind::Min => self.output.push_str("min"),
                    ReduceKind::Max => self.output.push_str("max"),
                    ReduceKind::And => self.output.push_str("and"),
                    ReduceKind::Or => self.output.push_str("or"),
                    ReduceKind::Custom(init, func) => {
                        self.output.push('(');
                        self.format_expr(init);
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                        self.format_expr(func);
                        self.output.push(')');
                    }
                }
            }
            Expr::Range(start, end, _) => {
                self.format_expr(start);
                self.output.push_str("..");
                self.format_expr(end);
            }
            Expr::Contains(elem, arr, _) => {
                self.format_expr(elem);
                self.output.push_str(" @ ");
                self.format_expr(arr);
            }
            Expr::Try(inner, _) => {
                self.format_expr(inner);
                self.output.push('?');
            }
            Expr::Coalesce(value, default, _) => {
                self.format_expr(value);
                self.output.push_str(" ?? ");
                self.format_expr(default);
            }
            Expr::Error(msg, _) => {
                self.output.push_str("error");
                if let Some(m) = msg {
                    self.output.push('(');
                    self.format_expr(m);
                    self.output.push(')');
                }
            }
            Expr::TryCatch { body, error_name, handler, .. } => {
                self.output.push_str("try {\n");
                self.indent_level += 1;
                self.write_indent();
                self.format_expr(body);
                self.output.push('\n');
                self.indent_level -= 1;
                self.write_indent();
                self.output.push_str("} catch ");
                self.output.push_str(error_name);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                self.write_indent();
                self.format_expr(handler);
                self.output.push('\n');
                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');
            }
            Expr::Struct(type_name, fields, _) => {
                self.output.push_str(type_name);
                self.output.push_str(" { ");
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(name);
                    self.output.push_str(": ");
                    self.format_expr(value);
                }
                self.output.push_str(" }");
            }
            Expr::ListComprehension { expr, var, iter, cond, .. } => {
                self.output.push('[');
                self.format_expr(expr);
                self.output.push_str(" for ");
                self.output.push_str(var);
                self.output.push_str(" in ");
                self.format_expr(iter);
                if let Some(condition) = cond {
                    self.output.push_str(" if ");
                    self.format_expr(condition);
                }
                self.output.push(']');
            }
            Expr::SetComprehension { expr, var, iter, cond, .. } => {
                self.output.push_str("#{");
                self.format_expr(expr);
                self.output.push_str(" for ");
                self.output.push_str(var);
                self.output.push_str(" in ");
                self.format_expr(iter);
                if let Some(condition) = cond {
                    self.output.push_str(" if ");
                    self.format_expr(condition);
                }
                self.output.push('}');
            }
        }
    }

    fn format_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Wildcard(_) => {
                self.output.push('_');
            }
            Pattern::Literal(expr) => {
                self.format_expr(expr);
            }
            Pattern::Binding(name, _) => {
                self.output.push_str(name);
            }
            Pattern::Tuple(patterns, _) => {
                self.output.push('(');
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.format_pattern(p);
                }
                self.output.push(')');
            }
            Pattern::Array(patterns, _) => {
                self.output.push('[');
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.format_pattern(p);
                }
                self.output.push(']');
            }
            Pattern::Struct(fields, _) => {
                self.output.push('{');
                for (i, (field, pat)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.output.push_str(field);
                    if let Some(p) = pat {
                        self.output.push(':');
                        if self.config.space_after_colon {
                            self.output.push(' ');
                        }
                        self.format_pattern(p);
                    }
                }
                self.output.push('}');
            }
            Pattern::Variant(name, inner, _) => {
                self.output.push_str(name);
                if let Some(p) = inner {
                    self.output.push('(');
                    self.format_pattern(p);
                    self.output.push(')');
                }
            }
            Pattern::Range(start, end, _) => {
                self.format_expr(start);
                self.output.push_str("..");
                self.format_expr(end);
            }
            Pattern::Or(patterns, _) => {
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" | ");
                    }
                    self.format_pattern(p);
                }
            }
        }
    }

    fn format_type(&mut self, ty: &TypeExpr) {
        match ty {
            TypeExpr::Simple(name) => self.output.push_str(name),
            TypeExpr::Array(inner) => {
                self.output.push('[');
                self.format_type(inner);
                self.output.push(']');
            }
            TypeExpr::Set(inner) => {
                self.output.push_str("#{");
                self.format_type(inner);
                self.output.push('}');
            }
            TypeExpr::Map(key, value) => {
                self.output.push('{');
                self.format_type(key);
                self.output.push(':');
                if self.config.space_after_colon {
                    self.output.push(' ');
                }
                self.format_type(value);
                self.output.push('}');
            }
            TypeExpr::Tuple(types) => {
                self.output.push('(');
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.format_type(t);
                }
                self.output.push(')');
            }
            TypeExpr::Optional(inner) => {
                self.output.push('?');
                self.format_type(inner);
            }
            TypeExpr::Result(inner) => {
                self.output.push('!');
                self.format_type(inner);
            }
            TypeExpr::Function(params, ret) => {
                self.output.push('(');
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.format_type(p);
                }
                self.output.push_str(") -> ");
                self.format_type(ret);
            }
            TypeExpr::Struct(fields) => {
                self.output.push('{');
                for (i, (name, ty)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.output.push_str(name);
                    self.output.push(':');
                    if self.config.space_after_colon {
                        self.output.push(' ');
                    }
                    self.format_type(ty);
                }
                self.output.push('}');
            }
            TypeExpr::TypeVar(name) => {
                self.output.push_str(name);
            }
            TypeExpr::Generic(name, args) => {
                self.output.push_str(name);
                self.output.push('<');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push(',');
                        if self.config.space_after_comma {
                            self.output.push(' ');
                        }
                    }
                    self.format_type(arg);
                }
                self.output.push('>');
            }
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str(&self.config.indent);
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

fn binary_op_str(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::Eq => "==",
        BinaryOp::NotEq => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Gt => ">",
        BinaryOp::LtEq => "<=",
        BinaryOp::GtEq => ">=",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
        BinaryOp::Concat => "++",
    }
}

fn unary_op_str(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "!",
        UnaryOp::Len => "#",
    }
}

fn escape_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_function() {
        let source = "add(a,b)=a+b";
        let mut formatter = Formatter::new();
        let result = formatter.format_source(source).unwrap();
        assert!(result.contains("add(a, b) = a + b"));
    }

    #[test]
    fn test_format_array() {
        let source = "[1,2,3]";
        let mut formatter = Formatter::new();
        let result = formatter.format_source(source).unwrap();
        assert!(result.contains("[1, 2, 3]"));
    }

    #[test]
    fn test_format_ffi() {
        let source = r#"ffi "c" { fn abs(n:i32)->i32 }"#;
        let mut formatter = Formatter::new();
        let result = formatter.format_source(source).unwrap();
        assert!(result.contains("fn abs(n: i32) -> i32"));
    }
}
