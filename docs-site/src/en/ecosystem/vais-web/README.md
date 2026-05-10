# VaisX (vais-web)

VaisX is a compile-time reactivity frontend framework workbench built on top of
the Vais language. Current public claims are tied to the promoted Vais Web
gates: runtime `61/77`, unit `390/390`, package `3272/3272`, full-build
`24/24`.

## Features

### Runtime Gates

Reactivity analysis and DOM update generation are implementation surfaces. Size
claims such as a sub-3KB runtime require a dedicated size gate before they are
public guarantees.

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
│  JS/WASM outputs    │  ← experimental unless gated
│  (reactivity built-in) │
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  VaisX Runtime      │  ← runtime 61/77 gate
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
├── runtime/         # Core runtime, size gate pending
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
└── WASM codegen     → experimental wasm32 target unless gated
        ↓
vaislang/vais-lang/packages/vais-web  (this package)
        ↓
vaislang/vais-lang/packages/vais-server  (SSR integration)
```

Compatibility with the core compiler should be judged by the current full-build
and package gates, not by historical phase numbers.

---

## Comparison with Other Frameworks

| Item | VaisX | React | Vue 3 | Svelte |
|---|---|---|---|---|
| Runtime gate | 61/77 smoke | n/a | n/a | n/a |
| Reactivity model | Compile-time | Virtual DOM | Proxy | Compile-time |
| Language | Vais | JSX/TSX | SFC + TSX | Svelte |
| SSR/SSG | Built-in | Requires Next.js | Requires Nuxt | Requires SvelteKit |
| File-based routing | Built-in | Separate config | Separate config | Built-in |

---

## Next Steps

- [Getting Started](./getting-started.md) — From installation to running your first app
- [Syntax Guide](./syntax.md) — VaisX template directives and reactivity primitives
- [Component System](./components.md) — Props, Events, Slots, Context
