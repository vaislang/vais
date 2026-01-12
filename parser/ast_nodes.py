"""
AOEL AST Nodes - 추상 구문 트리 노드 정의
AI-Optimized Executable Language Parser v0.2
"""

from dataclasses import dataclass, field
from typing import List, Optional, Dict, Any, Union
from enum import Enum, auto


# =============================================================================
# 기본 타입 정의
# =============================================================================

class UnitType(Enum):
    FUNCTION = "FUNCTION"
    SERVICE = "SERVICE"
    PIPELINE = "PIPELINE"
    MODULE = "MODULE"


class GoalType(Enum):
    TRANSFORM = "TRANSFORM"
    VALIDATE = "VALIDATE"
    AGGREGATE = "AGGREGATE"
    FILTER = "FILTER"
    ROUTE = "ROUTE"
    COMPOSE = "COMPOSE"
    FETCH = "FETCH"


class PriorityType(Enum):
    CORRECTNESS = "CORRECTNESS"
    PERFORMANCE = "PERFORMANCE"
    MEMORY = "MEMORY"
    LATENCY = "LATENCY"
    THROUGHPUT = "THROUGHPUT"


class FailureStrategy(Enum):
    ABORT = "ABORT"
    RETRY = "RETRY"
    FALLBACK = "FALLBACK"
    DEFAULT = "DEFAULT"


class ConstraintType(Enum):
    REQUIRE = "REQUIRE"
    FORBID = "FORBID"
    PREFER = "PREFER"
    INVARIANT = "INVARIANT"


class OpType(Enum):
    # 데이터 변환
    MAP = "MAP"
    FILTER = "FILTER"
    REDUCE = "REDUCE"
    TRANSFORM = "TRANSFORM"
    FLATTEN = "FLATTEN"
    GROUP = "GROUP"
    SORT = "SORT"
    DISTINCT = "DISTINCT"
    # 흐름 제어
    BRANCH = "BRANCH"
    MERGE = "MERGE"
    SPLIT = "SPLIT"
    JOIN = "JOIN"
    RACE = "RACE"
    # 외부 연산
    FETCH = "FETCH"
    STORE = "STORE"
    CALL = "CALL"
    EMIT = "EMIT"
    SUBSCRIBE = "SUBSCRIBE"
    # 검증
    VALIDATE = "VALIDATE"
    SANITIZE = "SANITIZE"
    AUTHORIZE = "AUTHORIZE"


class TargetType(Enum):
    ANY = "ANY"
    CPU = "CPU"
    GPU = "GPU"
    WASM = "WASM"
    NATIVE = "NATIVE"


class MemoryType(Enum):
    BOUNDED = "BOUNDED"
    UNBOUNDED = "UNBOUNDED"
    STACK_ONLY = "STACK_ONLY"


class IsolationType(Enum):
    NONE = "NONE"
    THREAD = "THREAD"
    PROCESS = "PROCESS"
    CONTAINER = "CONTAINER"


class CacheType(Enum):
    NONE = "NONE"
    LRU = "LRU"
    TTL = "TTL"


class VerifyType(Enum):
    ASSERT = "ASSERT"
    PROPERTY = "PROPERTY"
    INVARIANT = "INVARIANT"
    POSTCONDITION = "POSTCONDITION"
    TEST = "TEST"


# =============================================================================
# AST 기본 노드
# =============================================================================

@dataclass
class ASTNode:
    """AST 기본 노드"""
    line: int = 0
    column: int = 0


@dataclass
class SourceLocation:
    """소스 코드 위치 정보"""
    line: int
    column: int
    end_line: Optional[int] = None
    end_column: Optional[int] = None


# =============================================================================
# 타입 노드
# =============================================================================

@dataclass
class TypeNode(ASTNode):
    """타입 기본 노드"""
    pass


@dataclass
class PrimitiveType(TypeNode):
    """원시 타입"""
    name: str = ""  # INT32, STRING, BOOL, etc.


@dataclass
class ArrayType(TypeNode):
    """배열 타입"""
    element_type: Optional['TypeNode'] = None


@dataclass
class MapType(TypeNode):
    """맵 타입"""
    key_type: Optional['TypeNode'] = None
    value_type: Optional['TypeNode'] = None


@dataclass
class StructType(TypeNode):
    """구조체 타입"""
    fields: Dict[str, 'TypeNode'] = field(default_factory=dict)


@dataclass
class OptionalType(TypeNode):
    """옵셔널 타입"""
    inner_type: Optional['TypeNode'] = None


@dataclass
class UnionType(TypeNode):
    """유니온 타입"""
    types: List['TypeNode'] = field(default_factory=list)


@dataclass
class TypeRef(TypeNode):
    """타입 참조 (@로 시작)"""
    ref: str = ""


# =============================================================================
# 표현식 노드
# =============================================================================

@dataclass
class Expression(ASTNode):
    """표현식 기본 노드"""
    pass


@dataclass
class Literal(Expression):
    """리터럴 값"""
    value: Any = None
    literal_type: str = ""  # "number", "float", "string", "bool", "regex"


@dataclass
class Identifier(Expression):
    """식별자"""
    name: str = ""


@dataclass
class QualifiedName(Expression):
    """정규화된 이름 (dot 구분)"""
    parts: List[str] = field(default_factory=list)

    @property
    def full_name(self) -> str:
        return ".".join(self.parts)


@dataclass
class ExternalRef(Expression):
    """외부 참조 (@로 시작)"""
    ref: str = ""


@dataclass
class FieldAccess(Expression):
    """필드 접근"""
    base: Optional['Expression'] = None
    field_name: str = ""

    @property
    def field(self) -> str:
        return self.field_name


@dataclass
class BinaryOp(Expression):
    """이항 연산"""
    left: Optional['Expression'] = None
    operator: str = ""  # +, -, *, /, ==, !=, <, >, <=, >=, AND, OR, etc.
    right: Optional['Expression'] = None


@dataclass
class UnaryOp(Expression):
    """단항 연산"""
    operator: str = ""  # NOT, -
    operand: Optional['Expression'] = None


@dataclass
class FunctionCall(Expression):
    """함수 호출"""
    name: str = ""
    arguments: List['Expression'] = field(default_factory=list)


@dataclass
class ArrayAccess(Expression):
    """배열 인덱스 접근"""
    base: Optional['Expression'] = None
    index: Optional['Expression'] = None


# =============================================================================
# 제약 조건 노드
# =============================================================================

@dataclass
class Constraint(ASTNode):
    """제약 조건"""
    constraint_type: Optional[ConstraintType] = None
    expression: Optional['Expression'] = None


@dataclass
class InputConstraint(ASTNode):
    """입력 필드 제약"""
    expression: Optional['Expression'] = None


# =============================================================================
# 블록 노드
# =============================================================================

@dataclass
class UnitBlock(ASTNode):
    """UNIT 블록"""
    unit_type: Optional[UnitType] = None
    unit_id: Optional['QualifiedName'] = None
    version: Optional[str] = None


@dataclass
class MetaEntry(ASTNode):
    """META 엔트리"""
    key: str = ""
    value: Any = None


@dataclass
class MetaBlock(ASTNode):
    """META 블록"""
    entries: List[MetaEntry] = field(default_factory=list)


@dataclass
class InputEntry(ASTNode):
    """INPUT 엔트리"""
    name: str = ""
    type_node: Optional['TypeNode'] = None
    constraints: List[InputConstraint] = field(default_factory=list)


@dataclass
class InputBlock(ASTNode):
    """INPUT 블록"""
    entries: List[InputEntry] = field(default_factory=list)


@dataclass
class OutputEntry(ASTNode):
    """OUTPUT 엔트리"""
    name: str = ""
    type_node: Optional['TypeNode'] = None
    constraints: List[InputConstraint] = field(default_factory=list)


@dataclass
class OutputBlock(ASTNode):
    """OUTPUT 블록"""
    entries: List[OutputEntry] = field(default_factory=list)


@dataclass
class GoalSpec(ASTNode):
    """GOAL 명세"""
    inputs: List['Expression'] = field(default_factory=list)
    outputs: List['Expression'] = field(default_factory=list)


@dataclass
class IntentBlock(ASTNode):
    """INTENT 블록"""
    goal_type: Optional[GoalType] = None
    goal_spec: Optional[GoalSpec] = None
    priorities: List[PriorityType] = field(default_factory=list)
    failure_strategy: Optional[FailureStrategy] = None
    failure_ref: Optional['Expression'] = None


@dataclass
class ConstraintBlock(ASTNode):
    """CONSTRAINT 블록"""
    constraints: List[Constraint] = field(default_factory=list)


@dataclass
class NodeParam(ASTNode):
    """노드 파라미터"""
    name: str = ""
    value: Optional['Expression'] = None


@dataclass
class FlowNode(ASTNode):
    """FLOW 노드"""
    node_id: str = ""
    op_type: Optional[OpType] = None
    params: List[NodeParam] = field(default_factory=list)
    custom_op: Optional[str] = None


@dataclass
class FlowEdge(ASTNode):
    """FLOW 엣지"""
    source: Optional['Expression'] = None
    target: Optional['Expression'] = None
    condition: Optional['Expression'] = None


@dataclass
class FlowBlock(ASTNode):
    """FLOW 블록"""
    nodes: List[FlowNode] = field(default_factory=list)
    edges: List[FlowEdge] = field(default_factory=list)


@dataclass
class ExecutionBlock(ASTNode):
    """EXECUTION 블록"""
    parallel: bool = False
    target: TargetType = TargetType.ANY
    memory: MemoryType = MemoryType.UNBOUNDED
    memory_limit: Optional[str] = None
    isolation: IsolationType = IsolationType.NONE
    cache: CacheType = CacheType.NONE
    cache_size: Optional[int] = None


@dataclass
class VerifyEntry(ASTNode):
    """VERIFY 엔트리"""
    verify_type: Optional[VerifyType] = None
    expression: Optional['Expression'] = None
    test_ref: Optional[str] = None


@dataclass
class VerifyBlock(ASTNode):
    """VERIFY 블록"""
    entries: List[VerifyEntry] = field(default_factory=list)


# =============================================================================
# 최상위 노드
# =============================================================================

@dataclass
class AOELUnit(ASTNode):
    """AOEL 유닛 (최상위 노드)"""
    unit: Optional[UnitBlock] = None
    meta: Optional[MetaBlock] = None
    input: Optional[InputBlock] = None
    output: Optional[OutputBlock] = None
    intent: Optional[IntentBlock] = None
    constraint: Optional[ConstraintBlock] = None
    flow: Optional[FlowBlock] = None
    execution: Optional[ExecutionBlock] = None
    verify: Optional[VerifyBlock] = None

    def __repr__(self):
        if self.unit and self.unit.unit_id:
            return f"AOELUnit({self.unit.unit_type.value if self.unit.unit_type else 'UNKNOWN'} {self.unit.unit_id.full_name})"
        return "AOELUnit(UNKNOWN)"


# =============================================================================
# AST 방문자 패턴
# =============================================================================

class ASTVisitor:
    """AST 방문자 기본 클래스"""

    def visit(self, node: ASTNode) -> Any:
        method_name = f"visit_{type(node).__name__}"
        visitor = getattr(self, method_name, self.generic_visit)
        return visitor(node)

    def generic_visit(self, node: ASTNode) -> Any:
        """기본 방문 메서드"""
        return None


class ASTPrinter(ASTVisitor):
    """AST 출력 방문자"""

    def __init__(self):
        self.indent = 0

    def _print(self, text: str):
        print("  " * self.indent + text)

    def visit_AOELUnit(self, node: AOELUnit):
        self._print(f"AOELUnit:")
        self.indent += 1
        if node.unit:
            self.visit(node.unit)
        if node.meta:
            self.visit(node.meta)
        if node.input:
            self.visit(node.input)
        if node.output:
            self.visit(node.output)
        if node.intent:
            self.visit(node.intent)
        if node.constraint:
            self.visit(node.constraint)
        if node.flow:
            self.visit(node.flow)
        if node.execution:
            self.visit(node.execution)
        if node.verify:
            self.visit(node.verify)
        self.indent -= 1

    def visit_UnitBlock(self, node: UnitBlock):
        unit_type = node.unit_type.value if node.unit_type else "UNKNOWN"
        unit_id = node.unit_id.full_name if node.unit_id else "UNKNOWN"
        self._print(f"UNIT {unit_type} {unit_id} {node.version or ''}")

    def visit_MetaBlock(self, node: MetaBlock):
        self._print("META:")
        self.indent += 1
        for entry in node.entries:
            self._print(f"{entry.key}: {entry.value}")
        self.indent -= 1

    def visit_InputBlock(self, node: InputBlock):
        self._print("INPUT:")
        self.indent += 1
        for entry in node.entries:
            self._print(f"{entry.name}: {entry.type_node}")
        self.indent -= 1

    def visit_OutputBlock(self, node: OutputBlock):
        self._print("OUTPUT:")
        self.indent += 1
        for entry in node.entries:
            self._print(f"{entry.name}: {entry.type_node}")
        self.indent -= 1

    def visit_IntentBlock(self, node: IntentBlock):
        goal_type = node.goal_type.value if node.goal_type else "UNKNOWN"
        self._print(f"INTENT: GOAL {goal_type}")

    def visit_ConstraintBlock(self, node: ConstraintBlock):
        self._print(f"CONSTRAINT: {len(node.constraints)} constraints")

    def visit_FlowBlock(self, node: FlowBlock):
        self._print(f"FLOW: {len(node.nodes)} nodes, {len(node.edges)} edges")

    def visit_ExecutionBlock(self, node: ExecutionBlock):
        self._print(f"EXECUTION: parallel={node.parallel}, target={node.target.value}")

    def visit_VerifyBlock(self, node: VerifyBlock):
        self._print(f"VERIFY: {len(node.entries)} entries")
