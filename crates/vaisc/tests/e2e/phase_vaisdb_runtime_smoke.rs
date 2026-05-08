use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

static VAISDB_SMOKE_LOCK: Mutex<()> = Mutex::new(());
static VAISDB_SMOKE_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn e2e_vaisdb_00_sql_build_plan_catalog_runtime_smoke() {
    assert_vaisdb_smoke_runs("sql_build_plan_catalog_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_01_sql_catalog_schema_runtime_smoke() {
    assert_vaisdb_smoke_runs("sql_catalog_schema_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_02_sql_parser_planner_runtime_smoke() {
    assert_vaisdb_smoke_runs("sql_parser_planner_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_03_sql_planner_precision_runtime_smoke() {
    assert_vaisdb_smoke_runs("sql_planner_precision_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_04_sql_planner_format_runtime_smoke() {
    assert_vaisdb_smoke_runs("sql_planner_format_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_05_storage_bytes_runtime_smoke() {
    assert_vaisdb_smoke_runs("storage_bytes_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_06_vector_distance_runtime_smoke() {
    assert_vaisdb_smoke_runs("vector_distance_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_10_hnsw_bulk_runtime_smoke() {
    assert_vaisdb_smoke_runs("hnsw_bulk_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_11_hnsw_delete_runtime_smoke() {
    assert_vaisdb_smoke_runs("hnsw_delete_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_12_hnsw_insert_runtime_smoke() {
    assert_vaisdb_smoke_runs("hnsw_insert_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_13_hnsw_node_runtime_smoke() {
    assert_vaisdb_smoke_runs("hnsw_node_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_14_hnsw_search_runtime_smoke() {
    assert_vaisdb_smoke_runs("hnsw_search_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_15_hnsw_wal_runtime_smoke() {
    assert_vaisdb_smoke_runs("hnsw_wal_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_16_vector_engine_bulk_runtime_smoke() {
    assert_vaisdb_smoke_runs("vector_engine_bulk_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_17_hnsw_multilayer_recall_runtime_smoke() {
    assert_vaisdb_smoke_runs("hnsw_multilayer_recall_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_18_hnsw_bulk_wal_runtime_smoke() {
    assert_vaisdb_smoke_runs("hnsw_bulk_wal_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_19_vector_storage_wal_runtime_smoke() {
    assert_vaisdb_smoke_runs("vector_storage_wal_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_20_vector_storage_overflow_wal_runtime_smoke() {
    assert_vaisdb_smoke_runs("vector_storage_overflow_wal_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_21_vector_data_write_redo_runtime_smoke() {
    assert_vaisdb_smoke_runs("vector_data_write_redo_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_22_vector_recovery_dispatch_redo_runtime_smoke() {
    assert_vaisdb_smoke_runs("vector_recovery_dispatch_redo_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_23_vector_recovery_segment_replay_runtime_smoke() {
    assert_vaisdb_smoke_runs("vector_recovery_segment_replay_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_24_vector_recovery_checkpoint_replay_runtime_smoke() {
    assert_vaisdb_smoke_runs("vector_recovery_checkpoint_replay_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_25_checkpoint_metadata_restart_runtime_smoke() {
    assert_vaisdb_smoke_runs("checkpoint_metadata_restart_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_26_recovery_full_perform_runtime_smoke() {
    assert_vaisdb_smoke_runs("recovery_full_perform_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_27_checkpoint_truncation_archive_runtime_smoke() {
    assert_vaisdb_smoke_runs("checkpoint_truncation_archive_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_28_wal_segment_filename_read_dir_runtime_smoke() {
    assert_vaisdb_smoke_runs("wal_segment_filename_read_dir_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_29_hnsw_search_recall_larger_runtime_smoke() {
    assert_vaisdb_smoke_runs("hnsw_search_recall_larger_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_30_embedded_durability_runtime_smoke() {
    assert_vaisdb_smoke_runs("embedded_durability_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_31_transaction_mvcc_runtime_smoke() {
    assert_vaisdb_smoke_runs("transaction_mvcc_smoke.vais", 0);
}

#[test]
fn e2e_vaisdb_32_transaction_visibility_runtime_smoke() {
    assert_vaisdb_smoke_runs("transaction_visibility_smoke.vais", 0);
}

fn assert_vaisdb_smoke_runs(fixture: &str, expected_exit: i32) {
    let _guard = VAISDB_SMOKE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let output = run_vaisdb_smoke_fixture(fixture);
    let actual_exit = output.status.code().unwrap_or(-1);
    assert!(
        output.status.success() || actual_exit == expected_exit,
        "VaisDB runtime smoke fixture `{fixture}` should compile, link, and exit {expected_exit}; got status {:?} / exit {actual_exit}.\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        actual_exit,
        expected_exit,
        "VaisDB runtime smoke fixture `{fixture}` exited unexpectedly.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_vaisdb_smoke_fixture(fixture: &str) -> std::process::Output {
    let compiler_root = compiler_root();
    let fixture_path = compiler_root.join("tests/vaisdb/smoke").join(fixture);
    assert!(
        fixture_path.is_file(),
        "missing VaisDB smoke fixture at {}",
        fixture_path.display()
    );

    let smoke_root = fixture_path
        .parent()
        .expect("VaisDB smoke fixture should have a parent directory");
    let _ = std::fs::remove_dir_all(smoke_root.join(".vais-cache"));
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join(".vais-cache"));
    remove_vais_cache_dirs(&compiler_root.join("std"));
    remove_vais_cache_dirs(&workspace_root(&compiler_root).join("lang/packages/vaisdb/src"));

    let isolated_dir = isolated_smoke_dir(fixture);
    let isolated_tmp_dir = isolated_dir.join("tmp");
    let isolated_fixture_path = isolated_dir.join(
        fixture_path
            .file_name()
            .expect("VaisDB smoke fixture should have a file name"),
    );
    std::fs::create_dir_all(&isolated_dir).expect("failed to create isolated smoke source dir");
    std::fs::create_dir_all(&isolated_tmp_dir).expect("failed to create isolated smoke temp dir");
    std::fs::copy(&fixture_path, &isolated_fixture_path)
        .expect("failed to copy VaisDB smoke fixture into isolated source dir");

    let exe_dir = PathBuf::from("/tmp/vais-smoke");
    std::fs::create_dir_all(&exe_dir).expect("failed to create /tmp/vais-smoke");
    let exe_path = exe_dir.join(format!(
        "vaisdb_runtime_{}_{}",
        fixture.replace(['/', '.'], "_"),
        std::process::id()
    ));
    let build = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("build")
        .arg(&isolated_fixture_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .current_dir(&isolated_dir)
        .env("VAIS_STD_PATH", std_link(&compiler_root))
        .env("VAIS_DEP_PATHS", vaisdb_dep_paths(&compiler_root))
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to spawn vaisc for VaisDB runtime smoke build");

    assert!(
        build.status.success(),
        "VaisDB runtime smoke fixture `{fixture}` failed to build.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = Command::new(&exe_path)
        .current_dir(&isolated_dir)
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to run VaisDB runtime smoke executable");
    let _ = std::fs::remove_file(&exe_path);
    let _ = std::fs::remove_dir_all(&isolated_dir);
    run
}

fn isolated_smoke_dir(fixture: &str) -> PathBuf {
    let unique = VAISDB_SMOKE_ID.fetch_add(1, Ordering::SeqCst);
    PathBuf::from("/tmp/vais-smoke-src").join(format!(
        "{}_{}_{}",
        fixture.replace(['/', '.'], "_"),
        std::process::id(),
        unique
    ))
}

fn compiler_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to canonicalize compiler root")
}

fn workspace_root(compiler_root: &Path) -> &Path {
    compiler_root
        .parent()
        .expect("compiler root should have workspace parent")
}

fn std_link(compiler_root: &Path) -> PathBuf {
    let tmp_std_root = PathBuf::from("/tmp/vais-lib");
    std::fs::create_dir_all(&tmp_std_root).expect("failed to create /tmp/vais-lib");
    let std_link = tmp_std_root.join("std");
    let compiler_std = compiler_root.join("std");
    if std_link.exists() {
        let already_correct = std_link
            .canonicalize()
            .map(|path| path == compiler_std)
            .unwrap_or(false);
        if !already_correct {
            let _ = std::fs::remove_file(&std_link);
            let _ = std::fs::remove_dir(&std_link);
        }
    }
    if !std_link.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(&compiler_std, &std_link)
            .expect("failed to create /tmp/vais-lib/std symlink");
    }
    std_link
}

fn vaisdb_dep_paths(compiler_root: &Path) -> String {
    let std_link = std_link(compiler_root);
    let vaisdb_src = workspace_root(compiler_root).join("lang/packages/vaisdb/src");
    format!("{}:{}", vaisdb_src.display(), std_link.display())
}

fn remove_vais_cache_dirs(root: &Path) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name().and_then(|name| name.to_str()) == Some(".vais-cache") {
            let _ = std::fs::remove_dir_all(&path);
            continue;
        }
        if path.is_dir() {
            remove_vais_cache_dirs(&path);
        }
    }
}
