//! BorrowChecker implementation - the main dataflow analysis engine.

use super::*;
use super::cfg::{compute_liveness, cfg_predecessors, cfg_successors};

/// The main borrow checker structure.
pub struct BorrowChecker<'a> {
    body: &'a Body,
    block_states: HashMap<BasicBlockId, BlockState>,
    local_states: HashMap<Local, LocalState>,
    active_borrows: HashMap<Local, Vec<BorrowInfo>>,
    errors: Vec<BorrowError>,
    liveness: LivenessInfo,
}

impl<'a> BorrowChecker<'a> {
    pub(super) fn new(body: &'a Body) -> Self {
        let mut local_states = HashMap::new();

        // Initialize all locals as uninitialized, except:
        // - _0 (return place) starts as Owned (will be written before return)
        // - Parameters (_1 .. _1+params.len()) start as Owned
        local_states.insert(Local(0), LocalState::Uninitialized);
        for i in 1..=body.params.len() {
            local_states.insert(
                Local(i as u32),
                LocalState::Owned(Location::new(BasicBlockId(0), 0)),
            );
        }

        // All other locals start uninitialized
        for i in (body.params.len() + 1)..body.locals.len() {
            local_states.insert(Local(i as u32), LocalState::Uninitialized);
        }

        // Compute liveness information
        let liveness = compute_liveness(body);

        Self {
            body,
            block_states: HashMap::new(),
            local_states,
            active_borrows: HashMap::new(),
            errors: vec![],
            liveness,
        }
    }

    /// Initialize the entry state for the entry block (bb0).
    fn initial_states(&self) -> (HashMap<Local, LocalState>, HashMap<Local, Vec<BorrowInfo>>) {
        let mut states = HashMap::new();
        states.insert(Local(0), LocalState::Uninitialized);
        for i in 1..=self.body.params.len() {
            states.insert(
                Local(i as u32),
                LocalState::Owned(Location::new(BasicBlockId(0), 0)),
            );
        }
        for i in (self.body.params.len() + 1)..self.body.locals.len() {
            states.insert(Local(i as u32), LocalState::Uninitialized);
        }
        (states, HashMap::new())
    }

    /// Get the type of a local.
    fn local_type(&self, local: Local) -> &MirType {
        &self.body.locals[local.0 as usize].ty
    }

    /// Check if a local's type is Copy.
    fn is_copy(&self, local: Local) -> bool {
        self.local_type(local).is_copy()
    }

    /// Expire borrows whose borrow_target is no longer live.
    fn expire_borrows(&mut self, current_location: Location) {
        // Remove borrows whose borrow_target's last use is before the current location
        for borrows in self.active_borrows.values_mut() {
            borrows.retain(|borrow| {
                if let Some(target) = borrow.borrow_target {
                    if let Some(&last_use) = self.liveness.last_use.get(&target) {
                        // Keep the borrow if its target is used at or after the current location
                        // Compare by block ID first, then statement index
                        if last_use.block.0 < current_location.block.0 {
                            return false; // Expired - target was last used in an earlier block
                        } else if last_use.block.0 == current_location.block.0
                            && last_use.statement_index < current_location.statement_index
                        {
                            return false; // Expired - target was last used earlier in this block
                        }
                    }
                    // If target is not in liveness map, keep the borrow (it might never be used,
                    // but that doesn't mean the borrow is expired - it could still conflict)
                }
                true // Keep the borrow
            });
        }

        // Remove empty entries
        self.active_borrows.retain(|_, borrows| !borrows.is_empty());
    }

    /// Activate reserved mutable borrows when their borrow_target is used.
    fn activate_reserved_borrows(&mut self, used_local: Local, location: Location) {
        // First pass: identify which borrows need activation and check for conflicts
        let mut activations = Vec::new();

        for (borrowed_local, borrows) in &self.active_borrows {
            for (idx, borrow) in borrows.iter().enumerate() {
                if borrow.kind == BorrowKind::ReservedMutable
                    && borrow.borrow_target == Some(used_local)
                {
                    // Check for conflicts with other borrows of the same local
                    let has_conflict = borrows.iter().any(|other| {
                        other.location != borrow.location
                            && (other.kind == BorrowKind::Shared
                                || other.kind == BorrowKind::Mutable)
                    });

                    activations.push((*borrowed_local, idx, has_conflict, borrow.location));
                }
            }
        }

        // Second pass: apply activations and report conflicts
        for (borrowed_local, idx, has_conflict, borrow_location) in activations {
            if has_conflict {
                self.errors.push(BorrowError::MutableBorrowConflict {
                    local: borrowed_local,
                    first_borrow: borrow_location,
                    second_borrow: location,
                });
            }

            // Activate the borrow
            if let Some(borrows) = self.active_borrows.get_mut(&borrowed_local) {
                if let Some(borrow) = borrows.get_mut(idx) {
                    borrow.kind = BorrowKind::Mutable;
                }
            }
        }
    }

    /// Check if a place can be used (read) at the given location.
    fn check_use(&mut self, place: &Place, location: Location) {
        let local = place.local;

        // Copy types can always be used
        if self.is_copy(local) {
            return;
        }

        let state = self
            .local_states
            .get(&local)
            .cloned()
            .unwrap_or(LocalState::Uninitialized);

        match state {
            LocalState::Moved(moved_at) => {
                self.errors.push(BorrowError::UseAfterMove {
                    local,
                    moved_at,
                    used_at: location,
                });
            }
            LocalState::Dropped(freed_at) => {
                self.errors.push(BorrowError::UseAfterFree {
                    local,
                    freed_at,
                    used_at: location,
                });
            }
            LocalState::Owned(_) | LocalState::Uninitialized => {
                // OK to use
            }
        }
    }

    /// Record a move of a place at the given location.
    fn record_move(&mut self, place: &Place, location: Location) {
        let local = place.local;

        // Copy types don't move
        if self.is_copy(local) {
            return;
        }

        // Check for active borrows
        if let Some(borrows) = self.active_borrows.get(&local) {
            if let Some(first_borrow) = borrows.first() {
                self.errors.push(BorrowError::MoveWhileBorrowed {
                    local,
                    borrow_at: first_borrow.location,
                    move_at: location,
                });
            }
        }

        // Check current state
        let state = self
            .local_states
            .get(&local)
            .cloned()
            .unwrap_or(LocalState::Uninitialized);

        match state {
            LocalState::Dropped(freed_at) => {
                self.errors.push(BorrowError::UseAfterFree {
                    local,
                    freed_at,
                    used_at: location,
                });
            }
            LocalState::Owned(_) | LocalState::Uninitialized => {
                // Record the move
                self.local_states.insert(local, LocalState::Moved(location));
            }
            LocalState::Moved(moved_at) => {
                // Already moved - report use-after-move
                self.errors.push(BorrowError::UseAfterMove {
                    local,
                    moved_at,
                    used_at: location,
                });
            }
        }
    }

    /// Record an assignment to a place.
    fn record_assign(&mut self, place: &Place, location: Location) {
        let local = place.local;

        // Assignment reinitializes the local (makes it Owned again)
        self.local_states.insert(local, LocalState::Owned(location));

        // Clear any active borrows (assignment invalidates them)
        self.active_borrows.remove(&local);
    }

    /// Record a borrow of a place.
    fn record_borrow(
        &mut self,
        place: &Place,
        kind: BorrowKind,
        location: Location,
        borrow_target: Option<Local>,
    ) {
        let local = place.local;

        // Check for existing borrows
        let existing = self.active_borrows.entry(local).or_default();

        for borrow in existing.iter() {
            match (borrow.kind, kind) {
                (BorrowKind::Mutable, BorrowKind::Mutable) => {
                    self.errors.push(BorrowError::MutableBorrowConflict {
                        local,
                        first_borrow: borrow.location,
                        second_borrow: location,
                    });
                }
                (BorrowKind::Mutable, BorrowKind::Shared) => {
                    self.errors.push(BorrowError::BorrowWhileMutablyBorrowed {
                        local,
                        mutable_borrow: borrow.location,
                        shared_borrow: location,
                    });
                }
                (BorrowKind::Shared, BorrowKind::Mutable) => {
                    self.errors.push(BorrowError::BorrowWhileMutablyBorrowed {
                        local,
                        mutable_borrow: location,
                        shared_borrow: borrow.location,
                    });
                }
                (BorrowKind::Shared, BorrowKind::Shared) => {
                    // Multiple shared borrows are OK
                }
                // Two-phase borrow interactions
                (BorrowKind::ReservedMutable, BorrowKind::Shared)
                | (BorrowKind::Shared, BorrowKind::ReservedMutable) => {
                    // Reserved mutable + shared is OK (not yet activated)
                }
                (BorrowKind::ReservedMutable, BorrowKind::ReservedMutable) => {
                    // Multiple reserved mutables are OK (not yet activated)
                }
                (BorrowKind::ReservedMutable, BorrowKind::Mutable) => {
                    // Reserved + active mutable → conflict
                    self.errors.push(BorrowError::MutableBorrowConflict {
                        local,
                        first_borrow: borrow.location,
                        second_borrow: location,
                    });
                }
                (BorrowKind::Mutable, BorrowKind::ReservedMutable) => {
                    // Active mutable + reserved → conflict
                    self.errors.push(BorrowError::MutableBorrowConflict {
                        local,
                        first_borrow: borrow.location,
                        second_borrow: location,
                    });
                }
            }
        }

        existing.push(BorrowInfo {
            kind,
            location,
            borrowed_local: local,
            borrow_target,
        });
    }

    /// Record a drop of a place.
    fn record_drop(&mut self, place: &Place, location: Location) {
        let local = place.local;

        // Copy types don't need explicit drops (no-op)
        if self.is_copy(local) {
            return;
        }

        let state = self
            .local_states
            .get(&local)
            .cloned()
            .unwrap_or(LocalState::Uninitialized);

        match state {
            LocalState::Dropped(first_drop) => {
                self.errors.push(BorrowError::DoubleFree {
                    local,
                    first_drop,
                    second_drop: location,
                });
            }
            LocalState::Moved(_) => {
                // No error: dropping an already-moved value is a no-op
                // (the value was already moved, so drop has nothing to do)
            }
            LocalState::Owned(_) | LocalState::Uninitialized => {
                self.local_states
                    .insert(local, LocalState::Dropped(location));
            }
        }
    }

    /// Check an operand at the given location.
    fn check_operand(&mut self, operand: &Operand, location: Location) {
        match operand {
            Operand::Copy(place) => {
                self.check_use(place, location);
                // Activate any reserved borrows targeting this local
                self.activate_reserved_borrows(place.local, location);
            }
            Operand::Move(place) => {
                self.check_use(place, location);
                // Activate any reserved borrows targeting this local
                self.activate_reserved_borrows(place.local, location);
                self.record_move(place, location);
            }
            Operand::Constant(_) => {
                // Constants are always OK
            }
        }
    }

    /// Check an rvalue at the given location.
    fn check_rvalue(&mut self, rvalue: &Rvalue, location: Location, assign_target: Option<Local>) {
        match rvalue {
            Rvalue::Use(operand) => {
                self.check_operand(operand, location);
            }
            Rvalue::BinaryOp(_, left, right) => {
                self.check_operand(left, location);
                self.check_operand(right, location);
            }
            Rvalue::UnaryOp(_, operand) => {
                self.check_operand(operand, location);
            }
            Rvalue::Ref(place) => {
                self.check_use(place, location);
                // Determine borrow kind based on local mutability
                let kind = if place.local.0 < self.body.locals.len() as u32
                    && self.body.locals[place.local.0 as usize].is_mutable
                {
                    // For now, use regular Mutable borrows.
                    // Two-phase borrows (ReservedMutable) can be enabled later for specific patterns.
                    BorrowKind::Mutable
                } else {
                    BorrowKind::Shared
                };
                self.record_borrow(place, kind, location, assign_target);
            }
            Rvalue::Aggregate(_, operands) => {
                for operand in operands {
                    self.check_operand(operand, location);
                }
            }
            Rvalue::Discriminant(place) => {
                self.check_use(place, location);
            }
            Rvalue::Cast(operand, _) => {
                self.check_operand(operand, location);
            }
            Rvalue::Len(place) => {
                self.check_use(place, location);
            }
        }
    }

    /// Check a statement at the given location.
    fn check_statement(&mut self, statement: &Statement, location: Location) {
        match statement {
            Statement::Assign(place, rvalue) => {
                // Check the rvalue first (reads on RHS), passing the assign target
                self.check_rvalue(rvalue, location, Some(place.local));
                // Then record the assignment (write on LHS)
                self.record_assign(place, location);
            }
            Statement::Drop(place) => {
                self.record_drop(place, location);
            }
            Statement::Nop => {
                // No-op
            }
        }
    }

    /// Check a terminator at the given location.
    fn check_terminator(&mut self, terminator: &Terminator, location: Location) {
        match terminator {
            Terminator::Goto(_) => {
                // No operands to check
            }
            Terminator::SwitchInt { discriminant, .. } => {
                self.check_operand(discriminant, location);
            }
            Terminator::Return => {
                // Check that the return place (_0) is properly initialized
                let return_local = Local(0);
                if !self.is_copy(return_local) {
                    let state = self
                        .local_states
                        .get(&return_local)
                        .cloned()
                        .unwrap_or(LocalState::Uninitialized);
                    match state {
                        LocalState::Moved(moved_at) => {
                            self.errors.push(BorrowError::UseAfterMove {
                                local: return_local,
                                moved_at,
                                used_at: location,
                            });
                        }
                        LocalState::Dropped(freed_at) => {
                            self.errors.push(BorrowError::UseAfterFree {
                                local: return_local,
                                freed_at,
                                used_at: location,
                            });
                        }
                        LocalState::Owned(_) | LocalState::Uninitialized => {
                            // OK
                        }
                    }
                }
            }
            Terminator::Call { args, .. } => {
                for arg in args {
                    self.check_operand(arg, location);
                }
            }
            Terminator::TailCall { args, .. } => {
                for arg in args {
                    self.check_operand(arg, location);
                }
            }
            Terminator::Unreachable => {
                // No checks needed
            }
            Terminator::Assert { cond, .. } => {
                self.check_operand(cond, location);
            }
        }
    }

    /// Join (merge) two local states conservatively.
    /// Conservative means: choose the "worse" state (moved/dropped over owned).
    fn join_local_state(a: &LocalState, b: &LocalState) -> LocalState {
        use LocalState::*;
        match (a, b) {
            // Same states remain the same
            (Uninitialized, Uninitialized) => Uninitialized,
            (Owned(loc), Owned(_)) => Owned(*loc), // Keep first location
            (Moved(loc), Moved(_)) => Moved(*loc),
            (Dropped(loc), Dropped(_)) => Dropped(*loc),

            // Conservative: if one is moved, result is moved
            (Moved(loc), _) | (_, Moved(loc)) => Moved(*loc),

            // Conservative: if one is dropped, result is dropped
            (Dropped(loc), _) | (_, Dropped(loc)) => Dropped(*loc),

            // Conservative: uninitialized "wins" over owned
            (Uninitialized, _) | (_, Uninitialized) => Uninitialized,
        }
    }

    /// Join states from multiple predecessors.
    fn join_states(
        predecessors: &[BasicBlockId],
        block_states: &HashMap<BasicBlockId, BlockState>,
        initial_states: &HashMap<Local, LocalState>,
        initial_borrows: &HashMap<Local, Vec<BorrowInfo>>,
    ) -> (HashMap<Local, LocalState>, HashMap<Local, Vec<BorrowInfo>>) {
        if predecessors.is_empty() {
            // Entry block: use initial states
            return (initial_states.clone(), initial_borrows.clone());
        }

        // Collect all exit states from predecessors
        let pred_states: Vec<_> = predecessors
            .iter()
            .filter_map(|pred_id| block_states.get(pred_id))
            .collect();

        if pred_states.is_empty() {
            // No predecessor data yet, use initial states
            return (initial_states.clone(), initial_borrows.clone());
        }

        // Merge local states
        let mut merged_states = HashMap::new();

        // Collect all locals mentioned in any predecessor
        let mut all_locals = std::collections::HashSet::new();
        for state in &pred_states {
            all_locals.extend(state.exit_states.keys().copied());
        }
        for local in initial_states.keys() {
            all_locals.insert(*local);
        }

        for local in all_locals {
            let first_state = pred_states[0]
                .exit_states
                .get(&local)
                .cloned()
                .unwrap_or(LocalState::Uninitialized);

            let merged = pred_states.iter().skip(1).fold(first_state, |acc, state| {
                let next = state
                    .exit_states
                    .get(&local)
                    .cloned()
                    .unwrap_or(LocalState::Uninitialized);
                Self::join_local_state(&acc, &next)
            });

            merged_states.insert(local, merged);
        }

        // Merge borrows: take union (conservative)
        let mut merged_borrows: HashMap<Local, Vec<BorrowInfo>> = HashMap::new();
        for state in &pred_states {
            for (local, borrows) in &state.exit_borrows {
                merged_borrows
                    .entry(*local)
                    .or_default()
                    .extend(borrows.iter().cloned());
            }
        }

        // Deduplicate borrows (same local + same location)
        for borrows in merged_borrows.values_mut() {
            borrows.sort_by_key(|b| (b.location.block.0, b.location.statement_index));
            borrows.dedup();
        }

        (merged_states, merged_borrows)
    }

    /// Analyze a single block, returning its exit state and any errors found.
    #[allow(clippy::type_complexity)]
    fn analyze_block(
        &mut self,
        block: &BasicBlock,
        block_id: BasicBlockId,
        entry_states: &HashMap<Local, LocalState>,
        entry_borrows: &HashMap<Local, Vec<BorrowInfo>>,
    ) -> (
        HashMap<Local, LocalState>,
        HashMap<Local, Vec<BorrowInfo>>,
        Vec<BorrowError>,
    ) {
        // Set up temporary state for this block
        self.local_states = entry_states.clone();
        self.active_borrows = entry_borrows.clone();
        let mut block_errors = Vec::new();

        // Expire borrows at block entry
        let entry_location = Location::new(block_id, 0);
        self.expire_borrows(entry_location);

        // Check each statement
        for (stmt_idx, statement) in block.statements.iter().enumerate() {
            let location = Location::new(block_id, stmt_idx);
            // Expire borrows before each statement
            self.expire_borrows(location);
            self.check_statement(statement, location);
        }

        // Check terminator
        if let Some(ref terminator) = block.terminator {
            let location = Location::new(block_id, block.statements.len());
            // Expire borrows before terminator
            self.expire_borrows(location);
            self.check_terminator(terminator, location);
        }

        // Collect errors generated in this block
        block_errors.append(&mut self.errors);

        // Return exit state
        (
            self.local_states.clone(),
            self.active_borrows.clone(),
            block_errors,
        )
    }

    /// Run the borrow checker on the body using worklist-based CFG analysis.
    pub(super) fn check(&mut self) -> Vec<BorrowError> {
        let predecessors = cfg_predecessors(self.body);
        let successors = cfg_successors(self.body);
        let (initial_states, initial_borrows) = self.initial_states();

        // Worklist: blocks that need to be (re-)analyzed
        let mut worklist: Vec<BasicBlockId> = vec![BasicBlockId(0)];
        let mut worklist_set: std::collections::HashSet<BasicBlockId> =
            std::collections::HashSet::from([BasicBlockId(0)]);

        // Maximum iterations to prevent infinite loops
        let max_iterations = self.body.basic_blocks.len() * 4;
        let mut iterations = 0;

        // Worklist algorithm
        while let Some(block_id) = worklist.pop() {
            worklist_set.remove(&block_id);
            iterations += 1;

            if iterations > max_iterations {
                // Safety: prevent infinite loops
                break;
            }

            let block = &self.body.basic_blocks[block_id.0 as usize];

            // Compute entry state by joining predecessors
            let preds = predecessors
                .get(&block_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            let (entry_states, entry_borrows) =
                Self::join_states(preds, &self.block_states, &initial_states, &initial_borrows);

            // Check if entry state changed
            let state_changed = if let Some(existing) = self.block_states.get(&block_id) {
                existing.entry_states != entry_states || existing.entry_borrows != entry_borrows
            } else {
                true
            };

            if !state_changed && self.block_states.contains_key(&block_id) {
                // No change, skip re-analysis
                continue;
            }

            // Analyze the block to compute exit state
            let (exit_states, exit_borrows, _block_errors) =
                self.analyze_block(block, block_id, &entry_states, &entry_borrows);

            // Update block state
            let new_state = BlockState {
                entry_states: entry_states.clone(),
                exit_states: exit_states.clone(),
                entry_borrows: entry_borrows.clone(),
                exit_borrows: exit_borrows.clone(),
            };

            // Check if exit state changed
            let exit_changed = if let Some(existing) = self.block_states.get(&block_id) {
                existing.exit_states != new_state.exit_states
                    || existing.exit_borrows != new_state.exit_borrows
            } else {
                true
            };

            self.block_states.insert(block_id, new_state);

            // If exit state changed, add successors to worklist
            if exit_changed {
                if let Some(succs) = successors.get(&block_id) {
                    for succ in succs {
                        if worklist_set.insert(*succ) {
                            worklist.push(*succ);
                        }
                    }
                }
            }
        }

        // Final pass: collect all errors after reaching fixpoint
        let mut all_errors = Vec::new();
        for (block_idx, block) in self.body.basic_blocks.iter().enumerate() {
            let block_id = BasicBlockId(block_idx as u32);

            if let Some(block_state) = self.block_states.get(&block_id).cloned() {
                let (_exit_states, _exit_borrows, block_errors) = self.analyze_block(
                    block,
                    block_id,
                    &block_state.entry_states,
                    &block_state.entry_borrows,
                );
                all_errors.extend(block_errors);
            }
        }

        // Check lifetime constraints
        let lifetime_errors = self.check_lifetime_constraints();
        all_errors.extend(lifetime_errors);

        all_errors
    }

    /// Check lifetime outlives constraints.
    fn check_lifetime_constraints(&self) -> Vec<BorrowError> {
        let mut errors = Vec::new();

        // Build transitive closure of lifetime bounds
        let outlives_map = self.build_outlives_map();

        // Check each bound
        for (shorter, longer_lifetimes) in &self.body.lifetime_bounds {
            for longer in longer_lifetimes {
                // Check if shorter outlives longer (transitively)
                if !self.lifetime_outlives(shorter, longer, &outlives_map) {
                    errors.push(BorrowError::LifetimeViolation {
                        shorter: shorter.clone(),
                        longer: longer.clone(),
                        location: Location::new(BasicBlockId(0), 0),
                    });
                }
            }
        }

        errors
    }

    /// Build a map of lifetime outlives relationships.
    fn build_outlives_map(&self) -> HashMap<String, std::collections::HashSet<String>> {
        let mut map: HashMap<String, std::collections::HashSet<String>> = HashMap::new();

        // Initialize direct relationships from bounds
        for (shorter, longer_list) in &self.body.lifetime_bounds {
            let entry = map.entry(shorter.clone()).or_default();
            for longer in longer_list {
                entry.insert(longer.clone());
            }
        }

        // Compute transitive closure using Floyd-Warshall-like algorithm
        let lifetimes: Vec<String> = map.keys().cloned().collect();
        for k in &lifetimes {
            for i in &lifetimes {
                if let Some(i_set) = map.get(i).cloned() {
                    if i_set.contains(k) {
                        // i outlives k, so i should outlive everything k outlives
                        if let Some(k_set) = map.get(k).cloned() {
                            let entry = map.entry(i.clone()).or_default();
                            for j in k_set {
                                entry.insert(j);
                            }
                        }
                    }
                }
            }
        }

        map
    }

    /// Check if lifetime `shorter` outlives `longer` (transitively).
    fn lifetime_outlives(
        &self,
        shorter: &str,
        longer: &str,
        outlives_map: &HashMap<String, std::collections::HashSet<String>>,
    ) -> bool {
        if shorter == longer {
            return true;
        }
        if let Some(set) = outlives_map.get(shorter) {
            set.contains(longer)
        } else {
            false
        }
    }
}
