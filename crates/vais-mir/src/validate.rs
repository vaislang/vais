//! Structural validation for MIR modules.
//!
//! This validator checks whether a MIR body is internally well formed. It is
//! deliberately structural: it does not prove source-level semantics or perform
//! full type checking. Its job is to catch invalid local references, invalid
//! control-flow edges, missing terminators, and malformed function frame setup
//! before later MIR consumers treat the body as trusted input.

use crate::types::*;
use std::collections::HashSet;
use std::fmt;

/// A single structural MIR validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirValidationError {
    pub body: String,
    pub block: Option<BasicBlockId>,
    pub statement_index: Option<usize>,
    pub message: String,
}

impl MirValidationError {
    fn body(body: &Body, message: impl Into<String>) -> Self {
        Self {
            body: body.name.clone(),
            block: None,
            statement_index: None,
            message: message.into(),
        }
    }

    fn block(body: &Body, block: BasicBlockId, message: impl Into<String>) -> Self {
        Self {
            body: body.name.clone(),
            block: Some(block),
            statement_index: None,
            message: message.into(),
        }
    }

    fn statement(
        body: &Body,
        block: BasicBlockId,
        statement_index: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            body: body.name.clone(),
            block: Some(block),
            statement_index: Some(statement_index),
            message: message.into(),
        }
    }
}

impl fmt::Display for MirValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.body)?;
        if let Some(block) = self.block {
            write!(f, "{}: ", block)?;
        }
        if let Some(statement_index) = self.statement_index {
            write!(f, "statement {}: ", statement_index)?;
        }
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MirValidationError {}

pub type MirValidationResult = Result<(), Vec<MirValidationError>>;

/// Validate every body in a MIR module.
pub fn validate_module(module: &MirModule) -> MirValidationResult {
    let mut errors = Vec::new();
    let mut names = HashSet::new();

    for body in &module.bodies {
        if !names.insert(body.name.as_str()) {
            errors.push(MirValidationError::body(
                body,
                format!("duplicate function body `{}`", body.name),
            ));
        }

        if let Err(mut body_errors) = validate_body(body) {
            errors.append(&mut body_errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate one MIR function body.
pub fn validate_body(body: &Body) -> MirValidationResult {
    let mut errors = Vec::new();

    if body.locals.is_empty() {
        errors.push(MirValidationError::body(body, "missing return local _0"));
    } else if body.locals[0].ty != body.return_type {
        errors.push(MirValidationError::body(
            body,
            format!(
                "return local _0 has type {:?}, expected {:?}",
                body.locals[0].ty, body.return_type
            ),
        ));
    }

    if body.locals.len() < body.params.len() + 1 {
        errors.push(MirValidationError::body(
            body,
            format!(
                "only {} locals declared for {} params plus return local",
                body.locals.len(),
                body.params.len()
            ),
        ));
    } else {
        for (index, param_type) in body.params.iter().enumerate() {
            let local_index = index + 1;
            let declared_type = &body.locals[local_index].ty;
            if declared_type != param_type {
                errors.push(MirValidationError::body(
                    body,
                    format!(
                        "parameter local _{} has type {:?}, expected {:?}",
                        local_index, declared_type, param_type
                    ),
                ));
            }
        }
    }

    if body.basic_blocks.is_empty() {
        errors.push(MirValidationError::body(body, "body has no basic blocks"));
    }

    for (name, block) in &body.block_names {
        validate_block_target(
            body,
            *block,
            format!("named block `{}` points to missing block", name),
            &mut errors,
        );
    }

    for (block_index, block) in body.basic_blocks.iter().enumerate() {
        let block_id = BasicBlockId(block_index as u32);

        for (statement_index, statement) in block.statements.iter().enumerate() {
            validate_statement(body, block_id, statement_index, statement, &mut errors);
        }

        match &block.terminator {
            Some(terminator) => validate_terminator(body, block_id, terminator, &mut errors),
            None => errors.push(MirValidationError::block(
                body,
                block_id,
                "basic block is missing a terminator",
            )),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_statement(
    body: &Body,
    block: BasicBlockId,
    statement_index: usize,
    statement: &Statement,
    errors: &mut Vec<MirValidationError>,
) {
    match statement {
        Statement::Assign(place, rvalue) => {
            validate_place_in_statement(
                body,
                block,
                statement_index,
                place,
                "assignment target",
                errors,
            );
            validate_rvalue(body, block, statement_index, rvalue, errors);
        }
        Statement::Drop(place) => {
            validate_place_in_statement(body, block, statement_index, place, "drop target", errors);
        }
        Statement::Nop => {}
    }
}

fn validate_rvalue(
    body: &Body,
    block: BasicBlockId,
    statement_index: usize,
    rvalue: &Rvalue,
    errors: &mut Vec<MirValidationError>,
) {
    match rvalue {
        Rvalue::Use(operand) => {
            validate_operand_in_statement(
                body,
                block,
                statement_index,
                operand,
                "use operand",
                errors,
            );
        }
        Rvalue::BinaryOp(_, lhs, rhs) => {
            validate_operand_in_statement(body, block, statement_index, lhs, "binary lhs", errors);
            validate_operand_in_statement(body, block, statement_index, rhs, "binary rhs", errors);
        }
        Rvalue::UnaryOp(_, operand) | Rvalue::Cast(operand, _) => {
            validate_operand_in_statement(
                body,
                block,
                statement_index,
                operand,
                "unary operand",
                errors,
            );
        }
        Rvalue::Ref(place) => {
            validate_place_in_statement(
                body,
                block,
                statement_index,
                place,
                "reference target",
                errors,
            );
        }
        Rvalue::Aggregate(_, operands) => {
            for (index, operand) in operands.iter().enumerate() {
                validate_operand_in_statement(
                    body,
                    block,
                    statement_index,
                    operand,
                    format!("aggregate operand {}", index),
                    errors,
                );
            }
        }
        Rvalue::Discriminant(place) => {
            validate_place_in_statement(
                body,
                block,
                statement_index,
                place,
                "discriminant target",
                errors,
            );
        }
        Rvalue::Len(place) => {
            validate_place_in_statement(body, block, statement_index, place, "len target", errors);
        }
        Rvalue::VecPush(place, operand) => {
            validate_place_in_statement(
                body,
                block,
                statement_index,
                place,
                "vec push receiver",
                errors,
            );
            validate_operand_in_statement(
                body,
                block,
                statement_index,
                operand,
                "vec push value",
                errors,
            );
        }
    }
}

fn validate_terminator(
    body: &Body,
    block: BasicBlockId,
    terminator: &Terminator,
    errors: &mut Vec<MirValidationError>,
) {
    match terminator {
        Terminator::Goto(target) => {
            validate_block_target(body, *target, "goto target is missing", errors);
        }
        Terminator::SwitchInt {
            discriminant,
            targets,
            otherwise,
        } => {
            validate_operand_in_terminator(
                body,
                block,
                discriminant,
                "switch discriminant",
                errors,
            );
            let mut seen_values = HashSet::new();
            for (value, target) in targets {
                if !seen_values.insert(*value) {
                    errors.push(MirValidationError::block(
                        body,
                        block,
                        format!("duplicate switch value {}", value),
                    ));
                }
                validate_block_target(body, *target, "switch target is missing", errors);
            }
            validate_block_target(
                body,
                *otherwise,
                "switch otherwise target is missing",
                errors,
            );
        }
        Terminator::Return | Terminator::Unreachable => {}
        Terminator::Call {
            args,
            destination,
            target,
            ..
        } => {
            for (index, arg) in args.iter().enumerate() {
                validate_operand_in_terminator(
                    body,
                    block,
                    arg,
                    format!("call argument {}", index),
                    errors,
                );
            }
            validate_place_in_terminator(body, block, destination, "call destination", errors);
            validate_block_target(body, *target, "call continuation target is missing", errors);
        }
        Terminator::TailCall { args, .. } => {
            for (index, arg) in args.iter().enumerate() {
                validate_operand_in_terminator(
                    body,
                    block,
                    arg,
                    format!("tailcall argument {}", index),
                    errors,
                );
            }
        }
        Terminator::Assert { cond, target, .. } => {
            validate_operand_in_terminator(body, block, cond, "assert condition", errors);
            validate_block_target(body, *target, "assert success target is missing", errors);
        }
    }
}

fn validate_operand_in_statement(
    body: &Body,
    block: BasicBlockId,
    statement_index: usize,
    operand: &Operand,
    context: impl Into<String>,
    errors: &mut Vec<MirValidationError>,
) {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => {
            validate_place_in_statement(body, block, statement_index, place, context, errors)
        }
        Operand::Constant(_) => {}
    }
}

fn validate_operand_in_terminator(
    body: &Body,
    block: BasicBlockId,
    operand: &Operand,
    context: impl Into<String>,
    errors: &mut Vec<MirValidationError>,
) {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => {
            validate_place_in_terminator(body, block, place, context, errors);
        }
        Operand::Constant(_) => {}
    }
}

fn validate_place_in_statement(
    body: &Body,
    block: BasicBlockId,
    statement_index: usize,
    place: &Place,
    context: impl Into<String>,
    errors: &mut Vec<MirValidationError>,
) {
    let context = context.into();
    if !local_exists(body, place.local) {
        errors.push(MirValidationError::statement(
            body,
            block,
            statement_index,
            format!("{} references undeclared local {}", context, place.local),
        ));
    }

    for projection in &place.projections {
        if let Projection::Index(local) = projection {
            if !local_exists(body, *local) {
                errors.push(MirValidationError::statement(
                    body,
                    block,
                    statement_index,
                    format!("{} index references undeclared local {}", context, local),
                ));
            }
        }
    }
}

fn validate_place_in_terminator(
    body: &Body,
    block: BasicBlockId,
    place: &Place,
    context: impl Into<String>,
    errors: &mut Vec<MirValidationError>,
) {
    let context = context.into();
    if !local_exists(body, place.local) {
        errors.push(MirValidationError::block(
            body,
            block,
            format!("{} references undeclared local {}", context, place.local),
        ));
    }

    for projection in &place.projections {
        if let Projection::Index(local) = projection {
            if !local_exists(body, *local) {
                errors.push(MirValidationError::block(
                    body,
                    block,
                    format!("{} index references undeclared local {}", context, local),
                ));
            }
        }
    }
}

fn validate_block_target(
    body: &Body,
    target: BasicBlockId,
    context: impl Into<String>,
    errors: &mut Vec<MirValidationError>,
) {
    if target.0 as usize >= body.basic_blocks.len() {
        errors.push(MirValidationError::body(
            body,
            format!("{}: {}", context.into(), target),
        ));
    }
}

fn local_exists(body: &Body, local: Local) -> bool {
    (local.0 as usize) < body.locals.len()
}
