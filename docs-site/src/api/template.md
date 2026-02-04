# Template API Reference

> Template engine with variable interpolation, conditionals, loops, filters, and partials

## Import

```vais
U std/template
```

## Overview

The template module provides a lightweight template engine supporting:
- Variable interpolation `{{ var }}`
- Filters `{{ var | upper }}`
- Conditionals `{% if condition %}...{% endif %}`
- Loops `{% for item in list %}...{% endfor %}`
- Partials `{% include "name" %}`
- HTML escaping and security
- Compiled template caching

## Constants

### Node Types

| Constant | Value | Description |
|----------|-------|-------------|
| `NODE_TEXT` | 0 | Raw text node |
| `NODE_VAR` | 1 | Variable interpolation `{{ var }}` |
| `NODE_IF` | 2 | Conditional block `{% if cond %}` |
| `NODE_FOR` | 3 | Loop block `{% for item in list %}` |
| `NODE_INCLUDE` | 4 | Include partial `{% include "name" %}` |
| `NODE_FILTER` | 5 | Variable with filter `{{ var \| filter }}` |

### Filter Types

| Constant | Value | Description |
|----------|-------|-------------|
| `FILTER_NONE` | 0 | No filter applied |
| `FILTER_UPPER` | 1 | Convert to uppercase |
| `FILTER_LOWER` | 2 | Convert to lowercase |
| `FILTER_ESCAPE` | 3 | HTML escape |
| `FILTER_TRIM` | 4 | Trim whitespace |
| `FILTER_LENGTH` | 5 | Get string length |

### Value Types

| Constant | Value | Description |
|----------|-------|-------------|
| `VAL_STR` | 0 | String value |
| `VAL_INT` | 1 | Integer value |
| `VAL_LIST` | 2 | Array of string pointers |

### Buffer Sizes

| Constant | Value | Description |
|----------|-------|-------------|
| `TPL_MAX_NODES` | 256 | Maximum template nodes |
| `TPL_MAX_VARS` | 64 | Maximum context variables |
| `TPL_BUFFER_SIZE` | 65536 | Render buffer size (64KB) |
| `TPL_KEY_SIZE` | 128 | Maximum key length |
| `TPL_VAL_SIZE` | 4096 | Maximum value length |

## Structs

### TemplateCtx

Template context stores key-value pairs for variable interpolation.

```vais
S TemplateCtx {
    handle: i64    # Opaque pointer to C runtime context
}
```

#### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> TemplateCtx` | Create a new template context |
| `set_str` | `F set_str(&self, key: str, value: str) -> TemplateCtx` | Set a string variable (chainable) |
| `set_int` | `F set_int(&self, key: str, value: i64) -> TemplateCtx` | Set an integer variable (chainable) |
| `get` | `F get(&self, key: str) -> str` | Get a variable value (returns "" if not found) |
| `is_truthy` | `F is_truthy(&self, key: str) -> i64` | Check if variable exists and is truthy |
| `free` | `F free(&self) -> i64` | Free the context |
| `drop` | `F drop(&self) -> i64` | Alias for free (RAII pattern) |

**Truthy values**: Non-empty strings except "0" and "false"

### Template

Compiled template ready for rendering.

```vais
S Template {
    source: str,
    handle: i64    # Opaque pointer to parsed template
}
```

#### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `parse` | `F parse(source: str) -> Template` | Parse a template string |
| `render` | `F render(&self, ctx: &TemplateCtx) -> str` | Render template with context |
| `free` | `F free(&self) -> i64` | Free the template |
| `drop` | `F drop(&self) -> i64` | Alias for free (RAII pattern) |

## Functions

### Core Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `template_ctx_new` | `F template_ctx_new() -> TemplateCtx` | Create a new template context |
| `template_parse` | `F template_parse(source: str) -> Template` | Parse a template string |
| `template_render` | `F template_render(tmpl: &Template, ctx: &TemplateCtx) -> str` | Render a template with context |

### Convenience Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `template_quick_render` | `F template_quick_render(source: str, ctx: &TemplateCtx) -> str` | Parse and render in one step |
| `template_render_var` | `F template_render_var(source: str, key: str, value: str) -> str` | Render with single variable |

### HTML & Filters

| Function | Signature | Description |
|----------|-----------|-------------|
| `html_escape` | `F html_escape(input: str) -> str` | Escape HTML entities (&, <, >, ", ') |
| `apply_filter` | `F apply_filter(value: str, filter_name: str) -> str` | Apply named filter to value |

**Supported filters**: `"upper"`, `"lower"`, `"escape"`, `"trim"`, `"length"`

### Partials

| Function | Signature | Description |
|----------|-----------|-------------|
| `template_register_partial` | `F template_register_partial(name: str, source: str) -> i64` | Register a partial template by name |

## Template Syntax

### Variable Interpolation

```vais
{{ variable }}           # Replace with variable value
{{ user.name }}          # Nested access (if supported)
{{ variable | upper }}   # Apply filter to variable
```

### Conditionals

```vais
{% if condition %}
    Content when true
{% endif %}

{% if user %}
    Hello, {{ user }}!
{% endif %}
```

### Loops

```vais
{% for item in list %}
    Item: {{ item }}
{% endfor %}
```

### Partials

```vais
{% include "header" %}
{% include "footer" %}
```

## Usage Examples

### Basic Example

```vais
U std/template

F main() -> i64 {
    ctx := TemplateCtx::new()
    ctx.set_str("name", "World")

    tmpl := Template::parse("Hello, {{ name }}!")
    result := tmpl.render(&ctx)
    # result is "Hello, World!"

    tmpl.free()
    ctx.free()
    0
}
```

### Chaining Context Variables

```vais
U std/template

F main() -> i64 {
    ctx := TemplateCtx::new()
        .set_str("title", "My Page")
        .set_str("user", "Alice")
        .set_int("count", 42)

    tmpl := Template::parse("{{ title }}: {{ user }} ({{ count }})")
    result := tmpl.render(&ctx)
    # result is "My Page: Alice (42)"

    tmpl.free()
    ctx.free()
    0
}
```

### Quick Render (One-Shot)

```vais
U std/template

F main() -> i64 {
    ctx := TemplateCtx::new().set_str("name", "Bob")
    result := template_quick_render("Hi, {{ name }}!", &ctx)
    # result is "Hi, Bob!"
    ctx.free()
    0
}
```

### Single Variable Render

```vais
U std/template

F main() -> i64 {
    result := template_render_var(
        "Welcome, {{ user }}!",
        "user",
        "Charlie"
    )
    # result is "Welcome, Charlie!"
    0
}
```

### HTML Escaping

```vais
U std/template

F main() -> i64 {
    unsafe := "<script>alert('XSS')</script>"
    safe := html_escape(unsafe)
    # safe is "&lt;script&gt;alert(&#39;XSS&#39;)&lt;/script&gt;"
    0
}
```

### Applying Filters

```vais
U std/template

F main() -> i64 {
    upper := apply_filter("hello", "upper")   # "HELLO"
    lower := apply_filter("WORLD", "lower")   # "world"
    trimmed := apply_filter("  hi  ", "trim") # "hi"
    len := apply_filter("test", "length")     # "4"
    0
}
```

### Conditionals and Truthiness

```vais
U std/template

F main() -> i64 {
    ctx := TemplateCtx::new()
        .set_str("user", "Alice")
        .set_str("admin", "")

    # is_truthy returns 1 for non-empty, 0 for empty/"0"/"false"
    has_user := ctx.is_truthy("user")    # 1
    is_admin := ctx.is_truthy("admin")   # 0

    ctx.free()
    0
}
```

## Performance Notes

- Templates are compiled once and can be rendered multiple times
- Rendering uses a 64KB preallocated buffer for performance
- Context variable lookup is O(n) with max 64 variables
- For repeated rendering, parse once and reuse the Template

## Memory Management

- Template contexts and parsed templates must be explicitly freed
- Rendered strings are allocated and owned by caller
- Use `drop()` methods for RAII-style cleanup
- `template_quick_render` automatically frees the template
- `template_render_var` automatically frees both template and context
