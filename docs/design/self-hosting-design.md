# Vais Self-Hosting Design Document

> **버전**: 0.1.0
> **작성일**: 2026-01-21
> **목표**: vaisc 컴파일러를 Vais 언어로 재작성하여 self-hosting 달성

---

## 1. 개요

### 1.1 Self-hosting이란?

Self-hosting 컴파일러는 자기 자신의 소스 코드를 컴파일할 수 있는 컴파일러입니다. 이를 통해:
- 언어의 성숙도 검증
- 컴파일러 개발에 해당 언어 자체 사용 가능
- 부트스트래핑 가능 (최소한의 외부 의존성)

### 1.2 현재 상태

```
현재: Rust로 작성된 vaisc
목표: Vais로 작성된 vaisc-vais

부트스트래핑 체인:
  1. Rust vaisc (호스트 컴파일러)
  2. vaisc-vais (Vais로 작성, Rust vaisc로 컴파일)
  3. vaisc-vais2 (vaisc-vais로 자기 자신 컴파일)
```

### 1.3 구현 범위

| 컴포넌트 | 현재 (Rust) | 대상 (Vais) | LOC |
|----------|-------------|-------------|-----|
| Lexer | vais-lexer | self/lexer.vais | ~500 |
| AST | vais-ast | self/ast.vais | ~800 |
| Parser | vais-parser | self/parser.vais | ~2000 |
| Type Checker | vais-types | self/types.vais | ~2500 |
| Code Generator | vais-codegen | self/codegen.vais | ~3500 |
| **총계** | | | ~9300 |

---

## 2. 아키텍처

### 2.1 디렉토리 구조

```
self/                      # Self-hosted 컴파일러
├── lexer.vais            # 토크나이저
├── token.vais            # 토큰 정의
├── ast.vais              # AST 노드 정의
├── span.vais             # 소스 위치 추적
├── parser.vais           # 재귀 하강 파서
├── parser_expr.vais      # 표현식 파싱
├── parser_stmt.vais      # 문장 파싱
├── parser_type.vais      # 타입 파싱
├── types.vais            # 타입 정의
├── type_checker.vais     # 타입 검사
├── inference.vais        # 타입 추론
├── traits.vais           # 트레이트 검사
├── codegen.vais          # LLVM IR 생성
├── codegen_expr.vais     # 표현식 코드 생성
├── codegen_stmt.vais     # 문장 코드 생성
├── codegen_type.vais     # 타입 코드 생성
├── error.vais            # 에러 타입
└── main.vais             # CLI 진입점
```

### 2.2 의존성 흐름

```
┌─────────────────────────────────────────────────────────┐
│                    main.vais (CLI)                      │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                  lexer.vais + token.vais                │
│  tokenize(source: str) -> Result<Vec<Token>, LexError>  │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                      parser.vais                        │
│      parse(tokens: Vec<Token>) -> Result<Module, ...>   │
│  ├── parser_expr.vais (표현식)                          │
│  ├── parser_stmt.vais (문장)                            │
│  └── parser_type.vais (타입)                            │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   type_checker.vais                     │
│     check(module: Module) -> Result<TypedModule, ...>   │
│  ├── inference.vais (타입 추론)                         │
│  └── traits.vais (트레이트 검사)                        │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                     codegen.vais                        │
│     generate(module: TypedModule) -> String (LLVM IR)   │
│  ├── codegen_expr.vais (표현식 → IR)                    │
│  ├── codegen_stmt.vais (문장 → IR)                      │
│  └── codegen_type.vais (타입 → IR)                      │
└─────────────────────────────────────────────────────────┘
```

---

## 3. 단계별 구현 계획

### Phase 1: 기반 인프라 (1주)

#### 3.1.1 Span 및 에러 타입

```vais
# self/span.vais
S Span {
    start: u64,
    end: u64
}

S Spanned<T> {
    node: T,
    span: Span
}

# self/error.vais
E LexError {
    InvalidToken(u64),
    UnterminatedString(u64),
    InvalidEscape(u64)
}

E ParseError {
    UnexpectedToken { found: str, expected: str, span: Span },
    UnexpectedEof { span: Span },
    InvalidPattern { span: Span }
}

E TypeError {
    TypeMismatch { expected: str, found: str, span: Span },
    UndefinedVariable { name: str, span: Span },
    UndefinedFunction { name: str, span: Span }
}
```

#### 3.1.2 Token 정의

```vais
# self/token.vais
E Token {
    # 키워드
    KwF,           # function
    KwS,           # struct
    KwE,           # enum
    KwI,           # if
    KwL,           # loop
    KwM,           # match
    KwW,           # trait (W for "with")
    KwX,           # impl (X for "extend")
    KwT,           # type
    KwU,           # use
    KwP,           # pub
    KwA,           # async
    KwR,           # return
    KwB,           # break
    KwC,           # continue
    KwTrue,
    KwFalse,
    KwMut,
    KwLet,
    KwElse,        # E 키워드와 구분 필요

    # 타입 키워드
    TyI8, TyI16, TyI32, TyI64, TyI128,
    TyU8, TyU16, TyU32, TyU64, TyU128,
    TyF32, TyF64,
    TyBool, TyStr,

    # 리터럴
    Int(i64),
    Float(f64),
    String(str),
    Ident(str),

    # 연산자
    Plus, Minus, Star, Slash, Percent,
    Lt, Gt, LtEq, GtEq, EqEq, NotEq,
    Amp, Pipe, Caret, Tilde, Bang,
    Shl, Shr,
    And, Or,

    # 할당
    Eq, ColonEq,
    PlusEq, MinusEq, StarEq, SlashEq,

    # 구분자
    LParen, RParen,
    LBrace, RBrace,
    LBracket, RBracket,
    Comma, Colon, Semi, Dot, DotDot, DotDotEq,
    Arrow, FatArrow, ColonColon,
    Question, At, Hash,

    # 특수
    Eof
}

S SpannedToken {
    token: Token,
    span: Span
}
```

### Phase 2: Lexer 구현 (1주)

#### 3.2.1 Lexer 구조

```vais
# self/lexer.vais
U self.token
U self.span
U self.error
U std.vec
U std.string
U std.option

S Lexer {
    source: str,
    pos: u64,
    line: u64,
    col: u64
}

X Lexer {
    F new(source: str) -> Lexer = Lexer {
        source: source,
        pos: 0,
        line: 1,
        col: 1
    }

    F tokenize(self: &mut Lexer) -> Result<Vec<SpannedToken>, LexError> {
        let mut tokens: Vec<SpannedToken> = Vec.new()

        L {
            self.skip_whitespace_and_comments()

            I self.is_eof() {
                tokens.push(SpannedToken {
                    token: Token.Eof,
                    span: self.current_span()
                })
                B
            }

            let token = self.next_token()?
            tokens.push(token)
        }

        Ok(tokens)
    }

    F next_token(self: &mut Lexer) -> Result<SpannedToken, LexError> {
        let start = self.pos
        let c = self.peek()

        # 식별자 또는 키워드
        I self.is_ident_start(c) {
            R self.scan_ident_or_keyword()
        }

        # 숫자 리터럴
        I self.is_digit(c) {
            R self.scan_number()
        }

        # 문자열 리터럴
        I c == '"' {
            R self.scan_string()
        }

        # 연산자 및 구분자
        R self.scan_operator()
    }

    F scan_ident_or_keyword(self: &mut Lexer) -> Result<SpannedToken, LexError> {
        let start = self.pos
        let mut ident = String.new()

        L {
            let c = self.peek()
            I !self.is_ident_char(c) { B }
            ident.push_char(c)
            self.advance()
        }

        let token = M ident.as_str() {
            "F" => Token.KwF,
            "S" => Token.KwS,
            "E" => Token.KwE,
            "I" => Token.KwI,
            "L" => Token.KwL,
            "M" => Token.KwM,
            "W" => Token.KwW,
            "X" => Token.KwX,
            "T" => Token.KwT,
            "U" => Token.KwU,
            "P" => Token.KwP,
            "A" => Token.KwA,
            "R" => Token.KwR,
            "B" => Token.KwB,
            "C" => Token.KwC,
            "true" => Token.KwTrue,
            "false" => Token.KwFalse,
            "mut" => Token.KwMut,
            "let" => Token.KwLet,
            "else" => Token.KwElse,
            "i8" => Token.TyI8,
            "i16" => Token.TyI16,
            "i32" => Token.TyI32,
            "i64" => Token.TyI64,
            "i128" => Token.TyI128,
            "u8" => Token.TyU8,
            "u16" => Token.TyU16,
            "u32" => Token.TyU32,
            "u64" => Token.TyU64,
            "u128" => Token.TyU128,
            "f32" => Token.TyF32,
            "f64" => Token.TyF64,
            "bool" => Token.TyBool,
            "str" => Token.TyStr,
            _ => Token.Ident(ident)
        }

        Ok(SpannedToken {
            token: token,
            span: Span { start: start, end: self.pos }
        })
    }

    # 헬퍼 메서드들
    F peek(self: &Lexer) -> i8 =
        I self.pos < self.source.len() {
            self.source.char_at(self.pos)
        } E { 0 }

    F advance(self: &mut Lexer) {
        I self.pos < self.source.len() {
            I self.source.char_at(self.pos) == '\n' {
                self.line = self.line + 1
                self.col = 1
            } E {
                self.col = self.col + 1
            }
            self.pos = self.pos + 1
        }
    }

    F is_eof(self: &Lexer) -> bool = self.pos >= self.source.len()

    F is_ident_start(c: i8) -> bool =
        (c >= 'a' & c <= 'z') | (c >= 'A' & c <= 'Z') | c == '_'

    F is_ident_char(c: i8) -> bool =
        Self.is_ident_start(c) | (c >= '0' & c <= '9')

    F is_digit(c: i8) -> bool = c >= '0' & c <= '9'
}
```

### Phase 3: AST 정의 (1주)

#### 3.3.1 AST 노드 타입

```vais
# self/ast.vais
U self.span
U self.token
U std.vec
U std.option

# ===== 최상위 구조 =====

S Module {
    items: Vec<Spanned<Item>>
}

E Item {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    TypeAlias(TypeAlias),
    Use(UseDecl),
    Trait(Trait),
    Impl(Impl)
}

# ===== 함수 정의 =====

S Function {
    name: Spanned<str>,
    generics: Vec<GenericParam>,
    params: Vec<Param>,
    ret_type: Option<Spanned<Type>>,
    body: FunctionBody,
    is_pub: bool,
    is_async: bool,
    attributes: Vec<Attribute>
}

E FunctionBody {
    Expr(Box<Spanned<Expr>>),
    Block(Vec<Spanned<Stmt>>)
}

S Param {
    name: Spanned<str>,
    ty: Spanned<Type>,
    is_mut: bool
}

S GenericParam {
    name: Spanned<str>,
    bounds: Vec<Spanned<str>>
}

# ===== 타입 정의 =====

S Struct {
    name: Spanned<str>,
    generics: Vec<GenericParam>,
    fields: Vec<Field>,
    methods: Vec<Spanned<Function>>,
    is_pub: bool
}

S Field {
    name: Spanned<str>,
    ty: Spanned<Type>,
    is_pub: bool
}

S Enum {
    name: Spanned<str>,
    generics: Vec<GenericParam>,
    variants: Vec<Variant>,
    methods: Vec<Spanned<Function>>,
    is_pub: bool
}

S Variant {
    name: Spanned<str>,
    fields: VariantFields
}

E VariantFields {
    Unit,
    Tuple(Vec<Spanned<Type>>),
    Struct(Vec<Field>)
}

# ===== 트레이트 & Impl =====

S Trait {
    name: Spanned<str>,
    generics: Vec<GenericParam>,
    super_traits: Vec<Spanned<str>>,
    methods: Vec<TraitMethod>,
    assoc_types: Vec<AssocType>,
    is_pub: bool
}

S TraitMethod {
    name: Spanned<str>,
    generics: Vec<GenericParam>,
    params: Vec<Param>,
    ret_type: Option<Spanned<Type>>,
    default_body: Option<FunctionBody>,
    is_async: bool
}

S AssocType {
    name: Spanned<str>,
    bounds: Vec<Spanned<str>>
}

S Impl {
    generics: Vec<GenericParam>,
    trait_ref: Option<Spanned<str>>,
    target_type: Spanned<Type>,
    methods: Vec<Spanned<Function>>,
    assoc_types: Vec<(Spanned<str>, Spanned<Type>)>
}

# ===== 타입 표현 =====

E Type {
    Named(str),
    Array(Box<Type>, Option<u64>),
    Map(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Optional(Box<Type>),
    Result(Box<Type>, Option<Box<Type>>),
    Pointer(Box<Type>),
    Ref(Box<Type>),
    RefMut(Box<Type>),
    Fn(Vec<Type>, Box<Type>),
    Generic(str, Vec<Type>),
    Unit,
    Infer
}

# ===== 표현식 =====

E Expr {
    # 리터럴
    Int(i64),
    Float(f64),
    Bool(bool),
    String(str),
    Unit,

    # 변수 및 경로
    Ident(str),
    Path(Vec<str>),

    # 연산자
    Binary { op: BinOp, left: Box<Spanned<Expr>>, right: Box<Spanned<Expr>> },
    Unary { op: UnaryOp, expr: Box<Spanned<Expr>> },

    # 제어 흐름
    If { cond: Box<Spanned<Expr>>, then_branch: Box<Spanned<Expr>>, else_branch: Option<Box<Spanned<Expr>>> },
    Loop { pattern: Option<Pattern>, iter: Option<Box<Spanned<Expr>>>, body: Vec<Spanned<Stmt>> },
    Match { expr: Box<Spanned<Expr>>, arms: Vec<MatchArm> },

    # 호출
    Call { func: Box<Spanned<Expr>>, args: Vec<Spanned<Expr>> },
    MethodCall { receiver: Box<Spanned<Expr>>, method: str, args: Vec<Spanned<Expr>> },
    SelfCall(Vec<Spanned<Expr>>),  # @ 연산자

    # 접근
    Field { expr: Box<Spanned<Expr>>, field: str },
    Index { expr: Box<Spanned<Expr>>, index: Box<Spanned<Expr>> },

    # 생성
    StructLit { name: str, fields: Vec<(str, Spanned<Expr>)> },
    ArrayLit(Vec<Spanned<Expr>>),
    TupleLit(Vec<Spanned<Expr>>),

    # 람다
    Lambda { params: Vec<LambdaParam>, body: Box<Spanned<Expr>> },

    # 비동기
    Await(Box<Spanned<Expr>>),
    Spawn(Box<Spanned<Expr>>),

    # 기타
    Ternary { cond: Box<Spanned<Expr>>, then_val: Box<Spanned<Expr>>, else_val: Box<Spanned<Expr>> },
    Range { start: Option<Box<Spanned<Expr>>>, end: Option<Box<Spanned<Expr>>>, inclusive: bool },
    Cast { expr: Box<Spanned<Expr>>, ty: Type },
    Block(Vec<Spanned<Stmt>>)
}

S MatchArm {
    pattern: Pattern,
    guard: Option<Box<Spanned<Expr>>>,
    body: Box<Spanned<Expr>>
}

S LambdaParam {
    name: str,
    ty: Option<Type>
}

# ===== 문장 =====

E Stmt {
    Let { name: str, ty: Option<Type>, value: Option<Box<Spanned<Expr>>>, is_mut: bool },
    Expr(Box<Spanned<Expr>>),
    Return(Option<Box<Spanned<Expr>>>),
    Break(Option<Box<Spanned<Expr>>>),
    Continue
}

# ===== 패턴 =====

E Pattern {
    Wildcard,
    Ident { name: str, is_mut: bool },
    Literal(Literal),
    Tuple(Vec<Pattern>),
    Struct { name: str, fields: Vec<(str, Pattern)>, rest: bool },
    Variant { enum_name: Option<str>, variant: str, fields: VariantPattern },
    Range { start: Option<Literal>, end: Option<Literal>, inclusive: bool },
    Or(Vec<Pattern>)
}

E VariantPattern {
    Unit,
    Tuple(Vec<Pattern>),
    Struct(Vec<(str, Pattern)>)
}

E Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(str)
}

# ===== 연산자 =====

E BinOp {
    Add, Sub, Mul, Div, Mod,
    Lt, Lte, Gt, Gte, Eq, Neq,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
    Assign, AddAssign, SubAssign, MulAssign, DivAssign
}

E UnaryOp {
    Neg, Not, BitNot, Deref, Ref, RefMut
}

# ===== 기타 =====

S TypeAlias {
    name: Spanned<str>,
    generics: Vec<GenericParam>,
    ty: Spanned<Type>,
    is_pub: bool
}

S UseDecl {
    path: Vec<str>,
    alias: Option<str>
}

S Attribute {
    name: str,
    args: Vec<str>
}
```

### Phase 4: Parser 구현 (2주)

#### 3.4.1 Parser 구조

```vais
# self/parser.vais
U self.lexer
U self.token
U self.ast
U self.span
U self.error
U std.vec
U std.option

S Parser {
    tokens: Vec<SpannedToken>,
    pos: u64
}

X Parser {
    F new(tokens: Vec<SpannedToken>) -> Parser = Parser {
        tokens: tokens,
        pos: 0
    }

    F parse_module(self: &mut Parser) -> Result<Module, ParseError> {
        let mut items: Vec<Spanned<Item>> = Vec.new()

        L {
            I self.is_eof() { B }

            let item = self.parse_item()?
            items.push(item)
        }

        Ok(Module { items: items })
    }

    F parse_item(self: &mut Parser) -> Result<Spanned<Item>, ParseError> {
        let start = self.current_span().start

        # 속성 파싱
        let attrs = self.parse_attributes()?

        # 가시성
        let is_pub = self.eat(Token.KwP)

        # 비동기
        let is_async = self.eat(Token.KwA)

        let item = M self.peek_token() {
            Token.KwF => Item.Function(self.parse_function(is_pub, is_async, attrs)?),
            Token.KwS => Item.Struct(self.parse_struct(is_pub)?),
            Token.KwE => Item.Enum(self.parse_enum(is_pub)?),
            Token.KwT => Item.TypeAlias(self.parse_type_alias(is_pub)?),
            Token.KwU => Item.Use(self.parse_use()?),
            Token.KwW => Item.Trait(self.parse_trait(is_pub)?),
            Token.KwX => Item.Impl(self.parse_impl()?),
            _ => R Err(ParseError.UnexpectedToken {
                found: self.current_token_str(),
                expected: "item keyword",
                span: self.current_span()
            })
        }

        let end = self.prev_span().end
        Ok(Spanned { node: item, span: Span { start: start, end: end } })
    }

    F parse_function(self: &mut Parser, is_pub: bool, is_async: bool, attrs: Vec<Attribute>)
        -> Result<Function, ParseError> {
        self.expect(Token.KwF)?

        let name = self.parse_ident()?
        let generics = self.parse_generics()?

        self.expect(Token.LParen)?
        let params = self.parse_params()?
        self.expect(Token.RParen)?

        let ret_type = I self.eat(Token.Arrow) {
            Some(self.parse_type()?)
        } E { None }

        let body = I self.eat(Token.Eq) {
            FunctionBody.Expr(Box.new(self.parse_expr()?))
        } E {
            self.expect(Token.LBrace)?
            let stmts = self.parse_block_stmts()?
            self.expect(Token.RBrace)?
            FunctionBody.Block(stmts)
        }

        Ok(Function {
            name: name,
            generics: generics,
            params: params,
            ret_type: ret_type,
            body: body,
            is_pub: is_pub,
            is_async: is_async,
            attributes: attrs
        })
    }

    # ... 추가 파싱 메서드들
}
```

### Phase 5: Type Checker 구현 (2주)

#### 3.5.1 Type Checker 구조

```vais
# self/type_checker.vais
U self.ast
U self.types
U self.error
U std.hashmap
U std.vec
U std.option

S TypeChecker {
    # 정의 저장소
    structs: HashMap<str, StructDef>,
    enums: HashMap<str, EnumDef>,
    functions: HashMap<str, FunctionSig>,
    traits: HashMap<str, TraitDef>,

    # 제네릭
    generics: Vec<str>,
    generic_bounds: HashMap<str, Vec<str>>,
    substitutions: HashMap<str, ResolvedType>,

    # 트레이트 구현
    trait_impls: Vec<TraitImpl>,

    # 현재 컨텍스트
    scopes: Vec<HashMap<str, ResolvedType>>,
    current_fn_ret: Option<ResolvedType>,
    current_fn_name: Option<str>,
    in_loop: bool,

    # 타입 변수 카운터
    type_var_counter: u64
}

X TypeChecker {
    F new() -> TypeChecker = TypeChecker {
        structs: HashMap.new(),
        enums: HashMap.new(),
        functions: HashMap.new(),
        traits: HashMap.new(),
        generics: Vec.new(),
        generic_bounds: HashMap.new(),
        substitutions: HashMap.new(),
        trait_impls: Vec.new(),
        scopes: Vec.new(),
        current_fn_ret: None,
        current_fn_name: None,
        in_loop: false,
        type_var_counter: 0
    }

    F check_module(self: &mut TypeChecker, module: &Module) -> Result<(), TypeError> {
        # Pass 1: 모든 정의 수집
        L item : module.items {
            M item.node {
                Item.Struct(s) => self.register_struct(s),
                Item.Enum(e) => self.register_enum(e),
                Item.Function(f) => self.register_function(f),
                Item.Trait(t) => self.register_trait(t),
                Item.Impl(i) => self.register_impl(i),
                _ => ()
            }
        }

        # Pass 2: 타입 검사
        L item : module.items {
            M item.node {
                Item.Function(f) => self.check_function(f)?,
                Item.Struct(s) => self.check_struct_methods(s)?,
                Item.Enum(e) => self.check_enum_methods(e)?,
                Item.Impl(i) => self.check_impl(i)?,
                _ => ()
            }
        }

        Ok(())
    }

    F check_expr(self: &mut TypeChecker, expr: &Spanned<Expr>) -> Result<ResolvedType, TypeError> {
        M expr.node {
            Expr.Int(_) => Ok(ResolvedType.I64),
            Expr.Float(_) => Ok(ResolvedType.F64),
            Expr.Bool(_) => Ok(ResolvedType.Bool),
            Expr.String(_) => Ok(ResolvedType.Str),

            Expr.Ident(name) => self.lookup_variable(name, expr.span),

            Expr.Binary { op, left, right } => {
                let left_ty = self.check_expr(left)?
                let right_ty = self.check_expr(right)?
                self.check_binary_op(op, left_ty, right_ty, expr.span)
            },

            Expr.Call { func, args } => self.check_call(func, args, expr.span),

            Expr.If { cond, then_branch, else_branch } =>
                self.check_if(cond, then_branch, else_branch, expr.span),

            # ... 나머지 표현식 타입 검사

            _ => Err(TypeError.UnsupportedExpression { span: expr.span })
        }
    }

    # ... 추가 검사 메서드들
}
```

### Phase 6: Code Generator 구현 (2주)

#### 3.6.1 Code Generator 구조

```vais
# self/codegen.vais
U self.ast
U self.types
U std.string
U std.hashmap
U std.vec

S CodeGenerator {
    output: String,
    indent: u64,

    # 심볼 테이블
    local_vars: HashMap<str, str>,  # name -> LLVM 레지스터
    global_strings: Vec<(str, str)>,  # (name, value)

    # 카운터
    reg_counter: u64,
    label_counter: u64,
    string_counter: u64,

    # 타입 정보
    struct_layouts: HashMap<str, StructLayout>,

    # 현재 컨텍스트
    current_fn: Option<str>
}

S StructLayout {
    fields: Vec<(str, str)>,  # (field_name, llvm_type)
    size: u64
}

X CodeGenerator {
    F new() -> CodeGenerator = CodeGenerator {
        output: String.new(),
        indent: 0,
        local_vars: HashMap.new(),
        global_strings: Vec.new(),
        reg_counter: 0,
        label_counter: 0,
        string_counter: 0,
        struct_layouts: HashMap.new(),
        current_fn: None
    }

    F generate(self: &mut CodeGenerator, module: &Module) -> String {
        self.emit_header()
        self.emit_builtins()

        # 구조체 타입 정의
        L item : module.items {
            M item.node {
                Item.Struct(s) => self.emit_struct_type(s),
                Item.Enum(e) => self.emit_enum_type(e),
                _ => ()
            }
        }

        # 함수 선언
        L item : module.items {
            M item.node {
                Item.Function(f) => self.emit_function_decl(f),
                _ => ()
            }
        }

        # 함수 정의
        L item : module.items {
            M item.node {
                Item.Function(f) => self.emit_function(f),
                Item.Impl(i) => self.emit_impl(i),
                _ => ()
            }
        }

        self.emit_global_strings()

        self.output.clone()
    }

    F emit_function(self: &mut CodeGenerator, f: &Function) {
        self.current_fn = Some(f.name.node.clone())
        self.local_vars.clear()
        self.reg_counter = 0

        let ret_ty = self.type_to_llvm(f.ret_type)
        let params_str = self.format_params(f.params)

        self.emit_line(&format!("define {} @{}({}) {{", ret_ty, f.name.node, params_str))
        self.emit_line("entry:")
        self.indent = self.indent + 1

        # 파라미터를 로컬 변수로 할당
        L param : f.params {
            let ty = self.type_to_llvm(Some(param.ty))
            let alloca = self.fresh_reg()
            self.emit_line(&format!("  {} = alloca {}", alloca, ty))
            self.emit_line(&format!("  store {} %{}, {}* {}", ty, param.name.node, ty, alloca))
            self.local_vars.insert(param.name.node.clone(), alloca)
        }

        # 본문 생성
        M f.body {
            FunctionBody.Expr(e) => {
                let result = self.emit_expr(e)
                self.emit_line(&format!("  ret {} {}", ret_ty, result))
            },
            FunctionBody.Block(stmts) => {
                L stmt : stmts {
                    self.emit_stmt(stmt)
                }
            }
        }

        self.indent = self.indent - 1
        self.emit_line("}")
        self.emit_line("")

        self.current_fn = None
    }

    F emit_expr(self: &mut CodeGenerator, expr: &Spanned<Expr>) -> str {
        M expr.node {
            Expr.Int(n) => format!("{}", n),

            Expr.Binary { op, left, right } => {
                let left_val = self.emit_expr(left)
                let right_val = self.emit_expr(right)
                let result = self.fresh_reg()
                let op_str = self.binop_to_llvm(op)
                self.emit_line(&format!("  {} = {} i64 {}, {}", result, op_str, left_val, right_val))
                result
            },

            Expr.Ident(name) => {
                let ptr = self.local_vars.get(name).unwrap()
                let result = self.fresh_reg()
                self.emit_line(&format!("  {} = load i64, i64* {}", result, ptr))
                result
            },

            Expr.Call { func, args } => self.emit_call(func, args),

            Expr.If { cond, then_branch, else_branch } =>
                self.emit_if(cond, then_branch, else_branch),

            # ... 나머지 표현식 코드 생성

            _ => "0"  # TODO
        }
    }

    # 헬퍼 메서드들
    F fresh_reg(self: &mut CodeGenerator) -> str {
        let n = self.reg_counter
        self.reg_counter = self.reg_counter + 1
        format!("%{}", n)
    }

    F fresh_label(self: &mut CodeGenerator) -> str {
        let n = self.label_counter
        self.label_counter = self.label_counter + 1
        format!("label{}", n)
    }

    F emit_line(self: &mut CodeGenerator, line: &str) {
        self.output.push_str(line)
        self.output.push_char('\n')
    }

    F type_to_llvm(self: &CodeGenerator, ty: Option<&Spanned<Type>>) -> str {
        M ty {
            None => "void",
            Some(t) => M t.node {
                Type.Named("i8") | Type.Named("u8") => "i8",
                Type.Named("i16") | Type.Named("u16") => "i16",
                Type.Named("i32") | Type.Named("u32") => "i32",
                Type.Named("i64") | Type.Named("u64") => "i64",
                Type.Named("f32") => "float",
                Type.Named("f64") => "double",
                Type.Named("bool") => "i1",
                Type.Named("str") => "i8*",
                Type.Pointer(inner) => format!("{}*", self.type_to_llvm(Some(inner))),
                _ => "i64"  # 기본값
            }
        }
    }

    F binop_to_llvm(op: &BinOp) -> str = M op {
        BinOp.Add => "add",
        BinOp.Sub => "sub",
        BinOp.Mul => "mul",
        BinOp.Div => "sdiv",
        BinOp.Mod => "srem",
        BinOp.Lt => "icmp slt",
        BinOp.Lte => "icmp sle",
        BinOp.Gt => "icmp sgt",
        BinOp.Gte => "icmp sge",
        BinOp.Eq => "icmp eq",
        BinOp.Neq => "icmp ne",
        BinOp.BitAnd => "and",
        BinOp.BitOr => "or",
        BinOp.BitXor => "xor",
        BinOp.Shl => "shl",
        BinOp.Shr => "ashr",
        _ => "add"  # 기본값
    }
}
```

---

## 4. 부트스트래핑 전략

### 4.1 단계별 부트스트래핑

```
Stage 0: Rust vaisc (호스트 컴파일러)
    │
    │ 컴파일
    ▼
Stage 1: self/*.vais → vaisc-stage1 (첫 번째 Vais 컴파일러)
    │
    │ 자기 컴파일
    ▼
Stage 2: self/*.vais → vaisc-stage2 (Stage 1으로 컴파일)
    │
    │ 검증
    ▼
Stage 3: diff vaisc-stage1 vaisc-stage2 (바이너리 동일성 확인)
```

### 4.2 점진적 구현 전략

#### 단계 1: Minimal Vais (MVP)

최소한의 언어 기능만 지원:
- 기본 타입 (i64, bool, str)
- 함수 정의 및 호출
- 변수 (let)
- 기본 연산자
- if/else
- 단순 루프

#### 단계 2: Extended Vais

추가 기능:
- 구조체
- 열거형
- 메서드
- 패턴 매칭

#### 단계 3: Full Vais

완전한 언어 기능:
- 제네릭
- 트레이트
- 비동기
- 클로저

### 4.3 테스트 전략

```
1. 단위 테스트
   - 각 컴포넌트별 테스트
   - Rust vaisc와 동일한 출력 검증

2. 통합 테스트
   - examples/ 디렉토리의 모든 파일 컴파일
   - 실행 결과 비교

3. 부트스트래핑 테스트
   - Stage 1 == Stage 2 바이너리 동일성
   - 3-way 비교 (Rust, Stage1, Stage2)
```

---

## 5. 현재 Vais 언어 제약사항 분석

### 5.1 Self-hosting에 필요하지만 누락된 기능

| 기능 | 상태 | 대안 |
|------|------|------|
| String interpolation | 없음 | format!() 함수 사용 |
| Raw string literals | 없음 | 이스케이프 시퀀스 사용 |
| Derive 매크로 | 없음 | 수동 구현 |
| 패턴 매칭 @ 바인딩 | 있음 | - |

### 5.2 표준 라이브러리 의존성

Self-hosting에 필요한 std 모듈:
- `std/vec.vais` - 동적 배열
- `std/hashmap.vais` - 해시맵
- `std/string.vais` - 문자열 조작
- `std/option.vais` - Option 타입
- `std/result.vais` - Result 타입
- `std/file.vais` - 파일 I/O

---

## 6. 예상 일정

| Phase | 작업 | 기간 | 누적 |
|-------|------|------|------|
| 1 | 기반 인프라 (Span, Error, Token) | 1주 | 1주 |
| 2 | Lexer | 1주 | 2주 |
| 3 | AST 정의 | 1주 | 3주 |
| 4 | Parser | 2주 | 5주 |
| 5 | Type Checker | 2주 | 7주 |
| 6 | Code Generator | 2주 | 9주 |
| 7 | CLI 및 통합 | 1주 | 10주 |
| 8 | 부트스트래핑 검증 | 1주 | 11주 |

**총 예상 기간: 약 11주 (3개월)**

---

## 7. 위험 요소 및 완화 방안

### 7.1 위험 요소

1. **언어 기능 부족**: Self-hosting에 필요한 기능이 Vais에 없을 수 있음
2. **성능 문제**: Vais로 작성된 컴파일러가 Rust 버전보다 느릴 수 있음
3. **디버깅 어려움**: 컴파일러가 자기 자신을 컴파일할 때 버그 추적이 어려움

### 7.2 완화 방안

1. **점진적 구현**: 최소 기능부터 시작하여 점진적으로 확장
2. **철저한 테스트**: 각 단계에서 Rust 버전과 동일한 출력 검증
3. **상세한 로깅**: 디버그 모드에서 상세한 중간 결과 출력
4. **폴백 전략**: 문제 발생 시 Rust 버전으로 폴백 가능

---

## 8. 참고 자료

- [Bootstrapping a Compiler](https://en.wikipedia.org/wiki/Bootstrapping_(compilers))
- [The Rust Compiler Bootstrap](https://rustc-dev-guide.rust-lang.org/building/bootstrapping.html)
- [Go Compiler Bootstrap](https://go.dev/doc/go1.5#bootstrap)
- docs/LANGUAGE_SPEC.md - Vais 언어 사양
- crates/vais-*/src/*.rs - 현재 Rust 컴파일러 소스
