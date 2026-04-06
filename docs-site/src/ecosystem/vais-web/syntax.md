# VaisX 문법 가이드

VaisX 컴포넌트는 `.vaisx` 단일 파일로 작성됩니다. 파일은 `<script>`, `<template>`, `<style>` 세 블록으로 구성됩니다.

```vais
<script>
  <!-- Vais 언어로 작성하는 컴포넌트 로직 -->
</script>

<template>
  <!-- HTML + VaisX 지시문 -->
</template>

<style>
  /* 이 컴포넌트에만 적용되는 CSS */
</style>
```

---

## 스크립트 블록

### 변수 선언

Vais 언어의 `:=` 연산자로 변수를 선언합니다. 기본적으로 불변입니다.

```vais
<script>
  # 불변 바인딩
  title := "VaisX 앱"
  version := 1

  # 가변 변수
  mut counter := 0
</script>
```

### 함수 정의

`F` 키워드로 함수를 정의합니다. 비동기 함수는 `A F`를 사용합니다.

```vais
<script>
  F greet(name: String) -> String {
    R "안녕하세요, " + name + "!"
  }

  A F fetchData(url: String) -> Data {
    res := await fetch(url)
    R await res.json()
  }
</script>
```

### 구조체 정의

`S` 키워드로 구조체(데이터 타입)를 정의합니다.

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

### 서버 컴포넌트

`<script context="server">`로 선언하면 서버에서만 실행됩니다. 클라이언트에 JavaScript를 전송하지 않습니다.

```vais
<script context="server">
  # 이 블록의 코드는 서버에서만 실행됨
  # DB 접근, 환경 변수 사용 등 가능
</script>
```

---

## 반응성 기본 요소

### $state — 반응형 상태

`$state(초기값)`으로 반응형 상태를 선언합니다. 값이 변경되면 템플릿이 자동으로 업데이트됩니다.

```vais
<script>
  count := $state(0)
  name := $state("홍길동")
  items := $state([])
  isOpen := $state(false)

  F increment() {
    count += 1        # count 변경 → 템플릿 자동 업데이트
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

`$state`는 원시 타입(숫자, 문자열, 불리언)뿐만 아니라 배열과 객체도 지원합니다.

```vais
<script>
  todos := $state([
    { id: 1, text: "VaisX 배우기", done: false },
    { id: 2, text: "앱 만들기", done: false }
  ])

  F addTodo(text: String) {
    todos = [...todos, { id: todos.length + 1, text, done: false }]
  }
</script>
```

### $derived — 파생 값

`$derived(표현식)`은 의존하는 `$state`가 변경될 때 자동으로 재계산되는 읽기 전용 값입니다.

```vais
<script>
  count := $state(0)

  # count가 변경될 때마다 자동 재계산
  doubled := $derived(count * 2)
  isEven := $derived(count % 2 == 0)
  label := $derived(isEven ? "짝수" : "홀수")
</script>

<template>
  <p>{count} × 2 = {doubled}</p>
  <p>이 수는 {label}입니다.</p>
</template>
```

여러 `$state`에 의존하는 `$derived`도 가능합니다.

```vais
<script>
  firstName := $state("길동")
  lastName := $state("홍")

  fullName := $derived(lastName + firstName)
</script>
```

### $effect — 부작용

`$effect { }` 블록은 내부에서 읽는 반응형 값이 변경될 때마다 실행됩니다. 외부 시스템 동기화, 로깅, DOM 직접 조작 등에 사용합니다.

```vais
<script>
  count := $state(0)
  title := $state("VaisX")

  # count가 변경될 때마다 실행
  $effect {
    console.log("카운트 변경:", count)
  }

  # title이 변경될 때마다 document.title 업데이트
  $effect {
    document.title = title + " - My App"
  }
</script>
```

정리(cleanup) 함수를 반환할 수 있습니다.

```vais
<script>
  query := $state("")

  $effect {
    timer := setTimeout(() => {
      performSearch(query)
    }, 300)

    # 다음 실행 전 또는 컴포넌트 해제 시 호출
    R () => clearTimeout(timer)
  }
</script>
```

---

## 템플릿 지시문

### 텍스트 보간

중괄호 `{}`로 표현식 결과를 텍스트로 삽입합니다.

```html
<template>
  <p>{greeting}</p>
  <p>{user.name}님, 반갑습니다.</p>
  <p>합계: {price * quantity}원</p>
</template>
```

### 속성 바인딩 (:attr)

`:속성명={값}`으로 HTML 속성을 동적으로 바인딩합니다.

```html
<template>
  <input :value={inputValue} />
  <img :src={imageUrl} :alt={imageAlt} />
  <a :href={"/posts/" + post.slug}>{post.title}</a>
  <div :class={isActive ? "active" : ""}></div>
</template>
```

### @if / @else — 조건부 렌더링

조건에 따라 콘텐츠를 표시하거나 숨깁니다.

```html
<template>
  @if isLoggedIn {
    <p>환영합니다, {user.name}님!</p>
    <button @click={logout}>로그아웃</button>
  } @else {
    <p>로그인이 필요합니다.</p>
    <a href="/login">로그인</a>
  }
</template>
```

`@else if`로 다중 조건을 처리합니다.

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

### @each — 리스트 렌더링

배열을 순회하여 반복적인 콘텐츠를 렌더링합니다.

```html
<template>
  <ul>
    @each items as item {
      <li>{item.name}: {item.price}원</li>
    }
  </ul>
</template>
```

인덱스도 함께 사용할 수 있습니다.

```html
<template>
  <ol>
    @each items as item, index {
      <li>{index + 1}. {item.title}</li>
    }
  </ol>
</template>
```

키(`key`)를 지정하면 목록 업데이트 성능이 향상됩니다.

```html
<template>
  @each todos as todo (todo.id) {
    <li class={todo.done ? "done" : ""}>
      {todo.text}
    </li>
  }
</template>
```

### @click — 이벤트 바인딩

`@이벤트명={핸들러}`로 DOM 이벤트를 바인딩합니다.

```html
<template>
  <button @click={handleClick}>클릭</button>
  <input @input={handleInput} />
  <form @submit={handleSubmit}>
    <!-- 폼 내용 -->
  </form>
</template>
```

인라인 핸들러도 가능합니다.

```html
<template>
  <button @click={() => { count += 1 }}>+1</button>
  <button @click={() => { isOpen = !isOpen }}>토글</button>
</template>
```

자주 사용하는 이벤트 수식어입니다.

```html
<template>
  <!-- 기본 동작 방지 -->
  <form @submit.prevent={handleSubmit}>...</form>

  <!-- 이벤트 버블링 방지 -->
  <div @click.stop={handleClick}>...</div>

  <!-- 한 번만 실행 -->
  <button @click.once={handleClick}>한 번만</button>
</template>
```

### @await — 비동기 렌더링

Promise를 처리하는 비동기 블록입니다.

```html
<template>
  @await fetchUser(userId) {
    # 로딩 중
    <p>불러오는 중...</p>
  } then user {
    # 성공 시
    <p>{user.name}님의 프로필</p>
    <img :src={user.avatar} />
  } catch error {
    # 에러 시
    <p class="error">오류: {error.message}</p>
  }
</template>
```

---

## 스타일 블록

`<style>` 블록의 CSS는 기본적으로 해당 컴포넌트에만 범위가 한정됩니다 (scoped CSS).

```vais
<style>
  /* 이 컴포넌트의 요소에만 적용됨 */
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

글로벌 스타일이 필요한 경우 `:global()` 선택자를 사용합니다.

```vais
<style>
  /* 전역 CSS 초기화 */
  :global(*, *::before, *::after) {
    box-sizing: border-box;
    margin: 0;
  }

  /* 이 컴포넌트 내에서만 scoped */
  .container {
    max-width: 1200px;
    margin: 0 auto;
  }
</style>
```

---

## 서버 액션

`#[server]` 속성을 붙인 함수는 서버에서만 실행됩니다. 폼의 `action` 속성에 연결하여 사용합니다.

```vais
<script>
  #[server]
  A F createPost(formData: FormData) -> ActionResult {
    title := formData.get("title")
    content := formData.get("content")

    I title == "" {
      R ActionResult { status: "error", errors: { title: "제목을 입력하세요" } }
    }

    savePost({ title, content })
    R ActionResult { status: "success" }
  }
</script>

<template>
  <form action={createPost}>
    <input type="text" name="title" placeholder="제목" />
    <textarea name="content" placeholder="내용"></textarea>
    <button type="submit">게시</button>
  </form>
</template>
```

서버 액션은 JavaScript가 비활성화된 환경에서도 폼이 정상 동작하는 점진적 기능 향상을 지원합니다.

---

## Vais 언어 문법 참조

| 패턴 | 의미 | 예시 |
|---|---|---|
| `name := value` | 불변 바인딩 | `x := 42` |
| `mut name := value` | 가변 변수 | `mut count := 0` |
| `name := $state(v)` | 반응형 상태 | `count := $state(0)` |
| `name := $derived(e)` | 파생 값 | `double := $derived(n * 2)` |
| `$effect { }` | 부작용 블록 | `$effect { log(x) }` |
| `F name() { }` | 함수 정의 | `F add(a, b) { R a + b }` |
| `A F name() { }` | 비동기 함수 | `A F load() { }` |
| `S Name { }` | 구조체 정의 | `S User { name: String }` |
| `I cond { }` | if 조건 | `I x > 0 { R true }` |
| `E I cond { }` | else if | `E I x == 0 { }` |
| `E { }` | else | `E { R false }` |
| `R value` | 반환 | `R result` |
| `#[server]` | 서버 전용 속성 | `#[server] A F load()` |
| `P { name: Type }` | Props 선언 | `P { title: String }` |
| `emit name(args)` | 이벤트 발생 | `emit change(value)` |

---

## 관련 문서

- [빠른 시작](./getting-started.md) — 프로젝트 생성과 개발 서버
- [컴포넌트 시스템](./components.md) — Props, Events, Slots, Context
