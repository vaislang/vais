//! AST to IR lowering
//!
//! Converts the parsed AST into executable IR format.

use aoel_ast::{
    BinaryOp, Expr, FlowBlock, FlowEdge, FlowNode, InputBlock, LiteralKind, MetaBlock, MetaKey,
    OpType, OutputBlock, UnaryOp, Unit,
};

use crate::instruction::{EdgeIR, Instruction, NodeIR, NodeOpType, OpCode, ReduceOp};
use crate::module::{Function, Module, ModuleMetadata};
use crate::value::Value;

/// Lower an AST Unit to an IR Module
pub fn lower(unit: &Unit) -> Module {
    let mut lowerer = Lowerer::new();
    lowerer.lower_unit(unit)
}

/// AST to IR lowering context
struct Lowerer;

impl Lowerer {
    fn new() -> Self {
        Self
    }

    /// Lower a complete Unit to a Module
    fn lower_unit(&mut self, unit: &Unit) -> Module {
        let mut module = Module::new(unit.full_name());

        // Set version if present
        if let Some(version) = unit.version() {
            module = module.with_version(version.to_string());
        }

        // Lower metadata
        module.metadata = self.lower_meta(&unit.meta);

        // Lower inputs
        self.lower_inputs(&unit.input, &mut module);

        // Lower outputs
        self.lower_outputs(&unit.output, &mut module);

        // Lower FLOW graph
        module.main = self.lower_flow(&unit.flow);

        // Compute execution order
        module.main.find_entry_nodes();
        module.main.find_exit_nodes();
        module.main.compute_execution_order();

        module
    }

    /// Lower META block to ModuleMetadata
    fn lower_meta(&self, meta: &MetaBlock) -> ModuleMetadata {
        let mut metadata = ModuleMetadata::default();

        for entry in &meta.entries {
            match entry.key {
                MetaKey::Domain => {
                    if let aoel_ast::MetaValue::String(s) = &entry.value {
                        metadata.domain = Some(s.clone());
                    }
                }
                MetaKey::Determinism => {
                    if let aoel_ast::MetaValue::Bool(b) = &entry.value {
                        metadata.deterministic = *b;
                    }
                }
                MetaKey::Pure => {
                    if let aoel_ast::MetaValue::Bool(b) = &entry.value {
                        metadata.pure = *b;
                    }
                }
                _ => {}
            }
        }

        metadata
    }

    /// Lower INPUT block to module inputs
    fn lower_inputs(&self, input: &InputBlock, module: &mut Module) {
        for field in &input.fields {
            module.add_input(field.name.name.clone(), type_to_string(&field.ty));
        }
    }

    /// Lower OUTPUT block to module outputs
    fn lower_outputs(&self, output: &OutputBlock, module: &mut Module) {
        for field in &output.fields {
            module.add_output(field.name.name.clone(), type_to_string(&field.ty));
        }
    }

    /// Lower FLOW block to Function
    fn lower_flow(&mut self, flow: &FlowBlock) -> Function {
        let mut function = Function::new();

        // Build edge map: source node -> [(target_node, target_port)]
        let mut output_edges: std::collections::HashMap<String, Vec<(String, String)>> =
            std::collections::HashMap::new();

        for edge in &flow.edges {
            let (source_node, _source_port) = self.extract_edge_endpoint(&edge.source);
            let (target_node, target_port) = self.extract_edge_endpoint(&edge.target);

            // Track edges from nodes to OUTPUT
            if target_node.to_uppercase() == "OUTPUT" {
                output_edges
                    .entry(source_node.clone())
                    .or_default()
                    .push((target_node, target_port));
            }
        }

        // Lower nodes with edge information
        for node in &flow.nodes {
            let outputs = output_edges.get(&node.id.name).cloned().unwrap_or_default();
            let node_ir = self.lower_flow_node_with_outputs(node, &outputs);
            function.add_node(node_ir);
        }

        // Lower edges
        for edge in &flow.edges {
            let edge_ir = self.lower_flow_edge(edge);
            function.add_edge(edge_ir);
        }

        function
    }

    /// Lower a FlowNode to NodeIR (legacy, without output routing)
    fn lower_flow_node(&mut self, node: &FlowNode) -> NodeIR {
        self.lower_flow_node_with_outputs(node, &[])
    }

    /// Lower a FlowNode to NodeIR with output routing information
    fn lower_flow_node_with_outputs(
        &mut self,
        node: &FlowNode,
        outputs: &[(String, String)],
    ) -> NodeIR {
        let op_type = self.lower_op_type(&node.op_type);
        let mut node_ir = NodeIR::new(node.id.name.clone(), op_type);

        let mut instructions = Vec::new();

        // Build parameter map for lookups
        let param_map: std::collections::HashMap<String, &Expr> = node
            .params
            .iter()
            .map(|p| (p.name.name.clone(), &p.value))
            .collect();

        // For specific op types, add specialized instructions
        match node.op_type {
            OpType::Transform => {
                // Check for "value" parameter (simple value output)
                if let Some(value_expr) = param_map.get("value") {
                    // Lower the value expression
                    instructions.extend(self.lower_expr(value_expr));

                    // Store to outputs
                    for (_target_node, target_port) in outputs {
                        instructions.push(Instruction::new(OpCode::Dup));
                        instructions.push(Instruction::new(OpCode::StoreOutput(
                            target_port.clone(),
                        )));
                    }
                    // Pop the extra value if there were outputs (Dup leaves one on stack)
                    if !outputs.is_empty() {
                        instructions.push(Instruction::new(OpCode::Pop));
                    }
                } else if let Some(op_expr) = param_map.get("op") {
                    // Check for arithmetic operations: op=ADD, left=..., right=...
                    if let Expr::Ident(op_ident) = op_expr {
                        let op_name = op_ident.name.to_uppercase();
                        if let (Some(left), Some(right)) =
                            (param_map.get("left"), param_map.get("right"))
                        {
                            // Load left operand
                            instructions.extend(self.lower_expr(left));
                            // Load right operand
                            instructions.extend(self.lower_expr(right));

                            // Apply operation
                            let opcode = match op_name.as_str() {
                                "ADD" => OpCode::Add,
                                "SUB" => OpCode::Sub,
                                "MUL" => OpCode::Mul,
                                "DIV" => OpCode::Div,
                                "MOD" => OpCode::CallBuiltin("MOD".to_string(), 2),
                                _ => OpCode::Nop,
                            };
                            instructions.push(Instruction::new(opcode));

                            // Store result to outputs
                            for (_target_node, target_port) in outputs {
                                instructions.push(Instruction::new(OpCode::Dup));
                                instructions.push(Instruction::new(OpCode::StoreOutput(
                                    target_port.clone(),
                                )));
                            }
                            // Pop the extra value if there were outputs
                            if !outputs.is_empty() {
                                instructions.push(Instruction::new(OpCode::Pop));
                            }
                        }
                    }
                } else {
                    // Generic parameter handling: store each parameter to local variable
                    for param in &node.params {
                        let value_instructions = self.lower_expr(&param.value);
                        instructions.extend(value_instructions);
                        instructions.push(Instruction::new(OpCode::Store(
                            param.name.name.clone(),
                        )));
                    }
                }
            }
            OpType::Map => {
                // Map operation: process each element
                if let Some(param) = node.params.first() {
                    let body = self.lower_expr(&param.value);
                    instructions.push(Instruction::new(OpCode::Map(Box::new(body))));
                }
                // Store result to outputs
                for (_target_node, target_port) in outputs {
                    instructions.push(Instruction::new(OpCode::Dup));
                    instructions.push(Instruction::new(OpCode::StoreOutput(target_port.clone())));
                }
            }
            OpType::Filter => {
                // Filter operation: keep matching elements
                if let Some(param) = node.params.first() {
                    let predicate = self.lower_expr(&param.value);
                    instructions.push(Instruction::new(OpCode::Filter(Box::new(predicate))));
                }
                // Store result to outputs
                for (_target_node, target_port) in outputs {
                    instructions.push(Instruction::new(OpCode::Dup));
                    instructions.push(Instruction::new(OpCode::StoreOutput(target_port.clone())));
                }
            }
            OpType::Reduce => {
                // Reduce operation
                instructions.push(Instruction::new(OpCode::Reduce(
                    ReduceOp::Custom(Box::new(vec![])),
                    Value::Void,
                )));
                // Store result to outputs
                for (_target_node, target_port) in outputs {
                    instructions.push(Instruction::new(OpCode::Dup));
                    instructions.push(Instruction::new(OpCode::StoreOutput(target_port.clone())));
                }
            }
            _ => {
                // Default: just execute parameter expressions
            }
        }

        node_ir = node_ir.with_instructions(instructions);
        node_ir
    }

    /// Lower AST OpType to IR NodeOpType
    fn lower_op_type(&self, op_type: &OpType) -> NodeOpType {
        match op_type {
            OpType::Map => NodeOpType::Map,
            OpType::Filter => NodeOpType::Filter,
            OpType::Reduce => NodeOpType::Reduce(ReduceOp::Sum),
            OpType::Transform => NodeOpType::Transform,
            OpType::Branch => NodeOpType::Branch,
            OpType::Merge => NodeOpType::Merge,
            OpType::Fetch => NodeOpType::Fetch,
            OpType::Store => NodeOpType::Store,
            OpType::Validate => NodeOpType::Validate,
            // Default to Transform for other op types
            _ => NodeOpType::Transform,
        }
    }

    /// Lower a FlowEdge to EdgeIR
    fn lower_flow_edge(&mut self, edge: &FlowEdge) -> EdgeIR {
        let (source_node, source_port) = self.extract_edge_endpoint(&edge.source);
        let (target_node, target_port) = self.extract_edge_endpoint(&edge.target);

        let mut edge_ir = EdgeIR::new(source_node, source_port, target_node, target_port);

        // Lower condition if present
        if let Some(condition) = &edge.condition {
            let condition_instructions = self.lower_expr(condition);
            edge_ir = edge_ir.with_condition(condition_instructions);
        }

        edge_ir
    }

    /// Extract node ID and port from edge endpoint expression
    fn extract_edge_endpoint(&self, expr: &Expr) -> (String, String) {
        match expr {
            Expr::Ident(ident) => (ident.name.clone(), "default".to_string()),
            Expr::FieldAccess(field_access) => {
                let base = match &field_access.base {
                    Expr::Ident(ident) => ident.name.clone(),
                    _ => "unknown".to_string(),
                };
                (base, field_access.field.name.clone())
            }
            _ => ("unknown".to_string(), "default".to_string()),
        }
    }

    /// Lower an expression to IR instructions
    fn lower_expr(&mut self, expr: &Expr) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        match expr {
            Expr::Literal(lit) => {
                let value = self.lower_literal(&lit.kind);
                instructions.push(Instruction::new(OpCode::Const(value)));
            }

            Expr::Ident(ident) => {
                instructions.push(Instruction::new(OpCode::Load(ident.name.clone())));
            }

            Expr::FieldAccess(field_access) => {
                // Check if this is an input/output field access
                if let Expr::Ident(base) = &field_access.base {
                    match base.name.to_uppercase().as_str() {
                        "INPUT" => {
                            instructions.push(Instruction::new(OpCode::LoadInput(
                                field_access.field.name.clone(),
                            )));
                        }
                        "OUTPUT" => {
                            instructions.push(Instruction::new(OpCode::StoreOutput(
                                field_access.field.name.clone(),
                            )));
                        }
                        _ => {
                            // Regular field access
                            instructions.extend(self.lower_expr(&field_access.base));
                            instructions.push(Instruction::new(OpCode::GetField(
                                field_access.field.name.clone(),
                            )));
                        }
                    }
                } else {
                    // Nested field access
                    instructions.extend(self.lower_expr(&field_access.base));
                    instructions.push(Instruction::new(OpCode::GetField(
                        field_access.field.name.clone(),
                    )));
                }
            }

            Expr::Binary(binary) => {
                // Lower left operand
                instructions.extend(self.lower_expr(&binary.left));
                // Lower right operand
                instructions.extend(self.lower_expr(&binary.right));
                // Add binary operation
                let opcode = self.lower_binary_op(&binary.op);
                instructions.push(Instruction::new(opcode));
            }

            Expr::Unary(unary) => {
                // Lower operand
                instructions.extend(self.lower_expr(&unary.operand));
                // Add unary operation
                let opcode = match unary.op {
                    UnaryOp::Not => OpCode::Not,
                    UnaryOp::Neg => OpCode::Neg,
                };
                instructions.push(Instruction::new(opcode));
            }

            Expr::Call(call) => {
                // Lower arguments
                let arg_count = call.args.len();
                for arg in &call.args {
                    instructions.extend(self.lower_expr(arg));
                }
                // Add function call
                instructions.push(Instruction::new(OpCode::CallBuiltin(
                    call.name.name.clone(),
                    arg_count,
                )));
            }

            Expr::Index(index) => {
                // Lower base
                instructions.extend(self.lower_expr(&index.base));
                // Lower index
                instructions.extend(self.lower_expr(&index.index));
                // Add index operation
                instructions.push(Instruction::new(OpCode::Index));
            }

            Expr::Grouped(grouped) => {
                // Just lower the inner expression
                instructions.extend(self.lower_expr(&grouped.inner));
            }

            Expr::ExternalRef(ext_ref) => {
                // External references become string constants for now
                instructions.push(Instruction::new(OpCode::Const(Value::String(
                    ext_ref.path.clone(),
                ))));
            }
        }

        instructions
    }

    /// Lower a literal to a Value
    fn lower_literal(&self, kind: &LiteralKind) -> Value {
        match kind {
            LiteralKind::Integer(n) => Value::Int(*n),
            LiteralKind::Float(f) => Value::Float(*f),
            LiteralKind::String(s) => Value::String(s.clone()),
            LiteralKind::Bool(b) => Value::Bool(*b),
            LiteralKind::Regex(r) => Value::String(r.clone()), // Store as string for now
            LiteralKind::Duration(d) => Value::String(d.clone()), // Store as string for now
            LiteralKind::Size(s) => Value::String(s.clone()),  // Store as string for now
            LiteralKind::Void => Value::Void,
        }
    }

    /// Lower a binary operator to an OpCode
    fn lower_binary_op(&self, op: &BinaryOp) -> OpCode {
        match op {
            BinaryOp::Add => OpCode::Add,
            BinaryOp::Sub => OpCode::Sub,
            BinaryOp::Mul => OpCode::Mul,
            BinaryOp::Div => OpCode::Div,
            BinaryOp::Eq => OpCode::Eq,
            BinaryOp::Neq => OpCode::Neq,
            BinaryOp::Lt => OpCode::Lt,
            BinaryOp::Gt => OpCode::Gt,
            BinaryOp::Lte => OpCode::Lte,
            BinaryOp::Gte => OpCode::Gte,
            BinaryOp::And => OpCode::And,
            BinaryOp::Or => OpCode::Or,
            // For IN/MATCH/etc, use builtin calls
            BinaryOp::In => OpCode::CallBuiltin("IN".to_string(), 2),
            BinaryOp::Match => OpCode::CallBuiltin("MATCH".to_string(), 2),
            BinaryOp::Xor => OpCode::CallBuiltin("XOR".to_string(), 2),
            BinaryOp::Implies => OpCode::CallBuiltin("IMPLIES".to_string(), 2),
        }
    }
}

/// Convert AST Type to string representation
fn type_to_string(ty: &aoel_ast::Type) -> String {
    match ty {
        aoel_ast::Type::Primitive(p) => p.kind.as_str().to_string(),
        aoel_ast::Type::Array(arr) => format!("ARRAY[{}]", type_to_string(&arr.element_type)),
        aoel_ast::Type::Optional(opt) => format!("{}?", type_to_string(&opt.inner_type)),
        aoel_ast::Type::Map(map) => {
            format!(
                "MAP[{}, {}]",
                type_to_string(&map.key_type),
                type_to_string(&map.value_type)
            )
        }
        aoel_ast::Type::Struct(s) => {
            let field_strs: Vec<String> = s
                .fields
                .iter()
                .map(|f| format!("{}: {}", f.name.name, type_to_string(&f.ty)))
                .collect();
            format!("STRUCT {{ {} }}", field_strs.join(", "))
        }
        aoel_ast::Type::Union(u) => {
            let type_strs: Vec<String> = u.types.iter().map(type_to_string).collect();
            type_strs.join(" | ")
        }
        aoel_ast::Type::Ref(ext_ref) => ext_ref.path.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoel_ast::*;
    use aoel_lexer::Span;

    fn dummy_span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn int_type() -> Type {
        Type::Primitive(PrimitiveType {
            kind: PrimitiveKind::Int,
            span: dummy_span(),
        })
    }

    fn create_minimal_unit(name: &str) -> Unit {
        Unit {
            header: UnitHeader {
                kind: UnitKind::Function,
                name: QualifiedName::new(
                    vec![Ident {
                        name: name.to_string(),
                        span: dummy_span(),
                    }],
                    dummy_span(),
                ),
                version: None,
                span: dummy_span(),
            },
            meta: MetaBlock {
                entries: vec![],
                span: dummy_span(),
            },
            input: InputBlock {
                fields: vec![InputField {
                    name: Ident {
                        name: "x".to_string(),
                        span: dummy_span(),
                    },
                    ty: int_type(),
                    constraints: vec![],
                    span: dummy_span(),
                }],
                span: dummy_span(),
            },
            output: OutputBlock {
                fields: vec![OutputField {
                    name: Ident {
                        name: "result".to_string(),
                        span: dummy_span(),
                    },
                    ty: int_type(),
                    constraints: vec![],
                    span: dummy_span(),
                }],
                span: dummy_span(),
            },
            intent: IntentBlock {
                goal_type: GoalType::Transform,
                goal_spec: GoalSpec {
                    inputs: vec![],
                    outputs: vec![],
                    span: dummy_span(),
                },
                priorities: vec![],
                on_failure: None,
                span: dummy_span(),
            },
            constraint: ConstraintBlock {
                constraints: vec![],
                span: dummy_span(),
            },
            flow: FlowBlock {
                nodes: vec![FlowNode {
                    id: Ident {
                        name: "process".to_string(),
                        span: dummy_span(),
                    },
                    op_type: OpType::Transform,
                    params: vec![],
                    custom_op: None,
                    span: dummy_span(),
                }],
                edges: vec![
                    FlowEdge {
                        source: Expr::Ident(Ident {
                            name: "INPUT".to_string(),
                            span: dummy_span(),
                        }),
                        target: Expr::Ident(Ident {
                            name: "process".to_string(),
                            span: dummy_span(),
                        }),
                        params: vec![],
                        condition: None,
                        span: dummy_span(),
                    },
                    FlowEdge {
                        source: Expr::Ident(Ident {
                            name: "process".to_string(),
                            span: dummy_span(),
                        }),
                        target: Expr::Ident(Ident {
                            name: "OUTPUT".to_string(),
                            span: dummy_span(),
                        }),
                        params: vec![],
                        condition: None,
                        span: dummy_span(),
                    },
                ],
                span: dummy_span(),
            },
            execution: ExecutionBlock {
                parallel: false,
                target: TargetKind::Any,
                memory: MemoryMode::Unbounded,
                isolation: IsolationKind::None,
                cache: CacheMode::None,
                span: dummy_span(),
            },
            verify: VerifyBlock {
                entries: vec![],
                span: dummy_span(),
            },
            span: dummy_span(),
        }
    }

    #[test]
    fn test_lower_basic_unit() {
        let unit = create_minimal_unit("TestUnit");
        let module = lower(&unit);

        assert_eq!(module.name, "TestUnit");
        assert!(module.inputs.contains_key("x"));
        assert!(module.outputs.contains_key("result"));
        assert_eq!(module.main.nodes.len(), 1);
        assert_eq!(module.main.edges.len(), 2);
    }

    #[test]
    fn test_lower_literal_expr() {
        let lowerer = Lowerer::new();

        let int_value = lowerer.lower_literal(&LiteralKind::Integer(42));
        assert_eq!(int_value, Value::Int(42));

        let float_value = lowerer.lower_literal(&LiteralKind::Float(3.14));
        assert_eq!(float_value, Value::Float(3.14));

        let bool_value = lowerer.lower_literal(&LiteralKind::Bool(true));
        assert_eq!(bool_value, Value::Bool(true));

        let string_value = lowerer.lower_literal(&LiteralKind::String("hello".to_string()));
        assert_eq!(string_value, Value::String("hello".to_string()));
    }

    #[test]
    fn test_lower_binary_op() {
        let lowerer = Lowerer::new();

        assert_eq!(lowerer.lower_binary_op(&BinaryOp::Add), OpCode::Add);
        assert_eq!(lowerer.lower_binary_op(&BinaryOp::Sub), OpCode::Sub);
        assert_eq!(lowerer.lower_binary_op(&BinaryOp::Mul), OpCode::Mul);
        assert_eq!(lowerer.lower_binary_op(&BinaryOp::Div), OpCode::Div);
        assert_eq!(lowerer.lower_binary_op(&BinaryOp::Eq), OpCode::Eq);
        assert_eq!(lowerer.lower_binary_op(&BinaryOp::And), OpCode::And);
        assert_eq!(lowerer.lower_binary_op(&BinaryOp::Or), OpCode::Or);
    }

    #[test]
    fn test_lower_expr_instructions() {
        let mut lowerer = Lowerer::new();

        // Test integer literal
        let expr = Expr::Literal(Literal::integer(42, dummy_span()));
        let instructions = lowerer.lower_expr(&expr);
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0].opcode, OpCode::Const(Value::Int(42)));

        // Test identifier
        let expr = Expr::Ident(Ident {
            name: "x".to_string(),
            span: dummy_span(),
        });
        let instructions = lowerer.lower_expr(&expr);
        assert_eq!(instructions.len(), 1);
        assert_eq!(
            instructions[0].opcode,
            OpCode::Load("x".to_string())
        );
    }

    #[test]
    fn test_execution_order() {
        let unit = create_minimal_unit("TestOrder");
        let module = lower(&unit);

        // Should have entry/exit nodes computed
        // Entry nodes are nodes connected FROM INPUT
        assert!(!module.main.entry_nodes.is_empty(), "entry_nodes should not be empty");
        // Exit nodes are nodes connected TO OUTPUT
        assert!(!module.main.exit_nodes.is_empty(), "exit_nodes should not be empty");
        // Execution order contains the topologically sorted node IDs
        // Our test has one node "process" so it should be in the order
        assert_eq!(module.main.execution_order.len(), 1);
        assert_eq!(module.main.execution_order[0], "process");
    }
}
