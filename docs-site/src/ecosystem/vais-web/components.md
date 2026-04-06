# 컴포넌트 시스템

VaisX의 컴포넌트 시스템은 Props, Events, Slots, Context 네 가지 메커니즘으로 컴포넌트 간 데이터와 동작을 공유합니다.

---

## 컴포넌트 기본 구조

`.vaisx` 파일 하나가 컴포넌트 하나입니다. 다른 컴포넌트를 `<template>` 내에서 태그로 사용합니다.

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
<!-- app/page.vaisx — Button 컴포넌트 사용 -->
<script>
  import Button from "../components/Button.vaisx"
</script>

<template>
  <Button label="클릭하세요" />
</template>
```

---

## Props

### Props 선언 — P { }

`P { }` 블록으로 컴포넌트가 받을 Props를 선언합니다. 각 필드는 이름과 타입으로 구성됩니다.

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
    <span>{count}개</span>
  </div>
</template>
```

### 기본값 지정

Props에 기본값을 지정할 수 있습니다.

```vais
<script>
  P {
    title: String = "제목 없음",
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

### 컴포넌트에 Props 전달

```vais
<!-- 정적 값 -->
<Card title="안녕하세요" count={42} isActive={true} />

<!-- 동적 바인딩 -->
<Card :title={post.title} :count={post.views} :isActive={isSelected} />

<!-- 스프레드 전달 -->
<Card {...postProps} />
```

### Props 유효성 검사

Props 타입은 컴파일 타임에 검사됩니다. 잘못된 타입을 전달하면 빌드 에러가 발생합니다.

```vais
<!-- 컴파일 에러: String이 필요한데 Int 전달 -->
<Card title={42} />

<!-- 컴파일 에러: 필수 Props 누락 -->
<Card />
```

---

## Events

### emit으로 이벤트 발생

자식 컴포넌트에서 부모로 데이터를 전달할 때 `emit`을 사용합니다.

```vais
<!-- Counter.vaisx -->
<script>
  P { initialValue: Int = 0 }

  count := $state(initialValue)

  F increment() {
    count += 1
    emit change(count)   # 부모에게 새 값 전달
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

### 부모에서 이벤트 수신

`@이벤트명={핸들러}`로 자식 컴포넌트의 이벤트를 수신합니다.

```vais
<!-- app/page.vaisx -->
<script>
  import Counter from "../components/Counter.vaisx"

  total := $state(0)

  F handleChange(newValue: Int) {
    total = newValue
    console.log("카운터 변경:", newValue)
  }
</script>

<template>
  <Counter
    :initialValue={5}
    @change={handleChange}
  />
  <p>현재 값: {total}</p>
</template>
```

### 여러 이벤트 선언

컴포넌트는 여러 이벤트를 발생시킬 수 있습니다.

```vais
<!-- SearchInput.vaisx -->
<script>
  P { placeholder: String = "검색..." }

  query := $state("")

  F handleInput(e) {
    query = e.target.value
    emit input(query)     # 입력마다 발생
  }

  F handleSubmit() {
    emit submit(query)    # 제출 시 발생
    emit clear()          # 인자 없는 이벤트
  }
</script>

<template>
  <div class="search">
    <input
      :value={query}
      :placeholder={placeholder}
      @input={handleInput}
    />
    <button @click={handleSubmit}>검색</button>
  </div>
</template>
```

```vais
<!-- 부모에서 수신 -->
<SearchInput
  @input={handleInput}
  @submit={handleSearch}
  @clear={clearResults}
/>
```

---

## Slots

Slots은 부모 컴포넌트가 자식 컴포넌트의 내부 콘텐츠를 제어할 수 있게 합니다.

### 기본 Slot

`{slot}`이 자식 내용이 삽입될 위치입니다.

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
<!-- 사용 -->
<Card>
  <h2>카드 제목</h2>
  <p>카드 내용입니다.</p>
</Card>
```

### 이름 있는 Slot

여러 위치에 콘텐츠를 삽입할 때 이름 있는 Slot을 사용합니다.

```vais
<!-- Modal.vaisx -->
<template>
  <div class="modal-overlay">
    <div class="modal">
      <header class="modal-header">
        {slot name="header"}
      </header>
      <main class="modal-body">
        {slot}          <!-- 기본 슬롯 -->
      </main>
      <footer class="modal-footer">
        {slot name="footer"}
      </footer>
    </div>
  </div>
</template>
```

```vais
<!-- 사용 -->
<Modal>
  <template slot="header">
    <h2>확인</h2>
  </template>

  <!-- 기본 슬롯 내용 -->
  <p>정말로 삭제하시겠습니까?</p>

  <template slot="footer">
    <button @click={cancel}>취소</button>
    <button @click={confirm}>확인</button>
  </template>
</Modal>
```

### Slot 기본값

Slot에 내용이 전달되지 않았을 때 표시할 기본 콘텐츠를 지정합니다.

```vais
<!-- Button.vaisx -->
<template>
  <button class="btn">
    {slot}
    <!-- 슬롯 기본값: 내용이 없을 때 사용됨 -->
    {slot default="클릭"}
  </button>
</template>
```

### Scoped Slot

자식 컴포넌트의 데이터를 Slot 콘텐츠에서 사용할 수 있습니다.

```vais
<!-- List.vaisx -->
<script>
  P { items: Array<Item> }
</script>

<template>
  <ul>
    @each items as item, index {
      <li>
        <!-- item과 index를 슬롯에 노출 -->
        {slot item={item} index={index}}
      </li>
    }
  </ul>
</template>
```

```vais
<!-- 사용: 슬롯 데이터를 let으로 받음 -->
<List :items={products}>
  <template let:item let:index>
    <span>{index + 1}. {item.name} — {item.price}원</span>
  </template>
</List>
```

---

## Context

Context는 Props를 단계별로 내려보내지 않고 컴포넌트 트리에서 데이터를 공유하는 방법입니다.

### Context 제공 — setContext

부모 컴포넌트에서 `setContext`로 데이터를 제공합니다.

```vais
<!-- ThemeProvider.vaisx -->
<script>
  import { setContext } from "vaisx"

  P {
    theme: String = "light"
  }

  # 하위 컴포넌트 모두에서 접근 가능
  setContext("theme", {
    current: theme,
    toggle: F() {
      # theme 토글 로직
    }
  })
</script>

<template>
  <div class={"app theme-" + theme}>
    {slot}
  </div>
</template>
```

### Context 소비 — getContext

하위 컴포넌트에서 `getContext`로 데이터를 받습니다.

```vais
<!-- Button.vaisx — 어느 깊이에 있어도 theme에 접근 가능 -->
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

### 반응형 Context

Context에 `$state`를 결합하면 값이 변경될 때 하위 컴포넌트도 자동으로 업데이트됩니다.

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

  # $state 값을 context로 제공
  setContext("auth", { user, isLoading, login, logout })
</script>

<template>
  @if isLoading {
    <p>로딩 중...</p>
  } @else {
    {slot}
  }
</template>
```

```vais
<!-- UserProfile.vaisx — 어느 곳에서든 auth 접근 -->
<script>
  import { getContext } from "vaisx"

  auth := getContext("auth")
</script>

<template>
  @if auth.user {
    <div class="profile">
      <img :src={auth.user.avatar} />
      <p>{auth.user.name}</p>
      <button @click={auth.logout}>로그아웃</button>
    </div>
  } @else {
    <a href="/login">로그인</a>
  }
</template>
```

---

## 서버 컴포넌트

`<script context="server">`로 선언된 컴포넌트는 서버에서만 렌더링됩니다. 클라이언트에 JavaScript를 전송하지 않아 초기 로드가 빠릅니다.

```vais
<!-- BlogPost.vaisx — 서버 컴포넌트 -->
<script context="server">
  P { slug: String }

  # 서버에서만 실행 — DB 직접 접근 가능
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

서버 컴포넌트 내에서 클라이언트 컴포넌트를 사용하면 해당 부분만 클라이언트 JavaScript가 포함됩니다.

```vais
<!-- BlogPage.vaisx — 서버 컴포넌트 -->
<script context="server">
  import BlogPost from "../components/BlogPost.vaisx"
  import LikeButton from "../components/LikeButton.vaisx"  # 클라이언트 컴포넌트

  P { slug: String }
  post := getPostBySlug(slug)
</script>

<template>
  <BlogPost :slug={slug} />
  <!-- LikeButton만 클라이언트 JS 포함 -->
  <LikeButton :postId={post.id} :initialLikes={post.likes} />
</template>
```

---

## 컴포넌트 수명주기

VaisX는 컴파일 타임 반응성을 기반으로 하여 전통적인 수명주기 훅 대신 `$effect`로 수명주기를 처리합니다.

```vais
<script>
  # 컴포넌트 마운트 시 실행 ($effect는 첫 렌더링 후 실행)
  $effect {
    console.log("컴포넌트 마운트됨")

    # 정리 함수: 컴포넌트 해제 시 실행
    R () => {
      console.log("컴포넌트 해제됨")
    }
  }

  # 특정 상태 변화에 반응
  query := $state("")
  $effect {
    I query.length > 2 {
      performSearch(query)
    }
  }
</script>
```

---

## 컴포넌트 패턴

### 컨테이너/프레젠테이션 분리

```vais
<!-- PostListContainer.vaisx — 데이터 관리 -->
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
<!-- PostList.vaisx — 표시만 담당 -->
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

### 재사용 가능한 로직 — 함수 모듈

반응성 로직을 일반 Vais 모듈로 추출하여 여러 컴포넌트에서 재사용합니다.

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
    <button @click={reset}>리셋</button>
    <button @click={increment}>+</button>
  </div>
</template>
```

---

## 관련 문서

- [빠른 시작](./getting-started.md) — 프로젝트 생성과 개발 서버
- [문법 가이드](./syntax.md) — 템플릿 지시문과 반응성 기본 요소
