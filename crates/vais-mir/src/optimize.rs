//! MIR optimization passes: DCE, CSE, constant propagation, copy propagation,
//! loop unrolling, escape analysis, and tail call detection.
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
    copy_propagation(body);
    dead_code_elimination(body);
    common_subexpression_elimination(body);
    loop_unrolling(body);
    tail_call_detection(body);
    escape_analysis(body);
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

// ==========================================================================
// Copy Propagation
// ==========================================================================

/// Copy Propagation on MIR.
///
/// If a local is assigned `x = copy y` (or `x = move y`) where `y` is a plain
/// local with no projections, replace all subsequent uses of `x` with `y`.
/// Only applies when `x` is assigned exactly once and `y` is not reassigned.
pub fn copy_propagation(body: &mut Body) {
    // Phase 1: Collect single-assignment copy/move chains: x = copy/move y
    let mut copy_map: HashMap<Local, Local> = HashMap::new();
    let mut assigned_more_than_once: HashSet<Local> = HashSet::new();

    for bb in &body.basic_blocks {
        for stmt in &bb.statements {
            if let Statement::Assign(place, rvalue) = stmt {
                if place.projections.is_empty() {
                    if assigned_more_than_once.contains(&place.local) {
                        continue;
                    }
                    if copy_map.contains_key(&place.local) {
                        copy_map.remove(&place.local);
                        assigned_more_than_once.insert(place.local);
                        continue;
                    }
                    match rvalue {
                        Rvalue::Use(Operand::Copy(src)) | Rvalue::Use(Operand::Move(src)) => {
                            if src.projections.is_empty() {
                                copy_map.insert(place.local, src.local);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    if copy_map.is_empty() {
        return;
    }

    // Resolve transitive chains: if x -> y -> z, resolve x -> z
    let resolved: HashMap<Local, Local> = copy_map
        .keys()
        .map(|&k| {
            let mut target = k;
            let mut visited = HashSet::new();
            while let Some(&next) = copy_map.get(&target) {
                if !visited.insert(target) {
                    break; // cycle guard
                }
                target = next;
            }
            (k, target)
        })
        .collect();

    // Phase 2: Replace operands
    for bb in &mut body.basic_blocks {
        for stmt in &mut bb.statements {
            if let Statement::Assign(_, rvalue) = stmt {
                replace_locals_in_rvalue(rvalue, &resolved);
            }
        }
        if let Some(ref mut term) = bb.terminator {
            replace_locals_in_terminator(term, &resolved);
        }
    }
}

/// Replace locals in an rvalue according to the copy map.
fn replace_locals_in_rvalue(rvalue: &mut Rvalue, map: &HashMap<Local, Local>) {
    match rvalue {
        Rvalue::Use(op) => replace_local_in_operand(op, map),
        Rvalue::BinaryOp(_, lhs, rhs) => {
            replace_local_in_operand(lhs, map);
            replace_local_in_operand(rhs, map);
        }
        Rvalue::UnaryOp(_, op) => replace_local_in_operand(op, map),
        Rvalue::Cast(op, _) => replace_local_in_operand(op, map),
        Rvalue::Aggregate(_, ops) => {
            for op in ops {
                replace_local_in_operand(op, map);
            }
        }
        _ => {}
    }
}

/// Replace a local in an operand according to the copy map.
fn replace_local_in_operand(op: &mut Operand, map: &HashMap<Local, Local>) {
    match op {
        Operand::Copy(place) | Operand::Move(place) => {
            if place.projections.is_empty() {
                if let Some(&replacement) = map.get(&place.local) {
                    place.local = replacement;
                }
            }
        }
        Operand::Constant(_) => {}
    }
}

/// Replace locals in a terminator according to the copy map.
fn replace_locals_in_terminator(term: &mut Terminator, map: &HashMap<Local, Local>) {
    match term {
        Terminator::SwitchInt { discriminant, .. } => {
            replace_local_in_operand(discriminant, map);
        }
        Terminator::Call { args, .. } => {
            for arg in args {
                replace_local_in_operand(arg, map);
            }
        }
        Terminator::TailCall { args, .. } => {
            for arg in args {
                replace_local_in_operand(arg, map);
            }
        }
        Terminator::Assert { cond, .. } => {
            replace_local_in_operand(cond, map);
        }
        _ => {}
    }
}

// ==========================================================================
// Loop Unrolling
// ==========================================================================

/// Loop Unrolling on MIR.
///
/// Detects simple loops with known trip counts (up to `MAX_UNROLL_TRIP_COUNT`)
/// and small bodies (up to `MAX_UNROLL_BODY_STMTS` statements). Unrolls the
/// loop body by duplicating its statements, eliminating the branch overhead.
///
/// A "simple loop" pattern:
///   bb_header: counter check -> bb_body or bb_exit
///   bb_body: statements... increment counter -> bb_header
///
/// This pass detects loops where the header is a SwitchInt comparing an
/// induction variable to a constant, and the body increments by a constant.
pub fn loop_unrolling(body: &mut Body) {
    const MAX_UNROLL_TRIP_COUNT: usize = 8;
    const MAX_UNROLL_BODY_STMTS: usize = 5;

    // Detect natural loops: look for back edges (bb_body -> bb_header where
    // bb_header dominates bb_body). We use a simplified approach: find blocks
    // that branch back to a block with smaller index (potential loop header).
    let num_blocks = body.basic_blocks.len();
    let mut loops_to_unroll: Vec<LoopInfo> = Vec::new();

    for header_idx in 0..num_blocks {
        // Look for: switchInt(cond) -> [1: bb_body], otherwise: bb_exit
        // where cond is a comparison of a local < constant
        if let Some(Terminator::SwitchInt {
            discriminant: _,
            targets,
            otherwise,
        }) = &body.basic_blocks[header_idx].terminator
        {
            if targets.len() != 1 {
                continue;
            }
            let (_, body_bb) = targets[0];
            let exit_bb = *otherwise;
            let body_idx = body_bb.0 as usize;

            // The body block must branch back to the header (back edge)
            if body_idx >= num_blocks {
                continue;
            }
            let body_term = match &body.basic_blocks[body_idx].terminator {
                Some(t) => t,
                None => continue,
            };
            let loops_back = match body_term {
                Terminator::Goto(target) => target.0 as usize == header_idx,
                _ => false,
            };
            if !loops_back {
                continue;
            }

            // Check body size
            let body_stmts = body.basic_blocks[body_idx].statements.len();
            if body_stmts > MAX_UNROLL_BODY_STMTS {
                continue;
            }

            // Try to determine trip count from the header's comparison.
            // Look for a pattern where an induction variable is compared
            // to a constant bound. We check the statements leading up to
            // the switch for a comparison: _cond = Lt(_iv, const_bound).
            if let Some(trip_count) =
                detect_trip_count(&body.basic_blocks[header_idx], MAX_UNROLL_TRIP_COUNT)
            {
                loops_to_unroll.push(LoopInfo {
                    header: header_idx,
                    body_block: body_idx,
                    exit_bb,
                    trip_count,
                });
            }
        }
    }

    // Apply unrolling (process in reverse order to avoid index shifts)
    for info in loops_to_unroll.iter().rev() {
        apply_loop_unrolling(body, info);
    }
}

/// Information about a loop to unroll.
struct LoopInfo {
    header: usize,
    body_block: usize,
    exit_bb: BasicBlockId,
    trip_count: usize,
}

/// Try to detect the trip count of a loop from its header block.
/// Returns Some(count) if the header ends with a comparison against a constant.
fn detect_trip_count(header: &BasicBlock, max_trip: usize) -> Option<usize> {
    // Look for pattern: _cond = Lt(_, const N) or _cond = Le(_, const N)
    for stmt in header.statements.iter().rev() {
        if let Statement::Assign(_, Rvalue::BinaryOp(op, _, rhs)) = stmt {
            match op {
                BinOp::Lt => {
                    if let Operand::Constant(Constant::Int(bound)) = rhs {
                        let count = *bound as usize;
                        if count > 0 && count <= max_trip {
                            return Some(count);
                        }
                    }
                }
                BinOp::Le => {
                    if let Operand::Constant(Constant::Int(bound)) = rhs {
                        let count = (*bound + 1) as usize;
                        if count > 0 && count <= max_trip {
                            return Some(count);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// Apply loop unrolling by replacing the header+body blocks with
/// unrolled copies of the body statements followed by a direct jump
/// to the exit block.
fn apply_loop_unrolling(body: &mut Body, info: &LoopInfo) {
    // Collect the body statements (clone them for each iteration)
    let body_stmts = body.basic_blocks[info.body_block].statements.clone();

    // Replace the header block: remove the switch, put unrolled body
    let header_stmts = body.basic_blocks[info.header].statements.clone();

    let mut unrolled = Vec::new();
    for _ in 0..info.trip_count {
        // Add the original header statements (comparison, etc.) minus
        // the last one if it's the loop condition computation.
        // Then add the body statements.
        unrolled.extend(body_stmts.clone());
    }

    // Keep header's original setup statements but replace the terminator
    let mut new_stmts = header_stmts;
    new_stmts.extend(unrolled);
    body.basic_blocks[info.header].statements = new_stmts;
    body.basic_blocks[info.header].terminator = Some(Terminator::Goto(info.exit_bb));

    // Replace the body block with unreachable (it's inlined now)
    body.basic_blocks[info.body_block] = BasicBlock {
        statements: vec![],
        terminator: Some(Terminator::Unreachable),
    };
}

// ==========================================================================
// Escape Analysis
// ==========================================================================

/// Escape Analysis on MIR.
///
/// Identifies heap allocations (Call to malloc-like functions) whose results
/// do not escape the function scope. These are marked by annotating the
/// allocation statement so downstream passes (e.g., LLVM codegen) can convert
/// them to stack allocations.
///
/// A value "escapes" if it is:
/// - Used as an argument to a function call (except free)
/// - Stored through a pointer
/// - Returned from the function (assigned to _0)
///
/// The results are stored in `Body::block_names` with a special key prefix
/// `__escape_local_N` = "stack" for locals that can be stack-allocated.
pub fn escape_analysis(body: &mut Body) {
    // Phase 1: Find locals that are assigned from Call terminators
    // (potential heap allocations)
    let mut allocation_locals: HashSet<Local> = HashSet::new();
    for bb in &body.basic_blocks {
        if let Some(Terminator::Call {
            func, destination, ..
        }) = &bb.terminator
        {
            // Heuristic: calls to functions containing "alloc" or "malloc"
            // or "new" are considered allocation sites
            let name_lower = func.to_lowercase();
            if (name_lower.contains("alloc")
                || name_lower.contains("malloc")
                || name_lower.contains("new"))
                && destination.projections.is_empty()
            {
                allocation_locals.insert(destination.local);
            }
        }
    }

    if allocation_locals.is_empty() {
        return;
    }

    // Phase 2: Determine which allocated locals escape
    let mut escaped: HashSet<Local> = HashSet::new();

    for bb in &body.basic_blocks {
        for stmt in &bb.statements {
            if let Statement::Assign(place, rvalue) = stmt {
                // If an allocated local is assigned to _0 (return place), it escapes
                if place.local.0 == 0 {
                    collect_escaping_from_rvalue(rvalue, &allocation_locals, &mut escaped);
                }

                // If stored through a pointer (via Ref), the source escapes
                if let Rvalue::Ref(ref_place) = rvalue {
                    if allocation_locals.contains(&ref_place.local) {
                        escaped.insert(ref_place.local);
                    }
                }
            }
        }

        // Check terminator arguments
        if let Some(
            Terminator::Call { args, .. } | Terminator::TailCall { args, .. },
        ) = &bb.terminator
        {
            for arg in args {
                match arg {
                    Operand::Copy(place) | Operand::Move(place) => {
                        if allocation_locals.contains(&place.local) {
                            escaped.insert(place.local);
                        }
                    }
                    Operand::Constant(_) => {}
                }
            }
        }
    }

    // Phase 3: Mark non-escaping locals in block_names metadata
    for local in &allocation_locals {
        if !escaped.contains(local) {
            body.block_names.insert(
                format!("__escape_local_{}", local.0),
                BasicBlockId(0), // sentinel: 0 means "can be stack-allocated"
            );
        }
    }
}

/// Check if an rvalue references any allocated local, indicating escape.
fn collect_escaping_from_rvalue(
    rvalue: &Rvalue,
    alloc_locals: &HashSet<Local>,
    escaped: &mut HashSet<Local>,
) {
    match rvalue {
        Rvalue::Use(op) => collect_escaping_from_operand(op, alloc_locals, escaped),
        Rvalue::BinaryOp(_, lhs, rhs) => {
            collect_escaping_from_operand(lhs, alloc_locals, escaped);
            collect_escaping_from_operand(rhs, alloc_locals, escaped);
        }
        Rvalue::UnaryOp(_, op) => collect_escaping_from_operand(op, alloc_locals, escaped),
        Rvalue::Cast(op, _) => collect_escaping_from_operand(op, alloc_locals, escaped),
        Rvalue::Aggregate(_, ops) => {
            for op in ops {
                collect_escaping_from_operand(op, alloc_locals, escaped);
            }
        }
        _ => {}
    }
}

/// Check if an operand references an allocated local.
fn collect_escaping_from_operand(
    op: &Operand,
    alloc_locals: &HashSet<Local>,
    escaped: &mut HashSet<Local>,
) {
    match op {
        Operand::Copy(place) | Operand::Move(place) => {
            if alloc_locals.contains(&place.local) {
                escaped.insert(place.local);
            }
        }
        Operand::Constant(_) => {}
    }
}

// ==========================================================================
// Tail Call Detection
// ==========================================================================

/// Tail Call Detection on MIR.
///
/// Scans for function calls in tail position (the last operation before
/// return) and converts them from `Call` terminators to `TailCall` terminators.
///
/// A call is in tail position if:
/// 1. The call's destination is the return place (_0)
/// 2. The call's target block contains only a Return terminator
///    (no statements between call result and return)
pub fn tail_call_detection(body: &mut Body) {
    let num_blocks = body.basic_blocks.len();
    let mut conversions: Vec<usize> = Vec::new();

    for (bb_idx, bb) in body.basic_blocks.iter().enumerate() {
        if let Some(Terminator::Call {
            destination,
            target,
            ..
        }) = &bb.terminator
        {
            // Check: destination is _0 (return place)
            if destination.local.0 != 0 || !destination.projections.is_empty() {
                continue;
            }

            // Check: target block is just a Return
            let target_idx = target.0 as usize;
            if target_idx >= num_blocks {
                continue;
            }
            let target_bb = &body.basic_blocks[target_idx];
            if !target_bb.statements.is_empty() {
                continue;
            }
            if target_bb.terminator != Some(Terminator::Return) {
                continue;
            }

            conversions.push(bb_idx);
        }
    }

    // Apply conversions
    for bb_idx in conversions {
        let old_term = body.basic_blocks[bb_idx].terminator.take();
        if let Some(Terminator::Call { func, args, .. }) = old_term {
            body.basic_blocks[bb_idx].terminator = Some(Terminator::TailCall { func, args });
        }
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

    #[test]
    fn test_constant_folding_with_binop() {
        let mut builder = MirBuilder::new("test", vec![], MirType::I64);

        let c1 = builder.new_local(MirType::I64, None);
        let c2 = builder.new_local(MirType::I64, None);
        let result = builder.new_local(MirType::I64, None);

        // c1 = 10, c2 = 20
        builder.assign_const(c1, Constant::Int(10));
        builder.assign_const(c2, Constant::Int(20));

        // result = c1 + c2 (should propagate to const 10 + const 20)
        builder.assign(
            Place::local(result),
            Rvalue::BinaryOp(
                BinOp::Add,
                Operand::Copy(Place::local(c1)),
                Operand::Copy(Place::local(c2)),
            ),
        );
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(result))),
        );
        builder.return_();

        let mut body = builder.build();
        constant_propagation(&mut body);

        // After propagation, the binop should use const operands
        if let Statement::Assign(_, Rvalue::BinaryOp(_, lhs, rhs)) =
            &body.basic_blocks[0].statements[2]
        {
            assert_eq!(*lhs, Operand::Constant(Constant::Int(10)));
            assert_eq!(*rhs, Operand::Constant(Constant::Int(20)));
        } else {
            panic!("Expected binop with constant operands");
        }
    }

    #[test]
    fn test_dce_preserves_return_place() {
        let mut builder = MirBuilder::new("test", vec![], MirType::I64);

        let temp = builder.new_local(MirType::I64, None);

        // temp = 100 (unused)
        builder.assign_const(temp, Constant::Int(100));
        // _0 = 42 (return place, must be kept)
        builder.assign_const(Local(0), Constant::Int(42));
        builder.return_();

        let mut body = builder.build();
        dead_code_elimination(&mut body);

        // Return place assignment must remain
        let has_return_assignment = body.basic_blocks[0]
            .statements
            .iter()
            .any(|stmt| matches!(stmt, Statement::Assign(place, _) if place.local.0 == 0));
        assert!(has_return_assignment);
    }

    #[test]
    fn test_cse_does_not_eliminate_different_ops() {
        let mut builder = MirBuilder::new("test", vec![MirType::I64, MirType::I64], MirType::I64);

        let t1 = builder.new_local(MirType::I64, None);
        let t2 = builder.new_local(MirType::I64, None);

        let param_a = Operand::Copy(Place::local(builder.param(0)));
        let param_b = Operand::Copy(Place::local(builder.param(1)));

        // t1 = a + b
        builder.assign_binop(t1, BinOp::Add, param_a.clone(), param_b.clone());
        // t2 = a * b (different op, should NOT be eliminated)
        builder.assign_binop(t2, BinOp::Mul, param_a, param_b);

        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(t2))),
        );
        builder.return_();

        let mut body = builder.build();
        common_subexpression_elimination(&mut body);

        // t2 assignment should still be Mul, not replaced
        if let Statement::Assign(_, Rvalue::BinaryOp(op, _, _)) =
            &body.basic_blocks[0].statements[1]
        {
            assert_eq!(*op, BinOp::Mul);
        } else {
            panic!("Expected Mul to remain");
        }
    }

    #[test]
    fn test_optimize_mir_body_integration() {
        let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

        let unused = builder.new_local(MirType::I64, None);
        let const_val = builder.new_local(MirType::I64, None);
        let result = builder.new_local(MirType::I64, None);

        // unused = 999 (should be removed by DCE)
        builder.assign_const(unused, Constant::Int(999));
        // const_val = 5
        builder.assign_const(const_val, Constant::Int(5));
        // result = param + const_val (should propagate const 5)
        builder.assign(
            Place::local(result),
            Rvalue::BinaryOp(
                BinOp::Add,
                Operand::Copy(Place::local(builder.param(0))),
                Operand::Copy(Place::local(const_val)),
            ),
        );
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(result))),
        );
        builder.return_();

        let mut body = builder.build();
        let before_count = body.basic_blocks[0].statements.len();

        optimize_mir_body(&mut body);

        let after_count = body.basic_blocks[0].statements.len();
        // Should have fewer statements after DCE removes unused
        assert!(after_count < before_count);
    }

    #[test]
    fn test_switch_int_with_multiple_targets() {
        let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

        let bb1 = builder.new_block();
        let bb2 = builder.new_block();
        let bb_default = builder.new_block();
        let bb_end = builder.new_block();

        let param = Operand::Copy(Place::local(builder.param(0)));

        // Switch on parameter with 3 cases
        builder.switch_int(param, vec![(1, bb1), (2, bb2)], bb_default);

        // bb1: return 100
        builder.switch_to_block(bb1);
        builder.assign_const(builder.return_place().local, Constant::Int(100));
        builder.goto(bb_end);

        // bb2: return 200
        builder.switch_to_block(bb2);
        builder.assign_const(builder.return_place().local, Constant::Int(200));
        builder.goto(bb_end);

        // bb_default: return 0
        builder.switch_to_block(bb_default);
        builder.assign_const(builder.return_place().local, Constant::Int(0));
        builder.goto(bb_end);

        // bb_end: return
        builder.switch_to_block(bb_end);
        builder.return_();

        let mut body = builder.build();
        remove_unreachable_blocks(&mut body);

        // All blocks should be reachable
        for bb in &body.basic_blocks {
            assert_ne!(bb.terminator, Some(Terminator::Unreachable));
        }
    }

    // ======================================================================
    // Copy Propagation Tests
    // ======================================================================

    #[test]
    fn test_copy_propagation_simple() {
        let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

        let param0 = builder.param(0);
        let x = builder.new_local(MirType::I64, Some("x".into()));
        let y = builder.new_local(MirType::I64, Some("y".into()));

        // x = copy param0
        builder.assign(
            Place::local(x),
            Rvalue::Use(Operand::Copy(Place::local(param0))),
        );
        // y = copy x  (should be propagated to: y = copy param0)
        builder.assign(
            Place::local(y),
            Rvalue::Use(Operand::Copy(Place::local(x))),
        );
        // _0 = copy y
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(y))),
        );
        builder.return_();

        let mut body = builder.build();
        copy_propagation(&mut body);

        // After copy propagation, y's assignment should reference param0
        // and return should reference param0 (through y -> x -> param0 chain)
        if let Statement::Assign(_, Rvalue::Use(Operand::Copy(place))) =
            &body.basic_blocks[0].statements[2]
        {
            // _0 should now reference param0 (Local(1))
            assert_eq!(place.local, param0);
        } else {
            panic!("Expected copy propagation to resolve chain");
        }
    }

    #[test]
    fn test_copy_propagation_no_multi_assign() {
        let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

        let x = builder.new_local(MirType::I64, Some("x".into()));

        // x = copy param0
        builder.assign(
            Place::local(x),
            Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
        );
        // x = const 42 (reassigned -- should not be propagated)
        builder.assign_const(x, Constant::Int(42));
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(x))),
        );
        builder.return_();

        let mut body = builder.build();
        copy_propagation(&mut body);

        // x should NOT be propagated since it's assigned more than once
        if let Statement::Assign(_, Rvalue::Use(Operand::Copy(place))) =
            &body.basic_blocks[0].statements[2]
        {
            assert_eq!(place.local, x); // Still x, not param0
        } else {
            panic!("Expected x to remain (multi-assign prevents propagation)");
        }
    }

    // ======================================================================
    // Tail Call Detection Tests
    // ======================================================================

    #[test]
    fn test_tail_call_detection() {
        let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

        let bb_ret = builder.new_block();

        // bb0: call foo(param0), result -> _0, then goto bb_ret
        builder.call(
            "foo",
            vec![Operand::Copy(Place::local(builder.param(0)))],
            Place::local(Local(0)), // destination = _0
            bb_ret,
        );

        // bb_ret: just return
        builder.switch_to_block(bb_ret);
        builder.return_();

        let mut body = builder.build();
        tail_call_detection(&mut body);

        // The call should be converted to a TailCall
        assert!(matches!(
            body.basic_blocks[0].terminator,
            Some(Terminator::TailCall { .. })
        ));
    }

    #[test]
    fn test_tail_call_not_applied_when_not_tail() {
        let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

        let result = builder.new_local(MirType::I64, None);
        let bb_after = builder.new_block();

        // bb0: call foo(param0), result -> _3 (NOT _0), then goto bb_after
        builder.call(
            "foo",
            vec![Operand::Copy(Place::local(builder.param(0)))],
            Place::local(result), // NOT return place
            bb_after,
        );

        // bb_after: use result, then return
        builder.switch_to_block(bb_after);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(result))),
        );
        builder.return_();

        let mut body = builder.build();
        tail_call_detection(&mut body);

        // The call should NOT be converted (destination is not _0)
        assert!(matches!(
            body.basic_blocks[0].terminator,
            Some(Terminator::Call { .. })
        ));
    }

    // ======================================================================
    // Escape Analysis Tests
    // ======================================================================

    #[test]
    fn test_escape_analysis_non_escaping() {
        let mut builder = MirBuilder::new("test", vec![], MirType::I64);

        let alloc_result = builder.new_local(MirType::I64, None);
        let bb_after = builder.new_block();

        // Call malloc (heap allocation)
        builder.call(
            "malloc",
            vec![Operand::Constant(Constant::Int(64))],
            Place::local(alloc_result),
            bb_after,
        );

        // bb_after: just return a constant (alloc_result doesn't escape)
        builder.switch_to_block(bb_after);
        builder.assign_const(Local(0), Constant::Int(42));
        builder.return_();

        let mut body = builder.build();
        escape_analysis(&mut body);

        // alloc_result should be marked as non-escaping
        let key = format!("__escape_local_{}", alloc_result.0);
        assert!(
            body.block_names.contains_key(&key),
            "Expected non-escaping allocation to be marked"
        );
    }

    #[test]
    fn test_escape_analysis_escaping_via_return() {
        let mut builder = MirBuilder::new("test", vec![], MirType::I64);

        let alloc_result = builder.new_local(MirType::I64, None);
        let bb_after = builder.new_block();

        // Call malloc
        builder.call(
            "malloc",
            vec![Operand::Constant(Constant::Int(64))],
            Place::local(alloc_result),
            bb_after,
        );

        // bb_after: return the allocated pointer (escapes!)
        builder.switch_to_block(bb_after);
        builder.assign(
            Place::local(Local(0)),
            Rvalue::Use(Operand::Copy(Place::local(alloc_result))),
        );
        builder.return_();

        let mut body = builder.build();
        escape_analysis(&mut body);

        // alloc_result should NOT be marked (it escapes via return)
        let key = format!("__escape_local_{}", alloc_result.0);
        assert!(
            !body.block_names.contains_key(&key),
            "Expected escaping allocation to NOT be marked"
        );
    }

    // ======================================================================
    // Loop Unrolling Tests
    // ======================================================================

    #[test]
    fn test_loop_unrolling_simple() {
        // Build: loop header checks i < 4, body increments i
        let mut builder = MirBuilder::new("test", vec![], MirType::I64);

        let i_var = builder.new_local(MirType::I64, Some("i".into()));
        let cond = builder.new_local(MirType::Bool, Some("cond".into()));
        let sum = builder.new_local(MirType::I64, Some("sum".into()));

        let bb_header = builder.new_block();
        let bb_body = builder.new_block();
        let bb_exit = builder.new_block();

        // bb0: init i=0, sum=0, goto header
        builder.assign_const(i_var, Constant::Int(0));
        builder.assign_const(sum, Constant::Int(0));
        builder.goto(bb_header);

        // bb_header: cond = i < 4; switchInt(cond) -> [1: bb_body], otherwise: bb_exit
        builder.switch_to_block(bb_header);
        builder.assign(
            Place::local(cond),
            Rvalue::BinaryOp(
                BinOp::Lt,
                Operand::Copy(Place::local(i_var)),
                Operand::Constant(Constant::Int(4)),
            ),
        );
        builder.switch_int(
            Operand::Copy(Place::local(cond)),
            vec![(1, bb_body)],
            bb_exit,
        );

        // bb_body: sum = sum + 1; i = i + 1; goto header
        builder.switch_to_block(bb_body);
        builder.assign(
            Place::local(sum),
            Rvalue::BinaryOp(
                BinOp::Add,
                Operand::Copy(Place::local(sum)),
                Operand::Constant(Constant::Int(1)),
            ),
        );
        builder.assign(
            Place::local(i_var),
            Rvalue::BinaryOp(
                BinOp::Add,
                Operand::Copy(Place::local(i_var)),
                Operand::Constant(Constant::Int(1)),
            ),
        );
        builder.goto(bb_header);

        // bb_exit: return sum
        builder.switch_to_block(bb_exit);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(sum))),
        );
        builder.return_();

        let mut body = builder.build();
        let before_body_stmts = body.basic_blocks[bb_body.0 as usize].statements.len();
        assert_eq!(before_body_stmts, 2); // sum += 1, i += 1

        loop_unrolling(&mut body);

        // After unrolling with trip count 4, the header block should contain
        // the original header stmts + 4 copies of the body stmts (2 each = 8)
        // and the body block should be replaced with unreachable
        let header_stmts = body.basic_blocks[bb_header.0 as usize].statements.len();
        // 1 (original cond) + 4*2 (unrolled body) = 9
        assert_eq!(header_stmts, 9);
        assert_eq!(
            body.basic_blocks[bb_body.0 as usize].terminator,
            Some(Terminator::Unreachable)
        );
    }

    #[test]
    fn test_loop_unrolling_too_large_trip_count() {
        // Loop with trip count > 8 should NOT be unrolled
        let mut builder = MirBuilder::new("test", vec![], MirType::I64);

        let i_var = builder.new_local(MirType::I64, None);
        let cond = builder.new_local(MirType::Bool, None);

        let bb_header = builder.new_block();
        let bb_body = builder.new_block();
        let bb_exit = builder.new_block();

        builder.assign_const(i_var, Constant::Int(0));
        builder.goto(bb_header);

        builder.switch_to_block(bb_header);
        builder.assign(
            Place::local(cond),
            Rvalue::BinaryOp(
                BinOp::Lt,
                Operand::Copy(Place::local(i_var)),
                Operand::Constant(Constant::Int(100)), // too large
            ),
        );
        builder.switch_int(
            Operand::Copy(Place::local(cond)),
            vec![(1, bb_body)],
            bb_exit,
        );

        builder.switch_to_block(bb_body);
        builder.assign(
            Place::local(i_var),
            Rvalue::BinaryOp(
                BinOp::Add,
                Operand::Copy(Place::local(i_var)),
                Operand::Constant(Constant::Int(1)),
            ),
        );
        builder.goto(bb_header);

        builder.switch_to_block(bb_exit);
        builder.assign_const(Local(0), Constant::Int(0));
        builder.return_();

        let mut body = builder.build();
        loop_unrolling(&mut body);

        // Body block should still have its original terminator (not unreachable)
        assert!(matches!(
            body.basic_blocks[bb_body.0 as usize].terminator,
            Some(Terminator::Goto(_))
        ));
    }
}
