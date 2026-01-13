# Vais Grammar Specification v0.2
## Hybrid Model: 연산 기반 + 데이터 흐름 + 제약 충족

---

## 0. 설계 원칙

### 0.1 AI 친화성 원칙

| 원칙 | 설명 | 근거 |
|------|------|------|
| **결정성** | 같은 입력 → 같은 출력 | LLM은 결정적 구조를 더 잘 이해 |
| **명시성** | 모든 것을 명시적으로 선언 | 암묵적 규칙은 AI 오류 유발 |
| **구조 일관성** | 같은 의미 = 같은 구조 | IRCoder 연구 결과 |
| **최소 모호성** | 문법적 모호성 0 | 파싱 비용 최소화 |
| **블록 격리** | 각 블록은 독립적으로 수정 가능 | 부분 수정 비용 최소화 |

### 0.2 하이브리드 구성

```
┌─────────────────────────────────────────────────────────┐
│                    Vais 하이브리드 모델                   │
├─────────────────────────────────────────────────────────┤
│  [Layer 1] 의도 계층 (WHAT)                              │
│    - INTENT: 목표 선언                                   │
│    - CONSTRAINT: 제약 조건                               │
├─────────────────────────────────────────────────────────┤
│  [Layer 2] 연산 계층 (HOW - Abstract)                    │
│    - OP: 사전 정의된 연산 집합                            │
│    - FLOW: 데이터 흐름 그래프                             │
├─────────────────────────────────────────────────────────┤
│  [Layer 3] 실행 계층 (HOW - Concrete)                    │
│    - EXECUTION: 실행 환경 명세                           │
│    - VERIFY: 검증 조건                                   │
└─────────────────────────────────────────────────────────┘
```

---

## 1. 파일 구조 (v0.2)

```ebnf
VaisFile     ::= UNIT META INPUT OUTPUT INTENT CONSTRAINT FLOW EXECUTION VERIFY END
```

### 1.1 블록 순서 (고정)

```
UNIT          # 유닛 식별
META          # 메타데이터
INPUT         # 입력 정의
OUTPUT        # 출력 정의 (신규)
INTENT        # 의도 선언
CONSTRAINT    # 제약 조건
FLOW          # 데이터 흐름 (신규)
EXECUTION     # 실행 명세
VERIFY        # 검증 조건
END           # 종료
```

---

## 2. 타입 시스템

### 2.1 원시 타입 (Primitive Types)

```ebnf
PrimitiveType ::= 'INT' | 'INT8' | 'INT16' | 'INT32' | 'INT64'
               |  'UINT' | 'UINT8' | 'UINT16' | 'UINT32' | 'UINT64'
               |  'FLOAT32' | 'FLOAT64'
               |  'BOOL'
               |  'STRING'
               |  'BYTES'
               |  'VOID'
```

### 2.2 복합 타입 (Composite Types)

```ebnf
CompositeType ::= ArrayType | MapType | StructType | OptionalType | UnionType

ArrayType     ::= 'ARRAY' '<' Type '>'
MapType       ::= 'MAP' '<' Type ',' Type '>'
StructType    ::= 'STRUCT' '{' FieldList '}'
OptionalType  ::= 'OPTIONAL' '<' Type '>'
UnionType     ::= 'UNION' '<' TypeList '>'

FieldList     ::= Field (',' Field)*
Field         ::= Identifier ':' Type
TypeList      ::= Type ('|' Type)*
Type          ::= PrimitiveType | CompositeType | TypeRef
TypeRef       ::= '@' Identifier
```

### 2.3 타입 예시

```vais
# 기본 타입
INT32
STRING
BOOL

# 배열
ARRAY<INT32>
ARRAY<STRING>

# 맵
MAP<STRING, INT32>
MAP<STRING, ARRAY<FLOAT64>>

# 구조체
STRUCT {id: INT64, name: STRING, active: BOOL}

# 옵셔널
OPTIONAL<STRING>

# 유니온
UNION<INT32 | STRING | VOID>
```

---

## 3. 블록 정의

### 3.1 UNIT 블록

```ebnf
UnitBlock    ::= 'UNIT' UnitType VaisUnit Version?
UnitType     ::= 'FUNCTION' | 'SERVICE' | 'PIPELINE' | 'MODULE'
VaisUnit       ::= QualifiedName
Version      ::= 'V' Number '.' Number '.' Number
QualifiedName ::= Identifier ('.' Identifier)*
```

**예시:**
```vais
UNIT FUNCTION auth.user.validate V1.0.0
```

### 3.2 META 블록

```ebnf
MetaBlock    ::= 'META' MetaEntry+ 'ENDMETA'
MetaEntry    ::= MetaKey MetaValue
MetaKey      ::= 'DOMAIN' | 'DETERMINISM' | 'IDEMPOTENT' | 'PURE' | 'TIMEOUT' | 'RETRY'
MetaValue    ::= Literal | Boolean | Duration
Duration     ::= Number TimeUnit
TimeUnit     ::= 'ms' | 's' | 'm' | 'h'
```

**예시:**
```vais
META
  DOMAIN finance.transaction
  DETERMINISM true
  IDEMPOTENT true
  PURE false
  TIMEOUT 30s
  RETRY 3
ENDMETA
```

### 3.3 INPUT 블록

```ebnf
InputBlock   ::= 'INPUT' InputEntry+ 'ENDINPUT'
InputEntry   ::= Identifier ':' Type InputConstraint*
InputConstraint ::= '[' ConstraintExpr ']'
```

**예시:**
```vais
INPUT
  user_id   : INT64        [> 0]
  email     : STRING       [MATCH /^[^@]+@[^@]+$/]
  amount    : FLOAT64      [>= 0.0, <= 1000000.0]
  tags      : ARRAY<STRING> [LEN <= 10]
  metadata  : OPTIONAL<MAP<STRING, STRING>>
ENDINPUT
```

### 3.4 OUTPUT 블록 (신규)

```ebnf
OutputBlock  ::= 'OUTPUT' OutputEntry+ 'ENDOUTPUT'
OutputEntry  ::= Identifier ':' Type OutputConstraint*
OutputConstraint ::= '[' ConstraintExpr ']'
```

**예시:**
```vais
OUTPUT
  success   : BOOL
  result    : OPTIONAL<STRUCT {id: INT64, status: STRING}>
  error     : OPTIONAL<STRUCT {code: INT32, message: STRING}>
ENDOUTPUT
```

### 3.5 INTENT 블록 (재설계)

```ebnf
IntentBlock  ::= 'INTENT' GoalDecl PriorityDecl? FailureDecl? 'ENDINTENT'
GoalDecl     ::= 'GOAL' GoalType ':' GoalSpec
GoalType     ::= 'TRANSFORM' | 'VALIDATE' | 'AGGREGATE' | 'FILTER' | 'ROUTE' | 'COMPOSE'
GoalSpec     ::= InputRef '->' OutputRef
PriorityDecl ::= 'PRIORITY' PriorityList
PriorityList ::= PriorityItem ('>' PriorityItem)*
PriorityItem ::= 'CORRECTNESS' | 'PERFORMANCE' | 'MEMORY' | 'LATENCY' | 'THROUGHPUT'
FailureDecl  ::= 'ON_FAILURE' FailureStrategy
FailureStrategy ::= 'ABORT' | 'RETRY' | 'FALLBACK' FallbackRef | 'DEFAULT' DefaultValue
```

**예시:**
```vais
INTENT
  GOAL TRANSFORM: input.raw_data -> output.processed_data
  PRIORITY CORRECTNESS > LATENCY > MEMORY
  ON_FAILURE FALLBACK @fallback_handler
ENDINTENT
```

### 3.6 CONSTRAINT 블록 (확장)

```ebnf
ConstraintBlock ::= 'CONSTRAINT' ConstraintEntry+ 'ENDCONSTRAINT'
ConstraintEntry ::= ConstraintType ConstraintExpr
ConstraintType  ::= 'REQUIRE' | 'FORBID' | 'PREFER' | 'INVARIANT'
ConstraintExpr  ::= LogicalExpr | TemporalExpr | ResourceExpr

LogicalExpr    ::= Term (LogicalOp Term)*
LogicalOp      ::= 'AND' | 'OR' | 'XOR' | 'IMPLIES'
Term           ::= Comparison | '(' LogicalExpr ')' | 'NOT' Term
Comparison     ::= Operand CompareOp Operand
CompareOp      ::= '==' | '!=' | '<' | '<=' | '>' | '>=' | 'IN' | 'MATCH'

TemporalExpr   ::= 'BEFORE' Ref Ref | 'AFTER' Ref Ref | 'WITHIN' Duration
ResourceExpr   ::= 'MEMORY' CompareOp Size | 'CPU' CompareOp Percentage
```

**예시:**
```vais
CONSTRAINT
  REQUIRE input.amount > 0
  REQUIRE input.user_id != 0
  FORBID output.error AND output.success
  INVARIANT output.success IMPLIES output.result != VOID
  PREFER MEMORY <= 512MB
  REQUIRE WITHIN 100ms
ENDCONSTRAINT
```

### 3.7 FLOW 블록 (신규 - 핵심)

```ebnf
FlowBlock    ::= 'FLOW' NodeDecl+ EdgeDecl+ 'ENDFLOW'

NodeDecl     ::= 'NODE' NodeID ':' OpType OpParams?
NodeID       ::= Identifier
OpType       ::= BuiltinOp | CustomOp
BuiltinOp    ::= 'MAP' | 'FILTER' | 'REDUCE' | 'TRANSFORM' | 'VALIDATE'
               | 'SPLIT' | 'MERGE' | 'BRANCH' | 'JOIN'
               | 'FETCH' | 'STORE' | 'CALL'
CustomOp     ::= '@' QualifiedName
OpParams     ::= '(' ParamList ')'
ParamList    ::= Param (',' Param)*
Param        ::= Identifier '=' Literal | Identifier '=' Ref

EdgeDecl     ::= 'EDGE' EdgeSpec
EdgeSpec     ::= NodeRef '->' NodeRef EdgeCondition?
NodeRef      ::= NodeID | NodeID '.' PortID | 'INPUT' '.' Identifier | 'OUTPUT' '.' Identifier
PortID       ::= Identifier
EdgeCondition ::= 'WHEN' LogicalExpr
```

**예시:**
```vais
FLOW
  # 노드 정의
  NODE validate_input  : VALIDATE (schema=@schemas.user_input)
  NODE fetch_user      : FETCH (source=@db.users, key=input.user_id)
  NODE check_permission: CALL (@auth.check_permission)
  NODE transform_data  : TRANSFORM (mapper=@mappers.user_to_response)
  NODE handle_error    : TRANSFORM (mapper=@mappers.error_response)

  # 엣지 정의 (데이터 흐름)
  EDGE INPUT.user_id     -> validate_input
  EDGE validate_input.ok -> fetch_user
  EDGE validate_input.err -> handle_error
  EDGE fetch_user.data   -> check_permission
  EDGE fetch_user.err    -> handle_error
  EDGE check_permission.ok -> transform_data
  EDGE check_permission.denied -> handle_error
  EDGE transform_data    -> OUTPUT.result
  EDGE handle_error      -> OUTPUT.error
ENDFLOW
```

### 3.8 EXECUTION 블록

```ebnf
ExecutionBlock ::= 'EXECUTION' ExecutionEntry+ 'ENDEXECUTION'
ExecutionEntry ::= ExecutionKey ExecutionValue

ExecutionKey   ::= 'PARALLEL' | 'TARGET' | 'MEMORY' | 'ISOLATION' | 'CACHE'
ExecutionValue ::= Boolean | TargetSpec | MemorySpec | IsolationSpec | CacheSpec

TargetSpec     ::= 'ANY' | 'CPU' | 'GPU' | 'WASM' | 'NATIVE'
MemorySpec     ::= 'BOUNDED' Size | 'UNBOUNDED' | 'STACK_ONLY'
IsolationSpec  ::= 'NONE' | 'THREAD' | 'PROCESS' | 'CONTAINER'
CacheSpec      ::= 'NONE' | 'LRU' Size | 'TTL' Duration
```

**예시:**
```vais
EXECUTION
  PARALLEL true
  TARGET WASM
  MEMORY BOUNDED 256MB
  ISOLATION THREAD
  CACHE LRU 1000
ENDEXECUTION
```

### 3.9 VERIFY 블록

```ebnf
VerifyBlock  ::= 'VERIFY' VerifyEntry+ 'ENDVERIFY'
VerifyEntry  ::= VerifyType VerifyExpr
VerifyType   ::= 'ASSERT' | 'PROPERTY' | 'INVARIANT' | 'POSTCONDITION' | 'TEST'

VerifyExpr   ::= LogicalExpr
              | 'FORALL' Identifier 'IN' Ref ':' LogicalExpr
              | 'EXISTS' Identifier 'IN' Ref ':' LogicalExpr
              | 'EVENTUALLY' LogicalExpr
              | 'ALWAYS' LogicalExpr
```

**예시:**
```vais
VERIFY
  ASSERT output.success OR output.error != VOID
  PROPERTY FORALL item IN output.result.items: item.id > 0
  INVARIANT NOT (output.success AND output.error != VOID)
  POSTCONDITION output.success IMPLIES output.result.status == "completed"
  TEST @tests.unit.validate_user_flow
ENDVERIFY
```

### 3.10 END 블록

```ebnf
EndBlock     ::= 'END'
```

---

## 4. 내장 연산 (Built-in Operations)

### 4.1 데이터 변환 연산

| 연산 | 시그니처 | 설명 |
|------|----------|------|
| `MAP` | `ARRAY<A> -> ARRAY<B>` | 각 요소에 함수 적용 |
| `FILTER` | `ARRAY<A> -> ARRAY<A>` | 조건에 맞는 요소만 유지 |
| `REDUCE` | `ARRAY<A> -> B` | 요소들을 단일 값으로 축약 |
| `TRANSFORM` | `A -> B` | 단일 값 변환 |
| `FLATTEN` | `ARRAY<ARRAY<A>> -> ARRAY<A>` | 중첩 배열 평탄화 |
| `GROUP` | `ARRAY<A> -> MAP<K, ARRAY<A>>` | 키 기준 그룹화 |
| `SORT` | `ARRAY<A> -> ARRAY<A>` | 정렬 |
| `DISTINCT` | `ARRAY<A> -> ARRAY<A>` | 중복 제거 |

### 4.2 흐름 제어 연산

| 연산 | 설명 |
|------|------|
| `BRANCH` | 조건에 따라 분기 (N개 출력 포트) |
| `MERGE` | 여러 흐름을 하나로 병합 |
| `SPLIT` | 하나의 흐름을 여러 개로 분리 |
| `JOIN` | 여러 흐름의 결과를 대기 후 결합 |
| `RACE` | 여러 흐름 중 가장 빠른 결과 선택 |

### 4.3 외부 연산

| 연산 | 설명 |
|------|------|
| `FETCH` | 외부 데이터 소스에서 조회 |
| `STORE` | 외부 데이터 소스에 저장 |
| `CALL` | 외부 유닛 호출 |
| `EMIT` | 이벤트 발행 |
| `SUBSCRIBE` | 이벤트 구독 |

### 4.4 검증 연산

| 연산 | 설명 |
|------|------|
| `VALIDATE` | 스키마 기반 검증 |
| `SANITIZE` | 입력 정제 |
| `AUTHORIZE` | 권한 검증 |

---

## 5. 참조 시스템

### 5.1 참조 문법

```ebnf
Ref          ::= LocalRef | ExternalRef | LiteralRef
LocalRef     ::= 'input.' Identifier | 'output.' Identifier | NodeID '.' PortID
ExternalRef  ::= '@' QualifiedName
LiteralRef   ::= Literal
```

### 5.2 참조 예시

```vais
# 로컬 참조
input.user_id          # 입력 필드
output.result          # 출력 필드
validate_input.ok      # 노드의 출력 포트

# 외부 참조
@schemas.user_input    # 외부 스키마
@db.users              # 외부 데이터 소스
@auth.check_permission # 외부 유닛
@mappers.user_to_response # 외부 매퍼

# 리터럴
"default_value"
42
true
```

---

## 6. 전체 예시

### 6.1 사용자 인증 유닛

```vais
UNIT FUNCTION auth.user.authenticate V1.0.0

META
  DOMAIN auth.security
  DETERMINISM true
  IDEMPOTENT true
  TIMEOUT 5s
  RETRY 0
ENDMETA

INPUT
  username  : STRING    [LEN >= 3, LEN <= 50, MATCH /^[a-zA-Z0-9_]+$/]
  password  : STRING    [LEN >= 8]
  client_ip : STRING    [MATCH /^\d{1,3}(\.\d{1,3}){3}$/]
ENDINPUT

OUTPUT
  authenticated : BOOL
  token         : OPTIONAL<STRING>
  user          : OPTIONAL<STRUCT {id: INT64, role: STRING}>
  error         : OPTIONAL<STRUCT {code: INT32, message: STRING}>
ENDOUTPUT

INTENT
  GOAL VALIDATE: input.credentials -> output.authenticated
  PRIORITY CORRECTNESS > LATENCY > THROUGHPUT
  ON_FAILURE ABORT
ENDINTENT

CONSTRAINT
  REQUIRE input.username != ""
  REQUIRE input.password != ""
  FORBID output.authenticated AND output.error != VOID
  FORBID NOT output.authenticated AND output.token != VOID
  INVARIANT output.authenticated IMPLIES output.user != VOID
  INVARIANT output.authenticated IMPLIES output.token != VOID
  REQUIRE WITHIN 500ms
ENDCONSTRAINT

FLOW
  NODE sanitize_input  : SANITIZE (rules=@rules.auth_input)
  NODE check_rate_limit: CALL (@security.rate_limiter, key=input.client_ip)
  NODE fetch_user      : FETCH (source=@db.users, key=sanitize_input.username)
  NODE verify_password : CALL (@crypto.verify_hash, hash=fetch_user.password_hash, plain=sanitize_input.password)
  NODE generate_token  : CALL (@auth.token_generator, user_id=fetch_user.id)
  NODE build_response  : TRANSFORM (mapper=@mappers.auth_success)
  NODE build_error     : TRANSFORM (mapper=@mappers.auth_error)

  EDGE INPUT.username   -> sanitize_input
  EDGE INPUT.password   -> sanitize_input
  EDGE INPUT.client_ip  -> check_rate_limit

  EDGE check_rate_limit.blocked -> build_error
  EDGE check_rate_limit.ok      -> fetch_user

  EDGE sanitize_input.username  -> fetch_user
  EDGE fetch_user.not_found     -> build_error
  EDGE fetch_user.data          -> verify_password

  EDGE sanitize_input.password  -> verify_password
  EDGE verify_password.invalid  -> build_error
  EDGE verify_password.valid    -> generate_token

  EDGE fetch_user.data          -> generate_token
  EDGE generate_token.token     -> build_response
  EDGE fetch_user.data          -> build_response

  EDGE build_response           -> OUTPUT.authenticated
  EDGE build_response           -> OUTPUT.token
  EDGE build_response           -> OUTPUT.user
  EDGE build_error              -> OUTPUT.error
ENDFLOW

EXECUTION
  PARALLEL false
  TARGET ANY
  MEMORY BOUNDED 64MB
  ISOLATION THREAD
  CACHE NONE
ENDEXECUTION

VERIFY
  ASSERT output.authenticated OR output.error != VOID
  PROPERTY output.token != VOID IMPLIES LEN(output.token) >= 32
  INVARIANT NOT (output.authenticated AND output.error != VOID)
  POSTCONDITION output.authenticated IMPLIES output.user.id > 0
  TEST @tests.auth.authenticate_success
  TEST @tests.auth.authenticate_invalid_password
  TEST @tests.auth.authenticate_rate_limited
ENDVERIFY

END
```

### 6.2 데이터 집계 파이프라인

```vais
UNIT PIPELINE analytics.daily_report V1.2.0

META
  DOMAIN analytics.reporting
  DETERMINISM true
  IDEMPOTENT true
  TIMEOUT 5m
ENDMETA

INPUT
  date_range : STRUCT {start: INT64, end: INT64}  [start <= end]
  filters    : OPTIONAL<STRUCT {category: STRING, region: STRING}>
ENDINPUT

OUTPUT
  report : STRUCT {
    total_revenue: FLOAT64,
    order_count: INT64,
    avg_order_value: FLOAT64,
    by_category: ARRAY<STRUCT {category: STRING, revenue: FLOAT64, count: INT64}>,
    by_region: ARRAY<STRUCT {region: STRING, revenue: FLOAT64, count: INT64}>
  }
  metadata : STRUCT {generated_at: INT64, query_time_ms: INT64}
ENDOUTPUT

INTENT
  GOAL AGGREGATE: input.date_range -> output.report
  PRIORITY CORRECTNESS > PERFORMANCE > MEMORY
  ON_FAILURE ABORT
ENDINTENT

CONSTRAINT
  REQUIRE input.date_range.end - input.date_range.start <= 31 * 24 * 60 * 60
  REQUIRE output.report.total_revenue >= 0
  REQUIRE output.report.order_count >= 0
  INVARIANT output.report.order_count > 0 IMPLIES output.report.avg_order_value > 0
  PREFER MEMORY <= 1GB
ENDCONSTRAINT

FLOW
  NODE fetch_orders   : FETCH (source=@db.orders, range=input.date_range, filters=input.filters)
  NODE calc_totals    : REDUCE (fn=@reducers.sum_revenue)
  NODE group_category : GROUP (key="category")
  NODE group_region   : GROUP (key="region")
  NODE agg_category   : MAP (fn=@mappers.category_aggregate)
  NODE agg_region     : MAP (fn=@mappers.region_aggregate)
  NODE merge_results  : MERGE
  NODE build_report   : TRANSFORM (mapper=@mappers.daily_report)
  NODE add_metadata   : TRANSFORM (mapper=@mappers.add_timestamp)

  EDGE INPUT.date_range -> fetch_orders
  EDGE INPUT.filters    -> fetch_orders

  EDGE fetch_orders     -> calc_totals
  EDGE fetch_orders     -> group_category
  EDGE fetch_orders     -> group_region

  EDGE group_category   -> agg_category
  EDGE group_region     -> agg_region

  EDGE calc_totals      -> merge_results
  EDGE agg_category     -> merge_results
  EDGE agg_region       -> merge_results

  EDGE merge_results    -> build_report
  EDGE build_report     -> add_metadata

  EDGE add_metadata.report   -> OUTPUT.report
  EDGE add_metadata.metadata -> OUTPUT.metadata
ENDFLOW

EXECUTION
  PARALLEL true
  TARGET CPU
  MEMORY BOUNDED 1GB
  ISOLATION PROCESS
  CACHE LRU 100
ENDEXECUTION

VERIFY
  ASSERT output.report.total_revenue >= 0
  ASSERT output.report.order_count >= 0
  PROPERTY FORALL cat IN output.report.by_category: cat.revenue >= 0
  PROPERTY FORALL reg IN output.report.by_region: reg.revenue >= 0
  POSTCONDITION output.metadata.generated_at > input.date_range.end
  TEST @tests.analytics.daily_report_empty
  TEST @tests.analytics.daily_report_single_day
  TEST @tests.analytics.daily_report_full_month
ENDVERIFY

END
```

---

## 7. 컴파일 파이프라인

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Parse     │ -> │  Validate   │ -> │  Optimize   │ -> │  Generate   │
│   (구문)    │    │  (의미)     │    │  (흐름)     │    │  (코드)     │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
      │                  │                  │                  │
      v                  v                  v                  v
   AST 생성          타입 검사          FLOW 최적화      타겟 코드
                    제약 검증          병렬화 분석      (Python/Rust/WASM)
                    참조 해결          캐시 삽입
```

### 7.1 Parse 단계
- 토큰화 및 AST 생성
- 문법 오류 검출
- 블록 순서 검증

### 7.2 Validate 단계
- 타입 검사 (INPUT/OUTPUT 일치)
- 제약 조건 정적 검증
- 외부 참조 해결 및 존재 확인
- FLOW 그래프 유효성 (사이클, 미연결 노드)

### 7.3 Optimize 단계
- 데이터 흐름 분석
- 병렬 실행 가능 노드 식별
- 불필요한 노드 제거 (Dead node elimination)
- 캐시 포인트 삽입

### 7.4 Generate 단계
- 타겟 언어 선택 (EXECUTION.TARGET 기반)
- 코드 템플릿 적용
- 타입 매핑
- 런타임 바인딩 코드 생성

---

## 8. 에러 코드 체계

| 코드 | 범주 | 설명 |
|------|------|------|
| E1xxx | Parse | 구문 오류 |
| E2xxx | Type | 타입 오류 |
| E3xxx | Constraint | 제약 조건 위반 |
| E4xxx | Flow | 흐름 그래프 오류 |
| E5xxx | Reference | 참조 오류 |
| E6xxx | Execution | 실행 설정 오류 |
| E7xxx | Verify | 검증 실패 |

---

## 9. 향후 확장 예정

### 9.1 v0.3 예정
- [ ] 제네릭 타입 (`GENERIC<T>`)
- [ ] 매크로 시스템
- [ ] 모듈 시스템 (IMPORT/EXPORT)

### 9.2 v0.4 예정
- [ ] 스트리밍 타입 (`STREAM<T>`)
- [ ] 상태 머신 블록 (`STATE`)
- [ ] 트랜잭션 블록 (`TRANSACTION`)

### 9.3 v1.0 예정
- [ ] 공식 표준 라이브러리
- [ ] LSP (Language Server Protocol) 지원
- [ ] 다중 타겟 컴파일러

---

## 10. Vais vs 기존 기술 비교

| 측면 | Vais | LLVM IR | Protobuf | SQL |
|------|------|---------|----------|-----|
| **목적** | AI-생성 코드 표현 | 컴파일러 최적화 | 데이터 직렬화 | 데이터 쿼리 |
| **1차 사용자** | AI | 컴파일러 | 개발자 | 개발자 |
| **추상화 수준** | 높음 (의도 포함) | 낮음 | 중간 | 중간 |
| **결정성** | 100% | 100% | 100% | 쿼리 의존 |
| **실행 가능** | Yes (컴파일 후) | Yes | No | Yes (DB 내) |
| **흐름 제어** | 데이터 흐름 그래프 | CFG | N/A | 선언적 |
| **타입 시스템** | 구조적 + 제약 | 저수준 | 구조적 | 스키마 기반 |

---

## 부록 A: EBNF 전체 문법

```ebnf
(* Top Level *)
VaisFile     ::= UnitBlock MetaBlock InputBlock OutputBlock IntentBlock
                 ConstraintBlock FlowBlock ExecutionBlock VerifyBlock EndBlock

(* Literals *)
Identifier   ::= [a-zA-Z_][a-zA-Z0-9_]*
Number       ::= [0-9]+
Float        ::= [0-9]+ '.' [0-9]+
String       ::= '"' [^"]* '"'
Boolean      ::= 'true' | 'false'
Literal      ::= Number | Float | String | Boolean

(* See sections 3.1-3.10 for block definitions *)
```

---

**문서 버전**: 0.2.0
**최종 수정**: 2026-01-11
**상태**: Draft
