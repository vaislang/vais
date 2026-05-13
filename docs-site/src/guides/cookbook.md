# Vais Cookbook — 실전 레시피 모음

이 페이지는 Vais 언어로 자주 사용되는 패턴과 작업을 빠르게 해결할 수 있는 실전 코드 레시피를 제공합니다. 각 레시피는 바로 실행 가능한 완성형 코드입니다.

---

## 기본 (Basics)

### Hello World

```vais
fn main() {
  print("Hello, Vais!")
}
```

### 명령줄 인수 처리

```vais
use std/args

fn main() {
  args := get_args()
  I args.len() > 1 {
    print("First argument: ")
    print(args.get(1))
  } else {
    print("No arguments provided")
  }
}
```

### 환경변수 읽기

```vais
use std/env

fn main() {
  path := env_get("PATH")
  match path {
    Some(p) => print(p),
    None => print("PATH not set")
  }
}
```

---

## 문자열 (Strings)

### 문자열 결합

```vais
fn main() {
  first := "Hello"
  second := "World"
  result := first + " " + second
  print(result)  # "Hello World"
}
```

### 문자열 분할

```vais
use std/string

fn main() {
  text := "apple,banana,cherry"
  parts := str_split(text, ",")
  I parts.len() > 0 {
    print(parts.get(0))  # "apple"
  }
}
```

### 문자열 포맷팅

```vais
fn main() {
  name := "Alice"
  age := 30
  msg := "Name: " + name + ", Age: " + i64_to_str(age)
  print(msg)
}
```

### 문자열 트림 및 대소문자 변환

```vais
use std/string

fn main() {
  text := "  Hello  "
  trimmed := str_trim(text)
  upper := str_to_upper(trimmed)
  lower := str_to_lower(trimmed)
  print(upper)  # "HELLO"
  print(lower)  # "hello"
}
```

---

## 컬렉션 (Collections)

### Vec 생성 및 조작

```vais
use std/collections

fn main() {
  v := Vec::new<i64>()
  v.push(10)
  v.push(20)
  v.push(30)

  I v.len() > 0 {
    first := v.get(0)
    print(i64_to_str(first))  # "10"
  }
}
```

### HashMap 사용

```vais
use std/collections

fn main() {
  map := HashMap::new<str, i64>()
  map.insert("apple", 5)
  map.insert("banana", 10)

  match map.get("apple") {
    Some(val) => print(i64_to_str(val)),  # "5"
    None => print("Not found")
  }
}
```

### 정렬

```vais
use std/collections

fn main() {
  v := Vec::new<i64>()
  v.push(30)
  v.push(10)
  v.push(20)

  v.sort()

  # 정렬 결과: [10, 20, 30]
  L i:0..v.len() {
    print(i64_to_str(v.get(i)))
  }
}
```

### 필터 및 맵

```vais
use std/collections

fn is_even(x: i64) -> bool {
  x % 2 == 0
}

fn double(x: i64) -> i64 {
  x * 2
}

fn main() {
  v := Vec::new<i64>()
  v.push(1)
  v.push(2)
  v.push(3)
  v.push(4)

  evens := v.filter(is_even)
  doubled := evens.map(double)

  # 결과: [4, 8]
}
```

---

## 파일 I/O

### 파일 읽기

```vais
use std/io

fn main() {
  content := file_read("input.txt")
  match content {
    Ok(data) => print(data),
    Err(e) => print("Error reading file")
  }
}
```

### 파일 쓰기

```vais
use std/io

fn main() {
  result := file_write("output.txt", "Hello, file!")
  match result {
    Ok(_) => print("Write successful"),
    Err(e) => print("Write failed")
  }
}
```

### 디렉토리 탐색

```vais
use std/io

fn main() {
  entries := dir_read(".")
  match entries {
    Ok(files) => {
      L i:0..files.len() {
        print(files.get(i))
      }
    },
    Err(e) => print("Error reading directory")
  }
}
```

### CSV 파싱

```vais
use std/string

fn main() {
  csv := "name,age,city\nAlice,30,NYC\nBob,25,LA"
  lines := str_split(csv, "\n")

  L i:1..lines.len() {
    line := lines.get(i)
    fields := str_split(line, ",")
    I fields.len() >= 3 {
      name := fields.get(0)
      age := fields.get(1)
      city := fields.get(2)
      print("Name: " + name + ", Age: " + age + ", City: " + city)
    }
  }
}
```

---

## 에러 처리 (Error Handling)

### Result 체이닝

```vais
use std/io

fn process_file(path: str) -> Result<i64, str> {
  content := file_read(path)?
  len := str_len(content)
  Ok(len)
}

fn main() {
  match process_file("data.txt") {
    Ok(len) => print("File length: " + i64_to_str(len)),
    Err(e) => print("Error: " + e)
  }
}
```

### ? 연산자 사용

```vais
use std/io

fn read_and_parse(path: str) -> Result<i64, str> {
  content := file_read(path)?
  num := str_to_i64(content)?
  Ok(num * 2)
}

fn main() {
  result := read_and_parse("number.txt")
  match result {
    Ok(n) => print(i64_to_str(n)),
    Err(e) => print("Failed: " + e)
  }
}
```

### 커스텀 에러 타입

```vais
enum MyError {
  NotFound(str),
  InvalidInput(str),
  IoError(str)
}

fn find_user(id: i64) -> Result<str, MyError> {
  I id < 0 {
    return Err(MyError::InvalidInput("ID must be positive"))
  }
  I id > 1000 {
    return Err(MyError::NotFound("User not found"))
  }
  Ok("User_" + i64_to_str(id))
}

fn main() {
  match find_user(1500) {
    Ok(user) => print(user),
    Err(MyError::NotFound(msg)) => print("Not found: " + msg),
    Err(MyError::InvalidInput(msg)) => print("Invalid: " + msg),
    Err(_) => print("Unknown error")
  }
}
```

---

## 비동기 (Async)

### 기본 async/await

```vais
A fn fetch_data() -> str {
  # 비동기 작업 시뮬레이션
  return "Data fetched"
}

A fn main() {
  result := fetch_data().Y
  print(result)
}
```

### HTTP 요청

```vais
use std/http

A fn fetch_url(url: str) -> Result<str, str> {
  response := http_get(url).Y?
  Ok(response.body)
}

A fn main() {
  match fetch_url("https://api.example.com/data").Y {
    Ok(body) => print(body),
    Err(e) => print("HTTP error: " + e)
  }
}
```

### 동시 작업

```vais
use std/async

A fn task1() -> i64 {
  return 42
}

A fn task2() -> i64 {
  return 100
}

A fn main() {
  future1 := spawn(task1())
  future2 := spawn(task2())

  result1 := future1.Y
  result2 := future2.Y

  total := result1 + result2
  print("Total: " + i64_to_str(total))  # "Total: 142"
}
```

---

## FFI (Foreign Function Interface)

### C 함수 호출

```vais
extern fn strlen(s: i64) -> i64

fn main() {
  text := "Hello"
  ptr := str_to_ptr(text)
  length := strlen(ptr)
  print("Length: " + i64_to_str(length))  # "Length: 5"
}
```

### 공유 라이브러리 사용

```vais
extern fn my_c_function(x: i64) -> i64

fn main() {
  result := my_c_function(10)
  print(i64_to_str(result))
}
```

---

## JSON

### JSON 파싱

```vais
use std/json

fn main() {
  json_str := "{\"name\":\"Alice\",\"age\":30}"
  parsed := json_parse(json_str)

  match parsed {
    Ok(obj) => {
      name := json_get_string(obj, "name")
      age := json_get_i64(obj, "age")
      print("Name: " + name)
      print("Age: " + i64_to_str(age))
    },
    Err(e) => print("Parse error")
  }
}
```

### JSON 생성

```vais
use std/json

fn main() {
  obj := json_object_new()
  json_set_string(obj, "name", "Bob")
  json_set_i64(obj, "age", 25)
  json_set_bool(obj, "active", true)

  json_str := json_stringify(obj)
  print(json_str)  # {"name":"Bob","age":25,"active":true}
}
```

### JSON 배열

```vais
use std/json

fn main() {
  arr := json_array_new()
  json_array_push_i64(arr, 10)
  json_array_push_i64(arr, 20)
  json_array_push_i64(arr, 30)

  json_str := json_stringify(arr)
  print(json_str)  # [10,20,30]
}
```

---

## 네트워킹 (Networking)

### TCP 서버

```vais
use std/net

fn handle_client(conn: TcpStream) {
  msg := tcp_read(conn)
  match msg {
    Ok(data) => {
      response := "Echo: " + data
      tcp_write(conn, response)
    },
    Err(e) => print("Read error")
  }
  tcp_close(conn)
}

fn main() {
  listener := tcp_listen("127.0.0.1:8080")
  match listener {
    Ok(l) => {
      print("Server listening on :8080")
      L {
        match tcp_accept(l) {
          Ok(conn) => handle_client(conn),
          Err(e) => print("Accept error")
        }
      }
    },
    Err(e) => print("Listen error")
  }
}
```

### HTTP 서버

```vais
use std/http

fn handler(req: HttpRequest) -> HttpResponse {
  I req.path == "/" {
    return http_response(200, "Hello, HTTP!")
  } else {
    return http_response(404, "Not Found")
  }
}

fn main() {
  server := http_server_new("127.0.0.1:8000", handler)
  match server {
    Ok(s) => {
      print("HTTP server running on :8000")
      http_serve(s)
    },
    Err(e) => print("Server error")
  }
}
```

### WebSocket 서버

```vais
use std/websocket

fn on_message(conn: WsConnection, msg: str) {
  print("Received: " + msg)
  ws_send(conn, "Echo: " + msg)
}

fn main() {
  server := ws_server_new("127.0.0.1:9000", on_message)
  match server {
    Ok(s) => {
      print("WebSocket server on :9000")
      ws_serve(s)
    },
    Err(e) => print("WS error")
  }
}
```

---

## 동시성 (Concurrency)

### 스레드 생성

```vais
use std/thread

fn worker(id: i64) {
  print("Worker " + i64_to_str(id) + " started")
}

fn main() {
  t1 := thread_spawn(|| { worker(1) })
  t2 := thread_spawn(|| { worker(2) })

  thread_join(t1)
  thread_join(t2)

  print("All workers done")
}
```

### 뮤텍스 (Mutex)

```vais
use std/sync

fn main() {
  counter := mutex_new(0)

  t1 := thread_spawn(|| {
    L i:0..1000 {
      mutex_lock(counter)
      val := mutex_get(counter)
      mutex_set(counter, val + 1)
      mutex_unlock(counter)
    }
  })

  thread_join(t1)

  final_val := mutex_get(counter)
  print("Counter: " + i64_to_str(final_val))
}
```

### 채널 (Channel)

```vais
use std/channel

fn sender(ch: Channel<i64>) {
  L i:0..10 {
    channel_send(ch, i)
  }
  channel_close(ch)
}

fn main() {
  ch := channel_new<i64>()

  thread_spawn(|| { sender(ch) })

  L {
    match channel_recv(ch) {
      Ok(val) => print(i64_to_str(val)),
      Err(_) => B  # 채널 닫힘
    }
  }

  print("All messages received")
}
```

### 여러 채널 select

```vais
use std/channel

fn main() {
  ch1 := channel_new<i64>()
  ch2 := channel_new<i64>()

  thread_spawn(|| { channel_send(ch1, 10) })
  thread_spawn(|| { channel_send(ch2, 20) })

  set := channel_set_new()
  channel_set_add(set, ch1)
  channel_set_add(set, ch2)

  match channel_select(set) {
    Ok((ch, val)) => print("Received: " + i64_to_str(val)),
    Err(_) => print("No data")
  }
}
```

---

## 추가 팁

### Self-Recursion (@)

```vais
fn factorial(n: i64) -> i64 {
  I n <= 1 {
    return 1
  }
  return n * @(n - 1)  # @ = self-recursion
}

fn main() {
  result := factorial(5)
  print(i64_to_str(result))  # "120"
}
```

### Ternary 연산자

```vais
fn main() {
  x := 10
  sign := x >= 0 ? "positive" : "negative"
  print(sign)  # "positive"
}
```

### Range 반복

```vais
fn main() {
  L i:0..10 {
    print(i64_to_str(i))
  }
  # 0부터 9까지 출력
}
```

### Pattern Matching with Guards

```vais
enum Status {
  Active(i64),
  Inactive
}

fn check_status(s: Status) -> str {
  match s {
    Active(n) I n > 100 => "High activity",
    Active(n) => "Normal activity",
    Inactive => "No activity"
  }
}

fn main() {
  status := Status::Active(150)
  msg := check_status(status)
  print(msg)  # "High activity"
}
```

---

이 Cookbook은 Vais 언어의 주요 패턴과 관용구를 다룹니다. 더 많은 예제와 심화 내용은 [Language Guide](../language/README.md)와 [Standard Library API](../api/README.md)를 참고하세요.
