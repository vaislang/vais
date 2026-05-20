use std::path::{Path, PathBuf};
use std::process::Command;

/// std/postgres TEXT ownership runtime smoke. This is deliberately serverless:
/// it validates the std wrapper build/link path and uses libpq's synthetic
/// PGresult constructors from a tiny C probe to prove the C runtime copy/free
/// boundary without requiring a running PostgreSQL server.
///
/// Certified surface:
/// 1. `std/postgres` builds and auto-links `postgres_runtime.c` + libpq.
/// 2. Invalid-handle `PgResult.get_text_owned()` returns a non-owned empty
///    wrapper whose manual `drop()` alias is an idempotent no-op.
/// 3. `__pg_getvalue_copy()` and `__pg_fname_copy()` return malloc-owned
///    copies independent of the source PGresult, and `__pg_free_text()`
///    releases them.
/// 4. `__pg_error_message_copy()` returns a malloc-owned copy independent of
///    the source PGconn, stable after `PQfinish()`.
///
/// Non-claims: live server connection/query semantics, arbitrary libpq
/// deployment discovery, and borrowed `PgResult.get_text()` use after
/// `PgResult.clear()`.
#[test]
fn e2e_std_postgres_text_owned_boundary_runtime_smoke() {
    let Some(pg) = pg_config() else {
        eprintln!(
            "e2e_std_postgres_text_owned_boundary_runtime_smoke: skipped (pg_config not found)"
        );
        return;
    };

    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("postgres_text_owned_smoke");

    let source = r#"
use std/postgres.{PgConnection, PgResult, PgText}

fn main() -> i64 {
    res := PgResult::from_handle(0)
    I res.is_valid() != 0 { return 1 }

    text := res.get_text_owned(0, 0)
    I str_eq(text.as_str(), "") == 0 { return 2 }
    I text.is_owned() != 0 { return 3 }
    I text.drop() != 0 { return 4 }
    I text.drop() != 0 { return 5 }

    name := res.field_name_owned(0)
    I str_eq(name.as_str(), "") == 0 { return 6 }
    I name.is_owned() != 0 { return 7 }
    I name.drop() != 0 { return 8 }
    I name.drop() != 0 { return 9 }

    conn := PgConnection { handle: 0, host: "", port: 0, dbname: "", user: "", is_connected: 0 }
    msg := conn.error_message_owned()
    I msg.is_owned() != 1 { return 11 }
    I str_nonempty(msg.as_str()) == 0 { return 12 }
    I msg.drop() != 0 { return 13 }
    I msg.drop() != 0 { return 14 }

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
        .expect("run postgres std wrapper smoke binary");
    assert!(
        run.status.success(),
        "postgres std wrapper smoke binary exited non-zero: code={:?} stdout={} stderr={}",
        run.status.code(),
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    run_c_pgresult_copy_probe(temp.path(), &pg, &std_dir);
}

fn run_c_pgresult_copy_probe(temp: &Path, pg: &PgConfig, std_dir: &str) {
    let probe_path = temp.join("pg_text_copy_probe.c");
    let exe_path = temp.join("pg_text_copy_probe");
    let runtime = Path::new(std_dir).join("postgres_runtime.c");

    let source = r#"
#include <libpq-fe.h>
#include <stddef.h>
#include <string.h>

extern const char* __pg_getvalue(long result, long row, long col);
extern const char* __pg_getvalue_copy(long result, long row, long col);
extern const char* __pg_fname(long result, long col);
extern const char* __pg_fname_copy(long result, long col);
extern const char* __pg_error_message(long handle);
extern const char* __pg_error_message_copy(long handle);
extern long __pg_free_text(const char* ptr);

int main(void) {
    PGresult* res = PQmakeEmptyPGresult(NULL, PGRES_TUPLES_OK);
    if (res == NULL) return 1;

    PGresAttDesc attrs[1];
    memset(attrs, 0, sizeof(attrs));
    attrs[0].name = (char*)"name";
    attrs[0].tableid = 0;
    attrs[0].columnid = 0;
    attrs[0].format = 0;
    attrs[0].typid = 25;
    attrs[0].typlen = -1;
    attrs[0].atttypmod = -1;

    if (PQsetResultAttrs(res, 1, attrs) != 1) {
        PQclear(res);
        return 2;
    }
    if (PQsetvalue(res, 0, 0, (char*)"alpha", 5) != 1) {
        PQclear(res);
        return 3;
    }

    const char* borrowed = __pg_getvalue((long)res, 0, 0);
    const char* copy = __pg_getvalue_copy((long)res, 0, 0);
    if (copy == NULL) {
        PQclear(res);
        return 4;
    }
    if (strcmp(copy, "alpha") != 0) {
        __pg_free_text(copy);
        PQclear(res);
        return 5;
    }
    if (copy == borrowed) {
        __pg_free_text(copy);
        PQclear(res);
        return 6;
    }

    const char* borrowed_name = __pg_fname((long)res, 0);
    const char* name_copy = __pg_fname_copy((long)res, 0);
    if (name_copy == NULL) {
        __pg_free_text(copy);
        PQclear(res);
        return 7;
    }
    if (strcmp(name_copy, "name") != 0) {
        __pg_free_text(copy);
        __pg_free_text(name_copy);
        PQclear(res);
        return 8;
    }
    if (name_copy == borrowed_name) {
        __pg_free_text(copy);
        __pg_free_text(name_copy);
        PQclear(res);
        return 9;
    }

    PQclear(res);
    if (strcmp(copy, "alpha") != 0) {
        __pg_free_text(copy);
        __pg_free_text(name_copy);
        return 10;
    }
    if (strcmp(name_copy, "name") != 0) {
        __pg_free_text(copy);
        __pg_free_text(name_copy);
        return 11;
    }
    if (__pg_free_text(copy) != 0) return 12;
    if (__pg_free_text(name_copy) != 0) return 13;

    const char* empty = __pg_getvalue_copy(0, 0, 0);
    if (empty == NULL) return 14;
    if (strcmp(empty, "") != 0) {
        __pg_free_text(empty);
        return 15;
    }
    if (__pg_free_text(empty) != 0) return 16;

    const char* empty_name = __pg_fname_copy(0, 0);
    if (empty_name == NULL) return 17;
    if (strcmp(empty_name, "") != 0) {
        __pg_free_text(empty_name);
        return 18;
    }
    if (__pg_free_text(empty_name) != 0) return 19;

    PGconn* conn = PQconnectStart("");
    if (conn == NULL) return 20;
    const char* borrowed_err = __pg_error_message((long)conn);
    const char* err_copy = __pg_error_message_copy((long)conn);
    if (err_copy == NULL) {
        PQfinish(conn);
        return 21;
    }
    if (strcmp(err_copy, borrowed_err) != 0) {
        __pg_free_text(err_copy);
        PQfinish(conn);
        return 22;
    }
    if (err_copy == borrowed_err) {
        __pg_free_text(err_copy);
        PQfinish(conn);
        return 23;
    }
    PQfinish(conn);
    (void)strlen(err_copy);
    if (__pg_free_text(err_copy) != 0) return 25;

    const char* no_conn = __pg_error_message_copy(0);
    if (no_conn == NULL) return 26;
    if (strcmp(no_conn, "No connection") != 0) {
        __pg_free_text(no_conn);
        return 27;
    }
    if (__pg_free_text(no_conn) != 0) return 28;
    if (__pg_free_text(NULL) != 0) return 29;

    return 0;
}
"#;

    std::fs::write(&probe_path, source).expect("write C postgres probe");

    let build = Command::new("clang")
        .arg(&probe_path)
        .arg(&runtime)
        .arg(format!("-I{}", pg.include_dir))
        .arg(format!("-L{}", pg.lib_dir))
        .arg("-lpq")
        .arg("-o")
        .arg(&exe_path)
        .output()
        .expect("clang postgres C probe invocation");

    assert!(
        build.status.success(),
        "clang postgres C probe failed: stdout={}\nstderr={}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = Command::new(&exe_path)
        .output()
        .expect("run postgres C copy/free probe");
    assert!(
        run.status.success(),
        "postgres C copy/free probe exited non-zero: code={:?} stdout={} stderr={}",
        run.status.code(),
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

struct PgConfig {
    include_dir: String,
    lib_dir: String,
}

fn pg_config() -> Option<PgConfig> {
    Some(PgConfig {
        include_dir: pg_config_value("--includedir")?,
        lib_dir: pg_config_value("--libdir")?,
    })
}

fn pg_config_value(flag: &str) -> Option<String> {
    let output = Command::new("pg_config").arg(flag).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
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
