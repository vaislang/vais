# Vais 플러그인 시스템 설계

> **작성일**: 2026-01-20
> **상태**: 설계 완료
> **구현 대상**: vais-plugin, vaisc

---

## 1. 개요

### 목표
- 컴파일러 확장 포인트 제공 (최적화 패스, 린트 규칙, 코드 생성)
- 동적 플러그인 로딩 (`.so`/`.dylib`)
- 설정 파일 기반 플러그인 관리 (`vais-plugins.toml`)

### 핵심 원칙
1. **안전성**: 플러그인이 컴파일러를 크래시시키지 않도록
2. **단순성**: 플러그인 작성이 쉬워야 함
3. **성능**: 플러그인 로딩/실행 오버헤드 최소화

---

## 2. 플러그인 확장 포인트

### 2.1 컴파일 파이프라인 단계별 확장

```
Source Code (.vais)
    ↓
[1] Lexer → [Plugin: Lint] 토큰 레벨 검사
    ↓
[2] Parser → [Plugin: Transform] AST 변환
    ↓
[3] Type Checker → [Plugin: Lint] 타입 레벨 검사
    ↓
[4] Code Generator
    ↓
[5] IR Optimizer → [Plugin: Optimize] 커스텀 최적화 패스
    ↓
[6] Linker
```

### 2.2 플러그인 타입

| 타입 | 설명 | 입력/출력 | 예시 |
|------|------|----------|------|
| **Lint** | 코드 품질 검사 | AST → Diagnostics | 복잡도 경고, 스타일 검사 |
| **Transform** | AST 변환 | AST → AST | 매크로 확장, 코드 생성 |
| **Optimize** | IR 최적화 | LLVM IR → LLVM IR | 커스텀 최적화 패스 |
| **Codegen** | 추가 출력 생성 | AST → 파일 | 바인딩 생성, 문서 생성 |

---

## 3. 아키텍처

### 3.1 크레이트 구조

```
crates/
├── vais-plugin/              # 플러그인 시스템 코어
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs            # 메인 API
│       ├── traits.rs         # 플러그인 트레이트 정의
│       ├── loader.rs         # 동적 로딩
│       ├── registry.rs       # 플러그인 레지스트리
│       └── config.rs         # 설정 파일 파싱
│
└── vaisc/                    # CLI (플러그인 통합)
    └── src/
        └── main.rs
```

### 3.2 핵심 트레이트

```rust
// crates/vais-plugin/src/traits.rs

use vais_ast::Module;
use std::any::Any;

/// 플러그인 메타데이터
pub struct PluginInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
}

/// 모든 플러그인의 기본 트레이트
pub trait Plugin: Send + Sync {
    /// 플러그인 정보 반환
    fn info(&self) -> PluginInfo;

    /// 플러그인 초기화 (설정 전달)
    fn init(&mut self, config: &PluginConfig) -> Result<(), String>;

    /// 다운캐스팅을 위한 Any 반환
    fn as_any(&self) -> &dyn Any;
}

/// Lint 플러그인
pub trait LintPlugin: Plugin {
    /// AST 검사 후 진단 메시지 반환
    fn check(&self, module: &Module) -> Vec<Diagnostic>;
}

/// Transform 플러그인
pub trait TransformPlugin: Plugin {
    /// AST 변환
    fn transform(&self, module: Module) -> Result<Module, String>;
}

/// Optimize 플러그인
pub trait OptimizePlugin: Plugin {
    /// LLVM IR 최적화
    fn optimize(&self, ir: &str) -> Result<String, String>;

    /// 최적화 레벨 (언제 실행되어야 하는지)
    fn opt_level(&self) -> OptLevel {
        OptLevel::O2
    }
}

/// Codegen 플러그인
pub trait CodegenPlugin: Plugin {
    /// 추가 파일 생성
    fn generate(&self, module: &Module, output_dir: &Path) -> Result<Vec<PathBuf>, String>;
}

/// 진단 메시지
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub span: Option<vais_ast::Span>,
    pub help: Option<String>,
}

pub enum DiagnosticLevel {
    Warning,
    Error,
    Info,
}
```

### 3.3 플러그인 로더

```rust
// crates/vais-plugin/src/loader.rs

use libloading::{Library, Symbol};
use std::path::Path;

/// 동적 라이브러리에서 플러그인 로드
pub fn load_plugin(path: &Path) -> Result<Box<dyn Plugin>, String> {
    unsafe {
        let lib = Library::new(path)
            .map_err(|e| format!("Cannot load plugin: {}", e))?;

        // 플러그인은 반드시 `create_plugin` 함수를 export해야 함
        let create: Symbol<fn() -> Box<dyn Plugin>> = lib
            .get(b"create_plugin")
            .map_err(|e| format!("Invalid plugin: {}", e))?;

        Ok(create())
    }
}

/// 플러그인 타입 확인
pub fn downcast_plugin<T: 'static>(plugin: &dyn Plugin) -> Option<&T> {
    plugin.as_any().downcast_ref::<T>()
}
```

### 3.4 플러그인 레지스트리

```rust
// crates/vais-plugin/src/registry.rs

use std::collections::HashMap;

pub struct PluginRegistry {
    lint_plugins: Vec<Box<dyn LintPlugin>>,
    transform_plugins: Vec<Box<dyn TransformPlugin>>,
    optimize_plugins: Vec<Box<dyn OptimizePlugin>>,
    codegen_plugins: Vec<Box<dyn CodegenPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            lint_plugins: Vec::new(),
            transform_plugins: Vec::new(),
            optimize_plugins: Vec::new(),
            codegen_plugins: Vec::new(),
        }
    }

    /// 설정 파일에서 플러그인 로드
    pub fn load_from_config(&mut self, config: &PluginConfig) -> Result<(), String> {
        for plugin_path in &config.plugins {
            let plugin = load_plugin(plugin_path)?;
            self.register(plugin)?;
        }
        Ok(())
    }

    /// 플러그인 등록 (타입에 따라 적절한 벡터에 추가)
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), String> {
        // 다운캐스팅으로 타입 확인
        if let Some(_lint) = downcast_plugin::<dyn LintPlugin>(&*plugin) {
            // 안전한 방법으로 재변환 필요
        }
        // ... 각 타입별 처리
        Ok(())
    }

    /// Lint 플러그인 실행
    pub fn run_lint(&self, module: &Module) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for plugin in &self.lint_plugins {
            diagnostics.extend(plugin.check(module));
        }
        diagnostics
    }

    /// Transform 플러그인 실행
    pub fn run_transform(&self, module: Module) -> Result<Module, String> {
        let mut result = module;
        for plugin in &self.transform_plugins {
            result = plugin.transform(result)?;
        }
        Ok(result)
    }

    /// Optimize 플러그인 실행
    pub fn run_optimize(&self, ir: &str, level: OptLevel) -> Result<String, String> {
        let mut result = ir.to_string();
        for plugin in &self.optimize_plugins {
            if plugin.opt_level() <= level {
                result = plugin.optimize(&result)?;
            }
        }
        Ok(result)
    }

    /// Codegen 플러그인 실행
    pub fn run_codegen(&self, module: &Module, output_dir: &Path) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();
        for plugin in &self.codegen_plugins {
            files.extend(plugin.generate(module, output_dir)?);
        }
        Ok(files)
    }
}
```

### 3.5 설정 파일

```toml
# vais-plugins.toml

[plugins]
# 로컬 플러그인
path = [
    "./plugins/my-lint-plugin.dylib",
    "./plugins/my-optimizer.dylib",
]

# 설치된 플러그인 (이름으로 참조)
enabled = [
    "vais-lint-complexity",
    "vais-bindgen",
]

# 플러그인별 설정
[plugins.config]
"vais-lint-complexity" = { max_complexity = 10 }
"vais-bindgen" = { language = "python", output_dir = "./bindings" }
```

```rust
// crates/vais-plugin/src/config.rs

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct PluginsConfig {
    pub plugins: PluginsSection,
}

#[derive(Deserialize)]
pub struct PluginsSection {
    /// 로컬 플러그인 경로
    pub path: Vec<PathBuf>,

    /// 설치된 플러그인 이름
    pub enabled: Vec<String>,

    /// 플러그인별 설정
    pub config: HashMap<String, toml::Value>,
}

impl PluginsConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read config: {}", e))?;
        toml::from_str(&content)
            .map_err(|e| format!("Invalid config: {}", e))
    }
}

/// 플러그인에 전달되는 설정
pub struct PluginConfig {
    pub values: HashMap<String, toml::Value>,
}
```

---

## 4. 플러그인 작성 예시

### 4.1 Lint 플러그인 (복잡도 검사)

```rust
// plugins/complexity-lint/src/lib.rs

use vais_plugin::{Plugin, LintPlugin, PluginInfo, Diagnostic, DiagnosticLevel, PluginConfig};
use vais_ast::{Module, Expr, Stmt};

pub struct ComplexityLintPlugin {
    max_complexity: usize,
}

impl Plugin for ComplexityLintPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "complexity-lint",
            version: "0.1.0",
            description: "Warns about overly complex functions",
        }
    }

    fn init(&mut self, config: &PluginConfig) -> Result<(), String> {
        if let Some(max) = config.values.get("max_complexity") {
            self.max_complexity = max.as_integer()
                .ok_or("max_complexity must be integer")? as usize;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl LintPlugin for ComplexityLintPlugin {
    fn check(&self, module: &Module) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for item in &module.items {
            if let vais_ast::Item::Function(func) = &item.node {
                let complexity = calculate_complexity(&func.body);
                if complexity > self.max_complexity {
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Warning,
                        message: format!(
                            "Function '{}' has complexity {} (max: {})",
                            func.name.node, complexity, self.max_complexity
                        ),
                        span: Some(func.name.span),
                        help: Some("Consider breaking this function into smaller parts".to_string()),
                    });
                }
            }
        }

        diagnostics
    }
}

fn calculate_complexity(body: &FunctionBody) -> usize {
    // Cyclomatic complexity 계산
    // if/match/loop 등 분기문마다 +1
    1 // 기본값
}

// 플러그인 export 함수
#[no_mangle]
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(ComplexityLintPlugin { max_complexity: 10 })
}
```

### 4.2 Optimize 플러그인 (커스텀 최적화)

```rust
// plugins/custom-optimizer/src/lib.rs

use vais_plugin::{Plugin, OptimizePlugin, PluginInfo, PluginConfig, OptLevel};

pub struct CustomOptimizer;

impl Plugin for CustomOptimizer {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "custom-optimizer",
            version: "0.1.0",
            description: "Custom LLVM IR optimization pass",
        }
    }

    fn init(&mut self, _config: &PluginConfig) -> Result<(), String> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl OptimizePlugin for CustomOptimizer {
    fn optimize(&self, ir: &str) -> Result<String, String> {
        // 예: 특정 패턴을 더 효율적인 코드로 변환
        let result = ir.to_string();
        // ... 최적화 로직 ...
        Ok(result)
    }

    fn opt_level(&self) -> OptLevel {
        OptLevel::O2  // O2 이상에서만 실행
    }
}

#[no_mangle]
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(CustomOptimizer)
}
```

### 4.3 Codegen 플러그인 (Python 바인딩 생성)

```rust
// plugins/python-bindgen/src/lib.rs

use vais_plugin::{Plugin, CodegenPlugin, PluginInfo, PluginConfig};
use vais_ast::{Module, Item, Function};
use std::path::{Path, PathBuf};
use std::fs;

pub struct PythonBindgenPlugin {
    output_dir: PathBuf,
}

impl Plugin for PythonBindgenPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "python-bindgen",
            version: "0.1.0",
            description: "Generates Python bindings for Vais functions",
        }
    }

    fn init(&mut self, config: &PluginConfig) -> Result<(), String> {
        if let Some(dir) = config.values.get("output_dir") {
            self.output_dir = PathBuf::from(
                dir.as_str().ok_or("output_dir must be string")?
            );
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CodegenPlugin for PythonBindgenPlugin {
    fn generate(&self, module: &Module, _output_dir: &Path) -> Result<Vec<PathBuf>, String> {
        let mut output = String::from("# Auto-generated Python bindings\n\n");
        output.push_str("import ctypes\n\n");

        for item in &module.items {
            if let Item::Function(func) = &item.node {
                if func.is_pub {
                    output.push_str(&generate_python_binding(func));
                }
            }
        }

        let binding_path = self.output_dir.join("bindings.py");
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("Cannot create output dir: {}", e))?;
        fs::write(&binding_path, output)
            .map_err(|e| format!("Cannot write bindings: {}", e))?;

        Ok(vec![binding_path])
    }
}

fn generate_python_binding(func: &Function) -> String {
    format!(
        "def {}({}):\n    # TODO: implement FFI call\n    pass\n\n",
        func.name.node,
        func.params.iter()
            .map(|p| p.node.name.node.clone())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[no_mangle]
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(PythonBindgenPlugin {
        output_dir: PathBuf::from("./bindings"),
    })
}
```

---

## 5. CLI 통합

### 5.1 main.rs 수정

```rust
// crates/vaisc/src/main.rs

use vais_plugin::{PluginRegistry, PluginsConfig};

fn main() {
    let cli = Cli::parse();

    // 플러그인 로드
    let mut plugins = PluginRegistry::new();
    if let Some(config_path) = find_plugin_config() {
        if let Ok(config) = PluginsConfig::load(&config_path) {
            if let Err(e) = plugins.load_from_config(&config) {
                eprintln!("Warning: Failed to load plugins: {}", e);
            }
        }
    }

    let result = match cli.command {
        Some(Commands::Build { input, output, .. }) => {
            cmd_build_with_plugins(&input, output, &plugins, cli.verbose)
        }
        // ... 기존 명령들
    };

    // ...
}

fn cmd_build_with_plugins(
    input: &PathBuf,
    output: Option<PathBuf>,
    plugins: &PluginRegistry,
    verbose: bool,
) -> Result<(), String> {
    // 1. 소스 로드
    let source = fs::read_to_string(input)?;
    let ast = parse(&source)?;

    // 2. Lint 플러그인 실행
    let diagnostics = plugins.run_lint(&ast);
    for diag in &diagnostics {
        print_diagnostic(diag);
    }
    if diagnostics.iter().any(|d| d.level == DiagnosticLevel::Error) {
        return Err("Lint errors found".to_string());
    }

    // 3. Transform 플러그인 실행
    let transformed = plugins.run_transform(ast)?;

    // 4. 타입 체크
    let mut checker = TypeChecker::new();
    checker.check_module(&transformed)?;

    // 5. 코드 생성
    let mut codegen = CodeGenerator::new(module_name);
    let raw_ir = codegen.generate_module(&transformed)?;

    // 6. Optimize 플러그인 실행
    let optimized = plugins.run_optimize(&raw_ir, opt_level)?;

    // 7. 기존 최적화 패스
    let ir = optimize_ir(&optimized, opt_level);

    // 8. Codegen 플러그인 실행
    let output_dir = output.as_ref()
        .and_then(|p| p.parent())
        .unwrap_or(Path::new("."));
    let generated = plugins.run_codegen(&transformed, output_dir)?;
    for path in &generated {
        if verbose {
            println!("Generated: {}", path.display());
        }
    }

    // 9. IR 저장 및 컴파일
    // ... 기존 로직
}

fn find_plugin_config() -> Option<PathBuf> {
    let names = ["vais-plugins.toml", ".vais-plugins.toml"];
    let mut dir = std::env::current_dir().ok()?;

    loop {
        for name in &names {
            let path = dir.join(name);
            if path.exists() {
                return Some(path);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    None
}
```

### 5.2 CLI 옵션 추가

```rust
#[derive(Parser)]
struct Cli {
    // ... 기존 옵션들

    /// Disable all plugins
    #[arg(long)]
    no_plugins: bool,

    /// Load additional plugin
    #[arg(long, value_name = "PATH")]
    plugin: Vec<PathBuf>,
}
```

---

## 6. 의존성

```toml
# crates/vais-plugin/Cargo.toml
[package]
name = "vais-plugin"
version = "0.0.1"
edition = "2021"

[dependencies]
vais-ast = { path = "../vais-ast" }
libloading = "0.8"
toml = "0.8"
serde = { version = "1.0", features = ["derive"] }
```

---

## 7. 구현 계획

| 단계 | 작업 | 추천 모델 | 예상 라인 |
|------|------|----------|----------|
| 1 | vais-plugin 크레이트 생성 (traits) | Sonnet | ~200 |
| 2 | 플러그인 로더 구현 | Sonnet | ~150 |
| 3 | 레지스트리 구현 | Sonnet | ~200 |
| 4 | 설정 파일 파서 | Haiku | ~100 |
| 5 | CLI 통합 | Sonnet | ~150 |
| 6 | 예제 플러그인 (complexity-lint) | Sonnet | ~150 |
| 7 | 테스트 및 검증 | Opus | ~100 |

---

## 8. 보안 고려사항

### 8.1 플러그인 실행 환경
- 플러그인은 컴파일러와 같은 프로세스에서 실행
- unsafe 코드 실행 가능 (동적 로딩 특성)
- 신뢰할 수 있는 플러그인만 사용 권장

### 8.2 안전 장치
- 플러그인 서명 검증 (향후 구현)
- 허용된 경로에서만 플러그인 로드
- 타임아웃 설정 (무한 루프 방지)

---

## 9. 향후 확장

### 9.1 플러그인 패키지 관리자
```bash
vaisc plugin install vais-lint-complexity
vaisc plugin list
vaisc plugin remove vais-lint-complexity
```

### 9.2 플러그인 레지스트리 (온라인)
- 중앙 플러그인 저장소
- 버전 관리 및 의존성 해결
- 보안 검증

### 9.3 WASM 플러그인
- WebAssembly 기반 샌드박스 실행
- 더 안전한 플러그인 실행
- 크로스 플랫폼 호환성

---

## 10. 결론

이 플러그인 시스템은 Vais 컴파일러의 확장성을 크게 향상시킵니다:
- Lint, Transform, Optimize, Codegen 4가지 확장 포인트
- 동적 로딩으로 컴파일러 재빌드 없이 기능 추가
- 설정 파일 기반으로 프로젝트별 플러그인 관리
