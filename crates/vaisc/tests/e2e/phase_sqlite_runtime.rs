use std::path::{Path, PathBuf};
use std::process::Command;

/// std/sqlite TEXT-column ownership runtime smoke. Validates that:
/// 1. The std/sqlite module compiles end-to-end (vaisc build → linker → exe,
///    auto-linked against sqlite_runtime.c + -lsqlite3).
/// 2. `Database.memory()` opens an in-memory database.
/// 3. A TEXT column read via `column_text_owned()` survives the next
///    `step()` (the C runtime returns a malloc-owned copy, not SQLite-managed
///    memory invalidated by stepping).
/// 4. An empty TEXT value reads back as an empty string.
/// 5. Owned values can be released through the explicit free boundary
///    (`SqliteText.free()` / manual `SqliteText.drop()` alias), and a second
///    release is an idempotent no-op.
/// 6. The legacy `column_text()` + `free_column_text()` path still works.
///
/// This certifies the explicit TEXT read/free ownership boundary added to
/// `__sqlite_column_text` / `__sqlite_free_text`. It does NOT make broad
/// SQLite semantics claims — it is a single bounded ownership smoke.
#[test]
fn e2e_std_sqlite_text_owned_free_runtime_smoke() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("sqlite_text_owned_smoke");

    // Exit codes are intentionally kept under 200.
    let source = r#"
use std/sqlite.{Database, SqliteText, free_column_text}

fn main() -> i64 {
    db := Database.memory()
    I db.is_valid() == 0 { return 1 }

    rc := db.exec("CREATE TABLE t (id INTEGER, name TEXT)")
    I rc != 0 { return 2 }

    rc2 := db.exec("INSERT INTO t (id, name) VALUES (1, 'alpha'), (2, 'beta'), (3, '')")
    I rc2 != 0 { return 3 }

    stmt := db.prepare("SELECT name FROM t ORDER BY id")
    I stmt.is_valid() == 0 { return 4 }

    # Row 1: 'alpha' — take an owned copy.
    s1 := stmt.step()
    I s1 != 100 { return 5 }
    first := stmt.column_text_owned(0)

    # Step to row 2 — the owned copy from row 1 must remain readable.
    s2 := stmt.step()
    I s2 != 100 { return 6 }
    second := stmt.column_text_owned(0)

    # first still points at its own malloc copy, unaffected by the step().
    I str_eq(first.as_str(), "alpha") == 0 { return 7 }
    I str_eq(second.as_str(), "beta") == 0 { return 8 }

    # Row 3: empty TEXT value reads back as an empty string.
    s3 := stmt.step()
    I s3 != 100 { return 9 }
    third := stmt.column_text_owned(0)
    I str_eq(third.as_str(), "") == 0 { return 10 }

    # Statement is exhausted.
    s4 := stmt.step()
    I s4 != 101 { return 11 }

    # Explicit release of the owned values. drop() is a manual alias for free().
    I first.drop() != 0 { return 12 }
    I second.free() != 0 { return 13 }
    I third.drop() != 0 { return 14 }

    # Double release through SqliteText.drop() is idempotent: returns 0, no crash.
    I first.drop() != 0 { return 15 }
    I first.is_owned() != 0 { return 16 }

    stmt.finalize()

    # Legacy column_text() + free_column_text() once.
    legacy := db.prepare("SELECT name FROM t ORDER BY id LIMIT 1")
    I legacy.is_valid() == 0 { return 17 }
    ls := legacy.step()
    I ls != 100 { return 18 }
    legacy_text := legacy.column_text(0)
    I str_eq(legacy_text, "alpha") == 0 { return 19 }
    I free_column_text(legacy_text) != 0 { return 20 }
    legacy.finalize()

    db.close()
    0
}

# Minimal byte-wise string comparison (avoids relying on stdlib str helpers
# whose C runtime is not linked for a std/sqlite-only build). `load_byte` is a
# compiler builtin used unqualified across std/.
fn str_eq(a: str, b: str) -> i64 {
    i := mut 0
    L {
        ca := load_byte((a as i64) + i)
        cb := load_byte((b as i64) + i)
        I ca != cb { return 0 }
        I ca == 0 { return 1 }
        i = i + 1
    }
    1
}
"#;

    std::fs::write(&main_path, source).expect("write main.vais");

    let vaisc = vaisc_path();
    let std_dir = std_dir();

    let build = Command::new(&vaisc)
        .args([
            "build",
            main_path.to_str().expect("main.vais path utf8"),
            "-o",
            exe_path.to_str().expect("exe path utf8"),
        ])
        .env("VAIS_STD_PATH", &std_dir)
        .output()
        .expect("vaisc build invocation");

    assert!(
        build.status.success(),
        "vaisc build failed: stdout={}\nstderr={}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = Command::new(&exe_path)
        .output()
        .expect("run sqlite text smoke binary");
    assert!(
        run.status.success(),
        "sqlite text smoke binary exited non-zero: code={:?} stdout={} stderr={}",
        run.status.code(),
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

/// std/sqlite prepared statement smoke. Validates the next bounded sqlite
/// runtime surface:
/// 1. Prepared INSERT parameter binding for int/text/null values.
/// 2. Statement reset + clear_bindings reuse.
/// 3. Column count/name/type/int reads on a SELECT result.
/// 4. Borrowed sqlite-owned column_name()/errmsg() pointers are readable inside
///    their owning statement/database lifetimes and are not freed by the caller.
/// 5. Owned column_name()/errmsg() copies survive finalize()/close() and are
///    released through the explicit wrapper boundary.
///
/// This deliberately remains a small runtime boundary. It does not claim broad
/// SQL compatibility, planner behavior, or borrowed pointer lifetime after
/// finalize/close.
#[test]
fn e2e_std_sqlite_prepare_bind_reset_metadata_runtime_smoke() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("sqlite_prepare_bind_metadata_smoke");

    let source = r#"
use std/sqlite.{Database}

fn main() -> i64 {
    db := Database.memory()
    I db.is_valid() == 0 { return 1 }

    I db.exec("CREATE TABLE t (id INTEGER, name TEXT)") != 0 { return 2 }

    ins := db.prepare("INSERT INTO t (id, name) VALUES (?, ?)")
    I ins.is_valid() == 0 { return 3 }
    I ins.bind_int(1, 1) != 0 { return 4 }
    I ins.bind_text(2, "alpha") != 0 { return 5 }
    I ins.execute() != 0 { return 6 }
    I db.changes() != 1 { return 7 }

    I ins.reset() != 0 { return 8 }
    I ins.clear_bindings() != 0 { return 9 }
    I ins.bind_int(1, 2) != 0 { return 10 }
    I ins.bind_text(2, "beta") != 0 { return 11 }
    I ins.execute() != 0 { return 12 }

    I ins.reset() != 0 { return 13 }
    I ins.clear_bindings() != 0 { return 14 }
    I ins.bind_int(1, 3) != 0 { return 15 }
    I ins.bind_null(2) != 0 { return 16 }
    I ins.execute() != 0 { return 17 }
    ins.finalize()

    stmt := db.prepare("SELECT id AS item_id, name AS item_name FROM t ORDER BY id")
    I stmt.is_valid() == 0 { return 18 }
    I stmt.columns() != 2 { return 19 }
    I str_eq(stmt.column_name(0), "item_id") == 0 { return 20 }
    I str_eq(stmt.column_name(1), "item_name") == 0 { return 21 }
    name0 := stmt.column_name_owned(0)
    name1 := stmt.column_name_owned(1)
    I str_eq(name0.as_str(), "item_id") == 0 { return 22 }
    I str_eq(name1.as_str(), "item_name") == 0 { return 23 }

    I stmt.step() != 100 { return 24 }
    I stmt.column_type(0) != 1 { return 25 }
    I stmt.column_type(1) != 3 { return 26 }
    I stmt.column_int(0) != 1 { return 27 }
    first := stmt.column_text_owned(1)
    I str_eq(first.as_str(), "alpha") == 0 { return 28 }
    first.free()

    I stmt.step() != 100 { return 29 }
    I stmt.column_int(0) != 2 { return 30 }
    second := stmt.column_text_owned(1)
    I str_eq(second.as_str(), "beta") == 0 { return 31 }
    second.free()

    I stmt.step() != 100 { return 32 }
    I stmt.column_int(0) != 3 { return 33 }
    I stmt.column_type(1) != 5 { return 34 }
    I stmt.step() != 101 { return 35 }
    stmt.finalize()

    # Owned metadata copies remain readable after the statement is finalized.
    I str_eq(name0.as_str(), "item_id") == 0 { return 36 }
    I str_eq(name1.as_str(), "item_name") == 0 { return 37 }
    I name0.drop() != 0 { return 38 }
    I name1.free() != 0 { return 39 }

    bad := db.prepare("SELECT missing FROM missing_table")
    I bad.is_valid() != 0 {
        bad.finalize()
        return 40
    }
    I str_nonempty(db.error_message()) == 0 { return 41 }
    err := db.error_message_owned()
    I str_nonempty(err.as_str()) == 0 { return 42 }

    db.close()
    # Owned errmsg copy remains readable after the database is closed.
    I str_nonempty(err.as_str()) == 0 { return 43 }
    I err.drop() != 0 { return 44 }
    0
}

fn str_nonempty(s: str) -> i64 {
    I load_byte(s as i64) == 0 { 0 } else { 1 }
}

fn str_eq(a: str, b: str) -> i64 {
    i := mut 0
    L {
        ca := load_byte((a as i64) + i)
        cb := load_byte((b as i64) + i)
        I ca != cb { return 0 }
        I ca == 0 { return 1 }
        i = i + 1
    }
    1
}
"#;

    std::fs::write(&main_path, source).expect("write main.vais");

    let vaisc = vaisc_path();
    let std_dir = std_dir();

    let build = Command::new(&vaisc)
        .args([
            "build",
            main_path.to_str().expect("main.vais path utf8"),
            "-o",
            exe_path.to_str().expect("exe path utf8"),
        ])
        .env("VAIS_STD_PATH", &std_dir)
        .output()
        .expect("vaisc build invocation");

    assert!(
        build.status.success(),
        "vaisc build failed: stdout={}\nstderr={}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = Command::new(&exe_path)
        .output()
        .expect("run sqlite prepare/bind metadata smoke binary");
    assert!(
        run.status.success(),
        "sqlite prepare/bind metadata smoke binary exited non-zero: code={:?} stdout={} stderr={}",
        run.status.code(),
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

/// std/sqlite transaction and DML utility smoke. Validates a bounded helper
/// surface on a real in-memory SQLite connection:
/// 1. create_table/drop_table plus table_exists/table_count.
/// 2. begin/rollback and begin_immediate/commit transaction boundaries.
/// 3. changes() and last_insert_id() after explicit INSERT/UPDATE/DELETE.
/// 4. exec_many() as the public multi-statement alias for sqlite3_exec.
///
/// This does not claim broad SQL compatibility, WAL/concurrency semantics, or
/// arbitrary untrusted table-name handling in table_count/drop_table.
#[test]
fn e2e_std_sqlite_transaction_dml_util_runtime_smoke() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("sqlite_transaction_dml_util_smoke");

    let source = r#"
use std/sqlite.{Database, table_exists, table_count, exec_many}

fn main() -> i64 {
    db := Database.memory()
    I db.is_valid() == 0 { return 1 }

    I db.create_table("CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT)") != 0 { return 2 }
    I table_exists(&db, "items") != 1 { return 3 }
    I table_exists(&db, "missing") != 0 { return 4 }
    I table_count(&db, "items") != 0 { return 5 }

    I db.begin() != 0 { return 6 }
    I db.exec("INSERT INTO items (id, name) VALUES (1, 'rolled-back')") != 0 { return 7 }
    I db.changes() != 1 { return 8 }
    I db.last_insert_id() != 1 { return 9 }
    I table_count(&db, "items") != 1 { return 10 }
    I db.rollback() != 0 { return 11 }
    I table_count(&db, "items") != 0 { return 12 }

    I db.begin_immediate() != 0 { return 13 }
    I db.exec("INSERT INTO items (id, name) VALUES (2, 'committed')") != 0 { return 14 }
    I db.changes() != 1 { return 15 }
    I db.last_insert_id() != 2 { return 16 }
    I db.commit() != 0 { return 17 }
    I table_count(&db, "items") != 1 { return 18 }

    I exec_many(&db, "UPDATE items SET name='updated' WHERE id=2; DELETE FROM items WHERE id=2;") != 0 { return 19 }
    I db.changes() != 1 { return 20 }
    I table_count(&db, "items") != 0 { return 21 }

    I db.drop_table("items") != 0 { return 22 }
    I table_exists(&db, "items") != 0 { return 23 }

    db.close()
    0
}
"#;

    std::fs::write(&main_path, source).expect("write main.vais");

    let vaisc = vaisc_path();
    let std_dir = std_dir();

    let build = Command::new(&vaisc)
        .args([
            "build",
            main_path.to_str().expect("main.vais path utf8"),
            "-o",
            exe_path.to_str().expect("exe path utf8"),
        ])
        .env("VAIS_STD_PATH", &std_dir)
        .output()
        .expect("vaisc build invocation");

    assert!(
        build.status.success(),
        "vaisc build failed: stdout={}\nstderr={}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = Command::new(&exe_path)
        .output()
        .expect("run sqlite transaction/DML utility smoke binary");
    assert!(
        run.status.success(),
        "sqlite transaction/DML utility smoke binary exited non-zero: code={:?} stdout={} stderr={}",
        run.status.code(),
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

fn vaisc_path() -> PathBuf {
    let exe = std::env::current_exe().expect("current_exe");
    let mut dir = exe.parent().expect("test binary parent dir").to_path_buf();
    while dir
        .file_name()
        .is_some_and(|f| f != "release" && f != "debug")
    {
        dir = dir.parent().expect("walk to release/debug").to_path_buf();
    }
    dir.join("vaisc")
}

fn std_dir() -> String {
    if let Ok(p) = std::env::var("VAIS_STD_PATH") {
        return p;
    }
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    while !path.join("std").is_dir() {
        path = path
            .parent()
            .expect("walk up to compiler root")
            .to_path_buf();
    }
    path.join("std")
        .to_str()
        .expect("std path utf8")
        .to_string()
}
