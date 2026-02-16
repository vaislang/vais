# Vais Cookbook — 실전 레시피 모음

이 페이지는 Vais 언어로 자주 사용되는 패턴과 작업을 빠르게 해결할 수 있는 실전 코드 레시피를 제공합니다. 각 레시피는 바로 실행 가능한 완성형 코드입니다.

---

## 기본 (Basics)

### Hello World

```vais
F main() {
  print("Hello, Vais!")
}
```

### 명령줄 인수 처리

```vais
U std/args

F main() {
  args := get_args()
  I args.len() > 1 {
    print("First argument: ")
    print(args.get(1))
  } E {
    print("No arguments provided")
  }
}
```

### 환경변수 읽기

```vais
U std/env

F main() {
  path := env_get("PATH")
  M path {
    Some(p) => print(p),
    None => print("PATH not set")
  }
}
```

---

## 문자열 (Strings)

### 문자열 결합

```vais
F main() {
  first := "Hello"
  second := "World"
  result := first + " " + second
  print(result)  # "Hello World"
}
```

### 문자열 분할

```vais
U std/string

F main() {
  text := "apple,banana,cherry"
  parts := str_split(text, ",")
  I parts.len() > 0 {
    print(parts.get(0))  # "apple"
  }
}
```

### 문자열 포맷팅

```vais
F main() {
  name := "Alice"
  age := 30
  msg := "Name: " + name + ", Age: " + i64_to_str(age)
  print(msg)
}
```

### 문자열 트림 및 대소문자 변환

```vais
U std/string

F main() {
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
U std/collections

F main() {
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
U std/collections

F main() {
  map := HashMap::new<str, i64>()
  map.insert("apple", 5)
  map.insert("banana", 10)

  M map.get("apple") {
    Some(val) => print(i64_to_str(val)),  # "5"
    None => print("Not found")
  }
}
```

### 정렬

```vais
U std/collections

F main() {
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
U std/collections

F is_even(x: i64) -> bool {
  x % 2 == 0
}

F double(x: i64) -> i64 {
  x * 2
}

F main() {
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
U std/io

F main() {
  content := file_read("input.txt")
  M content {
    Ok(data) => print(data),
    Err(e) => print("Error reading file")
  }
}
```

### 파일 쓰기

```vais
U std/io

F main() {
  result := file_write("output.txt", "Hello, file!")
  M result {
    Ok(_) => print("Write successful"),
    Err(e) => print("Write failed")
  }
}
```

### 디렉토리 탐색

```vais
U std/io

F main() {
  entries := dir_read(".")
  M entries {
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
U std/string

F main() {
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
U std/io

F process_file(path: str) -> Result<i64, str> {
  content := file_read(path)?
  len := str_len(content)
  Ok(len)
}

F main() {
  M process_file("data.txt") {
    Ok(len) => print("File length: " + i64_to_str(len)),
    Err(e) => print("Error: " + e)
  }
}
```

### ? 연산자 사용

```vais
U std/io

F read_and_parse(path: str) -> Result<i64, str> {
  content := file_read(path)?
  num := str_to_i64(content)?
  Ok(num * 2)
}

F main() {
  result := read_and_parse("number.txt")
  M result {
    Ok(n) => print(i64_to_str(n)),
    Err(e) => print("Failed: " + e)
  }
}
```

### 커스텀 에러 타입

```vais
E MyError {
  NotFound(str),
  InvalidInput(str),
  IoError(str)
}

F find_user(id: i64) -> Result<str, MyError> {
  I id < 0 {
    R Err(MyError::InvalidInput("ID must be positive"))
  }
  I id > 1000 {
    R Err(MyError::NotFound("User not found"))
  }
  Ok("User_" + i64_to_str(id))
}

F main() {
  M find_user(1500) {
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
A F fetch_data() -> str {
  # 비동기 작업 시뮬레이션
  R "Data fetched"
}

A F main() {
  result := fetch_data().Y
  print(result)
}
```

### HTTP 요청

```vais
U std/http

A F fetch_url(url: str) -> Result<str, str> {
  response := http_get(url).Y?
  Ok(response.body)
}

A F main() {
  M fetch_url("https://api.example.com/data").Y {
    Ok(body) => print(body),
    Err(e) => print("HTTP error: " + e)
  }
}
```

### 동시 작업

```vais
U std/async

A F task1() -> i64 {
  R 42
}

A F task2() -> i64 {
  R 100
}

A F main() {
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
extern F strlen(s: i64) -> i64

F main() {
  text := "Hello"
  ptr := str_to_ptr(text)
  length := strlen(ptr)
  print("Length: " + i64_to_str(length))  # "Length: 5"
}
```

### 공유 라이브러리 사용

```vais
extern F my_c_function(x: i64) -> i64

F main() {
  result := my_c_function(10)
  print(i64_to_str(result))
}
```

---

## JSON

### JSON 파싱

```vais
U std/json

F main() {
  json_str := "{\"name\":\"Alice\",\"age\":30}"
  parsed := json_parse(json_str)

  M parsed {
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
U std/json

F main() {
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
U std/json

F main() {
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
U std/net

F handle_client(conn: TcpStream) {
  msg := tcp_read(conn)
  M msg {
    Ok(data) => {
      response := "Echo: " + data
      tcp_write(conn, response)
    },
    Err(e) => print("Read error")
  }
  tcp_close(conn)
}

F main() {
  listener := tcp_listen("127.0.0.1:8080")
  M listener {
    Ok(l) => {
      print("Server listening on :8080")
      L {
        M tcp_accept(l) {
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
U std/http

F handler(req: HttpRequest) -> HttpResponse {
  I req.path == "/" {
    R http_response(200, "Hello, HTTP!")
  } E {
    R http_response(404, "Not Found")
  }
}

F main() {
  server := http_server_new("127.0.0.1:8000", handler)
  M server {
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
U std/websocket

F on_message(conn: WsConnection, msg: str) {
  print("Received: " + msg)
  ws_send(conn, "Echo: " + msg)
}

F main() {
  server := ws_server_new("127.0.0.1:9000", on_message)
  M server {
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
U std/thread

F worker(id: i64) {
  print("Worker " + i64_to_str(id) + " started")
}

F main() {
  t1 := thread_spawn(|| { worker(1) })
  t2 := thread_spawn(|| { worker(2) })

  thread_join(t1)
  thread_join(t2)

  print("All workers done")
}
```

### 뮤텍스 (Mutex)

```vais
U std/sync

F main() {
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
U std/channel

F sender(ch: Channel<i64>) {
  L i:0..10 {
    channel_send(ch, i)
  }
  channel_close(ch)
}

F main() {
  ch := channel_new<i64>()

  thread_spawn(|| { sender(ch) })

  L {
    M channel_recv(ch) {
      Ok(val) => print(i64_to_str(val)),
      Err(_) => B  # 채널 닫힘
    }
  }

  print("All messages received")
}
```

### 여러 채널 select

```vais
U std/channel

F main() {
  ch1 := channel_new<i64>()
  ch2 := channel_new<i64>()

  thread_spawn(|| { channel_send(ch1, 10) })
  thread_spawn(|| { channel_send(ch2, 20) })

  set := channel_set_new()
  channel_set_add(set, ch1)
  channel_set_add(set, ch2)

  M channel_select(set) {
    Ok((ch, val)) => print("Received: " + i64_to_str(val)),
    Err(_) => print("No data")
  }
}
```

---

## 추가 팁

### Self-Recursion (@)

```vais
F factorial(n: i64) -> i64 {
  I n <= 1 {
    R 1
  }
  R n * @(n - 1)  # @ = self-recursion
}

F main() {
  result := factorial(5)
  print(i64_to_str(result))  # "120"
}
```

### Ternary 연산자

```vais
F main() {
  x := 10
  sign := x >= 0 ? "positive" : "negative"
  print(sign)  # "positive"
}
```

### Range 반복

```vais
F main() {
  L i:0..10 {
    print(i64_to_str(i))
  }
  # 0부터 9까지 출력
}
```

### Pattern Matching with Guards

```vais
E Status {
  Active(i64),
  Inactive
}

F check_status(s: Status) -> str {
  M s {
    Active(n) I n > 100 => "High activity",
    Active(n) => "Normal activity",
    Inactive => "No activity"
  }
}

F main() {
  status := Status::Active(150)
  msg := check_status(status)
  print(msg)  # "High activity"
}
```

---

이 Cookbook은 Vais 언어의 주요 패턴과 관용구를 다룹니다. 더 많은 예제와 심화 내용은 [Language Guide](../language/README.md)와 [Standard Library API](../api/README.md)를 참고하세요.
