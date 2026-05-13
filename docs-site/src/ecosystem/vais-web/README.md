# VaisX (vais-web)

VaisX는 Vais 언어 위에서 동작하는 컴파일 타임 반응성 frontend framework
workbench입니다. 현재 공개 claim은 runtime `61/77`, unit `390/390`, package
`3272/3272`, full-build `24/24`, shared-schema product `9/9` gate에 묶입니다.

## 특징

### Runtime Gates

반응성 분석과 DOM 업데이트 생성은 구현 표면입니다. 3KB 미만 runtime 같은
size claim은 dedicated size gate가 생기기 전까지 공개 보증으로 쓰지
않습니다.

### 단일 파일 컴포넌트 (.vaisx)

각 컴포넌트는 `.vaisx` 확장자의 단일 파일로 작성합니다. 파일 하나에 로직(`<script>`), 마크업(`<template>`), 스타일(`<style>`)이 모두 포함됩니다.

```vais
<script>
  name := "VaisX"
  count := $state(0)

  F increment() {
    count += 1
  }
</script>

<template>
  <h1>안녕하세요, {name}!</h1>
  <p>카운트: {count}</p>
  <button @click={increment}>+1</button>
</template>

<style>
  h1 { color: #3b82f6; }
</style>
```

### 파일 기반 라우팅

`app/` 디렉토리 구조가 URL 경로와 직접 매핑됩니다. 별도의 라우터 설정 없이 파일만 만들면 라우트가 생성됩니다.

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

### SSR / SSG 지원

페이지별로 렌더링 전략을 선택할 수 있습니다.

- **SSG (정적 사이트 생성)**: 빌드 시 HTML 미리 생성. `vaisx.config.ts`의 `prerender` 배열 또는 `entries()` 함수로 설정.
- **SSR (서버 사이드 렌더링)**: 요청마다 서버에서 HTML 생성. `load()` 함수로 데이터 로딩.
- **클라이언트 전용**: `<script context="client">`로 선언 시 서버 렌더링 없이 브라우저에서만 실행.

### Vais 언어 통합

VaisX는 Vais 언어의 타입 시스템과 문법을 그대로 사용합니다. TypeScript 대신 Vais 언어로 컴포넌트 로직을 작성합니다.

| Vais 키워드 | 의미 |
|---|---|
| `F name() { }` | 함수 정의 |
| `A F name() { }` | 비동기 함수 정의 |
| `S Name { field: Type }` | 구조체 정의 |
| `:=` | 변수 바인딩 (불변) |
| `mut name := value` | 가변 변수 |
| `I cond { }` | if 조건문 |
| `E I cond { }` | else if |
| `else { }` | else |
| `R value` | 반환 |

---

## 아키텍처

```
소스 파일 (.vaisx)
        │
        ▼
┌─────────────────────┐
│  VaisX 컴파일러      │  ← Rust (vaisx-compiler crate)
│  - vaisx-parser     │
│  - vais-codegen-js  │
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  JS/WASM outputs    │  ← experimental unless gated
│  (반응성 코드 내장)  │
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  VaisX 런타임       │  ← runtime 61/77 gate
│  (DOM 패치, 이벤트) │
└─────────────────────┘
```

### 컴파일러 파이프라인

1. **파싱**: `vaisx-parser`가 `.vaisx` 파일을 분석하여 `<script>`, `<template>`, `<style>` 블록을 분리합니다.
2. **Vais AST 생성**: `<script>` 블록의 Vais 코드를 `vais-parser`로 파싱하여 AST를 생성합니다.
3. **반응성 분석**: `$state`, `$derived`, `$effect` 선언을 추적하여 의존성 그래프를 빌드합니다.
4. **JS 코드 생성**: `vais-codegen-js`가 AST를 최적화된 JavaScript ESM 코드로 변환합니다.
5. **템플릿 컴파일**: 템플릿 지시문(`@if`, `@each`, 등)을 세밀한 DOM 업데이트 함수로 컴파일합니다.

### 패키지 구성

```
packages/
├── runtime/         # 코어 runtime, size gate pending
├── cli/             # 프로젝트 스캐폴딩 CLI
├── kit/             # 공유 타입 및 인터페이스
├── plugin/          # Vite 호환 플러그인
├── devtools/        # 반응성 그래프 & 프로파일러
├── hmr/             # Hot Module Replacement
├── components/      # 내장 UI 컴포넌트
├── store/           # 상태 관리
├── query/           # 데이터 페칭
├── forms/           # 폼 처리
├── auth/            # 인증
├── i18n/            # 국제화
└── testing/         # 테스트 유틸리티
```

### 업스트림 의존성

VaisX는 Vais 코어 컴파일러(`vaislang/vais`)에 의존합니다.

```
vaislang/vais (컴파일러)
├── vais-codegen-js  → JS ESM 코드 생성
├── vais-parser      → Vais 소스 파싱
├── vais-ast         → AST 타입 정의
└── WASM codegen     → experimental wasm32 target unless gated
        ↓
vaislang/vais-lang/packages/vais-web  (이 패키지)
        ↓
vaislang/vais-lang/packages/vais-server  (SSR 연동)
```

코어 컴파일러 호환성은 historical phase number가 아니라 현재 full-build 및
package gate로 판단합니다.

---

## 다른 프레임워크와의 비교

| 항목 | VaisX | React | Vue 3 | Svelte |
|---|---|---|---|---|
| Runtime gate | 61/77 smoke + shared schema 9/9 | n/a | n/a | n/a |
| 반응성 방식 | 컴파일 타임 | 가상 DOM | 프록시 | 컴파일 타임 |
| 언어 | Vais | JSX/TSX | SFC + TSX | Svelte |
| SSR/SSG | 내장 | Next.js 필요 | Nuxt 필요 | SvelteKit 필요 |
| 파일 기반 라우팅 | 내장 | 별도 설정 | 별도 설정 | 내장 |

---

## 다음 단계

- [빠른 시작](./getting-started.md) — 설치부터 첫 앱 실행까지
- [문법 가이드](./syntax.md) — VaisX 템플릿 지시문과 반응성 기본 요소
- [컴포넌트 시스템](./components.md) — Props, Events, Slots, Context
