//! MIR builder: constructs MIR bodies from a high-level description.

use crate::types::*;

/// Builder for constructing MIR function bodies incrementally.
pub struct MirBuilder {
    name: String,
    params: Vec<MirType>,
    return_type: MirType,
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlock>,
    current_block: usize,
    next_local: u32,
}

impl MirBuilder {
    /// Create a new MIR builder for a function.
    pub fn new(name: impl Into<String>, params: Vec<MirType>, return_type: MirType) -> Self {
        let mut locals = vec![];
        // _0: return place
        locals.push(LocalDecl {
            name: Some("_return".into()),
            ty: return_type.clone(),
            is_mutable: true,
            lifetime: None,
        });
        // _1.._n: parameters
        let param_count = params.len() as u32;
        for (i, ty) in params.iter().enumerate() {
            locals.push(LocalDecl {
                name: Some(format!("_arg{}", i)),
                ty: ty.clone(),
                is_mutable: false,
                lifetime: None,
            });
        }

        let mut builder = Self {
            name: name.into(),
            params,
            return_type,
            locals,
            blocks: vec![],
            current_block: 0,
            next_local: 1 + param_count,
        };

        // Create entry block (bb0)
        builder.new_block();
        builder
    }

    /// Create a new basic block and return its ID.
    pub fn new_block(&mut self) -> BasicBlockId {
        let id = BasicBlockId(self.blocks.len() as u32);
        self.blocks.push(BasicBlock::new());
        id
    }

    /// Switch to building a different basic block.
    pub fn switch_to_block(&mut self, block: BasicBlockId) {
        self.current_block = block.0 as usize;
    }

    /// Allocate a new local variable (temporary).
    pub fn new_local(&mut self, ty: MirType, name: Option<String>) -> Local {
        let local = Local(self.next_local);
        self.next_local += 1;
        self.locals.push(LocalDecl {
            name,
            ty,
            is_mutable: true,
            lifetime: None,
        });
        local
    }

    /// The return place (_0).
    pub fn return_place(&self) -> Place {
        Place::local(Local(0))
    }

    /// Get a parameter as a local (1-indexed: _1, _2, ...).
    pub fn param(&self, index: usize) -> Local {
        Local((index + 1) as u32)
    }

    /// Push a statement to the current block.
    pub fn push_stmt(&mut self, stmt: Statement) {
        self.blocks[self.current_block].statements.push(stmt);
    }

    /// Assign an rvalue to a place in the current block.
    pub fn assign(&mut self, place: Place, rvalue: Rvalue) {
        self.push_stmt(Statement::Assign(place, rvalue));
    }

    /// Assign a constant to a local.
    pub fn assign_const(&mut self, local: Local, constant: Constant) {
        self.assign(
            Place::local(local),
            Rvalue::Use(Operand::Constant(constant)),
        );
    }

    /// Assign a binary operation result to a local.
    pub fn assign_binop(&mut self, dest: Local, op: BinOp, lhs: Operand, rhs: Operand) {
        self.assign(Place::local(dest), Rvalue::BinaryOp(op, lhs, rhs));
    }

    /// Drop a place.
    pub fn drop(&mut self, place: Place) {
        self.push_stmt(Statement::Drop(place));
    }

    /// Set the terminator for the current block.
    pub fn terminate(&mut self, terminator: Terminator) {
        self.blocks[self.current_block].terminator = Some(terminator);
    }

    /// Terminate with a goto.
    pub fn goto(&mut self, target: BasicBlockId) {
        self.terminate(Terminator::Goto(target));
    }

    /// Terminate with a return.
    pub fn return_(&mut self) {
        self.terminate(Terminator::Return);
    }

    /// Terminate with a conditional branch.
    pub fn switch_int(
        &mut self,
        discriminant: Operand,
        targets: Vec<(i64, BasicBlockId)>,
        otherwise: BasicBlockId,
    ) {
        self.terminate(Terminator::SwitchInt {
            discriminant,
            targets,
            otherwise,
        });
    }

    /// Terminate with a function call.
    pub fn call(
        &mut self,
        func: impl Into<String>,
        args: Vec<Operand>,
        destination: Place,
        target: BasicBlockId,
    ) {
        self.terminate(Terminator::Call {
            func: func.into(),
            args,
            destination,
            target,
        });
    }

    /// Build the final MIR body.
    pub fn build(self) -> Body {
        Body {
            name: self.name,
            params: self.params,
            return_type: self.return_type,
            locals: self.locals,
            basic_blocks: self.blocks,
            block_names: Default::default(),
            lifetime_params: vec![],
            lifetime_bounds: vec![],
        }
    }
}
