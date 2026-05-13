//! Completion handler implementation

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::ai_completion::{generate_ai_completions, CompletionContext as AiContext};
use crate::backend::VaisBackend;
use crate::type_resolve::{CompletionKind, LspType, TypeContext};

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
        // Try type-aware completion first
        let type_completions = type_aware_method_completions(backend, uri, position);
        if !type_completions.is_empty() {
            items.extend(type_completions);
        } else {
            // Fallback to generic method completions
            items.extend(method_completions(backend, uri));
        }
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

/// Type-aware method/field completions using AST-based type resolution
fn type_aware_method_completions(
    backend: &VaisBackend,
    uri: &tower_lsp::lsp_types::Url,
    position: Position,
) -> Vec<CompletionItem> {
    let mut items = vec![];

    let doc = match backend.documents.get(uri) {
        Some(doc) => doc,
        None => return items,
    };

    let ast = match &doc.ast {
        Some(ast) => ast,
        None => return items,
    };

    // Extract the identifier before the dot
    let line_idx = position.line as usize;
    let line = match doc.content.get_line(line_idx) {
        Some(line) => line,
        None => return items,
    };

    let line_str: String = line.chars().collect();
    let col = position.character as usize;

    // Walk backwards from the dot to find the identifier
    if col < 2 {
        return items;
    }

    // Find the expression before the dot
    let before_dot = &line_str[..col.saturating_sub(1)];
    let ident_before_dot = extract_trailing_identifier(before_dot);
    if ident_before_dot.is_empty() {
        return items;
    }

    // Build type context from AST
    let mut type_ctx = TypeContext::from_module(ast);

    // Collect variable bindings up to cursor position
    let line_start = doc.content.line_to_char(line_idx);
    let cursor_offset = line_start + col;
    type_ctx.collect_variable_bindings(ast, cursor_offset);

    // Resolve the type of the identifier before the dot
    let resolved_type: LspType = if let Some(ty) = type_ctx.variable_types.get(ident_before_dot) {
        ty.clone()
    } else {
        // Check if it's a direct struct/enum name (static access)
        if type_ctx.structs.contains_key(ident_before_dot)
            || type_ctx.enum_variants.contains_key(ident_before_dot)
        {
            LspType::Named(ident_before_dot.to_string())
        } else {
            return items;
        }
    };

    // Get completions for the resolved type
    let type_name = match &resolved_type {
        LspType::Named(name) => name.clone(),
        LspType::Primitive(name) => name.clone(),
        LspType::Array(_) => "Vec".to_string(),
        LspType::Optional(_) => "Option".to_string(),
        LspType::Result(_, _) => "Result".to_string(),
        _ => return items,
    };

    let completions = type_ctx.get_dot_completions(&type_name);
    for entry in completions {
        let (kind, sort_prefix) = match entry.kind {
            CompletionKind::Field => (CompletionItemKind::FIELD, "0"), // Fields sort first
            CompletionKind::Method => (CompletionItemKind::METHOD, "1"),
        };

        let detail = if let Some(ref trait_name) = entry.from_trait {
            format!("{} (from trait {})", entry.detail, trait_name)
        } else {
            entry.detail
        };

        items.push(CompletionItem {
            label: entry.label.clone(),
            kind: Some(kind),
            detail: Some(detail),
            insert_text: Some(entry.insert_text),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some(format!("{}{}", sort_prefix, entry.label)),
            ..Default::default()
        });
    }

    // For well-known types, add builtin methods
    match type_name.as_str() {
        "Vec" | "Array" => {
            add_vec_builtins(&mut items);
        }
        "Option" => {
            add_option_builtins(&mut items);
        }
        "Result" => {
            add_result_builtins(&mut items);
        }
        "str" | "String" => {
            add_string_builtins(&mut items);
        }
        _ => {}
    }

    items
}

/// Extract trailing identifier from a string (e.g., "  foo.bar" -> "bar", "x" -> "x")
fn extract_trailing_identifier(s: &str) -> &str {
    let s = s.trim_end();
    let start = s
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);
    &s[start..]
}

fn add_vec_builtins(items: &mut Vec<CompletionItem>) {
    let methods = [
        ("len", "Get length", "len()", "fn() -> i64"),
        ("is_empty", "Check if empty", "is_empty()", "fn() -> bool"),
        ("push", "Push element to end", "push(${1:value})", "fn(T)"),
        (
            "pop",
            "Remove and return last element",
            "pop()",
            "fn() -> T",
        ),
        (
            "get",
            "Get element at index",
            "get(${1:index})",
            "fn(i64) -> Option<T>",
        ),
        ("first", "Get first element", "first()", "fn() -> Option<T>"),
        ("last", "Get last element", "last()", "fn() -> Option<T>"),
        ("clear", "Remove all elements", "clear()", "fn()"),
        (
            "contains",
            "Check if contains element",
            "contains(${1:value})",
            "fn(T) -> bool",
        ),
        ("iter", "Get iterator", "iter()", "fn() -> Iterator<T>"),
        ("reverse", "Reverse in place", "reverse()", "fn()"),
        ("sort", "Sort in place", "sort()", "fn()"),
        ("clone", "Clone the vector", "clone()", "fn() -> Vec<T>"),
    ];
    for (name, doc, snippet, sig) in methods {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some(sig.to_string()),
            documentation: Some(Documentation::String(doc.to_string())),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some(format!("1{}", name)),
            ..Default::default()
        });
    }
}

fn add_option_builtins(items: &mut Vec<CompletionItem>) {
    let methods = [
        ("is_some", "Check if Some", "is_some()", "fn() -> bool"),
        ("is_none", "Check if None", "is_none()", "fn() -> bool"),
        (
            "unwrap",
            "Unwrap value (panics if None)",
            "unwrap()",
            "fn() -> T",
        ),
        (
            "unwrap_or",
            "Unwrap with default",
            "unwrap_or(${1:default})",
            "fn(T) -> T",
        ),
        (
            "map",
            "Transform inner value",
            "map(|${1:v}| ${2:expr})",
            "fn(fn(T) -> U) -> Option<U>",
        ),
        (
            "and_then",
            "Chain computation",
            "and_then(|${1:v}| ${2:expr})",
            "fn(fn(T) -> Option<U>) -> Option<U>",
        ),
    ];
    for (name, doc, snippet, sig) in methods {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some(sig.to_string()),
            documentation: Some(Documentation::String(doc.to_string())),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some(format!("1{}", name)),
            ..Default::default()
        });
    }
}

fn add_result_builtins(items: &mut Vec<CompletionItem>) {
    let methods = [
        ("is_ok", "Check if Ok", "is_ok()", "fn() -> bool"),
        ("is_err", "Check if Err", "is_err()", "fn() -> bool"),
        (
            "unwrap",
            "Unwrap Ok (panics if Err)",
            "unwrap()",
            "fn() -> T",
        ),
        (
            "unwrap_or",
            "Unwrap with default",
            "unwrap_or(${1:default})",
            "fn(T) -> T",
        ),
        ("unwrap_err", "Unwrap Err", "unwrap_err()", "fn() -> E"),
        (
            "map",
            "Transform Ok value",
            "map(|${1:v}| ${2:expr})",
            "fn(fn(T) -> U) -> Result<U, E>",
        ),
        (
            "map_err",
            "Transform Err value",
            "map_err(|${1:e}| ${2:expr})",
            "fn(fn(E) -> F) -> Result<T, F>",
        ),
    ];
    for (name, doc, snippet, sig) in methods {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some(sig.to_string()),
            documentation: Some(Documentation::String(doc.to_string())),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some(format!("1{}", name)),
            ..Default::default()
        });
    }
}

fn add_string_builtins(items: &mut Vec<CompletionItem>) {
    let methods = [
        ("len", "Get string length", "len()", "fn() -> i64"),
        ("is_empty", "Check if empty", "is_empty()", "fn() -> bool"),
        (
            "contains",
            "Check if contains substring",
            "contains(${1:substr})",
            "fn(str) -> bool",
        ),
        (
            "starts_with",
            "Check if starts with prefix",
            "starts_with(${1:prefix})",
            "fn(str) -> bool",
        ),
        (
            "ends_with",
            "Check if ends with suffix",
            "ends_with(${1:suffix})",
            "fn(str) -> bool",
        ),
        (
            "to_uppercase",
            "Convert to uppercase",
            "to_uppercase()",
            "fn() -> str",
        ),
        (
            "to_lowercase",
            "Convert to lowercase",
            "to_lowercase()",
            "fn() -> str",
        ),
        ("trim", "Trim whitespace", "trim()", "fn() -> str"),
        (
            "split",
            "Split by delimiter",
            "split(${1:delimiter})",
            "fn(str) -> Vec<str>",
        ),
        ("clone", "Clone the string", "clone()", "fn() -> str"),
    ];
    for (name, doc, snippet, sig) in methods {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some(sig.to_string()),
            documentation: Some(Documentation::String(doc.to_string())),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some(format!("1{}", name)),
            ..Default::default()
        });
    }
}
