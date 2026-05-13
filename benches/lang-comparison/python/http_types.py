# HTTP types with error handling
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

class HttpErrorKind(Enum):
    BAD_REQUEST = "BadRequest"
    NOT_FOUND = "NotFound"
    INTERNAL = "Internal"

@dataclass
class HttpError(Exception):
    kind: HttpErrorKind
    message: str

@dataclass
class Header:
    name: str
    value: str

@dataclass
class Request:
    method: str
    path: str
    headers: list[Header] = field(default_factory=list)
    body: str = ""

@dataclass
class Response:
    status: int
    headers: list[Header] = field(default_factory=list)
    body: str = ""

def parse_method(s: str) -> str:
    if s in ("GET", "POST", "PUT", "DELETE"):
        return s
    raise HttpError(HttpErrorKind.BAD_REQUEST, "invalid method")

def route(req: Request) -> Response:
    if req.path == "/":
        return Response(status=200, body="OK")
    elif req.path == "/health":
        return Response(status=200, body="healthy")
    else:
        return Response(status=404, body="not found")

def format_response(res: Response) -> str:
    return f"HTTP/1.1 {res.status} {res.body}"

def main():
    req = Request(method="GET", path="/")
    res = route(req)
    print(format_response(res))

if __name__ == "__main__":
    main()
