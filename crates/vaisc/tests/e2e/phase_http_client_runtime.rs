use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
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
