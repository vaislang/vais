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
        if let Some(Terminator::Call { args, .. } | Terminator::TailCall { args, .. }) =
            &bb.terminator
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
#[path = "optimize_tests.rs"]
mod optimize_tests;
