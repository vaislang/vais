"""
Vais Parser - 구문 분석기
Vibe AI SSW Language Parser v0.2
"""

from typing import List, Optional, Any, Set
from dataclasses import dataclass

from lexer import Token, TokenType, Lexer, tokenize
from ast_nodes import (
    ASTNode, VaisUnit, UnitBlock, MetaBlock, MetaEntry,
    InputBlock, InputEntry, OutputBlock, OutputEntry,
    IntentBlock, GoalSpec, ConstraintBlock, Constraint,
    FlowBlock, FlowNode, FlowEdge, NodeParam,
    ExecutionBlock, VerifyBlock, VerifyEntry,
    TypeNode, PrimitiveType, ArrayType, MapType, StructType, OptionalType, UnionType, TypeRef,
    Expression, Literal, Identifier, QualifiedName, ExternalRef, FieldAccess,
    BinaryOp, UnaryOp, FunctionCall, ArrayAccess, InputConstraint,
    UnitType, GoalType, PriorityType, FailureStrategy, ConstraintType,
    OpType, TargetType, MemoryType, IsolationType, CacheType, VerifyType
)


@dataclass
class ParseError(Exception):
    """파싱 에러"""
    message: str
    line: int
    column: int
    token: Optional[Token] = None

    def __str__(self):
        return f"ParseError at L{self.line}:C{self.column}: {self.message}"


class Parser:
    """Vais 파서"""

    def __init__(self, tokens: List[Token]):
        # 줄바꿈 토큰 제거
        self.tokens = [t for t in tokens if t.type != TokenType.NEWLINE]
        self.pos = 0
        self.errors: List[ParseError] = []

    def current(self) -> Token:
        """현재 토큰"""
        if self.pos < len(self.tokens):
            return self.tokens[self.pos]
        return self.tokens[-1]  # EOF

    def peek(self, offset: int = 0) -> Token:
        """앞의 토큰 확인"""
        pos = self.pos + offset
        if pos < len(self.tokens):
            return self.tokens[pos]
        return self.tokens[-1]  # EOF

    def advance(self) -> Token:
        """다음 토큰으로 이동"""
        token = self.current()
        if self.pos < len(self.tokens) - 1:
            self.pos += 1
        return token

    def expect(self, *token_types: TokenType) -> Token:
        """특정 토큰 타입 기대"""
        token = self.current()
        if token.type not in token_types:
            expected = " or ".join(t.name for t in token_types)
            raise ParseError(
                f"Expected {expected}, got {token.type.name} '{token.value}'",
                token.line, token.column, token
            )
        return self.advance()

    def match(self, *token_types: TokenType) -> bool:
        """토큰 타입 매치 확인"""
        return self.current().type in token_types

    def skip_newlines(self):
        """줄바꿈 스킵"""
        while self.match(TokenType.NEWLINE):
            self.advance()

    # =========================================================================
    # 타입 파싱
    # =========================================================================

    def parse_type(self) -> TypeNode:
        """타입 파싱"""
        token = self.current()

        # 원시 타입
        primitive_types = {
            TokenType.INT, TokenType.INT8, TokenType.INT16, TokenType.INT32, TokenType.INT64,
            TokenType.UINT, TokenType.UINT8, TokenType.UINT16, TokenType.UINT32, TokenType.UINT64,
            TokenType.FLOAT32, TokenType.FLOAT64, TokenType.BOOL, TokenType.STRING,
            TokenType.BYTES, TokenType.VOID
        }

        if token.type in primitive_types:
            self.advance()
            return PrimitiveType(name=token.value, line=token.line, column=token.column)

        # ARRAY<T>
        if token.type == TokenType.ARRAY:
            self.advance()
            self.expect(TokenType.LT)
            element_type = self.parse_type()
            self.expect(TokenType.GT)
            return ArrayType(element_type=element_type, line=token.line, column=token.column)

        # MAP<K, V>
        if token.type == TokenType.MAP:
            self.advance()
            self.expect(TokenType.LT)
            key_type = self.parse_type()
            self.expect(TokenType.COMMA)
            value_type = self.parse_type()
            self.expect(TokenType.GT)
            return MapType(key_type=key_type, value_type=value_type, line=token.line, column=token.column)

        # STRUCT {...}
        if token.type == TokenType.STRUCT:
            self.advance()
            self.expect(TokenType.LBRACE)
            fields = {}
            while not self.match(TokenType.RBRACE):
                field_name = self.expect(TokenType.IDENTIFIER).value
                self.expect(TokenType.COLON)
                field_type = self.parse_type()
                fields[field_name] = field_type
                if self.match(TokenType.COMMA):
                    self.advance()
            self.expect(TokenType.RBRACE)
            return StructType(fields=fields, line=token.line, column=token.column)

        # OPTIONAL<T>
        if token.type == TokenType.OPTIONAL:
            self.advance()
            self.expect(TokenType.LT)
            inner_type = self.parse_type()
            self.expect(TokenType.GT)
            return OptionalType(inner_type=inner_type, line=token.line, column=token.column)

        # UNION<T1 | T2>
        if token.type == TokenType.UNION:
            self.advance()
            self.expect(TokenType.LT)
            types = [self.parse_type()]
            while self.match(TokenType.PIPE):
                self.advance()
                types.append(self.parse_type())
            self.expect(TokenType.GT)
            return UnionType(types=types, line=token.line, column=token.column)

        # 타입 참조 (@ref)
        if token.type == TokenType.EXTERNAL_REF:
            self.advance()
            return TypeRef(ref=token.value, line=token.line, column=token.column)

        raise ParseError(f"Expected type, got {token.type.name}", token.line, token.column, token)

    # =========================================================================
    # 표현식 파싱
    # =========================================================================

    def parse_expression(self) -> Expression:
        """표현식 파싱 (연산자 우선순위 처리)"""
        return self.parse_or_expression()

    def parse_or_expression(self) -> Expression:
        """OR 표현식"""
        left = self.parse_and_expression()
        while self.match(TokenType.OR):
            op = self.advance().value
            right = self.parse_and_expression()
            left = BinaryOp(left=left, operator=op, right=right, line=left.line, column=left.column)
        return left

    def parse_and_expression(self) -> Expression:
        """AND 표현식"""
        left = self.parse_comparison()
        while self.match(TokenType.AND):
            op = self.advance().value
            right = self.parse_comparison()
            left = BinaryOp(left=left, operator=op, right=right, line=left.line, column=left.column)
        return left

    def parse_comparison(self) -> Expression:
        """비교 표현식"""
        left = self.parse_additive()
        compare_ops = {TokenType.EQ, TokenType.NEQ, TokenType.LT, TokenType.GT,
                       TokenType.LTE, TokenType.GTE, TokenType.IN, TokenType.MATCH}
        while self.current().type in compare_ops:
            op = self.advance().value
            right = self.parse_additive()
            left = BinaryOp(left=left, operator=op, right=right, line=left.line, column=left.column)
        return left

    def parse_additive(self) -> Expression:
        """덧셈/뺄셈 표현식"""
        left = self.parse_multiplicative()
        while self.match(TokenType.PLUS, TokenType.MINUS):
            op = self.advance().value
            right = self.parse_multiplicative()
            left = BinaryOp(left=left, operator=op, right=right, line=left.line, column=left.column)
        return left

    def parse_multiplicative(self) -> Expression:
        """곱셈/나눗셈 표현식"""
        left = self.parse_unary()
        while self.match(TokenType.STAR, TokenType.SLASH):
            op = self.advance().value
            right = self.parse_unary()
            left = BinaryOp(left=left, operator=op, right=right, line=left.line, column=left.column)
        return left

    def parse_unary(self) -> Expression:
        """단항 표현식"""
        if self.match(TokenType.NOT, TokenType.MINUS):
            op = self.advance().value
            operand = self.parse_unary()
            return UnaryOp(operator=op, operand=operand, line=operand.line, column=operand.column)
        return self.parse_primary()

    def parse_primary(self) -> Expression:
        """기본 표현식"""
        token = self.current()

        # 괄호
        if token.type == TokenType.LPAREN:
            self.advance()
            expr = self.parse_expression()
            self.expect(TokenType.RPAREN)
            return expr

        # 리터럴
        if token.type == TokenType.NUMBER:
            self.advance()
            return Literal(value=int(token.value.rstrip('smhKMGB')), literal_type="number",
                          line=token.line, column=token.column)

        if token.type == TokenType.FLOAT:
            self.advance()
            return Literal(value=float(token.value), literal_type="float",
                          line=token.line, column=token.column)

        if token.type == TokenType.STRING_LITERAL:
            self.advance()
            return Literal(value=token.value, literal_type="string",
                          line=token.line, column=token.column)

        if token.type == TokenType.BOOLEAN:
            self.advance()
            return Literal(value=token.value == "true", literal_type="bool",
                          line=token.line, column=token.column)

        if token.type == TokenType.REGEX:
            self.advance()
            return Literal(value=token.value, literal_type="regex",
                          line=token.line, column=token.column)

        # VOID
        if token.type == TokenType.VOID:
            self.advance()
            return Literal(value=None, literal_type="void",
                          line=token.line, column=token.column)

        # 외부 참조
        if token.type == TokenType.EXTERNAL_REF:
            self.advance()
            return ExternalRef(ref=token.value, line=token.line, column=token.column)

        # 내장 함수 (LEN, CONTAINS, NOW, etc.)
        builtin_funcs = {TokenType.LEN, TokenType.CONTAINS, TokenType.RANGE, TokenType.NOW,
                         TokenType.SUM, TokenType.COUNT}
        if token.type in builtin_funcs:
            func_name = self.advance().value
            if self.match(TokenType.LPAREN):
                self.advance()
                args = []
                if not self.match(TokenType.RPAREN):
                    args.append(self.parse_expression())
                    while self.match(TokenType.COMMA):
                        self.advance()
                        args.append(self.parse_expression())
                self.expect(TokenType.RPAREN)
                return FunctionCall(name=func_name, arguments=args, line=token.line, column=token.column)
            return Identifier(name=func_name, line=token.line, column=token.column)

        # 식별자 (필드 접근 포함)
        if token.type == TokenType.IDENTIFIER:
            return self.parse_field_access()

        # INPUT/OUTPUT 키워드 (필드 접근용)
        if token.type in {TokenType.INPUT, TokenType.OUTPUT}:
            base_name = self.advance().value
            if self.match(TokenType.DOT):
                self.advance()
                field = self.expect(TokenType.IDENTIFIER).value
                base = Identifier(name=base_name, line=token.line, column=token.column)
                return FieldAccess(base=base, field_name=field, line=token.line, column=token.column)
            return Identifier(name=base_name, line=token.line, column=token.column)

        raise ParseError(f"Unexpected token {token.type.name}", token.line, token.column, token)

    def parse_field_access(self) -> Expression:
        """필드 접근 파싱 (a.b.c)"""
        token = self.current()
        parts = [self.expect(TokenType.IDENTIFIER).value]

        while self.match(TokenType.DOT):
            self.advance()
            if self.match(TokenType.IDENTIFIER):
                parts.append(self.advance().value)
            else:
                break

        if len(parts) == 1:
            return Identifier(name=parts[0], line=token.line, column=token.column)

        # 필드 접근 체인 생성
        expr: Expression = Identifier(name=parts[0], line=token.line, column=token.column)
        for part in parts[1:]:
            expr = FieldAccess(base=expr, field_name=part, line=token.line, column=token.column)
        return expr

    def parse_qualified_name(self) -> QualifiedName:
        """정규화된 이름 파싱 (a.b.c)"""
        token = self.current()
        parts = [self.expect(TokenType.IDENTIFIER).value]

        while self.match(TokenType.DOT):
            self.advance()
            parts.append(self.expect(TokenType.IDENTIFIER).value)

        return QualifiedName(parts=parts, line=token.line, column=token.column)

    # =========================================================================
    # 블록 파싱
    # =========================================================================

    def parse_unit_block(self) -> UnitBlock:
        """UNIT 블록 파싱"""
        token = self.expect(TokenType.UNIT)

        # 유닛 타입
        unit_type_map = {
            TokenType.FUNCTION: UnitType.FUNCTION,
            TokenType.SERVICE: UnitType.SERVICE,
            TokenType.PIPELINE: UnitType.PIPELINE,
            TokenType.MODULE: UnitType.MODULE,
        }
        unit_type_token = self.expect(*unit_type_map.keys())
        unit_type = unit_type_map[unit_type_token.type]

        # 유닛 ID
        unit_id = self.parse_qualified_name()

        # 버전 (선택)
        version = None
        if self.match(TokenType.VERSION):
            version = self.advance().value

        return UnitBlock(unit_type=unit_type, unit_id=unit_id, version=version,
                        line=token.line, column=token.column)

    def parse_meta_block(self) -> MetaBlock:
        """META 블록 파싱"""
        token = self.expect(TokenType.META)
        entries = []

        while not self.match(TokenType.ENDMETA):
            entry = self.parse_meta_entry()
            entries.append(entry)

        self.expect(TokenType.ENDMETA)
        return MetaBlock(entries=entries, line=token.line, column=token.column)

    def parse_meta_entry(self) -> MetaEntry:
        """META 엔트리 파싱"""
        key_tokens = {
            TokenType.DOMAIN, TokenType.DETERMINISM, TokenType.IDEMPOTENT,
            TokenType.PURE, TokenType.TIMEOUT, TokenType.RETRY
        }
        key_token = self.expect(*key_tokens)
        key = key_token.value

        # 값 파싱
        value: Any
        if self.match(TokenType.BOOLEAN):
            value = self.advance().value == "true"
        elif self.match(TokenType.NUMBER):
            value = self.advance().value
        elif self.match(TokenType.IDENTIFIER):
            value = self.parse_qualified_name().full_name
        else:
            value = self.advance().value

        return MetaEntry(key=key, value=value, line=key_token.line, column=key_token.column)

    def parse_input_block(self) -> InputBlock:
        """INPUT 블록 파싱"""
        token = self.expect(TokenType.INPUT)
        entries = []

        while not self.match(TokenType.ENDINPUT):
            entry = self.parse_input_entry()
            entries.append(entry)

        self.expect(TokenType.ENDINPUT)
        return InputBlock(entries=entries, line=token.line, column=token.column)

    def parse_input_entry(self) -> InputEntry:
        """INPUT 엔트리 파싱"""
        name_token = self.expect(TokenType.IDENTIFIER)
        name = name_token.value
        self.expect(TokenType.COLON)
        type_node = self.parse_type()

        # 제약조건 [...] (선택)
        constraints = []
        if self.match(TokenType.LBRACKET):
            self.advance()
            while not self.match(TokenType.RBRACKET):
                expr = self.parse_expression()
                constraints.append(InputConstraint(expression=expr, line=expr.line, column=expr.column))
                if self.match(TokenType.COMMA):
                    self.advance()
            self.expect(TokenType.RBRACKET)

        return InputEntry(name=name, type_node=type_node, constraints=constraints,
                         line=name_token.line, column=name_token.column)

    def parse_output_block(self) -> OutputBlock:
        """OUTPUT 블록 파싱"""
        token = self.expect(TokenType.OUTPUT)
        entries = []

        while not self.match(TokenType.ENDOUTPUT):
            entry = self.parse_output_entry()
            entries.append(entry)

        self.expect(TokenType.ENDOUTPUT)
        return OutputBlock(entries=entries, line=token.line, column=token.column)

    def parse_output_entry(self) -> OutputEntry:
        """OUTPUT 엔트리 파싱"""
        name_token = self.expect(TokenType.IDENTIFIER)
        name = name_token.value
        self.expect(TokenType.COLON)
        type_node = self.parse_type()

        # 제약조건 [...] (선택)
        constraints = []
        if self.match(TokenType.LBRACKET):
            self.advance()
            while not self.match(TokenType.RBRACKET):
                expr = self.parse_expression()
                constraints.append(InputConstraint(expression=expr, line=expr.line, column=expr.column))
                if self.match(TokenType.COMMA):
                    self.advance()
            self.expect(TokenType.RBRACKET)

        return OutputEntry(name=name, type_node=type_node, constraints=constraints,
                          line=name_token.line, column=name_token.column)

    def parse_intent_block(self) -> IntentBlock:
        """INTENT 블록 파싱"""
        token = self.expect(TokenType.INTENT)

        # GOAL
        self.expect(TokenType.GOAL)
        goal_type_map = {
            TokenType.TRANSFORM: GoalType.TRANSFORM,
            TokenType.VALIDATE: GoalType.VALIDATE,
            TokenType.AGGREGATE: GoalType.AGGREGATE,
            TokenType.FILTER: GoalType.FILTER,
            TokenType.ROUTE: GoalType.ROUTE,
            TokenType.COMPOSE: GoalType.COMPOSE,
            TokenType.FETCH: GoalType.FETCH,
        }
        goal_type_token = self.expect(*goal_type_map.keys())
        goal_type = goal_type_map[goal_type_token.type]

        self.expect(TokenType.COLON)

        # 입력 참조들
        inputs = [self.parse_expression()]
        while self.match(TokenType.COMMA):
            self.advance()
            if self.match(TokenType.ARROW):
                break
            inputs.append(self.parse_expression())

        self.expect(TokenType.ARROW)

        # 출력 참조들
        outputs = [self.parse_expression()]
        while self.match(TokenType.COMMA):
            self.advance()
            outputs.append(self.parse_expression())

        goal_spec = GoalSpec(inputs=inputs, outputs=outputs, line=goal_type_token.line, column=goal_type_token.column)

        # PRIORITY (선택)
        priorities = []
        if self.match(TokenType.PRIORITY):
            self.advance()
            priority_map = {
                TokenType.CORRECTNESS: PriorityType.CORRECTNESS,
                TokenType.PERFORMANCE: PriorityType.PERFORMANCE,
                TokenType.MEMORY: PriorityType.MEMORY,
                TokenType.LATENCY: PriorityType.LATENCY,
                TokenType.THROUGHPUT: PriorityType.THROUGHPUT,
            }
            priority_token = self.expect(*priority_map.keys())
            priorities.append(priority_map[priority_token.type])
            while self.match(TokenType.GT):
                self.advance()
                priority_token = self.expect(*priority_map.keys())
                priorities.append(priority_map[priority_token.type])

        # ON_FAILURE (선택)
        failure_strategy = None
        failure_ref = None
        if self.match(TokenType.ON_FAILURE):
            self.advance()
            strategy_map = {
                TokenType.ABORT: FailureStrategy.ABORT,
                TokenType.RETRY: FailureStrategy.RETRY,
                TokenType.FALLBACK: FailureStrategy.FALLBACK,
                TokenType.DEFAULT: FailureStrategy.DEFAULT,
            }
            strategy_token = self.expect(*strategy_map.keys())
            failure_strategy = strategy_map[strategy_token.type]
            if failure_strategy in {FailureStrategy.FALLBACK, FailureStrategy.DEFAULT}:
                failure_ref = self.parse_expression()

        self.expect(TokenType.ENDINTENT)

        return IntentBlock(goal_type=goal_type, goal_spec=goal_spec, priorities=priorities,
                          failure_strategy=failure_strategy, failure_ref=failure_ref,
                          line=token.line, column=token.column)

    def parse_constraint_block(self) -> ConstraintBlock:
        """CONSTRAINT 블록 파싱"""
        token = self.expect(TokenType.CONSTRAINT)
        constraints = []

        constraint_type_map = {
            TokenType.REQUIRE: ConstraintType.REQUIRE,
            TokenType.FORBID: ConstraintType.FORBID,
            TokenType.PREFER: ConstraintType.PREFER,
            TokenType.INVARIANT: ConstraintType.INVARIANT,
        }

        while not self.match(TokenType.ENDCONSTRAINT):
            # WITHIN 특수 처리
            if self.match(TokenType.REQUIRE) and self.peek(1).type == TokenType.WITHIN:
                self.advance()  # REQUIRE
                self.advance()  # WITHIN
                time_value = self.advance().value
                expr = Literal(value=time_value, literal_type="duration",
                              line=self.current().line, column=self.current().column)
                constraints.append(Constraint(
                    constraint_type=ConstraintType.REQUIRE,
                    expression=expr,
                    line=self.current().line, column=self.current().column
                ))
                continue

            if self.current().type in constraint_type_map:
                constraint_type_token = self.advance()
                constraint_type = constraint_type_map[constraint_type_token.type]
                expr = self.parse_expression()
                constraints.append(Constraint(
                    constraint_type=constraint_type,
                    expression=expr,
                    line=constraint_type_token.line, column=constraint_type_token.column
                ))
            else:
                # 알 수 없는 토큰 스킵
                self.advance()

        self.expect(TokenType.ENDCONSTRAINT)
        return ConstraintBlock(constraints=constraints, line=token.line, column=token.column)

    def parse_flow_block(self) -> FlowBlock:
        """FLOW 블록 파싱"""
        token = self.expect(TokenType.FLOW)
        nodes = []
        edges = []

        while not self.match(TokenType.ENDFLOW):
            if self.match(TokenType.NODE):
                nodes.append(self.parse_flow_node())
            elif self.match(TokenType.EDGE):
                edges.append(self.parse_flow_edge())
            else:
                # 알 수 없는 토큰 스킵
                self.advance()

        self.expect(TokenType.ENDFLOW)
        return FlowBlock(nodes=nodes, edges=edges, line=token.line, column=token.column)

    def parse_flow_node(self) -> FlowNode:
        """FLOW NODE 파싱"""
        token = self.expect(TokenType.NODE)
        node_id = self.expect(TokenType.IDENTIFIER).value
        self.expect(TokenType.COLON)

        # 연산 타입
        op_type_map = {
            TokenType.MAP: OpType.MAP,
            TokenType.FILTER: OpType.FILTER,
            TokenType.REDUCE: OpType.REDUCE,
            TokenType.TRANSFORM: OpType.TRANSFORM,
            TokenType.BRANCH: OpType.BRANCH,
            TokenType.MERGE: OpType.MERGE,
            TokenType.SPLIT: OpType.SPLIT,
            TokenType.JOIN: OpType.JOIN,
            TokenType.RACE: OpType.RACE,
            TokenType.FETCH: OpType.FETCH,
            TokenType.STORE: OpType.STORE,
            TokenType.CALL: OpType.CALL,
            TokenType.EMIT: OpType.EMIT,
            TokenType.SUBSCRIBE: OpType.SUBSCRIBE,
            TokenType.VALIDATE: OpType.VALIDATE,
            TokenType.SANITIZE: OpType.SANITIZE,
            TokenType.AUTHORIZE: OpType.AUTHORIZE,
        }

        custom_op = None
        if self.match(TokenType.EXTERNAL_REF):
            custom_op = self.advance().value
            op_type = OpType.CALL  # 커스텀 연산은 CALL로 처리
        else:
            op_type_token = self.expect(*op_type_map.keys())
            op_type = op_type_map[op_type_token.type]

        # 파라미터 (선택)
        params = []
        if self.match(TokenType.LPAREN):
            self.advance()
            while not self.match(TokenType.RPAREN):
                param = self.parse_node_param()
                params.append(param)
                if self.match(TokenType.COMMA):
                    self.advance()
            self.expect(TokenType.RPAREN)

        return FlowNode(node_id=node_id, op_type=op_type, params=params, custom_op=custom_op,
                       line=token.line, column=token.column)

    def parse_node_param(self) -> NodeParam:
        """노드 파라미터 파싱"""
        name_token = self.expect(TokenType.IDENTIFIER)
        name = name_token.value
        self.expect(TokenType.ASSIGN)
        value = self.parse_expression()
        return NodeParam(name=name, value=value, line=name_token.line, column=name_token.column)

    def parse_flow_edge(self) -> FlowEdge:
        """FLOW EDGE 파싱"""
        token = self.expect(TokenType.EDGE)

        # 소스 (INPUT.field 또는 node.port)
        source = self.parse_expression()

        self.expect(TokenType.ARROW)

        # 타겟 (OUTPUT.field 또는 node.port)
        target = self.parse_expression()

        # WHEN 조건 (선택)
        condition = None
        if self.match(TokenType.WHEN):
            self.advance()
            condition = self.parse_expression()

        return FlowEdge(source=source, target=target, condition=condition,
                       line=token.line, column=token.column)

    def parse_execution_block(self) -> ExecutionBlock:
        """EXECUTION 블록 파싱"""
        token = self.expect(TokenType.EXECUTION)

        parallel = False
        target = TargetType.ANY
        memory = MemoryType.UNBOUNDED
        memory_limit = None
        isolation = IsolationType.NONE
        cache = CacheType.NONE
        cache_size = None

        while not self.match(TokenType.ENDEXECUTION):
            if self.match(TokenType.PARALLEL):
                self.advance()
                parallel = self.expect(TokenType.BOOLEAN).value == "true"
            elif self.match(TokenType.TARGET):
                self.advance()
                target_map = {
                    TokenType.ANY: TargetType.ANY,
                    TokenType.CPU: TargetType.CPU,
                    TokenType.GPU: TargetType.GPU,
                    TokenType.WASM: TargetType.WASM,
                    TokenType.NATIVE: TargetType.NATIVE,
                }
                target_token = self.expect(*target_map.keys())
                target = target_map[target_token.type]
            elif self.match(TokenType.MEMORY):
                self.advance()
                memory_map = {
                    TokenType.BOUNDED: MemoryType.BOUNDED,
                    TokenType.UNBOUNDED: MemoryType.UNBOUNDED,
                    TokenType.STACK_ONLY: MemoryType.STACK_ONLY,
                }
                memory_token = self.expect(*memory_map.keys())
                memory = memory_map[memory_token.type]
                if memory == MemoryType.BOUNDED and self.match(TokenType.NUMBER):
                    memory_limit = self.advance().value
            elif self.match(TokenType.ISOLATION):
                self.advance()
                isolation_map = {
                    TokenType.NONE: IsolationType.NONE,
                    TokenType.THREAD: IsolationType.THREAD,
                    TokenType.PROCESS: IsolationType.PROCESS,
                    TokenType.CONTAINER: IsolationType.CONTAINER,
                }
                isolation_token = self.expect(*isolation_map.keys())
                isolation = isolation_map[isolation_token.type]
            elif self.match(TokenType.CACHE):
                self.advance()
                cache_map = {
                    TokenType.NONE: CacheType.NONE,
                    TokenType.LRU: CacheType.LRU,
                    TokenType.TTL: CacheType.TTL,
                }
                cache_token = self.expect(*cache_map.keys())
                cache = cache_map[cache_token.type]
                if cache in {CacheType.LRU, CacheType.TTL} and self.match(TokenType.NUMBER):
                    cache_size = int(self.advance().value)
            else:
                # 알 수 없는 토큰 스킵
                self.advance()

        self.expect(TokenType.ENDEXECUTION)
        return ExecutionBlock(parallel=parallel, target=target, memory=memory,
                             memory_limit=memory_limit, isolation=isolation,
                             cache=cache, cache_size=cache_size,
                             line=token.line, column=token.column)

    def parse_verify_block(self) -> VerifyBlock:
        """VERIFY 블록 파싱"""
        token = self.expect(TokenType.VERIFY)
        entries = []

        verify_type_map = {
            TokenType.ASSERT: VerifyType.ASSERT,
            TokenType.PROPERTY: VerifyType.PROPERTY,
            TokenType.INVARIANT: VerifyType.INVARIANT,
            TokenType.POSTCONDITION: VerifyType.POSTCONDITION,
            TokenType.TEST: VerifyType.TEST,
        }

        while not self.match(TokenType.ENDVERIFY):
            if self.current().type in verify_type_map:
                verify_type_token = self.advance()
                verify_type = verify_type_map[verify_type_token.type]

                if verify_type == VerifyType.TEST:
                    # TEST @reference
                    test_ref = self.expect(TokenType.EXTERNAL_REF).value
                    entries.append(VerifyEntry(
                        verify_type=verify_type,
                        test_ref=test_ref,
                        line=verify_type_token.line, column=verify_type_token.column
                    ))
                else:
                    # 표현식
                    expr = self.parse_expression()
                    entries.append(VerifyEntry(
                        verify_type=verify_type,
                        expression=expr,
                        line=verify_type_token.line, column=verify_type_token.column
                    ))
            else:
                # 알 수 없는 토큰 스킵
                self.advance()

        self.expect(TokenType.ENDVERIFY)
        return VerifyBlock(entries=entries, line=token.line, column=token.column)

    # =========================================================================
    # 메인 파싱
    # =========================================================================

    def parse(self) -> VaisUnit:
        """전체 Vais 유닛 파싱"""
        unit = self.parse_unit_block()
        meta = self.parse_meta_block()
        input_block = self.parse_input_block()
        output_block = self.parse_output_block()
        intent = self.parse_intent_block()
        constraint = self.parse_constraint_block()
        flow = self.parse_flow_block()
        execution = self.parse_execution_block()
        verify = self.parse_verify_block()
        self.expect(TokenType.END)

        return VaisUnit(
            unit=unit, meta=meta, input=input_block, output=output_block,
            intent=intent, constraint=constraint, flow=flow,
            execution=execution, verify=verify,
            line=unit.line, column=unit.column
        )


def parse(source: str) -> VaisUnit:
    """소스 코드 파싱 헬퍼 함수"""
    tokens = tokenize(source)
    parser = Parser(tokens)
    return parser.parse()


if __name__ == "__main__":
    # 테스트
    test_source = '''
UNIT FUNCTION examples.add_numbers V1.0.0

META
  DOMAIN examples.math
  DETERMINISM true
  PURE true
ENDMETA

INPUT
  a : INT32
  b : INT32
ENDINPUT

OUTPUT
  sum : INT32
ENDOUTPUT

INTENT
  GOAL TRANSFORM: input.a, input.b -> output.sum
  PRIORITY CORRECTNESS
ENDINTENT

CONSTRAINT
  REQUIRE input.a >= -1000000 AND input.a <= 1000000
  REQUIRE input.b >= -1000000 AND input.b <= 1000000
ENDCONSTRAINT

FLOW
  NODE add : TRANSFORM (op=ADD, left=input.a, right=input.b)

  EDGE INPUT.a -> add
  EDGE INPUT.b -> add
  EDGE add     -> OUTPUT.sum
ENDFLOW

EXECUTION
  PARALLEL false
  TARGET ANY
  MEMORY STACK_ONLY
ENDEXECUTION

VERIFY
  ASSERT output.sum == input.a + input.b
  TEST @tests.math.add_positive
ENDVERIFY

END
'''

    try:
        ast = parse(test_source)
        print(f"✅ 파싱 성공: {ast}")
        print(f"  Unit: {ast.unit.unit_type.value} {ast.unit.unit_id.full_name}")
        print(f"  Meta entries: {len(ast.meta.entries)}")
        print(f"  Input fields: {len(ast.input.entries)}")
        print(f"  Output fields: {len(ast.output.entries)}")
        print(f"  Intent: GOAL {ast.intent.goal_type.value}")
        print(f"  Constraints: {len(ast.constraint.constraints)}")
        print(f"  Flow nodes: {len(ast.flow.nodes)}")
        print(f"  Flow edges: {len(ast.flow.edges)}")
        print(f"  Verify entries: {len(ast.verify.entries)}")
    except ParseError as e:
        print(f"❌ 파싱 실패: {e}")
