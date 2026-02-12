//! Completion handler implementation

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::ai_completion::{generate_ai_completions, CompletionContext as AiContext};
use crate::backend::VaisBackend;

pub(crate) async fn handle_completion(
    backend: &VaisBackend,
    params: CompletionParams,
) -> Result<Option<CompletionResponse>> {
    let uri = &params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;

    let mut items = vec![];

    // Check if we're completing after a dot (method completion)
    let is_method_completion = if let Some(doc) = backend.documents.get(uri) {
        let line_idx = position.line as usize;
        if let Some(line) = doc.content.get_line(line_idx) {
            let line_str: String = line.chars().collect();
            let col = position.character as usize;
            col > 0 && line_str.chars().nth(col - 1) == Some('.')
        } else {
            false
        }
    } else {
        false
    };

    if is_method_completion {
        items.extend(method_completions(backend, uri));
        return Ok(Some(CompletionResponse::Array(items)));
    }

    // Add keyword completions
    items.extend(keyword_completions());

    // Add type completions
    items.extend(type_completions());

    // Add builtin function completions
    items.extend(builtin_completions());

    // Add standard library module completions
    items.extend(std_module_completions());

    // Add math items
    items.extend(math_completions());

    // Add IO functions
    items.extend(io_completions());

    // Add common type constructors
    items.extend(constructor_completions());

    // Add functions/structs/enums from current document
    items.extend(document_symbol_completions(backend, uri));

    // AI-based completions: analyze context and suggest patterns
    if let Some(doc) = backend.documents.get(uri) {
        let content: String = doc.content.chars().collect();
        let ai_ctx = AiContext::from_document(&content, position, doc.ast.as_ref());
        items.extend(generate_ai_completions(&ai_ctx));
    }

    Ok(Some(CompletionResponse::Array(items)))
}

fn method_completions(
    backend: &VaisBackend,
    uri: &tower_lsp::lsp_types::Url,
) -> Vec<CompletionItem> {
    let mut items = vec![];

    // Add common method completions
    let methods = [
        ("len", "Get length", "len()"),
        ("is_empty", "Check if empty", "is_empty()"),
        ("push", "Push element", "push(${1:value})"),
        ("pop", "Pop element", "pop()"),
        ("get", "Get element", "get(${1:index})"),
        ("clone", "Clone value", "clone()"),
        ("drop", "Drop/free", "drop()"),
        ("print", "Print value", "print()"),
        (
            "unwrap_or",
            "Unwrap with default",
            "unwrap_or(${1:default})",
        ),
        ("is_some", "Check if Some", "is_some()"),
        ("is_none", "Check if None", "is_none()"),
        ("is_ok", "Check if Ok", "is_ok()"),
        ("is_err", "Check if Err", "is_err()"),
        ("await", "Await async result", "await"),
    ];

    for (name, detail, snippet) in methods {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some(detail.to_string()),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });
    }

    // Add methods from impl blocks in current document
    if let Some(doc) = backend.documents.get(uri) {
        if let Some(ast) = &doc.ast {
            for item in &ast.items {
                if let vais_ast::Item::Impl(impl_block) = &item.node {
                    for method in &impl_block.methods {
                        let params_str: Vec<String> = method
                            .node
                            .params
                            .iter()
                            .filter(|p| p.name.node != "self")
                            .enumerate()
                            .map(|(i, p)| format!("${{{}:{}}}", i + 1, p.name.node))
                            .collect();

                        items.push(CompletionItem {
                            label: method.node.name.node.clone(),
                            kind: Some(CompletionItemKind::METHOD),
                            detail: Some(format!("Method of {:?}", impl_block.target_type.node)),
                            insert_text: Some(format!(
                                "{}({})",
                                method.node.name.node,
                                params_str.join(", ")
                            )),
                            insert_text_format: Some(InsertTextFormat::SNIPPET),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    items
}

fn keyword_completions() -> Vec<CompletionItem> {
    let keywords = [
        (
            "F",
            "Function definition",
            "F ${1:name}($2) -> ${3:type} {\n\t$0\n}",
        ),
        (
            "S",
            "Struct definition",
            "S ${1:Name} {\n\t${2:field}: ${3:type}\n}",
        ),
        ("E", "Enum definition", "E ${1:Name} {\n\t${2:Variant}\n}"),
        ("I", "If expression", "I ${1:condition} {\n\t$0\n}"),
        ("L", "Loop expression", "L ${1:item}: ${2:iter} {\n\t$0\n}"),
        (
            "M",
            "Match expression",
            "M ${1:expr} {\n\t${2:pattern} => $0\n}",
        ),
        ("R", "Return", "R $0"),
        ("B", "Break", "B"),
        ("C", "Continue", "C"),
        ("W", "Trait definition", "W ${1:Name} {\n\t$0\n}"),
        ("X", "Impl block", "X ${1:Type} {\n\t$0\n}"),
        ("U", "Use/Import", "U ${1:std/module}"),
        (
            "A",
            "Async function",
            "A F ${1:name}($2) -> ${3:type} {\n\t$0\n}",
        ),
        ("spawn", "Spawn async task", "spawn ${1:expr}"),
        ("await", "Await async result", "await"),
        ("true", "Boolean true", "true"),
        ("false", "Boolean false", "false"),
    ];

    keywords
        .iter()
        .map(|(kw, detail, snippet)| CompletionItem {
            label: kw.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail.to_string()),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

fn type_completions() -> Vec<CompletionItem> {
    let types = [
        ("i8", "8-bit signed integer"),
        ("i16", "16-bit signed integer"),
        ("i32", "32-bit signed integer"),
        ("i64", "64-bit signed integer"),
        ("i128", "128-bit signed integer"),
        ("u8", "8-bit unsigned integer"),
        ("u16", "16-bit unsigned integer"),
        ("u32", "32-bit unsigned integer"),
        ("u64", "64-bit unsigned integer"),
        ("u128", "128-bit unsigned integer"),
        ("f32", "32-bit floating point"),
        ("f64", "64-bit floating point"),
        ("bool", "Boolean type"),
        ("str", "String type"),
    ];

    types
        .iter()
        .map(|(ty, doc)| CompletionItem {
            label: ty.to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some(doc.to_string()),
            ..Default::default()
        })
        .collect()
}

fn builtin_completions() -> Vec<CompletionItem> {
    let builtins = [
        (
            "puts",
            "Print string with newline",
            "puts(${1:s})",
            "fn(str) -> i64",
        ),
        (
            "putchar",
            "Print single character",
            "putchar(${1:c})",
            "fn(i64) -> i64",
        ),
        (
            "print_i64",
            "Print 64-bit integer",
            "print_i64(${1:n})",
            "fn(i64) -> i64",
        ),
        (
            "print_f64",
            "Print 64-bit float",
            "print_f64(${1:n})",
            "fn(f64) -> i64",
        ),
        (
            "malloc",
            "Allocate heap memory",
            "malloc(${1:size})",
            "fn(i64) -> i64",
        ),
        (
            "free",
            "Free heap memory",
            "free(${1:ptr})",
            "fn(i64) -> i64",
        ),
        (
            "memcpy",
            "Copy memory",
            "memcpy(${1:dst}, ${2:src}, ${3:n})",
            "fn(i64, i64, i64) -> i64",
        ),
        (
            "strlen",
            "Get string length",
            "strlen(${1:s})",
            "fn(i64) -> i64",
        ),
        (
            "load_i64",
            "Load i64 from memory",
            "load_i64(${1:ptr})",
            "fn(i64) -> i64",
        ),
        (
            "store_i64",
            "Store i64 to memory",
            "store_i64(${1:ptr}, ${2:val})",
            "fn(i64, i64) -> i64",
        ),
        (
            "load_byte",
            "Load byte from memory",
            "load_byte(${1:ptr})",
            "fn(i64) -> i64",
        ),
        (
            "store_byte",
            "Store byte to memory",
            "store_byte(${1:ptr}, ${2:val})",
            "fn(i64, i64) -> i64",
        ),
    ];

    builtins
        .iter()
        .map(|(name, doc, snippet, sig)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(sig.to_string()),
            documentation: Some(Documentation::String(doc.to_string())),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

fn std_module_completions() -> Vec<CompletionItem> {
    let std_modules = [
        ("std/math", "Math functions (sin, cos, sqrt, etc.)"),
        ("std/io", "Input/output functions"),
        ("std/option", "Option type (Some, None)"),
        ("std/result", "Result type (Ok, Err)"),
        ("std/vec", "Dynamic array (Vec)"),
        ("std/string", "String type and operations"),
        ("std/hashmap", "Hash map collection"),
        ("std/file", "File I/O operations"),
        ("std/iter", "Iterator trait"),
        ("std/future", "Async Future type"),
        ("std/rc", "Reference counting (Rc)"),
        ("std/box", "Heap allocation (Box)"),
        ("std/arena", "Arena allocator"),
        ("std/runtime", "Async runtime"),
    ];

    std_modules
        .iter()
        .map(|(module, doc)| CompletionItem {
            label: module.to_string(),
            kind: Some(CompletionItemKind::MODULE),
            detail: Some(doc.to_string()),
            ..Default::default()
        })
        .collect()
}

fn math_completions() -> Vec<CompletionItem> {
    let math_items = [
        ("PI", "π = 3.14159...", CompletionItemKind::CONSTANT),
        ("E", "e = 2.71828...", CompletionItemKind::CONSTANT),
        ("TAU", "τ = 2π", CompletionItemKind::CONSTANT),
        ("sqrt", "Square root", CompletionItemKind::FUNCTION),
        ("pow", "Power function", CompletionItemKind::FUNCTION),
        ("sin", "Sine", CompletionItemKind::FUNCTION),
        ("cos", "Cosine", CompletionItemKind::FUNCTION),
        ("tan", "Tangent", CompletionItemKind::FUNCTION),
        ("log", "Natural logarithm", CompletionItemKind::FUNCTION),
        ("exp", "Exponential", CompletionItemKind::FUNCTION),
        ("floor", "Floor function", CompletionItemKind::FUNCTION),
        ("ceil", "Ceiling function", CompletionItemKind::FUNCTION),
        ("abs", "Absolute value (f64)", CompletionItemKind::FUNCTION),
        (
            "abs_i64",
            "Absolute value (i64)",
            CompletionItemKind::FUNCTION,
        ),
        ("min", "Minimum (f64)", CompletionItemKind::FUNCTION),
        ("max", "Maximum (f64)", CompletionItemKind::FUNCTION),
    ];

    math_items
        .iter()
        .map(|(name, doc, kind)| CompletionItem {
            label: name.to_string(),
            kind: Some(*kind),
            detail: Some(doc.to_string()),
            ..Default::default()
        })
        .collect()
}

fn io_completions() -> Vec<CompletionItem> {
    let io_items = [
        ("read_i64", "Read integer from stdin", "read_i64()"),
        ("read_f64", "Read float from stdin", "read_f64()"),
        (
            "read_line",
            "Read line from stdin",
            "read_line(${1:buffer}, ${2:max_len})",
        ),
        ("read_char", "Read character from stdin", "read_char()"),
        (
            "prompt_i64",
            "Prompt and read integer",
            "prompt_i64(${1:prompt})",
        ),
        (
            "prompt_f64",
            "Prompt and read float",
            "prompt_f64(${1:prompt})",
        ),
    ];

    io_items
        .iter()
        .map(|(name, doc, snippet)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(doc.to_string()),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

fn constructor_completions() -> Vec<CompletionItem> {
    let constructors = [
        ("Some", "Option::Some variant", "Some(${1:value})"),
        ("None", "Option::None variant", "None"),
        ("Ok", "Result::Ok variant", "Ok(${1:value})"),
        ("Err", "Result::Err variant", "Err(${1:error})"),
    ];

    constructors
        .iter()
        .map(|(name, doc, snippet)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::CONSTRUCTOR),
            detail: Some(doc.to_string()),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

fn document_symbol_completions(
    backend: &VaisBackend,
    uri: &tower_lsp::lsp_types::Url,
) -> Vec<CompletionItem> {
    let mut items = vec![];

    if let Some(doc) = backend.documents.get(uri) {
        if let Some(ast) = &doc.ast {
            for item in &ast.items {
                match &item.node {
                    vais_ast::Item::Function(f) => {
                        let params_str: Vec<String> = f
                            .params
                            .iter()
                            .enumerate()
                            .map(|(i, p)| format!("${{{}:{}}}", i + 1, p.name.node))
                            .collect();

                        let ret_str = f
                            .ret_type
                            .as_ref()
                            .map(|t| format!(" -> {:?}", t.node))
                            .unwrap_or_default();

                        items.push(CompletionItem {
                            label: f.name.node.clone(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: Some(format!(
                                "fn({}){}",
                                f.params
                                    .iter()
                                    .map(|p| format!("{}: {:?}", p.name.node, p.ty.node))
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                ret_str
                            )),
                            insert_text: Some(format!(
                                "{}({})",
                                f.name.node,
                                params_str.join(", ")
                            )),
                            insert_text_format: Some(InsertTextFormat::SNIPPET),
                            ..Default::default()
                        });
                    }
                    vais_ast::Item::Struct(s) => {
                        // Add struct name
                        items.push(CompletionItem {
                            label: s.name.node.clone(),
                            kind: Some(CompletionItemKind::STRUCT),
                            detail: Some("Struct".to_string()),
                            ..Default::default()
                        });

                        // Add struct literal completion
                        let fields_str: Vec<String> = s
                            .fields
                            .iter()
                            .enumerate()
                            .map(|(i, f)| {
                                format!("{}: ${{{}:{}}}", f.name.node, i + 1, f.name.node)
                            })
                            .collect();

                        items.push(CompletionItem {
                            label: format!("{} {{ }}", s.name.node),
                            kind: Some(CompletionItemKind::CONSTRUCTOR),
                            detail: Some("Struct literal".to_string()),
                            insert_text: Some(format!(
                                "{} {{ {} }}",
                                s.name.node,
                                fields_str.join(", ")
                            )),
                            insert_text_format: Some(InsertTextFormat::SNIPPET),
                            ..Default::default()
                        });
                    }
                    vais_ast::Item::Enum(e) => {
                        items.push(CompletionItem {
                            label: e.name.node.clone(),
                            kind: Some(CompletionItemKind::ENUM),
                            detail: Some("Enum".to_string()),
                            ..Default::default()
                        });

                        // Add enum variants
                        for variant in &e.variants {
                            let variant_text = match &variant.fields {
                                vais_ast::VariantFields::Unit => variant.name.node.clone(),
                                vais_ast::VariantFields::Tuple(types) => {
                                    let fields_str: Vec<String> = types
                                        .iter()
                                        .enumerate()
                                        .map(|(i, _)| format!("${{{}}}", i + 1))
                                        .collect();
                                    format!("{}({})", variant.name.node, fields_str.join(", "))
                                }
                                vais_ast::VariantFields::Struct(fields) => {
                                    let fields_str: Vec<String> = fields
                                        .iter()
                                        .enumerate()
                                        .map(|(i, f)| {
                                            format!(
                                                "{}: ${{{}:{}}}",
                                                f.name.node,
                                                i + 1,
                                                f.name.node
                                            )
                                        })
                                        .collect();
                                    format!("{} {{ {} }}", variant.name.node, fields_str.join(", "))
                                }
                            };

                            items.push(CompletionItem {
                                label: variant.name.node.clone(),
                                kind: Some(CompletionItemKind::ENUM_MEMBER),
                                detail: Some(format!("{}::{}", e.name.node, variant.name.node)),
                                insert_text: Some(variant_text),
                                insert_text_format: Some(InsertTextFormat::SNIPPET),
                                ..Default::default()
                            });
                        }
                    }
                    vais_ast::Item::Trait(t) => {
                        items.push(CompletionItem {
                            label: t.name.node.clone(),
                            kind: Some(CompletionItemKind::INTERFACE),
                            detail: Some("Trait".to_string()),
                            ..Default::default()
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    items
}
