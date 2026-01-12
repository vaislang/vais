//! AST Visitor pattern

use crate::*;

/// Visitor trait for traversing the AST
pub trait Visitor {
    type Output;

    // Unit
    fn visit_unit(&mut self, unit: &Unit) -> Self::Output;

    // Blocks
    fn visit_meta_block(&mut self, block: &MetaBlock) -> Self::Output;
    fn visit_input_block(&mut self, block: &InputBlock) -> Self::Output;
    fn visit_output_block(&mut self, block: &OutputBlock) -> Self::Output;
    fn visit_intent_block(&mut self, block: &IntentBlock) -> Self::Output;
    fn visit_constraint_block(&mut self, block: &ConstraintBlock) -> Self::Output;
    fn visit_flow_block(&mut self, block: &FlowBlock) -> Self::Output;
    fn visit_execution_block(&mut self, block: &ExecutionBlock) -> Self::Output;
    fn visit_verify_block(&mut self, block: &VerifyBlock) -> Self::Output;

    // Flow elements
    fn visit_flow_node(&mut self, node: &FlowNode) -> Self::Output;
    fn visit_flow_edge(&mut self, edge: &FlowEdge) -> Self::Output;

    // Types
    fn visit_type(&mut self, ty: &Type) -> Self::Output;

    // Expressions
    fn visit_expr(&mut self, expr: &Expr) -> Self::Output;
}

/// Mutable visitor trait
pub trait VisitorMut {
    type Output;

    fn visit_unit_mut(&mut self, unit: &mut Unit) -> Self::Output;
    fn visit_expr_mut(&mut self, expr: &mut Expr) -> Self::Output;
    fn visit_type_mut(&mut self, ty: &mut Type) -> Self::Output;
}

/// Walk trait for AST nodes (traversal helper)
pub trait Walk {
    fn walk<V: Visitor>(&self, visitor: &mut V) -> V::Output;
}

impl Walk for Unit {
    fn walk<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_unit(self)
    }
}

impl Walk for Expr {
    fn walk<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_expr(self)
    }
}

impl Walk for Type {
    fn walk<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_type(self)
    }
}

/// Default visitor implementation that does nothing
/// Useful as a base for visitors that only care about specific nodes
pub struct NoopVisitor;

impl Visitor for NoopVisitor {
    type Output = ();

    fn visit_unit(&mut self, _unit: &Unit) {}
    fn visit_meta_block(&mut self, _block: &MetaBlock) {}
    fn visit_input_block(&mut self, _block: &InputBlock) {}
    fn visit_output_block(&mut self, _block: &OutputBlock) {}
    fn visit_intent_block(&mut self, _block: &IntentBlock) {}
    fn visit_constraint_block(&mut self, _block: &ConstraintBlock) {}
    fn visit_flow_block(&mut self, _block: &FlowBlock) {}
    fn visit_execution_block(&mut self, _block: &ExecutionBlock) {}
    fn visit_verify_block(&mut self, _block: &VerifyBlock) {}
    fn visit_flow_node(&mut self, _node: &FlowNode) {}
    fn visit_flow_edge(&mut self, _edge: &FlowEdge) {}
    fn visit_type(&mut self, _ty: &Type) {}
    fn visit_expr(&mut self, _expr: &Expr) {}
}
