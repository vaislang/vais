#!/usr/bin/env python3
"""
Vais Parser 테스트 스크립트
예제 파일들을 파싱하고 검증 결과를 출력
"""

import sys
import os
from pathlib import Path

# 현재 디렉토리를 path에 추가
sys.path.insert(0, str(Path(__file__).parent))

from lexer import tokenize, TokenType
from parser import parse, ParseError
from validator import validate, ErrorSeverity
from ast_nodes import ASTPrinter


def test_lexer(source: str, name: str):
    """렉서 테스트"""
    print(f"\n{'='*60}")
    print(f"🔤 Lexer Test: {name}")
    print('='*60)

    tokens = tokenize(source)
    non_newline = [t for t in tokens if t.type != TokenType.NEWLINE]

    print(f"Total tokens: {len(tokens)} (non-newline: {len(non_newline)})")
    print(f"First 20 tokens:")
    for i, token in enumerate(non_newline[:20]):
        print(f"  {i+1:3}. {token}")
    if len(non_newline) > 20:
        print(f"  ... and {len(non_newline) - 20} more")

    return tokens


def test_parser(source: str, name: str):
    """파서 테스트"""
    print(f"\n{'='*60}")
    print(f"🌳 Parser Test: {name}")
    print('='*60)

    try:
        ast = parse(source)
        print(f"✅ 파싱 성공!")
        print(f"\n📋 AST 요약:")
        print(f"  UNIT: {ast.unit.unit_type.value} {ast.unit.unit_id.full_name} {ast.unit.version or ''}")
        print(f"  META: {len(ast.meta.entries)} entries")
        for entry in ast.meta.entries:
            print(f"    - {entry.key}: {entry.value}")
        print(f"  INPUT: {len(ast.input.entries)} fields")
        for entry in ast.input.entries:
            print(f"    - {entry.name}: {entry.type_node}")
        print(f"  OUTPUT: {len(ast.output.entries)} fields")
        for entry in ast.output.entries:
            print(f"    - {entry.name}: {entry.type_node}")
        print(f"  INTENT: GOAL {ast.intent.goal_type.value}")
        if ast.intent.priorities:
            print(f"    PRIORITY: {' > '.join(p.value for p in ast.intent.priorities)}")
        print(f"  CONSTRAINT: {len(ast.constraint.constraints)} constraints")
        print(f"  FLOW: {len(ast.flow.nodes)} nodes, {len(ast.flow.edges)} edges")
        for node in ast.flow.nodes:
            params_str = ", ".join(f"{p.name}=..." for p in node.params)
            print(f"    - NODE {node.node_id}: {node.op_type.value}({params_str})")
        print(f"  EXECUTION: parallel={ast.execution.parallel}, target={ast.execution.target.value}")
        print(f"  VERIFY: {len(ast.verify.entries)} entries")

        return ast
    except ParseError as e:
        print(f"❌ 파싱 실패: {e}")
        return None


def test_validator(ast, name: str):
    """검증기 테스트"""
    print(f"\n{'='*60}")
    print(f"✓ Validator Test: {name}")
    print('='*60)

    if not ast:
        print("❌ AST가 없어 검증 스킵")
        return

    errors = validate(ast)

    error_count = sum(1 for e in errors if e.severity == ErrorSeverity.ERROR)
    warning_count = sum(1 for e in errors if e.severity == ErrorSeverity.WARNING)
    info_count = sum(1 for e in errors if e.severity == ErrorSeverity.INFO)

    if errors:
        print(f"검증 결과: {error_count} errors, {warning_count} warnings, {info_count} info")
        for error in errors:
            icon = "❌" if error.severity == ErrorSeverity.ERROR else "⚠️" if error.severity == ErrorSeverity.WARNING else "ℹ️"
            print(f"  {icon} {error}")
    else:
        print("✅ 검증 통과! 이슈 없음")

    return errors


def test_file(filepath: str):
    """파일 테스트"""
    name = Path(filepath).name
    print(f"\n{'#'*70}")
    print(f"# Testing: {name}")
    print('#'*70)

    try:
        with open(filepath, 'r') as f:
            source = f.read()

        # test_lexer(source, name)  # 토큰 출력은 너무 길어서 스킵
        ast = test_parser(source, name)
        test_validator(ast, name)

        return ast is not None

    except FileNotFoundError:
        print(f"❌ 파일 없음: {filepath}")
        return False
    except Exception as e:
        print(f"❌ 에러: {e}")
        import traceback
        traceback.print_exc()
        return False


def main():
    """메인 테스트 실행"""
    print("="*70)
    print("🚀 Vais Parser Prototype Test Suite")
    print("="*70)

    # 예제 디렉토리 찾기
    script_dir = Path(__file__).parent
    examples_dir = script_dir.parent / "examples"

    if not examples_dir.exists():
        print(f"❌ 예제 디렉토리 없음: {examples_dir}")
        return

    # 모든 .vais 파일 테스트
    vais_files = sorted(examples_dir.glob("*.vais"))

    if not vais_files:
        print(f"❌ .vais 파일 없음: {examples_dir}")
        return

    print(f"\n📁 발견된 예제 파일: {len(vais_files)}개")
    for f in vais_files:
        print(f"  - {f.name}")

    # 테스트 실행
    results = {}
    for filepath in vais_files:
        success = test_file(str(filepath))
        results[filepath.name] = success

    # 결과 요약
    print("\n" + "="*70)
    print("📊 테스트 결과 요약")
    print("="*70)

    passed = sum(1 for v in results.values() if v)
    failed = len(results) - passed

    for name, success in results.items():
        icon = "✅" if success else "❌"
        print(f"  {icon} {name}")

    print(f"\n총 {len(results)}개 중 {passed}개 성공, {failed}개 실패")

    if failed > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
