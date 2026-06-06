# nl 예제 코퍼스 (P9 인프라)

**검증된 nl 예제.** P9(예제 코퍼스 = 최강 레버, cold-start 1/5→5/5)의 핵심 인프라.
모든 `# expect: N` 예제는 `scripts/test.sh`로 빌드+실행+값 검증된다 (현재 30/30 PASS; 러너 전체는
self-host codegen 모듈 포함 48/48).

> 사용: `scripts/test.sh` (전체) / `scripts/test.sh e03_recursion` (하나).
> AI에게 nl을 가르칠 때 이 예제들을 컨텍스트로 제공하면 cold-start 정확도가 오른다(실측).

## 문법 커버리지 인덱스

| 예제 | 커버 문법 | expect |
|------|-----------|--------|
| c1 | enum + match + `=> return` | 2 |
| c2 | list 리터럴 + `.sum()` | 60 |
| c3 | `bitnot()` | 5 |
| c4 | struct 기본 | 42 |
| c5 | 클로저 캡처 (in-scope) | 7 |
| d3run | Result + `?` 전파 | 6 |
| d4b | List 파라미터 + for | 9 |
| d5run | pub struct | 42 |
| fr1 | for-range `0..=n` | 15 |
| fr2 | for-collection | 60 |
| t1 | 함수 + `let` | 7 |
| e01 | 중첩 struct (3-deep) | 9 |
| e02 | enum 다중 payload | 42 |
| e03 | 재귀 (factorial) | 120 |
| e04 | 상호 재귀 | 1 |
| e05 | 다중 함수 + 중첩 호출 | 24 |
| e06 | for-range 합산 | 55 |
| e07 | else if 체인 | 2 |
| e08 | Option + arm-return | 8 |
| e09 | struct 메서드 체인 | 25 |
| e10 | bool 논리 (and/or/not) | 1 |
| e11 | while 루프 | 10 |
| e12 | exclusive range `..` | 10 |
| e13 | 중첩 for | 9 |
| e14 | print (출력) | 0 |
| e15 | List 재귀 (`&List<T>` borrow) | 10 |
| e16 | Option match + payload 바인딩 | 42 |
| e17 | struct 반환 → 필드 접근 | 12 |
| e18 | while 누적기 (가변 acc + 카운터) | 30 |
| e19 | 문자열 보간 출력 `print("{x}")` | 0 |

## 미커버 (트랜스파일러/Vais 한계 — ROADMAP TRACKED)
- `.filter()` — Vais 백엔드 버그 (task_7cfebeba).
- `Map<K,V>` (HashMap) — `Map()` 생성자 + `.get()` Option 시맨틱 미사상(transpiler 갭, PRELUDE 🔶). 별도 task.
- 중첩 `match` (arm 안에 `=> match {...}`) — 라인-재작성기 P001. 일반 중첩은 함수분리로 우회 가능.

## 규약
- 첫 줄 `# expect: N` = main이 반환할 exit code (mod 256).
- 실행형(main이 값 반환)만 expect 부착. 라이브러리 조각은 미부착(러너 skip).
