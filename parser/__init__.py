"""
AOEL Parser Package
AI-Optimized Executable Language Parser v0.2
"""

from .lexer import Lexer, Token, TokenType, tokenize
from .ast_nodes import (
    AOELUnit, ASTNode, ASTVisitor, ASTPrinter,
    UnitBlock, MetaBlock, InputBlock, OutputBlock,
    IntentBlock, ConstraintBlock, FlowBlock, ExecutionBlock, VerifyBlock,
    FlowNode, FlowEdge, NodeParam,
    TypeNode, PrimitiveType, ArrayType, MapType, StructType, OptionalType, UnionType,
    Expression, Literal, Identifier, QualifiedName, ExternalRef, FieldAccess, BinaryOp,
    UnitType, GoalType, PriorityType, FailureStrategy, ConstraintType,
    OpType, TargetType, MemoryType, IsolationType, CacheType, VerifyType
)
from .parser import Parser, ParseError, parse
from .validator import Validator, ValidationError, ErrorSeverity, validate

__version__ = "0.2.0"
__all__ = [
    # Lexer
    "Lexer", "Token", "TokenType", "tokenize",
    # AST
    "AOELUnit", "ASTNode", "ASTVisitor", "ASTPrinter",
    "UnitBlock", "MetaBlock", "InputBlock", "OutputBlock",
    "IntentBlock", "ConstraintBlock", "FlowBlock", "ExecutionBlock", "VerifyBlock",
    "FlowNode", "FlowEdge", "NodeParam",
    "TypeNode", "PrimitiveType", "ArrayType", "MapType", "StructType", "OptionalType", "UnionType",
    "Expression", "Literal", "Identifier", "QualifiedName", "ExternalRef", "FieldAccess", "BinaryOp",
    "UnitType", "GoalType", "PriorityType", "FailureStrategy", "ConstraintType",
    "OpType", "TargetType", "MemoryType", "IsolationType", "CacheType", "VerifyType",
    # Parser
    "Parser", "ParseError", "parse",
    # Validator
    "Validator", "ValidationError", "ErrorSeverity", "validate",
]
