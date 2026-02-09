// HTTP types with error handling
use std::fmt;

#[derive(Debug)]
enum HttpError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            HttpError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            HttpError::Internal(msg) => write!(f, "Internal: {}", msg),
        }
    }
}

struct Header {
    name: String,
    value: String,
}

struct Request {
    method: String,
    path: String,
    headers: Vec<Header>,
    body: String,
}

struct Response {
    status: u16,
    headers: Vec<Header>,
    body: String,
}

fn parse_method(s: &str) -> Result<&str, HttpError> {
    match s {
        "GET" | "POST" | "PUT" | "DELETE" => Ok(s),
        _ => Err(HttpError::BadRequest("invalid method".to_string())),
    }
}

fn route(req: &Request) -> Response {
    match req.path.as_str() {
        "/" => Response { status: 200, headers: vec![], body: "OK".to_string() },
        "/health" => Response { status: 200, headers: vec![], body: "healthy".to_string() },
        _ => Response { status: 404, headers: vec![], body: "not found".to_string() },
    }
}

fn format_response(res: &Response) -> String {
    format!("HTTP/1.1 {} {}", res.status, res.body)
}

fn main() {
    let req = Request {
        method: "GET".to_string(),
        path: "/".to_string(),
        headers: vec![],
        body: String::new(),
    };
    let res = route(&req);
    println!("{}", format_response(&res));
}
