# Step 10 (A1 hard blocks) findings — first iteration (2026-05-04)

This file records empirical findings from the first iteration of
Order Step 10 (A1 hard blocks). Mirrors STEP7_FINDINGS / STEP11_FINDINGS
structure.

## Index (F-A1-NN → 한 줄 요약)

| ID | 한 줄 요약 |
|---|---|
| F-A1-01 | Master Plan v17 §A1 candidates_count=0 grep 검증 — 4 keyword 후보 발견 |

## Context

Master Plan v17 declares A1 (Hard block — uncertified-unused) with
`candidates_count = 0` + note "Zero confirmed candidates. Every
candidate requires syntax-aware preflight first."

This step's deliverable: per A2 promotion sibling — for each A1 site,
parser/type-check rejection + negative fixture + lightweight enforcement.
Sequencing: only after corresponding A2.

## Findings

### F-A1-01 — Lexer keyword grep finds 4 A1 candidates (2026-05-04)

Master Plan v17 §A1 candidates_count = 0 was verified by lexer
keyword grep against baseline (compiler/std/ + lang/packages/*/src/):

| keyword       | baseline use count | A1 status |
|---------------|--------------------|-----------|
| `effect`      | 0 | **A1 candidate** (LEXER_KEYWORDS.md 명시: "reserved; no grammar production yet") |
| `O` (union)   | 0 | A1 candidate (C-style union declaration, never declared anywhere) |
| `affine`      | 0 | A1 candidate (affine type modifier, never used) |
| `macro`       | 0 | A1 candidate (`macro foo {...}` declaration, never declared) |
| `linear`      | 20 | NOT A1 (uncertified-used; A2 candidate) |
| `partial`     | 119 | NOT A1 (uncertified-used; vaisdb partial fn 광범위 사용) |
| `unsafe`      | 3 | NOT A1 (uncertified-used) |
| `yield`       | 2 | NOT A1 (uncertified-used) |

Grep methodology:
```bash
# word boundaries to avoid false positives
grep -rE "\<<keyword>\>" compiler/std/ lang/packages/ --include='*.vais'
# declaration form check for `O` / `macro` (행 시작)
grep -rEc "^O [A-Z]" ...
grep -rE "^[[:space:]]*macro [a-z]" ...
```

A1 candidates summary:
- `effect`: most clear-cut. lexer_keywords.md 자체에 "no grammar
  production yet" 명시되어 있어 syntax-aware preflight 결과가
  자명. Lexer가 token으로 emit하지만 parser가 아무것도 못 함.
- `O`/`affine`/`macro`: parser 도달 가능성 (grammar production 존재
  여부) 확인 필요. 만약 grammar 있으나 사용처 0이면 A1; 만약 grammar
  자체 미완성이면 effect와 동급 A1.

Recommendation:
- Master Plan v17 §A1 candidates_count = 0 → **4 A1 candidate 등록**
  for next master-plan revision (after v17). syntax-aware preflight
  per candidate 후 A1 hard-block 진행.
- Step 10 sequencing: A1 deliverable는 "parser/type-check rejection +
  negative fixture". `effect`의 경우 parser는 이미 무시하므로 lexer가
  "reserved-but-unused keyword warning" emit하도록 하는 것이 가장
  자연스러운 hard-block 형식.
- 다른 3개는 grammar production 보유 여부를 vais-parser에서 확인 후
  결정 (grammar 있으면 parser-level reject; 없으면 effect와 동일 처리).

Status: Step 10 first iteration RECONNAISSANCE LANDED. 4 A1 candidate
documented. master-plan inventory update + syntax-aware preflight per
candidate is next-iteration scope.

### F-A1-02 — A1 hard-block first iteration (2026-05-05)

Master Plan v23. F-A1-01의 4 candidate를 hard-block으로 진행한 결과:

| keyword | result | notes |
|---|---|---|
| `effect` | A1-LANDED | parser top-level에서 이미 P001 reject. compiler 변경 0, fixture만 (compiler/tests/empirical/A1/A1-01_effect_keyword/). |
| `O` (union) | A1-LANDED | silent accept 발견. parser site item/mod.rs:88 Token::Union arm을 ParseError + 'A1 hard block' marker로 전환. fixture A1-03_O_union_decl/. |
| `macro` | A1-LANDED | silent accept (`macro foo!{...}`) 발견. parser site item/mod.rs:109 Token::Macro arm 동일 패턴. fixture A1-04_macro_decl/. |
| `affine` | A1→A2 RECLASSIFY | hard-block 시도 → INTEGRITY LIVING_SPEC pass=116/117 regression → CLAUDE 규칙 4 revert. baseline 사용 발견: docs/language/LIVING_SPEC/01_keywords/linear_affine_annotation.vais (test_affine_var, x := affine 100). master-plan.toml에 A2-06으로 등록. |

**LESSON L-008 신설**: F-A1-01 grep methodology가 `compiler/std/` + `lang/packages/`만
검사하고 `docs/language/LIVING_SPEC/`을 누락했다. CLAUDE 규칙 2가 LIVING_SPEC을
"executable authoritative spec"으로 명시함에도 정찰 grep 누락. 향후 A1 candidate
측정 시 3 location 모두 grep 의무.

**Infrastructure**:
- `compiler/scripts/check-empirical.sh`: A1 class 추가 (case statement 3 사이트 + usage doc).
- 3 A1 fixture PASS (`bash scripts/check-empirical.sh A1` → 3/0/0/0).
- 전체 empirical: 31 → 34 PASS.

Status: Step 10 PARTIAL_DONE. 3/4 candidate LANDED + 1 candidate (affine A2-06) reclassified.
다음 iteration scope: A2-06 affine lifecycle (promote-then-block, multi-iter — A2 일반 절차에 따라 진행).
