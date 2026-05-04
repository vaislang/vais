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
