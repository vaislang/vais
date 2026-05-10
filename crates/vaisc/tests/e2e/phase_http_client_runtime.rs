use std::io::{BufRead, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn e2e_std_http_client_loopback_post_json_runtime_smoke() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("http_client_loopback_smoke");

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback listener");
    listener
        .set_nonblocking(true)
        .expect("set listener nonblocking");
    let port = listener.local_addr().expect("read listener address").port();

    let source = r#"
use std/http_client

fn smoke_contains(haystack: str, needle: str) -> i64 {
    smoke_contains_rec(haystack, needle, 0)
}

fn smoke_contains_rec(haystack: str, needle: str, i: i64) -> i64 {
    I needle.len() == 0 { return 1 }
    I i >= haystack.len() { return 0 }
    I smoke_match_prefix(haystack, needle, i, 0) == 1 { return 1 }
    smoke_contains_rec(haystack, needle, i + 1)
}

fn smoke_match_prefix(haystack: str, needle: str, hi: i64, ni: i64) -> i64 {
    nc := needle.char_at(ni)
    I nc == 0 { return 1 }
    hc := haystack.char_at(hi)
    I hc == 0 { return 0 }
    I hc != nc { return 0 }
    smoke_match_prefix(haystack, needle, hi + 1, ni + 1)
}

fn main() -> i64 {
    response := http_post("http://127.0.0.1:__PORT__/ssr/render", "{\"route\":\"/dashboard\",\"props\":\"state\"}")
    I response.error_code != 0 { return 1 }
    I response.status != 200 { return 2 }
    body := response.body_text()
    I smoke_contains(body, "\"status\":200") != 1 { return 3 }
    I smoke_contains(body, "app") != 1 { return 4 }
    response.drop()
    0
}
"#
    .replace("__PORT__", &port.to_string());
    std::fs::write(&main_path, source).expect("write http_client fixture");

    let compiler_root = compiler_root();
    let std_path = std_link(&compiler_root);
    let dep_paths = format!("{}:{}", temp.path().display(), std_path.display());

    let build = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("-v")
        .arg("build")
        .arg(&main_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .current_dir(temp.path())
        .env("VAIS_STD_PATH", &std_path)
        .env("VAIS_DEP_PATHS", dep_paths)
        .output()
        .expect("spawn vaisc build");
    assert!(
        build.status.success(),
        "std/http_client loopback fixture failed to build\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let server = thread::spawn(move || accept_one_http_request(listener));
    let run = Command::new(&exe_path)
        .current_dir(temp.path())
        .output()
        .expect("run std/http_client loopback fixture");
    let request_text = server.join().expect("join loopback server");

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "std/http_client loopback fixture exited unexpectedly\nstdout:\n{}\nstderr:\n{}\nrequest:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        request_text
    );
    assert!(
        request_text.contains("POST /ssr/render HTTP/1.1"),
        "request did not contain expected request line:\n{}",
        request_text
    );
    assert!(
        request_text.contains("Host: 127.0.0.1:"),
        "request did not contain Host header with loopback port:\n{}",
        request_text
    );
    assert!(
        request_text.contains("Content-Type: application/json"),
        "request did not contain JSON content type:\n{}",
        request_text
    );
    assert!(
        request_text.contains(r#"{"route":"/dashboard","props":"state"}"#),
        "request did not contain expected JSON body:\n{}",
        request_text
    );
}

#[test]
fn e2e_std_http_client_loopback_absolute_redirect_runtime_smoke() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("http_client_redirect_smoke");

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback listener");
    listener
        .set_nonblocking(true)
        .expect("set listener nonblocking");
    let port = listener.local_addr().expect("read listener address").port();

    let source = r#"
use std/http_client

fn smoke_contains(haystack: str, needle: str) -> i64 {
    smoke_contains_rec(haystack, needle, 0)
}

fn smoke_contains_rec(haystack: str, needle: str, i: i64) -> i64 {
    I needle.len() == 0 { return 1 }
    I i >= haystack.len() { return 0 }
    I smoke_match_prefix(haystack, needle, i, 0) == 1 { return 1 }
    smoke_contains_rec(haystack, needle, i + 1)
}

fn smoke_match_prefix(haystack: str, needle: str, hi: i64, ni: i64) -> i64 {
    nc := needle.char_at(ni)
    I nc == 0 { return 1 }
    hc := haystack.char_at(hi)
    I hc == 0 { return 0 }
    I hc != nc { return 0 }
    smoke_match_prefix(haystack, needle, hi + 1, ni + 1)
}

fn main() -> i64 {
    response := http_get("http://127.0.0.1:__PORT__/redirect")
    I response.error_code != 0 { return 1 }
    I response.status != 200 { return 2 }
    body := response.body_text()
    I smoke_contains(body, "redirect-ok") != 1 { return 3 }
    response.drop()
    0
}
"#
    .replace("__PORT__", &port.to_string());
    std::fs::write(&main_path, source).expect("write http_client redirect fixture");

    let compiler_root = compiler_root();
    let std_path = std_link(&compiler_root);
    let dep_paths = format!("{}:{}", temp.path().display(), std_path.display());

    let build = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("-v")
        .arg("build")
        .arg(&main_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .current_dir(temp.path())
        .env("VAIS_STD_PATH", &std_path)
        .env("VAIS_DEP_PATHS", dep_paths)
        .output()
        .expect("spawn vaisc build");
    assert!(
        build.status.success(),
        "std/http_client redirect fixture failed to build\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let server = thread::spawn(move || accept_absolute_redirect_then_final(listener, port));
    let run = Command::new(&exe_path)
        .current_dir(temp.path())
        .output()
        .expect("run std/http_client redirect fixture");
    let requests = server.join().expect("join loopback redirect server");

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "std/http_client redirect fixture exited unexpectedly\nstdout:\n{}\nstderr:\n{}\nfirst request:\n{}\nsecond request:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        requests.0,
        requests.1
    );
    assert!(
        requests.0.contains("GET /redirect HTTP/1.1"),
        "first request did not contain expected redirect request line:\n{}",
        requests.0
    );
    assert!(
        requests.1.contains("GET /final HTTP/1.1"),
        "second request did not contain expected final request line:\n{}",
        requests.1
    );
    assert!(
        requests.0.contains("Host: 127.0.0.1:") && requests.1.contains("Host: 127.0.0.1:"),
        "redirect requests did not contain Host headers with loopback port:\nfirst:\n{}\nsecond:\n{}",
        requests.0,
        requests.1
    );
}

#[test]
fn e2e_std_http_client_loopback_root_relative_redirect_runtime_smoke() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("http_client_relative_redirect_smoke");

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback listener");
    listener
        .set_nonblocking(true)
        .expect("set listener nonblocking");
    let port = listener.local_addr().expect("read listener address").port();

    let source = r#"
use std/http_client

fn smoke_contains(haystack: str, needle: str) -> i64 {
    smoke_contains_rec(haystack, needle, 0)
}

fn smoke_contains_rec(haystack: str, needle: str, i: i64) -> i64 {
    I needle.len() == 0 { return 1 }
    I i >= haystack.len() { return 0 }
    I smoke_match_prefix(haystack, needle, i, 0) == 1 { return 1 }
    smoke_contains_rec(haystack, needle, i + 1)
}

fn smoke_match_prefix(haystack: str, needle: str, hi: i64, ni: i64) -> i64 {
    nc := needle.char_at(ni)
    I nc == 0 { return 1 }
    hc := haystack.char_at(hi)
    I hc == 0 { return 0 }
    I hc != nc { return 0 }
    smoke_match_prefix(haystack, needle, hi + 1, ni + 1)
}

fn main() -> i64 {
    response := http_get("http://127.0.0.1:__PORT__/redirect")
    I response.error_code != 0 { return 1 }
    I response.status != 200 { return 2 }
    body := response.body_text()
    I smoke_contains(body, "redirect-ok") != 1 { return 3 }
    response.drop()
    0
}
"#
    .replace("__PORT__", &port.to_string());
    std::fs::write(&main_path, source).expect("write http_client relative redirect fixture");

    let compiler_root = compiler_root();
    let std_path = std_link(&compiler_root);
    let dep_paths = format!("{}:{}", temp.path().display(), std_path.display());

    let build = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("-v")
        .arg("build")
        .arg(&main_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .current_dir(temp.path())
        .env("VAIS_STD_PATH", &std_path)
        .env("VAIS_DEP_PATHS", dep_paths)
        .output()
        .expect("spawn vaisc build");
    assert!(
        build.status.success(),
        "std/http_client relative redirect fixture failed to build\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let server = thread::spawn(move || accept_root_relative_redirect_then_final(listener));
    let run = Command::new(&exe_path)
        .current_dir(temp.path())
        .output()
        .expect("run std/http_client relative redirect fixture");
    let requests = server
        .join()
        .expect("join loopback relative redirect server");

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "std/http_client relative redirect fixture exited unexpectedly\nstdout:\n{}\nstderr:\n{}\nfirst request:\n{}\nsecond request:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        requests.0,
        requests.1
    );
    assert!(
        requests.0.contains("GET /redirect HTTP/1.1"),
        "first request did not contain expected redirect request line:\n{}",
        requests.0
    );
    assert!(
        requests.1.contains("GET /final HTTP/1.1"),
        "second request did not contain expected final request line:\n{}",
        requests.1
    );
    assert!(
        requests.0.contains("Host: 127.0.0.1:") && requests.1.contains("Host: 127.0.0.1:"),
        "redirect requests did not contain Host headers with loopback port:\nfirst:\n{}\nsecond:\n{}",
        requests.0,
        requests.1
    );
}

#[test]
fn e2e_std_http_client_loopback_scheme_relative_redirect_runtime_smoke() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp
        .path()
        .join("http_client_scheme_relative_redirect_smoke");

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback listener");
    listener
        .set_nonblocking(true)
        .expect("set listener nonblocking");
    let port = listener.local_addr().expect("read listener address").port();

    let source = r#"
use std/http_client

fn smoke_contains(haystack: str, needle: str) -> i64 {
    smoke_contains_rec(haystack, needle, 0)
}

fn smoke_contains_rec(haystack: str, needle: str, i: i64) -> i64 {
    I needle.len() == 0 { return 1 }
    I i >= haystack.len() { return 0 }
    I smoke_match_prefix(haystack, needle, i, 0) == 1 { return 1 }
    smoke_contains_rec(haystack, needle, i + 1)
}

fn smoke_match_prefix(haystack: str, needle: str, hi: i64, ni: i64) -> i64 {
    nc := needle.char_at(ni)
    I nc == 0 { return 1 }
    hc := haystack.char_at(hi)
    I hc == 0 { return 0 }
    I hc != nc { return 0 }
    smoke_match_prefix(haystack, needle, hi + 1, ni + 1)
}

fn main() -> i64 {
    response := http_get("http://127.0.0.1:__PORT__/redirect")
    I response.error_code != 0 { return 1 }
    I response.status != 200 { return 2 }
    body := response.body_text()
    I smoke_contains(body, "redirect-ok") != 1 { return 3 }
    response.drop()
    0
}
"#
    .replace("__PORT__", &port.to_string());
    std::fs::write(&main_path, source).expect("write http_client scheme-relative redirect fixture");

    let compiler_root = compiler_root();
    let std_path = std_link(&compiler_root);
    let dep_paths = format!("{}:{}", temp.path().display(), std_path.display());

    let build = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("-v")
        .arg("build")
        .arg(&main_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .current_dir(temp.path())
        .env("VAIS_STD_PATH", &std_path)
        .env("VAIS_DEP_PATHS", dep_paths)
        .output()
        .expect("spawn vaisc build");
    assert!(
        build.status.success(),
        "std/http_client scheme-relative redirect fixture failed to build\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let server = thread::spawn(move || accept_scheme_relative_redirect_then_final(listener, port));
    let run = Command::new(&exe_path)
        .current_dir(temp.path())
        .output()
        .expect("run std/http_client scheme-relative redirect fixture");
    let requests = server
        .join()
        .expect("join loopback scheme-relative redirect server");

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "std/http_client scheme-relative redirect fixture exited unexpectedly\nstdout:\n{}\nstderr:\n{}\nfirst request:\n{}\nsecond request:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        requests.0,
        requests.1
    );
    assert!(
        requests.0.contains("GET /redirect HTTP/1.1"),
        "first request did not contain expected redirect request line:\n{}",
        requests.0
    );
    assert!(
        requests.1.contains("GET /final HTTP/1.1"),
        "second request did not contain expected final request line:\n{}",
        requests.1
    );
    assert!(
        requests.0.contains("Host: 127.0.0.1:") && requests.1.contains("Host: 127.0.0.1:"),
        "redirect requests did not contain Host headers with loopback port:\nfirst:\n{}\nsecond:\n{}",
        requests.0,
        requests.1
    );
}

#[test]
fn e2e_std_http_client_loopback_path_relative_redirect_runtime_smoke() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("http_client_path_relative_redirect_smoke");

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback listener");
    listener
        .set_nonblocking(true)
        .expect("set listener nonblocking");
    let port = listener.local_addr().expect("read listener address").port();

    let source = r#"
use std/http_client

fn smoke_contains(haystack: str, needle: str) -> i64 {
    smoke_contains_rec(haystack, needle, 0)
}

fn smoke_contains_rec(haystack: str, needle: str, i: i64) -> i64 {
    I needle.len() == 0 { return 1 }
    I i >= haystack.len() { return 0 }
    I smoke_match_prefix(haystack, needle, i, 0) == 1 { return 1 }
    smoke_contains_rec(haystack, needle, i + 1)
}

fn smoke_match_prefix(haystack: str, needle: str, hi: i64, ni: i64) -> i64 {
    nc := needle.char_at(ni)
    I nc == 0 { return 1 }
    hc := haystack.char_at(hi)
    I hc == 0 { return 0 }
    I hc != nc { return 0 }
    smoke_match_prefix(haystack, needle, hi + 1, ni + 1)
}

fn main() -> i64 {
    response := http_get("http://127.0.0.1:__PORT__/docs/start/index")
    I response.error_code != 0 { return 1 }
    I response.status != 200 { return 2 }
    body := response.body_text()
    I smoke_contains(body, "redirect-ok") != 1 { return 3 }
    response.drop()
    0
}
"#
    .replace("__PORT__", &port.to_string());
    std::fs::write(&main_path, source).expect("write http_client path-relative redirect fixture");

    let compiler_root = compiler_root();
    let std_path = std_link(&compiler_root);
    let dep_paths = format!("{}:{}", temp.path().display(), std_path.display());

    let build = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("-v")
        .arg("build")
        .arg(&main_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .current_dir(temp.path())
        .env("VAIS_STD_PATH", &std_path)
        .env("VAIS_DEP_PATHS", dep_paths)
        .output()
        .expect("spawn vaisc build");
    assert!(
        build.status.success(),
        "std/http_client path-relative redirect fixture failed to build\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let server = thread::spawn(move || accept_path_relative_redirect_then_final(listener));
    let run = Command::new(&exe_path)
        .current_dir(temp.path())
        .output()
        .expect("run std/http_client path-relative redirect fixture");
    let requests = server
        .join()
        .expect("join loopback path-relative redirect server");

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "std/http_client path-relative redirect fixture exited unexpectedly\nstdout:\n{}\nstderr:\n{}\nfirst request:\n{}\nsecond request:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        requests.0,
        requests.1
    );
    assert!(
        requests.0.contains("GET /docs/start/index HTTP/1.1"),
        "first request did not contain expected redirect request line:\n{}",
        requests.0
    );
    assert!(
        requests.1.contains("GET /docs/start/final HTTP/1.1"),
        "second request did not contain normalized final request line:\n{}",
        requests.1
    );
    assert!(
        requests.0.contains("Host: 127.0.0.1:") && requests.1.contains("Host: 127.0.0.1:"),
        "redirect requests did not contain Host headers with loopback port:\nfirst:\n{}\nsecond:\n{}",
        requests.0,
        requests.1
    );
}

#[test]
fn e2e_std_http_client_loopback_query_fragment_relative_redirect_runtime_smoke() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp
        .path()
        .join("http_client_query_fragment_relative_redirect_smoke");

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback listener");
    listener
        .set_nonblocking(true)
        .expect("set listener nonblocking");
    let port = listener.local_addr().expect("read listener address").port();

    let source = r#"
use std/http_client

fn smoke_contains(haystack: str, needle: str) -> i64 {
    smoke_contains_rec(haystack, needle, 0)
}

fn smoke_contains_rec(haystack: str, needle: str, i: i64) -> i64 {
    I needle.len() == 0 { return 1 }
    I i >= haystack.len() { return 0 }
    I smoke_match_prefix(haystack, needle, i, 0) == 1 { return 1 }
    smoke_contains_rec(haystack, needle, i + 1)
}

fn smoke_match_prefix(haystack: str, needle: str, hi: i64, ni: i64) -> i64 {
    nc := needle.char_at(ni)
    I nc == 0 { return 1 }
    hc := haystack.char_at(hi)
    I hc == 0 { return 0 }
    I hc != nc { return 0 }
    smoke_match_prefix(haystack, needle, hi + 1, ni + 1)
}

fn main() -> i64 {
    response := http_get("http://127.0.0.1:__PORT__/docs/start/index?old=1")
    I response.error_code != 0 { return 1 }
    I response.status != 200 { return 2 }
    body := response.body_text()
    I smoke_contains(body, "redirect-ok") != 1 { return 3 }
    response.drop()
    0
}
"#
    .replace("__PORT__", &port.to_string());
    std::fs::write(&main_path, source).expect("write http_client query/fragment redirect fixture");

    let compiler_root = compiler_root();
    let std_path = std_link(&compiler_root);
    let dep_paths = format!("{}:{}", temp.path().display(), std_path.display());

    let build = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("-v")
        .arg("build")
        .arg(&main_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .current_dir(temp.path())
        .env("VAIS_STD_PATH", &std_path)
        .env("VAIS_DEP_PATHS", dep_paths)
        .output()
        .expect("spawn vaisc build");
    assert!(
        build.status.success(),
        "std/http_client query/fragment redirect fixture failed to build\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let server =
        thread::spawn(move || accept_query_fragment_relative_redirects_then_final(listener));
    let run = Command::new(&exe_path)
        .current_dir(temp.path())
        .output()
        .expect("run std/http_client query/fragment redirect fixture");
    let requests = server
        .join()
        .expect("join loopback query/fragment redirect server");

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "std/http_client query/fragment redirect fixture exited unexpectedly\nstdout:\n{}\nstderr:\n{}\nfirst request:\n{}\nsecond request:\n{}\nthird request:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        requests.0,
        requests.1,
        requests.2
    );
    assert!(
        requests.0.contains("GET /docs/start/index?old=1 HTTP/1.1"),
        "first request did not contain expected original query request line:\n{}",
        requests.0
    );
    assert!(
        requests
            .1
            .contains("GET /docs/start/index?view=next HTTP/1.1"),
        "second request did not contain query-only redirect request line:\n{}",
        requests.1
    );
    assert!(
        requests
            .2
            .contains("GET /docs/start/index?view=next HTTP/1.1"),
        "third request should preserve path/query and omit fragment in request line:\n{}",
        requests.2
    );
    assert!(
        !requests.2.contains("#section"),
        "fragment must not be sent in the HTTP request line:\n{}",
        requests.2
    );
    assert!(
        requests.0.contains("Host: 127.0.0.1:")
            && requests.1.contains("Host: 127.0.0.1:")
            && requests.2.contains("Host: 127.0.0.1:"),
        "redirect requests did not contain Host headers with loopback port:\nfirst:\n{}\nsecond:\n{}\nthird:\n{}",
        requests.0,
        requests.1,
        requests.2
    );
}

#[test]
fn e2e_std_http_client_loopback_307_preserves_post_runtime_smoke() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("http_client_307_preserves_post_smoke");

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback listener");
    listener
        .set_nonblocking(true)
        .expect("set listener nonblocking");
    let port = listener.local_addr().expect("read listener address").port();

    let source = r#"
use std/http_client

fn smoke_contains(haystack: str, needle: str) -> i64 {
    smoke_contains_rec(haystack, needle, 0)
}

fn smoke_contains_rec(haystack: str, needle: str, i: i64) -> i64 {
    I needle.len() == 0 { return 1 }
    I i >= haystack.len() { return 0 }
    I smoke_match_prefix(haystack, needle, i, 0) == 1 { return 1 }
    smoke_contains_rec(haystack, needle, i + 1)
}

fn smoke_match_prefix(haystack: str, needle: str, hi: i64, ni: i64) -> i64 {
    nc := needle.char_at(ni)
    I nc == 0 { return 1 }
    hc := haystack.char_at(hi)
    I hc == 0 { return 0 }
    I hc != nc { return 0 }
    smoke_match_prefix(haystack, needle, hi + 1, ni + 1)
}

fn main() -> i64 {
    response := http_post("http://127.0.0.1:__PORT__/submit", "{\"event\":\"keep-method\"}")
    I response.error_code != 0 { return 1 }
    I response.status != 200 { return 2 }
    body := response.body_text()
    I smoke_contains(body, "redirect-ok") != 1 { return 3 }
    response.drop()
    0
}
"#
    .replace("__PORT__", &port.to_string());
    std::fs::write(&main_path, source).expect("write http_client 307 redirect fixture");

    let compiler_root = compiler_root();
    let std_path = std_link(&compiler_root);
    let dep_paths = format!("{}:{}", temp.path().display(), std_path.display());

    let build = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("-v")
        .arg("build")
        .arg(&main_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .current_dir(temp.path())
        .env("VAIS_STD_PATH", &std_path)
        .env("VAIS_DEP_PATHS", dep_paths)
        .output()
        .expect("spawn vaisc build");
    assert!(
        build.status.success(),
        "std/http_client 307 redirect fixture failed to build\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let server = thread::spawn(move || accept_307_preserve_redirect_then_final(listener));
    let run = Command::new(&exe_path)
        .current_dir(temp.path())
        .output()
        .expect("run std/http_client 307 redirect fixture");
    let requests = server.join().expect("join loopback 307 redirect server");

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "std/http_client 307 redirect fixture exited unexpectedly\nstdout:\n{}\nstderr:\n{}\nfirst request:\n{}\nsecond request:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        requests.0,
        requests.1
    );
    assert!(
        requests.0.contains("POST /submit HTTP/1.1"),
        "first request did not contain expected POST request line:\n{}",
        requests.0
    );
    assert!(
        requests.1.contains("POST /submit/final HTTP/1.1"),
        "second request did not preserve POST on 307 redirect:\n{}",
        requests.1
    );
    assert!(
        requests.1.contains("Content-Type: application/json"),
        "second request did not preserve JSON content type:\n{}",
        requests.1
    );
    assert!(
        requests.1.contains(r#"{"event":"keep-method"}"#),
        "second request did not preserve JSON body:\n{}",
        requests.1
    );
    assert!(
        requests.0.contains("Host: 127.0.0.1:") && requests.1.contains("Host: 127.0.0.1:"),
        "redirect requests did not contain Host headers with loopback port:\nfirst:\n{}\nsecond:\n{}",
        requests.0,
        requests.1
    );
}

#[test]
fn e2e_std_http_client_loopback_https_redirect_runtime_smoke() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("http_client_https_redirect_smoke");
    let cert_path = temp.path().join("cert.pem");
    let key_path = temp.path().join("key.pem");
    let script_path = temp.path().join("tls_redirect_server.py");

    generate_self_signed_loopback_cert(&cert_path, &key_path);
    write_tls_redirect_server_script(&script_path);

    let mut server = Command::new("python3")
        .arg(&script_path)
        .arg(&cert_path)
        .arg(&key_path)
        .arg(temp.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn local TLS redirect server");
    let stdout = server
        .stdout
        .take()
        .expect("local TLS redirect server stdout");
    let mut port_line = String::new();
    std::io::BufReader::new(stdout)
        .read_line(&mut port_line)
        .expect("read local TLS redirect server port");
    let port: u16 = port_line
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("invalid local TLS redirect server port line: {port_line:?}"));

    let source = r#"
use std/http_client

fn smoke_contains(haystack: str, needle: str) -> i64 {
    smoke_contains_rec(haystack, needle, 0)
}

fn smoke_contains_rec(haystack: str, needle: str, i: i64) -> i64 {
    I needle.len() == 0 { return 1 }
    I i >= haystack.len() { return 0 }
    I smoke_match_prefix(haystack, needle, i, 0) == 1 { return 1 }
    smoke_contains_rec(haystack, needle, i + 1)
}

fn smoke_match_prefix(haystack: str, needle: str, hi: i64, ni: i64) -> i64 {
    nc := needle.char_at(ni)
    I nc == 0 { return 1 }
    hc := haystack.char_at(hi)
    I hc == 0 { return 0 }
    I hc != nc { return 0 }
    smoke_match_prefix(haystack, needle, hi + 1, ni + 1)
}

fn main() -> i64 {
    client := HttpClient::new()
    client.insecure_tls()
    response := client.get("https://127.0.0.1:__PORT__/secure/redirect")
    I response.error_code != 0 { return 1 }
    I response.status != 200 { return 2 }
    body := response.body_text()
    I smoke_contains(body, "https-redirect-ok") != 1 { return 3 }
    response.drop()
    0
}
"#
    .replace("__PORT__", &port.to_string());
    std::fs::write(&main_path, source).expect("write http_client HTTPS redirect fixture");

    let compiler_root = compiler_root();
    let std_path = std_link(&compiler_root);
    let dep_paths = format!("{}:{}", temp.path().display(), std_path.display());

    let build = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("-v")
        .arg("build")
        .arg(&main_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .current_dir(temp.path())
        .env("VAIS_STD_PATH", &std_path)
        .env("VAIS_DEP_PATHS", dep_paths)
        .output()
        .expect("spawn vaisc build");
    if !build.status.success() {
        let _ = server.kill();
    }
    assert!(
        build.status.success(),
        "std/http_client HTTPS redirect fixture failed to build\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = Command::new(&exe_path)
        .current_dir(temp.path())
        .output()
        .expect("run std/http_client HTTPS redirect fixture");
    let server_output = server
        .wait_with_output()
        .expect("wait for local TLS redirect server");
    let first_request =
        std::fs::read_to_string(temp.path().join("tls_request1.txt")).unwrap_or_default();
    let second_request =
        std::fs::read_to_string(temp.path().join("tls_request2.txt")).unwrap_or_default();

    assert!(
        server_output.status.success(),
        "local TLS redirect server exited non-zero\nstatus={:?}\nstderr:\n{}\nfirst request:\n{}\nsecond request:\n{}",
        server_output.status.code(),
        String::from_utf8_lossy(&server_output.stderr),
        first_request,
        second_request
    );
    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "std/http_client HTTPS redirect fixture exited unexpectedly\nstdout:\n{}\nstderr:\n{}\nfirst request:\n{}\nsecond request:\n{}\nserver stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        first_request,
        second_request,
        String::from_utf8_lossy(&server_output.stderr)
    );
    assert!(
        first_request.contains("GET /secure/redirect HTTP/1.1"),
        "first TLS request did not contain expected request line:\n{}",
        first_request
    );
    assert!(
        second_request.contains("GET /secure/final HTTP/1.1"),
        "second TLS request did not follow HTTPS redirect:\n{}",
        second_request
    );
    assert!(
        first_request.contains("Host: 127.0.0.1:")
            && second_request.contains("Host: 127.0.0.1:"),
        "TLS redirect requests did not contain Host headers with loopback port:\nfirst:\n{}\nsecond:\n{}",
        first_request,
        second_request
    );
}

#[test]
fn e2e_std_http_client_single_arg_return_ir_regression() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("http_client_return_ir_smoke");

    let source = r#"
use std/http_client

fn main() -> i64 {
    response := http_get("http://127.0.0.1/")
    response.drop()
    deleted := http_delete("http://127.0.0.1/")
    deleted.drop()
    0
}
"#;
    std::fs::write(&main_path, source).expect("write http_client IR fixture");

    let compiler_root = compiler_root();
    let std_path = std_link(&compiler_root);
    let dep_paths = format!("{}:{}", temp.path().display(), std_path.display());

    let build = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("build")
        .arg(&main_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .current_dir(temp.path())
        .env("VAIS_STD_PATH", &std_path)
        .env("VAIS_DEP_PATHS", dep_paths)
        .output()
        .expect("spawn vaisc build");
    assert!(
        build.status.success(),
        "std/http_client IR fixture failed to build\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let http_ir = read_http_client_cache_ir(temp.path());
    for signature in [
        "define %HttpResponse @http_get",
        "define %HttpResponse @http_delete",
    ] {
        let fn_ir = extract_function_ir(&http_ir, signature);
        assert!(
            !fn_ir.contains("alloca %HttpResponse*"),
            "{signature} must not allocate a pointer slot for its tail HttpResponse\n{fn_ir}"
        );
        assert!(
            !fn_ir.contains("bitcast %HttpResponse**"),
            "{signature} must not reinterpret a HttpResponse** as HttpResponse*\n{fn_ir}"
        );
        assert!(
            fn_ir.contains("load %HttpResponse, %HttpResponse* %response"),
            "{signature} should return by loading the HttpResponse alloca\n{fn_ir}"
        );
    }
}

fn accept_one_http_request(listener: TcpListener) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    let (mut stream, _) = loop {
        match listener.accept() {
            Ok(pair) => break pair,
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(
                    Instant::now() < deadline,
                    "timed out waiting for std/http_client loopback connection"
                );
                thread::sleep(Duration::from_millis(25));
            }
            Err(err) => panic!("failed to accept std/http_client loopback connection: {err}"),
        }
    };

    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set stream read timeout");

    let mut request = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                request.extend_from_slice(&buffer[..n]);
                let text = String::from_utf8_lossy(&request);
                if text.contains("\r\n\r\n")
                    && text.contains(r#"{"route":"/dashboard","props":"state"}"#)
                {
                    break;
                }
            }
            Err(err)
                if err.kind() == std::io::ErrorKind::WouldBlock
                    || err.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(err) => panic!("failed to read std/http_client request: {err}"),
        }
    }

    let body = r#"{"html":"<div id='app'></div>","status":200}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(response.as_bytes())
        .expect("write std/http_client response");
    stream.flush().expect("flush std/http_client response");

    String::from_utf8_lossy(&request).into_owned()
}

fn accept_absolute_redirect_then_final(listener: TcpListener, port: u16) -> (String, String) {
    accept_redirect_then_final(listener, format!("http://127.0.0.1:{}/final", port))
}

fn accept_root_relative_redirect_then_final(listener: TcpListener) -> (String, String) {
    accept_redirect_then_final(listener, "/final".to_string())
}

fn accept_scheme_relative_redirect_then_final(
    listener: TcpListener,
    port: u16,
) -> (String, String) {
    accept_redirect_then_final(listener, format!("//127.0.0.1:{}/final", port))
}

fn accept_path_relative_redirect_then_final(listener: TcpListener) -> (String, String) {
    accept_redirect_then_final(listener, "./next/../final".to_string())
}

fn accept_query_fragment_relative_redirects_then_final(
    listener: TcpListener,
) -> (String, String, String) {
    let (mut first_stream, first_request) = accept_http_request(&listener);
    let query_redirect = "HTTP/1.1 302 Found\r\nLocation: ?view=next\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    first_stream
        .write_all(query_redirect.as_bytes())
        .expect("write std/http_client query redirect response");
    first_stream
        .flush()
        .expect("flush std/http_client query redirect response");

    let (mut second_stream, second_request) = accept_http_request(&listener);
    let fragment_redirect =
        "HTTP/1.1 302 Found\r\nLocation: #section\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    second_stream
        .write_all(fragment_redirect.as_bytes())
        .expect("write std/http_client fragment redirect response");
    second_stream
        .flush()
        .expect("flush std/http_client fragment redirect response");

    let (mut third_stream, third_request) = accept_http_request(&listener);
    let body = "redirect-ok";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    third_stream
        .write_all(response.as_bytes())
        .expect("write std/http_client final response");
    third_stream
        .flush()
        .expect("flush std/http_client final response");

    (first_request, second_request, third_request)
}

fn accept_307_preserve_redirect_then_final(listener: TcpListener) -> (String, String) {
    let (mut first_stream, first_request) =
        accept_http_request_containing(&listener, r#"{"event":"keep-method"}"#);
    let redirect = "HTTP/1.1 307 Temporary Redirect\r\nLocation: /submit/final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    first_stream
        .write_all(redirect.as_bytes())
        .expect("write std/http_client 307 redirect response");
    first_stream
        .flush()
        .expect("flush std/http_client 307 redirect response");

    let (mut second_stream, second_request) =
        accept_http_request_containing(&listener, r#"{"event":"keep-method"}"#);
    let body = "redirect-ok";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    second_stream
        .write_all(response.as_bytes())
        .expect("write std/http_client final response");
    second_stream
        .flush()
        .expect("flush std/http_client final response");

    (first_request, second_request)
}

fn generate_self_signed_loopback_cert(cert_path: &Path, key_path: &Path) {
    let output = Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-nodes",
            "-sha256",
            "-days",
            "1",
            "-subj",
            "/CN=127.0.0.1",
            "-addext",
            "subjectAltName=IP:127.0.0.1",
            "-keyout",
            key_path.to_str().expect("key path utf8"),
            "-out",
            cert_path.to_str().expect("cert path utf8"),
        ])
        .output()
        .expect("spawn openssl self-signed certificate generation");
    assert!(
        output.status.success(),
        "openssl self-signed certificate generation failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_tls_redirect_server_script(script_path: &Path) {
    let script = r#"
import pathlib
import socket
import ssl
import sys

cert_path = sys.argv[1]
key_path = sys.argv[2]
out_dir = pathlib.Path(sys.argv[3])

context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(certfile=cert_path, keyfile=key_path)

listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
listener.bind(("127.0.0.1", 0))
listener.listen(2)
listener.settimeout(20)
print(listener.getsockname()[1], flush=True)

def read_request(conn):
    conn.settimeout(10)
    chunks = []
    while True:
        data = conn.recv(4096)
        if not data:
            break
        chunks.append(data)
        joined = b"".join(chunks)
        if b"\r\n\r\n" in joined:
            return joined
    return b"".join(chunks)

try:
    raw, _ = listener.accept()
    with context.wrap_socket(raw, server_side=True) as conn:
        request = read_request(conn)
        (out_dir / "tls_request1.txt").write_bytes(request)
        conn.sendall(
            b"HTTP/1.1 302 Found\r\n"
            b"Location: /secure/final\r\n"
            b"Content-Length: 0\r\n"
            b"Connection: close\r\n"
            b"\r\n"
        )

    raw, _ = listener.accept()
    with context.wrap_socket(raw, server_side=True) as conn:
        request = read_request(conn)
        (out_dir / "tls_request2.txt").write_bytes(request)
        body = b"https-redirect-ok"
        conn.sendall(
            b"HTTP/1.1 200 OK\r\n"
            b"Content-Type: text/plain\r\n"
            b"Content-Length: " + str(len(body)).encode("ascii") + b"\r\n"
            b"Connection: close\r\n"
            b"\r\n" + body
        )
finally:
    listener.close()
"#;
    std::fs::write(script_path, script).expect("write local TLS redirect server script");
}

fn accept_redirect_then_final(listener: TcpListener, location: String) -> (String, String) {
    let (mut first_stream, first_request) = accept_http_request(&listener);
    let redirect = format!(
        "HTTP/1.1 302 Found\r\nLocation: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        location
    );
    first_stream
        .write_all(redirect.as_bytes())
        .expect("write std/http_client redirect response");
    first_stream
        .flush()
        .expect("flush std/http_client redirect response");

    let (mut second_stream, second_request) = accept_http_request(&listener);
    let body = "redirect-ok";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    second_stream
        .write_all(response.as_bytes())
        .expect("write std/http_client final response");
    second_stream
        .flush()
        .expect("flush std/http_client final response");

    (first_request, second_request)
}

fn accept_http_request(listener: &TcpListener) -> (std::net::TcpStream, String) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let (mut stream, _) = loop {
        match listener.accept() {
            Ok(pair) => break pair,
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(
                    Instant::now() < deadline,
                    "timed out waiting for std/http_client loopback connection"
                );
                thread::sleep(Duration::from_millis(25));
            }
            Err(err) => panic!("failed to accept std/http_client loopback connection: {err}"),
        }
    };

    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set stream read timeout");

    let mut request = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                request.extend_from_slice(&buffer[..n]);
                if String::from_utf8_lossy(&request).contains("\r\n\r\n") {
                    break;
                }
            }
            Err(err)
                if err.kind() == std::io::ErrorKind::WouldBlock
                    || err.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(err) => panic!("failed to read std/http_client request: {err}"),
        }
    }

    (stream, String::from_utf8_lossy(&request).into_owned())
}

fn accept_http_request_containing(
    listener: &TcpListener,
    needle: &str,
) -> (std::net::TcpStream, String) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let (mut stream, _) = loop {
        match listener.accept() {
            Ok(pair) => break pair,
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(
                    Instant::now() < deadline,
                    "timed out waiting for std/http_client loopback connection"
                );
                thread::sleep(Duration::from_millis(25));
            }
            Err(err) => panic!("failed to accept std/http_client loopback connection: {err}"),
        }
    };

    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set stream read timeout");

    let mut request = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                request.extend_from_slice(&buffer[..n]);
                let text = String::from_utf8_lossy(&request);
                if text.contains("\r\n\r\n") && text.contains(needle) {
                    break;
                }
            }
            Err(err)
                if err.kind() == std::io::ErrorKind::WouldBlock
                    || err.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(err) => panic!("failed to read std/http_client request: {err}"),
        }
    }

    (stream, String::from_utf8_lossy(&request).into_owned())
}

fn read_http_client_cache_ir(temp_root: &Path) -> String {
    let cache_dir = temp_root.join(".vais-cache");
    let entries = std::fs::read_dir(&cache_dir)
        .unwrap_or_else(|err| panic!("failed to read cache dir {}: {err}", cache_dir.display()));
    for entry in entries {
        let path = entry.expect("cache entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("ll") {
            continue;
        }
        let ir = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        if ir.contains("define %HttpResponse @http_get") {
            return ir;
        }
    }
    panic!(
        "failed to find std/http_client IR in {}",
        cache_dir.display()
    );
}

fn extract_function_ir<'a>(ir: &'a str, signature: &str) -> &'a str {
    let start = ir
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature {signature}"));
    let rest = &ir[start..];
    let end = rest
        .find("\n}")
        .unwrap_or_else(|| panic!("missing function terminator for {signature}"))
        + 2;
    &rest[..end]
}

fn compiler_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to canonicalize compiler root")
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
