#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;

/// Structured input for more intelligent fuzzing
#[derive(Debug, Arbitrary)]
struct VaisProgram {
    items: Vec<VaisItem>,
}

#[derive(Debug, Arbitrary)]
enum VaisItem {
    Function(FunctionDef),
    Struct(StructDef),
    Enum(EnumDef),
}

#[derive(Debug, Arbitrary)]
struct FunctionDef {
    name: SmallString,
    params: Vec<(SmallString, VaisType)>,
    ret_type: VaisType,
    body: VaisExpr,
}

#[derive(Debug, Arbitrary)]
struct StructDef {
    name: SmallString,
    fields: Vec<(SmallString, VaisType)>,
}

#[derive(Debug, Arbitrary)]
struct EnumDef {
    name: SmallString,
    variants: Vec<SmallString>,
}

#[derive(Debug, Arbitrary)]
enum VaisType {
    I64,
    F64,
    Bool,
    Str,
    Unit,
    Array(Box<VaisType>),
    Option(Box<VaisType>),
}

#[derive(Debug, Arbitrary)]
enum VaisExpr {
    Literal(i32),
    BoolLit(bool),
    Var(SmallString),
    BinOp(Box<VaisExpr>, BinOp, Box<VaisExpr>),
    If(Box<VaisExpr>, Box<VaisExpr>, Box<VaisExpr>),
    Block(Vec<VaisStmt>, Box<VaisExpr>),
}

#[derive(Debug, Arbitrary)]
enum VaisStmt {
    Let(SmallString, VaisExpr),
}

#[derive(Debug, Arbitrary)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Gt,
    Eq,
    And,
    Or,
}

/// Small string to avoid huge allocations
#[derive(Debug)]
struct SmallString(String);

impl<'a> Arbitrary<'a> for SmallString {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(1..=15)?;
        let first = u.int_in_range(b'a'..=b'z')? as char;
        let rest: String = (0..len - 1)
            .map(|_| {
                let c = u.int_in_range(b'a'..=b'z').unwrap_or(b'a');
                c as char
            })
            .collect();
        Ok(SmallString(format!("{}{}", first, rest)))
    }
}

impl VaisProgram {
    fn to_source(&self) -> String {
        let mut source = String::new();
        for item in &self.items {
            source.push_str(&item.to_source());
            source.push('\n');
        }
        source
    }
}

impl VaisItem {
    fn to_source(&self) -> String {
        match self {
            VaisItem::Function(f) => f.to_source(),
            VaisItem::Struct(s) => s.to_source(),
            VaisItem::Enum(e) => e.to_source(),
        }
    }
}

impl FunctionDef {
    fn to_source(&self) -> String {
        let params: Vec<String> = self
            .params
            .iter()
            .map(|(name, ty)| format!("{}: {}", name.0, ty.to_source()))
            .collect();
        format!(
            "F {}({}) -> {} {{ {} }}",
            self.name.0,
            params.join(", "),
            self.ret_type.to_source(),
            self.body.to_source()
        )
    }
}

impl StructDef {
    fn to_source(&self) -> String {
        let fields: Vec<String> = self
            .fields
            .iter()
            .map(|(name, ty)| format!("{}: {}", name.0, ty.to_source()))
            .collect();
        format!("S {} {{ {} }}", self.name.0, fields.join(", "))
    }
}

impl EnumDef {
    fn to_source(&self) -> String {
        if self.variants.is_empty() {
            format!("E {} {{ Empty }}", self.name.0)
        } else {
            let variants: Vec<&str> = self.variants.iter().map(|v| v.0.as_str()).collect();
            format!("E {} {{ {} }}", self.name.0, variants.join(", "))
        }
    }
}

impl VaisType {
    fn to_source(&self) -> String {
        match self {
            VaisType::I64 => "i64".to_string(),
            VaisType::F64 => "f64".to_string(),
            VaisType::Bool => "bool".to_string(),
            VaisType::Str => "str".to_string(),
            VaisType::Unit => "()".to_string(),
            VaisType::Array(inner) => format!("[{}]", inner.to_source()),
            VaisType::Option(inner) => format!("Option<{}>", inner.to_source()),
        }
    }
}

impl VaisExpr {
    fn to_source(&self) -> String {
        match self {
            VaisExpr::Literal(n) => n.to_string(),
            VaisExpr::BoolLit(b) => b.to_string(),
            VaisExpr::Var(s) => s.0.clone(),
            VaisExpr::BinOp(l, op, r) => {
                format!("({} {} {})", l.to_source(), op.to_source(), r.to_source())
            }
            VaisExpr::If(cond, then, else_) => {
                format!(
                    "I {} {{ {} }} E {{ {} }}",
                    cond.to_source(),
                    then.to_source(),
                    else_.to_source()
                )
            }
            VaisExpr::Block(stmts, expr) => {
                let stmts_str: Vec<String> = stmts.iter().map(|s| s.to_source()).collect();
                if stmts_str.is_empty() {
                    expr.to_source()
                } else {
                    format!("{}\n{}", stmts_str.join("\n"), expr.to_source())
                }
            }
        }
    }
}

impl VaisStmt {
    fn to_source(&self) -> String {
        match self {
            VaisStmt::Let(name, expr) => format!("{} := {}", name.0, expr.to_source()),
        }
    }
}

impl BinOp {
    fn to_source(&self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::Eq => "==",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }
}

fuzz_target!(|program: VaisProgram| {
    let source = program.to_source();

    if source.len() > 50_000 {
        return;
    }

    use vais_codegen::CodeGenerator;
    use vais_parser::parse;
    use vais_types::TypeChecker;

    if let Ok(module) = parse(&source) {
        let mut checker = TypeChecker::new();
        if checker.check_module(&module).is_ok() {
            let mut gen = CodeGenerator::new("fuzz_test");
            let _ = gen.generate_module(&module);
        }
    }
});
