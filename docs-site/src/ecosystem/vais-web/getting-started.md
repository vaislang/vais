# 빠른 시작 가이드

## 사전 요구 사항

- **Node.js** 20 이상
- **pnpm** 9 이상 (`npm install -g pnpm`)

---

## 설치

VaisX CLI로 새 프로젝트를 생성합니다.

```bash
pnpm create vaisx@latest my-app
```

대화형 프롬프트에서 옵션을 선택합니다.

```
? 프로젝트 이름: my-app
? 템플릿 선택:
  ● 기본 (hello world)
  ○ 카운터 앱
  ○ Todo 앱 (SSR + 서버 액션)
  ○ 블로그 (SSG + 동적 라우트)
```

또는 플래그로 바로 생성합니다.

```bash
pnpm create vaisx@latest my-app --template default
```

---

## 프로젝트 구조

생성된 프로젝트의 기본 구조입니다.

```
my-app/
├── app/
│   ├── page.vaisx       # 홈 페이지 (/)
│   └── layout.vaisx     # 공통 레이아웃
├── package.json
├── vaisx.config.ts      # VaisX 설정
└── README.md
```

### app/ 디렉토리

URL 구조와 직접 매핑됩니다. 파일을 추가하면 자동으로 라우트가 생성됩니다.

| 파일 | URL |
|---|---|
| `app/page.vaisx` | `/` |
| `app/about/page.vaisx` | `/about` |
| `app/posts/[slug]/page.vaisx` | `/posts/:slug` |
| `app/layout.vaisx` | 모든 페이지에 적용되는 공통 레이아웃 |
| `app/error.vaisx` | 전역 에러 바운더리 |

### vaisx.config.ts

프로젝트 설정 파일입니다.

```typescript
export default {
  srcDir: "app",
  outDir: "dist",
  // SSG 경로 목록 (빌드 시 정적 HTML 생성)
  prerender: ["/", "/about"],
};
```

---

## 개발 서버 시작

```bash
cd my-app
pnpm install
pnpm dev
```

브라우저에서 `http://localhost:3000`을 열면 앱이 실행됩니다.

개발 서버는 다음 기능을 제공합니다.

- **HMR (Hot Module Replacement)**: 파일 저장 시 페이지 전체 새로고침 없이 변경 내용 즉시 반영
- **에러 오버레이**: 컴파일 에러와 런타임 에러를 브라우저에 직접 표시
- **반응성 DevTools**: 브라우저 개발자 도구에서 반응성 그래프와 상태 변화 확인

---

## 첫 번째 컴포넌트

`app/page.vaisx`를 열어 첫 번째 컴포넌트를 작성합니다.

```vais
<script>
  greeting := "안녕하세요, VaisX!"
  count := $state(0)

  F increment() {
    count += 1
  }
</script>

<template>
  <main>
    <h1>{greeting}</h1>
    <p>클릭 수: {count}</p>
    <button @click={increment}>클릭</button>
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

저장하면 HMR이 즉시 브라우저에 반영합니다.

---

## 레이아웃 컴포넌트

`app/layout.vaisx`는 모든 페이지를 감싸는 공통 레이아웃입니다. `{slot}`이 각 페이지 내용이 삽입되는 위치입니다.

```vais
<script context="server">
  # 서버 컴포넌트 — 클라이언트에 JS를 전송하지 않음
</script>

<template>
  <html lang="ko">
    <head>
      <meta charset="utf-8" />
      <meta name="viewport" content="width=device-width, initial-scale=1" />
      <title>My VaisX App</title>
    </head>
    <body>
      <nav>
        <a href="/">홈</a>
        <a href="/about">소개</a>
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

## 빌드

```bash
pnpm build
```

`dist/` 디렉토리에 최적화된 결과물이 생성됩니다. `prerender` 설정에 지정된 경로는 정적 HTML로 미리 생성됩니다.

## 프로덕션 서버 실행

```bash
pnpm start
```

기본 포트는 3000입니다. `PORT` 환경 변수로 변경할 수 있습니다.

```bash
PORT=8080 pnpm start
```

---

## 다음 단계 예제

### 카운터 앱

`$state`, `$derived`, `$effect`의 기본 반응성을 확인하려면 카운터 예제를 참고하세요.

```bash
pnpm create vaisx@latest counter-demo --template counter
```

### Todo 앱 (SSR + 서버 액션)

서버 사이드 데이터 로딩과 서버 액션(`#[server] A F`)을 사용하는 Todo 앱입니다.

```bash
pnpm create vaisx@latest todo-demo --template todo
```

### 블로그 (SSG + 동적 라우트)

`entries()` 함수와 동적 라우트 `[slug]`를 사용한 정적 사이트 생성 예제입니다.

```bash
pnpm create vaisx@latest blog-demo --template blog
```

---

## 문제 해결

### Node.js 버전 오류

VaisX는 Node.js 20 이상이 필요합니다.

```bash
node --version  # v20.0.0 이상이어야 함
```

### pnpm 설치 오류

```bash
npm install -g pnpm@latest
```

### 컴파일 에러: "unknown keyword"

Vais 언어 키워드(`F`, `S`, `I`, `R`, `A` 등)는 대소문자를 구분합니다. 소문자로 쓰지 않도록 주의하세요.

---

## 관련 문서

- [문법 가이드](./syntax.md) — 템플릿 지시문과 반응성 기본 요소
- [컴포넌트 시스템](./components.md) — Props, Events, Slots, Context
