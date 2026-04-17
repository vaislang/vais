# VaisX (vais-web)

VaisX is a compile-time reactivity frontend framework built on top of the Vais language. It fully resolves the reactivity graph at build time, minimizing runtime overhead.

## Features

### Ultralight Runtime (< 3KB)

All reactivity logic is processed at compile time. `$state`, `$derived`, and `$effect` are transformed into optimized DOM update code at build time, leaving a core runtime of less than 3KB. This is significantly smaller than React (~45KB), Vue (~34KB), and the Svelte runtime.

### Single-File Component (.vaisx)

Each component is written as a single file with the `.vaisx` extension. One file contains all of the logic (`<script>`), markup (`<template>`), and styles (`<style>`).

```vais
<script>
  name := "VaisX"
  count := $state(0)

  F increment() {
    count += 1
  }
</script>

<template>
  <h1>Hello, {name}!</h1>
  <p>Count: {count}</p>
  <button @click={increment}>+1</button>
</template>

<style>
  h1 { color: #3b82f6; }
</style>
```

### File-Based Routing

The `app/` directory structure maps directly to URL paths. Routes are created simply by adding files — no separate router configuration needed.

```
app/
├── page.vaisx          → /
├── about/
│   └── page.vaisx      → /about
└── posts/
    ├── page.vaisx      → /posts
    └── [slug]/
        └── page.vaisx  → /posts/:slug
```

### SSR / SSG Support

You can choose a rendering strategy per page.

- **SSG (Static Site Generation)**: Pre-generates HTML at build time. Configured via the `prerender` array or `entries()` function in `vaisx.config.ts`.
- **SSR (Server-Side Rendering)**: Generates HTML on the server for each request. Data loading is handled via the `load()` function.
- **Client-Only**: Declared with `<script context="client">` — runs only in the browser with no server rendering.

### Vais Language Integration

VaisX uses the Vais language's type system and syntax directly. Component logic is written in Vais instead of TypeScript.

| Vais Keyword | Meaning |
|---|---|
| `F name() { }` | Function definition |
| `A F name() { }` | Async function definition |
| `S Name { field: Type }` | Struct definition |
| `:=` | Variable binding (immutable) |
| `mut name := value` | Mutable variable |
| `I cond { }` | if conditional |
| `E I cond { }` | else if |
| `E { }` | else |
| `R value` | Return |

---

## Architecture

```
Source file (.vaisx)
        │
        ▼
┌─────────────────────┐
│  VaisX Compiler     │  ← Rust (vaisx-compiler crate)
│  - vaisx-parser     │
│  - vais-codegen-js  │
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  Optimized JS/WASM  │  ← Bundled by Vite plugin
│  (reactivity built-in) │
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  VaisX Runtime      │  ← < 3KB
│  (DOM patching, events) │
└─────────────────────┘
```

### Compiler Pipeline

1. **Parsing**: `vaisx-parser` analyzes `.vaisx` files and separates the `<script>`, `<template>`, and `<style>` blocks.
2. **Vais AST generation**: Vais code in the `<script>` block is parsed with `vais-parser` to produce an AST.
3. **Reactivity analysis**: `$state`, `$derived`, and `$effect` declarations are tracked to build a dependency graph.
4. **JS code generation**: `vais-codegen-js` transforms the AST into optimized JavaScript ESM code.
5. **Template compilation**: Template directives (`@if`, `@each`, etc.) are compiled into fine-grained DOM update functions.

### Package Layout

```
packages/
├── runtime/         # Core runtime (< 3KB)
├── cli/             # Project scaffolding CLI
├── kit/             # Shared types and interfaces
├── plugin/          # Vite-compatible plugin
├── devtools/        # Reactivity graph & profiler
├── hmr/             # Hot Module Replacement
├── components/      # Built-in UI components
├── store/           # State management
├── query/           # Data fetching
├── forms/           # Form handling
├── auth/            # Authentication
├── i18n/            # Internationalization
└── testing/         # Testing utilities
```

### Upstream Dependencies

VaisX depends on the Vais core compiler (`vaislang/vais`).

```
vaislang/vais (compiler)
├── vais-codegen-js  → JS ESM code generation
├── vais-parser      → Vais source parsing
├── vais-ast         → AST type definitions
└── WASM codegen     → wasm32 compile target
        ↓
vaislang/vais-lang/packages/vais-web  (this package)
        ↓
vaislang/vais-lang/packages/vais-server  (SSR integration)
```

Compatibility with the core compiler is guaranteed by 220 contract tests (targeting Phase 139+).

---

## Comparison with Other Frameworks

| Item | VaisX | React | Vue 3 | Svelte |
|---|---|---|---|---|
| Runtime size | < 3KB | ~45KB | ~34KB | ~2KB |
| Reactivity model | Compile-time | Virtual DOM | Proxy | Compile-time |
| Language | Vais | JSX/TSX | SFC + TSX | Svelte |
| SSR/SSG | Built-in | Requires Next.js | Requires Nuxt | Requires SvelteKit |
| File-based routing | Built-in | Separate config | Separate config | Built-in |

---

## Next Steps

- [Getting Started](./getting-started.md) — From installation to running your first app
- [Syntax Guide](./syntax.md) — VaisX template directives and reactivity primitives
- [Component System](./components.md) — Props, Events, Slots, Context
