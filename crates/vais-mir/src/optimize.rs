//! MIR optimization passes: DCE, CSE, constant propagation.
//!
//! These operate on the structured MIR CFG rather than text-based LLVM IR,
//! providing more reliable and composable optimizations.

use crate::types::*;
use std::collections::{HashMap, HashSet};

/// Apply all MIR optimization passes to a module.
pub fn optimize_mir_module(module: &mut MirModule) {
    for body in &mut module.bodies {
        optimize_mir_body(body);
    }
}

/// Apply all MIR optimization passes to a single function body.
pub fn optimize_mir_body(body: &mut Body) {
    constant_propagation(body);
    dead_code_elimination(body);
    common_subexpression_elimination(body);
    remove_unreachable_blocks(body);
}

/// Dead Code Elimination on MIR.
///
/// Removes assignments to locals that are never read by any subsequent
/// statement or terminator in the function body.
pub fn dead_code_elimination(body: &mut Body) {
    // Collect all locals that are *used* (read from)
    let used_locals = collect_used_locals(body);

    // Remove assignments to unused locals (keep _0 return place always)
    for bb in &mut body.basic_blocks {
        bb.statements.retain(|stmt| {
            match stmt {
                Statement::Assign(place, _) => {
                    // Always keep return place (_0)
                    if place.local.0 == 0 {
                        return true;
                    }
                    // Keep if the local is used somewhere
                    used_locals.contains(&place.local)
                }
                Statement::Drop(_) | Statement::Nop => true,
            }
        });
    }
}

/// Collect all locals that are read (used as operands) in the body.
fn collect_used_locals(body: &Body) -> HashSet<Local> {
    let mut used = HashSet::new();

    for bb in &body.basic_blocks {
        for stmt in &bb.statements {
            match stmt {
                Statement::Assign(_, rvalue) => {
                    collect_rvalue_reads(rvalue, &mut used);
                }
                Statement::Drop(place) => {
                    used.insert(place.local);
                }
                Statement::Nop => {}
            }
        }

        if let Some(ref term) = bb.terminator {
            collect_terminator_reads(term, &mut used);
        }
    }

    used
}

/// Collect locals read by an rvalue.
fn collect_rvalue_reads(rvalue: &Rvalue, used: &mut HashSet<Local>) {
    match rvalue {
        Rvalue::Use(op) => collect_operand_reads(op, used),
        Rvalue::BinaryOp(_, lhs, rhs) => {
            collect_operand_reads(lhs, used);
            collect_operand_reads(rhs, used);
        }
        Rvalue::UnaryOp(_, op) => collect_operand_reads(op, used),
        Rvalue::Ref(place) => {
            used.insert(place.local);
        }
        Rvalue::Aggregate(_, ops) => {
            for op in ops {
                collect_operand_reads(op, used);
            }
        }
        Rvalue::Discriminant(place) => {
            used.insert(place.local);
        }
        Rvalue::Cast(op, _) => collect_operand_reads(op, used),
        Rvalue::Len(place) => {
            used.insert(place.local);
        }
    }
}

/// Collect locals read by an operand.
fn collect_operand_reads(op: &Operand, used: &mut HashSet<Local>) {
    match op {
        Operand::Copy(place) | Operand::Move(place) => {
            used.insert(place.local);
            // Also collect locals used in projections
            for proj in &place.projections {
                if let Projection::Index(local) = proj {
                    used.insert(*local);
                }
            }
        }
        Operand::Constant(_) => {}
    }
}

/// Collect locals read by a terminator.
fn collect_terminator_reads(term: &Terminator, used: &mut HashSet<Local>) {
    match term {
        Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
        Terminator::SwitchInt { discriminant, .. } => {
            collect_operand_reads(discriminant, used);
        }
        Terminator::Call {
            args, destination, ..
        } => {
            for arg in args {
                collect_operand_reads(arg, used);
            }
            // The destination is written, but if it has projections those are reads
            for proj in &destination.projections {
                if let Projection::Index(local) = proj {
                    used.insert(*local);
                }
            }
        }
        Terminator::TailCall { args, .. } => {
            for arg in args {
                collect_operand_reads(arg, used);
            }
        }
        Terminator::Assert { cond, .. } => {
            collect_operand_reads(cond, used);
        }
    }
}

/// Common Subexpression Elimination on MIR.
///
/// Within each basic block, if the same rvalue is computed twice,
/// the second computation is replaced with a copy of the first result.
pub fn common_subexpression_elimination(body: &mut Body) {
    for bb in &mut body.basic_blocks {
        let mut expr_map: HashMap<String, Local> = HashMap::new();
        let mut replacements: Vec<(usize, Local)> = vec![];

        for (i, stmt) in bb.statements.iter().enumerate() {
            if let Statement::Assign(place, rvalue) = stmt {
                // Only CSE simple rvalues (binary ops, unary ops)
                if let Some(key) = rvalue_key(rvalue) {
                    if let Some(&existing_local) = expr_map.get(&key) {
                        // This expression was already computed
                        replacements.push((i, existing_local));
                    } else {
                        // Record this expression
                        if place.projections.is_empty() {
                            expr_map.insert(key, place.local);
                        }
                    }
                }
            }
        }

        // Apply replacements (replace rvalue with copy of existing result)
        for (idx, existing_local) in replacements {
            if let Statement::Assign(place, _) = &bb.statements[idx] {
                let place = place.clone();
                bb.statements[idx] = Statement::Assign(
                    place,
                    Rvalue::Use(Operand::Copy(Place::local(existing_local))),
                );
            }
        }
    }
}

/// Generate a canonical key for an rvalue for CSE purposes.
fn rvalue_key(rvalue: &Rvalue) -> Option<String> {
    match rvalue {
        Rvalue::BinaryOp(op, lhs, rhs) => Some(format!("{:?}({:?},{:?})", op, lhs, rhs)),
        Rvalue::UnaryOp(op, operand) => Some(format!("{:?}({:?})", op, operand)),
        // Don't CSE other rvalues (side effects, aggregates, etc.)
        _ => None,
    }
}

/// Constant propagation on MIR.
///
/// If a local is assigned a constant and never reassigned, propagate the
/// constant to all uses of that local.
pub fn constant_propagation(body: &mut Body) {
    // Collect single-assignment constants
    let mut const_map: HashMap<Local, Constant> = HashMap::new();
    let mut assigned_more_than_once: HashSet<Local> = HashSet::new();

    for bb in &body.basic_blocks {
        for stmt in &bb.statements {
            if let Statement::Assign(place, rvalue) = stmt {
                if place.projections.is_empty() {
                    if assigned_more_than_once.contains(&place.local) {
                        continue;
                    }
                    if const_map.contains_key(&place.local) {
                        // Assigned more than once - remove from const_map
                        const_map.remove(&place.local);
                        assigned_more_than_once.insert(place.local);
                        continue;
                    }
                    if let Rvalue::Use(Operand::Constant(c)) = rvalue {
                        const_map.insert(place.local, c.clone());
                    }
                }
            }
        }
    }

    if const_map.is_empty() {
        return;
    }

    // Propagate constants to operands
    for bb in &mut body.basic_blocks {
        for stmt in &mut bb.statements {
            if let Statement::Assign(_, rvalue) = stmt {
                propagate_in_rvalue(rvalue, &const_map);
            }
        }
        if let Some(ref mut term) = bb.terminator {
            propagate_in_terminator(term, &const_map);
        }
    }
}

/// Propagate constants into an rvalue.
fn propagate_in_rvalue(rvalue: &mut Rvalue, const_map: &HashMap<Local, Constant>) {
    match rvalue {
        Rvalue::Use(op) => propagate_in_operand(op, const_map),
        Rvalue::BinaryOp(_, lhs, rhs) => {
            propagate_in_operand(lhs, const_map);
            propagate_in_operand(rhs, const_map);
        }
        Rvalue::UnaryOp(_, op) => propagate_in_operand(op, const_map),
        Rvalue::Cast(op, _) => propagate_in_operand(op, const_map),
        Rvalue::Aggregate(_, ops) => {
            for op in ops {
                propagate_in_operand(op, const_map);
            }
        }
        _ => {}
    }
}

/// Propagate constants into an operand.
fn propagate_in_operand(op: &mut Operand, const_map: &HashMap<Local, Constant>) {
    match op {
        Operand::Copy(place) | Operand::Move(place) => {
            if place.projections.is_empty() {
                if let Some(c) = const_map.get(&place.local) {
                    *op = Operand::Constant(c.clone());
                }
            }
        }
        Operand::Constant(_) => {}
    }
}

/// Propagate constants into a terminator.
fn propagate_in_terminator(term: &mut Terminator, const_map: &HashMap<Local, Constant>) {
    match term {
        Terminator::SwitchInt { discriminant, .. } => {
            propagate_in_operand(discriminant, const_map);
        }
        Terminator::Call { args, .. } => {
            for arg in args {
                propagate_in_operand(arg, const_map);
            }
        }
        Terminator::TailCall { args, .. } => {
            for arg in args {
                propagate_in_operand(arg, const_map);
            }
        }
        Terminator::Assert { cond, .. } => {
            propagate_in_operand(cond, const_map);
        }
        _ => {}
    }
}

/// Remove unreachable basic blocks.
///
/// Starting from bb0 (entry), mark all reachable blocks via BFS,
/// then remove unreachable ones.
pub fn remove_unreachable_blocks(body: &mut Body) {
    if body.basic_blocks.is_empty() {
        return;
    }

    let num_blocks = body.basic_blocks.len();
    let mut reachable = vec![false; num_blocks];
    let mut worklist = vec![0usize]; // Start from bb0
    reachable[0] = true;

    while let Some(idx) = worklist.pop() {
        if let Some(ref term) = body.basic_blocks[idx].terminator {
            for succ in terminator_successors(term) {
                let succ_idx = succ.0 as usize;
                if succ_idx < num_blocks && !reachable[succ_idx] {
                    reachable[succ_idx] = true;
                    worklist.push(succ_idx);
                }
            }
        }
    }

    // Replace unreachable blocks with empty blocks (to preserve indices)
    for (i, is_reachable) in reachable.iter().enumerate() {
        if !is_reachable {
            body.basic_blocks[i] = BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Unreachable),
            };
        }
    }
}

/// Get successor block IDs from a terminator.
fn terminator_successors(term: &Terminator) -> Vec<BasicBlockId> {
    match term {
        Terminator::Goto(bb) => vec![*bb],
        Terminator::SwitchInt {
            targets, otherwise, ..
        } => {
            let mut succs: Vec<BasicBlockId> = targets.iter().map(|(_, bb)| *bb).collect();
            succs.push(*otherwise);
            succs
        }
        Terminator::Return | Terminator::Unreachable | Terminator::TailCall { .. } => vec![],
        Terminator::Call { target, .. } => vec![*target],
        Terminator::Assert { target, .. } => vec![*target],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MirBuilder;

    #[test]
    fn test_dce_removes_unused_locals() {
        let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

        let unused = builder.new_local(MirType::I64, Some("unused".into()));
        let used = builder.new_local(MirType::I64, Some("used".into()));

        // Assign to unused (should be removed)
        builder.assign_const(unused, Constant::Int(42));
        // Assign to used
        builder.assign_const(used, Constant::Int(10));
        // Return place uses 'used'
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(used))),
        );
        builder.return_();

        let mut body = builder.build();
        dead_code_elimination(&mut body);

        // The unused assignment should be removed
        // Entry block should have 2 statements: assign to 'used' + assign to return place
        assert_eq!(body.basic_blocks[0].statements.len(), 2);
    }

    #[test]
    fn test_cse_eliminates_duplicate_binop() {
        let mut builder = MirBuilder::new("test", vec![MirType::I64, MirType::I64], MirType::I64);

        let t1 = builder.new_local(MirType::I64, None);
        let t2 = builder.new_local(MirType::I64, None);

        let param_a = Operand::Copy(Place::local(builder.param(0)));
        let param_b = Operand::Copy(Place::local(builder.param(1)));

        // Same binary op twice
        builder.assign_binop(t1, BinOp::Add, param_a.clone(), param_b.clone());
        builder.assign_binop(t2, BinOp::Add, param_a, param_b);

        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(t2))),
        );
        builder.return_();

        let mut body = builder.build();
        common_subexpression_elimination(&mut body);

        // The second assignment should now be a copy of t1
        if let Statement::Assign(_, Rvalue::Use(Operand::Copy(place))) =
            &body.basic_blocks[0].statements[1]
        {
            assert_eq!(place.local, t1);
        } else {
            panic!("Expected CSE to replace with copy");
        }
    }

    #[test]
    fn test_constant_propagation() {
        let mut builder = MirBuilder::new("test", vec![], MirType::I64);

        let c = builder.new_local(MirType::I64, None);
        let result = builder.new_local(MirType::I64, None);

        builder.assign_const(c, Constant::Int(42));
        builder.assign(
            Place::local(result),
            Rvalue::BinaryOp(
                BinOp::Add,
                Operand::Copy(Place::local(c)),
                Operand::Constant(Constant::Int(1)),
            ),
        );
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(result))),
        );
        builder.return_();

        let mut body = builder.build();
        constant_propagation(&mut body);

        // The use of 'c' in the binop should be replaced with const 42
        if let Statement::Assign(_, Rvalue::BinaryOp(_, lhs, _)) =
            &body.basic_blocks[0].statements[1]
        {
            assert_eq!(*lhs, Operand::Constant(Constant::Int(42)));
        } else {
            panic!("Expected constant propagation");
        }
    }

    #[test]
    fn test_unreachable_block_removal() {
        let mut builder = MirBuilder::new("test", vec![], MirType::I64);

        let bb1 = builder.new_block();
        let _bb2 = builder.new_block(); // unreachable

        builder.assign_const(Local(0), Constant::Int(0));
        builder.goto(bb1);

        builder.switch_to_block(bb1);
        builder.return_();

        // bb2 is unreachable (no goto/switch targets it)
        builder.switch_to_block(_bb2);
        builder.assign_const(Local(0), Constant::Int(99));
        builder.return_();

        let mut body = builder.build();
        remove_unreachable_blocks(&mut body);

        // bb2 should be replaced with unreachable
        assert_eq!(
            body.basic_blocks[2].terminator,
            Some(Terminator::Unreachable)
        );
        assert!(body.basic_blocks[2].statements.is_empty());
    }
}
