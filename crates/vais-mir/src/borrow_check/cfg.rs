//! CFG utilities: liveness analysis and control-flow graph helpers.

use super::*;

/// Compute liveness information for all locals in the body.
/// Returns the last location where each local is used.
pub(super) fn compute_liveness(body: &Body) -> LivenessInfo {
    let mut last_use: HashMap<Local, Location> = HashMap::new();

    for (block_idx, block) in body.basic_blocks.iter().enumerate() {
        let block_id = BasicBlockId(block_idx as u32);

        // Process statements
        for (stmt_idx, statement) in block.statements.iter().enumerate() {
            let location = Location::new(block_id, stmt_idx);

            // Extract locals used in this statement
            match statement {
                Statement::Assign(_place, rvalue) => {
                    // Check rvalue for used locals
                    visit_rvalue_locals(rvalue, &mut |local| {
                        last_use.insert(local, location);
                    });
                }
                Statement::Drop(place) => {
                    last_use.insert(place.local, location);
                }
                Statement::Nop => {}
            }
        }

        // Process terminator
        if let Some(ref terminator) = block.terminator {
            let location = Location::new(block_id, block.statements.len());

            match terminator {
                Terminator::SwitchInt { discriminant, .. } => {
                    visit_operand_locals(discriminant, &mut |local| {
                        last_use.insert(local, location);
                    });
                }
                Terminator::Call { args, .. } | Terminator::TailCall { args, .. } => {
                    for arg in args {
                        visit_operand_locals(arg, &mut |local| {
                            last_use.insert(local, location);
                        });
                    }
                }
                Terminator::Assert { cond, .. } => {
                    visit_operand_locals(cond, &mut |local| {
                        last_use.insert(local, location);
                    });
                }
                Terminator::Return | Terminator::Goto(_) | Terminator::Unreachable => {}
            }
        }
    }

    LivenessInfo { last_use }
}

/// Visit all locals used in an operand.
fn visit_operand_locals<F>(operand: &Operand, f: &mut F)
where
    F: FnMut(Local),
{
    match operand {
        Operand::Copy(place) | Operand::Move(place) => {
            f(place.local);
        }
        Operand::Constant(_) => {}
    }
}

/// Visit all locals used in an rvalue.
fn visit_rvalue_locals<F>(rvalue: &Rvalue, f: &mut F)
where
    F: FnMut(Local),
{
    match rvalue {
        Rvalue::Use(operand) => {
            visit_operand_locals(operand, f);
        }
        Rvalue::BinaryOp(_, left, right) => {
            visit_operand_locals(left, f);
            visit_operand_locals(right, f);
        }
        Rvalue::UnaryOp(_, operand) => {
            visit_operand_locals(operand, f);
        }
        Rvalue::Ref(place) => {
            f(place.local);
        }
        Rvalue::Aggregate(_, operands) => {
            for operand in operands {
                visit_operand_locals(operand, f);
            }
        }
        Rvalue::Discriminant(place) => {
            f(place.local);
        }
        Rvalue::Cast(operand, _) => {
            visit_operand_locals(operand, f);
        }
        Rvalue::Len(place) => {
            f(place.local);
        }
    }
}

/// Compute the predecessor blocks for each block in the CFG.
/// Returns a map from block ID to the list of blocks that can jump to it.
pub fn cfg_predecessors(body: &Body) -> HashMap<BasicBlockId, Vec<BasicBlockId>> {
    let mut predecessors: HashMap<BasicBlockId, Vec<BasicBlockId>> = HashMap::new();

    for (block_idx, block) in body.basic_blocks.iter().enumerate() {
        let block_id = BasicBlockId(block_idx as u32);

        if let Some(ref terminator) = block.terminator {
            let successors = match terminator {
                Terminator::Goto(target) => vec![*target],
                Terminator::SwitchInt {
                    targets, otherwise, ..
                } => {
                    let mut succ = targets
                        .iter()
                        .map(|(_, target)| *target)
                        .collect::<Vec<_>>();
                    succ.push(*otherwise);
                    succ
                }
                Terminator::Return | Terminator::Unreachable => vec![],
                Terminator::Call { target, .. } => vec![*target],
                Terminator::TailCall { .. } => vec![],
                Terminator::Assert { target, .. } => vec![*target],
            };

            for successor in successors {
                predecessors.entry(successor).or_default().push(block_id);
            }
        }
    }

    predecessors
}

/// Compute the successor blocks for each block in the CFG.
/// Returns a map from block ID to the list of blocks that this block can jump to.
pub fn cfg_successors(body: &Body) -> HashMap<BasicBlockId, Vec<BasicBlockId>> {
    let mut successors: HashMap<BasicBlockId, Vec<BasicBlockId>> = HashMap::new();

    for (block_idx, block) in body.basic_blocks.iter().enumerate() {
        let block_id = BasicBlockId(block_idx as u32);

        if let Some(ref terminator) = block.terminator {
            let succ = match terminator {
                Terminator::Goto(target) => vec![*target],
                Terminator::SwitchInt {
                    targets, otherwise, ..
                } => {
                    let mut s = targets
                        .iter()
                        .map(|(_, target)| *target)
                        .collect::<Vec<_>>();
                    s.push(*otherwise);
                    s
                }
                Terminator::Return | Terminator::Unreachable => vec![],
                Terminator::Call { target, .. } => vec![*target],
                Terminator::TailCall { .. } => vec![],
                Terminator::Assert { target, .. } => vec![*target],
            };

            successors.insert(block_id, succ);
        } else {
            successors.insert(block_id, vec![]);
        }
    }

    successors
}
