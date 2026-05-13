//! Format patterns

use super::*;

impl Formatter {
    /// Format a pattern
    pub(crate) fn format_pattern(&self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Wildcard => "_".to_string(),
            Pattern::Ident(name) => name.to_string(),
            Pattern::Literal(lit) => match lit {
                Literal::Int(n) => n.to_string(),
                Literal::Float(n) => n.to_string(),
                Literal::Bool(b) => b.to_string(),
                Literal::String(s) => format!("\"{}\"", s),
            },
            Pattern::Tuple(patterns) => {
                let ps: Vec<String> = patterns
                    .iter()
                    .map(|p| self.format_pattern(&p.node))
                    .collect();
                format!("({})", ps.join(", "))
            }
            Pattern::Struct { name, fields } => {
                let fs: Vec<String> = fields
                    .iter()
                    .map(|(n, p)| {
                        if let Some(pat) = p {
                            format!("{}: {}", n.node, self.format_pattern(&pat.node))
                        } else {
                            n.node.to_string()
                        }
                    })
                    .collect();
                format!("{} {{ {} }}", name.node, fs.join(", "))
            }
            Pattern::Variant { name, fields } => {
                if fields.is_empty() {
                    name.node.to_string()
                } else {
                    let fs: Vec<String> = fields
                        .iter()
                        .map(|p| self.format_pattern(&p.node))
                        .collect();
                    format!("{}({})", name.node, fs.join(", "))
                }
            }
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let mut s = String::with_capacity(64);
                if let Some(st) = start {
                    s.push_str(&self.format_pattern(&st.node));
                }
                s.push_str(if *inclusive { "..=" } else { ".." });
                if let Some(en) = end {
                    s.push_str(&self.format_pattern(&en.node));
                }
                s
            }
            Pattern::Or(patterns) => {
                let ps: Vec<String> = patterns
                    .iter()
                    .map(|p| self.format_pattern(&p.node))
                    .collect();
                ps.join(" | ")
            }
            Pattern::Alias { name, pattern } => {
                format!("{} @ {}", name, self.format_pattern(&pattern.node))
            }
        }
    }
}
