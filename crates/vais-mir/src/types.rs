//! MIR type definitions: bodies, basic blocks, statements, terminators, operands.

use std::collections::HashMap;
use std::fmt;

/// A unique identifier for a local variable or temporary in MIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Local(pub u32);

impl fmt::Display for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "_{}", self.0)
    }
}

/// A unique identifier for a basic block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicBlockId(pub u32);

impl fmt::Display for BasicBlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

/// MIR type representation (simplified from ResolvedType).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MirType {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Str,
    Unit,
    Pointer(Box<MirType>),
    Ref(Box<MirType>),
    Array(Box<MirType>),
    Tuple(Vec<MirType>),
    Struct(String),
    Enum(String),
    Function {
        params: Vec<MirType>,
        ret: Box<MirType>,
    },
    Never,
}

/// A constant value in MIR.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Unit,
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Int(v) => write!(f, "{}", v),
            Constant::Float(v) => write!(f, "{}", v),
            Constant::Bool(v) => write!(f, "{}", v),
            Constant::Str(v) => write!(f, "\"{}\"", v),
            Constant::Unit => write!(f, "()"),
        }
    }
}

/// An operand in a MIR instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    /// Copy the value from a local (for Copy types).
    Copy(Place),
    /// Move the value from a local (transfers ownership).
    Move(Place),
    /// A constant value.
    Constant(Constant),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Copy(p) => write!(f, "copy {}", p),
            Operand::Move(p) => write!(f, "move {}", p),
            Operand::Constant(c) => write!(f, "const {}", c),
        }
    }
}

/// A place (lvalue) in MIR — a location that can be read from or written to.
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    pub local: Local,
    pub projections: Vec<Projection>,
}

impl Place {
    pub fn local(local: Local) -> Self {
        Self {
            local,
            projections: vec![],
        }
    }

    pub fn field(mut self, index: u32) -> Self {
        self.projections.push(Projection::Field(index));
        self
    }

    pub fn deref(mut self) -> Self {
        self.projections.push(Projection::Deref);
        self
    }

    pub fn index(mut self, local: Local) -> Self {
        self.projections.push(Projection::Index(local));
        self
    }
}

impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.local)?;
        for proj in &self.projections {
            match proj {
                Projection::Deref => write!(f, ".*")?,
                Projection::Field(i) => write!(f, ".{}", i)?,
                Projection::Index(l) => write!(f, "[{}]", l)?,
            }
        }
        Ok(())
    }
}

/// A projection step on a Place.
#[derive(Debug, Clone, PartialEq)]
pub enum Projection {
    Deref,
    Field(u32),
    Index(Local),
}

/// Binary operations in MIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "Add"),
            BinOp::Sub => write!(f, "Sub"),
            BinOp::Mul => write!(f, "Mul"),
            BinOp::Div => write!(f, "Div"),
            BinOp::Rem => write!(f, "Rem"),
            BinOp::BitAnd => write!(f, "BitAnd"),
            BinOp::BitOr => write!(f, "BitOr"),
            BinOp::BitXor => write!(f, "BitXor"),
            BinOp::Shl => write!(f, "Shl"),
            BinOp::Shr => write!(f, "Shr"),
            BinOp::Eq => write!(f, "Eq"),
            BinOp::Ne => write!(f, "Ne"),
            BinOp::Lt => write!(f, "Lt"),
            BinOp::Le => write!(f, "Le"),
            BinOp::Gt => write!(f, "Gt"),
            BinOp::Ge => write!(f, "Ge"),
        }
    }
}

/// Unary operations in MIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnOp::Neg => write!(f, "Neg"),
            UnOp::Not => write!(f, "Not"),
        }
    }
}

/// Right-hand side of an assignment (rvalue).
#[derive(Debug, Clone, PartialEq)]
pub enum Rvalue {
    /// Use an operand directly.
    Use(Operand),
    /// Binary operation.
    BinaryOp(BinOp, Operand, Operand),
    /// Unary operation.
    UnaryOp(UnOp, Operand),
    /// Create a reference to a place.
    Ref(Place),
    /// Aggregate construction (struct literal, tuple, array).
    Aggregate(AggregateKind, Vec<Operand>),
    /// Get the discriminant of an enum.
    Discriminant(Place),
    /// Cast between types.
    Cast(Operand, MirType),
    /// Get the length of an array/string.
    Len(Place),
}

/// Kind of aggregate value being constructed.
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateKind {
    Tuple,
    Array,
    Struct(String),
    Enum(String, u32), // enum name, variant index
}

/// A statement in a basic block (non-terminating).
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Assign an rvalue to a place.
    Assign(Place, Rvalue),
    /// Drop a value (run destructor and deallocate).
    Drop(Place),
    /// A no-op (used for source location tracking).
    Nop,
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Assign(place, rvalue) => write!(f, "{} = {:?}", place, rvalue),
            Statement::Drop(place) => write!(f, "drop({})", place),
            Statement::Nop => write!(f, "nop"),
        }
    }
}

/// A terminator ends a basic block with a control flow transfer.
#[derive(Debug, Clone, PartialEq)]
pub enum Terminator {
    /// Jump unconditionally to a block.
    Goto(BasicBlockId),
    /// Branch on a boolean condition.
    SwitchInt {
        discriminant: Operand,
        targets: Vec<(i64, BasicBlockId)>,
        otherwise: BasicBlockId,
    },
    /// Return from the function.
    Return,
    /// Call a function.
    Call {
        func: String,
        args: Vec<Operand>,
        destination: Place,
        target: BasicBlockId,
    },
    /// Unreachable code.
    Unreachable,
    /// Assert a condition, panic if false.
    Assert {
        cond: Operand,
        expected: bool,
        msg: String,
        target: BasicBlockId,
    },
}

impl fmt::Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terminator::Goto(bb) => write!(f, "goto -> {}", bb),
            Terminator::SwitchInt {
                discriminant,
                targets,
                otherwise,
            } => {
                write!(f, "switchInt({}) -> [", discriminant)?;
                for (val, bb) in targets {
                    write!(f, "{}: {}, ", val, bb)?;
                }
                write!(f, "otherwise: {}]", otherwise)
            }
            Terminator::Return => write!(f, "return"),
            Terminator::Call {
                func,
                args,
                destination,
                target,
            } => {
                write!(f, "{} = {}(", destination, func)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ") -> {}", target)
            }
            Terminator::Unreachable => write!(f, "unreachable"),
            Terminator::Assert {
                cond,
                expected,
                msg,
                target,
            } => write!(
                f,
                "assert({}, {}, \"{}\") -> {}",
                cond, expected, msg, target
            ),
        }
    }
}

/// A basic block: a sequence of statements followed by a terminator.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Option<Terminator>,
}

impl BasicBlock {
    pub fn new() -> Self {
        Self {
            statements: vec![],
            terminator: None,
        }
    }
}

impl Default for BasicBlock {
    fn default() -> Self {
        Self::new()
    }
}

/// Local variable declaration in a MIR body.
#[derive(Debug, Clone)]
pub struct LocalDecl {
    pub name: Option<String>,
    pub ty: MirType,
    pub is_mutable: bool,
}

/// A MIR function body.
#[derive(Debug, Clone)]
pub struct Body {
    pub name: String,
    pub params: Vec<MirType>,
    pub return_type: MirType,
    /// Local variable declarations. _0 is the return place.
    pub locals: Vec<LocalDecl>,
    /// Basic blocks forming the control flow graph.
    pub basic_blocks: Vec<BasicBlock>,
    /// Named block map for lookup.
    pub block_names: HashMap<String, BasicBlockId>,
}

impl Body {
    /// Pretty-print the MIR body.
    pub fn display(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("fn {}(", self.name));
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("_{}: {:?}", i + 1, param));
        }
        out.push_str(&format!(") -> {:?}", self.return_type));
        out.push_str(" {\n");

        // Locals
        for (i, local) in self.locals.iter().enumerate() {
            let mutability = if local.is_mutable { "mut " } else { "" };
            let name = local
                .name
                .as_deref()
                .map(|n| format!(" // {}", n))
                .unwrap_or_default();
            out.push_str(&format!(
                "    let {}{}: {:?};{}\n",
                mutability,
                Local(i as u32),
                local.ty,
                name
            ));
        }
        out.push('\n');

        // Basic blocks
        for (i, bb) in self.basic_blocks.iter().enumerate() {
            out.push_str(&format!("    {}: {{\n", BasicBlockId(i as u32)));
            for stmt in &bb.statements {
                out.push_str(&format!("        {};\n", stmt));
            }
            if let Some(ref term) = bb.terminator {
                out.push_str(&format!("        {};\n", term));
            }
            out.push_str("    }\n\n");
        }

        out.push_str("}\n");
        out
    }
}

/// A MIR module containing multiple function bodies.
#[derive(Debug, Clone)]
pub struct MirModule {
    pub name: String,
    pub bodies: Vec<Body>,
    pub structs: HashMap<String, Vec<(String, MirType)>>,
    pub enums: HashMap<String, Vec<(String, Vec<MirType>)>>,
}

impl MirModule {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bodies: vec![],
            structs: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    pub fn display(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("// MIR module: {}\n\n", self.name));

        for (name, fields) in &self.structs {
            out.push_str(&format!("struct {} {{\n", name));
            for (fname, ftype) in fields {
                out.push_str(&format!("    {}: {:?},\n", fname, ftype));
            }
            out.push_str("}\n\n");
        }

        for body in &self.bodies {
            out.push_str(&body.display());
            out.push('\n');
        }

        out
    }
}
