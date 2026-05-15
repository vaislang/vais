use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

static VAIS_SERVER_SMOKE_LOCK: Mutex<()> = Mutex::new(());
static VAIS_SERVER_SMOKE_ID: AtomicUsize = AtomicUsize::new(0);

macro_rules! skip_if_no_vais_server_package {
    () => {{
        let compiler_root = compiler_root();
        if vais_server_package_roots(&compiler_root).is_none() {
            eprintln!(
                "SKIP: vais-server source root not found; set VAIS_LANG_ROOT to run this smoke"
            );
            return;
        }
    }};
}

#[test]
fn e2e_vais_server_00_minimal_runtime_smoke() {
    assert_vais_server_smoke_runs("minimal_runtime_smoke.vais", 0);
}

#[test]
fn e2e_vais_server_01_vaisdb_embedded_integration_smoke() {
    assert_vais_server_smoke_runs("vaisdb_embedded_integration_smoke.vais", 0);
}

#[test]
fn e2e_vais_server_02_request_router_runtime_smoke() {
    assert_vais_server_smoke_runs("request_router_runtime_smoke.vais", 0);
}

#[test]
fn e2e_vais_server_03_path_query_runtime_smoke() {
    assert_vais_server_smoke_runs("path_query_runtime_smoke.vais", 0);
}

#[test]
fn e2e_vais_server_04_wildcard_runtime_smoke() {
    assert_vais_server_smoke_runs("wildcard_runtime_smoke.vais", 0);
}

#[test]
fn e2e_vais_server_05_body_parser_runtime_smoke() {
    assert_vais_server_smoke_runs("body_parser_runtime_smoke.vais", 0);
}

#[test]
fn e2e_vais_server_06_middleware_pipeline_runtime_smoke() {
    assert_vais_server_smoke_runs("middleware_pipeline_runtime_smoke.vais", 0);
}

#[test]
fn e2e_vais_server_07_ssr_api_runtime_smoke() {
    assert_vais_server_smoke_runs("ssr_api_runtime_smoke.vais", 0);
}

#[test]
fn e2e_vais_server_08_auth_password_runtime_smoke() {
    assert_vais_server_smoke_runs("auth_password_runtime_smoke.vais", 0);
}

#[test]
fn e2e_vais_server_09_auth_session_runtime_smoke() {
    assert_vais_server_smoke_runs("auth_session_runtime_smoke.vais", 0);
}

#[test]
fn e2e_vais_server_10_auth_oauth_runtime_smoke() {
    assert_vais_server_smoke_runs("auth_oauth_runtime_smoke.vais", 0);
}

#[test]
fn e2e_vais_server_11_auth_jwt_runtime_smoke() {
    assert_vais_server_smoke_runs("auth_jwt_runtime_smoke.vais", 0);
}

#[test]
fn e2e_vais_server_12_ssr_forwarding_runtime_smoke() {
    skip_if_no_vais_server_package!();
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind SSR upstream listener");
    listener
        .set_nonblocking(true)
        .expect("set SSR upstream listener nonblocking");
    let port = listener
        .local_addr()
        .expect("read SSR upstream address")
        .port();

    let source = r#"
use src/api/ssr
use src/http/header

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
    response := forward_ssr_render("http://127.0.0.1:__PORT__", "/products/sku-42", "state")
    I response.status.code != 202 {
        println("FAIL ssr forwarding status")
        return 1
    }
    I response.status.reason != "Accepted" {
        println("FAIL ssr forwarding reason")
        return 2
    }
    I response.headers.get(CONTENT_TYPE) != "application/json" {
        println("FAIL ssr forwarding content type")
        return 3
    }
    I smoke_contains(response.body, "from-node") != 1 {
        println("FAIL ssr forwarding body")
        return 4
    }
    I smoke_contains(response.body, "\"status\":202") != 1 {
        println("FAIL ssr forwarding body status")
        return 5
    }
    println("VAIS_SERVER_SSR_FORWARDING_RUNTIME_OK")
    0
}
"#
    .replace("__PORT__", &port.to_string());

    let (run, request_text) = run_vais_server_generated_loopback_smoke(
        "ssr_forwarding_runtime_smoke.vais",
        &source,
        listener,
    );

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "vais-server SSR forwarding smoke exited unexpectedly.\nstdout:\n{}\nstderr:\n{}\nrequest:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        request_text
    );
    assert!(
        request_text.contains("POST /ssr/render HTTP/1.1"),
        "SSR forwarding request did not contain expected request line:\n{}",
        request_text
    );
    assert!(
        request_text.contains("Host: 127.0.0.1:"),
        "SSR forwarding request did not contain Host header with loopback port:\n{}",
        request_text
    );
    assert!(
        request_text.contains("Content-Type: application/json"),
        "SSR forwarding request did not contain JSON content type:\n{}",
        request_text
    );
    assert!(
        request_text.contains(r#"{"route":"/products/sku-42","props":"state"}"#),
        "SSR forwarding request did not contain expected JSON body:\n{}",
        request_text
    );
}

#[test]
fn e2e_vais_server_13_ssr_forwarding_error_mapping_runtime_smoke() {
    skip_if_no_vais_server_package!();
    let status_listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("bind SSR status upstream listener");
    status_listener
        .set_nonblocking(true)
        .expect("set SSR status upstream listener nonblocking");
    let status_port = status_listener
        .local_addr()
        .expect("read SSR status upstream address")
        .port();

    let drop_listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("bind SSR drop upstream listener");
    drop_listener
        .set_nonblocking(true)
        .expect("set SSR drop upstream listener nonblocking");
    let drop_port = drop_listener
        .local_addr()
        .expect("read SSR drop upstream address")
        .port();

    let source = r#"
use src/api/ssr
use src/http/header

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
    upstream_status := forward_ssr_render("http://127.0.0.1:__STATUS_PORT__", "/missing", "state")
    I upstream_status.status.code != 503 {
        println("FAIL ssr forwarding upstream status code")
        return 1
    }
    I upstream_status.status.reason != "Service Unavailable" {
        println("FAIL ssr forwarding upstream reason")
        return 2
    }
    I upstream_status.headers.get(CONTENT_TYPE) != "application/json" {
        println("FAIL ssr forwarding upstream content type")
        return 3
    }
    I smoke_contains(upstream_status.body, "remote-down") != 1 {
        println("FAIL ssr forwarding upstream body")
        return 4
    }

    transport_failure := forward_ssr_render("http://127.0.0.1:__DROP_PORT__", "/dashboard", "state")
    I transport_failure.status.code != 502 {
        println("FAIL ssr forwarding transport status code")
        return 10
    }
    I transport_failure.status.reason != "Bad Gateway" {
        println("FAIL ssr forwarding transport reason")
        return 11
    }
    I transport_failure.headers.get(CONTENT_TYPE) != "application/json" {
        println("FAIL ssr forwarding transport content type")
        return 12
    }
    I smoke_contains(transport_failure.body, "SSR upstream unavailable") != 1 {
        println("FAIL ssr forwarding transport body")
        return 13
    }

    println("VAIS_SERVER_SSR_FORWARDING_ERROR_MAPPING_RUNTIME_OK")
    0
}
"#
    .replace("__STATUS_PORT__", &status_port.to_string())
    .replace("__DROP_PORT__", &drop_port.to_string());

    let (run, status_request, drop_request) = run_vais_server_generated_dual_loopback_smoke(
        "ssr_forwarding_error_mapping_runtime_smoke.vais",
        &source,
        status_listener,
        drop_listener,
    );

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "vais-server SSR forwarding error-mapping smoke exited unexpectedly.\nstdout:\n{}\nstderr:\n{}\nstatus request:\n{}\ndrop request:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        status_request,
        drop_request
    );
    assert!(
        status_request.contains(r#"{"route":"/missing","props":"state"}"#),
        "SSR forwarding status request did not contain expected JSON body:\n{}",
        status_request
    );
    assert!(
        drop_request.contains(r#"{"route":"/dashboard","props":"state"}"#),
        "SSR forwarding dropped request did not contain expected JSON body:\n{}",
        drop_request
    );
}

#[test]
fn e2e_vais_server_14_ssr_forwarding_timeout_runtime_smoke() {
    skip_if_no_vais_server_package!();
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind SSR timeout listener");
    listener
        .set_nonblocking(true)
        .expect("set SSR timeout listener nonblocking");
    let port = listener
        .local_addr()
        .expect("read SSR timeout address")
        .port();

    let source = r#"
use src/api/ssr
use src/http/header

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
    response := forward_ssr_render_with_timeout("http://127.0.0.1:__PORT__", "/slow", "state", 300)
    I response.status.code != 504 {
        println("FAIL ssr forwarding timeout status code")
        return 1
    }
    I response.status.reason != "Gateway Timeout" {
        println("FAIL ssr forwarding timeout reason")
        return 2
    }
    I response.headers.get(CONTENT_TYPE) != "application/json" {
        println("FAIL ssr forwarding timeout content type")
        return 3
    }
    I smoke_contains(response.body, "SSR upstream timeout") != 1 {
        println("FAIL ssr forwarding timeout body")
        return 4
    }
    println("VAIS_SERVER_SSR_FORWARDING_TIMEOUT_RUNTIME_OK")
    0
}
"#
    .replace("__PORT__", &port.to_string());

    let (run, request_text) = run_vais_server_generated_timeout_smoke(
        "ssr_forwarding_timeout_runtime_smoke.vais",
        &source,
        listener,
    );

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "vais-server SSR forwarding timeout smoke exited unexpectedly.\nstdout:\n{}\nstderr:\n{}\nrequest:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        request_text
    );
    assert!(
        request_text.contains(r#"{"route":"/slow","props":"state"}"#),
        "SSR forwarding timeout request did not contain expected JSON body:\n{}",
        request_text
    );
}

#[test]
fn e2e_vais_server_15_ssr_forwarding_retry_runtime_smoke() {
    skip_if_no_vais_server_package!();
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind SSR retry listener");
    listener
        .set_nonblocking(true)
        .expect("set SSR retry listener nonblocking");
    let port = listener
        .local_addr()
        .expect("read SSR retry address")
        .port();

    let source = r#"
use src/api/ssr
use src/http/header

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
    response := forward_ssr_render_with_retry("http://127.0.0.1:__PORT__", "/retry", "state", 1000, 1)
    I response.status.code != 200 {
        println("FAIL ssr forwarding retry status code")
        return 1
    }
    I response.status.reason != "OK" {
        println("FAIL ssr forwarding retry reason")
        return 2
    }
    I response.headers.get(CONTENT_TYPE) != "application/json" {
        println("FAIL ssr forwarding retry content type")
        return 3
    }
    I smoke_contains(response.body, "retry-ok") != 1 {
        println("FAIL ssr forwarding retry body")
        return 4
    }
    println("VAIS_SERVER_SSR_FORWARDING_RETRY_RUNTIME_OK")
    0
}
"#
    .replace("__PORT__", &port.to_string());

    let (run, first_request, second_request) = run_vais_server_generated_retry_smoke(
        "ssr_forwarding_retry_runtime_smoke.vais",
        &source,
        listener,
    );

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "vais-server SSR forwarding retry smoke exited unexpectedly.\nstdout:\n{}\nstderr:\n{}\nfirst request:\n{}\nsecond request:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        first_request,
        second_request
    );
    assert!(
        first_request.contains(r#"{"route":"/retry","props":"state"}"#),
        "first SSR retry request did not contain expected JSON body:\n{}",
        first_request
    );
    assert!(
        second_request.contains(r#"{"route":"/retry","props":"state"}"#),
        "second SSR retry request did not contain expected JSON body:\n{}",
        second_request
    );
}

#[test]
fn e2e_vais_server_16_ssr_forwarding_retry_budget_observability_runtime_smoke() {
    skip_if_no_vais_server_package!();
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("bind SSR retry budget observability listener");
    listener
        .set_nonblocking(true)
        .expect("set SSR retry budget observability listener nonblocking");
    let port = listener
        .local_addr()
        .expect("read SSR retry budget observability address")
        .port();

    let source = r#"
use src/api/ssr
use src/http/header

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
    response := forward_ssr_render_with_retry_observed("http://127.0.0.1:__PORT__", "/exhaust", "state", 1000, 2, 25, 5)
    I response.status.code != 502 {
        println("FAIL ssr retry budget status code")
        return 1
    }
    I response.status.reason != "Bad Gateway" {
        println("FAIL ssr retry budget reason")
        return 2
    }
    I response.headers.get(CONTENT_TYPE) != "application/json" {
        println("FAIL ssr retry budget content type")
        return 3
    }
    I response.headers.get("X-SSR-Retry-Budget") != "exhausted" {
        println("FAIL ssr retry budget header")
        return 4
    }
    I response.headers.get("X-SSR-Retry-Backoff") != "base+jitter" {
        println("FAIL ssr retry backoff header")
        return 5
    }
    I response.headers.get("X-SSR-Retry-Last-Error") != "transport" {
        println("FAIL ssr retry last error header")
        return 6
    }
    I smoke_contains(response.body, "SSR retry budget exhausted") != 1 {
        println("FAIL ssr retry budget body marker")
        return 7
    }
    I smoke_contains(response.body, "backoff") != 1 {
        println("FAIL ssr retry budget backoff marker")
        return 8
    }
    I smoke_contains(response.body, "jitter") != 1 {
        println("FAIL ssr retry budget jitter marker")
        return 9
    }
    println("VAIS_SERVER_SSR_FORWARDING_RETRY_BUDGET_OBSERVABILITY_RUNTIME_OK")
    0
}
"#
    .replace("__PORT__", &port.to_string());

    let (run, requests) = run_vais_server_generated_retry_budget_smoke(
        "ssr_forwarding_retry_budget_observability_runtime_smoke.vais",
        &source,
        listener,
    );

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "vais-server SSR retry budget observability smoke exited unexpectedly.\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}\nrequests:\n{:?}",
        run.status,
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        requests
    );
    assert_eq!(
        requests.len(),
        3,
        "expected 3 dropped retry attempts (initial + 2 retries), got {}: {:?}",
        requests.len(),
        requests
    );
    for (idx, request) in requests.iter().enumerate() {
        assert!(
            request.contains(r#"{"route":"/exhaust","props":"state"}"#),
            "SSR retry budget request #{} did not contain expected JSON body:\n{}",
            idx,
            request
        );
    }
}

#[test]
fn e2e_vais_server_17_ssr_nested_json_props_runtime_smoke() {
    skip_if_no_vais_server_package!();
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("bind SSR nested JSON upstream listener");
    listener
        .set_nonblocking(true)
        .expect("set SSR nested JSON upstream listener nonblocking");
    let port = listener
        .local_addr()
        .expect("read SSR nested JSON upstream address")
        .port();

    let source = r#"
use src/api/ssr
use src/http/header
use src/http/method
use src/http/request

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

fn smoke_local_nested_json_props() -> i64 {
    body := "{\"route\":\"/dashboard\",\"props\":{\"user\":{\"name\":\"Ada\"},\"items\":[\"one\",\"two\"]}}"
    parsed := parse_render_request(body)
    I parsed.route != "/dashboard" {
        println("FAIL ssr nested parse route")
        return 1
    }
    I smoke_contains(parsed.props, "\"user\":{\"name\":\"Ada\"}") != 1 {
        println("FAIL ssr nested parse object props")
        return 2
    }
    I smoke_contains(parsed.props, "\"items\":[\"one\",\"two\"]") != 1 {
        println("FAIL ssr nested parse array props")
        return 3
    }

    req := mut Request.new(HttpMethod.Post, "/api/hydrate")
    req.headers = req.headers.set(CONTENT_TYPE, "application/json")
    req.body = body

    response := handle_ssr_hydrate(req)
    I response.status.code != 200 {
        println("FAIL ssr nested hydrate status")
        return 4
    }
    I response.headers.get(CONTENT_TYPE) != "application/json" {
        println("FAIL ssr nested hydrate content type")
        return 5
    }
    I smoke_contains(response.body, "\"data\": {\"user\":{\"name\":\"Ada\"}") != 1 {
        println("FAIL ssr nested hydrate data object")
        return 6
    }
    I smoke_contains(response.body, "\"items\":[\"one\",\"two\"]") != 1 {
        println("FAIL ssr nested hydrate data array")
        return 7
    }
    I smoke_contains(response.body, "\"route\": \"/dashboard\"") != 1 {
        println("FAIL ssr nested hydrate route")
        return 8
    }
    0
}

fn main() -> i64 {
    nested_props := "{\"user\":{\"name\":\"Ada\"},\"items\":[\"one\",\"two\"]}"
    response := forward_ssr_render("http://127.0.0.1:__PORT__", "/nested", nested_props)
    I response.status.code != 202 {
        println("FAIL ssr nested forwarding status")
        return 20
    }
    I response.headers.get(CONTENT_TYPE) != "application/json" {
        println("FAIL ssr nested forwarding content type")
        return 21
    }
    I smoke_contains(response.body, "from-node") != 1 {
        println("FAIL ssr nested forwarding body")
        return 22
    }

    local_result := smoke_local_nested_json_props()
    I local_result != 0 { return local_result }

    println("VAIS_SERVER_SSR_NESTED_JSON_PROPS_RUNTIME_OK")
    0
}
"#
    .replace("__PORT__", &port.to_string());

    let (run, request_text) = run_vais_server_generated_loopback_smoke(
        "ssr_nested_json_props_runtime_smoke.vais",
        &source,
        listener,
    );

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "vais-server SSR nested JSON props smoke exited unexpectedly.\nstdout:\n{}\nstderr:\n{}\nrequest:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        request_text
    );
    assert!(
        request_text.contains(
            r#"{"route":"/nested","props":{"user":{"name":"Ada"},"items":["one","two"]}}"#
        ),
        "SSR nested JSON forwarding request did not preserve props as raw JSON:\n{}",
        request_text
    );
}

#[test]
fn e2e_vais_server_18_ssr_json_escape_runtime_smoke() {
    skip_if_no_vais_server_package!();
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind SSR escape upstream listener");
    listener
        .set_nonblocking(true)
        .expect("set SSR escape upstream listener nonblocking");
    let port = listener
        .local_addr()
        .expect("read SSR escape upstream address")
        .port();

    let source = r#"
use src/api/ssr
use src/http/header

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
    route := "/quoted\"path\\tail"
    props := "state \"Ada\" \\ tail"

    local := do_hydrate(route, props)
    I local.status.code != 200 {
        println("FAIL ssr escape local status")
        return 1
    }
    I local.headers.get(CONTENT_TYPE) != "application/json" {
        println("FAIL ssr escape local content type")
        return 2
    }
    println(local.body)

    response := forward_ssr_render("http://127.0.0.1:__PORT__", route, props)
    I response.status.code != 202 {
        println("FAIL ssr escape forwarding status")
        return 10
    }
    I response.headers.get(CONTENT_TYPE) != "application/json" {
        println("FAIL ssr escape forwarding content type")
        return 11
    }
    I smoke_contains(response.body, "from-node") != 1 {
        println("FAIL ssr escape forwarding body")
        return 12
    }

    println("VAIS_SERVER_SSR_JSON_ESCAPE_RUNTIME_OK")
    0
}
"#
    .replace("__PORT__", &port.to_string());

    let (run, request_text) = run_vais_server_generated_loopback_smoke(
        "ssr_json_escape_runtime_smoke.vais",
        &source,
        listener,
    );

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "vais-server SSR JSON escape smoke exited unexpectedly.\nstdout:\n{}\nstderr:\n{}\nrequest:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        request_text
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        stdout.contains(r#""data": "state \"Ada\" \\ tail""#),
        "SSR hydrate response did not JSON-escape string props.\nstdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains(r#""route": "/quoted\"path\\tail""#),
        "SSR hydrate response did not JSON-escape route.\nstdout:\n{}",
        stdout
    );
    assert!(
        request_text.contains(r#"{"route":"/quoted\"path\\tail","props":"state \"Ada\" \\ tail"}"#),
        "SSR forwarding request did not JSON-escape string payload fields:\n{}",
        request_text
    );
}

#[test]
fn e2e_vais_server_19_ssr_json_grammar_runtime_smoke() {
    skip_if_no_vais_server_package!();
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("bind SSR JSON grammar upstream listener");
    listener
        .set_nonblocking(true)
        .expect("set SSR JSON grammar upstream listener nonblocking");
    let port = listener
        .local_addr()
        .expect("read SSR JSON grammar upstream address")
        .port();

    let source = r#"
use src/api/ssr
use src/http/header

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
    valid := "{\"text\":\"line\\nnext\",\"arr\":[true,false,null,-12.5e+2],\"nested\":{\"x\":0}}"
    local_valid := do_hydrate("/json-valid", valid)
    I local_valid.status.code != 200 {
        println("FAIL ssr json grammar valid local status")
        return 1
    }
    I local_valid.headers.get(CONTENT_TYPE) != "application/json" {
        println("FAIL ssr json grammar valid local content type")
        return 2
    }
    I smoke_contains(local_valid.body, "\"arr\":[true,false,null,-12.5e+2]") != 1 {
        println("FAIL ssr json grammar valid array")
        return 3
    }
    I smoke_contains(local_valid.body, "\"nested\":{\"x\":0}") != 1 {
        println("FAIL ssr json grammar valid object")
        return 4
    }
    println(local_valid.body)

    invalid := "{\"bad\":[1,]}"
    local_invalid := do_hydrate("/json-invalid", invalid)
    I local_invalid.status.code != 200 {
        println("FAIL ssr json grammar invalid local status")
        return 10
    }
    println(local_invalid.body)

    response := forward_ssr_render("http://127.0.0.1:__PORT__", "/json-invalid", invalid)
    I response.status.code != 202 {
        println("FAIL ssr json grammar forwarding status")
        return 20
    }
    I response.headers.get(CONTENT_TYPE) != "application/json" {
        println("FAIL ssr json grammar forwarding content type")
        return 21
    }
    I smoke_contains(response.body, "from-node") != 1 {
        println("FAIL ssr json grammar forwarding body")
        return 22
    }

    println("VAIS_SERVER_SSR_JSON_GRAMMAR_RUNTIME_OK")
    0
}
"#
    .replace("__PORT__", &port.to_string());

    let (run, request_text) = run_vais_server_generated_loopback_smoke(
        "ssr_json_grammar_runtime_smoke.vais",
        &source,
        listener,
    );

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "vais-server SSR JSON grammar smoke exited unexpectedly.\nstdout:\n{}\nstderr:\n{}\nrequest:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        request_text
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        stdout.contains(r#""data": {"text":"line\nnext","arr":[true,false,null,-12.5e+2],"nested":{"x":0}}"#),
        "SSR hydrate response did not preserve complete valid JSON grammar values as raw JSON.\nstdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains(r#""data": "{\"bad\":[1,]}""#),
        "SSR hydrate response did not escape invalid JSON props as a string.\nstdout:\n{}",
        stdout
    );
    assert!(
        request_text.contains(r#"{"route":"/json-invalid","props":"{\"bad\":[1,]}"}"#),
        "SSR forwarding request did not escape invalid JSON props as a string:\n{}",
        request_text
    );
}

fn run_vais_server_generated_retry_budget_smoke(
    fixture: &str,
    source: &str,
    listener: TcpListener,
) -> (std::process::Output, Vec<String>) {
    let _guard = VAIS_SERVER_SMOKE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let compiler_root = compiler_root();
    let isolated_dir = isolated_smoke_dir(fixture);
    let isolated_tmp_dir = isolated_dir.join("tmp");
    let isolated_fixture_path = isolated_dir.join(fixture);
    std::fs::create_dir_all(&isolated_dir).expect("failed to create isolated smoke source dir");
    std::fs::create_dir_all(&isolated_tmp_dir).expect("failed to create isolated smoke temp dir");
    std::fs::write(&isolated_fixture_path, source)
        .expect("failed to write generated vais-server smoke fixture");

    let exe_dir = PathBuf::from("/tmp/vais-smoke");
    std::fs::create_dir_all(&exe_dir).expect("failed to create /tmp/vais-smoke");
    let exe_path = exe_dir.join(format!(
        "vais_server_runtime_{}_{}",
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
        .env("VAIS_DEP_PATHS", vais_server_dep_paths(&compiler_root))
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to spawn vaisc for generated vais-server runtime smoke build");

    assert!(
        build.status.success(),
        "generated vais-server runtime smoke fixture `{fixture}` failed to build.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let server = thread::spawn(move || accept_n_ssr_drop_requests(listener, 3));
    let run = Command::new(&exe_path)
        .current_dir(&isolated_dir)
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to run generated vais-server runtime smoke executable");
    let requests = server
        .join()
        .expect("join SSR retry budget upstream listener");
    let _ = std::fs::remove_file(&exe_path);
    let _ = std::fs::remove_dir_all(&isolated_dir);
    (run, requests)
}

fn accept_n_ssr_drop_requests(listener: TcpListener, count: usize) -> Vec<String> {
    let mut collected = Vec::with_capacity(count);
    for _ in 0..count {
        let cloned = listener.try_clone().expect("clone retry budget listener");
        let request = accept_one_ssr_request_and_then(cloned, |_stream| {});
        collected.push(request);
    }
    collected
}

fn assert_vais_server_smoke_runs(fixture: &str, expected_exit: i32) {
    let _guard = VAIS_SERVER_SMOKE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(output) = run_vais_server_smoke_fixture(fixture) else {
        return;
    };
    let actual_exit = output.status.code().unwrap_or(-1);
    assert!(
        output.status.success() || actual_exit == expected_exit,
        "vais-server runtime smoke fixture `{fixture}` should compile, link, and exit {expected_exit}; got status {:?} / exit {actual_exit}.\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        actual_exit,
        expected_exit,
        "vais-server runtime smoke fixture `{fixture}` exited unexpectedly.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_vais_server_smoke_fixture(fixture: &str) -> Option<std::process::Output> {
    let compiler_root = compiler_root();
    let Some((server_root, vaisdb_src)) = vais_server_package_roots(&compiler_root) else {
        eprintln!("SKIP: vais-server source root not found; set VAIS_LANG_ROOT to run {fixture}");
        return None;
    };
    let fixture_path = compiler_root.join("tests/vais-server/smoke").join(fixture);
    assert!(
        fixture_path.is_file(),
        "missing vais-server smoke fixture at {}",
        fixture_path.display()
    );

    let smoke_root = fixture_path
        .parent()
        .expect("vais-server smoke fixture should have a parent directory");
    let _ = std::fs::remove_dir_all(smoke_root.join(".vais-cache"));
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join(".vais-cache"));
    remove_vais_cache_dirs(&compiler_root.join("std"));
    remove_vais_cache_dirs(&server_root);
    remove_vais_cache_dirs(&vaisdb_src);

    let isolated_dir = isolated_smoke_dir(fixture);
    let isolated_tmp_dir = isolated_dir.join("tmp");
    let isolated_fixture_path = isolated_dir.join(
        fixture_path
            .file_name()
            .expect("vais-server smoke fixture should have a file name"),
    );
    std::fs::create_dir_all(&isolated_dir).expect("failed to create isolated smoke source dir");
    std::fs::create_dir_all(&isolated_tmp_dir).expect("failed to create isolated smoke temp dir");
    std::fs::copy(&fixture_path, &isolated_fixture_path)
        .expect("failed to copy vais-server smoke fixture into isolated source dir");

    let exe_dir = PathBuf::from("/tmp/vais-smoke");
    std::fs::create_dir_all(&exe_dir).expect("failed to create /tmp/vais-smoke");
    let exe_path = exe_dir.join(format!(
        "vais_server_runtime_{}_{}",
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
        .env("VAIS_DEP_PATHS", vais_server_dep_paths(&compiler_root))
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to spawn vaisc for vais-server runtime smoke build");

    assert!(
        build.status.success(),
        "vais-server runtime smoke fixture `{fixture}` failed to build.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = Command::new(&exe_path)
        .current_dir(&isolated_dir)
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to run vais-server runtime smoke executable");
    let _ = std::fs::remove_file(&exe_path);
    let _ = std::fs::remove_dir_all(&isolated_dir);
    Some(run)
}

fn run_vais_server_generated_loopback_smoke(
    fixture: &str,
    source: &str,
    listener: TcpListener,
) -> (std::process::Output, String) {
    let _guard = VAIS_SERVER_SMOKE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let compiler_root = compiler_root();
    let isolated_dir = isolated_smoke_dir(fixture);
    let isolated_tmp_dir = isolated_dir.join("tmp");
    let isolated_fixture_path = isolated_dir.join(fixture);
    std::fs::create_dir_all(&isolated_dir).expect("failed to create isolated smoke source dir");
    std::fs::create_dir_all(&isolated_tmp_dir).expect("failed to create isolated smoke temp dir");
    std::fs::write(&isolated_fixture_path, source)
        .expect("failed to write generated vais-server smoke fixture");

    let exe_dir = PathBuf::from("/tmp/vais-smoke");
    std::fs::create_dir_all(&exe_dir).expect("failed to create /tmp/vais-smoke");
    let exe_path = exe_dir.join(format!(
        "vais_server_runtime_{}_{}",
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
        .env("VAIS_DEP_PATHS", vais_server_dep_paths(&compiler_root))
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to spawn vaisc for generated vais-server runtime smoke build");

    assert!(
        build.status.success(),
        "generated vais-server runtime smoke fixture `{fixture}` failed to build.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let server = thread::spawn(move || accept_one_ssr_forwarding_request(listener));
    let run = Command::new(&exe_path)
        .current_dir(&isolated_dir)
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to run generated vais-server runtime smoke executable");
    let request_text = server.join().expect("join SSR upstream listener");
    let _ = std::fs::remove_file(&exe_path);
    let _ = std::fs::remove_dir_all(&isolated_dir);
    (run, request_text)
}

fn run_vais_server_generated_dual_loopback_smoke(
    fixture: &str,
    source: &str,
    status_listener: TcpListener,
    drop_listener: TcpListener,
) -> (std::process::Output, String, String) {
    let _guard = VAIS_SERVER_SMOKE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let compiler_root = compiler_root();
    let isolated_dir = isolated_smoke_dir(fixture);
    let isolated_tmp_dir = isolated_dir.join("tmp");
    let isolated_fixture_path = isolated_dir.join(fixture);
    std::fs::create_dir_all(&isolated_dir).expect("failed to create isolated smoke source dir");
    std::fs::create_dir_all(&isolated_tmp_dir).expect("failed to create isolated smoke temp dir");
    std::fs::write(&isolated_fixture_path, source)
        .expect("failed to write generated vais-server smoke fixture");

    let exe_dir = PathBuf::from("/tmp/vais-smoke");
    std::fs::create_dir_all(&exe_dir).expect("failed to create /tmp/vais-smoke");
    let exe_path = exe_dir.join(format!(
        "vais_server_runtime_{}_{}",
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
        .env("VAIS_DEP_PATHS", vais_server_dep_paths(&compiler_root))
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to spawn vaisc for generated vais-server runtime smoke build");

    assert!(
        build.status.success(),
        "generated vais-server runtime smoke fixture `{fixture}` failed to build.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let status_server =
        thread::spawn(move || accept_one_ssr_status_mapping_request(status_listener));
    let drop_server = thread::spawn(move || accept_one_ssr_drop_request(drop_listener));
    let run = Command::new(&exe_path)
        .current_dir(&isolated_dir)
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to run generated vais-server runtime smoke executable");
    let status_request = status_server
        .join()
        .expect("join SSR status upstream listener");
    let drop_request = drop_server.join().expect("join SSR drop upstream listener");
    let _ = std::fs::remove_file(&exe_path);
    let _ = std::fs::remove_dir_all(&isolated_dir);
    (run, status_request, drop_request)
}

fn run_vais_server_generated_timeout_smoke(
    fixture: &str,
    source: &str,
    listener: TcpListener,
) -> (std::process::Output, String) {
    let _guard = VAIS_SERVER_SMOKE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let compiler_root = compiler_root();
    let isolated_dir = isolated_smoke_dir(fixture);
    let isolated_tmp_dir = isolated_dir.join("tmp");
    let isolated_fixture_path = isolated_dir.join(fixture);
    std::fs::create_dir_all(&isolated_dir).expect("failed to create isolated smoke source dir");
    std::fs::create_dir_all(&isolated_tmp_dir).expect("failed to create isolated smoke temp dir");
    std::fs::write(&isolated_fixture_path, source)
        .expect("failed to write generated vais-server smoke fixture");

    let exe_dir = PathBuf::from("/tmp/vais-smoke");
    std::fs::create_dir_all(&exe_dir).expect("failed to create /tmp/vais-smoke");
    let exe_path = exe_dir.join(format!(
        "vais_server_runtime_{}_{}",
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
        .env("VAIS_DEP_PATHS", vais_server_dep_paths(&compiler_root))
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to spawn vaisc for generated vais-server runtime smoke build");

    assert!(
        build.status.success(),
        "generated vais-server runtime smoke fixture `{fixture}` failed to build.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let server = thread::spawn(move || accept_one_ssr_timeout_request(listener));
    let run = Command::new(&exe_path)
        .current_dir(&isolated_dir)
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to run generated vais-server runtime smoke executable");
    let request_text = server.join().expect("join SSR timeout upstream listener");
    let _ = std::fs::remove_file(&exe_path);
    let _ = std::fs::remove_dir_all(&isolated_dir);
    (run, request_text)
}

fn run_vais_server_generated_retry_smoke(
    fixture: &str,
    source: &str,
    listener: TcpListener,
) -> (std::process::Output, String, String) {
    let _guard = VAIS_SERVER_SMOKE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let compiler_root = compiler_root();
    let isolated_dir = isolated_smoke_dir(fixture);
    let isolated_tmp_dir = isolated_dir.join("tmp");
    let isolated_fixture_path = isolated_dir.join(fixture);
    std::fs::create_dir_all(&isolated_dir).expect("failed to create isolated smoke source dir");
    std::fs::create_dir_all(&isolated_tmp_dir).expect("failed to create isolated smoke temp dir");
    std::fs::write(&isolated_fixture_path, source)
        .expect("failed to write generated vais-server smoke fixture");

    let exe_dir = PathBuf::from("/tmp/vais-smoke");
    std::fs::create_dir_all(&exe_dir).expect("failed to create /tmp/vais-smoke");
    let exe_path = exe_dir.join(format!(
        "vais_server_runtime_{}_{}",
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
        .env("VAIS_DEP_PATHS", vais_server_dep_paths(&compiler_root))
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to spawn vaisc for generated vais-server runtime smoke build");

    assert!(
        build.status.success(),
        "generated vais-server runtime smoke fixture `{fixture}` failed to build.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let server = thread::spawn(move || accept_two_ssr_retry_requests(listener));
    let run = Command::new(&exe_path)
        .current_dir(&isolated_dir)
        .env("TMPDIR", &isolated_tmp_dir)
        .env("TMP", &isolated_tmp_dir)
        .env("TEMP", &isolated_tmp_dir)
        .output()
        .expect("failed to run generated vais-server runtime smoke executable");
    let (first_request, second_request) = server.join().expect("join SSR retry upstream listener");
    let _ = std::fs::remove_file(&exe_path);
    let _ = std::fs::remove_dir_all(&isolated_dir);
    (run, first_request, second_request)
}

fn accept_one_ssr_forwarding_request(listener: TcpListener) -> String {
    accept_one_ssr_request_and_then(listener, |stream| {
        let body = r#"{"html":"<main data-route='/products/sku-42'>from-node</main>","head":"<title>remote</title>","status":202}"#;
        let response = format!(
            "HTTP/1.1 202 Accepted\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write SSR upstream response");
        stream.flush().expect("flush SSR upstream response");
    })
}

fn accept_one_ssr_timeout_request(listener: TcpListener) -> String {
    accept_one_ssr_request_and_then(listener, |_stream| {
        thread::sleep(Duration::from_millis(1000));
    })
}

fn accept_two_ssr_retry_requests(listener: TcpListener) -> (String, String) {
    let first = accept_one_ssr_request_and_then(
        listener.try_clone().expect("clone retry listener"),
        |_stream| {},
    );
    let second = accept_one_ssr_request_and_then(listener, |stream| {
        let body = r#"{"html":"<main>retry-ok</main>","status":200}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write SSR retry upstream response");
        stream.flush().expect("flush SSR retry upstream response");
    });
    (first, second)
}

fn accept_one_ssr_status_mapping_request(listener: TcpListener) -> String {
    let request = accept_one_ssr_request_and_then(listener, |stream| {
        let body = r#"{"error":"remote-down","status":503}"#;
        let response = format!(
            "HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write SSR status upstream response");
        stream.flush().expect("flush SSR status upstream response");
    });
    request
}

fn accept_one_ssr_drop_request(listener: TcpListener) -> String {
    accept_one_ssr_request_and_then(listener, |_stream| {})
}

fn accept_one_ssr_request_and_then(
    listener: TcpListener,
    respond: impl FnOnce(&mut std::net::TcpStream),
) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    let (mut stream, _) = loop {
        match listener.accept() {
            Ok(pair) => break pair,
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(
                    Instant::now() < deadline,
                    "timed out waiting for vais-server SSR forwarding connection"
                );
                thread::sleep(Duration::from_millis(25));
            }
            Err(err) => panic!("failed to accept vais-server SSR forwarding connection: {err}"),
        }
    };

    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set SSR upstream stream read timeout");

    let mut request = Vec::new();
    let mut buffer = [0_u8; 4096];
    let read_deadline = Instant::now() + Duration::from_secs(15);
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                request.extend_from_slice(&buffer[..n]);
                if request_has_complete_http_body(&request) {
                    break;
                }
            }
            Err(err)
                if err.kind() == std::io::ErrorKind::WouldBlock
                    || err.kind() == std::io::ErrorKind::TimedOut =>
            {
                if request_has_complete_http_body(&request) {
                    break;
                }
                assert!(
                    Instant::now() < read_deadline,
                    "timed out reading complete vais-server SSR forwarding request; partial request:\n{}",
                    String::from_utf8_lossy(&request)
                );
                thread::sleep(Duration::from_millis(25));
            }
            Err(err) => panic!("failed to read vais-server SSR forwarding request: {err}"),
        }
    }

    respond(&mut stream);
    String::from_utf8_lossy(&request).into_owned()
}

fn isolated_smoke_dir(fixture: &str) -> PathBuf {
    let unique = VAIS_SERVER_SMOKE_ID.fetch_add(1, Ordering::SeqCst);
    PathBuf::from("/tmp/vais-smoke-src").join(format!(
        "vais_server_{}_{}_{}",
        fixture.replace(['/', '.'], "_"),
        std::process::id(),
        unique
    ))
}

fn request_has_complete_http_body(request: &[u8]) -> bool {
    let text = String::from_utf8_lossy(request);
    let Some(header_end) = text.find("\r\n\r\n") else {
        return false;
    };
    let headers = &text[..header_end];
    let body_len = request.len().saturating_sub(header_end + 4);
    let Some(content_length) = headers.lines().find_map(|line| {
        line.strip_prefix("Content-Length:")
            .or_else(|| line.strip_prefix("content-length:"))
            .and_then(|raw| raw.trim().parse::<usize>().ok())
    }) else {
        return true;
    };
    body_len >= content_length
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

fn lang_root(compiler_root: &Path) -> Option<PathBuf> {
    if let Ok(path) = std::env::var("VAIS_LANG_ROOT") {
        let path = PathBuf::from(path);
        if path.is_dir() {
            return Some(path);
        }
    }

    for candidate in [
        workspace_root(compiler_root).join("lang"),
        compiler_root.join("vais-lang"),
        compiler_root.join("lang"),
    ] {
        if candidate.is_dir() {
            return Some(candidate);
        }
    }

    None
}

fn vais_server_package_roots(compiler_root: &Path) -> Option<(PathBuf, PathBuf)> {
    let lang_root = lang_root(compiler_root)?;
    let server_root = lang_root.join("packages/vais-server");
    let vaisdb_src = lang_root.join("packages/vaisdb/src");
    (server_root.is_dir() && vaisdb_src.is_dir()).then_some((server_root, vaisdb_src))
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

fn vais_server_dep_paths(compiler_root: &Path) -> String {
    let std_link = std_link(compiler_root);
    let (server_root, vaisdb_src) = vais_server_package_roots(compiler_root)
        .expect("vais-server source root should be available before building smoke fixtures");
    format!(
        "{}:{}:{}",
        server_root.display(),
        vaisdb_src.display(),
        std_link.display()
    )
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
