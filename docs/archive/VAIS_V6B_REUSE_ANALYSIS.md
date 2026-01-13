# Vais v6b 기존 코드 재사용 분석

**Date:** 2026-01-12

---

## 요약

| 컴포넌트 | 재사용 가능 | 수정 필요 | 비고 |
|----------|-------------|-----------|------|
| vais-ir/value.rs | 100% | - | Value enum 그대로 사용 |
| vais-ir/instruction.rs | 90% | 확장 | OpCode 일부 추가 필요 |
| vais-vm/vm.rs | 80% | 확장 | 새 OpCode 처리 추가 |
| vais-vm/builtins.rs | 70% | 확장 | 새 빌트인 ~15개 추가 |
| vais-lexer/* | 0% | 전면 재작성 | v6b 문법 완전히 다름 |
| vais-parser/* | 0% | 전면 재작성 | v6b 문법 완전히 다름 |
| vais-ast/* | 30% | 대부분 재작성 | 표현식 노드만 일부 참조 |
| vais-typeck/* | 50% | 수정 | 타입 추론 로직 재사용 |

---

## 1. 완전 재사용 가능 (100%)

### 1.1 vais-ir/value.rs

```rust
pub enum Value {
    Void,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Struct { name: String, fields: HashMap<String, Value> },
    Optional(Option<Box<Value>>),
    Error(String),
}
```

**판단:** v6b에서 필요한 모든 값 타입 지원. 그대로 사용.

---

## 2. 확장 필요 (70-90%)

### 2.1 vais-ir/instruction.rs

**현재 지원 OpCode:**
- Stack: Const, Pop, Dup
- Variables: Load, Store, LoadInput, StoreOutput
- Arithmetic: Add, Sub, Mul, Div, Neg
- Comparison: Eq, Neq, Lt, Gt, Lte, Gte
- Logical: And, Or, Not
- Collection: Len, Index, GetField, MakeArray, MakeStruct
- Array: Map, Filter, Reduce
- Control: Jump, JumpIf, JumpIfNot, CallNode, Return
- Special: CallBuiltin, Nop, Halt, Error

**v6b 추가 필요:**
```rust
// 새로 추가할 OpCode
Mod,          // % 연산
Slice,        // 배열/문자열 슬라이싱
Range,        // 범위 생성 (..)
In,           // @ 연산자 (포함 여부)
RecurseCall,  // $ 재귀 호출
```

### 2.2 vais-vm/builtins.rs

**현재 지원:**
- LEN, UPPER, LOWER, TRIM, CONTAINS
- ABS, MIN, MAX, SUM, AVG
- MAP, FILTER, REDUCE

**v6b 추가 필요 (~15개):**
```rust
// Collection 추가
FIRST, LAST, NTH, FLIP, SET, FLATTEN
SORT, SORTD, TAKE, DROP, SLICE

// Search 추가
FIND, FINDL, IDX, IDXF, ARGMAX, ARGMIN

// Aggregation 추가
ALL, ANY, NONE, COUNT

// String 추가
SPLIT, JOIN, REP, SUB, REPL, STARTS, ENDS, TRIML, TRIMR

// Math 추가
NEG, FLOOR, CEIL, ROUND, POW, SQRT

// Utility 추가
ZIP, RANGE, ENUM, KEYS, VALS

// Error 추가
ERR, TRY, NILQ, OKQ
```

### 2.3 vais-vm/vm.rs

**재사용 가능:**
- 스택 기반 실행 로직
- Map/Filter/Reduce 처리
- 변수 스코프 관리
- 에러 처리

**수정 필요:**
- 새 OpCode 처리 추가
- 재귀 호출 ($) 스택 관리
- 새 빌트인 호출 연동

---

## 3. 전면 재작성 필요 (0%)

### 3.1 vais-lexer

**이유:** v1 문법과 v6b 문법이 완전히 다름

**v1 토큰 (현재):**
```
UNIT, META, ENDMETA, INPUT, OUTPUT, FLOW...
INT, STRING, BOOL, ARRAY, STRUCT...
```

**v6b 토큰 (새로):**
```
Identifier, Integer, Float, String
LParen, RParen, LBracket, RBracket, LBrace, RBrace
Equals, Comma, Colon, Dot, DotAt, DotQuestion, DotSlash
Question, Dollar, Hash, Underscore, At, DotDot
Plus, Minus, Star, Slash, Percent
Lt, Gt, Lte, Gte, EqEq, Neq, And, Or, Not
Let, Nil, Err, True, False
```

### 3.2 vais-parser

**이유:** 파싱 대상 문법이 완전히 다름

**v1 구조:**
```
UNIT name FUNCTION
  META ... ENDMETA
  INPUT ... ENDINPUT
  FLOW ... ENDFLOW
END
```

**v6b 구조:**
```
name(params) = body
```

---

## 4. 부분 재사용 가능 (30-50%)

### 4.1 vais-ast

**재사용 가능:**
- 표현식 노드 개념 (BinaryExpr, UnaryExpr, CallExpr, IndexExpr)
- 타입 표현 구조

**재작성 필요:**
- FunctionDef 노드
- Lambda 노드 (implicit _ parameter)
- ChainExpr 노드 (.@, .?, ./)
- LetExpr 노드
- TernaryExpr 노드

### 4.2 vais-typeck

**재사용 가능:**
- 기본 타입 추론 알고리즘
- 타입 통합 로직
- 에러 보고 구조

**수정 필요:**
- 람다 인자 타입 추론 (_)
- 체이닝 연산 타입 전파
- Optional 타입 처리 (?T)

---

## 5. 구현 우선순위

### Phase 1-A: 새로 구현 (핵심)
1. **vais-lexer-v6b** - 새 렉서
2. **vais-parser-v6b** - 새 파서
3. **vais-ast-v6b** - 새 AST 정의

### Phase 1-B: 확장 (기존 활용)
4. **vais-ir** - OpCode 추가
5. **vais-vm** - 새 OpCode 처리
6. **vais-vm/builtins** - 빌트인 추가

### Phase 1-C: 통합
7. **vais-cli** - 새 파이프라인 연결
8. **테스트** - 벤치마크 예제로 검증

---

## 6. 예상 작업량

| 컴포넌트 | 예상 라인 | 복잡도 |
|----------|-----------|--------|
| lexer-v6b | ~300 | 중 |
| parser-v6b | ~800 | 상 |
| ast-v6b | ~200 | 중 |
| ir 확장 | ~100 | 하 |
| vm 확장 | ~200 | 중 |
| builtins 확장 | ~500 | 중 |
| cli 수정 | ~100 | 하 |
| 테스트 | ~400 | 중 |
| **총계** | **~2,600** | - |

---

## 7. 결론

v6b 구현에서 **약 60%의 기존 코드를 재사용** 가능:
- IR 계층 (value, instruction): 90% 재사용
- VM 계층 (vm, builtins): 75% 재사용
- Frontend (lexer, parser, ast): 전면 재작성

핵심 작업은 **새로운 문법을 위한 Lexer/Parser 구현**이며,
실행 엔진(VM)과 값 표현(IR)은 기존 코드를 최대한 활용.
