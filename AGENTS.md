# AGENTS.md — New Vais 프로젝트 자율 작업 가이드 (Codex / AI 에이전트용)

> 이 문서는 **맥락 없이 들어온 AI 에이전트**(Codex 등)가 New Vais 프로젝트를 안전하게
> 자율 진행하기 위한 진입점이다. 작업 시작 전 **반드시 끝까지 읽어라.**
> 진실의 원천(source of truth)은 `ROADMAP.md`다. 이 문서는 *방법론*을 담는다.

---

## 0. 30초 요약

- **New Vais**는 AI가 정확히 쓰는 언어로 확정된 새 Vais mainline이다. repo 경로는 `/Users/sswoo/study/projects/vais`이고, checked-in 소스 확장자는 `.vais`다.
- 현재 bootstrap 파이프라인: `New Vais(.vais/.nl) 소스` → `legacy_vais_bootstrap.py` → `Legacy Vais` → `vaisc build` → 네이티브 LLVM 실행.
  `nl2vais.py`는 기존 호출을 위한 compatibility wrapper다.
- **메인 산출물**: `compiler/self/fixpoint_full.vais` — New Vais로 작성한 self-host 컴파일러 seed.
  입력 New Vais 프로그램 문자열을 받아 **토큰화 → 평가/codegen → LLVM IR을 emit**한다.
- **현재 상태**: front(tokenize/eval) + codegen(print로 IR emit) 전체 self-host arc 동작.
  세 self-host tier(산술 / 산술+변수 / 함수+재귀) 전부 source→value end-to-end이고,
  `fixpoint_full.vais` 전체 소스가 1세대 컴파일러를 만들며 그 컴파일러가 실제 `fixpoint.vais`/`fixpoint2.vais`/`fixpoint3.vais`/`fixpoint_full.vais`를 다시 컴파일한다.
- **stage oracle**: 긴 게이트가 stage1/stage2 compiler IR을 비교한다. source-position 기반
  `@.sNNN`/`@.fmtNNN` global 이름만 정규화하고, 그 외 IR은 byte-for-byte 일치해야 한다.
- **현재 New Vais 명령 계약**: repo-local `scripts/vaisc`가 공식 `vaisc` 전환 대상이다.
  Legacy `vaisc` 바이너리는 내부 bootstrap/oracle 용도다.
- **다음 목표**: ROADMAP의 migration/hardening 큐. 자체 컴파일러 mainline을 유지하면서 Legacy Vais 의존을 bootstrap/oracle로 계속 축소한다.

---

## 1. 두 repo 경계 (가장 중요 — 헷갈리면 사고 난다)

이 작업은 **별개의 두 git repo**를 오간다:

| repo | 경로 | 역할 | 규칙 |
|------|------|------|------|
| **New Vais** | `/Users/sswoo/study/projects/vais` | 언어 자체(트랜스파일러/self-host 컴파일러/예제/게이트). | 이 문서 |
| **Legacy Vais** | `/Users/sswoo/study/projects/vais-legacy/compiler` | bootstrap/oracle 백엔드 컴파일러(`vaisc`) | **그 repo의 `CLAUDE.md` 엄격 규칙** |

- New Vais를 깊게 파다 보면 **Legacy Vais 컴파일러 버그**(codegen miscompile, ICE 등)를 만난다. 이때
  Legacy Vais repo를 수정하게 되는데, **그 repo의 `CLAUDE.md` 규칙이 그때부터 적용된다**(§5 참조).
  빈도 감각: 최근 세션들에서 New Vais 작업 중 Legacy Vais 버그 근본수정이 여러 건 있었다(`&Vec` borrow
  codegen, struct-return ICE, `%` 이스케이프 등). 현재 해결/추적 중인 Legacy Vais 갭은 ROADMAP의
  TRACKED 참조. **드물지 않다**고 가정하라.
- 커밋은 **각 repo에 따로** 한다. New Vais 변경은 New Vais repo에, Legacy Vais 변경은 Legacy Vais repo에.
- `vaisc`는 PATH에 있어야 하고, std 해석을 위해 `VAIS_COMPILER_ROOT`가 Legacy Vais repo를
  가리켜야 한다(게이트 스크립트는 기본값 `/Users/sswoo/study/projects/vais-legacy/compiler` 사용).

---

## 2. 세션 시작 루틴 (매번)

```bash
cd /Users/sswoo/study/projects/vais
# 1) 현재 목표 파악
sed -n '1,45p' ROADMAP.md          # 상태 헤더 + 남은 항목 + 우선순위 큐
# 2) 최근 작업 맥락 (WORKLOG는 최신이 맨 위)
sed -n '1,60p' WORKLOG.md
# 3) self-host 컴파일러 능력 현황
sed -n '100,110p' compiler/self/SELF_HOST.md   # fixpoint_full 능력 행
# 4) baseline 게이트 (작업 전 반드시 green 확인 — §3)
bash scripts/test-fixpoint-full.sh   # self-host codegen e2e + focused regression fixtures
bash scripts/test-fixpoint-full-self.sh # long full-source + retarget + stage1/stage2 compare gate
bash scripts/test.sh                 # 값-정확성 aggregate (예제+self-host 모듈)
bash scripts/test-vaisc.sh           # New Vais `vaisc` CLI/IR/build/run + Legacy oracle smoke
bash scripts/test-vaisc-front.sh     # New Vais day-1 native front accept/reject contract
bash scripts/test-vaisc-direct.sh    # New Vais direct LLVM emitter, Legacy 없이 emit/build/run
bash scripts/test-vaisc-errors.sh    # New Vais native P4 diagnostics: 좌표/help/fix
bash scripts/test-vaisc-parity.sh    # New Vais native/bootstrap/tracked parity manifest
bash scripts/test-vais-extension-migration.sh # .vais corpus를 임시 .nl mirror로 검증
```

전제: Legacy Vais repo에 빌드된 `target/debug/vaisc` 또는 `target/release/vaisc`가 있어야 한다.
bootstrap/oracle 스크립트는 `scripts/legacy-vaisc-env.sh`를 통해 이 repo-local binary를 PATH보다 우선한다.
없으면 `LEGACY_VAISC=/path/to/vaisc`로 지정하거나, 마지막 fallback으로 PATH의 `vaisc`를 사용한다.
게이트는 테스트마다 transpile+vaisc build+clang+run을 하므로 **느리다**(특히 e2e).
백그라운드로 시작하고 `RESULT:` 줄을 기다려라.

**baseline을 먼저 측정하라.** 작업 전 게이트가 green인지 확인하지 않으면, 나중에
실패가 내 변경 탓인지 원래 깨져 있었는지 구분할 수 없다.
*baseline이 이미 빨간 경우*: 내 작업과 무관할 수 있다. 실패가 ROADMAP §TRACKED의 Legacy Vais
백엔드 버그 때문인지 먼저 확인하고, 그게 아니면 사용자에게 보고 후 진행 여부를 정하라.

---

## 3. 게이트 (안전망 — 회귀 0이 절대 규칙)

| 게이트 | 명령 | 의미 |
|--------|------|------|
| **self-host e2e** | `bash scripts/test-fixpoint-full.sh` | `fixpoint_full.vais`가 New Vais 프로그램을 컴파일→IR→실행, **exit/stdout 값** 검증. stage drift 원인(direct double-string decode, callee List<Struct> arg authority) 회귀 포함. |
| **full-source self-host** | `bash scripts/test-fixpoint-full-self.sh` | 실제 `fixpoint_full.vais` 전체 소스가 1세대 컴파일러를 만들고, 그 컴파일러가 실제 `fixpoint.vais`/`fixpoint2.vais`/`fixpoint3.vais`/`fixpoint_full.vais`를 다시 컴파일해 final IR exit 24/50/120/42까지 확인. 마지막에 stage1/stage2 compiler IR을 normalized byte-compare한다. 느린 긴 게이트. |
| **값-정확성 aggregate** | `bash scripts/test.sh` | `examples/*.vais`(첫 줄 `# expect: N`) + self-host 모듈 빌드+실행+값 비교. 현재 **112/112**. |
| **New Vais CLI smoke** | `bash scripts/test-vaisc.sh` | `.vais` 입력을 repo-local `scripts/vaisc`로 LLVM IR emit/build/run하고 Legacy bootstrap oracle과 exit 값을 비교. |
| **New Vais front contract** | `bash scripts/test-vaisc-front.sh` | native subset과 print/putchar, simple struct, payload-free enum/match, small Int-coded payload enum/match, single-Int closure return, List push/len/index/sum slice를 accepted source로 실행하고, bad helper signature/for/broader payload enum/match/broader closure/Rust `&&`/unsupported method/string 등 unsupported 문법이 `help:` 진단으로 실패하는지 확인. |
| **New Vais direct emitter** | `bash scripts/test-vaisc-direct.sh` | `--engine direct`가 Legacy Vais 없이 단일 `main` arithmetic return을 LLVM IR로 emit/build/run하고 bootstrap engine과 값을 비교. |
| **New Vais P4 diagnostics** | `bash scripts/test-vaisc-errors.sh` | native `vaisc` 오류가 source 좌표, 원문 line, caret, `help:`, `fix:`를 포함하는지 확인. |
| **New Vais parity manifest** | `bash scripts/test-vaisc-parity.sh` | `tools/vaisc-parity.tsv`에 기록된 `native-supported` 예제와 trusted self-host tier를 New Vais `vaisc`와 Legacy oracle 양쪽에서 값 비교하고, `bootstrap-only`/`tracked` 상태가 stale하지 않은지 확인. |
| 기타 tier별 | `scripts/test-fixpoint*.sh` | 개별 codegen 영역(array/list/str/struct/imperative 등) |

규칙:
- **모든 변경 후 두 메인 게이트(e2e + aggregate)를 돌려 회귀 0을 확인**하고 커밋한다.
- 게이트가 **1개라도** 실패하면 그 변경은 잘못된 것이다. revert하거나 고친다.
- 게이트 통과 = "컴파일됨"이 아니라 **"실행 결과 값이 맞음"**(P7b: 컴파일≠정답).

### 게이트 실행 함정 (실측으로 학습됨)
- **동시 실행 금지**: 게이트 스크립트는 테스트마다 `rm -rf /tmp/.vais-cache`를 한다.
  e2e와 aggregate를 **동시에 백그라운드로 돌리면** 서로 캐시를 지워 카운트가 interleave되고
  거짓 실패가 난다. **반드시 직렬로** 하나씩 돌려라.
- e2e는 느리다(~160 테스트 × transpile+build+clang+run). 백그라운드로 돌리고
  `RESULT:` 줄이 나올 때까지 기다린다(완료 알림 또는 폴링).

---

## 4. 검증된 작업 방법론 (이 패턴을 따르면 안전하다)

이 프로젝트는 수백 iter에 걸쳐 다음 패턴이 검증됐다. **이탈하지 마라.**

1. **Recon 먼저** — 추측 금지. "이 기능 안 됨"은 보통 *증상*이다. grep + 실제 측정으로
   진짜 원인을 찾아라. (예: "print 비필요"라고 메모돼 있었으나 recon 결과 self-host
   codegen의 핵심 emission이었음 → 메모가 틀렸다.)
2. **아키텍처 변경 전 clang 스킴 격리검증** — fixpoint_full을 건드리기 전에, 목표 LLVM IR을
   `/tmp/x.ll`에 손으로 써서 `clang`으로 빌드+실행해 **스킴이 맞는지 먼저 확인**한다.
   (예: printf 보간 → `@.fmt` + `call printf` IR을 격리 실행, run=14 확인 후 구현.)
3. **fixpoint_full.compile() 실제 fragment로 측정** — 값은 **≤255**로(8-bit exit code 절단).
   stdout 검증이 필요하면 stdout을 본다(exit code 아님).
4. **probe "OK"를 의심하라** — garbage 메모리가 우연히 맞는 값을 줄 수 있다. **생성된 IR을
   직접 확인**해 진짜 맞는지 검증하라(named-fn golden ref와 대조).
5. **e2e 가드 추가 + 헤비 회귀** — 새 능력마다 `test-fixpoint-full.sh`에 값/stdout 가드를
   추가하고, 두 메인 게이트로 회귀 0 확인.
6. **격리 후 통합** — 개별 fragment로 동작 확인 → 통합 프로그램으로 end-to-end 확인.
7. **막히면 추적(TRACKED)하고 넘어가라** — half-merge가 동작하는 코드를 깨면 **revert가 옳다**.
   ROADMAP의 TRACKED 항목에 file:line 진단을 남기고 다른 scale-unblocked 항목으로.
8. **매 step 커밋** — 검증된 단계마다 커밋. 큰 아키텍처 변경은 **Steps로 분리 + 체크포인트
   커밋**(예: `-> List`를 Steps1-3 / Steps4-5로 나눠 각각 게이트 통과 후 커밋).

---

## 5. Legacy Vais 컴파일러 repo 수정 시 (⚠️ 별개 repo, 엄격 규칙)

New Vais 작업 중 Legacy Vais 버그를 만나 `/Users/sswoo/study/projects/vais-legacy/compiler/`를 고칠 때는
**그 repo의 `CLAUDE.md`를 먼저 읽고** 다음을 엄수한다:

- **baseline 기록 먼저** — 수정 전 `cd compiler && bash scripts/check-integrity.sh`로 현재
  상태를 측정. `check-integrity.sh`가 canonical aggregate 게이트다.
- **회귀 0 절대** — 수정 후 `check-integrity.sh` 재실행. **1개 테스트라도** 회귀하면 revert.
- **근본 추적만(root-cause only)** — stopgap/우회로 working code를 깨면 안 된다. half-fix가
  악화시키면 revert가 정답(이 프로젝트에서 stopgap revert가 여러 번 옳았다).
- **근거 검증** — Vais 문법은 **모델 기억 금지**. `compiler/docs/language/`,
  `compiler/docs/LANGUAGE_SPEC.md`, 실행 가능한 fixture로 확인.
- **codegen 버그는 silent miscompile 위험이 높다** — 컴파일만 통과하고 **값이 틀릴** 수 있다.
  반드시 **생성 IR 확인 + clang 빌드 + 실행 값 검증**까지 한다. (Vais 게이트의 VALUE
  CORRECTNESS 단계가 이 맹점을 막으려 추가됐다.)
- **두 repo는 별개 커밋** — Legacy Vais 수정은 Legacy Vais repo에 커밋, New Vais 가드는 New Vais repo에 커밋.
- Vais 커밋 메시지 끝에 Co-Authored-By 라인(그 repo 규칙 따름).

---

## 6. 함정 카탈로그 (실측으로 학습된 것들 — 다시 밟지 마라)

### Legacy bootstrap adapter (`legacy_vais_bootstrap.py`, old `nl2vais.py` wrapper)
- **line/token rewriter이지 진짜 파서가 아니다.** 단일 라인 nested if/else
  (`else { ...; if .. {} else {} }`)를 못 다룬다 → **여러 줄로 작성**하라.
- `expand_for_loops`의 brace 카운트는 `#` 주석 안의 brace도 센다 → **루프 본문 주석에
  unbalanced brace 금지**.
- **`{{`/`}}` 이스케이프**: nl 문자열 안에서 `{{`→Vais `\{`(리터럴 brace), `{x}`=보간.
  컴파일러에 먹이는 프로그램에서 보간을 원하면 `{{var}}`로 써야 outer transpiler가
  `\{var\}`로 바꿔 compile()에 `{var}`가 도달한다.

### fixpoint_full.vais (self-host 컴파일러)
- **lone-`{` vs `{ident}` 모호성**: 트랜스파일러가 리터럴 brace와 보간을 둘 다 단일 brace로
  전달 → 구분은 **Vais lexer 규칙**(`{`+식별자+`}`만 보간, lone `{`=리터럴)으로. `interp_end`
  헬퍼가 이 규칙을 단일 구현하고 fmt_len/emit_fmt_bytes/arg-loader 3곳이 공유한다.
- **슬롯 예약은 두 collector 모두**: 새 변수 바인딩 구문을 추가하면 `add_local_slots`(함수 내)
  **와** `collect_top_slots`(top-level) **양쪽**에 분기를 추가해야 한다.
- List 표현: local List = stack 버퍼(scalar `[64 x i64]`, struct-element `[64*nf+1 x i64]`,
  stride=필드수, 길이 헤더 at `buf[64*nf]`). List param(is_arr=4)=`i64*` 포인터.

### 측정/검증
- **8-bit exit code 절단**: 프로그램 exit 값은 `value % 256`. 값 검증은 **≤255**로 하거나
  stdout을 봐라(519%256=7로 오인 가능).
- **probe garbage 우연일치**: 잘못된 GEP가 우연히 그럴듯한 값을 읽을 수 있다. `x * 100`처럼
  곱해보면 garbage(보통 0)가 드러난다. **IR 직접 확인이 최종 판정**.
- **테스트 프로그램은 Python으로 빌드**: bash heredoc 안의 backtick `` `...` ``은
  command substitution을 트리거한다. 테스트 프로그램은 Python 스크립트로 `/tmp/q.vais`를
  써서 만들어라(게이트 스크립트의 `check`/`check_out`이 이미 그렇게 한다).

---

## 7. 자율 진행 루프 (한 iter의 형태)

```
1. ROADMAP에서 다음 scale-unblocked 항목 선택 (또는 TRACKED 항목 dedicated)
2. baseline 게이트 green 확인
3. recon (grep + 측정으로 진짜 갭/원인 파악)
4. [아키텍처 변경이면] clang 스킴 격리검증 먼저
5. 구현 (큰 변경은 Steps 분리)
6. fixpoint_full.compile() fragment로 값 측정 (≤255 / stdout)
7. e2e 가드 추가
8. 두 메인 게이트 직렬 실행 → 회귀 0 확인
9. 커밋 (New Vais repo; Legacy Vais 수정이면 Legacy Vais repo에 별도)
10. WORKLOG(맨 위) + ROADMAP(게이트 카운트/항목 상태) + SELF_HOST.md 갱신
11. 막혔으면 TRACKED에 file:line 남기고 다른 항목으로
```

**매 iter 가치 있는 1건+** 을 목표로. 막히면 멈추지 말고 추적하고 넘어가라.

---

## 8. 남은 작업 (ROADMAP §방향 결정 / 현재 우선순위 큐 참조)

- **migration/hardening 큐**: checked-in 확장자는 `.vais`로 전환됐고 repo 폴더도 `/projects/vais`로 승격됐다. 남은 것은 `.nl` 입력과 compatibility wrapper 제거 여부다.
- **NV-C0 완료 상태 유지**: `scripts/vaisc`와 `scripts/test-vaisc.sh`가 깨지면 제품 경계 회귀로 본다.
- **NV-C1 완료 상태 유지**: `scripts/vaisc` front preflight와 `scripts/test-vaisc-front.sh`가 깨지면 native front 계약 회귀로 본다.
- **self-host hardening**: stage oracle은 해결됐으므로, 새 stage drift 원인은 짧은 회귀 fixture와 긴 stage compare gate 양쪽에 묶어둔다.
- **Legacy Vais 백엔드 버그 (TRACKED)**: bootstrap/oracle 경로를 파다 만나는 codegen 버그들. §5 규칙으로 근본 수정.

---

## 9. 절대 하지 말 것

- ❌ baseline 측정 없이 작업 시작
- ❌ 게이트 회귀(1개라도)를 무시하고 커밋
- ❌ 두 repo를 헷갈려 New Vais 변경을 Legacy Vais repo에 커밋(또는 반대)
- ❌ Legacy Vais 문법을 모델 기억으로 작성(docs/fixture로 검증)
- ❌ Legacy Vais codegen 수정 후 IR/실행 값 검증 생략(silent miscompile)
- ❌ 단일 라인 nested if/else를 트랜스파일러에 먹임
- ❌ probe "OK"를 IR 확인 없이 신뢰
- ❌ e2e와 aggregate 게이트 동시 실행(cache race)
- ❌ half-merge가 working code를 깼는데 revert 안 함
