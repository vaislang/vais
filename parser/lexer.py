"""
Vais Lexer - 토큰화 모듈
Vibe AI SSW Language Parser v0.2
"""

from dataclasses import dataclass
from enum import Enum, auto
from typing import List, Optional, Iterator
import re


class TokenType(Enum):
    """Vais 토큰 타입"""
    # 블록 키워드
    UNIT = auto()
    META = auto()
    ENDMETA = auto()
    INPUT = auto()
    ENDINPUT = auto()
    OUTPUT = auto()
    ENDOUTPUT = auto()
    INTENT = auto()
    ENDINTENT = auto()
    CONSTRAINT = auto()
    ENDCONSTRAINT = auto()
    FLOW = auto()
    ENDFLOW = auto()
    EXECUTION = auto()
    ENDEXECUTION = auto()
    VERIFY = auto()
    ENDVERIFY = auto()
    END = auto()

    # UNIT 타입
    FUNCTION = auto()
    SERVICE = auto()
    PIPELINE = auto()
    MODULE = auto()

    # META 키워드
    DOMAIN = auto()
    DETERMINISM = auto()
    IDEMPOTENT = auto()
    PURE = auto()
    TIMEOUT = auto()
    RETRY = auto()

    # INTENT 키워드
    GOAL = auto()
    PRIORITY = auto()
    ON_FAILURE = auto()

    # GOAL 타입
    TRANSFORM = auto()
    VALIDATE = auto()
    AGGREGATE = auto()
    FILTER = auto()
    ROUTE = auto()
    COMPOSE = auto()
    FETCH = auto()

    # PRIORITY 값
    CORRECTNESS = auto()
    PERFORMANCE = auto()
    MEMORY = auto()
    LATENCY = auto()
    THROUGHPUT = auto()

    # FAILURE 전략
    ABORT = auto()
    FALLBACK = auto()
    DEFAULT = auto()

    # CONSTRAINT 키워드
    REQUIRE = auto()
    FORBID = auto()
    PREFER = auto()
    INVARIANT = auto()
    WITHIN = auto()

    # FLOW 키워드
    NODE = auto()
    EDGE = auto()
    WHEN = auto()

    # 연산 타입
    MAP = auto()
    REDUCE = auto()
    SPLIT = auto()
    MERGE = auto()
    BRANCH = auto()
    JOIN = auto()
    RACE = auto()
    STORE = auto()
    CALL = auto()
    EMIT = auto()
    SUBSCRIBE = auto()
    SANITIZE = auto()
    AUTHORIZE = auto()

    # EXECUTION 키워드
    PARALLEL = auto()
    TARGET = auto()
    ISOLATION = auto()
    CACHE = auto()

    # TARGET 값
    ANY = auto()
    CPU = auto()
    GPU = auto()
    WASM = auto()
    NATIVE = auto()

    # MEMORY 값
    BOUNDED = auto()
    UNBOUNDED = auto()
    STACK_ONLY = auto()

    # ISOLATION 값
    NONE = auto()
    THREAD = auto()
    PROCESS = auto()
    CONTAINER = auto()

    # CACHE 값
    LRU = auto()
    TTL = auto()

    # VERIFY 키워드
    ASSERT = auto()
    PROPERTY = auto()
    POSTCONDITION = auto()
    TEST = auto()
    FORALL = auto()
    EXISTS = auto()
    EVENTUALLY = auto()
    ALWAYS = auto()

    # 타입 키워드
    INT = auto()
    INT8 = auto()
    INT16 = auto()
    INT32 = auto()
    INT64 = auto()
    UINT = auto()
    UINT8 = auto()
    UINT16 = auto()
    UINT32 = auto()
    UINT64 = auto()
    FLOAT32 = auto()
    FLOAT64 = auto()
    BOOL = auto()
    STRING = auto()
    BYTES = auto()
    VOID = auto()
    ARRAY = auto()
    STRUCT = auto()
    OPTIONAL = auto()
    UNION = auto()

    # 논리 연산자
    AND = auto()
    OR = auto()
    XOR = auto()
    NOT = auto()
    IMPLIES = auto()
    IN = auto()
    MATCH = auto()
    LEN = auto()
    CONTAINS = auto()
    RANGE = auto()
    NOW = auto()
    SUM = auto()
    COUNT = auto()

    # 리터럴
    NUMBER = auto()
    FLOAT = auto()
    STRING_LITERAL = auto()
    BOOLEAN = auto()
    REGEX = auto()

    # 식별자 및 참조
    IDENTIFIER = auto()
    EXTERNAL_REF = auto()  # @로 시작
    VERSION = auto()       # V1.0.0

    # 구분자
    COLON = auto()         # :
    COMMA = auto()         # ,
    DOT = auto()           # .
    ARROW = auto()         # ->
    GT = auto()            # >
    LT = auto()            # <
    GTE = auto()           # >=
    LTE = auto()           # <=
    EQ = auto()            # ==
    NEQ = auto()           # !=
    ASSIGN = auto()        # =
    LPAREN = auto()        # (
    RPAREN = auto()        # )
    LBRACKET = auto()      # [
    LBRACE = auto()        # {
    RBRACKET = auto()      # ]
    RBRACE = auto()        # }
    PIPE = auto()          # |

    # 산술 연산자
    PLUS = auto()          # +
    MINUS = auto()         # -
    STAR = auto()          # *
    SLASH = auto()         # /

    # 특수
    COMMENT = auto()
    NEWLINE = auto()
    EOF = auto()
    UNKNOWN = auto()


@dataclass
class Token:
    """토큰 데이터 구조"""
    type: TokenType
    value: str
    line: int
    column: int

    def __repr__(self):
        return f"Token({self.type.name}, {self.value!r}, L{self.line}:C{self.column})"


# 키워드 매핑
KEYWORDS = {
    # 블록
    'UNIT': TokenType.UNIT,
    'META': TokenType.META,
    'ENDMETA': TokenType.ENDMETA,
    'INPUT': TokenType.INPUT,
    'ENDINPUT': TokenType.ENDINPUT,
    'OUTPUT': TokenType.OUTPUT,
    'ENDOUTPUT': TokenType.ENDOUTPUT,
    'INTENT': TokenType.INTENT,
    'ENDINTENT': TokenType.ENDINTENT,
    'CONSTRAINT': TokenType.CONSTRAINT,
    'ENDCONSTRAINT': TokenType.ENDCONSTRAINT,
    'FLOW': TokenType.FLOW,
    'ENDFLOW': TokenType.ENDFLOW,
    'EXECUTION': TokenType.EXECUTION,
    'ENDEXECUTION': TokenType.ENDEXECUTION,
    'VERIFY': TokenType.VERIFY,
    'ENDVERIFY': TokenType.ENDVERIFY,
    'END': TokenType.END,

    # UNIT 타입
    'FUNCTION': TokenType.FUNCTION,
    'SERVICE': TokenType.SERVICE,
    'PIPELINE': TokenType.PIPELINE,
    'MODULE': TokenType.MODULE,

    # META
    'DOMAIN': TokenType.DOMAIN,
    'DETERMINISM': TokenType.DETERMINISM,
    'IDEMPOTENT': TokenType.IDEMPOTENT,
    'PURE': TokenType.PURE,
    'TIMEOUT': TokenType.TIMEOUT,
    'RETRY': TokenType.RETRY,

    # INTENT
    'GOAL': TokenType.GOAL,
    'PRIORITY': TokenType.PRIORITY,
    'ON_FAILURE': TokenType.ON_FAILURE,
    'TRANSFORM': TokenType.TRANSFORM,
    'VALIDATE': TokenType.VALIDATE,
    'AGGREGATE': TokenType.AGGREGATE,
    'FILTER': TokenType.FILTER,
    'ROUTE': TokenType.ROUTE,
    'COMPOSE': TokenType.COMPOSE,
    'FETCH': TokenType.FETCH,
    'CORRECTNESS': TokenType.CORRECTNESS,
    'PERFORMANCE': TokenType.PERFORMANCE,
    'MEMORY': TokenType.MEMORY,
    'LATENCY': TokenType.LATENCY,
    'THROUGHPUT': TokenType.THROUGHPUT,
    'ABORT': TokenType.ABORT,
    'FALLBACK': TokenType.FALLBACK,
    'DEFAULT': TokenType.DEFAULT,

    # CONSTRAINT
    'REQUIRE': TokenType.REQUIRE,
    'FORBID': TokenType.FORBID,
    'PREFER': TokenType.PREFER,
    'INVARIANT': TokenType.INVARIANT,
    'WITHIN': TokenType.WITHIN,

    # FLOW
    'NODE': TokenType.NODE,
    'EDGE': TokenType.EDGE,
    'WHEN': TokenType.WHEN,
    'MAP': TokenType.MAP,
    'REDUCE': TokenType.REDUCE,
    'SPLIT': TokenType.SPLIT,
    'MERGE': TokenType.MERGE,
    'BRANCH': TokenType.BRANCH,
    'JOIN': TokenType.JOIN,
    'RACE': TokenType.RACE,
    'STORE': TokenType.STORE,
    'CALL': TokenType.CALL,
    'EMIT': TokenType.EMIT,
    'SUBSCRIBE': TokenType.SUBSCRIBE,
    'SANITIZE': TokenType.SANITIZE,
    'AUTHORIZE': TokenType.AUTHORIZE,

    # EXECUTION
    'PARALLEL': TokenType.PARALLEL,
    'TARGET': TokenType.TARGET,
    'ISOLATION': TokenType.ISOLATION,
    'CACHE': TokenType.CACHE,
    'ANY': TokenType.ANY,
    'CPU': TokenType.CPU,
    'GPU': TokenType.GPU,
    'WASM': TokenType.WASM,
    'NATIVE': TokenType.NATIVE,
    'BOUNDED': TokenType.BOUNDED,
    'UNBOUNDED': TokenType.UNBOUNDED,
    'STACK_ONLY': TokenType.STACK_ONLY,
    'NONE': TokenType.NONE,
    'THREAD': TokenType.THREAD,
    'PROCESS': TokenType.PROCESS,
    'CONTAINER': TokenType.CONTAINER,
    'LRU': TokenType.LRU,
    'TTL': TokenType.TTL,

    # VERIFY
    'ASSERT': TokenType.ASSERT,
    'PROPERTY': TokenType.PROPERTY,
    'POSTCONDITION': TokenType.POSTCONDITION,
    'TEST': TokenType.TEST,
    'FORALL': TokenType.FORALL,
    'EXISTS': TokenType.EXISTS,
    'EVENTUALLY': TokenType.EVENTUALLY,
    'ALWAYS': TokenType.ALWAYS,

    # 타입
    'INT': TokenType.INT,
    'INT8': TokenType.INT8,
    'INT16': TokenType.INT16,
    'INT32': TokenType.INT32,
    'INT64': TokenType.INT64,
    'UINT': TokenType.UINT,
    'UINT8': TokenType.UINT8,
    'UINT16': TokenType.UINT16,
    'UINT32': TokenType.UINT32,
    'UINT64': TokenType.UINT64,
    'FLOAT32': TokenType.FLOAT32,
    'FLOAT64': TokenType.FLOAT64,
    'BOOL': TokenType.BOOL,
    'STRING': TokenType.STRING,
    'BYTES': TokenType.BYTES,
    'VOID': TokenType.VOID,
    'ARRAY': TokenType.ARRAY,
    'STRUCT': TokenType.STRUCT,
    'OPTIONAL': TokenType.OPTIONAL,
    'UNION': TokenType.UNION,

    # 논리
    'AND': TokenType.AND,
    'OR': TokenType.OR,
    'XOR': TokenType.XOR,
    'NOT': TokenType.NOT,
    'IMPLIES': TokenType.IMPLIES,
    'IN': TokenType.IN,
    'MATCH': TokenType.MATCH,
    'LEN': TokenType.LEN,
    'CONTAINS': TokenType.CONTAINS,
    'RANGE': TokenType.RANGE,
    'NOW': TokenType.NOW,
    'SUM': TokenType.SUM,
    'COUNT': TokenType.COUNT,

    # 불리언
    'true': TokenType.BOOLEAN,
    'false': TokenType.BOOLEAN,
}


class Lexer:
    """Vais 렉서"""

    def __init__(self, source: str):
        self.source = source
        self.pos = 0
        self.line = 1
        self.column = 1
        self.tokens: List[Token] = []

    def peek(self, offset: int = 0) -> Optional[str]:
        """현재 위치에서 offset만큼 앞의 문자 반환"""
        pos = self.pos + offset
        if pos < len(self.source):
            return self.source[pos]
        return None

    def advance(self) -> Optional[str]:
        """다음 문자로 이동"""
        if self.pos >= len(self.source):
            return None
        char = self.source[self.pos]
        self.pos += 1
        if char == '\n':
            self.line += 1
            self.column = 1
        else:
            self.column += 1
        return char

    def skip_whitespace(self):
        """공백 스킵 (줄바꿈 제외)"""
        while self.peek() and self.peek() in ' \t\r':
            self.advance()

    def skip_comment(self):
        """주석 스킵"""
        if self.peek() == '#':
            while self.peek() and self.peek() != '\n':
                self.advance()

    def make_token(self, token_type: TokenType, value: str, line: int, column: int) -> Token:
        """토큰 생성"""
        return Token(token_type, value, line, column)

    def read_string(self) -> Token:
        """문자열 리터럴 읽기"""
        start_line = self.line
        start_column = self.column
        quote = self.advance()  # " 또는 '
        value = ""

        while self.peek() and self.peek() != quote:
            if self.peek() == '\\':
                self.advance()
                escaped = self.advance()
                if escaped == 'n':
                    value += '\n'
                elif escaped == 't':
                    value += '\t'
                elif escaped == '\\':
                    value += '\\'
                elif escaped == quote:
                    value += quote
                else:
                    value += escaped
            else:
                value += self.advance()

        if self.peek() == quote:
            self.advance()  # 닫는 따옴표

        return self.make_token(TokenType.STRING_LITERAL, value, start_line, start_column)

    def read_regex(self) -> Token:
        """정규식 리터럴 읽기 (/pattern/)"""
        start_line = self.line
        start_column = self.column
        self.advance()  # 시작 /
        value = ""

        while self.peek() and self.peek() != '/':
            if self.peek() == '\\':
                value += self.advance()
            value += self.advance()

        if self.peek() == '/':
            self.advance()  # 닫는 /

        return self.make_token(TokenType.REGEX, value, start_line, start_column)

    def read_number(self) -> Token:
        """숫자 리터럴 읽기"""
        start_line = self.line
        start_column = self.column
        value = ""
        is_float = False

        # 음수 처리
        if self.peek() == '-':
            value += self.advance()

        while self.peek() and (self.peek().isdigit() or self.peek() == '.'):
            if self.peek() == '.':
                if is_float:
                    break
                is_float = True
            value += self.advance()

        # 시간 단위 (ms, s, m, h)
        if self.peek() and self.peek() in 'msh':
            unit_start = self.pos
            if self.peek() == 'm' and self.peek(1) == 's':
                value += self.advance() + self.advance()
            elif self.peek() in 'smh':
                value += self.advance()

        # 크기 단위 (KB, MB, GB)
        if self.peek() and self.peek() in 'KMG':
            value += self.advance()
            if self.peek() == 'B':
                value += self.advance()

        token_type = TokenType.FLOAT if is_float else TokenType.NUMBER
        return self.make_token(token_type, value, start_line, start_column)

    def read_identifier(self) -> Token:
        """식별자 또는 키워드 읽기"""
        start_line = self.line
        start_column = self.column
        value = ""

        while self.peek() and (self.peek().isalnum() or self.peek() == '_'):
            value += self.advance()

        # 버전 체크 (V1.0.0)
        if value.startswith('V') and len(value) > 1 and value[1].isdigit():
            while self.peek() and (self.peek().isdigit() or self.peek() == '.'):
                value += self.advance()
            return self.make_token(TokenType.VERSION, value, start_line, start_column)

        # 키워드 체크
        token_type = KEYWORDS.get(value, TokenType.IDENTIFIER)
        return self.make_token(token_type, value, start_line, start_column)

    def read_external_ref(self) -> Token:
        """외부 참조 읽기 (@로 시작)"""
        start_line = self.line
        start_column = self.column
        self.advance()  # @
        value = "@"

        while self.peek() and (self.peek().isalnum() or self.peek() in '_.'):
            value += self.advance()

        return self.make_token(TokenType.EXTERNAL_REF, value, start_line, start_column)

    def tokenize(self) -> List[Token]:
        """전체 소스 토큰화"""
        while self.pos < len(self.source):
            self.skip_whitespace()
            self.skip_comment()

            if self.pos >= len(self.source):
                break

            char = self.peek()
            start_line = self.line
            start_column = self.column

            # 줄바꿈
            if char == '\n':
                self.advance()
                self.tokens.append(self.make_token(TokenType.NEWLINE, '\n', start_line, start_column))
                continue

            # 문자열
            if char in '"\'':
                self.tokens.append(self.read_string())
                continue

            # 정규식
            if char == '/' and self.peek(1) and self.peek(1) not in ' \t\n':
                self.tokens.append(self.read_regex())
                continue

            # 숫자
            if char.isdigit() or (char == '-' and self.peek(1) and self.peek(1).isdigit()):
                self.tokens.append(self.read_number())
                continue

            # 외부 참조
            if char == '@':
                self.tokens.append(self.read_external_ref())
                continue

            # 식별자/키워드
            if char.isalpha() or char == '_':
                self.tokens.append(self.read_identifier())
                continue

            # 2문자 연산자
            two_char = char + (self.peek(1) or '')
            if two_char == '->':
                self.advance()
                self.advance()
                self.tokens.append(self.make_token(TokenType.ARROW, '->', start_line, start_column))
                continue
            if two_char == '>=':
                self.advance()
                self.advance()
                self.tokens.append(self.make_token(TokenType.GTE, '>=', start_line, start_column))
                continue
            if two_char == '<=':
                self.advance()
                self.advance()
                self.tokens.append(self.make_token(TokenType.LTE, '<=', start_line, start_column))
                continue
            if two_char == '==':
                self.advance()
                self.advance()
                self.tokens.append(self.make_token(TokenType.EQ, '==', start_line, start_column))
                continue
            if two_char == '!=':
                self.advance()
                self.advance()
                self.tokens.append(self.make_token(TokenType.NEQ, '!=', start_line, start_column))
                continue

            # 1문자 연산자
            single_char_tokens = {
                ':': TokenType.COLON,
                ',': TokenType.COMMA,
                '.': TokenType.DOT,
                '>': TokenType.GT,
                '<': TokenType.LT,
                '=': TokenType.ASSIGN,
                '(': TokenType.LPAREN,
                ')': TokenType.RPAREN,
                '[': TokenType.LBRACKET,
                ']': TokenType.RBRACKET,
                '{': TokenType.LBRACE,
                '}': TokenType.RBRACE,
                '|': TokenType.PIPE,
                '+': TokenType.PLUS,
                '-': TokenType.MINUS,
                '*': TokenType.STAR,
                '/': TokenType.SLASH,
            }

            if char in single_char_tokens:
                self.advance()
                self.tokens.append(self.make_token(single_char_tokens[char], char, start_line, start_column))
                continue

            # 알 수 없는 문자
            self.advance()
            self.tokens.append(self.make_token(TokenType.UNKNOWN, char, start_line, start_column))

        # EOF 토큰
        self.tokens.append(self.make_token(TokenType.EOF, '', self.line, self.column))
        return self.tokens

    def get_tokens_without_newlines(self) -> List[Token]:
        """줄바꿈 토큰 제외한 토큰 목록"""
        return [t for t in self.tokens if t.type != TokenType.NEWLINE]


def tokenize(source: str) -> List[Token]:
    """소스 코드 토큰화 헬퍼 함수"""
    lexer = Lexer(source)
    return lexer.tokenize()


if __name__ == "__main__":
    # 테스트
    test_source = '''
UNIT FUNCTION examples.hello_world V1.0.0

META
  DOMAIN examples.basic
  DETERMINISM true
ENDMETA

INPUT
  name : STRING [LEN >= 1]
ENDINPUT

OUTPUT
  message : STRING
ENDOUTPUT

END
'''
    tokens = tokenize(test_source)
    for token in tokens:
        if token.type != TokenType.NEWLINE:
            print(token)
