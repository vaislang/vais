# 5-Minute Vibe Coding Guide

The fastest path for developers who want to generate Vais code with AI (Claude, Cursor, Copilot, etc.).
The goal is to reach "language + full ecosystem picture + first run" within 5 minutes.

---

## 1. One-Line Install

```bash
curl -fsSL https://vaislang.dev/install.sh | sh
vaisc --version    # v0.1.0
```

Windows PowerShell: `iwr https://vaislang.dev/install.ps1 | iex`
Homebrew: `brew install vaislang/vais/vais`

---

## 2. Teaching Vais to Your AI

If your AI doesn't know Vais syntax and the ecosystem, it will generate incorrect code.
Vais provides two context files following the **`llms.txt` standard**.

| File | Purpose | Size |
|------|---------|------|
| [llms.txt](https://vaislang.dev/llms.txt) | Curated index (links + key rules) | Small |
| [llms-full.txt](https://vaislang.dev/llms-full.txt) | Full docs concat — drag-and-drop ready | Large |

**Usage**: Attach `llms-full.txt` to Cursor's project context or a Claude conversation and
the AI will immediately understand the full scope of Vais syntax, standard library, VaisX, VaisDB, and vais-server.

---

## 3. Language Essentials in 5 Minutes

### Single-Character Keywords (designed to reduce AI token usage)

| Key | Meaning | Key | Meaning |
|-----|---------|-----|---------|
| `F` | function | `W` | trait |
| `S` | struct | `X` | impl |
| `E` | enum / else | `P` | pub |
| `I` | if | `D` | defer |
| `L` | loop | `A` | async |
| `M` | match | `Y` | await |
| `R` | return | `N` | extern |
| `T` | type alias | `G` | global |
| `U` | use (import) | `O` | union |

### Operators

- `:=` binding, `:= mut` mutable binding
- `?` try / ternary, `!` unwrap
- `|>` pipe, `..` range, `@` self-recursion
- `#` comment, `{expr}` string interpolation

### Type Coercion — **Strict Rules** (where AI makes the most mistakes)

```vais
# ❌ Wrong — implicit coercion is forbidden
x: i64 := 1.5      # error: implicit float → int not allowed

# ✅ Correct — explicit conversion with `as`
x: i64 := 1.5 as i64
flag: bool := 1 as bool
f: f64 := (42 as f64)
```

**Allowed implicit conversions**: integer widening (`i8→i64`), float literal inference (`f32↔f64`).
**Forbidden**: `bool↔i64`, `int↔float`, `str↔i64`, integer narrowing (`i64→i32`).

---

## 4. Hello World + Compile

```vais
# hello.vais
F main() -> i64 {
    puts("Hello, Vais!")
    0
}
```

```bash
vaisc build hello.vais    # → ./hello
./hello                    # Hello, Vais!
vaisc run hello.vais      # build + run
```

---

## 5. Three Real-World Examples

### Example 1: CLI Tool — FizzBuzz

```vais
F fizzbuzz(n: i64) -> i64 {
    LF i:1..n+1 {
        I i % 15 == 0      { puts("FizzBuzz") }
        EL I i % 3 == 0    { puts("Fizz") }
        EL I i % 5 == 0    { puts("Buzz") }
        EL                 { puts("{i}") }
    }
    0
}

F main() -> i64 {
    puts("=== FizzBuzz 1..20 ===")
    fizzbuzz(20)
}
```

Key: `LF` loop, `I/EL` if-else-if chain, string interpolation `{i}`.

### Example 2: HTTP Server (vais-server)

```vais
U core/app::{App, ServerConfig}
U core/context::{Context, Response}

F handle_hello(ctx: Context) -> Response {
    ctx.text(200, "Hello from vais-server!")
}

F handle_user(ctx: Context) -> Response {
    id := ctx.param("id")
    ctx.json(200, "{\"id\": \"{id}\", \"name\": \"Alice\"}")
}

F main() -> i64 {
    app := mut App.new(ServerConfig.default())
    app.get("/",          "handle_hello")
    app.get("/users/:id", "handle_user")
    app.listen(":8080")
    0
}
```

More: [vais-server Guide](../ecosystem/vais-server/README.md)

### Example 3: VaisDB — Hybrid Query

```vais
U vaisdb::{Database}

F main() -> i64 {
    db := Database.open("myapp.vaisdb")?

    # SQL + vector search in a single transaction
    db.execute("CREATE TABLE documents (id i64, title str, embedding vector(768))")?
    db.execute("INSERT INTO documents VALUES (1, 'Vais intro', @v1)")?

    results := db.query("
        SELECT id, title
        FROM documents
        VECTOR_SEARCH(embedding, @query, top_k=5)
        WHERE title LIKE '%Vais%'
    ")?

    LF row in results {
        puts("{row.id}: {row.title}")
    }
    0
}
```

More: [VaisDB Query Guide](../ecosystem/vaisdb/queries.md)

---

## 6. Full-Stack One-Pager

```
┌──────────── VaisX (Frontend) ──────────────┐
│ .vaisx components, < 3KB runtime, SSR/SSG  │
│ $state / $derived / $effect                │
└────────────────────┬───────────────────────┘
                     │ HTTP / WebSocket
┌────────────────────▼───────────────────────┐
│ vais-server (Backend)                      │
│ Express/Axum-style routing + middleware    │
│ Built-in JWT authentication                │
└────────────────────┬───────────────────────┘
                     │ Native Query API
┌────────────────────▼───────────────────────┐
│ VaisDB (Database)                          │
│ Vector + Graph + SQL + Full-text           │
│ Single `.vaisdb` file, single transaction  │
└────────────────────────────────────────────┘
```

---

## 7. Prompting AI Effectively

- **Provide context**: At the start of a conversation, share the `llms.txt` link or attach `llms-full.txt`.
- **Warn about type coercion**: Add one line: "Vais requires strict type coercion. No implicit coercion; use `as` explicitly."
- **Watch for removed keywords**: `weak`, `clone`, `consume` have been removed. Reject them if AI tries to use them.
- **Avoid experimental features**: `lazy`, `force`, HKT, `impl Trait` (non-dyn) are still limited. Exclude from production code.
- **Ecosystem imports**: VaisDB uses `U vaisdb::{Database}`, vais-server uses `U core/app::{App}`.

---

## 8. Next Steps

- **Language in depth**: [Language Specification](./language/language-spec.md)
- **Hands-on tutorials**: [CLI Tool](../tutorials/cli-tool.md), [HTTP Server](../tutorials/http-server.md), [WebSocket Chat](../tutorials/websocket-chat.md)
- **Ecosystem details**: [Ecosystem Overview](../ecosystem/README.md)
- **Playground**: [play.vaislang.dev](https://play.vaislang.dev) (40+ examples)
- **GitHub**: [vaislang/vais](https://github.com/vaislang/vais)
