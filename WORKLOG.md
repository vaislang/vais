# nl WORKLOG

## 2026-06-06 (/loop iter 1: P0 — 값-정확성 게이트)
- 폴더 구조를 언어 전체로 보강 (std/ tools/ docs/reference/ tests/).
- ROADMAP.md 신설 (단일 진실원, P0~P5 우선순위 + L3 진입 + TRACKED).
- **G1 값-정확성 러너** scripts/test.sh: examples/*.nl `# expect: N` → 트랜스파일+빌드+실행+exit 비교.
- **G2** 실행형 예제 11개에 expect 주석 부착.
- baseline: **11/11 PASS** (filter d6 등 Vais버그/no-main은 expect 없어 제외).
- 의미: 이제 모든 후속 작업이 러너 green으로 보호됨 (P7b 컴파일≠정답 인프라).

## 2026-06-06 (/loop iter 2: P1 — 예제 코퍼스 2배 확장)
- C1: 10개 신규 예제 (e01~e10): 중첩struct/enum-payload/재귀/상호재귀/다중함수/for/else-if/Option/
  메서드체인/bool논리. 전부 # expect + 러너 PASS.
- C2: examples/README.md — 문법 커버리지 인덱스 (21개 예제, 미커버 항목 명시).
- 코퍼스 11→21 (2배). **전체 21/21 PASS, 회귀 0.**
- P9 인프라 강화: AI cold-start 학습용 검증 예제 집합.

## 2026-06-06 (/loop iter 3: P2 — 트랜스파일러 견고화)
- T1: while 루프 → Vais `L { I !(cond) { B } ... }`. e11_while PASS.
- T2: exclusive range `..` → `I i >= hi { B }` (inclusive `..=`와 분리). e12 PASS.
- T3: tests/transpiler_test.py — 입력 nl→기대 Vais 출력 단위테스트 19/19 (트랜스파일러 회귀 방지).
- T4: nested for 견고성 — e13_nested_for PASS (중첩 idx 카운터 충돌 없음).
- 값-정확성 23/23 + 트랜스파일러 단위 19/19 둘 다 green. 트랜스파일러 미지원 구문 0.
