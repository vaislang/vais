# 컴파일러 내부 구조

이 문서는 Vais 컴파일러의 내부 아키텍처와 각 컴파일 단계를 설명합니다.

## 개요

Vais 컴파일러는 전통적인 다단계 파이프라인 구조를 따릅니다:

```
┌──────────────┐
│ .vais source │
└──────┬───────┘
       │
       ▼
┌──────────────┐      vais-lexer (logos 기반)
│    Lexer     │
└──────┬───────┘
       │ Tokens
       ▼
┌──────────────┐      vais-parser (재귀 하강)
│    Parser    │
└──────┬───────┘
       │ AST
       ▼
┌──────────────┐      vais-types (양방향 추론)
│ Type Checker │
└──────┬───────┘
       │ Typed AST
       ▼
┌──────────────┐      vais-mir (Borrow Checker)
│     MIR      │
└──────┬───────┘
       │
       ├─────────────┬─────────────┬──────────────┐
       ▼             ▼             ▼              ▼
  ┌─────────┐  ┌─────────┐  ┌─────────┐   ┌─────────┐
  │ LLVM IR │  │   .mjs  │  │  .wasm  │   │   JIT   │
  └────┬────┘  └─────────┘  └─────────┘   └─────────┘
       │
       ▼ clang
  ┌─────────┐
  │ binary  │
  └─────────┘
```

### 주요 Crate 구조

| Crate | 역할 |
|-------|------|
| `vais-lexer` | 소스 코드 → 토큰 스트림 |
| `vais-parser` | 토큰 → AST |
| `vais-ast` | AST 타입 정의 |
| `vais-types` | 타입 체킹 & 추론 |
| `vais-mir` | MIR 변환 & Borrow Checker |
| `vais-codegen` | LLVM IR 생성 (Inkwell) |
| `vais-codegen-js` | JavaScript ESM 생성 |
| `vais-jit` | Cranelift JIT 컴파일 |
| `vaisc` | CLI 드라이버 |

## 렉서 (Lexer)

### 토큰화 엔진

Vais는 [logos](https://github.com/maciejhirsz/logos) 라이브러리를 사용하여 고성능 토큰화를 수행합니다.

```rust
// vais-lexer/src/lib.rs
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    #[token("F")] Function,
    #[token("S")] Struct,
    #[token("E")] Enum,
    #[token("I")] If,
    #[token("L")] Loop,
    #[token("M")] Match,
    #[token("R")] Return,
    // ...
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")] Identifier,
    #[regex(r"[0-9]+")] IntLiteral,
    // ...
}
```

### 단일 문자 키워드 매핑

| 키워드 | 전체 이름 | 의미 |
|-------|----------|------|
| `F` | function | 함수 정의 |
| `S` | struct | 구조체 정의 |
| `E` | enum/else | 열거형 또는 else |
| `I` | if | 조건문 |
| `L` | loop | 무한 루프 |
| `M` | match | 패턴 매칭 |
| `R` | return | 함수 반환 |
| `B` | break | 루프 탈출 |
| `C` | continue | 다음 반복 |
| `T` | type | 타입 별칭 |
| `U` | use | 모듈 임포트 |
| `W` | trait | 트레잇 정의 |
| `X` | impl | 구현 블록 |
| `P` | pub | 공개 가시성 |
| `D` | defer | 지연 실행 |
| `A` | async | 비동기 함수 |
| `Y` | await | 비동기 대기 |
| `N` | extern | 외부 함수 |
| `G` | global | 전역 변수 |
| `O` | union | 공용체 |

### 특수 연산자

- `@` - self-recursion (현재 함수 재귀 호출)
- `:=` - 변수 바인딩
- `?` - try operator (Result/Option)
- `!` - unwrap operator
- `|>` - pipe operator
- `~` - 문자열 보간 (예: `~{expr}`)

### 성능 특성

- **Zero-copy**: 소스 문자열을 복사하지 않고 Span으로 참조
- **컴파일 타임 최적화**: logos는 DFA 기반 매칭 코드를 생성
- **벤치마크**: 50K 라인 → ~2ms (logos의 기여가 큼)

## 파서 (Parser)

### 재귀 하강 파서

Vais 파서는 수동으로 작성된 재귀 하강 파서입니다. LL(k) 문법을 지원하며, 대부분의 경우 1-lookahead로 충분합니다.

```rust
// vais-parser/src/lib.rs
pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn parse_module(&mut self) -> Result<Module, ParseError> {
        let mut items = vec![];
        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }
        Ok(Module { items })
    }
}
```

### 모듈식 구조

파서는 파싱 로직을 여러 모듈로 분할합니다:

| 파일 | 담당 영역 |
|------|----------|
| `lib.rs` | 파서 드라이버, 모듈 파싱 |
| `item.rs` | 최상위 아이템 (함수, 구조체, enum) |
| `types.rs` | 타입 표현식 파싱 |
| `expr.rs` | 표현식 파싱 (우선순위 등반) |
| `stmt.rs` | 구문 파싱 |
| `pattern.rs` | 패턴 파싱 (match arms) |

### AST 노드 타입

```rust
// vais-ast/src/lib.rs
pub enum Item {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    TypeAlias(TypeAlias),
    Use(Use),
    Trait(Trait),
    Impl(Impl),
}

pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub generic_params: Vec<String>,
    pub attributes: Vec<Attribute>,
}

pub enum Expr {
    IntLiteral(i64),
    BinaryOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Call { func: Box<Expr>, args: Vec<Expr> },
    FieldAccess { expr: Box<Expr>, field: String },
    If { cond: Box<Expr>, then: Block, else_: Option<Block> },
    Match { expr: Box<Expr>, arms: Vec<MatchArm> },
    // ...
}
```

### 에러 복구

파서는 panic 기반 예외 대신 `Result<T, ParseError>` 패턴을 사용합니다:

```rust
pub enum ParseError {
    UnexpectedToken { expected: String, found: Token },
    UnexpectedEof,
    InvalidSyntax { message: String },
}
```

에러 발생 시 Miette/Ariadne를 통해 진단 메시지를 생성합니다.

## 타입 체커 (Type Checker)

### 양방향 타입 추론

Vais는 **bidirectional type checking**을 사용합니다:

1. **Inference mode**: 표현식에서 타입을 추론 (bottom-up)
2. **Checking mode**: 기대 타입과 비교 검증 (top-down)

```rust
// vais-types/src/checker_expr.rs
impl TypeChecker {
    pub fn infer_expr(&mut self, expr: &Expr) -> Result<ResolvedType, TypeError> {
        match expr {
            Expr::IntLiteral(_) => Ok(ResolvedType::I64),
            Expr::BinaryOp { op, left, right } => {
                let left_ty = self.infer_expr(left)?;
                let right_ty = self.infer_expr(right)?;
                self.check_binary_op(op, &left_ty, &right_ty)
            }
            // ...
        }
    }

    pub fn check_expr(&mut self, expr: &Expr, expected: &ResolvedType)
        -> Result<(), TypeError> {
        let inferred = self.infer_expr(expr)?;
        self.unify(&inferred, expected)
    }
}
```

### 제네릭 해결

제네릭 함수/구조체는 **인스턴스화 시점**에 타입 파라미터를 구체 타입으로 치환합니다:

```rust
// vais-types/src/inference.rs
pub fn substitute_generics(
    ty: &ResolvedType,
    substitutions: &HashMap<String, ResolvedType>
) -> ResolvedType {
    match ty {
        ResolvedType::Generic(name) => {
            substitutions.get(name).cloned()
                .unwrap_or_else(|| ty.clone())
        }
        ResolvedType::Struct { name, type_args } => {
            let new_args = type_args.iter()
                .map(|arg| substitute_generics(arg, substitutions))
                .collect();
            ResolvedType::Struct { name: name.clone(), type_args: new_args }
        }
        // ...
    }
}
```

### 제약 해결 (Constraint Solving)

트레잇 바운드 및 타입 제약은 **Hindley-Milner 기반** 단일화(unification)로 해결됩니다:

```rust
pub fn unify(&mut self, a: &ResolvedType, b: &ResolvedType)
    -> Result<(), TypeError> {
    match (a, b) {
        (ResolvedType::I64, ResolvedType::I64) => Ok(()),
        (ResolvedType::Generic(name), ty) | (ty, ResolvedType::Generic(name)) => {
            self.bind_generic(name, ty)
        }
        (ResolvedType::Function { params: p1, ret: r1 },
         ResolvedType::Function { params: p2, ret: r2 }) => {
            for (t1, t2) in p1.iter().zip(p2.iter()) {
                self.unify(t1, t2)?;
            }
            self.unify(r1, r2)
        }
        _ => Err(TypeError::TypeMismatch { expected: b.clone(), found: a.clone() })
    }
}
```

### Result/Option 타입 시스템

Vais는 enum 기반 에러 처리를 사용합니다:

```vais
E Result<T, E> {
    Ok(T),
    Err(E),
}

E Option<T> {
    Some(T),
    None,
}
```

- `?` operator: `Result::Err` 또는 `Option::None` 시 early return
- `!` operator: unwrap (런타임 panic)

타입 체커는 `?` 사용 시 함수의 리턴 타입이 호환 가능한지 검증합니다.

## 중간 표현 (MIR)

### MIR 설계 목적

**MIR (Mid-level Intermediate Representation)**은 AST와 LLVM IR 사이의 중간 계층입니다:

1. **Borrow Checking**: 소유권/차용 규칙 검증
2. **Lifetime Inference**: 참조의 생명주기 분석
3. **최적화 패스**: 고수준 최적화 수행
4. **플랫폼 독립적**: LLVM, JS, WASM 공통 표현

```rust
// vais-mir/src/lib.rs
pub struct Body {
    pub locals: Vec<LocalDecl>,
    pub basic_blocks: Vec<BasicBlock>,
    pub lifetime_params: Vec<String>,
    pub lifetime_bounds: Vec<LifetimeBound>,
}

pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

pub enum Statement {
    Assign { place: Place, rvalue: Rvalue },
    StorageLive(Local),
    StorageDead(Local),
}

pub enum Terminator {
    Return,
    Goto { target: BasicBlock },
    SwitchInt { discr: Operand, targets: Vec<(u128, BasicBlock)> },
    Call { func: Operand, args: Vec<Operand>, destination: Place, target: BasicBlock },
}
```

### Borrow Checker 통합

MIR 생성 후 Borrow Checker가 다음을 검증합니다:

| 에러 코드 | 설명 |
|----------|------|
| E100 | Use After Move |
| E101 | Double Free |
| E102 | Use After Free |
| E103 | Mutable Borrow Conflict |
| E104 | Borrow While Mutably Borrowed |
| E105 | Move While Borrowed |
| E106 | Lifetime Violation |

```rust
// vais-mir/src/borrow_checker.rs
pub fn check_borrows(body: &Body) -> Result<(), BorrowError> {
    let mut checker = BorrowChecker::new();

    for block in &body.basic_blocks {
        for stmt in &block.statements {
            checker.visit_statement(stmt)?;
        }
        checker.visit_terminator(&block.terminator)?;
    }

    Ok(())
}
```

### 최적화 패스

MIR 레벨에서 수행되는 주요 최적화:

1. **Dead Code Elimination**: 미사용 변수/함수 제거
2. **Constant Folding**: 컴파일 타임 상수 계산
3. **Inlining**: 작은 함수 인라인화
4. **Alias Analysis**: 포인터 별칭 분석
5. **Bounds Check Elimination**: 범위 검사 제거
6. **Loop Vectorization**: SIMD 자동 벡터화
7. **Memory Layout Optimization**: 구조체 필드 재배치

```rust
// selfhost/mir_optimizer.vais
F mir_advanced_optimize_body(body: MirBody) -> MirBody {
    body := mir_alias_analysis(body)
    body := mir_bounds_check_elimination(body)
    body := mir_vectorize(body)
    body := mir_layout_optimization(body)
    body
}
```

## 코드 생성 (Codegen)

### LLVM IR 생성 (Inkwell)

Vais는 [Inkwell](https://github.com/TheDan64/inkwell) (LLVM 17)을 사용하여 LLVM IR을 생성합니다:

```rust
// vais-codegen/src/inkwell/generator.rs
pub struct CodeGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    functions: HashMap<String, FunctionValue<'ctx>>,
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn generate_function(&mut self, func: &Function) -> Result<()> {
        let fn_type = self.resolve_function_type(func)?;
        let fn_val = self.module.add_function(&func.name, fn_type, None);

        let entry_bb = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry_bb);

        for stmt in &func.body.statements {
            self.generate_statement(stmt)?;
        }

        Ok(())
    }
}
```

### Text IR vs Inkwell 경로

Vais는 두 가지 LLVM IR 생성 경로를 지원합니다:

| 경로 | 사용 시점 | 특징 |
|------|----------|------|
| **Text IR** | `--emit-ir` 플래그 없을 때 | 문자열 기반, 빠른 프로토타이핑 |
| **Inkwell** | `--emit-ir` 플래그 있을 때 (기본) | 타입 안전, 최적화 지원 |

```rust
// vaisc/src/main.rs
if use_inkwell {
    // Inkwell 백엔드 사용 (기본값)
    let codegen = InkwellCodeGenerator::new();
    codegen.generate_module(&typed_ast)?;
} else {
    // Text IR 백엔드 사용
    let ir = generate_text_ir(&typed_ast);
    std::fs::write("output.ll", ir)?;
}
```

### JavaScript ESM 생성

`--target js` 플래그 사용 시 JavaScript ESM 코드를 생성합니다:

```rust
// vais-codegen-js/src/lib.rs
pub fn generate_js(module: &Module) -> String {
    let mut output = String::new();

    for item in &module.items {
        match item {
            Item::Function(func) => {
                output.push_str(&format!("export function {}(", func.name));
                // 파라미터, 함수 본문 생성...
            }
        }
    }

    output
}
```

생성된 JavaScript는 **ES2020** 표준을 따르며 다음을 지원합니다:

- BigInt (i64/u64)
- Async/Await
- Module imports/exports
- TypedArray (배열 연산)

### WASM 코드 생성

`--target wasm32-unknown-unknown` 사용 시 WebAssembly 바이너리를 생성합니다:

```rust
// vais-codegen/src/wasm.rs
pub fn generate_wasm_module(module: &Module) -> Vec<u8> {
    let config = WasmConfig {
        import_memory: true,
        export_table: true,
    };

    // LLVM을 통해 WASM 바이너리 생성
    let target = Target::from_name("wasm32-unknown-unknown").unwrap();
    // ...
}
```

WASM 타겟은 다음을 지원합니다:

- **WASI**: 시스템 호출 인터페이스
- **JS Interop**: `#[wasm_import]` / `#[wasm_export]` 어트리뷰트
- **Component Model**: WebAssembly Component 표준

## 최적화

### 최적화 레벨

| 레벨 | 플래그 | 설명 | LLVM Pass |
|------|-------|------|-----------|
| O0 | (기본값) | 최적화 없음 | `-O0` |
| O1 | `-O1` | 기본 최적화 | `-O1` |
| O2 | `-O2` | 중간 최적화 | `-O2` |
| O3 | `-O3` | 최대 최적화 | `-O3` |

```bash
vaisc -O3 program.vais  # 최대 최적화
```

### 주요 최적화 패스

#### 1. 인라인화 (Inlining)

작은 함수를 호출 지점에 인라인화:

```rust
#[inline]  // 힌트 제공
F small_function(x: i64) -> i64 {
    x * 2
}
```

#### 2. 루프 최적화

- **Loop Unrolling**: 루프 펼치기
- **Loop Vectorization**: SIMD 변환
- **Loop Invariant Code Motion**: 불변식 이동

```vais
# 벡터화 가능한 루프
L i := 0; i < 1000; i := i + 1 {
    arr[i] := arr[i] * 2  # SIMD 명령어로 변환 가능
}
```

#### 3. 메모리 최적화

- **Escape Analysis**: 힙→스택 할당 변환
- **Dead Store Elimination**: 불필요한 저장 제거
- **Memory Layout**: 구조체 필드 재배치 (캐시 최적화)

```rust
// vais-codegen/src/advanced_opt/mir_layout.rs
// 핫 필드를 앞에 배치 (캐시 라인 효율)
S OptimizedStruct {
    hot_field: i64,    // 자주 접근
    cold_field: i64,   // 드물게 접근
}
```

### 병렬 컴파일

Vais는 모듈 의존성 그래프를 기반으로 병렬 컴파일을 수행합니다:

```rust
// vaisc/src/parallel.rs
pub fn parallel_compile(modules: Vec<Module>) -> Result<Vec<CompiledModule>> {
    let dag = DependencyGraph::build(&modules)?;
    let levels = dag.topological_sort()?;

    let (tx, rx) = mpsc::sync_channel(4);

    for level in levels {
        // 같은 레벨의 모듈은 병렬 컴파일 가능
        level.par_iter().for_each(|module| {
            let result = compile_module(module);
            tx.send(result).unwrap();
        });
    }

    // 결과 수집
    rx.iter().collect()
}
```

**성능 향상**:
- 파싱: 2.18x speedup (10 모듈)
- 코드 생성: 4.14x speedup (10 모듈)

## JIT 컴파일

### Cranelift 기반 JIT

Vais는 [Cranelift](https://cranelift.dev/)를 사용하여 즉시 실행 컴파일을 지원합니다:

```rust
// vais-jit/src/lib.rs
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};

pub struct JitCompiler {
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    module: JITModule,
}

impl JitCompiler {
    pub fn compile_and_run(&mut self, func: &Function) -> Result<i64> {
        // Cranelift IR로 변환
        let mut func_ctx = self.ctx.func;
        let entry_block = func_ctx.dfg.make_block();

        for stmt in &func.body.statements {
            self.translate_statement(stmt, &mut func_ctx)?;
        }

        // JIT 컴파일
        let id = self.module.declare_function(&func.name, Linkage::Export, &func_ctx.signature)?;
        self.module.define_function(id, &mut self.ctx)?;
        self.module.finalize_definitions();

        // 실행
        let code = self.module.get_finalized_function(id);
        let code_fn = unsafe { std::mem::transmute::<_, fn() -> i64>(code) };
        Ok(code_fn())
    }
}
```

### REPL 통합

JIT는 REPL에서 즉시 코드를 실행하는 데 사용됩니다:

```bash
$ vaisc --repl
vais> F fib(n: i64) -> i64 { n < 2 ? n : @(n-1) + @(n-2) }
vais> fib(10)
55
vais> :jit-stats
JIT Statistics:
  Compiled functions: 1
  Execution time: 0.023ms
  Tier: Baseline
```

### OSR (On-Stack Replacement)

핫 루프를 감지하여 실행 중 최적화된 코드로 전환:

```rust
// vais-jit/src/osr.rs
pub struct OsrPoint {
    pub loop_id: usize,
    pub hot_path_score: u64,
    pub compiled_code: Option<*const u8>,
}

pub fn check_osr(osr_point: &mut OsrPoint) -> bool {
    osr_point.hot_path_score += 1;
    if osr_point.hot_path_score > OSR_THRESHOLD {
        // 최적화된 버전으로 전환
        return true;
    }
    false
}
```

## 성능 벤치마크

### 컴파일 속도

| 소스 크기 | 파싱 시간 | 타입 체킹 | 코드 생성 | 전체 |
|----------|----------|----------|----------|------|
| 1K lines | 0.4ms | 0.8ms | 1.2ms | 2.4ms |
| 10K lines | 3.5ms | 7.2ms | 12.1ms | 22.8ms |
| 50K lines | 15.8ms | 35.3ms | 30.1ms | 81.2ms |
| 100K lines | 32.4ms | 71.5ms | 64.8ms | 168.7ms |

*측정 환경: M1 Pro, 10-core, 32GB RAM*

### 실행 속도

Fibonacci(35) 벤치마크 (단위: ms):

| 언어 | 시간 | 상대 속도 |
|------|------|----------|
| Vais (LLVM -O3) | 42.3 | 1.00x |
| Rust (rustc -O) | 41.8 | 0.99x |
| C (clang -O3) | 40.5 | 0.96x |
| Go (gc -O) | 58.7 | 1.39x |
| Vais (JIT) | 156.4 | 3.70x |

## 디버깅 & 진단

### IR 덤프

```bash
# LLVM IR 출력
vaisc --emit-ir program.vais

# MIR 출력
vaisc --emit-mir program.vais

# AST 출력
vaisc --dump-ast program.vais
```

### 에러 메시지

Vais는 Miette와 Ariadne를 사용하여 풍부한 에러 메시지를 제공합니다:

```
error[E032]: type inference failed
  ┌─ example.vais:5:12
  │
5 │ F add(a, b) { a + b }
  │       ^^^^ cannot infer type for parameter 'a'
  │
  = help: add explicit type annotation: `a: i64`
```

### 컴파일러 프로파일링

```bash
# 컴파일러 자체의 성능 프로파일링
vaisc --profile program.vais

# 출력:
# Phase breakdown:
#   Lexing:      2.3ms (3.5%)
#   Parsing:     8.7ms (13.2%)
#   Type Check:  31.2ms (47.3%)
#   Codegen:     23.8ms (36.0%)
#   Total:       66.0ms
```

## 참고 자료

- [LLVM Language Reference](https://llvm.org/docs/LangRef.html)
- [Cranelift Documentation](https://docs.rs/cranelift/)
- [Inkwell Documentation](https://thedan64.github.io/inkwell/)
- [Type Inference (Hindley-Milner)](https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system)
- [MIR Design (Rust)](https://rustc-dev-guide.rust-lang.org/mir/index.html)

## 다음 단계

- [최적화 가이드](./optimization.md) - 컴파일러 최적화 상세 가이드
- [MIR 사양](./mir-spec.md) - MIR 형식 정의
- [Borrow Checker](./borrow-checker.md) - 소유권 시스템 상세
