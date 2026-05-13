use super::*;
use ropey::Rope;

// ========== position_in_range tests ==========

#[test]
fn test_position_in_range_inside() {
    let range = Range {
        start: Position::new(1, 5),
        end: Position::new(3, 10),
    };
    assert!(position_in_range(&Position::new(2, 0), &range));
}

#[test]
fn test_position_in_range_at_start() {
    let range = Range {
        start: Position::new(1, 5),
        end: Position::new(3, 10),
    };
    assert!(position_in_range(&Position::new(1, 5), &range));
}

#[test]
fn test_position_in_range_at_end() {
    let range = Range {
        start: Position::new(1, 5),
        end: Position::new(3, 10),
    };
    assert!(position_in_range(&Position::new(3, 10), &range));
}

#[test]
fn test_position_in_range_before() {
    let range = Range {
        start: Position::new(1, 5),
        end: Position::new(3, 10),
    };
    assert!(!position_in_range(&Position::new(0, 0), &range));
}

#[test]
fn test_position_in_range_after() {
    let range = Range {
        start: Position::new(1, 5),
        end: Position::new(3, 10),
    };
    assert!(!position_in_range(&Position::new(4, 0), &range));
}

#[test]
fn test_position_in_range_same_line_before_start() {
    let range = Range {
        start: Position::new(1, 5),
        end: Position::new(3, 10),
    };
    assert!(!position_in_range(&Position::new(1, 3), &range));
}

#[test]
fn test_position_in_range_same_line_after_end() {
    let range = Range {
        start: Position::new(1, 5),
        end: Position::new(3, 10),
    };
    assert!(!position_in_range(&Position::new(3, 15), &range));
}

#[test]
fn test_position_in_range_single_line() {
    let range = Range {
        start: Position::new(5, 2),
        end: Position::new(5, 8),
    };
    assert!(position_in_range(&Position::new(5, 5), &range));
    assert!(!position_in_range(&Position::new(5, 1), &range));
    assert!(!position_in_range(&Position::new(5, 9), &range));
}

// ========== get_builtin_hover tests ==========

#[test]
fn test_builtin_hover_puts() {
    let hover = get_builtin_hover("puts");
    assert!(hover.is_some());
    let hover = hover.unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(str) -> i64"));
        assert!(markup.value.contains("Print a string"));
    } else {
        panic!("Expected markup content");
    }
}

#[test]
fn test_builtin_hover_malloc() {
    let hover = get_builtin_hover("malloc");
    assert!(hover.is_some());
    let hover = hover.unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(i64) -> i64"));
        assert!(markup.value.contains("Allocate"));
    } else {
        panic!("Expected markup content");
    }
}

#[test]
fn test_builtin_hover_sqrt() {
    let hover = get_builtin_hover("sqrt");
    assert!(hover.is_some());
    let hover = hover.unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(f64) -> f64"));
        assert!(markup.value.contains("Square root"));
    } else {
        panic!("Expected markup content");
    }
}

#[test]
fn test_builtin_hover_unknown() {
    assert!(get_builtin_hover("unknown_function").is_none());
}

#[test]
fn test_builtin_hover_pi() {
    let hover = get_builtin_hover("PI");
    assert!(hover.is_some());
}

#[test]
fn test_builtin_hover_all_io() {
    // Test all IO builtins exist
    for name in &["puts", "putchar", "read_i64", "read_f64", "read_char"] {
        assert!(
            get_builtin_hover(name).is_some(),
            "Missing hover for {}",
            name
        );
    }
}

#[test]
fn test_builtin_hover_all_math() {
    for name in &[
        "sqrt", "sin", "cos", "tan", "pow", "log", "exp", "floor", "ceil", "round", "abs",
        "abs_i64", "min", "max",
    ] {
        assert!(
            get_builtin_hover(name).is_some(),
            "Missing hover for {}",
            name
        );
    }
}

#[test]
fn test_builtin_hover_memory() {
    for name in &["malloc", "free", "memcpy", "strlen"] {
        assert!(
            get_builtin_hover(name).is_some(),
            "Missing hover for {}",
            name
        );
    }
}

#[test]
fn test_builtin_hover_load_store() {
    for name in &["load_i64", "store_i64", "load_byte", "store_byte"] {
        assert!(
            get_builtin_hover(name).is_some(),
            "Missing hover for {}",
            name
        );
    }
}

#[test]
fn test_builtin_hover_markup_format() {
    let hover = get_builtin_hover("puts").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert_eq!(markup.kind, MarkupKind::Markdown);
        assert!(markup.value.contains("```vais"));
        assert!(markup.value.contains("Built-in function"));
    } else {
        panic!("Expected markup content");
    }
}

// ========== Document / Symbol Def / Symbol Ref struct tests ==========

#[test]
fn test_symbol_def_construction() {
    let def = SymbolDef {
        name: "test_func".to_string(),
        kind: SymbolKind::FUNCTION,
        span: Span::new(0, 10),
    };
    assert_eq!(def.name, "test_func");
    assert_eq!(def.kind, SymbolKind::FUNCTION);
}

#[test]
fn test_symbol_ref_construction() {
    let sym_ref = SymbolRef {
        name: "x".to_string(),
        span: Span::new(5, 6),
    };
    assert_eq!(sym_ref.name, "x");
}

#[test]
fn test_folding_range_info_construction() {
    let info = FoldingRangeInfo {
        start_line: 1,
        end_line: 10,
        kind: Some(FoldingRangeKind::Region),
    };
    assert_eq!(info.start_line, 1);
    assert_eq!(info.end_line, 10);
}

#[test]
fn test_inlay_hint_info_construction() {
    let hint = InlayHintInfo {
        position: 42,
        label: ": i64".to_string(),
        kind: InlayHintKind::TYPE,
    };
    assert_eq!(hint.position, 42);
    assert_eq!(hint.label, ": i64");
}

#[test]
fn test_call_graph_entry_construction() {
    let entry = CallGraphEntry {
        caller: "main".to_string(),
        caller_span: Span::new(0, 4),
        callee: "foo".to_string(),
        call_span: Span::new(20, 23),
    };
    assert_eq!(entry.caller, "main");
    assert_eq!(entry.callee, "foo");
}

#[test]
fn test_document_construction() {
    let doc = Document {
        content: Rope::from_str("F main() -> i64 = 42"),
        ast: None,
        version: 1,
        symbol_cache: None,
    };
    assert_eq!(doc.version, 1);
    assert!(doc.ast.is_none());
    assert!(doc.symbol_cache.is_none());
}

#[test]
fn test_document_with_ast() {
    let source = "F main() -> i64 = 42";
    let ast = vais_parser::parse(source).ok();
    let doc = Document {
        content: Rope::from_str(source),
        ast,
        version: 2,
        symbol_cache: None,
    };
    assert_eq!(doc.version, 2);
    assert!(doc.ast.is_some());
}

#[test]
fn test_symbol_cache_construction() {
    let cache = SymbolCache {
        version: 1,
        definitions: vec![SymbolDef {
            name: "foo".to_string(),
            kind: SymbolKind::FUNCTION,
            span: Span::new(2, 5),
        }],
        references: vec![],
        call_graph: vec![],
    };
    assert_eq!(cache.version, 1);
    assert_eq!(cache.definitions.len(), 1);
    assert_eq!(cache.definitions[0].name, "foo");
}

// ========== Additional position_in_range edge case tests ==========

#[test]
fn test_position_in_range_single_char() {
    let range = Range {
        start: Position::new(0, 0),
        end: Position::new(0, 1),
    };
    assert!(position_in_range(&Position::new(0, 0), &range));
    assert!(position_in_range(&Position::new(0, 1), &range));
    assert!(!position_in_range(&Position::new(0, 2), &range));
}

#[test]
fn test_position_in_range_zero_width() {
    let range = Range {
        start: Position::new(5, 10),
        end: Position::new(5, 10),
    };
    assert!(position_in_range(&Position::new(5, 10), &range));
    assert!(!position_in_range(&Position::new(5, 9), &range));
    assert!(!position_in_range(&Position::new(5, 11), &range));
}

#[test]
fn test_position_in_range_multiline_middle() {
    let range = Range {
        start: Position::new(1, 0),
        end: Position::new(10, 0),
    };
    assert!(position_in_range(&Position::new(5, 50), &range));
}

#[test]
fn test_position_in_range_start_line_boundary() {
    let range = Range {
        start: Position::new(2, 5),
        end: Position::new(5, 10),
    };
    assert!(!position_in_range(&Position::new(2, 4), &range));
    assert!(position_in_range(&Position::new(2, 5), &range));
    assert!(position_in_range(&Position::new(2, 6), &range));
}

#[test]
fn test_position_in_range_end_line_boundary() {
    let range = Range {
        start: Position::new(2, 5),
        end: Position::new(5, 10),
    };
    assert!(position_in_range(&Position::new(5, 9), &range));
    assert!(position_in_range(&Position::new(5, 10), &range));
    assert!(!position_in_range(&Position::new(5, 11), &range));
}

// ========== Additional builtin hover tests ==========

#[test]
fn test_builtin_hover_print_i64() {
    let hover = get_builtin_hover("print_i64").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(i64) -> i64"));
    }
}

#[test]
fn test_builtin_hover_print_f64() {
    let hover = get_builtin_hover("print_f64").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(f64) -> i64"));
    }
}

#[test]
fn test_builtin_hover_free() {
    let hover = get_builtin_hover("free").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("Free heap memory"));
    }
}

#[test]
fn test_builtin_hover_memcpy() {
    let hover = get_builtin_hover("memcpy").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(i64, i64, i64) -> i64"));
    }
}

#[test]
fn test_builtin_hover_strlen() {
    let hover = get_builtin_hover("strlen").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(i64) -> i64"));
        assert!(markup.value.contains("length"));
    }
}

#[test]
fn test_builtin_hover_putchar() {
    let hover = get_builtin_hover("putchar").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(i64) -> i64"));
    }
}

#[test]
fn test_builtin_hover_tau() {
    let hover = get_builtin_hover("TAU").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("6.28318"));
    }
}

#[test]
fn test_builtin_hover_sin() {
    let hover = get_builtin_hover("sin").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(f64) -> f64"));
        assert!(markup.value.contains("Sine"));
    }
}

#[test]
fn test_builtin_hover_cos() {
    let hover = get_builtin_hover("cos").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("Cosine"));
    }
}

#[test]
fn test_builtin_hover_tan() {
    let hover = get_builtin_hover("tan").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("Tangent"));
    }
}

#[test]
fn test_builtin_hover_pow() {
    let hover = get_builtin_hover("pow").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(f64, f64) -> f64"));
    }
}

#[test]
fn test_builtin_hover_log() {
    let hover = get_builtin_hover("log").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("Natural logarithm"));
    }
}

#[test]
fn test_builtin_hover_exp() {
    let hover = get_builtin_hover("exp").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("Exponential"));
    }
}

#[test]
fn test_builtin_hover_floor() {
    let hover = get_builtin_hover("floor").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("Round down"));
    }
}

#[test]
fn test_builtin_hover_ceil() {
    let hover = get_builtin_hover("ceil").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("Round up"));
    }
}

#[test]
fn test_builtin_hover_round() {
    let hover = get_builtin_hover("round").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("Round to nearest"));
    }
}

#[test]
fn test_builtin_hover_abs() {
    let hover = get_builtin_hover("abs").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("Absolute value"));
    }
}

#[test]
fn test_builtin_hover_abs_i64() {
    let hover = get_builtin_hover("abs_i64").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(i64) -> i64"));
    }
}

#[test]
fn test_builtin_hover_min() {
    let hover = get_builtin_hover("min").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(f64, f64) -> f64"));
    }
}

#[test]
fn test_builtin_hover_max() {
    let hover = get_builtin_hover("max").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(f64, f64) -> f64"));
    }
}

#[test]
fn test_builtin_hover_read_i64() {
    let hover = get_builtin_hover("read_i64").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn() -> i64"));
    }
}

#[test]
fn test_builtin_hover_read_f64() {
    let hover = get_builtin_hover("read_f64").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn() -> f64"));
    }
}

#[test]
fn test_builtin_hover_read_line() {
    let hover = get_builtin_hover("read_line").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("Read line"));
    }
}

#[test]
fn test_builtin_hover_read_char() {
    let hover = get_builtin_hover("read_char").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("Read single character"));
    }
}

#[test]
fn test_builtin_hover_load_i64() {
    let hover = get_builtin_hover("load_i64").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("Load a 64-bit integer"));
    }
}

#[test]
fn test_builtin_hover_store_i64() {
    let hover = get_builtin_hover("store_i64").unwrap();
    if let HoverContents::Markup(markup) = &hover.contents {
        assert!(markup.value.contains("fn(i64, i64) -> i64"));
    }
}

#[test]
fn test_builtin_hover_no_range() {
    // All builtin hovers should have range: None
    let hover = get_builtin_hover("puts").unwrap();
    assert!(hover.range.is_none());
}

#[test]
fn test_builtin_hover_case_sensitive() {
    assert!(get_builtin_hover("Puts").is_none());
    assert!(get_builtin_hover("PUTS").is_none());
    assert!(get_builtin_hover("pi").is_none());
}

// ========== Additional struct/clone tests ==========

#[test]
fn test_symbol_def_clone() {
    let def = SymbolDef {
        name: "bar".to_string(),
        kind: SymbolKind::VARIABLE,
        span: Span::new(0, 3),
    };
    let cloned = def.clone();
    assert_eq!(cloned.name, "bar");
    assert_eq!(cloned.kind, SymbolKind::VARIABLE);
}

#[test]
fn test_symbol_ref_clone() {
    let sym_ref = SymbolRef {
        name: "y".to_string(),
        span: Span::new(10, 11),
    };
    let cloned = sym_ref.clone();
    assert_eq!(cloned.name, "y");
}

#[test]
fn test_symbol_cache_clone() {
    let cache = SymbolCache {
        version: 5,
        definitions: vec![],
        references: vec![],
        call_graph: vec![],
    };
    let cloned = cache.clone();
    assert_eq!(cloned.version, 5);
}

#[test]
fn test_call_graph_entry_clone() {
    let entry = CallGraphEntry {
        caller: "a".to_string(),
        caller_span: Span::new(0, 1),
        callee: "b".to_string(),
        call_span: Span::new(5, 6),
    };
    let cloned = entry.clone();
    assert_eq!(cloned.caller, "a");
    assert_eq!(cloned.callee, "b");
}

#[test]
fn test_folding_range_info_clone() {
    let info = FoldingRangeInfo {
        start_line: 0,
        end_line: 100,
        kind: Some(FoldingRangeKind::Imports),
    };
    let cloned = info.clone();
    assert_eq!(cloned.start_line, 0);
    assert_eq!(cloned.end_line, 100);
}

#[test]
fn test_inlay_hint_info_clone() {
    let hint = InlayHintInfo {
        position: 10,
        label: ": f64".to_string(),
        kind: InlayHintKind::TYPE,
    };
    let cloned = hint.clone();
    assert_eq!(cloned.position, 10);
    assert_eq!(cloned.label, ": f64");
}

#[test]
fn test_symbol_def_debug() {
    let def = SymbolDef {
        name: "test".to_string(),
        kind: SymbolKind::STRUCT,
        span: Span::new(0, 4),
    };
    let debug = format!("{:?}", def);
    assert!(debug.contains("test"));
}

#[test]
fn test_symbol_ref_debug() {
    let sym_ref = SymbolRef {
        name: "x".to_string(),
        span: Span::new(0, 1),
    };
    let debug = format!("{:?}", sym_ref);
    assert!(debug.contains("x"));
}

#[test]
fn test_symbol_cache_empty() {
    let cache = SymbolCache {
        version: 0,
        definitions: vec![],
        references: vec![],
        call_graph: vec![],
    };
    assert!(cache.definitions.is_empty());
    assert!(cache.references.is_empty());
    assert!(cache.call_graph.is_empty());
}

#[test]
fn test_symbol_cache_with_references() {
    let cache = SymbolCache {
        version: 1,
        definitions: vec![],
        references: vec![
            SymbolRef {
                name: "x".to_string(),
                span: Span::new(0, 1),
            },
            SymbolRef {
                name: "y".to_string(),
                span: Span::new(5, 6),
            },
        ],
        call_graph: vec![],
    };
    assert_eq!(cache.references.len(), 2);
}

#[test]
fn test_symbol_cache_with_call_graph() {
    let cache = SymbolCache {
        version: 1,
        definitions: vec![],
        references: vec![],
        call_graph: vec![CallGraphEntry {
            caller: "main".to_string(),
            caller_span: Span::new(0, 4),
            callee: "helper".to_string(),
            call_span: Span::new(20, 26),
        }],
    };
    assert_eq!(cache.call_graph.len(), 1);
    assert_eq!(cache.call_graph[0].callee, "helper");
}

#[test]
fn test_document_rope_content() {
    let source = "F main() -> i64 {\n    R 0\n}";
    let doc = Document {
        content: Rope::from_str(source),
        ast: None,
        version: 1,
        symbol_cache: None,
    };
    assert_eq!(doc.content.len_lines(), 3);
}

#[test]
fn test_document_multiple_versions() {
    let doc1 = Document {
        content: Rope::from_str("v1"),
        ast: None,
        version: 1,
        symbol_cache: None,
    };
    let doc2 = Document {
        content: Rope::from_str("v2"),
        ast: None,
        version: 2,
        symbol_cache: None,
    };
    assert!(doc2.version > doc1.version);
}

#[test]
fn test_folding_range_info_without_kind() {
    let info = FoldingRangeInfo {
        start_line: 5,
        end_line: 10,
        kind: None,
    };
    assert!(info.kind.is_none());
}

#[test]
fn test_symbol_def_kinds() {
    let kinds = vec![
        SymbolKind::FUNCTION,
        SymbolKind::VARIABLE,
        SymbolKind::STRUCT,
        SymbolKind::ENUM,
        SymbolKind::INTERFACE,
        SymbolKind::FIELD,
        SymbolKind::ENUM_MEMBER,
    ];
    for kind in kinds {
        let def = SymbolDef {
            name: "x".to_string(),
            kind,
            span: Span::new(0, 1),
        };
        assert_eq!(def.kind, kind);
    }
}

#[test]
fn test_inlay_hint_kinds() {
    let type_hint = InlayHintInfo {
        position: 0,
        label: ": i64".to_string(),
        kind: InlayHintKind::TYPE,
    };
    assert_eq!(type_hint.kind, InlayHintKind::TYPE);

    let param_hint = InlayHintInfo {
        position: 0,
        label: "x: ".to_string(),
        kind: InlayHintKind::PARAMETER,
    };
    assert_eq!(param_hint.kind, InlayHintKind::PARAMETER);
}
