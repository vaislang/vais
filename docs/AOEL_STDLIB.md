# AOEL Standard Library Design

**Version:** 1.0.0
**Date:** 2026-01-12

---

## Overview

표준 라이브러리는 **AOEL의 기본 도구 상자**입니다.

```
설계 원칙:
1. 필수적인 것만 포함 (과하지 않게)
2. 일관된 API 설계
3. 성능과 안전성 균형
4. 문서화 철저히
```

---

## Module Hierarchy

```
std/
├── core/           # 코어 (자동 import, 언어 기본)
├── io/             # 입출력
├── fs/             # 파일 시스템
├── net/            # 네트워킹
├── data/           # 데이터 포맷
├── text/           # 텍스트 처리
├── time/           # 시간/날짜
├── math/           # 수학
├── collections/    # 자료구조
├── async/          # 비동기
├── sync/           # 동기화
├── crypto/         # 암호화
├── encoding/       # 인코딩
├── test/           # 테스팅
├── log/            # 로깅
├── env/            # 환경
└── sys/            # 시스템
```

---

## std.core (자동 import)

언어의 기본 기능으로, 명시적 import 없이 사용 가능합니다.

### Types

```aoel
# 기본 타입 (builtin)
i, i8, i16, i32, i64       # 정수
u8, u16, u32, u64          # 부호 없는 정수
f, f32, f64                # 실수
b                          # 불리언
s                          # 문자열
void                       # 없음

# 컴파운드 타입
[T]                        # 배열
{K: V}                     # 맵
(T1, T2, ...)              # 튜플
?T                         # 옵션
!T                         # 결과
```

### Basic Functions

```aoel
# 출력
print(value)               # 표준 출력
println(value)             # 줄바꿈 포함
eprint(value)              # 표준 에러
eprintln(value)            # 에러 + 줄바꿈

# 타입 변환
str(value) -> s            # 문자열로
int(s) -> ?i               # 정수로
float(s) -> ?f             # 실수로
bool(value) -> b           # 불리언으로

# 유틸리티
len(collection) -> i       # 길이 (#과 동일)
range(start, end) -> [i]   # 범위
range(end) -> [i]          # 0부터
typeof(value) -> s         # 타입 이름
assert(cond, msg?)         # 어설션
panic(msg)                 # 패닉
```

### Option & Result

```aoel
# Option
some(value) -> ?T          # 값 있음
nil -> ?T                  # 값 없음

# Result
ok(value) -> !T            # 성공
err(msg) -> !T             # 실패

# 조작
value? -> T                # unwrap (nil/err면 전파)
value ?? default -> T      # unwrap with default
value! -> T                # unwrap or panic
```

---

## std.io

입출력 기본 기능.

### Traits

```aoel
# 읽기 인터페이스
trait Read {
    fn read(self, buf: &mut [u8]) -> !usize
    fn read_all(self) -> ![u8]
    fn read_line(self) -> !s
}

# 쓰기 인터페이스
trait Write {
    fn write(self, data: [u8]) -> !usize
    fn write_all(self, data: [u8]) -> !void
    fn flush(self) -> !void
}

# 탐색 인터페이스
trait Seek {
    fn seek(self, pos: SeekFrom) -> !i64
    fn position(self) -> !i64
}
```

### Standard I/O

```aoel
use std.io

# 표준 입력
line = io.stdin.read_line()?
all = io.stdin.read_all()?

# 표준 출력
io.stdout.write_all("Hello\n".bytes())?
io.stdout.flush()?

# 표준 에러
io.stderr.write_all("Error!\n".bytes())?
```

### Buffered I/O

```aoel
use std.io.{BufReader, BufWriter}

# 버퍼 읽기
reader = BufReader.new(file)
for line in reader.lines() {
    process(line)
}

# 버퍼 쓰기
writer = BufWriter.new(file)
writer.write_all(data)?
writer.flush()?
```

---

## std.fs

파일 시스템 작업.

### File Operations

```aoel
use std.fs

# 읽기
content = fs.read("file.txt")?              # 전체 읽기 (문자열)
bytes = fs.read_bytes("file.bin")?          # 전체 읽기 (바이트)
lines = fs.read_lines("file.txt")?          # 줄 단위

# 쓰기
fs.write("file.txt", content)?              # 전체 쓰기
fs.write_bytes("file.bin", bytes)?          # 바이트 쓰기
fs.append("file.txt", more_content)?        # 추가

# 파일 객체
file = fs.open("file.txt", "r")?            # 읽기 모드
file = fs.open("file.txt", "w")?            # 쓰기 모드
file = fs.open("file.txt", "a")?            # 추가 모드
file = fs.open("file.txt", "rw")?           # 읽기+쓰기
file.close()
```

### Path Operations

```aoel
use std.fs.path

# 경로 조작
p = path.join("dir", "subdir", "file.txt")
dir = path.dirname(p)                        # "dir/subdir"
name = path.basename(p)                      # "file.txt"
stem = path.stem(p)                          # "file"
ext = path.extension(p)                      # "txt"

# 경로 정보
path.exists("file.txt")                      # 존재 여부
path.is_file("file.txt")                     # 파일인지
path.is_dir("dir")                           # 디렉토리인지
path.is_absolute("/usr/bin")                 # 절대 경로인지

# 정규화
path.normalize("./dir/../other")             # "other"
path.absolute("relative")                    # 절대 경로로
```

### Directory Operations

```aoel
use std.fs

# 디렉토리 작업
fs.mkdir("new_dir")?                         # 생성
fs.mkdir_all("a/b/c")?                       # 재귀 생성
fs.rmdir("dir")?                             # 삭제
fs.rmdir_all("dir")?                         # 재귀 삭제

# 목록
entries = fs.read_dir(".")?
for entry in entries {
    println(entry.name + " - " + entry.type)
}

# 재귀 탐색
for entry in fs.walk(".") {
    if entry.is_file() && entry.name.ends(".aoel") {
        println(entry.path)
    }
}
```

### File Metadata

```aoel
use std.fs

meta = fs.metadata("file.txt")?
meta.size                                    # 크기 (바이트)
meta.modified                                # 수정 시간
meta.created                                 # 생성 시간
meta.permissions                             # 권한
meta.is_readonly                             # 읽기 전용
```

---

## std.net

네트워킹.

### HTTP Client

```aoel
use std.net.http

# 간단한 GET
response = http.get("https://api.example.com/data")?
body = response.text()?

# 옵션과 함께
response = http.get("https://api.example.com/data", {
    headers: {"Authorization": "Bearer token"},
    timeout: 30s,
})?

# POST
response = http.post("https://api.example.com/data", {
    json: {"name": "John", "age": 30},
})?

# 다른 메서드
http.put(url, options)?
http.patch(url, options)?
http.delete(url, options)?
http.head(url)?
```

### HTTP Response

```aoel
response.status                              # 200
response.status_text                         # "OK"
response.headers                             # {"Content-Type": "..."}
response.text()?                             # 문자열로
response.json()?                             # JSON 파싱
response.bytes()?                            # 바이트로
```

### HTTP Server

```aoel
use std.net.http.{Server, Request, Response}

server = Server.new()

server.get("/", fn(req) {
    Response.text("Hello, World!")
})

server.get("/users/:id", fn(req) {
    id = req.params["id"]
    user = find_user(id)?
    Response.json(user)
})

server.post("/users", fn(req) {
    data = req.json()?
    user = create_user(data)?
    Response.json(user).status(201)
})

server.listen(8080)?
```

### TCP/UDP

```aoel
use std.net.{TcpStream, TcpListener, UdpSocket}

# TCP 클라이언트
conn = TcpStream.connect("localhost:8080")?
conn.write_all("Hello".bytes())?
response = conn.read_all()?

# TCP 서버
listener = TcpListener.bind("0.0.0.0:8080")?
for conn in listener.incoming() {
    spawn {
        handle_connection(conn)
    }
}

# UDP
socket = UdpSocket.bind("0.0.0.0:8080")?
socket.send_to(data, "localhost:9000")?
(data, addr) = socket.recv_from()?
```

### URL

```aoel
use std.net.url

u = url.parse("https://user:pass@example.com:8080/path?q=1#hash")?
u.scheme                                     # "https"
u.username                                   # "user"
u.password                                   # "pass"
u.host                                       # "example.com"
u.port                                       # 8080
u.path                                       # "/path"
u.query                                      # "q=1"
u.fragment                                   # "hash"

# URL 인코딩
url.encode("hello world")                    # "hello%20world"
url.decode("hello%20world")                  # "hello world"
```

---

## std.data

데이터 포맷 처리.

### JSON

```aoel
use std.data.json

# 파싱
data = json.parse('{"name": "John", "age": 30}')?
name = data["name"].as_str()?
age = data["age"].as_int()?

# 직렬화
obj = {"name": "John", "age": 30}
text = json.stringify(obj)
pretty = json.stringify(obj, indent: 2)

# 타입 변환
type User = {name: s, age: i}
user: User = json.parse_as('{"name": "John", "age": 30}')?
```

### CSV

```aoel
use std.data.csv

# 읽기
records = csv.parse(content)?
for row in records {
    println(row[0] + ", " + row[1])
}

# 헤더 있는 CSV
records = csv.parse(content, header: true)?
for row in records {
    println(row["name"] + ", " + row["email"])
}

# 쓰기
csv.stringify([
    ["name", "age"],
    ["John", "30"],
    ["Jane", "25"],
])
```

### TOML

```aoel
use std.data.toml

# 파싱
config = toml.parse(content)?
version = config["package"]["version"].as_str()?

# 직렬화
toml.stringify(config)
```

### YAML

```aoel
use std.data.yaml

data = yaml.parse(content)?
yaml.stringify(data)
```

---

## std.text

텍스트 처리.

### Regex

```aoel
use std.text.regex

# 매칭
re = regex.compile(r"\d+")?
re.is_match("abc123")                        # true
re.find("abc123")                            # some("123")
re.find_all("a1b2c3")                        # ["1", "2", "3"]

# 캡처
re = regex.compile(r"(\w+)@(\w+)\.(\w+)")?
m = re.captures("test@example.com")?
m[0]                                         # "test@example.com"
m[1]                                         # "test"
m[2]                                         # "example"
m[3]                                         # "com"

# 치환
re.replace("a1b2c3", "X")                    # "aXbXcX"
re.replace_first("a1b2c3", "X")              # "aXb2c3"
```

### String Formatting

```aoel
use std.text.fmt

# 포맷팅
fmt.format("Hello, {}!", "World")            # "Hello, World!"
fmt.format("{} + {} = {}", 1, 2, 3)          # "1 + 2 = 3"
fmt.format("{name} is {age}", name: "John", age: 30)

# 숫자 포맷
fmt.number(1234567.89, sep: ",")             # "1,234,567.89"
fmt.percent(0.1234, decimals: 1)             # "12.3%"
fmt.bytes(1024 * 1024)                       # "1 MB"
```

### Template

```aoel
use std.text.template

tmpl = template.compile("Hello, {{name}}!")?
result = tmpl.render({name: "World"})        # "Hello, World!"

# 조건
tmpl = template.compile("""
{{if admin}}
  Welcome, Admin!
{{else}}
  Welcome, User!
{{end}}
""")?

# 반복
tmpl = template.compile("""
{{for item in items}}
  - {{item.name}}: {{item.price}}
{{end}}
""")?
```

---

## std.time

시간과 날짜.

### DateTime

```aoel
use std.time

# 현재 시간
now = time.now()
now.year, now.month, now.day
now.hour, now.minute, now.second
now.weekday                                  # 0=월, 6=일

# 생성
dt = time.datetime(2026, 1, 12, 15, 30, 0)

# 파싱
dt = time.parse("2026-01-12T15:30:00Z", "RFC3339")?
dt = time.parse("2026-01-12", "%Y-%m-%d")?

# 포맷팅
dt.format("RFC3339")                         # "2026-01-12T15:30:00Z"
dt.format("%Y년 %m월 %d일")                   # "2026년 01월 12일"
```

### Duration

```aoel
use std.time

# 생성
d = time.duration(hours: 2, minutes: 30)
d = 2.hours + 30.minutes                     # 동일
d = time.seconds(3600)                       # 1시간

# 속성
d.total_seconds                              # 총 초
d.total_minutes                              # 총 분
d.total_hours                                # 총 시간

# 산술
dt + 1.day                                   # 하루 후
dt - 1.week                                  # 일주일 전
dt2 - dt1                                    # Duration
```

### Timezone

```aoel
use std.time.tz

# 타임존 변환
utc = time.now()
kst = utc.in_tz("Asia/Seoul")
pst = utc.in_tz("America/Los_Angeles")

# UTC
utc_now = time.utc_now()
```

### Timer

```aoel
use std.time

# 슬립
time.sleep(1.second)
time.sleep(500.ms)

# 측정
start = time.instant()
do_work()
elapsed = start.elapsed()
println("Took: " + elapsed.as_ms().str + "ms")
```

---

## std.math

수학 함수.

### Basic

```aoel
use std.math

math.abs(-5)                                 # 5
math.min(3, 7)                               # 3
math.max(3, 7)                               # 7
math.clamp(x, 0, 100)                        # 0-100 사이

math.floor(3.7)                              # 3
math.ceil(3.2)                               # 4
math.round(3.5)                              # 4
math.trunc(3.9)                              # 3
```

### Constants

```aoel
math.PI                                      # 3.14159...
math.E                                       # 2.71828...
math.TAU                                     # 6.28318... (2π)
math.INF                                     # 무한대
math.NAN                                     # Not a Number
```

### Trigonometry

```aoel
math.sin(x), math.cos(x), math.tan(x)
math.asin(x), math.acos(x), math.atan(x)
math.atan2(y, x)
math.sinh(x), math.cosh(x), math.tanh(x)
```

### Exponential

```aoel
math.pow(2, 10)                              # 1024
math.sqrt(16)                                # 4.0
math.cbrt(27)                                # 3.0
math.exp(1)                                  # e
math.log(x)                                  # 자연로그
math.log10(x)                                # 상용로그
math.log2(x)                                 # 이진로그
```

### Random

```aoel
use std.math.random

random.int(1, 100)                           # 1-100 정수
random.float()                               # 0.0-1.0
random.bool()                                # true/false
random.choice([1, 2, 3, 4, 5])               # 랜덤 선택
random.shuffle([1, 2, 3, 4, 5])              # 섞기
random.sample([1, 2, 3, 4, 5], 3)            # 3개 샘플

# 시드 설정
random.seed(42)
```

---

## std.collections

고급 자료구조.

### Set

```aoel
use std.collections.Set

s = Set.new()
s = Set.from([1, 2, 3])

s.add(4)
s.remove(1)
s.has(2)                                     # true
s.len()                                      # 3

# 집합 연산
a | b                                        # 합집합
a & b                                        # 교집합
a - b                                        # 차집합
a ^ b                                        # 대칭차집합
```

### Queue

```aoel
use std.collections.Queue

q = Queue.new()
q.push(1)
q.push(2)
q.pop()                                      # some(1)
q.peek()                                     # some(2)
q.is_empty()
```

### Stack

```aoel
use std.collections.Stack

s = Stack.new()
s.push(1)
s.push(2)
s.pop()                                      # some(2)
s.peek()                                     # some(1)
```

### Heap (Priority Queue)

```aoel
use std.collections.Heap

# Min heap (기본)
h = Heap.new()
h.push(3)
h.push(1)
h.push(2)
h.pop()                                      # some(1)

# Max heap
h = Heap.max()
```

### LinkedList

```aoel
use std.collections.LinkedList

list = LinkedList.new()
list.push_front(1)
list.push_back(2)
list.pop_front()                             # some(1)
```

### OrderedMap

```aoel
use std.collections.OrderedMap

m = OrderedMap.new()
m.set("a", 1)
m.set("b", 2)
# 삽입 순서 유지
for (k, v) in m {
    println(k + ": " + v.str)
}
```

---

## std.async

비동기 프로그래밍.

### Spawn

```aoel
use std.async

# 태스크 생성
handle = async.spawn {
    expensive_work()
}

# 대기
result = handle.await

# 여러 태스크
handles = [1, 2, 3].@(n => async.spawn { work(n) })
results = async.join_all(handles).await
```

### Channel

```aoel
use std.async.channel

# Unbounded channel
(tx, rx) = channel.unbounded()

# Bounded channel
(tx, rx) = channel.bounded(100)

# 송신
tx.send(value).await
tx.try_send(value)?                          # 논블로킹

# 수신
value = rx.recv().await
value = rx.try_recv()?                       # 논블로킹

# 반복 수신
for value in rx {
    process(value)
}
```

### Select

```aoel
use std.async.select

result = select {
    value = rx1.recv() => handle1(value),
    value = rx2.recv() => handle2(value),
    _ = timeout(1.second) => handle_timeout(),
}
```

### Timeout

```aoel
use std.async

result = async.timeout(1.second, async_work())?
# 1초 안에 완료되지 않으면 에러
```

---

## std.sync

동기화 프리미티브.

### Mutex

```aoel
use std.sync.Mutex

counter = Mutex.new(0)

# 락 획득
{
    guard = counter.lock()
    *guard += 1
}  # 자동 해제

# try_lock
if guard = counter.try_lock() {
    *guard += 1
}
```

### RwLock

```aoel
use std.sync.RwLock

data = RwLock.new(initial_value)

# 읽기 (여러 스레드 동시 가능)
{
    guard = data.read()
    println(*guard)
}

# 쓰기 (독점)
{
    guard = data.write()
    *guard = new_value
}
```

### Atomic

```aoel
use std.sync.Atomic

counter = Atomic.new(0)
counter.add(1)
counter.sub(1)
counter.load()
counter.store(100)
counter.compare_exchange(expected, new)
```

---

## std.test

테스팅.

### Basic Tests

```aoel
use std.test

#[test]
fn test_addition() {
    assert_eq(1 + 1, 2)
}

#[test]
fn test_string() {
    assert("hello".starts("he"))
    assert_ne("hello", "world")
}

#[test]
#[should_panic]
fn test_panic() {
    panic("expected")
}
```

### Assertions

```aoel
assert(condition)                            # 조건 확인
assert(condition, "message")                 # 메시지 포함
assert_eq(actual, expected)                  # 같음
assert_ne(actual, expected)                  # 다름
assert_lt(a, b)                              # a < b
assert_gt(a, b)                              # a > b
assert_some(option)                          # some인지
assert_none(option)                          # nil인지
assert_ok(result)                            # ok인지
assert_err(result)                           # err인지
```

### Test Organization

```aoel
# 모듈 테스트
mod tests {
    use super.*

    #[test]
    fn test_internal() {
        assert(internal_function())
    }
}

# 픽스처
#[fixture]
fn setup_db() -> Database {
    Database.connect("test.db")
}

#[test]
fn test_with_db(db: Database) {
    db.query("SELECT 1")
}
```

---

## std.log

로깅.

```aoel
use std.log

log.debug("Debug message")
log.info("Info message")
log.warn("Warning message")
log.error("Error message")

# 구조화된 로깅
log.info("User logged in", {
    user_id: 123,
    ip: "192.168.1.1",
})

# 로그 레벨 설정
log.set_level(log.Level.Info)
```

---

## Summary

표준 라이브러리 설계 원칙:

| 원칙 | 설명 |
|------|------|
| **Minimal** | 필수 기능만 포함 |
| **Consistent** | 일관된 API 설계 |
| **Safe** | 안전한 기본값 |
| **Documented** | 모든 함수 문서화 |
| **Tested** | 높은 테스트 커버리지 |

**잘 설계된 표준 라이브러리 = 생산적인 개발자**
