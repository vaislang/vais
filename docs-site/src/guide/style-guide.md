# 코딩 스타일 가이드

Vais 커뮤니티가 권장하는 코딩 스타일입니다. 이 가이드를 따르면 코드가 일관되고 읽기 쉬워집니다.

## 네이밍 컨벤션 (Naming Conventions)

### 함수 이름

함수명은 **snake_case**를 사용합니다. 동사로 시작하는 것이 좋습니다.

```vais
# 좋은 예
F calculate_sum(numbers: [i64; 10]) -> i64 { }
F is_prime(n: i64) -> bool { }
F get_user_by_id(id: i64) -> Option<User> { }
F parse_json(text: str) -> Result<Json> { }
F validate_email(email: str) -> bool { }

# 나쁜 예
F CalcSum(numbers: [i64; 10]) -> i64 { }      # PascalCase 사용 금지
F isprime(n: i64) -> bool { }                 # 약자 사용
F getUserById(id: i64) -> Option<User> { }    # camelCase 사용 금지
F PARSE_JSON(text: str) -> Result<Json> { }   # SCREAMING_SNAKE_CASE 함수명 사용 금지
```

### 구조체 이름

구조체명은 **PascalCase**를 사용합니다.

```vais
# 좋은 예
S User {
    id: i64
    name: str
}

S HttpResponse {
    status_code: i64
    body: str
}

S LinkedListNode<T> {
    value: T
    next: i64
}

# 나쁜 예
S user { }                # snake_case 사용 금지
S userProfile { }         # camelCase 사용 금지
S USER_DATA { }           # SCREAMING_SNAKE_CASE 사용 금지
```

### 변수 이름

변수명은 **snake_case**를 사용합니다.

```vais
# 좋은 예
user_count := 0
total_price := 99.99
is_valid := true
temp_buffer := "data"
~ mutable_counter := 0

# 나쁬 예
UserCount := 0              # PascalCase 사용 금지
totalPrice := 99.99         # camelCase 사용 금지
TEMP_BUFFER := "data"       # 변수에 SCREAMING_SNAKE_CASE 사용 금지
x := 0                      # 의미 없는 한 글자 (루프 제외)
```

### 상수 이름

상수명은 **SCREAMING_SNAKE_CASE**를 사용합니다.

```vais
# 좋은 예
C MAX_BUFFER_SIZE: i64 = 1024
C DEFAULT_TIMEOUT: i64 = 30
C PI: f64 = 3.14159
C APP_VERSION: str = "1.0.0"
C ERROR_FILE_NOT_FOUND: i64 = 1

# 나쁜 예
C MaxBufferSize: i64 = 1024   # PascalCase 사용 금지
C max_buffer_size: i64 = 1024 # snake_case 사용 금지
C maxBufferSize: i64 = 1024   # camelCase 사용 금지
```

### Trait 이름

Trait명은 **PascalCase**를 사용합니다. 보통 형용사나 행동을 나타냅니다.

```vais
# 좋은 예
W Drawable {
    F draw(&self) -> i64
}

W Serializable {
    F serialize(&self) -> str
}

W Comparable {
    F compare(&self, other: i64) -> i64
}

W Reader {
    F read(&self) -> i64
}

# 나쁜 예
W drawable { }        # snake_case 사용 금지
W draw_able { }       # snake_case 사용 금지
```

### Enum 이름

Enum명과 변수명 모두 **PascalCase**를 사용합니다.

```vais
# 좋은 예
E Color {
    Red,
    Green,
    Blue
}

E HttpMethod {
    Get,
    Post,
    Put,
    Delete
}

color := Color.Red
method := HttpMethod.Post

# 나쁜 예
E color { }           # snake_case 사용 금지
E COLOR { }           # SCREAMING_SNAKE_CASE 사용 금지
E http_method { }     # snake_case 사용 금지
```

## 파일 및 모듈 구조

### 파일 이름

파일명은 **snake_case**를 사용합니다.

```
# 좋은 구조
src/
├── main.vais              # 메인 엔트리 포인트
├── lib.vais               # 라이브러리 메인
├── user.vais              # 사용자 관련 모듈
├── config.vais            # 설정 관련 모듈
├── database.vais          # 데이터베이스 관련 모듈
└── utils.vais             # 유틸리티 함수

# 나쁬 구조
src/
├── User.vais              # PascalCase 사용 금지
├── Config.vais            # PascalCase 사용 금지
├── get_user_by_id.vais   # 파일명이 너무 상세함
```

### 모듈 크기

파일이 너무 커지지 않도록 주의합니다.

```
권장 파일 크기:
- 300줄 이하: 이상적
- 300-500줄: 괜찮음
- 500줄 이상: 분할 검토
```

### 모듈 구성

관련 기능별로 파일을 구성합니다.

```
src/
├── http/
│   ├── request.vais       # HTTP 요청 타입
│   ├── response.vais      # HTTP 응답 타입
│   └── status.vais        # HTTP 상태 코드
├── database/
│   ├── connection.vais    # DB 연결
│   ├── query.vais         # 쿼리 빌더
│   └── migration.vais     # 마이그레이션
└── middleware/
    ├── auth.vais          # 인증
    └── logging.vais       # 로깅
```

## 주석 및 문서화

### 주석 스타일

```vais
# 한 줄 주석은 #를 사용합니다

# 여러 줄 주석은 여러 개의 #를 사용합니다
# 이것은 복잡한 로직을 설명합니다
# 각 줄마다 #를 붙입니다

F example() {
    # 코드 내 주석
    x := 42  # 인라인 주석은 간단하게
}
```

### API 문서화

공개 함수는 주석으로 문서화합니다.

```vais
# Calculate the sum of two numbers
#
# Arguments:
#   a: first number
#   b: second number
#
# Returns:
#   sum of a and b
#
# Example:
#   result := add(2, 3)  # result = 5
F add(a: i64, b: i64) -> i64 = a + b

# Parse a JSON string
#
# Arguments:
#   text: JSON string to parse
#
# Returns:
#   Ok(JsonValue) if parsing succeeds
#   Err(ParseError) if parsing fails
#
# Note:
#   This function is strict about JSON format
F parse_json(text: str) -> Result<JsonValue> {
    # ...
}
```

### 설명이 필요한 주석

복잡한 로직에만 주석을 작성합니다.

```vais
# 좋은 예: 왜 이렇게 하는지 설명
F find_user_efficiently(id: i64) -> Option<User> {
    # Use binary search on sorted user list
    # Linear search would be O(n), but we have 100k users
    # so O(log n) is critical for performance
    # ...
}

# 나쁜 예: 명확한 코드는 주석 불필요
F add(a: i64, b: i64) -> i64 {
    # Add a to b
    a + b
}
```

## 포매팅 (Formatting)

### 들여쓰기

4개의 스페이스를 사용합니다. 탭은 사용하지 않습니다.

```vais
# 좋은 예
F outer() {
    I true {
        puts("inner")
    }
}

# 나쁜 예 (탭 사용)
F outer() {
→   I true {
→       puts("inner")
→   }
}
```

### 줄 길이

한 줄은 100자를 넘지 않도록 합니다.

```vais
# 좋은 예
result := calculate_complex_value(arg1, arg2, arg3)

# 나쁜 예 (한 줄이 너무 김)
result := calculate_complex_value(arg1, arg2, arg3) + another_function(x, y, z) * some_multiplier

# 개선
result := calculate_complex_value(arg1, arg2, arg3)
extra := another_function(x, y, z) * some_multiplier
result = result + extra
```

### 공백

함수 정의와 구현 사이에 빈 줄을 추가합니다.

```vais
# 좋은 예
F first_function() -> i64 {
    42
}

F second_function() -> str {
    "hello"
}

# 나쁜 예 (빈 줄 없음)
F first_function() -> i64 {
    42
}
F second_function() -> str {
    "hello"
}
```

### 연산자 주변 공백

연산자 주변에 공백을 추가합니다.

```vais
# 좋은 예
x := a + b
result := value * factor + offset
I count > 0 {
    # ...
}

# 나쁜 예
x := a+b
result := value*factor+offset
I count>0 {
    # ...
}
```

## 구조체와 함수 구조

### 구조체 레이아웃

필드는 논리적으로 그룹화합니다.

```vais
# 좋은 예
S User {
    # 식별 정보
    id: i64
    name: str
    email: str

    # 프로필 정보
    age: i64
    bio: str

    # 상태 정보
    is_active: bool
    created_at: i64
    updated_at: i64
}

# 나쁜 예 (순서 무작위)
S User {
    name: str
    created_at: i64
    id: i64
    email: str
    is_active: bool
    age: i64
    updated_at: i64
    bio: str
}
```

### impl 블록 구조

impl 블록의 메서드를 논리적으로 정렬합니다.

```vais
X User {
    # 생성자
    F new(id: i64, name: str) -> User {
        User { id: id, name: name, email: "", age: 0, bio: "", is_active: true, created_at: 0, updated_at: 0 }
    }

    # 기본 접근자
    F get_id(&self) -> i64 = self.id
    F get_name(&self) -> str = self.name

    # 수정자
    F set_email(&self, email: str) {
        # ...
    }

    # 비즈니스 로직
    F is_valid(&self) -> bool {
        self.id > 0 && self.name.len() > 0
    }

    # 직렬화
    F to_string(&self) -> str {
        # ...
    }
}
```

## 에러 처리

에러 처리는 일관되게 합니다.

```vais
# 좋은 예: 명확한 에러 타입
E DatabaseError {
    ConnectionFailed,
    QueryFailed,
    TimeoutError
}

F query_user(id: i64) -> Result<User> {
    I id <= 0 {
        R Result.Err(DatabaseError.QueryFailed)
    }
    # ...
}

# 나쁜 예: 문자열로 에러 표현
F query_user_bad(id: i64) -> Result<User> {
    I id <= 0 {
        R Result.Err("bad id")
    }
    # ...
}
```

## 타입 시스템

명시적 타입 주석을 사용합니다.

```vais
# 좋은 예: 명확한 타입
F process_data(items: [i64; 10]) -> i64 {
    ~ sum: i64 = 0
    # ...
}

# 나쁜 예: 타입 생략
F process_data(items) {
    ~ sum := 0  # 타입 불명확
    # ...
}
```

## 성능 관련 스타일

### 불필요한 복사 피하기

```vais
# 좋은 예: 참조 사용
F process_large_data(data: &[i64; 1000]) {
    # data 사용
}

# 나쁜 예: 복사 발생
F process_large_data_bad(data: [i64; 1000]) {
    # 전체 배열이 복사됨
}
```

### 사전 할당 (Pre-allocation)

```vais
# 좋은 예: 사전 할당
F build_results() {
    ~ results: [i64; 1000]  # 미리 할당

    ~ i := 0
    L i < 1000 {
        results[i] = process(i)
        i = i + 1
    }
}

# 나쁜 예: 반복 할당
F build_results_bad() {
    ~ results: [i64]  # 동적 배열 (매번 재할당)
    # ...
}
```

## 테스트 작성

테스트 함수는 명확한 이름을 사용합니다.

```vais
# 테스트 함수는 test_로 시작
F test_add_positive_numbers() {
    result := add(2, 3)
    I result != 5 {
        puts("FAIL: test_add_positive_numbers")
        R
    }
    puts("PASS: test_add_positive_numbers")
}

F test_add_negative_numbers() {
    result := add(-2, -3)
    I result != -5 {
        puts("FAIL: test_add_negative_numbers")
        R
    }
    puts("PASS: test_add_negative_numbers")
}

F test_add_zero() {
    result := add(0, 0)
    I result != 0 {
        puts("FAIL: test_add_zero")
        R
    }
    puts("PASS: test_add_zero")
}
```

## 스타일 체크리스트

```
명명:
□ 함수: snake_case
□ 구조체: PascalCase
□ 변수: snake_case
□ 상수: SCREAMING_SNAKE_CASE
□ 파일: snake_case

포매팅:
□ 들여쓰기: 4 스페이스
□ 줄 길이: 100자 이내
□ 함수 사이: 빈 줄 추가
□ 연산자 주변: 공백 추가

문서화:
□ 공개 함수: 문서화 주석
□ 복잡한 로직: 설명 주석
□ 의미 없는 주석: 제거

구조:
□ 파일 크기: 500줄 이하
□ 논리적 그룹화
□ impl 메서드: 순서대로 정렬

테스트:
□ 중요한 함수: 테스트 작성
□ 테스트 이름: test_* 형식
□ 엣지 케이스: 테스트 포함
```

## 다음 단계

- [Getting Started 가이드](./getting-started.md): 기본 문법
- [에러 처리 가이드](./error-handling.md): 에러 처리 스타일
- [성능 튜닝 가이드](./performance.md): 성능 최적화
