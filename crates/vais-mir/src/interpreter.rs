//! Small reference interpreter for certified MIR subsets.
//!
//! This is intentionally not a general VM. It executes the Core subset needed
//! by strict MIR certification so MIR can become an independent semantic check
//! before LLVM output is trusted.

use crate::types::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

const DEFAULT_STEP_LIMIT: usize = 10_000;

/// Output captured by an interpreter run that opted into the stdout sink.
///
/// Step 17 stage 5a (Master Plan v17 §I-6): Path A of the MIR-vs-native
/// differential oracle needs `(exit_code, stdout)` instead of the bare
/// `MirValue` produced by [`interpret_function`]. This struct is the
/// uniform exchange shape that both the MIR interpreter side (here) and
/// the JIT/native side (vais-jit, future stage) lower into for diff.
#[derive(Debug, Clone, PartialEq)]
pub struct InterpreterRunOutput {
    /// Process-style exit code (8-bit truncated to match POSIX semantics).
    /// Mapped from the function return value: `MirValue::Int(n)` → `n & 0xFF`,
    /// `MirValue::Unit` → `0`, anything else → caller decides (we still
    /// return the raw value separately so the diff layer can short-circuit).
    pub exit_code: i64,
    /// stdout captured during the run via the builtin print intercept.
    pub stdout: String,
    /// The raw return value, for callers that want richer comparison than
    /// just exit code (e.g. `MirValue::Str` etc.). The diff oracle ignores
    /// this and uses `exit_code` + `stdout` only.
    pub return_value: MirValue,
}

/// Runtime value produced by the MIR interpreter.
#[derive(Debug, Clone, PartialEq)]
pub enum MirValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Struct(String, Vec<MirValue>),
    Enum(String, u32, Vec<MirValue>),
    Vec(Vec<MirValue>),
    Unit,
}

impl From<&Constant> for MirValue {
    fn from(value: &Constant) -> Self {
        match value {
            Constant::Int(v) => MirValue::Int(*v),
            Constant::Float(v) => MirValue::Float(*v),
            Constant::Bool(v) => MirValue::Bool(*v),
            Constant::Str(v) => MirValue::Str(v.clone()),
            Constant::Unit => MirValue::Unit,
        }
    }
}

/// Interpreter failure for unsupported MIR or invalid execution state.
#[derive(Debug, Clone, PartialEq)]
pub struct MirInterpretError {
    pub function: String,
    pub message: String,
}

impl MirInterpretError {
    fn new(function: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            function: function.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for MirInterpretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.function, self.message)
    }
}

impl std::error::Error for MirInterpretError {}

/// Interpret a named function in a MIR module.
pub fn interpret_function(
    module: &MirModule,
    function: &str,
    args: Vec<MirValue>,
) -> Result<MirValue, MirInterpretError> {
    Interpreter::new(module).call(function, args)
}

/// Step 17 stage 5a: interpret a named function with a stdout sink active.
///
/// Same as [`interpret_function`] but routes calls to recognized print-style
/// builtins (`print`, `println`, `print_int`, `print_str`) into a
/// per-run String buffer instead of erroring out with "function body not
/// found". Returns an [`InterpreterRunOutput`] with `exit_code` derived
/// from the return value, the captured `stdout`, and the raw `return_value`.
///
/// Backward compatibility: existing callers of [`interpret_function`] are
/// unaffected — the bare-call path treats unknown function names as errors,
/// just like before.
pub fn interpret_function_with_io(
    module: &MirModule,
    function: &str,
    args: Vec<MirValue>,
) -> Result<InterpreterRunOutput, MirInterpretError> {
    let mut interp = Interpreter::new(module);
    interp.stdout_sink = Some(RefCell::new(String::new()));
    let return_value = interp.call(function, args)?;
    let stdout = interp
        .stdout_sink
        .as_ref()
        .map(|cell| cell.borrow().clone())
        .unwrap_or_default();
    let exit_code = match &return_value {
        MirValue::Int(n) => n & 0xFF,
        MirValue::Unit => 0,
        _ => 0, // non-int/unit: leave to caller; diff oracle short-circuits anyway
    };
    Ok(InterpreterRunOutput {
        exit_code,
        stdout,
        return_value,
    })
}

struct Interpreter<'a> {
    bodies: HashMap<&'a str, &'a Body>,
    step_limit: usize,
    /// Step 17 stage 5a: when `Some`, calls to known print-style builtins
    /// are intercepted and pushed into this buffer (rather than failing
    /// with "function body not found"). The bare [`interpret_function`]
    /// entry leaves this `None` so its behavior is unchanged.
    stdout_sink: Option<RefCell<String>>,
}

impl<'a> Interpreter<'a> {
    fn new(module: &'a MirModule) -> Self {
        Self {
            bodies: module
                .bodies
                .iter()
                .map(|body| (body.name.as_str(), body))
                .collect(),
            step_limit: DEFAULT_STEP_LIMIT,
            stdout_sink: None,
        }
    }

    /// If `function` is a recognized print-style builtin AND a stdout sink
    /// is active, push the formatted args into the sink and return
    /// `Some(MirValue::Unit)`. Returns `None` if the call should not be
    /// intercepted.
    ///
    /// Recognized builtins (Step 17 stage 5a + B.5 expansion):
    ///
    /// | builtin       | behavior                                       | newline |
    /// |---------------|------------------------------------------------|---------|
    /// | print         | each arg formatted via write_value             | no      |
    /// | print_str     | alias of print (legacy std API)                | no      |
    /// | print_int     | first arg as i64 only                          | no      |
    /// | print_float   | first arg as f64 only                          | no      |
    /// | print_bool    | first arg as bool only ("true"/"false")        | no      |
    /// | println       | each arg formatted via write_value             | yes     |
    /// | eprint        | same as print but reserved for stderr later    | no      |
    /// | eprintln      | same as println but reserved for stderr later  | yes     |
    ///
    /// Note: eprint/eprintln currently route to the same sink as print —
    /// stage 5a does not split stdout vs stderr. A future stage may add a
    /// separate stderr_sink; until then differential oracle treats both as
    /// "captured output". JIT-side stage 5b will mirror the same model.
    fn try_intercept_builtin(&self, function: &str, args: &[MirValue]) -> Option<MirValue> {
        let sink = self.stdout_sink.as_ref()?;
        let mut buf = sink.borrow_mut();
        match function {
            "print" | "print_str" | "eprint" => {
                for arg in args {
                    Self::write_value(&mut buf, arg);
                }
                Some(MirValue::Unit)
            }
            "println" | "eprintln" => {
                for arg in args {
                    Self::write_value(&mut buf, arg);
                }
                buf.push('\n');
                Some(MirValue::Unit)
            }
            "print_int" => {
                if let Some(MirValue::Int(n)) = args.first() {
                    use std::fmt::Write;
                    let _ = write!(&mut *buf, "{}", n);
                }
                Some(MirValue::Unit)
            }
            "print_float" => {
                if let Some(MirValue::Float(f)) = args.first() {
                    use std::fmt::Write;
                    let _ = write!(&mut *buf, "{}", f);
                }
                Some(MirValue::Unit)
            }
            "print_bool" => {
                if let Some(MirValue::Bool(b)) = args.first() {
                    buf.push_str(if *b { "true" } else { "false" });
                }
                Some(MirValue::Unit)
            }
            _ => None,
        }
    }

    fn write_value(buf: &mut String, value: &MirValue) {
        use std::fmt::Write;
        match value {
            MirValue::Str(s) => buf.push_str(s),
            MirValue::Int(n) => {
                let _ = write!(buf, "{}", n);
            }
            MirValue::Float(f) => {
                let _ = write!(buf, "{}", f);
            }
            MirValue::Bool(b) => {
                buf.push_str(if *b { "true" } else { "false" });
            }
            MirValue::Unit => {}
            other => {
                let _ = write!(buf, "{:?}", other);
            }
        }
    }

    fn call(&self, function: &str, args: Vec<MirValue>) -> Result<MirValue, MirInterpretError> {
        // Step 17 stage 5a: builtin print intercept (only fires when a
        // stdout sink is active — see `interpret_function_with_io`).
        if let Some(value) = self.try_intercept_builtin(function, &args) {
            return Ok(value);
        }
        let body = self
            .bodies
            .get(function)
            .ok_or_else(|| MirInterpretError::new(function, "function body not found"))?;

        if args.len() != body.params.len() {
            return Err(MirInterpretError::new(
                function,
                format!(
                    "expected {} arguments, got {}",
                    body.params.len(),
                    args.len()
                ),
            ));
        }

        let mut frame = Frame::new(body, args)?;
        let mut current = BasicBlockId(0);
        let mut steps = 0usize;

        loop {
            steps += 1;
            if steps > self.step_limit {
                return Err(frame.error("interpreter step limit exceeded"));
            }

            let block = body
                .basic_blocks
                .get(current.0 as usize)
                .ok_or_else(|| frame.error(format!("missing basic block {}", current)))?;

            for statement in &block.statements {
                frame.eval_statement(statement)?;
            }

            let terminator = block
                .terminator
                .as_ref()
                .ok_or_else(|| frame.error(format!("{} is missing a terminator", current)))?;

            match terminator {
                Terminator::Goto(target) => current = *target,
                Terminator::SwitchInt {
                    discriminant,
                    targets,
                    otherwise,
                } => {
                    let key = frame
                        .eval_operand(discriminant)?
                        .branch_key(&frame.function)?;
                    current = targets
                        .iter()
                        .find_map(|(value, target)| (*value == key).then_some(*target))
                        .unwrap_or(*otherwise);
                }
                Terminator::Return => return frame.return_value(),
                Terminator::Call {
                    func,
                    args,
                    destination,
                    target,
                } => {
                    let values = args
                        .iter()
                        .map(|arg| frame.eval_operand(arg))
                        .collect::<Result<Vec<_>, _>>()?;
                    let result = self.call(func, values)?;
                    frame.write_place(destination, result)?;
                    current = *target;
                }
                Terminator::TailCall { func, args } => {
                    let values = args
                        .iter()
                        .map(|arg| frame.eval_operand(arg))
                        .collect::<Result<Vec<_>, _>>()?;
                    return self.call(func, values);
                }
                Terminator::Unreachable => {
                    return Err(frame.error(format!("executed unreachable block {}", current)));
                }
                Terminator::Assert {
                    cond,
                    expected,
                    msg,
                    target,
                } => {
                    let actual = frame.eval_operand(cond)?.truthy(&frame.function)?;
                    if actual != *expected {
                        return Err(frame.error(format!("assertion failed: {}", msg)));
                    }
                    current = *target;
                }
            }
        }
    }
}

struct Frame {
    function: String,
    locals: Vec<MirValue>,
}

impl Frame {
    fn new(body: &Body, args: Vec<MirValue>) -> Result<Self, MirInterpretError> {
        let mut locals = vec![MirValue::Unit; body.locals.len()];
        for (idx, arg) in args.into_iter().enumerate() {
            locals[idx + 1] = arg;
        }

        Ok(Self {
            function: body.name.clone(),
            locals,
        })
    }

    fn eval_statement(&mut self, statement: &Statement) -> Result<(), MirInterpretError> {
        match statement {
            Statement::Assign(place, rvalue) => {
                let value = self.eval_rvalue(rvalue)?;
                self.write_place(place, value)
            }
            Statement::Drop(_) | Statement::Nop => Ok(()),
        }
    }

    fn eval_rvalue(&self, rvalue: &Rvalue) -> Result<MirValue, MirInterpretError> {
        match rvalue {
            Rvalue::Use(operand) => self.eval_operand(operand),
            Rvalue::BinaryOp(op, lhs, rhs) => {
                let lhs = self.eval_operand(lhs)?;
                let rhs = self.eval_operand(rhs)?;
                eval_binop(*op, lhs, rhs, &self.function)
            }
            Rvalue::UnaryOp(op, operand) => {
                let value = self.eval_operand(operand)?;
                eval_unop(*op, value, &self.function)
            }
            Rvalue::Aggregate(AggregateKind::Struct(name), operands) => {
                let values = operands
                    .iter()
                    .map(|operand| self.eval_operand(operand))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(MirValue::Struct(name.clone(), values))
            }
            Rvalue::Aggregate(AggregateKind::Enum(name, variant_index), operands) => {
                let values = operands
                    .iter()
                    .map(|operand| self.eval_operand(operand))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(MirValue::Enum(name.clone(), *variant_index, values))
            }
            Rvalue::Aggregate(AggregateKind::Vec, operands) => {
                let values = operands
                    .iter()
                    .map(|operand| self.eval_operand(operand))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(MirValue::Vec(values))
            }
            Rvalue::Aggregate(kind, _) => Err(self.error(format!(
                "unsupported aggregate for MIR interpreter: {:?}",
                kind
            ))),
            Rvalue::Discriminant(place) => match self.read_place(place)? {
                MirValue::Enum(_, variant_index, _) => Ok(MirValue::Int(variant_index as i64)),
                value => {
                    Err(self.error(format!("cannot read discriminant from value {:?}", value)))
                }
            },
            Rvalue::Cast(operand, _) => self.eval_operand(operand),
            Rvalue::Len(place) => match self.read_place(place)? {
                MirValue::Vec(values) => Ok(MirValue::Int(values.len() as i64)),
                MirValue::Str(value) => Ok(MirValue::Int(value.chars().count() as i64)),
                value => Err(self.error(format!("cannot read len from value {:?}", value))),
            },
            Rvalue::VecPush(place, operand) => {
                let mut values = match self.read_place(place)? {
                    MirValue::Vec(values) => values,
                    value => return Err(self.error(format!("cannot push into value {:?}", value))),
                };
                values.push(self.eval_operand(operand)?);
                Ok(MirValue::Vec(values))
            }
            other => Err(self.error(format!(
                "unsupported rvalue for MIR interpreter: {:?}",
                other
            ))),
        }
    }

    fn eval_operand(&self, operand: &Operand) -> Result<MirValue, MirInterpretError> {
        match operand {
            Operand::Constant(value) => Ok(MirValue::from(value)),
            Operand::Copy(place) | Operand::Move(place) => self.read_place(place),
        }
    }

    fn read_place(&self, place: &Place) -> Result<MirValue, MirInterpretError> {
        let mut value = self
            .locals
            .get(place.local.0 as usize)
            .cloned()
            .ok_or_else(|| self.error(format!("local {} is not declared", place.local)))?;

        for projection in &place.projections {
            value = match (projection, value) {
                (Projection::Field(index), MirValue::Struct(name, fields)) => {
                    fields.get(*index as usize).cloned().ok_or_else(|| {
                        self.error(format!(
                            "field projection {} is out of bounds for struct `{}`",
                            index, name
                        ))
                    })?
                }
                (Projection::Field(index), MirValue::Enum(name, variant_index, fields)) => {
                    fields.get(*index as usize).cloned().ok_or_else(|| {
                        self.error(format!(
                            "field projection {} is out of bounds for enum `{}::{}`",
                            index, name, variant_index
                        ))
                    })?
                }
                (Projection::Index(index_local), MirValue::Vec(items)) => {
                    let index = self.read_index_local(*index_local)?;
                    items.get(index).cloned().ok_or_else(|| {
                        self.error(format!(
                            "index projection {} is out of bounds for vector of length {}",
                            index,
                            items.len()
                        ))
                    })?
                }
                (Projection::Deref, value) => {
                    return Err(self.error(format!(
                        "unsupported deref projection for MIR interpreter on {:?}",
                        value
                    )));
                }
                (Projection::Index(_), value) => {
                    return Err(self.error(format!(
                        "unsupported index projection for MIR interpreter on {:?}",
                        value
                    )));
                }
                (Projection::Field(index), value) => {
                    return Err(self.error(format!(
                        "cannot project field {} from value {:?}",
                        index, value
                    )));
                }
            };
        }

        Ok(value)
    }

    fn read_index_local(&self, local: Local) -> Result<usize, MirInterpretError> {
        match self
            .locals
            .get(local.0 as usize)
            .cloned()
            .ok_or_else(|| self.error(format!("index local {} is not declared", local)))?
        {
            MirValue::Int(value) if value >= 0 => Ok(value as usize),
            MirValue::Int(value) => Err(self.error(format!("negative vector index {}", value))),
            value => Err(self.error(format!("vector index must be an integer, got {:?}", value))),
        }
    }

    fn write_place(&mut self, place: &Place, value: MirValue) -> Result<(), MirInterpretError> {
        if !place.projections.is_empty() {
            return Err(self.error(format!(
                "unsupported projected assignment for MIR interpreter: {}",
                place
            )));
        }

        let local_index = place.local.0 as usize;
        if local_index >= self.locals.len() {
            return Err(self.error(format!("local {} is not declared", place.local)));
        }

        let slot = &mut self.locals[local_index];
        *slot = value;
        Ok(())
    }

    fn return_value(&self) -> Result<MirValue, MirInterpretError> {
        self.locals
            .first()
            .cloned()
            .ok_or_else(|| self.error("missing return local _0"))
    }

    fn error(&self, message: impl Into<String>) -> MirInterpretError {
        MirInterpretError::new(self.function.clone(), message)
    }
}

impl MirValue {
    fn branch_key(&self, function: &str) -> Result<i64, MirInterpretError> {
        match self {
            MirValue::Int(v) => Ok(*v),
            MirValue::Bool(v) => Ok(if *v { 1 } else { 0 }),
            other => Err(MirInterpretError::new(
                function,
                format!("cannot branch on value {:?}", other),
            )),
        }
    }

    fn truthy(&self, function: &str) -> Result<bool, MirInterpretError> {
        match self {
            MirValue::Bool(v) => Ok(*v),
            MirValue::Int(v) => Ok(*v != 0),
            other => Err(MirInterpretError::new(
                function,
                format!("cannot treat value {:?} as bool", other),
            )),
        }
    }
}

fn eval_binop(
    op: BinOp,
    lhs: MirValue,
    rhs: MirValue,
    function: &str,
) -> Result<MirValue, MirInterpretError> {
    match (lhs, rhs) {
        (MirValue::Int(lhs), MirValue::Int(rhs)) => eval_int_binop(op, lhs, rhs, function),
        (MirValue::Bool(lhs), MirValue::Bool(rhs)) => eval_bool_binop(op, lhs, rhs, function),
        (MirValue::Str(lhs), MirValue::Str(rhs)) => match op {
            BinOp::Eq => Ok(MirValue::Bool(lhs == rhs)),
            BinOp::Ne => Ok(MirValue::Bool(lhs != rhs)),
            _ => Err(MirInterpretError::new(
                function,
                format!("unsupported string binary operation {:?}", op),
            )),
        },
        (lhs, rhs) => Err(MirInterpretError::new(
            function,
            format!("unsupported binary operands {:?} and {:?}", lhs, rhs),
        )),
    }
}

fn eval_int_binop(
    op: BinOp,
    lhs: i64,
    rhs: i64,
    function: &str,
) -> Result<MirValue, MirInterpretError> {
    if matches!(op, BinOp::Div | BinOp::Rem) && rhs == 0 {
        return Err(MirInterpretError::new(function, "integer division by zero"));
    }

    let value = match op {
        BinOp::Add => MirValue::Int(lhs + rhs),
        BinOp::Sub => MirValue::Int(lhs - rhs),
        BinOp::Mul => MirValue::Int(lhs * rhs),
        BinOp::Div => MirValue::Int(lhs / rhs),
        BinOp::Rem => MirValue::Int(lhs % rhs),
        BinOp::BitAnd => MirValue::Int(lhs & rhs),
        BinOp::BitOr => MirValue::Int(lhs | rhs),
        BinOp::BitXor => MirValue::Int(lhs ^ rhs),
        BinOp::Shl => MirValue::Int(lhs << rhs),
        BinOp::Shr => MirValue::Int(lhs >> rhs),
        BinOp::Eq => MirValue::Bool(lhs == rhs),
        BinOp::Ne => MirValue::Bool(lhs != rhs),
        BinOp::Lt => MirValue::Bool(lhs < rhs),
        BinOp::Le => MirValue::Bool(lhs <= rhs),
        BinOp::Gt => MirValue::Bool(lhs > rhs),
        BinOp::Ge => MirValue::Bool(lhs >= rhs),
    };
    Ok(value)
}

fn eval_bool_binop(
    op: BinOp,
    lhs: bool,
    rhs: bool,
    function: &str,
) -> Result<MirValue, MirInterpretError> {
    match op {
        BinOp::Eq => Ok(MirValue::Bool(lhs == rhs)),
        BinOp::Ne => Ok(MirValue::Bool(lhs != rhs)),
        BinOp::BitAnd => Ok(MirValue::Bool(lhs & rhs)),
        BinOp::BitOr => Ok(MirValue::Bool(lhs | rhs)),
        _ => Err(MirInterpretError::new(
            function,
            format!("unsupported bool binary operation {:?}", op),
        )),
    }
}

fn eval_unop(op: UnOp, value: MirValue, function: &str) -> Result<MirValue, MirInterpretError> {
    match (op, value) {
        (UnOp::Neg, MirValue::Int(value)) => Ok(MirValue::Int(-value)),
        (UnOp::Neg, MirValue::Float(value)) => Ok(MirValue::Float(-value)),
        (UnOp::Not, MirValue::Bool(value)) => Ok(MirValue::Bool(!value)),
        (UnOp::Not, MirValue::Int(value)) => Ok(MirValue::Int(!value)),
        (op, value) => Err(MirInterpretError::new(
            function,
            format!("unsupported unary operation {:?} on {:?}", op, value),
        )),
    }
}
