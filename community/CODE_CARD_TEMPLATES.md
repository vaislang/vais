# Vais Code Card Templates

SNS용 코드 스니펫 비주얼 카드 템플릿 가이드

---

## 1. 카드 디자인 스펙

### 크기 및 비율
- **Instagram**: 1080x1080 (1:1)
- **Twitter/X**: 1200x675 (16:9)
- **LinkedIn**: 1200x627
- **General**: 1200x800 (3:2)

### 컬러 팔레트
```css
/* Primary Colors */
--vais-dark-bg: #1A202C       /* Main background */
--vais-darker-bg: #171923     /* Darker accents */
--vais-code-bg: #2D3748       /* Code block background */

/* Brand Colors */
--vais-green: #48BB78         /* Primary brand */
--vais-green-light: #68D391   /* Hover/accent */
--vais-blue: #667EEA          /* Secondary accent */
--vais-purple: #9F7AEA        /* Tertiary accent */

/* Syntax Colors */
--keyword: #F687B3            /* F, S, E, I, L, M, R, U */
--operator: #FBD38D           /* @, :=, => */
--string: #68D391             /* String literals */
--number: #90CDF4             /* Numbers */
--comment: #718096            /* Comments */
--type: #FC8181               /* Types */

/* UI Colors */
--text-primary: #F7FAFC
--text-secondary: #CBD5E0
--border: #4A5568
```

### 타이포그래피
```css
/* Fonts */
--font-code: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace
--font-display: 'Inter', 'SF Pro Display', sans-serif
--font-heading: 'Poppins', 'SF Pro Display', sans-serif

/* Sizes */
--size-title: 48px (Bold)
--size-subtitle: 24px (Medium)
--size-code: 18px (Regular)
--size-small: 14px (Regular)
--size-caption: 12px (Regular)
```

### 로고 및 워터마크
- **위치**: 좌상단 또는 좌하단
- **크기**: 40-60px
- **워터마크**: `@vaislang` + `vaislang.org` (우하단, 작게)
- **로고 색상**: Vais Green (#48BB78)

---

## 2. 카드 종류별 레이아웃 (6종)

### Type 1: 단일 코드 스니펫
**용도**: 간단한 코드 예제 공유

```
┌─────────────────────────────────┐
│ [Logo]                          │
│                                 │
│    "Fibonacci in Vais"          │
│    ═══════════════              │
│                                 │
│    ┌───────────────────┐        │
│    │ F fib(n:i64)      │        │
│    │   ->i64 =         │        │
│    │   n<2 ? n :       │        │
│    │   @(n-1)+@(n-2)   │        │
│    └───────────────────┘        │
│                                 │
│         @vaislang               │
└─────────────────────────────────┘
```

**요소**:
- 상단: 로고 (40px)
- 제목: 중앙 정렬, 32-48px
- 구분선: 장식용 얇은 선
- 코드 블록: 중앙 배치, 구문 하이라이팅
- 하단: 워터마크 @vaislang

---

### Type 2: 언어 비교 (Vais vs X)
**용도**: Vais의 간결함 강조

```
┌─────────────────────────────────┐
│ Vais vs Rust                    │
│                                 │
│ ┌──────────┐  ┌──────────┐     │
│ │ Vais     │  │ Rust     │     │
│ │ ─────    │  │ ─────    │     │
│ │ F fib... │  │ fn fib...│     │
│ │   (3)    │  │   (7)    │     │
│ └──────────┘  └──────────┘     │
│                                 │
│ Token count: 8 vs 24            │
│ ████████ vs ████████████████    │
│                                 │
│         @vaislang               │
└─────────────────────────────────┘
```

**요소**:
- 제목: "Vais vs X" (상단)
- 좌측: Vais 코드 (녹색 테두리)
- 우측: 비교 언어 코드 (회색 테두리)
- 하단: 토큰 수 비교 바 (시각적 바 차트)
- 통계: "8 tokens vs 24 tokens" (67% reduction)

**비교 대상 언어**:
- Rust, Go, TypeScript, Python, Java

---

### Type 3: 일일 팁
**용도**: 매일 Vais 팁 공유

```
┌─────────────────────────────────┐
│ 💡 Vais Tip #42                 │
│                                 │
│    Self-Recursion Operator      │
│                                 │
│    Instead of:                  │
│    F fact(n) = I n<2 {1}        │
│                E {n*fact(n-1)}  │
│                                 │
│    Use @:                       │
│    F fact(n) = n<2?1:n*@(n-1)   │
│                                 │
│    ✨ Shorter & clearer!        │
│                                 │
│         @vaislang               │
└─────────────────────────────────┘
```

**요소**:
- 아이콘: 💡 + "Vais Tip #N"
- 설명 텍스트 (간단히)
- Before/After 코드 비교
- 하이라이트: "Shorter & clearer!"

---

### Type 4: 기능 소개
**용도**: 새로운 기능 또는 핵심 기능 소개

```
┌─────────────────────────────────┐
│ Pattern Matching                │
│                                 │
│ M value {                       │
│   Some(x) => x * 2,             │
│   None => 0                     │
│ }                               │
│                                 │
│ ────────────────────            │
│                                 │
│ ✓ Exhaustive checking           │
│ ✓ Type-safe extraction          │
│ ✓ Expression-based              │
│                                 │
│         @vaislang               │
└─────────────────────────────────┘
```

**요소**:
- 기능명: 큰 텍스트 (상단)
- 코드 예제: 중앙 (구문 하이라이팅)
- 구분선
- 주요 특징: 3-4개 bullet points (체크마크)
- 워터마크

---

### Type 5: 릴리스 노트
**용도**: 새 버전 공지

```
┌─────────────────────────────────┐
│        Vais 0.0.2               │
│        ─────────                │
│                                 │
│ What's New:                     │
│                                 │
│ ✨ Generic monomorphization     │
│ 🚀 Trait dynamic dispatch       │
│ 📦 Standard library expansion   │
│ 🐛 Bug fixes & improvements     │
│ 📚 Documentation updates        │
│                                 │
│ ───────────────────────         │
│ Try it now!                     │
│ $ brew install vais             │
│                                 │
│         @vaislang               │
└─────────────────────────────────┘
```

**요소**:
- 버전 번호: 큰 텍스트 (중앙 상단)
- "What's New" 헤더
- 주요 변경사항: 5-6개 bullet (이모지 포함)
- CTA: "Try it now!" + 설치 명령어
- 워터마크

---

### Type 6: 퀴즈/챌린지
**용도**: 커뮤니티 참여 유도

```
┌─────────────────────────────────┐
│ 🧩 Vais Challenge #15           │
│                                 │
│ What does this code output?     │
│                                 │
│ ┌───────────────────────┐       │
│ │ F mystery(n:i64)      │       │
│ │   ->i64 =             │       │
│ │   I n<2 {n}           │       │
│ │   E {@(n-1)+@(n-2)}   │       │
│ │                       │       │
│ │ F main()->i64 =       │       │
│ │   mystery(7)          │       │
│ └───────────────────────┘       │
│                                 │
│ 💬 Answer in comments!          │
│                                 │
│         @vaislang               │
└─────────────────────────────────┘
```

**요소**:
- 아이콘: 🧩 + "Vais Challenge #N"
- 질문: "What does this code output?"
- 코드 블록: 미스터리 코드
- CTA: "Answer in comments!"

---

## 3. 실제 코드 예제 10개

### 1. Fibonacci (자재귀)
**카드 타입**: Type 1

```vais
# Fibonacci with self-recursion
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)

F main()->i64 = fib(10)  # Returns 55
```

**제목**: "Fibonacci in 1 Line"
**설명**: "The @ operator calls the current function recursively"

---

### 2. FizzBuzz (패턴 매칭)
**카드 타입**: Type 1

```vais
F fizzbuzz(n:i64)->i64 {
  M (n%3, n%5) {
    (0,0) => puts("FizzBuzz"),
    (0,_) => puts("Fizz"),
    (_,0) => puts("Buzz"),
    _ => print(n)
  }
  0
}
```

**제목**: "FizzBuzz with Pattern Matching"
**설명**: "Match on tuples for elegant logic"

---

### 3. Linked List (구조체 + 열거형)
**카드 타입**: Type 1

```vais
S Node<T> { val:T, next:List<T> }

E List<T> {
  Cons(Node<T>),
  Nil
}

F len<T>(list:List<T>)->i64 = M list {
  Cons(node) => 1 + @(node.next),
  Nil => 0
}
```

**제목**: "Generic Linked List"
**설명**: "Structs, enums, and generics working together"

---

### 4. Binary Search
**카드 타입**: Type 1

```vais
F bsearch(arr:[i64], target:i64)->i64 {
  L:lo:=0, hi:=arr.len()-1 {
    mid := (lo+hi)/2
    I arr[mid] == target { R mid }
    I arr[mid] < target { lo=mid+1 } E { hi=mid-1 }
    I lo>hi { R -1 }
  }
}
```

**제목**: "Binary Search"
**설명**: "Loop with early return"

---

### 5. 토큰 비교: Vais vs Rust
**카드 타입**: Type 2

**Vais** (8 tokens):
```vais
F fib(n:i64)->i64 = n<2?n:@(n-1)+@(n-2)
```

**Rust** (24 tokens):
```rust
fn fib(n: i64) -> i64 {
    if n < 2 { n } else { fib(n-1) + fib(n-2) }
}
```

**비교**:
- Vais: 8 tokens
- Rust: 24 tokens
- **67% reduction**

---

### 6. 패턴 매칭 (Option)
**카드 타입**: Type 4

```vais
U std/option

F unwrap_or<T>(opt:Option<T>, default:T)->T = M opt {
  Some(v) => v,
  None => default
}

F main()->i64 {
  x := Some(42)
  y := None
  unwrap_or(x, 0) + unwrap_or(y, 10)  # 52
}
```

**기능명**: "Pattern Matching"
**특징**:
- Exhaustive checking
- Type-safe extraction
- Expression-based

---

### 7. 클로저와 고차함수
**카드 타입**: Type 3 (Tip)

```vais
F map<T,U>(arr:[T], f:T->U)->[U] {
  result := []
  L x:arr { result.push(f(x)) }
  result
}

F main()->i64 {
  nums := [1,2,3,4,5]
  doubled := map(nums, |x| x*2)
  0
}
```

**팁**: "Vais Tip #23: Lambda Syntax"
**설명**: "Use |params| body for lambdas. Capture variables automatically!"

---

### 8. 구조체와 메서드
**카드 타입**: Type 1

```vais
S Point { x:f64, y:f64 }

I Point {
  F distance(self)->f64 = sqrt(self.x*self.x + self.y*self.y)
  F add(self, other:Point)->Point = Point{
    x: self.x + other.x,
    y: self.y + other.y
  }
}
```

**제목**: "Structs & Methods"
**설명**: "Implement methods with I blocks"

---

### 9. 에러 핸들링 (Result)
**카드 타입**: Type 4

```vais
U std/result

F divide(a:i64, b:i64)->Result<i64,String> =
  I b==0 { Err("Division by zero") }
  E { Ok(a/b) }

F main()->i64 {
  M divide(10, 2) {
    Ok(v) => v,      # 5
    Err(e) => 0
  }
}
```

**기능명**: "Error Handling"
**특징**:
- Result<T,E> type
- Explicit error handling
- Pattern matching on results

---

### 10. 비동기 프로그래밍 (미래 기능)
**카드 타입**: Type 5 (Release Note)

```vais
F fetch_data(url:String)->Future<String> = async {
  response := await http.get(url)
  response.text()
}

F main()->i64 = async {
  data := await fetch_data("https://api.example.com")
  puts(data)
  0
}
```

**버전**: "Vais 0.1.0"
**변경사항**:
- async/await 지원
- Future 타입
- 네트워크 표준 라이브러리
- 성능 최적화
- 문서 업데이트

---

## 4. 코드 카드 제작 가이드라인

### 코드 포맷팅
1. **들여쓰기**: 2 spaces (not tabs)
2. **줄 길이**: 최대 40자 (가독성)
3. **주석**: 필요시에만, # 사용
4. **공백**: 연산자 주변에 공백 유지

### 시각적 계층
1. **제목**: 가장 크고 굵게
2. **코드**: 중간 크기, 모노스페이스 폰트
3. **설명**: 작고 가늘게
4. **워터마크**: 가장 작게, 투명도 70%

### 색상 사용
- **배경**: 어두운 테마 (Dark Mode)
- **코드**: 구문 하이라이팅 (일관성 유지)
- **강조**: Vais Green 사용
- **대비**: WCAG AA 이상 (4.5:1)

### 접근성
- **대비비**: 최소 4.5:1
- **폰트 크기**: 최소 14px
- **alt 텍스트**: 코드 설명 포함

---

## 5. 사용 시나리오

### Daily Posts (매일)
- Type 3: Vais Tip #N
- Type 6: Vais Challenge #N

### Weekly Posts (주간)
- Type 1: Feature Showcase
- Type 2: Language Comparison

### Release Posts (릴리스 시)
- Type 5: Release Notes

### Educational Posts (교육)
- Type 4: Deep Dive into Features

---

## 6. 해시태그 전략

### 필수 해시태그
```
#Vais #VaisLang #SystemsProgramming #LLVM
```

### 상황별 해시태그
- **비교**: #Rust #Go #Programming
- **AI**: #AI #LLM #CodeGeneration
- **교육**: #LearnToCode #Programming101
- **커뮤니티**: #100DaysOfCode #DevCommunity

---

## 7. 템플릿 파일 위치

### 디자인 템플릿
- `community/templates/code-card.html` - 웹 기반 카드 생성기
- `community/templates/code-card.figma` - Figma 템플릿 (향후)
- `community/templates/code-card.sketch` - Sketch 템플릿 (향후)

### 예제 이미지
- `community/examples/type1-fibonacci.png`
- `community/examples/type2-vais-vs-rust.png`
- `community/examples/type3-tip-selfrecursion.png`
- `community/examples/type4-pattern-matching.png`
- `community/examples/type5-release-0.0.2.png`
- `community/examples/type6-challenge-15.png`

---

## 8. 자동화 도구

### 코드 카드 생성기
웹 기반 도구: `community/templates/code-card.html`

**사용법**:
1. 브라우저에서 `code-card.html` 열기
2. 카드 타입 선택
3. 제목 및 코드 입력
4. 미리보기 확인
5. PNG로 다운로드

**기능**:
- 실시간 미리보기
- Vais 구문 하이라이팅
- 다중 카드 타입 지원
- 반응형 디자인
- PNG 내보내기 (html2canvas)

---

## 9. 모범 사례

### DO
- 짧고 간결한 코드 사용
- 구문 하이라이팅 일관성 유지
- 브랜드 컬러 사용
- 워터마크 포함
- 읽기 쉬운 폰트 크기

### DON'T
- 너무 긴 코드 (10줄 이상)
- 복잡한 예제 (초보자 고려)
- 과도한 텍스트
- 낮은 대비비
- 로고 없이 게시

---

## 10. 성능 지표

### 목표
- **Instagram**: 1000+ likes/post
- **Twitter**: 500+ retweets/post
- **LinkedIn**: 100+ reactions/post

### 측정 항목
- 좋아요/리트윗/공유 수
- 댓글 수 및 품질
- 클릭률 (CTR)
- 웹사이트 방문 수
- GitHub 스타 증가

---

**마지막 업데이트**: 2026-01-31
**유지관리자**: Vais Community Team
**라이선스**: MIT
