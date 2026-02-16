//! WIT package definition and formatting logic

use super::interface::{
    WitExportItem, WitImportItem, WitInterface, WitMethodKind, WitResource, WitWorld,
};
use super::types::{WitFunction, WitResult, WitTypeDefinition};

#[derive(Debug, Clone)]
pub struct WitPackage {
    pub namespace: String,
    pub name: String,
    pub version: Option<String>,
    pub interfaces: Vec<WitInterface>,
    pub worlds: Vec<WitWorld>,
    pub docs: Option<String>,
}

impl WitPackage {
    /// Create a new WIT package
    pub fn new(namespace: &str, name: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            name: name.to_string(),
            version: None,
            interfaces: Vec::new(),
            worlds: Vec::new(),
            docs: None,
        }
    }

    /// Set version
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    /// Add interface
    pub fn add_interface(&mut self, interface: WitInterface) {
        self.interfaces.push(interface);
    }

    /// Add world
    pub fn add_world(&mut self, world: WitWorld) {
        self.worlds.push(world);
    }

    /// Generate WIT file content
    pub fn to_wit_string(&self) -> String {
        let mut output = String::new();

        // Package declaration
        if let Some(version) = &self.version {
            output.push_str(&format!(
                "package {}:{}@{};\n\n",
                self.namespace, self.name, version
            ));
        } else {
            output.push_str(&format!("package {}:{};\n\n", self.namespace, self.name));
        }

        // Package docs
        if let Some(docs) = &self.docs {
            for line in docs.lines() {
                output.push_str(&format!("/// {}\n", line));
            }
            output.push('\n');
        }

        // Interfaces
        for interface in &self.interfaces {
            output.push_str(&self.format_interface(interface));
            output.push('\n');
        }

        // Worlds
        for world in &self.worlds {
            output.push_str(&self.format_world(world));
            output.push('\n');
        }

        output
    }

    fn format_interface(&self, interface: &WitInterface) -> String {
        let mut output = String::new();

        if let Some(docs) = &interface.docs {
            for line in docs.lines() {
                output.push_str(&format!("/// {}\n", line));
            }
        }

        output.push_str(&format!("interface {} {{\n", interface.name));

        // Type definitions
        for typedef in &interface.types {
            output.push_str(&self.format_type_definition(typedef, 1));
            output.push('\n');
        }

        // Functions
        for function in &interface.functions {
            output.push_str(&self.format_function(function, 1));
            output.push('\n');
        }

        // Resources
        for resource in &interface.resources {
            output.push_str(&self.format_resource(resource, 1));
            output.push('\n');
        }

        output.push_str("}\n");
        output
    }

    pub(crate) fn format_type_definition(
        &self,
        typedef: &WitTypeDefinition,
        indent: usize,
    ) -> String {
        let indent_str = "  ".repeat(indent);
        match typedef {
            WitTypeDefinition::Record(record) => {
                let mut output = String::new();
                if let Some(docs) = &record.docs {
                    for line in docs.lines() {
                        output.push_str(&format!("{}/// {}\n", indent_str, line));
                    }
                }
                output.push_str(&format!("{}record {} {{\n", indent_str, record.name));
                for field in &record.fields {
                    if let Some(docs) = &field.docs {
                        output.push_str(&format!("{}  /// {}\n", indent_str, docs));
                    }
                    output.push_str(&format!("{}  {}: {},\n", indent_str, field.name, field.ty));
                }
                output.push_str(&format!("{}}}\n", indent_str));
                output
            }
            WitTypeDefinition::Variant(variant) => {
                let mut output = String::new();
                if let Some(docs) = &variant.docs {
                    for line in docs.lines() {
                        output.push_str(&format!("{}/// {}\n", indent_str, line));
                    }
                }
                output.push_str(&format!("{}variant {} {{\n", indent_str, variant.name));
                for case in &variant.cases {
                    if let Some(docs) = &case.docs {
                        output.push_str(&format!("{}  /// {}\n", indent_str, docs));
                    }
                    if let Some(ty) = &case.ty {
                        output.push_str(&format!("{}  {}({}),\n", indent_str, case.name, ty));
                    } else {
                        output.push_str(&format!("{}  {},\n", indent_str, case.name));
                    }
                }
                output.push_str(&format!("{}}}\n", indent_str));
                output
            }
            WitTypeDefinition::Enum(enum_def) => {
                let mut output = String::new();
                if let Some(docs) = &enum_def.docs {
                    for line in docs.lines() {
                        output.push_str(&format!("{}/// {}\n", indent_str, line));
                    }
                }
                output.push_str(&format!("{}enum {} {{\n", indent_str, enum_def.name));
                for case in &enum_def.cases {
                    if let Some(docs) = &case.docs {
                        output.push_str(&format!("{}  /// {}\n", indent_str, docs));
                    }
                    output.push_str(&format!("{}  {},\n", indent_str, case.name));
                }
                output.push_str(&format!("{}}}\n", indent_str));
                output
            }
            WitTypeDefinition::Flags(flags) => {
                let mut output = String::new();
                if let Some(docs) = &flags.docs {
                    for line in docs.lines() {
                        output.push_str(&format!("{}/// {}\n", indent_str, line));
                    }
                }
                output.push_str(&format!("{}flags {} {{\n", indent_str, flags.name));
                for flag in &flags.flags {
                    output.push_str(&format!("{}  {},\n", indent_str, flag));
                }
                output.push_str(&format!("{}}}\n", indent_str));
                output
            }
            WitTypeDefinition::Type { name, ty } => {
                format!("{}type {} = {};\n", indent_str, name, ty)
            }
        }
    }

    fn format_function(&self, function: &WitFunction, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        let mut output = String::new();

        if let Some(docs) = &function.docs {
            for line in docs.lines() {
                output.push_str(&format!("{}/// {}\n", indent_str, line));
            }
        }

        output.push_str(&format!("{}{}: func(", indent_str, function.name));

        // Parameters
        for (i, param) in function.params.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&format!("{}: {}", param.name, param.ty));
        }

        output.push(')');

        // Results
        if let Some(results) = &function.results {
            match results {
                WitResult::Anon(ty) => {
                    output.push_str(&format!(" -> {}", ty));
                }
                WitResult::Named(params) => {
                    output.push_str(" -> (");
                    for (i, param) in params.iter().enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        output.push_str(&format!("{}: {}", param.name, param.ty));
                    }
                    output.push(')');
                }
            }
        }

        output.push_str(";\n");
        output
    }

    fn format_resource(&self, resource: &WitResource, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        let mut output = String::new();

        if let Some(docs) = &resource.docs {
            for line in docs.lines() {
                output.push_str(&format!("{}/// {}\n", indent_str, line));
            }
        }

        output.push_str(&format!("{}resource {} {{\n", indent_str, resource.name));

        for method in &resource.methods {
            if let Some(docs) = &method.docs {
                output.push_str(&format!("{}  /// {}\n", indent_str, docs));
            }

            match method.kind {
                WitMethodKind::Constructor => {
                    output.push_str(&format!("{}  constructor(", indent_str));
                }
                WitMethodKind::Static => {
                    output.push_str(&format!("{}  {}: static func(", indent_str, method.name));
                }
                WitMethodKind::Method => {
                    output.push_str(&format!("{}  {}: func(", indent_str, method.name));
                }
            }

            // Parameters
            for (i, param) in method.params.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!("{}: {}", param.name, param.ty));
            }

            output.push(')');

            // Results
            if let Some(results) = &method.results {
                match results {
                    WitResult::Anon(ty) => {
                        output.push_str(&format!(" -> {}", ty));
                    }
                    WitResult::Named(params) => {
                        output.push_str(" -> (");
                        for (i, param) in params.iter().enumerate() {
                            if i > 0 {
                                output.push_str(", ");
                            }
                            output.push_str(&format!("{}: {}", param.name, param.ty));
                        }
                        output.push(')');
                    }
                }
            }

            output.push_str(";\n");
        }

        output.push_str(&format!("{}}}\n", indent_str));
        output
    }

    pub(crate) fn format_world(&self, world: &WitWorld) -> String {
        let mut output = String::new();

        if let Some(docs) = &world.docs {
            for line in docs.lines() {
                output.push_str(&format!("/// {}\n", line));
            }
        }

        output.push_str(&format!("world {} {{\n", world.name));

        // Imports
        for import in &world.imports {
            match &import.item {
                WitImportItem::Interface(name) => {
                    output.push_str(&format!("  import {};\n", name));
                }
                WitImportItem::Function(func) => {
                    output.push_str(&format!(
                        "  import {};\n",
                        self.format_function(func, 1).trim()
                    ));
                }
            }
        }

        // Exports
        for export in &world.exports {
            match &export.item {
                WitExportItem::Interface(name) => {
                    output.push_str(&format!("  export {};\n", name));
                }
                WitExportItem::Function(func) => {
                    output.push_str(&format!(
                        "  export {};\n",
                        self.format_function(func, 1).trim()
                    ));
                }
            }
        }

        output.push_str("}\n");
        output
    }
}
