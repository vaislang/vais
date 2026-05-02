use std::collections::{hash_map::DefaultHasher, BTreeMap, HashMap};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use vais_mir::interpreter::{interpret_function, MirValue};
use vais_mir::lower::lower_module_checked;
use vais_mir::validate::validate_module;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expect {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Check,
    Codegen,
    Run,
}

#[derive(Debug)]
struct Case {
    path: PathBuf,
    expect: Expect,
    stage: Stage,
    error: Option<String>,
    exit: Option<i32>,
    description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExclusionKind {
    Ignore,
    Partial,
}

#[derive(Debug)]
struct ExclusionCase {
    kind: ExclusionKind,
    path: PathBuf,
    test: String,
    reason: String,
}

const CERTIFICATION_EXCLUSION_AUDIT_FILES: &[&str] = &[
    "crates/vais-codegen/tests/call_arg_invariant_test.rs",
    "crates/vaisc/tests/integrity/compiler_stages.rs",
    "crates/vaisc/tests/integrity/compiler_syntax.rs",
];

#[test]
fn core_certification_manifest() {
    setup_std_symlink();
    let root = compiler_root();
    let fixture_root = root.join("tests/core");
    let manifest = fixture_root.join("manifest.tsv");
    let cases = read_manifest(&manifest);

    let mut failures = Vec::new();
    for case in &cases {
        let full_path = fixture_root.join(&case.path);
        if !full_path.exists() {
            failures.push(format!("missing fixture {}", full_path.display()));
            continue;
        }

        if let Err(err) = run_case(case, &full_path) {
            failures.push(format!(
                "{} [{}]: {}",
                case.path.display(),
                case.description,
                err
            ));
        }
    }

    eprintln!(
        "CORE_CERTIFICATION pass={} fail={} total={}",
        cases.len().saturating_sub(failures.len()),
        failures.len(),
        cases.len()
    );

    if !failures.is_empty() {
        panic!("Core certification failures:\n{}", failures.join("\n"));
    }
}

#[test]
fn core_mir_interpreter_matches_native_for_strict_subset() {
    setup_std_symlink();
    let root = compiler_root();
    let fixture_root = root.join("tests/core");
    let core_cases = read_manifest(&fixture_root.join("manifest.tsv"));
    let core_case_by_path: HashMap<PathBuf, &Case> = core_cases
        .iter()
        .map(|case| (case.path.clone(), case))
        .collect();
    let strict_entries = read_mir_strict_manifest(&root.join("tests/core/mir_strict.tsv"));

    let mut failures = Vec::new();
    for strict_path in strict_entries {
        let core_relative = strict_path
            .strip_prefix("tests/core")
            .unwrap_or(&strict_path)
            .to_path_buf();
        let Some(case) = core_case_by_path.get(&core_relative) else {
            failures.push(format!(
                "{} is in mir_strict.tsv but not tests/core/manifest.tsv",
                strict_path.display()
            ));
            continue;
        };

        if case.expect != Expect::Pass || case.stage != Stage::Run {
            failures.push(format!(
                "{} is strict-MIR certified but not a positive run fixture",
                strict_path.display()
            ));
            continue;
        }

        let full_path = root.join(&strict_path);
        if let Err(err) = compare_mir_and_native(case, &full_path) {
            failures.push(format!("{}: {}", strict_path.display(), err));
        }
    }

    if !failures.is_empty() {
        panic!(
            "Core MIR/native comparison failures:\n{}",
            failures.join("\n")
        );
    }
}

#[test]
fn core_certification_exclusion_manifest_is_current() {
    let root = compiler_root();
    let entries = read_exclusion_manifest(&root.join("tests/core/certification_exclusions.tsv"));
    let mut expected_ignores = BTreeMap::new();
    let mut expected_partials = BTreeMap::new();
    let mut failures = Vec::new();

    for entry in entries {
        let full_path = root.join(&entry.path);
        if !full_path.exists() {
            failures.push(format!(
                "exclusion source file is missing: {}",
                full_path.display()
            ));
            continue;
        }
        let key = (entry.path.clone(), entry.test.clone());
        match entry.kind {
            ExclusionKind::Ignore => {
                expected_ignores.insert(key, entry.reason.clone());
            }
            ExclusionKind::Partial => {
                expected_partials.insert(key, entry.reason.clone());
            }
        }
    }

    let actual_ignores = collect_certification_ignores(&root);
    for (key, actual_reason) in &actual_ignores {
        let Some(expected_reason) = expected_ignores.get(key) else {
            failures.push(format!(
                "untracked #[ignore] in certification surface: {}::{} ({})",
                key.0.display(),
                key.1,
                actual_reason
            ));
            continue;
        };
        if !actual_reason.contains(expected_reason) {
            failures.push(format!(
                "{}::{} ignore reason changed: expected substring '{}', actual '{}'",
                key.0.display(),
                key.1,
                expected_reason,
                actual_reason
            ));
        }
    }
    for key in expected_ignores.keys() {
        if !actual_ignores.contains_key(key) {
            failures.push(format!(
                "manifest lists ignored test that is no longer ignored: {}::{}",
                key.0.display(),
                key.1
            ));
        }
    }

    let actual_partials = collect_certification_partials(&root);
    for (key, actual_reason) in &actual_partials {
        let Some(expected_reason) = expected_partials.get(key) else {
            failures.push(format!(
                "untracked partial gate in certification surface: {}::{} ({})",
                key.0.display(),
                key.1,
                actual_reason
            ));
            continue;
        };
        if !actual_reason.contains(expected_reason) {
            failures.push(format!(
                "{}::{} partial marker changed: expected substring '{}', actual '{}'",
                key.0.display(),
                key.1,
                expected_reason,
                actual_reason
            ));
        }
    }
    for key in expected_partials.keys() {
        if !actual_partials.contains_key(key) {
            failures.push(format!(
                "manifest lists partial gate that is no longer partial: {}::{}",
                key.0.display(),
                key.1
            ));
        }
    }

    let deferred = read_non_comment_lines(&root.join("tests/core/mir_deferred.tsv"));
    if !deferred.is_empty() {
        failures.push(format!(
            "MIR deferred manifest must stay empty for Core v0 certification; entries:\n{}",
            deferred.join("\n")
        ));
    }

    if !failures.is_empty() {
        panic!(
            "Core certification exclusion audit failures:\n{}",
            failures.join("\n")
        );
    }
}

#[test]
fn core_freeze_criteria_doc_is_current() {
    let root = compiler_root();
    let manifest_cases = read_manifest(&root.join("tests/core/manifest.tsv"));
    let freeze_path = root.join("docs/certification/CORE_FREEZE_CRITERIA.md");
    let status_path = root.join("docs/certification/CURRENT_STATUS.md");
    let freeze = fs::read_to_string(&freeze_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", freeze_path.display(), e));
    let status = fs::read_to_string(&status_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", status_path.display(), e));
    let core_summary = format!(
        "CORE_CERTIFICATION pass={} fail=0 total={}",
        manifest_cases.len(),
        manifest_cases.len()
    );

    for expected in [
        "cargo test --release",
        "bash scripts/core-certify.sh",
        "bash scripts/check-integrity.sh",
        "CORE OK",
        "MIR OK",
        "CODEGEN OK",
        "ECOSYSTEM OK: syntax=200/? stages=14/? std=82/82 vaisdb=261/261",
        "BACKEND OK: phase158=18/18",
        "VAISDB RUNTIME OK: smoke=28/28",
        "git diff --check",
        "tests/core/mir_deferred.tsv",
        "tests/core/certification_exclusions.tsv",
        "error[CODE]",
        "does not mean that every Vais feature is",
        "Product work may resume in this order",
    ] {
        assert!(
            freeze.contains(expected),
            "CORE_FREEZE_CRITERIA.md is missing required freeze token: {expected}"
        );
    }

    assert!(
        freeze.contains(&core_summary),
        "CORE_FREEZE_CRITERIA.md must track current manifest summary {core_summary}"
    );
    assert!(
        status.contains("docs/certification/CORE_FREEZE_CRITERIA.md"),
        "CURRENT_STATUS.md must list CORE_FREEZE_CRITERIA.md as an active source of truth"
    );
}

#[test]
fn core_freeze_decision_doc_is_current() {
    let root = compiler_root();
    let manifest_cases = read_manifest(&root.join("tests/core/manifest.tsv"));
    let decision_path = root.join("docs/certification/CORE_FREEZE_DECISION.md");
    let status_path = root.join("docs/certification/CURRENT_STATUS.md");
    let roadmap_path = root.join("ROADMAP.md");
    let decision = fs::read_to_string(&decision_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", decision_path.display(), e));
    let status = fs::read_to_string(&status_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", status_path.display(), e));
    let roadmap = fs::read_to_string(&roadmap_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", roadmap_path.display(), e));
    let core_summary = format!(
        "CORE_CERTIFICATION pass={} fail=0 total={}",
        manifest_cases.len(),
        manifest_cases.len()
    );

    for expected in [
        "Status: Frozen for downstream re-entry",
        "certified Vais Core compiler is frozen for downstream re-entry",
        "not a claim that every Vais language feature",
        "CORE_FREEZE_CRITERIA.md",
        "cargo test --release",
        "bash scripts/core-certify.sh",
        "bash scripts/check-integrity.sh",
        "CORE OK",
        "MIR OK",
        "CODEGEN OK",
        "ECOSYSTEM OK: syntax=200/? stages=14/? std=82/82 vaisdb=261/261",
        "BACKEND OK: phase158=18/18",
        "VAISDB RUNTIME OK: smoke=28/28",
        "git diff --check",
        "Product work may resume in this order",
        "Only compiler regressions should modify the frozen Core compiler path",
    ] {
        assert!(
            decision.contains(expected),
            "CORE_FREEZE_DECISION.md is missing required freeze token: {expected}"
        );
    }

    assert!(
        decision.contains(&core_summary),
        "CORE_FREEZE_DECISION.md must track current manifest summary {core_summary}"
    );
    assert!(
        status.contains("docs/certification/CORE_FREEZE_DECISION.md"),
        "CURRENT_STATUS.md must list CORE_FREEZE_DECISION.md as an active source of truth"
    );
    assert!(
        roadmap.contains("Status: Certified Core frozen for downstream re-entry"),
        "compiler ROADMAP must reflect the active Core freeze decision"
    );
}

fn read_manifest(path: &Path) -> Vec<Case> {
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read manifest {}: {}", path.display(), e));
    let mut cases = Vec::new();

    for (idx, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() != 6 {
            panic!(
                "invalid manifest line {} in {}: expected 6 tab-separated columns, got {}",
                idx + 1,
                path.display(),
                cols.len()
            );
        }
        cases.push(Case {
            path: PathBuf::from(cols[0]),
            expect: parse_expect(cols[1], idx + 1),
            stage: parse_stage(cols[2], idx + 1),
            error: parse_optional_string(cols[3]),
            exit: parse_optional_i32(cols[4], idx + 1),
            description: cols[5].to_string(),
        });
    }

    assert!(
        !cases.is_empty(),
        "core manifest must contain at least one case"
    );
    cases
}

fn read_exclusion_manifest(path: &Path) -> Vec<ExclusionCase> {
    let contents = fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "failed to read certification exclusion manifest {}: {}",
            path.display(),
            e
        )
    });
    let mut entries = Vec::new();

    for (idx, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() != 4 {
            panic!(
                "invalid exclusion manifest line {} in {}: expected 4 tab-separated columns, got {}",
                idx + 1,
                path.display(),
                cols.len()
            );
        }
        let kind = match cols[0] {
            "ignore" => ExclusionKind::Ignore,
            "partial" => ExclusionKind::Partial,
            other => panic!("invalid exclusion kind '{}' on line {}", other, idx + 1),
        };
        if cols[3].trim().is_empty() {
            panic!("exclusion reason must not be empty on line {}", idx + 1);
        }
        entries.push(ExclusionCase {
            kind,
            path: PathBuf::from(cols[1]),
            test: cols[2].to_string(),
            reason: cols[3].to_string(),
        });
    }

    entries
}

fn read_mir_strict_manifest(path: &Path) -> Vec<PathBuf> {
    let contents = fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "failed to read MIR strict manifest {}: {}",
            path.display(),
            e
        )
    });
    let mut paths = Vec::new();

    for (idx, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() != 2 {
            panic!(
                "invalid MIR strict manifest line {} in {}: expected 2 columns, got {}",
                idx + 1,
                path.display(),
                cols.len()
            );
        }
        paths.push(PathBuf::from(cols[0]));
    }

    assert!(
        !paths.is_empty(),
        "MIR strict manifest must contain at least one case"
    );
    paths
}

fn collect_certification_ignores(root: &Path) -> BTreeMap<(PathBuf, String), String> {
    let mut ignored = BTreeMap::new();
    for relative in CERTIFICATION_EXCLUSION_AUDIT_FILES {
        let relative_path = PathBuf::from(relative);
        let contents = fs::read_to_string(root.join(&relative_path)).unwrap_or_else(|e| {
            panic!(
                "failed to read certification audit file {}: {}",
                relative, e
            )
        });
        let mut pending_ignore = None;
        for raw_line in contents.lines() {
            let line = raw_line.trim();
            if let Some(reason) = parse_ignore_reason(line) {
                pending_ignore = Some(reason);
                continue;
            }
            if let Some(name) = parse_fn_name(line) {
                if let Some(reason) = pending_ignore.take() {
                    ignored.insert((relative_path.clone(), name), reason);
                }
            }
        }
    }
    ignored
}

fn collect_certification_partials(root: &Path) -> BTreeMap<(PathBuf, String), String> {
    let mut partials = BTreeMap::new();
    for relative in CERTIFICATION_EXCLUSION_AUDIT_FILES {
        let relative_path = PathBuf::from(relative);
        let contents = fs::read_to_string(root.join(&relative_path)).unwrap_or_else(|e| {
            panic!(
                "failed to read certification audit file {}: {}",
                relative, e
            )
        });
        let mut current_fn = None;
        for raw_line in contents.lines() {
            let line = raw_line.trim();
            if let Some(name) = parse_fn_name(line) {
                current_fn = Some(name);
            }
            if line.contains("B3 partial") {
                if let Some(name) = &current_fn {
                    partials.insert(
                        (relative_path.clone(), name.clone()),
                        "B3 partial".to_string(),
                    );
                }
            }
        }
    }
    partials
}

fn parse_ignore_reason(line: &str) -> Option<String> {
    if !line.starts_with("#[ignore") {
        return None;
    }
    if let Some(start) = line.find('"') {
        if let Some(end) = line[start + 1..].find('"') {
            return Some(line[start + 1..start + 1 + end].to_string());
        }
    }
    Some("<missing ignore reason>".to_string())
}

fn parse_fn_name(line: &str) -> Option<String> {
    let rest = line
        .strip_prefix("fn ")
        .or_else(|| line.strip_prefix("pub fn "))?;
    let name = rest
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .next()
        .unwrap_or_default();
    (!name.is_empty()).then(|| name.to_string())
}

fn read_non_comment_lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e))
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn parse_expect(raw: &str, line: usize) -> Expect {
    match raw {
        "pass" => Expect::Pass,
        "fail" => Expect::Fail,
        other => panic!("invalid expect '{}' on manifest line {}", other, line),
    }
}

fn parse_stage(raw: &str, line: usize) -> Stage {
    match raw {
        "check" => Stage::Check,
        "codegen" => Stage::Codegen,
        "run" => Stage::Run,
        other => panic!("invalid stage '{}' on manifest line {}", other, line),
    }
}

fn parse_optional_string(raw: &str) -> Option<String> {
    if raw == "-" {
        None
    } else {
        Some(raw.to_string())
    }
}

fn parse_optional_i32(raw: &str, line: usize) -> Option<i32> {
    if raw == "-" {
        None
    } else {
        Some(
            raw.parse::<i32>()
                .unwrap_or_else(|_| panic!("invalid exit code '{}' on line {}", raw, line)),
        )
    }
}

fn run_case(case: &Case, path: &Path) -> Result<(), String> {
    match case.expect {
        Expect::Pass => run_positive(case, path),
        Expect::Fail => run_negative(case, path),
    }
}

fn run_positive(case: &Case, path: &Path) -> Result<(), String> {
    match case.stage {
        Stage::Check => {
            let out = run_check(path)?;
            require_success(&out, "check")
        }
        Stage::Codegen => {
            let out = run_codegen(path)?;
            require_success(&out, "codegen")
        }
        Stage::Run => {
            let output = run_build(path)?;
            require_success(&output.build, "build")?;
            let run = Command::new(&output.exe_path)
                .output()
                .map_err(|e| format!("failed to run executable: {}", e))?;
            let _ = fs::remove_file(&output.exe_path);
            let expected = case
                .exit
                .ok_or_else(|| "run pass case must declare expected exit code".to_string())?;
            let actual = run.status.code().unwrap_or(-1);
            if actual != expected {
                return Err(format!(
                    "run exit mismatch: expected {}, got {}\nstdout:\n{}\nstderr:\n{}",
                    expected,
                    actual,
                    String::from_utf8_lossy(&run.stdout),
                    String::from_utf8_lossy(&run.stderr)
                ));
            }
            Ok(())
        }
    }
}

fn run_negative(case: &Case, path: &Path) -> Result<(), String> {
    let out = match case.stage {
        Stage::Check => run_check(path)?,
        Stage::Codegen => run_codegen(path)?,
        Stage::Run => run_build(path)?.build,
    };

    if out.status.success() {
        return Err(format!(
            "{} unexpectedly succeeded; expected failure{}",
            stage_name(case.stage),
            case.error
                .as_ref()
                .map(|e| format!(" containing {}", e))
                .unwrap_or_default()
        ));
    }

    let error = case.error.as_deref().ok_or_else(|| {
        "negative Core fixtures must declare a stable diagnostic code".to_string()
    })?;
    require_core_diagnostic_code(case, error)?;

    let combined = combined_output(&out);
    require_structured_diagnostic_header(&combined, error)?;
    if !combined.contains(error) {
        return Err(format!(
            "expected failure output to contain '{}'\nactual output:\n{}",
            error, combined
        ));
    }

    Ok(())
}

fn require_core_diagnostic_code(case: &Case, code: &str) -> Result<(), String> {
    let mut chars = code.chars();
    let Some(prefix) = chars.next() else {
        return Err("diagnostic code must not be empty".to_string());
    };
    let suffix: String = chars.collect();
    if suffix.len() != 3 || !suffix.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(format!(
            "diagnostic code '{}' must use a stable family plus three digits",
            code
        ));
    }

    let allowed = match case.stage {
        Stage::Check => matches!(prefix, 'P' | 'E'),
        Stage::Codegen | Stage::Run => matches!(prefix, 'P' | 'E' | 'C'),
    };
    if !allowed {
        return Err(format!(
            "diagnostic code '{}' is not valid for {} stage",
            code,
            stage_name(case.stage)
        ));
    }

    Ok(())
}

fn require_structured_diagnostic_header(output: &str, code: &str) -> Result<(), String> {
    let header = format!("error[{code}]");
    if output.contains(&header) || strip_ansi_codes(output).contains(&header) {
        return Ok(());
    }

    Err(format!(
        "expected failure output to contain structured diagnostic header '{}'\nactual output:\n{}",
        header, output
    ))
}

fn strip_ansi_codes(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && matches!(chars.peek(), Some('[')) {
            chars.next();
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
        } else {
            output.push(ch);
        }
    }

    output
}

fn compare_mir_and_native(case: &Case, path: &Path) -> Result<(), String> {
    let output = run_build(path)?;
    require_success(&output.build, "build")?;
    let run = Command::new(&output.exe_path)
        .output()
        .map_err(|e| format!("failed to run executable: {}", e))?;
    let _ = fs::remove_file(&output.exe_path);

    let native_exit = run.status.code().unwrap_or(-1);
    let expected_exit = case
        .exit
        .ok_or_else(|| "strict MIR run case must declare expected exit code".to_string())?;
    if native_exit != expected_exit {
        return Err(format!(
            "native run exit mismatch before MIR comparison: expected {}, got {}\nstdout:\n{}\nstderr:\n{}",
            expected_exit,
            native_exit,
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        ));
    }

    let source = fs::read_to_string(path)
        .map_err(|e| format!("failed to read source for MIR comparison: {}", e))?;
    let module =
        vais_parser::parse(&source).map_err(|e| format!("failed to parse source: {}", e))?;
    let mir = lower_module_checked(&module).map_err(|errors| {
        format!(
            "strict MIR lowering failed:\n{}",
            errors
                .iter()
                .map(|error| format!("  - {}", error))
                .collect::<Vec<_>>()
                .join("\n")
        )
    })?;
    validate_module(&mir).map_err(|errors| {
        format!(
            "MIR validation failed:\n{}",
            errors
                .iter()
                .map(|error| format!("  - {}", error))
                .collect::<Vec<_>>()
                .join("\n")
        )
    })?;
    let interpreted = interpret_function(&mir, "main", vec![])
        .map_err(|e| format!("MIR interpretation failed: {}", e))?;
    let interpreted_exit = mir_value_to_exit_code(&interpreted)?;

    if interpreted_exit != native_exit {
        return Err(format!(
            "MIR/native exit mismatch: MIR {:?} -> {}, native {}",
            interpreted, interpreted_exit, native_exit
        ));
    }

    Ok(())
}

fn mir_value_to_exit_code(value: &MirValue) -> Result<i32, String> {
    match value {
        MirValue::Int(value) => Ok(value.rem_euclid(256) as i32),
        MirValue::Bool(value) => Ok(i32::from(*value)),
        MirValue::Unit => Ok(0),
        other => Err(format!(
            "cannot convert MIR value {:?} to process exit code",
            other
        )),
    }
}

fn require_success(out: &Output, label: &str) -> Result<(), String> {
    if out.status.success() {
        Ok(())
    } else {
        Err(format!("{} failed\n{}", label, combined_output(out)))
    }
}

fn run_check(path: &Path) -> Result<Output, String> {
    let out = Command::new(vaisc_path())
        .arg("check")
        .arg(path)
        .env("VAIS_STD_PATH", std_path())
        .env("VAIS_DEP_PATHS", std_path())
        .env("VAIS_CORE_CERTIFY", "1")
        .output()
        .map_err(|e| format!("failed to spawn vaisc check: {}", e));
    cleanup_generated_files(path);
    out
}

fn run_codegen(path: &Path) -> Result<Output, String> {
    let ir_path = unique_temp_path(path, "ll");
    let out = Command::new(vaisc_path())
        .arg("build")
        .arg(path)
        .arg("--emit-ir")
        .arg("-o")
        .arg(&ir_path)
        .arg("--force-rebuild")
        .env("VAIS_STD_PATH", std_path())
        .env("VAIS_DEP_PATHS", std_path())
        .env("VAIS_CORE_CERTIFY", "1")
        .output()
        .map_err(|e| format!("failed to spawn vaisc build --emit-ir: {}", e));
    let _ = fs::remove_file(&ir_path);
    cleanup_generated_files(path);
    out
}

struct BuildOutput {
    build: Output,
    exe_path: PathBuf,
}

fn run_build(path: &Path) -> Result<BuildOutput, String> {
    let exe_path = unique_temp_path(path, "exe");
    let build = Command::new(vaisc_path())
        .arg("build")
        .arg(path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .env("VAIS_STD_PATH", std_path())
        .env("VAIS_DEP_PATHS", std_path())
        .env("VAIS_CORE_CERTIFY", "1")
        .output()
        .map_err(|e| format!("failed to spawn vaisc build: {}", e))?;
    cleanup_generated_files(path);
    Ok(BuildOutput { build, exe_path })
}

fn cleanup_generated_files(path: &Path) {
    let _ = fs::remove_file(path.with_extension("ll"));
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir_all(parent.join(".vais-cache"));
    }
}

fn combined_output(out: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn stage_name(stage: Stage) -> &'static str {
    match stage {
        Stage::Check => "check",
        Stage::Codegen => "codegen",
        Stage::Run => "run",
    }
}

fn compiler_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to canonicalize compiler root")
}

fn vaisc_path() -> PathBuf {
    if let Some(path) = option_env!("CARGO_BIN_EXE_vaisc") {
        if !path.is_empty() {
            return PathBuf::from(path);
        }
    }
    if let Ok(path) = std::env::var("VAISC") {
        return PathBuf::from(path);
    }
    compiler_root().join("target/debug/vaisc")
}

fn std_path() -> &'static str {
    "/tmp/vais-lib/std"
}

fn setup_std_symlink() {
    let std_src = compiler_root().join("std");
    let link_dir = PathBuf::from("/tmp/vais-lib");
    let link_target = link_dir.join("std");

    if !link_dir.exists() {
        fs::create_dir_all(&link_dir).expect("failed to create /tmp/vais-lib");
    }

    if link_target.exists() {
        if let Ok(real) = link_target.canonicalize() {
            if real == std_src {
                return;
            }
        }
        let _ = fs::remove_file(&link_target);
        let _ = fs::remove_dir_all(&link_target);
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&std_src, &link_target).expect("failed to create std symlink");
    }
}

fn unique_temp_path(path: &Path, ext: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    format!("{:?}", std::thread::current().id()).hash(&mut hasher);
    PathBuf::from(format!("/tmp/vais_core_{:016x}.{}", hasher.finish(), ext))
}
