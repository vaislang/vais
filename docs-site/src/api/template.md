# Template API Reference

> Template engine with variable interpolation, conditionals, and loops

## Import

```vais
U std/template
```

## Template Syntax

| Syntax | Description |
|--------|-------------|
| `{{ variable }}` | Variable interpolation |
| `{{ var \| upper }}` | Variable with filter (placeholder) |
| `{% if condition %}...{% endif %}` | Conditional block |
| `{% for item in list %}...{% endfor %}` | Loop block |
| `{% include "partial" %}` | Include partial (placeholder) |

## Key Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `template_ctx_new` | `F template_ctx_new() -> i64` | Create template context |
| `template_ctx_set_str` | `F template_ctx_set_str(ctx: i64, key: i64, val: i64) -> i64` | Set string variable |
| `template_ctx_set_int` | `F template_ctx_set_int(ctx: i64, key: i64, val: i64) -> i64` | Set integer variable |
| `template_parse` | `F template_parse(tmpl: i64) -> i64` | Parse template string |
| `template_render` | `F template_render(tmpl: i64, ctx: i64) -> i64` | Render template |
| `template_free` | `F template_free(tmpl: i64) -> i64` | Free template |
| `template_ctx_free` | `F template_ctx_free(ctx: i64) -> i64` | Free context |

## Usage

```vais
U std/template

F main() -> i64 {
    ctx := template_ctx_new()
    template_ctx_set_str(ctx, "name", "World")
    tmpl := template_parse("Hello, {{ name }}!")
    result := template_render(tmpl, ctx)
    # result points to "Hello, World!"
    template_free(tmpl)
    template_ctx_free(ctx)
    0
}
```
