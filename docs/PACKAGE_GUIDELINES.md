# Vais 패키지 커뮤니티 가이드라인

이 문서는 Vais 패키지 레지스트리에 패키지를 배포하려는 개발자들을 위한 공식 가이드라인입니다. 패키지 생성부터 배포까지 전 과정에서 따라야 할 모범 사례를 제시합니다.

## 목차

1. [패키지 작성 가이드](#패키지-작성-가이드)
2. [버전 관리](#버전-관리)
3. [코드 품질 기준](#코드-품질-기준)
4. [보안 요구사항](#보안-요구사항)
5. [배포 절차](#배포-절차)
6. [커뮤니티 규칙](#커뮤니티-규칙)

---

## 패키지 작성 가이드

### vais.toml 구조

패키지 매니페스트 파일인 `vais.toml`은 패키지의 메타데이터와 의존성을 정의합니다. 다음은 완전한 예제입니다:

```toml
[package]
name = "json-parser"
version = "1.0.0"
authors = ["John Doe <john@example.com>", "Jane Smith <jane@example.com>"]
edition = "2024"
description = "고성능 JSON 파서 라이브러리"
license = "MIT"
homepage = "https://github.com/example/json-parser"
repository = "https://github.com/example/json-parser"
documentation = "https://docs.example.com/json-parser"
keywords = ["json", "parser", "serialization"]
categories = ["parser", "utilities"]
readme = "README.md"

[dependencies]
# 버전 명시
utf8-validator = "0.3"
# 버전 범위
string-utils = "1.0..=2.5"
# 로컬 경로 (개발 중)
internal-lib = { path = "../internal-lib" }
# 선택사항 의존성
serde-compat = { version = "0.1", optional = true }

[dev-dependencies]
test-fixtures = "0.2"
benchmark-utils = "0.1"

[build]
opt-level = 2
debug = false
```

### 필수 필드

| 필드 | 설명 | 예제 |
|------|------|------|
| `name` | 패키지 이름 (소문자, 하이픈 구분) | `json-parser` |
| `version` | SemVer 버전 | `1.0.0` |
| `authors` | 작성자 정보 | `["Name <email@example.com>"]` |
| `description` | 패키지 설명 (최대 200자) | `"고성능 JSON 파서"` |
| `license` | 라이선스 식별자 | `"MIT"`, `"Apache-2.0"` |

### 선택사항 필드

| 필드 | 설명 | 용도 |
|------|------|------|
| `homepage` | 프로젝트 홈페이지 URL | 사용자 정보 제공 |
| `repository` | 소스 코드 저장소 URL | 기여 유도 |
| `documentation` | 온라인 문서 URL | 사용자 교육 |
| `keywords` | 검색 키워드 (최대 5개) | 검색 기능 개선 |
| `categories` | 카테고리 (최대 5개) | 브라우징 지원 |
| `readme` | README 파일 경로 | 레지스트리 표시 |

### 디렉토리 구조 권장사항

표준 디렉토리 구조를 따르면 사용자가 패키지를 쉽게 이해하고 기여할 수 있습니다:

```
my-package/
├── vais.toml                 # 패키지 매니페스트 (필수)
├── README.md                 # 프로젝트 설명
├── LICENSE                   # 라이선스 파일
├── CHANGELOG.md              # 버전 변경 기록
├── src/
│   ├── lib.vais              # 라이브러리 진입점
│   ├── main.vais             # 바이너리 진입점 (선택)
│   └── ...                   # 내부 모듈
├── tests/
│   ├── test_basic.vais       # 기본 기능 테스트
│   ├── test_edge_cases.vais  # 엣지 케이스 테스트
│   └── ...
├── examples/
│   ├── basic.vais            # 기본 사용 예제
│   ├── advanced.vais         # 고급 사용 예제
│   └── ...
├── docs/
│   ├── API.md                # API 문서
│   ├── CONTRIBUTING.md       # 기여 가이드
│   └── ...
└── .gitignore                # 저장소 제외 파일
```

### 패키지 이름 규칙

- **소문자만 사용**: `json-parser` (O), `JsonParser` (X)
- **하이픈으로 단어 구분**: `http-client` (O), `httpclient` (X)
- **숫자 포함 가능**: `base64-encoder` (O)
- **의미 있는 이름**: 패키지의 기능을 명확하게 표현
- **전체 패키지명은 64자 이하**: 간결함 유지
- **이미 존재하는 이름 사용 금지**: [레지스트리 검색](https://registry.vais-lang.org) 확인

---

## 버전 관리

### SemVer 규칙 (의무)

Vais 레지스트리의 모든 패키지는 **Semantic Versioning 2.0.0**을 따라야 합니다.

버전 형식: `MAJOR.MINOR.PATCH` (예: `1.2.3`)

```
MAJOR: API 호환성이 깨지는 변경
  - 함수 시그니처 변경
  - 타입 정의 변경
  - 주요 기능 제거

MINOR: 하위 호환성을 유지하는 기능 추가
  - 새로운 public 함수 추가
  - 선택사항 매개변수 추가
  - 버그 수정

PATCH: 하위 호환성을 유지하는 버그 수정
  - 문서 개선
  - 성능 최적화
  - 내부 리팩토링
```

### 버전 변경 예시

```
0.1.0    첫 공개 릴리스
0.1.1    버그 수정
0.2.0    새 기능 추가 (하위 호환 유지)
1.0.0    첫 안정 릴리스
1.1.0    새 기능 추가
2.0.0    API 변경 (하위 호환성 깨짐)
2.0.1    버그 수정
```

### Pre-release 버전

개발 중인 버전은 다음 형식을 사용합니다:

```
1.0.0-alpha       초기 개발 단계
1.0.0-alpha.1     알파 2번째 버전
1.0.0-beta        베타 단계
1.0.0-beta.2      베타 3번째 버전
1.0.0-rc1         릴리스 후보 1
1.0.0-rc2         릴리스 후보 2
```

Pre-release 버전은 안정 버전보다 낮은 우선순위로 취급됩니다.

### 하위 호환성 유지 원칙

1. **공개 API 변경 금지**: 이미 배포된 버전의 public 함수 시그니처 변경 금지
2. **구조체 필드 추가 가능**: 새로운 필드는 기본값과 함께 추가 (MINOR 버전 상향)
3. **열거형 변형 추가 가능**: 새로운 열거형 값 추가는 가능 (MINOR 버전 상향)
4. **의존성 업그레이드**: 주요 버전 업그레이드는 MAJOR 버전 상향
5. **CHANGELOG 필수**: 각 릴리스마다 변경 사항 기록

### CHANGELOG 작성 예제

파일: `CHANGELOG.md`

```markdown
# Changelog

## [1.1.0] - 2024-01-15

### Added
- 새로운 `parse_streaming()` 함수로 대용량 파일 처리 지원
- 커스텀 에러 핸들러 옵션 추가

### Changed
- `parse()` 함수의 성능 30% 개선
- 오류 메시지 명확성 개선

### Fixed
- 유니코드 이스케이프 처리 버그 수정
- 메모리 누수 해결

### Deprecated
- `parse_unsafe()` 함수 (1.2.0에서 제거 예정)

## [1.0.0] - 2023-12-01

### Added
- 초기 안정 릴리스
- JSON 파싱 기본 기능
- 에러 처리 및 검증
```

---

## 코드 품질 기준

### 필수 기준

#### 1. 컴파일 통과

패키지를 배포하기 전에 반드시 통과해야 합니다:

```bash
# 에러 검사
vais pkg check

# 완전한 빌드 및 링크
vais pkg build

# 모든 테스트 성공
vais pkg test
```

모든 명령이 성공 상태(exit code 0)로 완료되어야 합니다.

#### 2. 기본 테스트

최소 다음 범위의 테스트 포함:

```vais
// tests/test_basic.vais
fn test_basic_functionality() {
    // 기본 기능 검증
    let result = parse("{\"key\": \"value\"}")
    assert(result != nil)
}

fn test_error_handling() {
    // 오류 처리 검증
    let result = parse("{invalid json}")
    assert(result == nil)
}

fn test_edge_cases() {
    // 엣지 케이스 검증
    let result = parse("")
    assert(result == nil)

    let result2 = parse("{}")
    assert(result2 != nil)
}
```

### 권장 기준

#### 1. 문서 주석 (///)

모든 공개 API에 문서 주석 추가:

```vais
/// JSON 문자열을 파싱합니다.
///
/// # Arguments
/// * `input` - 파싱할 JSON 문자열
///
/// # Returns
/// 성공 시 JSON 객체, 실패 시 nil
///
/// # Example
/// ```vais
/// let json = parse("{\"name\": \"Alice\"}")
/// ```
pub fn parse(input: string) {
    // 구현
}
```

#### 2. 예제 코드

`examples/` 디렉토리에 실행 가능한 예제 포함:

```vais
// examples/basic_usage.vais
// 패키지의 기본 사용법을 보여주는 예제

U json-parser

fn main() {
    let data = parse("{\"name\": \"Alice\", \"age\": 30}")

    // 오류 처리
    if data == nil {
        println("파싱 실패")
        return
    }

    println("파싱 성공")
}
```

#### 3. README.md

프로젝트 설명 및 사용 가이드:

```markdown
# JSON Parser

고성능 JSON 파서 라이브러리

## 설치

vais.toml에 다음을 추가하세요:

```toml
[dependencies]
json-parser = "1.0"
```

## 사용 예제

```vais
U json-parser

fn main() {
    let data = parse("{\"key\": \"value\"}")
}
```

## 특징

- 빠른 성능
- 메모리 효율적
- 완전한 오류 처리

## 라이선스

MIT
```

#### 4. API 문서

복잡한 패키지는 `docs/API.md` 포함:

```markdown
# API 문서

## 함수 목록

### parse(input: string)

JSON 문자열을 파싱합니다.

**Parameters:**
- `input` (string): 파싱할 JSON 문자열

**Returns:**
- JSON 객체 또는 nil

**Example:**
```vais
let result = parse("{}")
```

### stringify(obj)

JSON 객체를 문자열로 변환합니다.
...
```

---

## 보안 요구사항

### 패키지 서명 (필수)

모든 배포된 패키지는 SHA-256 체크섬으로 서명됩니다. 이는 자동으로 수행되지만, 배포자는 다음을 확인해야 합니다:

```bash
# 배포 전 패키지 검증
vais pkg verify my-package-1.0.0.tar.gz
```

### SBOM 생성 (권장)

소프트웨어 구성 명세서(Software Bill of Materials, CycloneDX 형식)를 생성하여 투명성 제공:

```bash
# SBOM 생성
vais pkg sbom my-package > sbom.xml
```

생성된 SBOM 파일은 패키지와 함께 배포되어야 합니다.

### 의존성 최소화 원칙

1. **필요한 의존성만 포함**
   - 사용하지 않는 의존성 제거
   - 기능성이 동일한 더 경량의 대체 패키지 검토

2. **의존성 버전 최소화**
   ```toml
   # 좋은 예: 필요한 최소 버전만 명시
   [dependencies]
   util-lib = "1.0"

   # 피해야 할 예: 과도한 상향 호환 버전 지정
   util-lib = "1.0..=10.0"
   ```

3. **선택사항 의존성 활용**
   ```toml
   # 필수가 아닌 통합은 선택사항으로
   [dependencies]
   serde-support = { version = "0.1", optional = true }
   ```

### unsafe 코드 문서화

unsafe 코드 사용 시 명확한 문서화 필수:

```vais
/// 메모리 직접 조작
///
/// # Safety
///
/// 다음 조건 만족 시에만 안전함:
/// - ptr는 유효한 포인터여야 함
/// - len은 할당된 메모리 크기 이상이어야 함
/// - 이 함수 호출 중에 ptr의 메모리가 수정되지 않아야 함
pub unsafe fn direct_access(ptr, len) {
    // 구현
}
```

### 악성 코드 금지

패키지는 다음을 포함할 수 없습니다:

- 시스템 리소스를 무단으로 접근하는 코드
- 사용자 동의 없이 데이터를 수집하는 코드
- 백도어나 원격 실행 코드
- 다른 패키지를 변조하는 코드
- 암호 채굴 코드

위 사항 위반 시 패키지는 즉시 yanked 처리되고, 작성자는 레지스트리에서 영구 제거될 수 있습니다.

---

## 배포 절차

### 1. 계정 준비

```bash
# 레지스트리에 가입
vais pkg signup

# 로그인
vais pkg login
# 사용자명과 토큰 입력
```

API 토큰 생성:

```bash
# 새로운 토큰 생성
vais pkg token create --name "publish-token"

# 출력된 토큰 저장 (다시 볼 수 없음)
# 환경 변수로 설정
export VAIS_REGISTRY_TOKEN="your-token-here"
```

### 2. 배포 전 체크리스트

배포 전 다음을 확인하세요:

- [ ] `vais pkg check` 성공
- [ ] `vais pkg build` 성공
- [ ] `vais pkg test` 모든 테스트 통과
- [ ] vais.toml의 버전이 git 태그와 일치
- [ ] CHANGELOG.md 최신 버전 기록
- [ ] README.md 최신 정보 반영
- [ ] 라이선스 파일 포함
- [ ] 선택사항 필드(homepage, repository 등) 정확한가
- [ ] 의존성 버전이 지나치게 제한되지 않았는가

### 3. 배포 명령

```bash
# 기본 배포
vais pkg publish

# 특정 버전 배포
vais pkg publish --version 1.0.0

# 드라이 런 (실제 배포 전 검증)
vais pkg publish --dry-run
```

### 4. 배포 후 검증

```bash
# 레지스트리에서 패키지 확인
vais pkg search my-package

# 패키지 정보 조회
vais pkg info my-package

# 특정 버전 정보
vais pkg info my-package@1.0.0
```

### 5. 버그 발견 시 대응

**마이너 버그 (PATCH):**

```bash
# 버그 수정
# vais.toml 버전을 1.0.1로 변경
vais pkg publish
```

**주요 버그 또는 보안 문제:**

```bash
# 버그 있는 버전 yanked (사용 불가능하게 표시)
vais pkg yank my-package@1.0.0

# 버그 수정 후 새 버전으로 배포
# vais.toml 버전을 1.0.1로 변경
vais pkg publish
```

### CI/CD 통합 (GitHub Actions 예제)

`.github/workflows/publish.yml`:

```yaml
name: Publish Package

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Vais
        run: |
          curl -sSf https://install.vais-lang.org | sh
          echo "$HOME/.vais/bin" >> $GITHUB_PATH

      - name: Check Package
        run: vais pkg check

      - name: Run Tests
        run: vais pkg test

      - name: Publish
        env:
          VAIS_REGISTRY_TOKEN: ${{ secrets.VAIS_REGISTRY_TOKEN }}
        run: vais pkg publish
```

설정 단계:

1. GitHub에서 새로운 Personal Access Token 생성
2. 저장소의 Settings > Secrets > New repository secret에서 `VAIS_REGISTRY_TOKEN` 추가
3. 태그를 밀어내면 자동으로 배포됨

```bash
# 새 버전 태그 지정 및 배포
git tag v1.0.0
git push origin v1.0.0
# 자동으로 GitHub Actions가 실행되고 배포됨
```

### Yanking 정책

버전 yanking은 다음 경우에 수행됩니다:

1. **심각한 버그 발견**
   ```bash
   vais pkg yank my-package@1.0.0 --reason "Critical memory leak"
   ```

2. **보안 취약점**
   ```bash
   vais pkg yank my-package@0.5.0 --reason "CVE-2024-00001"
   ```

3. **라이선스 문제**
   ```bash
   vais pkg yank my-package@2.0.0 --reason "License violation"
   ```

Yanked 버전은:
- 새로운 프로젝트에서 설치할 수 없음
- 기존 잠금 파일에서는 사용 가능 (기존 프로젝트 보호)
- 레지스트리에서 명확히 표시됨

---

## 커뮤니티 규칙

### 1. 이름 선점 금지 (Name Squatting)

패키지 이름은 실제 프로젝트를 위해 예약되어야 합니다.

**금지 사항:**
- 향후 사용할 목적으로 이름만 등록하고 방치
- 다른 프로젝트의 이름과 유사한 패키지 등록으로 혼동 유도
- 인기 있는 이름을 등록 후 판매 시도

**정책:**
- 6개월 이상 아무 활동 없는 패키지는 삭제 대상
- 유사 이름으로 인한 사용자 혼동이 명백한 경우 이름 변경 요청
- 반복적 위반 시 계정 영구 정지

### 2. 악성 코드 금지

패키지에는 다음이 포함될 수 없습니다:

**기술적 악성 코드:**
- 바이러스, 웜, 트로이목마
- 랜섬웨어
- 스파이웨어 또는 애드웨어
- 원격 실행 백도어

**행동상 악성:**
- 사용자 동의 없이 개인정보 수집
- 시스템 리소스를 과도하게 소비
- 다른 소프트웨어 손상

**감지 시 조치:**
1. 즉시 모든 버전 yanked
2. 계정 접근 제한
3. 필요시 법적 조치

### 3. 라이선스 명시 필수

모든 패키지는 명확한 라이선스를 포함해야 합니다.

**허용 라이선스:**
- MIT, Apache 2.0, GPL 3.0 등 인정된 오픈소스 라이선스
- 듀얼 라이선스 가능
- 독점 라이선스 (명시된 조건 필수)

**라이선스 파일 예제:**

`LICENSE`:
```
MIT License

Copyright (c) 2024 Your Name

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
...
```

**vais.toml에 명시:**
```toml
[package]
name = "my-package"
license = "MIT"
```

### 4. 분쟁 해결 절차

패키지 이름이나 기능에 관한 분쟁 발생 시:

**Step 1: 직접 협의 (7일)**
- 패키지 작성자에게 이메일 또는 GitHub Issue로 연락
- 기술적 해결 방안 모색
- 호의적 합의 시도

**Step 2: 레지스트리 중재 신청 (7일)**
- 분쟁 세부사항과 증거 제출
- registry-disputes@vais-lang.org로 신청
- 레지스트리 팀이 중재 시작

**Step 3: 심사 (14일)**
- 양측 의견 청취
- 정책 및 커뮤니티 표준 검토
- 결정 이유와 함께 통보

**Step 4: 항소 (7일)**
- 결정에 동의하지 않으면 항소 가능
- appeals@vais-lang.org로 신청
- 독립적 심사위원회 재검토

**일반적 결정:**
1. 이름 변경 요청
2. 동의 하에 이름 이전
3. 분쟁 상태로 표시 (양쪽 모두 접근 불가)
4. 계정 정지 또는 삭제

### 5. 커뮤니티 기여 권장

개선 사항 제시:

```bash
# 패키지 저장소에서 이슈 제출
# - 버그 리포트
# - 기능 제안
# - 문서 개선 아이디어

# 패치 제출
git clone <repository>
# 브랜치 생성 및 수정
git push origin feature-branch
# Pull Request 제출
```

기여자 인정:

패키지 유지자는 공헌자를 명확히 인정해야 합니다:

```toml
# vais.toml
[package]
authors = [
    "Original Author <author@example.com>",
    "Contributor Name <contributor@example.com>"
]
```

### 6. 폐기된 패키지 처리

유지보수를 중단하려는 경우:

```bash
# 1. 패키지를 deprecated 상태로 표시
vais pkg deprecate my-package \
  --replacement "new-package" \
  --message "이 패키지는 new-package로 대체되었습니다"

# 2. 모든 사용자에게 마이그레이션 가이드 제공
# README.md 및 CHANGELOG.md에 안내 작성

# 3. 최소 6개월 후 yanked 처리 가능
vais pkg yank my-package@* --reason "Project discontinued"
```

### 7. 이슈 신고

다음 경우 registry-abuse@vais-lang.org로 신고:

- 악성 코드 또는 의심 패키지
- 라이선스 위반
- 지식재산권 침해
- 스팸 또는 피싱 시도
- 기타 정책 위반

**신고 시 포함 사항:**
- 패키지 이름 및 버전
- 구체적 위반 사항
- 증거 자료
- 신고자 연락처

---

## FAQ

**Q: 처음 배포하는데 1.0.0부터 시작해야 하나요?**

A: 아니요. 충분히 테스트되지 않았으면 0.1.0부터 시작하세요. 하지만 반드시 vais.toml에서 버전을 명시하고 정확하게 유지하세요.

**Q: 의존성 버전을 너무 느슨하게 지정하면 문제가 되나요?**

A: 네. 최소 필요 버전만 명시하세요. 사용자는 자신의 vais.toml에서 버전을 더 제한할 수 있습니다.

**Q: 패키지를 완전히 삭제할 수 있나요?**

A: 직접 삭제는 불가능합니다. 모든 버전을 yank하거나 레지스트리에 요청하여 처리합니다.

**Q: 보안 취약점을 발견했어요. 어떻게 하나요?**

A: security@vais-lang.org로 비공개 신고하세요. 공개 이슈에서 취약점을 노출하면 안 됩니다.

**Q: 패키지 이름을 변경할 수 있나요?**

A: 한 번 배포된 이름은 변경할 수 없습니다. 새 이름으로 새 패키지를 만들고, 기존 패키지를 deprecated 처리하세요.

---

## 추가 자료

- [SemVer 공식 사이트](https://semver.org/)
- [SPDX 라이선스 목록](https://spdx.org/licenses/)
- [Vais 패키지 매니저 설명서](../design/package-manager-design.md)
- [Vais 커뮤니티 행동 강령](../CODE_OF_CONDUCT.md)

---

**최종 업데이트**: 2024년 1월

Vais 레지스트리에 기여해주셔서 감사합니다!
