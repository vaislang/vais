# VaisX Syntax Guide

VaisX components are written as `.vaisx` single files. A file consists of three blocks: `<script>`, `<template>`, and `<style>`.

```vais
<script>
  <!-- Component logic written in the Vais language -->
</script>

<template>
  <!-- HTML + VaisX directives -->
</template>

<style>
  /* CSS scoped to this component only */
</style>
```

---

## Script Block

### Variable Declarations

Variables are declared using the `:=` operator from the Vais language. They are immutable by default.

```vais
<script>
  # Immutable binding
  title := "VaisX App"
  version := 1

  # Mutable variable
  mut counter := 0
</script>
```

### Function Definitions

Functions are defined with the `F` keyword. Async functions use `A F`.

```vais
<script>
  F greet(name: String) -> String {
    R "Hello, " + name + "!"
  }

  A F fetchData(url: String) -> Data {
    res := await fetch(url)
    R await res.json()
  }
</script>
```

### Struct Definitions

Structs (data types) are defined with the `S` keyword.

```vais
<script>
  S User {
    id: Int,
    name: String,
    email: String
  }

  S Post {
    id: Int,
    title: String,
    author: User
  }
</script>
```

### Server Components

Declaring `<script context="server">` makes the component run only on the server. No JavaScript is sent to the client.

```vais
<script context="server">
  # Code in this block runs on the server only
  # DB access, environment variable usage, etc. are allowed
</script>
```

---

## Reactivity Primitives

### $state — Reactive State

Declare reactive state with `$state(initialValue)`. When the value changes, the template updates automatically.

```vais
<script>
  count := $state(0)
  name := $state("Jane Doe")
  items := $state([])
  isOpen := $state(false)

  F increment() {
    count += 1        # count changes → template updates automatically
  }

  F setName(newName: String) {
    name = newName
  }
</script>

<template>
  <p>{count}</p>
  <p>{name}</p>
</template>
```

`$state` supports not only primitive types (numbers, strings, booleans) but also arrays and objects.

```vais
<script>
  todos := $state([
    { id: 1, text: "Learn VaisX", done: false },
    { id: 2, text: "Build an app", done: false }
  ])

  F addTodo(text: String) {
    todos = [...todos, { id: todos.length + 1, text, done: false }]
  }
</script>
```

### $derived — Derived Values

`$derived(expression)` is a read-only value that is automatically recomputed whenever the `$state` values it depends on change.

```vais
<script>
  count := $state(0)

  # Automatically recomputed each time count changes
  doubled := $derived(count * 2)
  isEven := $derived(count % 2 == 0)
  label := $derived(isEven ? "even" : "odd")
</script>

<template>
  <p>{count} × 2 = {doubled}</p>
  <p>This number is {label}.</p>
</template>
```

`$derived` can also depend on multiple `$state` values.

```vais
<script>
  firstName := $state("John")
  lastName := $state("Doe")

  fullName := $derived(firstName + " " + lastName)
</script>
```

### $effect — Side Effects

A `$effect { }` block runs whenever any reactive value read inside it changes. Use it for syncing with external systems, logging, or direct DOM manipulation.

```vais
<script>
  count := $state(0)
  title := $state("VaisX")

  # Runs every time count changes
  $effect {
    console.log("Count changed:", count)
  }

  # Updates document.title every time title changes
  $effect {
    document.title = title + " - My App"
  }
</script>
```

A cleanup function can be returned.

```vais
<script>
  query := $state("")

  $effect {
    timer := setTimeout(() => {
      performSearch(query)
    }, 300)

    # Called before the next run or when the component is destroyed
    R () => clearTimeout(timer)
  }
</script>
```

---

## Template Directives

### Text Interpolation

Use curly braces `{}` to insert an expression's result as text.

```html
<template>
  <p>{greeting}</p>
  <p>Welcome, {user.name}.</p>
  <p>Total: {price * quantity}</p>
</template>
```

### Attribute Binding (:attr)

Dynamically bind HTML attributes with `:attributeName={value}`.

```html
<template>
  <input :value={inputValue} />
  <img :src={imageUrl} :alt={imageAlt} />
  <a :href={"/posts/" + post.slug}>{post.title}</a>
  <div :class={isActive ? "active" : ""}></div>
</template>
```

### @if / @else — Conditional Rendering

Show or hide content based on a condition.

```html
<template>
  @if isLoggedIn {
    <p>Welcome, {user.name}!</p>
    <button @click={logout}>Log out</button>
  } @else {
    <p>Please log in.</p>
    <a href="/login">Log in</a>
  }
</template>
```

Use `@else if` for multiple conditions.

```html
<template>
  @if score >= 90 {
    <span class="grade-a">A</span>
  } @else if score >= 80 {
    <span class="grade-b">B</span>
  } @else if score >= 70 {
    <span class="grade-c">C</span>
  } @else {
    <span class="grade-f">F</span>
  }
</template>
```

### @each — List Rendering

Iterate over an array to render repeated content.

```html
<template>
  <ul>
    @each items as item {
      <li>{item.name}: {item.price}</li>
    }
  </ul>
</template>
```

You can also access the index.

```html
<template>
  <ol>
    @each items as item, index {
      <li>{index + 1}. {item.title}</li>
    }
  </ol>
</template>
```

Specifying a `key` improves list update performance.

```html
<template>
  @each todos as todo (todo.id) {
    <li class={todo.done ? "done" : ""}>
      {todo.text}
    </li>
  }
</template>
```

### @click — Event Binding

Bind DOM events with `@eventName={handler}`.

```html
<template>
  <button @click={handleClick}>Click</button>
  <input @input={handleInput} />
  <form @submit={handleSubmit}>
    <!-- Form content -->
  </form>
</template>
```

Inline handlers are also supported.

```html
<template>
  <button @click={() => { count += 1 }}>+1</button>
  <button @click={() => { isOpen = !isOpen }}>Toggle</button>
</template>
```

Common event modifiers.

```html
<template>
  <!-- Prevent default behavior -->
  <form @submit.prevent={handleSubmit}>...</form>

  <!-- Stop event bubbling -->
  <div @click.stop={handleClick}>...</div>

  <!-- Run only once -->
  <button @click.once={handleClick}>Once</button>
</template>
```

### @await — Async Rendering

An async block that handles a Promise.

```html
<template>
  @await fetchUser(userId) {
    # Loading
    <p>Loading...</p>
  } then user {
    # On success
    <p>{user.name}'s profile</p>
    <img :src={user.avatar} />
  } catch error {
    # On error
    <p class="error">Error: {error.message}</p>
  }
</template>
```

---

## Style Block

CSS in the `<style>` block is scoped to the current component by default (scoped CSS).

```vais
<style>
  /* Applies only to elements in this component */
  h1 {
    color: #1f2937;
    font-size: 2rem;
  }

  .button {
    padding: 8px 16px;
    border-radius: 6px;
  }
</style>
```

Use the `:global()` selector when global styles are needed.

```vais
<style>
  /* Global CSS reset */
  :global(*, *::before, *::after) {
    box-sizing: border-box;
    margin: 0;
  }

  /* Scoped only within this component */
  .container {
    max-width: 1200px;
    margin: 0 auto;
  }
</style>
```

---

## Server Actions

Functions annotated with `#[server]` run only on the server. They are connected to a form's `action` attribute for use.

```vais
<script>
  #[server]
  A F createPost(formData: FormData) -> ActionResult {
    title := formData.get("title")
    content := formData.get("content")

    I title == "" {
      R ActionResult { status: "error", errors: { title: "Title is required" } }
    }

    savePost({ title, content })
    R ActionResult { status: "success" }
  }
</script>

<template>
  <form action={createPost}>
    <input type="text" name="title" placeholder="Title" />
    <textarea name="content" placeholder="Content"></textarea>
    <button type="submit">Publish</button>
  </form>
</template>
```

Server actions support progressive enhancement — forms work correctly even when JavaScript is disabled.

---

## Vais Language Syntax Reference

| Pattern | Meaning | Example |
|---|---|---|
| `name := value` | Immutable binding | `x := 42` |
| `mut name := value` | Mutable variable | `mut count := 0` |
| `name := $state(v)` | Reactive state | `count := $state(0)` |
| `name := $derived(e)` | Derived value | `double := $derived(n * 2)` |
| `$effect { }` | Side effect block | `$effect { log(x) }` |
| `F name() { }` | Function definition | `F add(a, b) { R a + b }` |
| `A F name() { }` | Async function | `A F load() { }` |
| `S Name { }` | Struct definition | `S User { name: String }` |
| `I cond { }` | if conditional | `I x > 0 { R true }` |
| `E I cond { }` | else if | `E I x == 0 { }` |
| `E { }` | else | `E { R false }` |
| `R value` | Return | `R result` |
| `#[server]` | Server-only attribute | `#[server] A F load()` |
| `P { name: Type }` | Props declaration | `P { title: String }` |
| `emit name(args)` | Emit event | `emit change(value)` |

---

## Related Docs

- [Getting Started](./getting-started.md) — Project creation and dev server
- [Component System](./components.md) — Props, Events, Slots, Context
