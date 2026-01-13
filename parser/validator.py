"""
Vais Validator - 의미 검증기
Vibe AI SSW Language Parser v0.2
"""

from dataclasses import dataclass, field
from typing import List, Dict, Set, Optional, Any
from enum import Enum, auto

from ast_nodes import (
    VaisUnit, ASTNode, ASTVisitor,
    UnitBlock, MetaBlock, InputBlock, OutputBlock,
    IntentBlock, ConstraintBlock, FlowBlock, ExecutionBlock, VerifyBlock,
    FlowNode, FlowEdge, InputEntry, OutputEntry,
    TypeNode, PrimitiveType, ArrayType, MapType, StructType, OptionalType, UnionType,
    Expression, Literal, Identifier, QualifiedName, ExternalRef, FieldAccess, BinaryOp,
    OpType
)


class ErrorSeverity(Enum):
    ERROR = "ERROR"
    WARNING = "WARNING"
    INFO = "INFO"


@dataclass
class ValidationError:
    """검증 에러"""
    code: str
    message: str
    severity: ErrorSeverity
    line: int
    column: int
    node: Optional[ASTNode] = None

    def __str__(self):
        return f"[{self.severity.value}] {self.code} at L{self.line}:C{self.column}: {self.message}"


class Validator(ASTVisitor):
    """Vais 의미 검증기"""

    def __init__(self):
        self.errors: List[ValidationError] = []
        self.input_fields: Dict[str, TypeNode] = {}
        self.output_fields: Dict[str, TypeNode] = {}
        self.flow_nodes: Dict[str, FlowNode] = {}
        self.flow_edges: List[FlowEdge] = []
        self.external_refs: Set[str] = set()

    def add_error(self, code: str, message: str, node: ASTNode,
                  severity: ErrorSeverity = ErrorSeverity.ERROR):
        """에러 추가"""
        self.errors.append(ValidationError(
            code=code,
            message=message,
            severity=severity,
            line=node.line,
            column=node.column,
            node=node
        ))

    def validate(self, unit: VaisUnit) -> List[ValidationError]:
        """전체 유닛 검증"""
        self.errors = []

        # 1. 블록 존재 검증
        self._validate_blocks_exist(unit)

        # 2. 필드 수집
        self._collect_fields(unit)

        # 3. 각 블록 검증
        self._validate_meta(unit.meta)
        self._validate_input(unit.input)
        self._validate_output(unit.output)
        self._validate_intent(unit.intent)
        self._validate_constraint(unit.constraint)
        self._validate_flow(unit.flow)
        self._validate_execution(unit.execution)
        self._validate_verify(unit.verify)

        # 4. 교차 블록 검증
        self._validate_cross_references(unit)

        return self.errors

    def _validate_blocks_exist(self, unit: VaisUnit):
        """모든 필수 블록 존재 확인"""
        if not unit.unit:
            self.add_error("E1001", "Missing UNIT block", unit)
        if not unit.meta:
            self.add_error("E1002", "Missing META block", unit)
        if not unit.input:
            self.add_error("E1003", "Missing INPUT block", unit)
        if not unit.output:
            self.add_error("E1004", "Missing OUTPUT block", unit)
        if not unit.intent:
            self.add_error("E1005", "Missing INTENT block", unit)
        if not unit.constraint:
            self.add_error("E1006", "Missing CONSTRAINT block", unit)
        if not unit.flow:
            self.add_error("E1007", "Missing FLOW block", unit)
        if not unit.execution:
            self.add_error("E1008", "Missing EXECUTION block", unit)
        if not unit.verify:
            self.add_error("E1009", "Missing VERIFY block", unit)

    def _collect_fields(self, unit: VaisUnit):
        """입력/출력 필드 수집"""
        if unit.input:
            for entry in unit.input.entries:
                if entry.name in self.input_fields:
                    self.add_error("E2001", f"Duplicate input field: {entry.name}", entry)
                self.input_fields[entry.name] = entry.type_node

        if unit.output:
            for entry in unit.output.entries:
                if entry.name in self.output_fields:
                    self.add_error("E2002", f"Duplicate output field: {entry.name}", entry)
                self.output_fields[entry.name] = entry.type_node

        if unit.flow:
            for node in unit.flow.nodes:
                if node.node_id in self.flow_nodes:
                    self.add_error("E4001", f"Duplicate flow node: {node.node_id}", node)
                self.flow_nodes[node.node_id] = node
            self.flow_edges = unit.flow.edges

    def _validate_meta(self, meta: MetaBlock):
        """META 블록 검증"""
        if not meta:
            return

        required_keys = {"DOMAIN", "DETERMINISM"}
        found_keys = {entry.key for entry in meta.entries}

        for key in required_keys:
            if key not in found_keys:
                self.add_error("E2010", f"Missing required META entry: {key}", meta,
                              ErrorSeverity.WARNING)

        # DETERMINISM은 boolean이어야 함
        for entry in meta.entries:
            if entry.key == "DETERMINISM" and not isinstance(entry.value, bool):
                self.add_error("E2011", "DETERMINISM must be boolean (true/false)", entry)

    def _validate_input(self, input_block: InputBlock):
        """INPUT 블록 검증"""
        if not input_block:
            return

        for entry in input_block.entries:
            self._validate_type(entry.type_node, entry)

    def _validate_output(self, output_block: OutputBlock):
        """OUTPUT 블록 검증"""
        if not output_block:
            return

        for entry in output_block.entries:
            self._validate_type(entry.type_node, entry)

    def _validate_type(self, type_node: TypeNode, parent: ASTNode):
        """타입 검증"""
        if isinstance(type_node, ArrayType):
            self._validate_type(type_node.element_type, parent)
        elif isinstance(type_node, MapType):
            self._validate_type(type_node.key_type, parent)
            self._validate_type(type_node.value_type, parent)
        elif isinstance(type_node, StructType):
            for field_type in type_node.fields.values():
                self._validate_type(field_type, parent)
        elif isinstance(type_node, OptionalType):
            self._validate_type(type_node.inner_type, parent)
        elif isinstance(type_node, UnionType):
            for t in type_node.types:
                self._validate_type(t, parent)

    def _validate_intent(self, intent: IntentBlock):
        """INTENT 블록 검증"""
        if not intent:
            return

        # GOAL 타입 검증
        if not intent.goal_type:
            self.add_error("E3001", "Missing GOAL type in INTENT", intent)

        # GOAL 명세 검증
        if intent.goal_spec:
            if not intent.goal_spec.inputs:
                self.add_error("E3002", "GOAL must have at least one input", intent)
            if not intent.goal_spec.outputs:
                self.add_error("E3003", "GOAL must have at least one output", intent)

    def _validate_constraint(self, constraint: ConstraintBlock):
        """CONSTRAINT 블록 검증"""
        if not constraint:
            return

        # 각 제약조건의 표현식에서 참조 검증
        for c in constraint.constraints:
            self._validate_expression_refs(c.expression, c)

    def _validate_flow(self, flow: FlowBlock):
        """FLOW 블록 검증"""
        if not flow:
            return

        # 노드 검증
        for node in flow.nodes:
            self._validate_flow_node(node)

        # 엣지 검증
        for edge in flow.edges:
            self._validate_flow_edge(edge)

        # 그래프 연결성 검증
        self._validate_flow_connectivity(flow)

    def _validate_flow_node(self, node: FlowNode):
        """FLOW 노드 검증"""
        # 연산 타입별 필수 파라미터 검증
        required_params = {
            OpType.MAP: ["fn"],
            OpType.FILTER: ["condition"],
            OpType.REDUCE: ["fn"],
            OpType.FETCH: ["source"],
            OpType.STORE: ["target"],
            OpType.CALL: [],
            OpType.BRANCH: [],
            OpType.TRANSFORM: [],
        }

        if node.op_type in required_params:
            param_names = {p.name for p in node.params}
            for required in required_params.get(node.op_type, []):
                if required not in param_names:
                    self.add_error("E4010", f"Node '{node.node_id}' missing required param: {required}",
                                  node, ErrorSeverity.WARNING)

    def _validate_flow_edge(self, edge: FlowEdge):
        """FLOW 엣지 검증"""
        # 소스 검증
        source_ref = self._get_node_ref(edge.source)
        if source_ref:
            base, port = source_ref
            if base == "INPUT":
                if port and port not in self.input_fields:
                    self.add_error("E4020", f"Unknown input field: {port}", edge)
            elif base not in self.flow_nodes and base != "INPUT":
                self.add_error("E4021", f"Unknown source node: {base}", edge)

        # 타겟 검증
        target_ref = self._get_node_ref(edge.target)
        if target_ref:
            base, port = target_ref
            if base == "OUTPUT":
                if port and port not in self.output_fields:
                    self.add_error("E4022", f"Unknown output field: {port}", edge)
            elif base not in self.flow_nodes and base != "OUTPUT":
                self.add_error("E4023", f"Unknown target node: {base}", edge)

    def _get_node_ref(self, expr: Expression) -> Optional[tuple]:
        """표현식에서 노드 참조 추출 (base, port)"""
        if isinstance(expr, Identifier):
            return (expr.name, None)
        elif isinstance(expr, FieldAccess):
            if isinstance(expr.base, Identifier):
                return (expr.base.name, expr.field)
        return None

    def _validate_flow_connectivity(self, flow: FlowBlock):
        """FLOW 그래프 연결성 검증"""
        if not flow.nodes:
            return

        # 모든 노드가 연결되어 있는지 확인
        connected_nodes: Set[str] = set()

        for edge in flow.edges:
            source_ref = self._get_node_ref(edge.source)
            target_ref = self._get_node_ref(edge.target)

            if source_ref:
                connected_nodes.add(source_ref[0])
            if target_ref:
                connected_nodes.add(target_ref[0])

        # INPUT, OUTPUT 제외
        connected_nodes.discard("INPUT")
        connected_nodes.discard("OUTPUT")

        for node_id, node in self.flow_nodes.items():
            if node_id not in connected_nodes:
                self.add_error("E4030", f"Node '{node_id}' is not connected to any edge",
                              node, ErrorSeverity.WARNING)

    def _validate_execution(self, execution: ExecutionBlock):
        """EXECUTION 블록 검증"""
        if not execution:
            return

        # BOUNDED 메모리에는 limit이 필요
        from ast_nodes import MemoryType
        if execution.memory == MemoryType.BOUNDED and not execution.memory_limit:
            self.add_error("E6001", "BOUNDED memory requires a limit (e.g., 256MB)",
                          execution, ErrorSeverity.WARNING)

    def _validate_verify(self, verify: VerifyBlock):
        """VERIFY 블록 검증"""
        if not verify:
            return

        from ast_nodes import VerifyType
        for entry in verify.entries:
            if entry.verify_type == VerifyType.TEST:
                if not entry.test_ref:
                    self.add_error("E7001", "TEST entry requires a reference", entry)
                else:
                    self.external_refs.add(entry.test_ref)
            elif not entry.expression:
                self.add_error("E7002", f"{entry.verify_type.value} entry requires an expression", entry)

    def _validate_expression_refs(self, expr: Expression, parent: ASTNode):
        """표현식 내 참조 검증"""
        if not expr:
            return

        if isinstance(expr, FieldAccess):
            if isinstance(expr.base, Identifier):
                base = expr.base.name
                if base == "input":
                    if expr.field not in self.input_fields:
                        self.add_error("E5001", f"Unknown input field: {expr.field}", parent)
                elif base == "output":
                    if expr.field not in self.output_fields:
                        self.add_error("E5002", f"Unknown output field: {expr.field}", parent)
        elif isinstance(expr, ExternalRef):
            self.external_refs.add(expr.ref)
        elif isinstance(expr, BinaryOp):
            self._validate_expression_refs(expr.left, parent)
            self._validate_expression_refs(expr.right, parent)

    def _validate_cross_references(self, unit: VaisUnit):
        """교차 블록 참조 검증"""
        # INTENT의 입출력 참조가 실제 필드와 일치하는지
        if unit.intent and unit.intent.goal_spec:
            for input_ref in unit.intent.goal_spec.inputs:
                self._validate_expression_refs(input_ref, unit.intent)
            for output_ref in unit.intent.goal_spec.outputs:
                self._validate_expression_refs(output_ref, unit.intent)


def validate(unit: VaisUnit) -> List[ValidationError]:
    """유닛 검증 헬퍼 함수"""
    validator = Validator()
    return validator.validate(unit)


# =============================================================================
# 에러 코드 정의
# =============================================================================

ERROR_CODES = {
    # E1xxx: 구조 에러
    "E1001": "Missing UNIT block",
    "E1002": "Missing META block",
    "E1003": "Missing INPUT block",
    "E1004": "Missing OUTPUT block",
    "E1005": "Missing INTENT block",
    "E1006": "Missing CONSTRAINT block",
    "E1007": "Missing FLOW block",
    "E1008": "Missing EXECUTION block",
    "E1009": "Missing VERIFY block",

    # E2xxx: 타입/필드 에러
    "E2001": "Duplicate input field",
    "E2002": "Duplicate output field",
    "E2010": "Missing required META entry",
    "E2011": "Invalid META value type",

    # E3xxx: INTENT 에러
    "E3001": "Missing GOAL type",
    "E3002": "Missing GOAL inputs",
    "E3003": "Missing GOAL outputs",

    # E4xxx: FLOW 에러
    "E4001": "Duplicate flow node",
    "E4010": "Missing required node parameter",
    "E4020": "Unknown input field in edge",
    "E4021": "Unknown source node in edge",
    "E4022": "Unknown output field in edge",
    "E4023": "Unknown target node in edge",
    "E4030": "Disconnected node",

    # E5xxx: 참조 에러
    "E5001": "Unknown input field reference",
    "E5002": "Unknown output field reference",

    # E6xxx: EXECUTION 에러
    "E6001": "Missing memory limit for BOUNDED",

    # E7xxx: VERIFY 에러
    "E7001": "Missing test reference",
    "E7002": "Missing verify expression",
}


if __name__ == "__main__":
    from parser import parse

    test_source = '''
UNIT FUNCTION examples.test V1.0.0

META
  DOMAIN examples.test
  DETERMINISM true
ENDMETA

INPUT
  x : INT32
ENDINPUT

OUTPUT
  y : INT32
ENDOUTPUT

INTENT
  GOAL TRANSFORM: input.x -> output.y
  PRIORITY CORRECTNESS
ENDINTENT

CONSTRAINT
  REQUIRE input.x > 0
  REQUIRE input.unknown_field > 0
ENDCONSTRAINT

FLOW
  NODE transform : TRANSFORM (op=ADD)
  NODE unused    : MAP (fn=identity)

  EDGE INPUT.x   -> transform
  EDGE transform -> OUTPUT.y
ENDFLOW

EXECUTION
  PARALLEL false
  TARGET ANY
  MEMORY BOUNDED
ENDEXECUTION

VERIFY
  ASSERT output.y > 0
  TEST @tests.example
ENDVERIFY

END
'''

    try:
        ast = parse(test_source)
        errors = validate(ast)

        print(f"검증 결과: {len(errors)} 개 이슈 발견\n")
        for error in errors:
            print(f"  {error}")
    except Exception as e:
        print(f"에러: {e}")
