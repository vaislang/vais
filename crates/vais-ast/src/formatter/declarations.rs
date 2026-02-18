//! Format top-level declarations

use super::*;

impl Formatter {
    /// Format a top-level item
    pub(crate) fn format_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => self.format_function(f),
            Item::Struct(s) => self.format_struct(s),
            Item::Enum(e) => self.format_enum(e),
            Item::Union(u) => self.format_union(u),
            Item::TypeAlias(t) => self.format_type_alias(t),
            Item::TraitAlias(ta) => self.format_trait_alias(ta),
            Item::Use(u) => self.format_use(u),
            Item::Trait(t) => self.format_trait(t),
            Item::Impl(i) => self.format_impl(i),
            Item::Macro(m) => self.format_macro(m),
            Item::ExternBlock(e) => self.format_extern_block(e),
            Item::Const(c) => self.format_const(c),
            Item::Global(g) => self.format_global(g),
            Item::Error { message, .. } => {
                // Format error nodes as comments to preserve them in formatted output
                self.output
                    .push_str(&format!("{}# ERROR: {}\n", self.indent(), message));
            }
        }
    }

    /// Format an extern block
    pub(crate) fn format_extern_block(&mut self, e: &ExternBlock) {
        let indent = self.indent();
        self.output.push_str(&indent);
        self.output.push_str("extern \"");
        self.output.push_str(&e.abi);
        self.output.push_str("\" {\n");

        self.push_indent();
        for func in &e.functions {
            self.output.push_str(&self.indent());
            self.output.push_str("F ");
            self.output.push_str(&func.name.node);
            self.output.push('(');

            let params: Vec<String> = func
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name.node, self.format_type(&p.ty.node)))
                .collect();
            self.output.push_str(&params.join(", "));

            if func.is_vararg {
                if !func.params.is_empty() {
                    self.output.push_str(", ");
                }
                self.output.push_str("...");
            }

            self.output.push(')');
            if let Some(ret) = &func.ret_type {
                self.output.push_str(" -> ");
                self.output.push_str(&self.format_type(&ret.node));
            }
            self.output.push_str(";\n");
        }
        self.pop_indent();

        self.output.push_str(&indent);
        self.output.push_str("}\n");
    }

    /// Format a constant definition
    pub(crate) fn format_const(&mut self, c: &ConstDef) {
        let indent = self.indent();
        self.output.push_str(&indent);
        if c.is_pub {
            self.output.push_str("P ");
        }
        self.output.push_str("C ");
        self.output.push_str(&c.name.node);
        self.output.push_str(": ");
        self.output.push_str(&self.format_type(&c.ty.node));
        self.output.push_str(" = ");
        self.format_expr(&c.value.node);
        self.output.push('\n');
    }

    /// Format a global variable definition
    pub(crate) fn format_global(&mut self, g: &GlobalDef) {
        let indent = self.indent();
        self.output.push_str(&indent);
        if g.is_pub {
            self.output.push_str("P ");
        }
        self.output.push_str("G ");
        self.output.push_str(&g.name.node);
        self.output.push_str(": ");
        self.output.push_str(&self.format_type(&g.ty.node));
        self.output.push_str(" = ");
        self.format_expr(&g.value.node);
        self.output.push('\n');
    }

    /// Format a function
    pub(crate) fn format_function(&mut self, f: &Function) {
        let indent = self.indent();

        // Attributes
        for attr in &f.attributes {
            self.output.push_str(&indent);
            self.output.push_str("#[");
            self.output.push_str(&attr.name);
            if !attr.args.is_empty() {
                self.output.push('(');
                self.output.push_str(&attr.args.join(", "));
                self.output.push(')');
            }
            self.output.push_str("]\n");
        }

        self.output.push_str(&indent);

        // pub
        if f.is_pub {
            self.output.push_str("pub ");
        }

        // async
        if f.is_async {
            self.output.push_str("async ");
        }

        // F name
        self.output.push_str("F ");
        self.output.push_str(&f.name.node);

        // Generics
        if !f.generics.is_empty() {
            self.output.push('<');
            let mut first = true;
            for g in &f.generics {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                if g.bounds.is_empty() {
                    self.output.push_str(&g.name.node);
                } else {
                    self.output.push_str(&g.name.node);
                    self.output.push_str(": ");
                    let bounds: Vec<&str> = g.bounds.iter().map(|b| b.node.as_str()).collect();
                    self.output.push_str(&bounds.join(" + "));
                }
            }
            self.output.push('>');
        }

        // Parameters
        self.output.push('(');
        let params: Vec<String> = f
            .params
            .iter()
            .map(|p| {
                let mut s = String::with_capacity(64);
                if p.is_mut {
                    s.push_str("mut ");
                }
                s.push_str(&p.name.node);
                s.push_str(": ");
                s.push_str(&self.format_type(&p.ty.node));
                s
            })
            .collect();
        self.output.push_str(&params.join(", "));
        self.output.push(')');

        // Return type
        if let Some(ret) = &f.ret_type {
            self.output.push_str(" -> ");
            self.output.push_str(&self.format_type(&ret.node));
        }

        // Body
        match &f.body {
            FunctionBody::Expr(expr) => {
                self.output.push_str(" = ");
                self.output.push_str(&self.format_expr(&expr.node));
                self.output.push('\n');
            }
            FunctionBody::Block(stmts) => {
                self.output.push_str(" {\n");
                self.push_indent();
                for stmt in stmts {
                    self.format_stmt(&stmt.node);
                }
                self.pop_indent();
                self.output.push_str(&self.indent());
                self.output.push_str("}\n");
            }
        }
    }

    /// Format a struct
    pub(crate) fn format_struct(&mut self, s: &Struct) {
        let indent = self.indent();

        self.output.push_str(&indent);
        if s.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str("S ");
        self.output.push_str(&s.name.node);

        // Generics
        if !s.generics.is_empty() {
            self.output.push('<');
            let mut first = true;
            for g in &s.generics {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                self.output.push_str(&g.name.node);
            }
            self.output.push('>');
        }

        self.output.push_str(" {\n");
        self.push_indent();

        // Fields
        for field in &s.fields {
            self.output.push_str(&self.indent());
            if field.is_pub {
                self.output.push_str("pub ");
            }
            self.output.push_str(&field.name.node);
            self.output.push_str(": ");
            self.output.push_str(&self.format_type(&field.ty.node));
            self.output.push_str(",\n");
        }

        // Methods
        if !s.methods.is_empty() && !s.fields.is_empty() {
            self.output.push('\n');
        }
        for method in &s.methods {
            self.format_function(&method.node);
        }

        self.pop_indent();
        self.output.push_str(&self.indent());
        self.output.push_str("}\n");
    }

    /// Format an enum
    pub(crate) fn format_enum(&mut self, e: &Enum) {
        let indent = self.indent();

        self.output.push_str(&indent);
        if e.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str("E ");
        self.output.push_str(&e.name.node);

        // Generics
        if !e.generics.is_empty() {
            self.output.push('<');
            let mut first = true;
            for g in &e.generics {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                self.output.push_str(&g.name.node);
            }
            self.output.push('>');
        }

        self.output.push_str(" {\n");
        self.push_indent();

        for variant in &e.variants {
            self.output.push_str(&self.indent());
            self.output.push_str(&variant.name.node);

            match &variant.fields {
                VariantFields::Unit => {}
                VariantFields::Tuple(types) => {
                    self.output.push('(');
                    let types: Vec<String> =
                        types.iter().map(|t| self.format_type(&t.node)).collect();
                    self.output.push_str(&types.join(", "));
                    self.output.push(')');
                }
                VariantFields::Struct(fields) => {
                    self.output.push_str(" {\n");
                    self.push_indent();
                    for field in fields {
                        self.output.push_str(&self.indent());
                        self.output.push_str(&field.name.node);
                        self.output.push_str(": ");
                        self.output.push_str(&self.format_type(&field.ty.node));
                        self.output.push_str(",\n");
                    }
                    self.pop_indent();
                    self.output.push_str(&self.indent());
                    self.output.push('}');
                }
            }
            self.output.push_str(",\n");
        }

        self.pop_indent();
        self.output.push_str(&self.indent());
        self.output.push_str("}\n");
    }

    /// Format a union (untagged, C-style)
    pub(crate) fn format_union(&mut self, u: &Union) {
        let indent = self.indent();

        self.output.push_str(&indent);
        if u.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str("O ");
        self.output.push_str(&u.name.node);

        // Generics
        if !u.generics.is_empty() {
            self.output.push('<');
            let mut first = true;
            for g in &u.generics {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                self.output.push_str(&g.name.node);
            }
            self.output.push('>');
        }

        self.output.push_str(" {\n");
        self.push_indent();

        for field in &u.fields {
            self.output.push_str(&self.indent());
            self.output.push_str(&field.name.node);
            self.output.push_str(": ");
            self.output.push_str(&self.format_type(&field.ty.node));
            self.output.push_str(",\n");
        }

        self.pop_indent();
        self.output.push_str(&indent);
        self.output.push_str("}\n");
    }

    /// Format a type alias
    pub(crate) fn format_type_alias(&mut self, t: &TypeAlias) {
        let indent = self.indent();

        self.output.push_str(&indent);
        if t.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str("T ");
        self.output.push_str(&t.name.node);

        if !t.generics.is_empty() {
            self.output.push('<');
            let mut first = true;
            for g in &t.generics {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                self.output.push_str(&g.name.node);
            }
            self.output.push('>');
        }

        self.output.push_str(" = ");
        self.output.push_str(&self.format_type(&t.ty.node));
        self.output.push('\n');
    }

    pub(crate) fn format_trait_alias(&mut self, ta: &TraitAlias) {
        let indent = self.indent();
        self.output.push_str(&indent);
        if ta.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str("T ");
        self.output.push_str(&ta.name.node);
        if !ta.generics.is_empty() {
            self.output.push('<');
            let mut first = true;
            for g in &ta.generics {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                self.output.push_str(&g.name.node);
            }
            self.output.push('>');
        }
        self.output.push_str(" = ");
        let bounds: Vec<&str> = ta.bounds.iter().map(|b| b.node.as_str()).collect();
        self.output.push_str(&bounds.join(" + "));
        self.output.push('\n');
    }

    /// Format a use statement
    pub(crate) fn format_use(&mut self, u: &Use) {
        let indent = self.indent();

        self.output.push_str(&indent);
        self.output.push_str("U ");
        let path: Vec<&str> = u.path.iter().map(|p| p.node.as_str()).collect();
        self.output.push_str(&path.join("::"));

        if let Some(items) = &u.items {
            if items.len() == 1 {
                self.output.push('.');
                self.output.push_str(&items[0].node);
            } else if !items.is_empty() {
                self.output.push_str(".{");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&item.node);
                }
                self.output.push('}');
            }
        }

        if let Some(alias) = &u.alias {
            self.output.push_str(" as ");
            self.output.push_str(&alias.node);
        }

        self.output.push('\n');
    }

    /// Format a trait
    pub(crate) fn format_trait(&mut self, t: &Trait) {
        let indent = self.indent();

        self.output.push_str(&indent);
        if t.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str("W ");
        self.output.push_str(&t.name.node);

        if !t.generics.is_empty() {
            self.output.push('<');
            let mut first = true;
            for g in &t.generics {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                self.output.push_str(&g.name.node);
            }
            self.output.push('>');
        }

        if !t.super_traits.is_empty() {
            self.output.push_str(": ");
            let traits: Vec<&str> = t.super_traits.iter().map(|s| s.node.as_str()).collect();
            self.output.push_str(&traits.join(" + "));
        }

        self.output.push_str(" {\n");
        self.push_indent();

        // Associated types (with GAT support)
        for at in &t.associated_types {
            self.output.push_str(&self.indent());
            self.output.push_str("T ");
            self.output.push_str(&at.name.node);
            // GAT: output generic parameters
            if !at.generics.is_empty() {
                self.output.push('<');
                let gparams: Vec<String> = at
                    .generics
                    .iter()
                    .map(|g| {
                        let mut s = g.name.node.clone();
                        if !g.bounds.is_empty() {
                            s.push_str(": ");
                            let bounds: Vec<&str> =
                                g.bounds.iter().map(|b| b.node.as_str()).collect();
                            s.push_str(&bounds.join(" + "));
                        }
                        s
                    })
                    .collect();
                self.output.push_str(&gparams.join(", "));
                self.output.push('>');
            }
            if !at.bounds.is_empty() {
                self.output.push_str(": ");
                let bounds: Vec<&str> = at.bounds.iter().map(|b| b.node.as_str()).collect();
                self.output.push_str(&bounds.join(" + "));
            }
            if let Some(default) = &at.default {
                self.output.push_str(" = ");
                self.output.push_str(&self.format_type(&default.node));
            }
            self.output.push('\n');
        }

        // Methods
        for method in &t.methods {
            self.output.push_str(&self.indent());
            if method.is_const {
                self.output.push_str("C ");
            }
            if method.is_async {
                self.output.push_str("A ");
            }
            self.output.push_str("F ");
            self.output.push_str(&method.name.node);
            self.output.push('(');
            let params: Vec<String> = method
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name.node, self.format_type(&p.ty.node)))
                .collect();
            self.output.push_str(&params.join(", "));
            self.output.push(')');

            if let Some(ret) = &method.ret_type {
                self.output.push_str(" -> ");
                self.output.push_str(&self.format_type(&ret.node));
            }

            if let Some(body) = &method.default_body {
                match body {
                    FunctionBody::Expr(expr) => {
                        self.output.push_str(" = ");
                        self.output.push_str(&self.format_expr(&expr.node));
                        self.output.push('\n');
                    }
                    FunctionBody::Block(stmts) => {
                        self.output.push_str(" {\n");
                        self.push_indent();
                        for stmt in stmts {
                            self.format_stmt(&stmt.node);
                        }
                        self.pop_indent();
                        self.output.push_str(&self.indent());
                        self.output.push_str("}\n");
                    }
                }
            } else {
                self.output.push('\n');
            }
        }

        self.pop_indent();
        self.output.push_str(&self.indent());
        self.output.push_str("}\n");
    }

    /// Format an impl block
    pub(crate) fn format_impl(&mut self, i: &Impl) {
        let indent = self.indent();

        self.output.push_str(&indent);
        self.output.push_str("X ");
        self.output.push_str(&self.format_type(&i.target_type.node));

        if let Some(trait_name) = &i.trait_name {
            self.output.push_str(": ");
            self.output.push_str(&trait_name.node);
        }

        self.output.push_str(" {\n");
        self.push_indent();

        for method in &i.methods {
            self.format_function(&method.node);
        }

        self.pop_indent();
        self.output.push_str(&self.indent());
        self.output.push_str("}\n");
    }
}
