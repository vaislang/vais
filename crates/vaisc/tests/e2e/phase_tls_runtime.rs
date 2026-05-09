use std::path::{Path, PathBuf};
use std::process::Command;

/// std/tls runtime smoke. Validates that:
/// 1. The std/tls module compiles end-to-end (vaisc build → linker → exe).
/// 2. `TlsContext.client()` produces a non-zero handle (real OpenSSL ctx).
/// 3. `TlsContext.free()` returns 0 (no FFI panic).
///
/// Promoted as the entry of the TLS Phase A surface (master-plan v74,
/// loop 69 iter 32). Does NOT exercise actual TLS handshake — that requires
/// a live server and certificate trust chain, which lives in a separate
/// gate (Phase B / external HTTPS smoke).
#[test]
fn e2e_std_tls_context_create_free_runtime_smoke() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("tls_create_free_smoke");

    let source = r#"
use std/tls.{TlsContext}

fn main() -> i64 {
    ctx := TlsContext.client()
    I ctx.handle == 0 { return 1 }
    res := ctx.free()
    I res != 0 { return 2 }
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
        .expect("run tls smoke binary");
    assert!(
        run.status.success(),
        "tls smoke binary exited non-zero: code={:?} stdout={} stderr={}",
        run.status.code(),
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

/// External HTTPS endpoint smoke. Env-gated (RUN_EXTERNAL_HTTPS=1). Skipped
/// in INTEGRITY default to avoid network flakiness. Validates the full
/// std/http_client HTTPS path end-to-end against a stable real server.
///
/// Endpoint: example.com (RFC-2606 reserved, IANA-maintained, stable
/// "Example Domain" body, HTTPS 200, valid Let's Encrypt / DigiCert chain).
#[test]
fn e2e_external_https_example_com_runtime_smoke() {
    if std::env::var("RUN_EXTERNAL_HTTPS").is_err() {
        eprintln!("e2e_external_https_example_com_runtime_smoke: skipped (set RUN_EXTERNAL_HTTPS=1 to run)");
        return;
    }

    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("https_external_smoke");

    let source = r#"
use std/http_client

fn main() -> i64 {
    response := http_get("https://example.com/")
    I response.error_code != 0 { return 1 }
    I response.status != 200 { return 2 }
    body := response.body_text()
    I body.len() <= 0 { return 3 }
    response.drop()
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
        .expect("run external https smoke binary");
    assert!(
        run.status.success(),
        "external https smoke binary exited non-zero: code={:?} stdout={} stderr={}",
        run.status.code(),
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

fn vaisc_path() -> PathBuf {
    let exe = std::env::current_exe().expect("current_exe");
    let mut dir = exe.parent().expect("test binary parent dir").to_path_buf();
    while dir.file_name().is_some_and(|f| f != "release" && f != "debug") {
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
        path = path.parent().expect("walk up to compiler root").to_path_buf();
    }
    path.join("std")
        .to_str()
        .expect("std path utf8")
        .to_string()
}
