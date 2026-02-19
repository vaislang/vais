//! Documentation generator for Vais - Rustdoc style
//!
//! Generates HTML documentation from Vais source files with doc comments (///).

use colored::Colorize;
use std::fs;
use std::path::PathBuf;

use vais_ast::{ConstDef, Enum, ExternFunction, Function, Item, Module, Struct, Trait};
use vais_parser::parse;

pub(super) mod extract;
pub(super) mod html;
pub(super) mod markdown;
#[cfg(test)]
mod tests;

/// Documentation item extracted from source
#[derive(Debug, Clone)]
pub(super) struct DocItem {
    pub(super) name: String,
    pub(super) kind: DocKind,
    pub(super) signature: String,
    pub(super) docs: Vec<String>,
    pub(super) params: Vec<ParamDoc>,
    pub(super) returns: Option<String>,
    pub(super) examples: Vec<String>,
    pub(super) _generics: Vec<GenericDoc>,
    pub(super) visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum DocKind {
    Function,
    Struct,
    Enum,
    Trait,
    Constant,
    ExternFunction,
}

#[derive(Debug, Clone)]
pub(super) struct ParamDoc {
    pub(super) name: String,
    pub(super) ty: String,
    pub(super) is_mut: bool,
}

#[derive(Debug, Clone)]
pub(super) struct GenericDoc {
    pub(super) name: String,
    pub(super) bounds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Visibility {
    Public,
    Private,
}

/// Module documentation
pub(super) struct ModuleDoc {
    pub(super) name: String,
    pub(super) path: PathBuf,
    pub(super) items: Vec<DocItem>,
}

/// Generate documentation from source files
pub fn run(input: &PathBuf, output: &PathBuf, format: &str) -> Result<(), String> {
    println!(
        "{} documentation from {}",
        "Generating".green().bold(),
        input.display()
    );

    // Create output directory
    fs::create_dir_all(output).map_err(|e| format!("Cannot create output directory: {}", e))?;

    // Collect source files
    let files = if input.is_dir() {
        collect_vais_files(input)?
    } else {
        vec![input.clone()]
    };

    if files.is_empty() {
        return Err("No .vais files found".to_string());
    }

    let mut all_docs = Vec::new();

    for file in &files {
        let source = fs::read_to_string(file)
            .map_err(|e| format!("Cannot read '{}': {}", file.display(), e))?;

        let ast =
            parse(&source).map_err(|e| format!("Parse error in '{}': {}", file.display(), e))?;

        let doc = extract::extract_documentation(file, &ast, &source);
        all_docs.push(doc);
    }

    // Generate output based on format
    match format {
        "markdown" | "md" => {
            markdown::generate_markdown_docs(&all_docs, output)?;
        }
        "html" => {
            html::generate_html_docs(&all_docs, output)?;
        }
        _ => {
            return Err(format!(
                "Unknown format: {}. Use 'markdown' or 'html'.",
                format
            ));
        }
    }

    println!(
        "{} Documentation written to {}",
        "Done".green().bold(),
        output.display()
    );
    Ok(())
}

/// Collect all .vais files in a directory recursively
fn collect_vais_files(dir: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir).map_err(|e| format!("Cannot read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Cannot read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            files.extend(collect_vais_files(&path)?);
        } else if path.extension().map(|e| e == "vais").unwrap_or(false) {
            files.push(path);
        }
    }

    Ok(files)
}
