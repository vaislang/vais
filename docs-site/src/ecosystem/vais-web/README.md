# VaisX (vais-web)

VaisX는 Vais 언어 위에서 동작하는 컴파일 타임 반응성 프론트엔드 프레임워크입니다. 빌드 시점에 반응성 그래프를 완전히 해석하여 런타임 오버헤드를 최소화합니다.

## 특징

### 초경량 런타임 (< 3KB)

반응성 로직 전체가 컴파일 타임에 처리됩니다. `$state`, `$derived`, `$effect`는 빌드 시 최적화된 DOM 업데이트 코드로 변환되며, 런타임에 남는 코어는 3KB 미만입니다. React(~45KB), Vue(~34KB), Svelte 런타임과 비교해 현저히 작습니다.

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
| `E { }` | else |
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
│  최적화된 JS/WASM    │  ← Vite 플러그인이 번들링
│  (반응성 코드 내장)  │
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  VaisX 런타임       │  ← < 3KB
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
├── runtime/         # 코어 런타임 (< 3KB)
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
└── WASM codegen     → wasm32 컴파일 타겟
        ↓
vaislang/vais-lang/packages/vais-web  (이 패키지)
        ↓
vaislang/vais-lang/packages/vais-server  (SSR 연동)
```

코어 컴파일러와의 호환성은 220개의 계약 테스트로 보장됩니다 (Phase 139+ 대상).

---

## 다른 프레임워크와의 비교

| 항목 | VaisX | React | Vue 3 | Svelte |
|---|---|---|---|---|
| 런타임 크기 | < 3KB | ~45KB | ~34KB | ~2KB |
| 반응성 방식 | 컴파일 타임 | 가상 DOM | 프록시 | 컴파일 타임 |
| 언어 | Vais | JSX/TSX | SFC + TSX | Svelte |
| SSR/SSG | 내장 | Next.js 필요 | Nuxt 필요 | SvelteKit 필요 |
| 파일 기반 라우팅 | 내장 | 별도 설정 | 별도 설정 | 내장 |

---

## 다음 단계

- [빠른 시작](./getting-started.md) — 설치부터 첫 앱 실행까지
- [문법 가이드](./syntax.md) — VaisX 템플릿 지시문과 반응성 기본 요소
- [컴포넌트 시스템](./components.md) — Props, Events, Slots, Context
