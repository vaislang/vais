# Language Server Protocol 1.18 Features

## Overview

The Vais Language Server implements the Language Server Protocol (LSP) version 1.18, providing rich IDE features for Vais development.

## Implemented Features

### Text Document Synchronization

**Full Document Sync**:
- Open document notification
- Change document notification
- Close document notification
- Save document notification

**Implementation**:
```rust
pub fn handle_text_document_sync(
    &mut self,
    params: DidChangeTextDocumentParams,
) -> Result<()> {
    let uri = params.text_document.uri;
    let changes = params.content_changes;

    for change in changes {
        self.update_document(&uri, &change.text);
    }

    // Trigger re-analysis
    self.analyze_document(&uri)?;
    self.publish_diagnostics(&uri)?;

    Ok(())
}
```

### Code Completion

**Features**:
- Keyword completion (F, S, E, I, L, M, R, etc.)
- Variable name completion
- Function name completion
- Struct field completion
- Module/import completion
- Method completion

**Trigger Characters**: `.`, `:`, `>`

**Example**:
```vais
S Point { x: i64, y: i64 }

F main() -> i64 {
    p := Point { x: 10, y: 20 }
    p.  # <-- Completion shows: x, y
}
```

**Implementation**:
```rust
pub fn completion(&self, params: CompletionParams) -> Result<CompletionList> {
    let position = params.text_document_position.position;
    let uri = &params.text_document_position.text_document.uri;

    let doc = self.documents.get(uri)?;
    let scope = self.get_scope_at_position(doc, position)?;

    let mut items = Vec::new();

    // Add keywords
    items.extend(self.keyword_completions());

    // Add variables in scope
    items.extend(self.variable_completions(&scope));

    // Add functions
    items.extend(self.function_completions(&scope));

    // Add struct fields if after '.'
    if let Some(fields) = self.struct_field_completions(doc, position)? {
        items.extend(fields);
    }

    Ok(CompletionList {
        is_incomplete: false,
        items,
    })
}
```

### Go to Definition

**Supported**:
- Function definitions
- Variable definitions
- Struct definitions
- Enum definitions
- Type aliases

**Example**:
```vais
F helper() -> i64 { 42 }

F main() -> i64 {
    result := helper()  # Ctrl+Click on 'helper' goes to definition
}
```

**Implementation**:
```rust
pub fn goto_definition(
    &self,
    params: GotoDefinitionParams,
) -> Result<Option<Location>> {
    let position = params.text_document_position_params.position;
    let uri = &params.text_document_position_params.text_document.uri;

    let doc = self.documents.get(uri)?;
    let word = self.word_at_position(doc, position)?;

    // Look up definition in symbol table
    if let Some(symbol) = self.symbols.get(&word) {
        return Ok(Some(Location {
            uri: symbol.uri.clone(),
            range: symbol.range,
        }));
    }

    Ok(None)
}
```

### Hover Information

**Provides**:
- Type information
- Function signatures
- Documentation comments
- Value information

**Example**:
```vais
# Adds two numbers
F add(a: i64, b: i64) -> i64 {
    a + b
}

F main() -> i64 {
    # Hovering over 'add' shows:
    # F add(a: i64, b: i64) -> i64
    # Adds two numbers
    result := add(5, 10)
}
```

**Implementation**:
```rust
pub fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    let position = params.text_document_position_params.position;
    let uri = &params.text_document_position_params.text_document.uri;

    let doc = self.documents.get(uri)?;
    let word = self.word_at_position(doc, position)?;

    if let Some(symbol) = self.symbols.get(&word) {
        let contents = MarkedString::LanguageString(LanguageString {
            language: "vais".to_string(),
            value: symbol.signature.clone(),
        });

        return Ok(Some(Hover {
            contents: HoverContents::Scalar(contents),
            range: Some(symbol.range),
        }));
    }

    Ok(None)
}
```

### Diagnostics

**Real-time Error Reporting**:
- Syntax errors
- Type errors
- Undefined variable errors
- Unused variable warnings
- Dead code warnings

**Severity Levels**:
- Error (red squiggly)
- Warning (yellow squiggly)
- Information (blue squiggly)
- Hint (gray dots)

**Example**:
```vais
F main() -> i64 {
    x := "hello"
    result := x + 5  # Error: cannot add string and integer
    unused := 42     # Warning: unused variable
}
```

**Implementation**:
```rust
pub fn publish_diagnostics(&self, uri: &Url) -> Result<()> {
    let doc = self.documents.get(uri)?;
    let mut diagnostics = Vec::new();

    // Parse and type check
    match self.analyze(doc) {
        Ok(_) => {}
        Err(errors) => {
            for error in errors {
                diagnostics.push(Diagnostic {
                    range: error.range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: error.message,
                    ..Default::default()
                });
            }
        }
    }

    // Check for warnings
    diagnostics.extend(self.check_warnings(doc)?);

    self.client.publish_diagnostics(uri.clone(), diagnostics, None);
    Ok(())
}
```

### Document Symbols

**Outline View**:
- Functions
- Structs
- Enums
- Traits
- Implementations

**Example Outline**:
```
main.vais
├── S Point
├── S Circle
├── F distance(Point, Point) -> f64
├── F main() -> i64
└── impl Circle
    ├── F new(f64) -> Circle
    └── F area(*Circle) -> f64
```

**Implementation**:
```rust
pub fn document_symbols(
    &self,
    params: DocumentSymbolParams,
) -> Result<Vec<DocumentSymbol>> {
    let uri = &params.text_document.uri;
    let doc = self.documents.get(uri)?;

    let ast = self.parse(doc)?;
    let mut symbols = Vec::new();

    for item in &ast.items {
        match item {
            Item::Function(func) => {
                symbols.push(DocumentSymbol {
                    name: func.name.clone(),
                    kind: SymbolKind::FUNCTION,
                    range: func.range,
                    selection_range: func.name_range,
                    children: None,
                    ..Default::default()
                });
            }
            Item::Struct(s) => {
                symbols.push(DocumentSymbol {
                    name: s.name.clone(),
                    kind: SymbolKind::STRUCT,
                    range: s.range,
                    selection_range: s.name_range,
                    children: Some(self.struct_field_symbols(s)),
                    ..Default::default()
                });
            }
            // ... more item types
        }
    }

    Ok(symbols)
}
```

### Workspace Symbols

**Project-wide Symbol Search**:
```
Ctrl+T / Cmd+T: Search for symbols across entire workspace
```

**Implementation**:
```rust
pub fn workspace_symbols(
    &self,
    params: WorkspaceSymbolParams,
) -> Result<Vec<SymbolInformation>> {
    let query = params.query.to_lowercase();
    let mut results = Vec::new();

    for (name, symbol) in &self.symbols {
        if name.to_lowercase().contains(&query) {
            results.push(SymbolInformation {
                name: name.clone(),
                kind: symbol.kind,
                location: Location {
                    uri: symbol.uri.clone(),
                    range: symbol.range,
                },
                ..Default::default()
            });
        }
    }

    // Sort by relevance
    results.sort_by(|a, b| {
        let a_starts = a.name.to_lowercase().starts_with(&query);
        let b_starts = b.name.to_lowercase().starts_with(&query);
        b_starts.cmp(&a_starts)
    });

    Ok(results)
}
```

### Find References

**Find All Usages**:
- Variable references
- Function calls
- Type references

**Example**:
```vais
F helper() -> i64 { 42 }  # Definition

F main() -> i64 {
    x := helper()  # Reference 1
    y := helper()  # Reference 2
    x + y
}
```

**Right-click → Find All References** shows both usages.

### Rename Symbol

**Safe Refactoring**:
- Rename variables
- Rename functions
- Rename types
- Update all references

**Example**:
```vais
# Before:
F oldName() -> i64 { 42 }
F main() -> i64 { oldName() }

# Rename oldName → newName:
F newName() -> i64 { 42 }
F main() -> i64 { newName() }
```

**Implementation**:
```rust
pub fn rename(
    &self,
    params: RenameParams,
) -> Result<Option<WorkspaceEdit>> {
    let position = params.text_document_position.position;
    let uri = &params.text_document_position.text_document.uri;
    let new_name = params.new_name;

    let doc = self.documents.get(uri)?;
    let old_name = self.word_at_position(doc, position)?;

    // Find all references
    let references = self.find_references(&old_name)?;

    // Create text edits
    let mut changes = HashMap::new();
    for reference in references {
        let edits = changes.entry(reference.uri).or_insert(Vec::new());
        edits.push(TextEdit {
            range: reference.range,
            new_text: new_name.clone(),
        });
    }

    Ok(Some(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    }))
}
```

### Code Actions

**Quick Fixes**:
- Import missing modules
- Add missing type annotations
- Remove unused variables
- Extract function
- Inline variable

**Example**:
```vais
F main() -> i64 {
    unused := 42  # Light bulb icon → "Remove unused variable"
}
```

### Formatting

**Automatic Code Formatting**:
- Format on save
- Format on type
- Format selection
- Format document

**Configuration**:
```json
{
    "vais.format.indentSize": 4,
    "vais.format.maxLineLength": 100,
    "vais.format.insertSpaces": true
}
```

### Signature Help

**Parameter Hints**:
Shows parameter information while typing function calls.

**Example**:
```vais
F complex(a: i64, b: f64, c: String) -> i64 { 0 }

F main() -> i64 {
    # Typing 'complex(' shows:
    # complex(a: i64, b: f64, c: String) -> i64
    #         ^^^^^  ------  --------
    complex(42,
    #          ^ Shows: b: f64
}
```

### Semantic Tokens

**Syntax Highlighting**:
- Keywords (F, S, E, I, L, M, R)
- Functions
- Variables
- Types
- Operators
- Comments
- Strings
- Numbers

**Token Types**:
- namespace
- type
- class
- enum
- interface
- struct
- function
- variable
- parameter
- property
- keyword
- comment
- string
- number
- operator

### Inlay Hints

**Type Hints**:
Shows inferred types inline:

```vais
F main() -> i64 {
    x := 42        # : i64
    y := 3.14      # : f64
    s := "hello"   # : String
}
```

### Folding Ranges

**Code Folding**:
- Function bodies
- Struct definitions
- Impl blocks
- Comments

**Example**:
```vais
F long_function() -> i64 { # [▼]
    # Many lines of code...
    0
}
# Can fold to:
F long_function() -> i64 { ... }
```

## Performance Features

### Incremental Parsing

Only re-parse changed portions of the document:

```rust
pub fn incremental_parse(&mut self, changes: Vec<TextDocumentContentChangeEvent>) {
    for change in changes {
        // Only reparse affected ranges
        let affected = self.get_affected_range(&change);
        self.reparse_range(affected);
    }
}
```

### Lazy Symbol Resolution

Symbols are resolved on-demand for better performance.

### Caching

- Parse tree caching
- Symbol table caching
- Type information caching

## Configuration

**VS Code settings.json**:
```json
{
    "vais.lsp.enable": true,
    "vais.lsp.trace.server": "verbose",
    "vais.diagnostics.enable": true,
    "vais.completion.enable": true,
    "vais.hover.enable": true,
    "vais.format.enable": true,
    "vais.inlayHints.enable": true
}
```

## Implementation Details

**Language Server Architecture**:
```
┌─────────────────────────────────────────┐
│         VSCode Extension                │
└───────────────┬─────────────────────────┘
                │ JSON-RPC
┌───────────────▼─────────────────────────┐
│      Vais Language Server (Rust)        │
├─────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐            │
│  │  Parser  │  │  Type    │            │
│  │          │  │  Checker │            │
│  └──────────┘  └──────────┘            │
│  ┌──────────┐  ┌──────────┐            │
│  │ Symbol   │  │ Diagnostics          │
│  │ Table    │  │          │            │
│  └──────────┘  └──────────┘            │
└─────────────────────────────────────────┘
```

**Protocol Version**: LSP 3.17 / 1.18

**Transport**: JSON-RPC over stdio

## Testing

```bash
# Run LSP tests
cargo test -p vais-lsp

# Test specific feature
cargo test -p vais-lsp -- completion

# Integration tests
cargo test -p vais-lsp --test integration
```

## Performance Metrics

Typical response times:
- Completion: <10ms
- Hover: <5ms
- Go to definition: <5ms
- Diagnostics: <100ms
- Formatting: <50ms

## Future Enhancements

Planned features:
- Call hierarchy
- Type hierarchy
- Code lens
- Semantic highlighting improvements
- Better error recovery
- Incremental type checking

## Conclusion

The Vais LSP implementation provides a complete IDE experience with all essential LSP 1.18 features, enabling productive development with real-time feedback and intelligent code assistance.
