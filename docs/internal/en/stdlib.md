# Vais Standard Library Design

**Version:** 1.0.0
**Date:** 2026-01-12

---

## Overview

The standard library is **Vais's basic toolbox**.

```
Design Principles:
1. Include only essentials (not too much)
2. Consistent API design
3. Balance between performance and safety
4. Thorough documentation
```

---

## Module Hierarchy

```
std/
├── core/           # Core (auto-imported, language basics)
├── io/             # Input/Output
├── fs/             # File system
├── net/            # Networking
├── data/           # Data formats
├── text/           # Text processing
├── time/           # Time/Date
├── math/           # Mathematics
├── collections/    # Data structures
├── async/          # Asynchronous
├── sync/           # Synchronization
├── crypto/         # Cryptography
├── encoding/       # Encoding
├── test/           # Testing
├── log/            # Logging
├── env/            # Environment
└── sys/            # System
```

---

## std.core (Auto-imported)

Basic language features available without explicit import.

### Types

```vais
# Basic types (builtin)
i, i8, i16, i32, i64       # Integers
u8, u16, u32, u64          # Unsigned integers
f, f32, f64                # Floats
b                          # Boolean
s                          # String
void                       # None

# Compound types
[T]                        # Array
{K: V}                     # Map
(T1, T2, ...)              # Tuple
?T                         # Option
!T                         # Result
```

### Basic Functions

```vais
# Output
print(value)               # Standard output
println(value)             # With newline
eprint(value)              # Standard error
eprintln(value)            # Error + newline

# Type conversion
str(value) -> s            # To string
int(s) -> ?i               # To integer
float(s) -> ?f             # To float
bool(value) -> b           # To boolean

# Utilities
len(collection) -> i       # Length (same as #)
range(start, end) -> [i]   # Range
range(end) -> [i]          # From 0
typeof(value) -> s         # Type name
assert(cond, msg?)         # Assertion
panic(msg)                 # Panic
```

### Option & Result

```vais
# Option
some(value) -> ?T          # Has value
nil -> ?T                  # No value

# Result
ok(value) -> !T            # Success
err(msg) -> !T             # Failure

# Operations
value? -> T                # Unwrap (propagate nil/err)
value ?? default -> T      # Unwrap with default
value! -> T                # Unwrap or panic
```

---

## std.io

Basic input/output functionality.

### Traits

```vais
# Read interface
trait Read {
    fn read(self, buf: &mut [u8]) -> !usize
    fn read_all(self) -> ![u8]
    fn read_line(self) -> !s
}

# Write interface
trait Write {
    fn write(self, data: [u8]) -> !usize
    fn write_all(self, data: [u8]) -> !void
    fn flush(self) -> !void
}

# Seek interface
trait Seek {
    fn seek(self, pos: SeekFrom) -> !i64
    fn position(self) -> !i64
}
```

### Standard I/O

```vais
use std.io

# Standard input
line = io.stdin.read_line()?
all = io.stdin.read_all()?

# Standard output
io.stdout.write_all("Hello\n".bytes())?
io.stdout.flush()?

# Standard error
io.stderr.write_all("Error!\n".bytes())?
```

### Buffered I/O

```vais
use std.io.{BufReader, BufWriter}

# Buffered reading
reader = BufReader.new(file)
for line in reader.lines() {
    process(line)
}

# Buffered writing
writer = BufWriter.new(file)
writer.write_all(data)?
writer.flush()?
```

---

## std.fs

File system operations.

### File Operations

```vais
use std.fs

# Reading
content = fs.read("file.txt")?              # Read all (string)
bytes = fs.read_bytes("file.bin")?          # Read all (bytes)
lines = fs.read_lines("file.txt")?          # Line by line

# Writing
fs.write("file.txt", content)?              # Write all
fs.write_bytes("file.bin", bytes)?          # Write bytes
fs.append("file.txt", more_content)?        # Append

# File object
file = fs.open("file.txt", "r")?            # Read mode
file = fs.open("file.txt", "w")?            # Write mode
file = fs.open("file.txt", "a")?            # Append mode
file = fs.open("file.txt", "rw")?           # Read+Write
file.close()
```

### Path Operations

```vais
use std.fs.path

# Path manipulation
p = path.join("dir", "subdir", "file.txt")
dir = path.dirname(p)                        # "dir/subdir"
name = path.basename(p)                      # "file.txt"
stem = path.stem(p)                          # "file"
ext = path.extension(p)                      # "txt"

# Path info
path.exists("file.txt")                      # Exists
path.is_file("file.txt")                     # Is file
path.is_dir("dir")                           # Is directory
path.is_absolute("/usr/bin")                 # Is absolute

# Normalization
path.normalize("./dir/../other")             # "other"
path.absolute("relative")                    # To absolute
```

### Directory Operations

```vais
use std.fs

# Directory operations
fs.mkdir("new_dir")?                         # Create
fs.mkdir_all("a/b/c")?                       # Create recursive
fs.rmdir("dir")?                             # Remove
fs.rmdir_all("dir")?                         # Remove recursive

# Listing
entries = fs.read_dir(".")?
for entry in entries {
    println(entry.name + " - " + entry.type)
}

# Recursive traversal
for entry in fs.walk(".") {
    if entry.is_file() && entry.name.ends(".vais") {
        println(entry.path)
    }
}
```

### File Metadata

```vais
use std.fs

meta = fs.metadata("file.txt")?
meta.size                                    # Size (bytes)
meta.modified                                # Modified time
meta.created                                 # Created time
meta.permissions                             # Permissions
meta.is_readonly                             # Read-only
```

---

## std.net

Networking.

### HTTP Client

```vais
use std.net.http

# Simple GET
response = http.get("https://api.example.com/data")?
body = response.text()?

# With options
response = http.get("https://api.example.com/data", {
    headers: {"Authorization": "Bearer token"},
    timeout: 30s,
})?

# POST
response = http.post("https://api.example.com/data", {
    json: {"name": "John", "age": 30},
})?

# Other methods
http.put(url, options)?
http.patch(url, options)?
http.delete(url, options)?
http.head(url)?
```

### HTTP Response

```vais
response.status                              # 200
response.status_text                         # "OK"
response.headers                             # {"Content-Type": "..."}
response.text()?                             # As string
response.json()?                             # Parse JSON
response.bytes()?                            # As bytes
```

### HTTP Server

```vais
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

```vais
use std.net.{TcpStream, TcpListener, UdpSocket}

# TCP client
conn = TcpStream.connect("localhost:8080")?
conn.write_all("Hello".bytes())?
response = conn.read_all()?

# TCP server
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

```vais
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

# URL encoding
url.encode("hello world")                    # "hello%20world"
url.decode("hello%20world")                  # "hello world"
```

---

## std.data

Data format processing.

### JSON

```vais
use std.data.json

# Parsing
data = json.parse('{"name": "John", "age": 30}')?
name = data["name"].as_str()?
age = data["age"].as_int()?

# Serialization
obj = {"name": "John", "age": 30}
text = json.stringify(obj)
pretty = json.stringify(obj, indent: 2)

# Type conversion
type User = {name: s, age: i}
user: User = json.parse_as('{"name": "John", "age": 30}')?
```

### CSV

```vais
use std.data.csv

# Reading
records = csv.parse(content)?
for row in records {
    println(row[0] + ", " + row[1])
}

# CSV with header
records = csv.parse(content, header: true)?
for row in records {
    println(row["name"] + ", " + row["email"])
}

# Writing
csv.stringify([
    ["name", "age"],
    ["John", "30"],
    ["Jane", "25"],
])
```

### TOML

```vais
use std.data.toml

# Parsing
config = toml.parse(content)?
version = config["package"]["version"].as_str()?

# Serialization
toml.stringify(config)
```

### YAML

```vais
use std.data.yaml

data = yaml.parse(content)?
yaml.stringify(data)
```

---

## std.text

Text processing.

### Regex

```vais
use std.text.regex

# Matching
re = regex.compile(r"\d+")?
re.is_match("abc123")                        # true
re.find("abc123")                            # some("123")
re.find_all("a1b2c3")                        # ["1", "2", "3"]

# Captures
re = regex.compile(r"(\w+)@(\w+)\.(\w+)")?
m = re.captures("test@example.com")?
m[0]                                         # "test@example.com"
m[1]                                         # "test"
m[2]                                         # "example"
m[3]                                         # "com"

# Replacement
re.replace("a1b2c3", "X")                    # "aXbXcX"
re.replace_first("a1b2c3", "X")              # "aXb2c3"
```

### String Formatting

```vais
use std.text.fmt

# Formatting
fmt.format("Hello, {}!", "World")            # "Hello, World!"
fmt.format("{} + {} = {}", 1, 2, 3)          # "1 + 2 = 3"
fmt.format("{name} is {age}", name: "John", age: 30)

# Number formatting
fmt.number(1234567.89, sep: ",")             # "1,234,567.89"
fmt.percent(0.1234, decimals: 1)             # "12.3%"
fmt.bytes(1024 * 1024)                       # "1 MB"
```

### Template

```vais
use std.text.template

tmpl = template.compile("Hello, {{name}}!")?
result = tmpl.render({name: "World"})        # "Hello, World!"

# Conditionals
tmpl = template.compile("""
{{if admin}}
  Welcome, Admin!
{{else}}
  Welcome, User!
{{end}}
""")?

# Loops
tmpl = template.compile("""
{{for item in items}}
  - {{item.name}}: {{item.price}}
{{end}}
""")?
```

---

## std.time

Time and date.

### DateTime

```vais
use std.time

# Current time
now = time.now()
now.year, now.month, now.day
now.hour, now.minute, now.second
now.weekday                                  # 0=Mon, 6=Sun

# Creation
dt = time.datetime(2026, 1, 12, 15, 30, 0)

# Parsing
dt = time.parse("2026-01-12T15:30:00Z", "RFC3339")?
dt = time.parse("2026-01-12", "%Y-%m-%d")?

# Formatting
dt.format("RFC3339")                         # "2026-01-12T15:30:00Z"
dt.format("%Y-%m-%d")                        # "2026-01-12"
```

### Duration

```vais
use std.time

# Creation
d = time.duration(hours: 2, minutes: 30)
d = 2.hours + 30.minutes                     # Same
d = time.seconds(3600)                       # 1 hour

# Properties
d.total_seconds                              # Total seconds
d.total_minutes                              # Total minutes
d.total_hours                                # Total hours

# Arithmetic
dt + 1.day                                   # One day later
dt - 1.week                                  # One week ago
dt2 - dt1                                    # Duration
```

### Timezone

```vais
use std.time.tz

# Timezone conversion
utc = time.now()
kst = utc.in_tz("Asia/Seoul")
pst = utc.in_tz("America/Los_Angeles")

# UTC
utc_now = time.utc_now()
```

### Timer

```vais
use std.time

# Sleep
time.sleep(1.second)
time.sleep(500.ms)

# Measurement
start = time.instant()
do_work()
elapsed = start.elapsed()
println("Took: " + elapsed.as_ms().str + "ms")
```

---

## std.math

Mathematical functions.

### Basic

```vais
use std.math

math.abs(-5)                                 # 5
math.min(3, 7)                               # 3
math.max(3, 7)                               # 7
math.clamp(x, 0, 100)                        # Between 0-100

math.floor(3.7)                              # 3
math.ceil(3.2)                               # 4
math.round(3.5)                              # 4
math.trunc(3.9)                              # 3
```

### Constants

```vais
math.PI                                      # 3.14159...
math.E                                       # 2.71828...
math.TAU                                     # 6.28318... (2π)
math.INF                                     # Infinity
math.NAN                                     # Not a Number
```

### Trigonometry

```vais
math.sin(x), math.cos(x), math.tan(x)
math.asin(x), math.acos(x), math.atan(x)
math.atan2(y, x)
math.sinh(x), math.cosh(x), math.tanh(x)
```

### Exponential

```vais
math.pow(2, 10)                              # 1024
math.sqrt(16)                                # 4.0
math.cbrt(27)                                # 3.0
math.exp(1)                                  # e
math.log(x)                                  # Natural log
math.log10(x)                                # Common log
math.log2(x)                                 # Binary log
```

### Random

```vais
use std.math.random

random.int(1, 100)                           # 1-100 integer
random.float()                               # 0.0-1.0
random.bool()                                # true/false
random.choice([1, 2, 3, 4, 5])               # Random choice
random.shuffle([1, 2, 3, 4, 5])              # Shuffle
random.sample([1, 2, 3, 4, 5], 3)            # Sample 3

# Seed
random.seed(42)
```

---

## std.collections

Advanced data structures.

### Set

```vais
use std.collections.Set

s = Set.new()
s = Set.from([1, 2, 3])

s.add(4)
s.remove(1)
s.has(2)                                     # true
s.len()                                      # 3

# Set operations
a | b                                        # Union
a & b                                        # Intersection
a - b                                        # Difference
a ^ b                                        # Symmetric difference
```

### Queue

```vais
use std.collections.Queue

q = Queue.new()
q.push(1)
q.push(2)
q.pop()                                      # some(1)
q.peek()                                     # some(2)
q.is_empty()
```

### Stack

```vais
use std.collections.Stack

s = Stack.new()
s.push(1)
s.push(2)
s.pop()                                      # some(2)
s.peek()                                     # some(1)
```

### Heap (Priority Queue)

```vais
use std.collections.Heap

# Min heap (default)
h = Heap.new()
h.push(3)
h.push(1)
h.push(2)
h.pop()                                      # some(1)

# Max heap
h = Heap.max()
```

### LinkedList

```vais
use std.collections.LinkedList

list = LinkedList.new()
list.push_front(1)
list.push_back(2)
list.pop_front()                             # some(1)
```

### OrderedMap

```vais
use std.collections.OrderedMap

m = OrderedMap.new()
m.set("a", 1)
m.set("b", 2)
# Maintains insertion order
for (k, v) in m {
    println(k + ": " + v.str)
}
```

---

## std.async

Asynchronous programming.

### Spawn

```vais
use std.async

# Create task
handle = async.spawn {
    expensive_work()
}

# Wait
result = handle.await

# Multiple tasks
handles = [1, 2, 3].@(n => async.spawn { work(n) })
results = async.join_all(handles).await
```

### Channel

```vais
use std.async.channel

# Unbounded channel
(tx, rx) = channel.unbounded()

# Bounded channel
(tx, rx) = channel.bounded(100)

# Send
tx.send(value).await
tx.try_send(value)?                          # Non-blocking

# Receive
value = rx.recv().await
value = rx.try_recv()?                       # Non-blocking

# Receive loop
for value in rx {
    process(value)
}
```

### Select

```vais
use std.async.select

result = select {
    value = rx1.recv() => handle1(value),
    value = rx2.recv() => handle2(value),
    _ = timeout(1.second) => handle_timeout(),
}
```

### Timeout

```vais
use std.async

result = async.timeout(1.second, async_work())?
# Error if not completed within 1 second
```

---

## std.sync

Synchronization primitives.

### Mutex

```vais
use std.sync.Mutex

counter = Mutex.new(0)

# Acquire lock
{
    guard = counter.lock()
    *guard += 1
}  # Auto-release

# try_lock
if guard = counter.try_lock() {
    *guard += 1
}
```

### RwLock

```vais
use std.sync.RwLock

data = RwLock.new(initial_value)

# Read (multiple threads can read simultaneously)
{
    guard = data.read()
    println(*guard)
}

# Write (exclusive)
{
    guard = data.write()
    *guard = new_value
}
```

### Atomic

```vais
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

Testing.

### Basic Tests

```vais
use std.test

#[test]
test_addition() = {
    assert_eq(1 + 1, 2)
}

#[test]
test_string() = {
    assert("hello".starts("he"))
    assert_ne("hello", "world")
}

#[test]
#[should_panic]
test_panic() = {
    panic("expected")
}
```

### Assertions

```vais
assert(condition)                            # Check condition
assert(condition, "message")                 # With message
assert_eq(actual, expected)                  # Equal
assert_ne(actual, expected)                  # Not equal
assert_lt(a, b)                              # a < b
assert_gt(a, b)                              # a > b
assert_some(option)                          # Is some
assert_none(option)                          # Is nil
assert_ok(result)                            # Is ok
assert_err(result)                           # Is err
```

### Test Organization

```vais
# Module tests
mod tests {
    use super.*

    #[test]
    test_internal() = {
        assert(internal_function())
    }
}

# Fixtures
#[fixture]
setup_db() -> Database = {
    Database.connect("test.db")
}

#[test]
test_with_db(db: Database) = {
    db.query("SELECT 1")
}
```

---

## std.log

Logging.

```vais
use std.log

log.debug("Debug message")
log.info("Info message")
log.warn("Warning message")
log.error("Error message")

# Structured logging
log.info("User logged in", {
    user_id: 123,
    ip: "192.168.1.1",
})

# Set log level
log.set_level(log.Level.Info)
```

---

## Summary

Standard library design principles:

| Principle | Description |
|-----------|-------------|
| **Minimal** | Include only essentials |
| **Consistent** | Consistent API design |
| **Safe** | Safe defaults |
| **Documented** | Document all functions |
| **Tested** | High test coverage |

**Well-designed standard library = Productive developers**
