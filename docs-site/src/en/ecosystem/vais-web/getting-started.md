# Getting Started

## Prerequisites

- **Node.js** 20 or later
- **pnpm** 9 or later (`npm install -g pnpm`)

---

## Installation

Create a new project using the VaisX CLI.

```bash
pnpm create vaisx@latest my-app
```

Select options from the interactive prompt.

```
? Project name: my-app
? Select a template:
  ● Default (hello world)
  ○ Counter app
  ○ Todo app (SSR + server actions)
  ○ Blog (SSG + dynamic routes)
```

Or create a project directly with flags.

```bash
pnpm create vaisx@latest my-app --template default
```

---

## Project Structure

The default structure of a generated project.

```
my-app/
├── app/
│   ├── page.vaisx       # Home page (/)
│   └── layout.vaisx     # Shared layout
├── package.json
├── vaisx.config.ts      # VaisX configuration
└── README.md
```

### app/ Directory

Maps directly to the URL structure. Routes are created automatically when you add files.

| File | URL |
|---|---|
| `app/page.vaisx` | `/` |
| `app/about/page.vaisx` | `/about` |
| `app/posts/[slug]/page.vaisx` | `/posts/:slug` |
| `app/layout.vaisx` | Shared layout applied to all pages |
| `app/error.vaisx` | Global error boundary |

### vaisx.config.ts

The project configuration file.

```typescript
export default {
  srcDir: "app",
  outDir: "dist",
  // List of SSG paths (pre-generates static HTML at build time)
  prerender: ["/", "/about"],
};
```

---

## Starting the Dev Server

```bash
cd my-app
pnpm install
pnpm dev
```

Open `http://localhost:3000` in your browser to see the app running.

The dev server provides the following features.

- **HMR (Hot Module Replacement)**: Changes are reflected instantly without a full page reload when a file is saved.
- **Error overlay**: Compile errors and runtime errors are displayed directly in the browser.
- **Reactivity DevTools**: Inspect the reactivity graph and state changes in the browser developer tools.

---

## Your First Component

Open `app/page.vaisx` to write your first component.

```vais
<script>
  greeting := "Hello, VaisX!"
  count := $state(0)

  F increment() {
    count += 1
  }
</script>

<template>
  <main>
    <h1>{greeting}</h1>
    <p>Clicks: {count}</p>
    <button @click={increment}>Click</button>
  </main>
</template>

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 40px;
    font-family: system-ui, sans-serif;
  }
  button {
    padding: 8px 16px;
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
  }
</style>
```

HMR will instantly reflect changes in the browser when you save.

---

## Layout Component

`app/layout.vaisx` is the shared layout that wraps all pages. `{slot}` is where each page's content will be inserted.

```vais
<script context="server">
  # Server component — no JS is sent to the client
</script>

<template>
  <html lang="en">
    <head>
      <meta charset="utf-8" />
      <meta name="viewport" content="width=device-width, initial-scale=1" />
      <title>My VaisX App</title>
    </head>
    <body>
      <nav>
        <a href="/">Home</a>
        <a href="/about">About</a>
      </nav>
      <div id="content">
        {slot}
      </div>
    </body>
  </html>
</template>

<style>
  nav { padding: 16px; background: #1f2937; }
  nav a { color: white; text-decoration: none; margin-right: 16px; }
  #content { padding: 24px; }
</style>
```

---

## Build

```bash
pnpm build
```

Optimized output is generated in the `dist/` directory. Paths specified in the `prerender` configuration are pre-generated as static HTML.

## Running the Production Server

```bash
pnpm start
```

The default port is 3000. You can change it with the `PORT` environment variable.

```bash
PORT=8080 pnpm start
```

---

## Example Next Steps

### Counter App

See the counter example to explore the basic reactivity of `$state`, `$derived`, and `$effect`.

```bash
pnpm create vaisx@latest counter-demo --template counter
```

### Todo App (SSR + Server Actions)

A Todo app using server-side data loading and server actions (`#[server] A F`).

```bash
pnpm create vaisx@latest todo-demo --template todo
```

### Blog (SSG + Dynamic Routes)

A static site generation example using the `entries()` function and dynamic routes `[slug]`.

```bash
pnpm create vaisx@latest blog-demo --template blog
```

---

## Troubleshooting

### Node.js Version Error

VaisX requires Node.js 20 or later.

```bash
node --version  # must be v20.0.0 or higher
```

### pnpm Installation Error

```bash
npm install -g pnpm@latest
```

### Compile Error: "unknown keyword"

Vais language keywords (`F`, `S`, `I`, `R`, `A`, etc.) are case-sensitive. Be careful not to write them in lowercase.

---

## Related Docs

- [Syntax Guide](./syntax.md) — Template directives and reactivity primitives
- [Component System](./components.md) — Props, Events, Slots, Context
