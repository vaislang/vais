//! IR module and function definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::instruction::{EdgeIR, NodeIR};
use crate::value::Value;

/// A complete IR module (compiled from a UNIT)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    /// Module name
    pub name: String,
    /// Module version
    pub version: Option<String>,
    /// Input field definitions
    pub inputs: HashMap<String, FieldDef>,
    /// Output field definitions
    pub outputs: HashMap<String, FieldDef>,
    /// The main function (FLOW graph)
    pub main: Function,
    /// Metadata
    pub metadata: ModuleMetadata,
}

/// Field definition for inputs/outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub type_name: String,
    pub default: Option<Value>,
}

/// Module metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleMetadata {
    pub domain: Option<String>,
    pub deterministic: bool,
    pub pure: bool,
    pub parallel: bool,
}

/// A function representing a FLOW graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    /// All nodes in the graph
    pub nodes: Vec<NodeIR>,
    /// All edges connecting nodes
    pub edges: Vec<EdgeIR>,
    /// Entry points (nodes connected to INPUT)
    pub entry_nodes: Vec<String>,
    /// Exit points (nodes connected to OUTPUT)
    pub exit_nodes: Vec<String>,
    /// Topologically sorted node order for execution
    pub execution_order: Vec<String>,
}

impl Module {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            main: Function::new(),
            metadata: ModuleMetadata::default(),
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn add_input(&mut self, name: impl Into<String>, type_name: impl Into<String>) {
        let name = name.into();
        self.inputs.insert(
            name.clone(),
            FieldDef {
                name,
                type_name: type_name.into(),
                default: None,
            },
        );
    }

    pub fn add_output(&mut self, name: impl Into<String>, type_name: impl Into<String>) {
        let name = name.into();
        self.outputs.insert(
            name.clone(),
            FieldDef {
                name,
                type_name: type_name.into(),
                default: None,
            },
        );
    }
}

impl Function {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            entry_nodes: Vec::new(),
            exit_nodes: Vec::new(),
            execution_order: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: NodeIR) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: EdgeIR) {
        self.edges.push(edge);
    }

    /// Compute topological sort of nodes for execution order
    pub fn compute_execution_order(&mut self) {
        // Build adjacency list
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize
        for node in &self.nodes {
            in_degree.insert(node.id.clone(), 0);
            adj.insert(node.id.clone(), Vec::new());
        }

        // Count incoming edges (only from defined nodes, skip INPUT/OUTPUT pseudo-nodes)
        for edge in &self.edges {
            // Skip edges from/to INPUT/OUTPUT pseudo-nodes for in-degree calculation
            let source_is_pseudo = edge.source_node == "INPUT" || edge.source_node == "OUTPUT";
            let target_is_pseudo = edge.target_node == "INPUT" || edge.target_node == "OUTPUT";

            // Only count edges between real nodes for in-degree
            if !source_is_pseudo && !target_is_pseudo {
                if let Some(degree) = in_degree.get_mut(&edge.target_node) {
                    *degree += 1;
                }
            }
            if let Some(neighbors) = adj.get_mut(&edge.source_node) {
                if !target_is_pseudo {
                    neighbors.push(edge.target_node.clone());
                }
            }
        }

        // Kahn's algorithm
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(k, _)| k.clone())
            .collect();

        let mut order = Vec::new();

        while let Some(node) = queue.pop() {
            order.push(node.clone());

            if let Some(neighbors) = adj.get(&node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        self.execution_order = order;
    }

    /// Find entry nodes (nodes with edges from INPUT)
    pub fn find_entry_nodes(&mut self) {
        self.entry_nodes = self
            .edges
            .iter()
            .filter(|e| e.source_node == "INPUT")
            .map(|e| e.target_node.clone())
            .collect();
    }

    /// Find exit nodes (nodes with edges to OUTPUT)
    pub fn find_exit_nodes(&mut self) {
        self.exit_nodes = self
            .edges
            .iter()
            .filter(|e| e.target_node == "OUTPUT")
            .map(|e| e.source_node.clone())
            .collect();
    }
}

impl Default for Function {
    fn default() -> Self {
        Self::new()
    }
}
