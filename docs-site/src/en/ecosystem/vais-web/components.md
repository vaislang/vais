# Component System

VaisX's component system shares data and behavior between components through four mechanisms: Props, Events, Slots, and Context.

---

## Basic Component Structure

One `.vaisx` file equals one component. Other components are used as tags inside `<template>`.

```vais
<!-- Button.vaisx -->
<script>
  P { label: String }
</script>

<template>
  <button class="btn">{label}</button>
</template>

<style>
  .btn {
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
  }
</style>
```

```vais
<!-- app/page.vaisx — using the Button component -->
<script>
  import Button from "../components/Button.vaisx"
</script>

<template>
  <Button label="Click me" />
</template>
```

---

## Props

### Props Declaration — P { }

Declare the props a component accepts using a `P { }` block. Each field consists of a name and a type.

```vais
<script>
  P {
    title: String,
    count: Int,
    isActive: Bool,
    items: Array<String>
  }
</script>

<template>
  <div class={isActive ? "active" : ""}>
    <h2>{title}</h2>
    <span>{count} items</span>
  </div>
</template>
```

### Default Values

Default values can be specified for props.

```vais
<script>
  P {
    title: String = "Untitled",
    size: String = "medium",
    disabled: Bool = false
  }
</script>

<template>
  <button
    class={"btn btn-" + size}
    :disabled={disabled}
  >
    {title}
  </button>
</template>
```

### Passing Props to a Component

```vais
<!-- Static values -->
<Card title="Hello" count={42} isActive={true} />

<!-- Dynamic binding -->
<Card :title={post.title} :count={post.views} :isActive={isSelected} />

<!-- Spread -->
<Card {...postProps} />
```

### Props Validation

Prop types are checked at compile time. Passing the wrong type causes a build error.

```vais
<!-- Compile error: String required but Int passed -->
<Card title={42} />

<!-- Compile error: required prop missing -->
<Card />
```

---

## Events

### Emitting Events with emit

Use `emit` to pass data from a child component to the parent.

```vais
<!-- Counter.vaisx -->
<script>
  P { initialValue: Int = 0 }

  count := $state(initialValue)

  F increment() {
    count += 1
    emit change(count)   # Pass new value to parent
  }

  F decrement() {
    count -= 1
    emit change(count)
  }
</script>

<template>
  <div class="counter">
    <button @click={decrement}>-</button>
    <span>{count}</span>
    <button @click={increment}>+</button>
  </div>
</template>
```

### Receiving Events in the Parent

Listen to child component events with `@eventName={handler}`.

```vais
<!-- app/page.vaisx -->
<script>
  import Counter from "../components/Counter.vaisx"

  total := $state(0)

  F handleChange(newValue: Int) {
    total = newValue
    console.log("Counter changed:", newValue)
  }
</script>

<template>
  <Counter
    :initialValue={5}
    @change={handleChange}
  />
  <p>Current value: {total}</p>
</template>
```

### Declaring Multiple Events

A component can emit multiple events.

```vais
<!-- SearchInput.vaisx -->
<script>
  P { placeholder: String = "Search..." }

  query := $state("")

  F handleInput(e) {
    query = e.target.value
    emit input(query)     # Emitted on each keystroke
  }

  F handleSubmit() {
    emit submit(query)    # Emitted on submit
    emit clear()          # Event with no arguments
  }
</script>

<template>
  <div class="search">
    <input
      :value={query}
      :placeholder={placeholder}
      @input={handleInput}
    />
    <button @click={handleSubmit}>Search</button>
  </div>
</template>
```

```vais
<!-- Receiving in the parent -->
<SearchInput
  @input={handleInput}
  @submit={handleSearch}
  @clear={clearResults}
/>
```

---

## Slots

Slots allow a parent component to control the inner content of a child component.

### Default Slot

`{slot}` is the position where child content will be inserted.

```vais
<!-- Card.vaisx -->
<template>
  <div class="card">
    <div class="card-body">
      {slot}
    </div>
  </div>
</template>

<style>
  .card {
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    overflow: hidden;
  }
  .card-body { padding: 16px; }
</style>
```

```vais
<!-- Usage -->
<Card>
  <h2>Card Title</h2>
  <p>Card content goes here.</p>
</Card>
```

### Named Slots

Use named slots when inserting content into multiple positions.

```vais
<!-- Modal.vaisx -->
<template>
  <div class="modal-overlay">
    <div class="modal">
      <header class="modal-header">
        {slot name="header"}
      </header>
      <main class="modal-body">
        {slot}          <!-- Default slot -->
      </main>
      <footer class="modal-footer">
        {slot name="footer"}
      </footer>
    </div>
  </div>
</template>
```

```vais
<!-- Usage -->
<Modal>
  <template slot="header">
    <h2>Confirm</h2>
  </template>

  <!-- Default slot content -->
  <p>Are you sure you want to delete this?</p>

  <template slot="footer">
    <button @click={cancel}>Cancel</button>
    <button @click={confirm}>Confirm</button>
  </template>
</Modal>
```

### Slot Default Content

Specify fallback content to display when nothing is passed to a slot.

```vais
<!-- Button.vaisx -->
<template>
  <button class="btn">
    {slot}
    <!-- Slot default: used when no content is provided -->
    {slot default="Click"}
  </button>
</template>
```

### Scoped Slots

Data from the child component can be used within the slot content.

```vais
<!-- List.vaisx -->
<script>
  P { items: Array<Item> }
</script>

<template>
  <ul>
    @each items as item, index {
      <li>
        <!-- Expose item and index to the slot -->
        {slot item={item} index={index}}
      </li>
    }
  </ul>
</template>
```

```vais
<!-- Usage: receive slot data with let -->
<List :items={products}>
  <template let:item let:index>
    <span>{index + 1}. {item.name} — {item.price}</span>
  </template>
</List>
```

---

## Context

Context is a way to share data across the component tree without passing props step by step.

### Providing Context — setContext

Provide data from a parent component using `setContext`.

```vais
<!-- ThemeProvider.vaisx -->
<script>
  import { setContext } from "vaisx"

  P {
    theme: String = "light"
  }

  # Accessible from all descendant components
  setContext("theme", {
    current: theme,
    toggle: F() {
      # Theme toggle logic
    }
  })
</script>

<template>
  <div class={"app theme-" + theme}>
    {slot}
  </div>
</template>
```

### Consuming Context — getContext

Receive data in a descendant component using `getContext`.

```vais
<!-- Button.vaisx — can access theme at any depth -->
<script>
  import { getContext } from "vaisx"

  themeCtx := getContext("theme")
</script>

<template>
  <button class={"btn btn-" + themeCtx.current}>
    {slot}
  </button>
</template>
```

### Reactive Context

Combining `$state` with context means descendant components automatically update when the value changes.

```vais
<!-- AuthProvider.vaisx -->
<script>
  import { setContext } from "vaisx"

  user := $state(null)
  isLoading := $state(true)

  A F login(credentials) {
    isLoading = true
    result := await authenticate(credentials)
    user = result.user
    isLoading = false
    emit login(user)
  }

  F logout() {
    user = null
    emit logout()
  }

  # Provide $state values as context
  setContext("auth", { user, isLoading, login, logout })
</script>

<template>
  @if isLoading {
    <p>Loading...</p>
  } @else {
    {slot}
  }
</template>
```

```vais
<!-- UserProfile.vaisx — access auth from anywhere -->
<script>
  import { getContext } from "vaisx"

  auth := getContext("auth")
</script>

<template>
  @if auth.user {
    <div class="profile">
      <img :src={auth.user.avatar} />
      <p>{auth.user.name}</p>
      <button @click={auth.logout}>Log out</button>
    </div>
  } @else {
    <a href="/login">Log in</a>
  }
</template>
```

---

## Server Components

Components declared with `<script context="server">` are rendered only on the server. No JavaScript is sent to the client, resulting in faster initial loads.

```vais
<!-- BlogPost.vaisx — server component -->
<script context="server">
  P { slug: String }

  # Runs on server only — direct DB access is allowed
  post := getPostBySlug(slug)
</script>

<template>
  <article>
    <h1>{post.title}</h1>
    <time>{post.date}</time>
    <div class="content">{post.content}</div>
  </article>
</template>
```

When a client component is used inside a server component, only that portion includes client-side JavaScript.

```vais
<!-- BlogPage.vaisx — server component -->
<script context="server">
  import BlogPost from "../components/BlogPost.vaisx"
  import LikeButton from "../components/LikeButton.vaisx"  # Client component

  P { slug: String }
  post := getPostBySlug(slug)
</script>

<template>
  <BlogPost :slug={slug} />
  <!-- Only LikeButton includes client JS -->
  <LikeButton :postId={post.id} :initialLikes={post.likes} />
</template>
```

---

## Component Lifecycle

VaisX is based on compile-time reactivity, so lifecycle is handled with `$effect` instead of traditional lifecycle hooks.

```vais
<script>
  # Runs when the component mounts ($effect runs after the first render)
  $effect {
    console.log("Component mounted")

    # Cleanup function: runs when the component is destroyed
    R () => {
      console.log("Component destroyed")
    }
  }

  # React to specific state changes
  query := $state("")
  $effect {
    I query.length > 2 {
      performSearch(query)
    }
  }
</script>
```

---

## Component Patterns

### Container / Presentation Separation

```vais
<!-- PostListContainer.vaisx — data management -->
<script context="server">
  import PostList from "./PostList.vaisx"

  #[server]
  A F load() -> PageData {
    posts := await fetchPosts()
    R PageData { posts }
  }
</script>

<template>
  <PostList :posts={posts} />
</template>
```

```vais
<!-- PostList.vaisx — presentation only -->
<script>
  P { posts: Array<Post> }
</script>

<template>
  <ul class="post-list">
    @each posts as post {
      <li>
        <a :href={"/posts/" + post.slug}>{post.title}</a>
      </li>
    }
  </ul>
</template>
```

### Reusable Logic — Function Modules

Extract reactivity logic into a regular Vais module to reuse across multiple components.

```vais
<!-- lib/useCounter.vais -->
F useCounter(initial: Int = 0) {
  count := $state(initial)
  doubled := $derived(count * 2)

  F increment() { count += 1 }
  F decrement() { count -= 1 }
  F reset() { count = initial }

  R { count, doubled, increment, decrement, reset }
}
```

```vais
<!-- Counter.vaisx -->
<script>
  import { useCounter } from "../lib/useCounter.vais"

  { count, doubled, increment, decrement, reset } := useCounter(0)
</script>

<template>
  <div>
    <p>{count} (×2 = {doubled})</p>
    <button @click={decrement}>-</button>
    <button @click={reset}>Reset</button>
    <button @click={increment}>+</button>
  </div>
</template>
```

---

## Related Docs

- [Getting Started](./getting-started.md) — Project creation and dev server
- [Syntax Guide](./syntax.md) — Template directives and reactivity primitives
