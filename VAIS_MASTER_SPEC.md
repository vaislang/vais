좋아.
그럼 **지금까지의 논의·결정·언어 스펙을 전부 정리해서
"이 파일 하나만 있으면 프로젝트가 다시 시작되는"**
👉 **단일 기준 md**를 만들어줄게.

이 문서는:

* ❌ 아이디어 메모 아님
* ❌ 탐색 문서 모음 아님
* ✅ **Vais 프로젝트의 헌법 + 언어 스펙 + 방향 선언**

이 파일 **하나만** 저장하면 된다.

---

## 📄 `VAIS_MASTER_SPEC.md`

````md
# Vais – AI-Optimized Executable Language
## AI가 가장 빠르게 이해 · 수정 · 컴파일하는 실행 언어

---

## 0. 이 문서의 지위 (중요)

이 문서는 Vais 프로젝트의 **유일한 기준 문서**다.

- 이 문서와 충돌하는 설계 ❌
- 이 문서 없이 추가되는 기능 ❌
- 이 문서에 없는 개념은 Vais이 아님

> ❗ Vais은 "사람을 위한 언어"가 아니다
> ❗ Vais의 1차 사용자는 **AI**다
> ❗ 사람은 볼 수 있고 수정할 수 있으나, 편의성은 고려하지 않는다

---

## 1. Vais의 존재 이유

기존 프로그래밍 언어의 공통 전제:

- 사람이 읽는다
- 사람이 이해한다
- 사람이 수정한다

Vais의 전제:

- **AI가 읽는다**
- **AI가 이해한다**
- **AI가 수정한다**
- 사람은 결과만 확인한다

Vais은 다음 목표를 가진다:

> **AI가 코드를 보고 즉시 구조를 파악하고
> 구조 일부만 수정하며
> 거의 즉시 컴파일할 수 있는가**

---

## 2. Vais의 정체

Vais은 다음 특성을 가진다.

| 항목 | 정의 |
|---|---|
| 언어 유형 | 실행 가능한 컴파일 언어 |
| 1차 사용자 | AI |
| 인간 가독성 | 고려하지 않음 |
| 인간 수정 | 가능 (비권장) |
| 의미 추론 | 없음 |
| 결정성 | 100% |
| 컴파일 | 초고속, 규칙 기반 |

Vais은 **AI가 설계하고 기계가 실행하는 언어**다.

---

## 3. 절대 설계 원칙 (Non-Negotiables)

아래 원칙은 변경 불가다.

1. 문법 모호성 0
2. 같은 의미 = 같은 구조
3. 의미 추론 금지
4. 실행 판단은 컴파일러 규칙으로만 수행
5. AI 추론은 컴파일 타임 이전에만 허용
6. 실행 중 AI 호출 금지

---

## 4. 인간을 배려하지 않는 이유

인간 편의성을 고려하면 반드시 발생하는 것들:

- 문법 설탕
- 암묵적 의미
- 표현의 자유
- 스타일 차이

이 모든 것은 **AI에게 해석 비용과 오류 가능성**을 만든다.

Vais은 의도적으로 다음을 포기한다:

- 짧은 키워드
- 중첩 문법
- 들여쓰기 의미
- 타입 추론
- 제어 흐름 문법

---

## 5. Vais이 존재하지 않는 개념들

Vais에는 **다음 개념이 없다**:

- 변수
- if / else
- for / while
- 함수 호출 문법
- 예외 처리
- 스코프
- 암묵적 타입
- 런타임 추론

👉 이 모든 것은 **컴파일러 내부 구현 문제**다.

---

## 6. Vais 파일의 전체 구조 (v0.1)

모든 Vais 파일은 아래 구조를 **반드시** 따른다.

```plaintext
UNIT
META
INPUT
INTENT
CONSTRAINT
EXECUTION
VERIFY
END
````

* 순서 고정
* 누락 불가
* 중복 불가

---

## 7. 문법 정의 (v0.1)

### 7.1 UNIT

```plaintext
UNIT <UnitType> <VaisUnit>
```

* UnitType: FUNCTION | SERVICE | PIPELINE
* VaisUnit: 전역 유일 식별자

---

### 7.2 META

```plaintext
META
  DOMAIN <domain>
  DETERMINISM <true|false>
ENDMETA
```

---

### 7.3 INPUT

```plaintext
INPUT
  <name> : <type>
ENDINPUT
```

* 타입 완전 명시
* 추론 없음

---

### 7.4 INTENT (핵심)

```plaintext
INTENT
  GOAL <action>
  PRIORITY <A> > <B> > <C>
ENDINTENT
```

* HOW 금지
* WHAT만 허용

---

### 7.5 CONSTRAINT

```plaintext
CONSTRAINT
  FORBID <rule>
  REQUIRE <rule>
ENDCONSTRAINT
```

---

### 7.6 EXECUTION

```plaintext
EXECUTION
  PARALLEL <true|false>
  TARGET <any|cpu|gpu>
  MEMORY <bounded|unbounded>
ENDEXECUTION
```

---

### 7.7 VERIFY

```plaintext
VERIFY
  PROPERTY <property>
  ASSERT <expression>
ENDVERIFY
```

---

### 7.8 END

```plaintext
END
```

---

## 8. 컴파일 모델

Vais 컴파일은 단 3단계다.

```text
1. Parse    → 구조 파싱
2. Validate → 규칙 검증
3. Generate → 타겟 코드 생성
```

* 의미 추론 ❌
* 스타일 처리 ❌
* 휴리스틱 ❌

---

## 9. AI와 Vais의 관계

AI는 다음 역할만 수행한다:

* Vais 코드 생성
* Vais 구조 수정
* 요구사항 변경 반영

AI는:

* Vais을 **실행하지 않는다**
* 런타임에 개입하지 않는다

---

## 10. Vais의 진짜 속도 정의

Vais의 속도는 다음에서 나온다:

* 파싱 비용 최소
* 구조 수정 비용 최소
* 컴파일 결정성
* AI 재추론 제거

> **Vais은 실행을 빠르게 하기 위한 언어가 아니라
> 변경과 재컴파일을 빠르게 하기 위한 언어다**

---

## 11. 성공 조건 (이것만 증명하면 됨)

Vais은 아래가 증명되면 성공이다.

1. AI가 Vais 파일을 즉시 이해한다
2. 요구 변경 시 일부 블록만 수정한다
3. 컴파일 시간이 기존 언어보다 짧다
4. 결과가 항상 재현 가능하다

---

## 12. 최종 선언

> Vais은
> 인간을 배제함으로써
> AI의 속도를 극대화한 언어다.
>
> 이것은 실험이 아니라
> **지금 구현 가능한 실행 언어다.**

---

```

---
