# inkwell 직접 통합 설계 문서

> **작성일**: 2026-01-21
> **상태**: 설계 중
> **난이도**: 상

## 1. 개요

### 1.1 현재 상황
- `vais-codegen`은 LLVM IR을 **문자열**로 생성
- 생성된 IR 텍스트를 `clang`에 전달하여 네이티브 바이너리 생성
- 장점: LLVM 설치 불필요, 단순한 구현
- 단점: 타입 안전성 부족, 런타임 IR 파싱 오버헤드, 에러 감지 지연

### 1.2 목표
- inkwell (Rust LLVM 바인딩)을 사용하여 LLVM API 직접 호출
- 컴파일 타임 타입 안전성 확보
- 더 빠른 코드 생성 (문자열 파싱 제거)
- JIT 컴파일 기반 마련

### 1.3 범위
**포함:**
- CodeGenerator를 inkwell 기반으로 리팩토링
- 기존 기능 100% 유지
- 테스트 통과 보장

**제외 (후속 작업):**
- JIT 실행 (별도 Phase)
- Self-hosting (별도 Phase)

---

## 2. 아키텍처

### 2.1 현재 구조
```
vais-codegen/
├── lib.rs          # CodeGenerator (문자열 IR 생성)
├── expr.rs         # 표현식 코드 생성
├── expr_helpers.rs # 표현식 헬퍼
├── expr_visitor.rs # 표현식 Visitor
├── stmt.rs         # 문장 코드 생성
├── stmt_visitor.rs # 문장 Visitor
├── types.rs        # 타입 정의
├── builtins.rs     # 빌트인 함수
├── debug.rs        # DWARF 디버그 정보
├── formatter.rs    # 코드 포맷터
├── optimize.rs     # 최적화 패스
└── visitor.rs      # Visitor 트레이트
```

### 2.2 새 구조
```
vais-codegen/
├── lib.rs              # 공통 인터페이스 및 타입
├── text/               # 기존 문자열 기반 (호환성 유지)
│   ├── mod.rs
│   ├── generator.rs
│   └── ...
├── inkwell/            # 새 inkwell 기반
│   ├── mod.rs
│   ├── generator.rs    # InkwellCodeGenerator
│   ├── types.rs        # LLVM 타입 매핑
│   ├── builtins.rs     # 빌트인 함수
│   └── debug.rs        # DWARF 디버그 (inkwell API)
├── formatter.rs        # (공통)
├── optimize.rs         # (공통 - inkwell 패스 매니저 추가)
└── visitor.rs          # (공통)
```

### 2.3 Feature Flag 전략
```toml
[features]
default = ["text-codegen"]
text-codegen = []
inkwell-codegen = ["inkwell"]
```

- `text-codegen`: 기존 문자열 기반 (기본값, LLVM 불필요)
- `inkwell-codegen`: inkwell 기반 (LLVM 17+ 필요)

---

## 3. 핵심 컴포넌트 설계

### 3.1 InkwellCodeGenerator

```rust
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::types::*;
use inkwell::values::*;

pub struct InkwellCodeGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    // 함수 정보
    functions: HashMap<String, FunctionValue<'ctx>>,

    // 구조체 정보
    structs: HashMap<String, StructType<'ctx>>,

    // 로컬 변수 (alloca 포인터)
    locals: HashMap<String, PointerValue<'ctx>>,

    // 문자열 상수
    string_constants: HashMap<String, GlobalValue<'ctx>>,

    // 루프 스택
    loop_stack: Vec<InkwellLoopLabels<'ctx>>,

    // 디버그 정보
    debug_info: Option<InkwellDebugInfo<'ctx>>,

    // 제네릭 치환
    generic_substitutions: HashMap<String, ResolvedType>,
}
```

### 3.2 타입 매핑

| Vais Type | LLVM Type (inkwell) |
|-----------|---------------------|
| `i8` | `context.i8_type()` |
| `i16` | `context.i16_type()` |
| `i32` | `context.i32_type()` |
| `i64` | `context.i64_type()` |
| `f32` | `context.f32_type()` |
| `f64` | `context.f64_type()` |
| `bool` | `context.bool_type()` |
| `str` | `context.i8_type().ptr_type(AddressSpace::default())` |
| `[T; N]` | `T.array_type(N)` |
| `*T` | `T.ptr_type(AddressSpace::default())` |
| `S` (struct) | `context.struct_type(&[...], false)` |
| `()` | `context.void_type()` |

### 3.3 표현식 생성

```rust
impl<'ctx> InkwellCodeGenerator<'ctx> {
    fn gen_expr(&mut self, expr: &Expr) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        match &expr.kind {
            ExprKind::Literal(lit) => self.gen_literal(lit),
            ExprKind::Var(name) => self.gen_var(name),
            ExprKind::Binary(op, lhs, rhs) => self.gen_binary(*op, lhs, rhs),
            ExprKind::Call(callee, args) => self.gen_call(callee, args),
            // ...
        }
    }

    fn gen_literal(&mut self, lit: &Literal) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        match lit {
            Literal::Int(n) => {
                Ok(self.context.i64_type().const_int(*n as u64, true).into())
            }
            Literal::Float(f) => {
                Ok(self.context.f64_type().const_float(*f).into())
            }
            Literal::Bool(b) => {
                Ok(self.context.bool_type().const_int(*b as u64, false).into())
            }
            Literal::String(s) => {
                self.gen_string_literal(s)
            }
            // ...
        }
    }
}
```

### 3.4 빌트인 함수

```rust
impl<'ctx> InkwellCodeGenerator<'ctx> {
    fn declare_builtins(&mut self) {
        // puts(str) -> i32
        let puts_type = self.context.i32_type().fn_type(
            &[self.context.i8_type().ptr_type(AddressSpace::default()).into()],
            false
        );
        self.module.add_function("puts", puts_type, None);

        // malloc(size) -> *i8
        let malloc_type = self.context.i8_type().ptr_type(AddressSpace::default()).fn_type(
            &[self.context.i64_type().into()],
            false
        );
        self.module.add_function("malloc", malloc_type, None);

        // free(*i8) -> void
        let free_type = self.context.void_type().fn_type(
            &[self.context.i8_type().ptr_type(AddressSpace::default()).into()],
            false
        );
        self.module.add_function("free", free_type, None);

        // ... 기타 빌트인
    }
}
```

---

## 4. 마이그레이션 계획

### Phase 1: 인프라 구축 (1단계)
1. [ ] inkwell 의존성 추가
2. [ ] feature flag 설정
3. [ ] 공통 인터페이스 정의 (trait CodeGenBackend)
4. [ ] InkwellCodeGenerator 기본 구조 생성

### Phase 2: 핵심 기능 (2단계)
1. [ ] 리터럴 생성 (Int, Float, Bool, String)
2. [ ] 변수 (alloca, load, store)
3. [ ] 산술/비교 연산
4. [ ] 함수 정의 및 호출
5. [ ] 제어 흐름 (if, loop, match)

### Phase 3: 고급 기능 (3단계)
1. [ ] 구조체 및 열거형
2. [ ] 클로저 및 람다
3. [ ] 제네릭 인스턴스화
4. [ ] async/await
5. [ ] 디버그 정보

### Phase 4: 최적화 및 검증 (4단계)
1. [ ] LLVM 패스 매니저 통합
2. [ ] 기존 테스트 전부 통과
3. [ ] 성능 벤치마크
4. [ ] 문서 업데이트

---

## 5. 의존성

### Cargo.toml
```toml
[dependencies]
inkwell = { version = "0.4", features = ["llvm17-0"], optional = true }

[features]
default = ["text-codegen"]
text-codegen = []
inkwell-codegen = ["inkwell"]
```

### 시스템 요구사항
- LLVM 17+ 설치 필요 (inkwell-codegen 사용 시)
- macOS: `brew install llvm@17`
- Ubuntu: `apt install llvm-17-dev`

---

## 6. 공통 인터페이스 설계

```rust
/// Backend-agnostic code generation trait
pub trait CodeGenBackend {
    type Output;
    type Error;

    fn generate_module(&mut self, module: &Module) -> Result<Self::Output, Self::Error>;
    fn generate_function(&mut self, func: &Function) -> Result<(), Self::Error>;
    fn generate_expr(&mut self, expr: &Expr) -> Result<String, Self::Error>;
    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<(), Self::Error>;
}

/// Text-based backend (current)
impl CodeGenBackend for CodeGenerator {
    type Output = String;  // LLVM IR text
    type Error = CodegenError;
    // ...
}

/// Inkwell-based backend (new)
#[cfg(feature = "inkwell-codegen")]
impl<'ctx> CodeGenBackend for InkwellCodeGenerator<'ctx> {
    type Output = Module<'ctx>;  // LLVM Module
    type Error = CodegenError;
    // ...
}
```

---

## 7. 리스크 및 대응

| 리스크 | 영향 | 대응 |
|--------|------|------|
| LLVM 버전 호환성 | 빌드 실패 | feature flag로 분리, CI에서 LLVM 버전 테스트 |
| 기존 기능 누락 | 회귀 버그 | 기존 테스트 100% 통과 필수 |
| 성능 저하 | 컴파일 속도 | 벤치마크 비교, 필요시 캐싱 추가 |
| API 복잡도 | 유지보수 어려움 | 명확한 추상화 계층 유지 |

---

## 8. 예상 일정

| 단계 | 작업 | 예상 시간 |
|------|------|----------|
| Phase 1 | 인프라 구축 | 1-2시간 |
| Phase 2 | 핵심 기능 | 4-6시간 |
| Phase 3 | 고급 기능 | 4-6시간 |
| Phase 4 | 검증 | 2-3시간 |

**총 예상**: 11-17시간 (복잡도에 따라 변동)

---

## 9. 참고 자료

- [inkwell 문서](https://thedan64.github.io/inkwell/)
- [LLVM Language Reference](https://llvm.org/docs/LangRef.html)
- [Rust LLVM tutorial](https://github.com/jauhien/iron-llvm)
