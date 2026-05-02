use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use vais_mir::lower::{lower_module_checked, MirLowerError};
use vais_mir::validate::{validate_module, MirValidationError};

#[derive(Debug, Clone)]
struct ManifestEntry {
    path: PathBuf,
    note: String,
}

#[test]
fn strict_core_fixture_subset_lowers_and_validates() {
    let entries = read_manifest("tests/core/mir_strict.tsv", "strict MIR fixture");
    assert!(
        !entries.is_empty(),
        "strict MIR fixture manifest must not be empty"
    );

    for entry in entries {
        let source_path = compiler_root().join(&entry.path);
        let source = fs::read_to_string(&source_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", source_path.display(), e));
        let module = vais_parser::parse(&source)
            .unwrap_or_else(|e| panic!("failed to parse {}: {}", source_path.display(), e));
        let mir = lower_module_checked(&module).unwrap_or_else(|errors| {
            panic!(
                "{} [{}] failed strict MIR lowering:\n{}",
                entry.path.display(),
                entry.note,
                format_lower_errors(&errors)
            )
        });

        validate_module(&mir).unwrap_or_else(|errors| {
            panic!(
                "{} [{}] produced invalid MIR:\n{}",
                entry.path.display(),
                entry.note,
                format_validation_errors(&errors)
            )
        });
    }
}

#[test]
fn deferred_core_fixtures_are_explicit_and_disjoint() {
    let strict = read_manifest("tests/core/mir_strict.tsv", "strict MIR fixture");
    let deferred = read_manifest("tests/core/mir_deferred.tsv", "deferred MIR fixture");

    let strict_paths: BTreeSet<PathBuf> = strict.into_iter().map(|entry| entry.path).collect();
    let mut deferred_paths = BTreeSet::new();

    for entry in deferred {
        let source_path = compiler_root().join(&entry.path);
        assert!(
            source_path.exists(),
            "deferred MIR fixture {} must exist",
            source_path.display()
        );
        assert!(
            !strict_paths.contains(&entry.path),
            "{} cannot be both strict and deferred",
            entry.path.display()
        );
        assert!(
            deferred_paths.insert(entry.path.clone()),
            "duplicate deferred MIR fixture {}",
            entry.path.display()
        );
        assert!(
            !entry.note.trim().is_empty(),
            "deferred MIR fixture {} must explain why it is not certified yet",
            entry.path.display()
        );
    }
}

#[test]
fn deferred_core_fixtures_do_not_silently_lower_strictly() {
    let deferred = read_manifest("tests/core/mir_deferred.tsv", "deferred MIR fixture");

    for entry in deferred {
        let source_path = compiler_root().join(&entry.path);
        let source = fs::read_to_string(&source_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", source_path.display(), e));
        let module = vais_parser::parse(&source)
            .unwrap_or_else(|e| panic!("failed to parse {}: {}", source_path.display(), e));
        let errors = match lower_module_checked(&module) {
            Ok(_) => {
                panic!(
                    "{} [{}] must remain outside strict MIR until its deferred reason is resolved",
                    entry.path.display(),
                    entry.note
                )
            }
            Err(errors) => errors,
        };

        assert!(
            !errors.is_empty(),
            "{} [{}] must report at least one strict lowering error",
            entry.path.display(),
            entry.note
        );
    }
}

fn read_manifest(path: &str, label: &str) -> Vec<ManifestEntry> {
    let manifest_path = compiler_root().join(path);
    let contents = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", manifest_path.display(), e));
    let mut entries = Vec::new();

    for (idx, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() != 2 {
            panic!(
                "invalid {} line {} in {}: expected 2 tab-separated columns, got {}",
                label,
                idx + 1,
                manifest_path.display(),
                cols.len()
            );
        }

        entries.push(ManifestEntry {
            path: PathBuf::from(cols[0]),
            note: cols[1].to_string(),
        });
    }

    entries
}

fn compiler_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to resolve compiler root")
}

fn format_lower_errors(errors: &[MirLowerError]) -> String {
    errors
        .iter()
        .map(|error| format!("  - {}", error))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_validation_errors(errors: &[MirValidationError]) -> String {
    errors
        .iter()
        .map(|error| format!("  - {}", error))
        .collect::<Vec<_>>()
        .join("\n")
}
