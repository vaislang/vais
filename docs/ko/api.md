# Vais API 레퍼런스

Vais의 모든 내장 함수에 대한 완전한 레퍼런스입니다.

## 목차

- [코어 함수](#코어-함수)
- [수학 함수](#수학-함수)
- [컬렉션 함수](#컬렉션-함수)
- [문자열 함수](#문자열-함수)
- [타입 변환](#타입-변환)
- [std.io - 파일 I/O](#stdio---파일-io)
- [std.json - JSON](#stdjson---json)
- [std.net - HTTP](#stdnet---http)

---

## 코어 함수

### print(...args)

값들을 stdout으로 출력합니다 (공백으로 구분).

```vais
print("Hello", "World")    // 출력: Hello World
print(1, 2, 3)             // 출력: 1 2 3
```

### println(...args)

값들을 stdout으로 출력하고 줄바꿈합니다.

```vais
println("첫 번째 줄")
println("두 번째 줄")
```

### len(value)

배열, 문자열, 맵의 길이를 반환합니다.

```vais
len([1, 2, 3])      // 3
len("hello")        // 5
len({a: 1, b: 2})   // 2
```

### type(value)

타입 이름을 문자열로 반환합니다.

```vais
type(42)            // "int"
type(3.14)          // "float"
type("hello")       // "string"
type([1, 2])        // "array"
type({a: 1})        // "map"
type(true)          // "bool"
```

### range(start, end)

시작부터 끝까지(끝 미포함)의 정수 배열을 생성합니다.

```vais
range(0, 5)         // [0, 1, 2, 3, 4]
range(1, 4)         // [1, 2, 3]
```

---

## 수학 함수

### abs(n)

절대값을 반환합니다.

```vais
abs(-42)            // 42
abs(42)             // 42
abs(-3.14)          // 3.14
```

### sqrt(n)

제곱근을 반환합니다.

```vais
sqrt(16)            // 4.0
sqrt(2)             // 1.4142135623730951
```

### pow(base, exp)

거듭제곱을 계산합니다.

```vais
pow(2, 10)          // 1024.0
pow(3, 3)           // 27.0
```

### sin(x), cos(x), tan(x)

삼각 함수입니다 (라디안 단위).

```vais
sin(0)              // 0.0
cos(0)              // 1.0
tan(0)              // 0.0
sin(3.14159 / 2)    // ~1.0
```

### log(x)

자연 로그(밑 e)를 반환합니다.

```vais
log(1)              // 0.0
log(2.71828)        // ~1.0
```

### exp(x)

지수 함수(e^x)를 계산합니다.

```vais
exp(0)              // 1.0
exp(1)              // 2.718281828...
```

### floor(x)

가장 가까운 정수로 내림합니다.

```vais
floor(3.7)          // 3.0
floor(-3.2)         // -4.0
```

### ceil(x)

가장 가까운 정수로 올림합니다.

```vais
ceil(3.2)           // 4.0
ceil(-3.7)          // -3.0
```

### round(x)

가장 가까운 정수로 반올림합니다.

```vais
round(3.5)          // 4.0
round(3.4)          // 3.0
```

### min(a, b)

두 값 중 최소값을 반환합니다.

```vais
min(3, 7)           // 3
min(-5, -2)         // -5
```

### max(a, b)

두 값 중 최대값을 반환합니다.

```vais
max(3, 7)           // 7
max(-5, -2)         // -2
```

---

## 컬렉션 함수

### head(arr)

배열의 첫 번째 요소를 반환합니다.

```vais
head([1, 2, 3])     // 1
```

### tail(arr)

첫 번째를 제외한 모든 요소를 반환합니다.

```vais
tail([1, 2, 3])     // [2, 3]
```

### init(arr)

마지막을 제외한 모든 요소를 반환합니다.

```vais
init([1, 2, 3])     // [1, 2]
```

### last(arr)

배열의 마지막 요소를 반환합니다.

```vais
last([1, 2, 3])     // 3
```

### reverse(arr)

배열 또는 문자열을 역순으로 반환합니다.

```vais
reverse([1, 2, 3])  // [3, 2, 1]
reverse("hello")    // "olleh"
```

### sort(arr)

배열을 오름차순으로 정렬합니다.

```vais
sort([3, 1, 4, 1, 5])    // [1, 1, 3, 4, 5]
sort(["c", "a", "b"])    // ["a", "b", "c"]
```

### unique(arr)

배열에서 중복을 제거합니다.

```vais
unique([1, 2, 2, 3, 3, 3])   // [1, 2, 3]
```

### concat(arr1, arr2)

두 배열을 연결합니다.

```vais
concat([1, 2], [3, 4])      // [1, 2, 3, 4]
```

### flatten(arr)

중첩 배열을 한 레벨 평탄화합니다.

```vais
flatten([[1, 2], [3, 4]])   // [1, 2, 3, 4]
```

### zip(arr1, arr2)

두 배열을 쌍으로 결합합니다.

```vais
zip([1, 2], ["a", "b"])     // [[1, "a"], [2, "b"]]
```

### enumerate(arr)

배열 요소에 인덱스를 추가합니다.

```vais
enumerate(["a", "b", "c"])  // [[0, "a"], [1, "b"], [2, "c"]]
```

### take(arr, n)

처음 n개의 요소를 가져옵니다.

```vais
take([1, 2, 3, 4, 5], 3)    // [1, 2, 3]
```

### drop(arr, n)

처음 n개의 요소를 제외합니다.

```vais
drop([1, 2, 3, 4, 5], 2)    // [3, 4, 5]
```

### slice(arr, start, end)

배열의 일부를 가져옵니다.

```vais
slice([1, 2, 3, 4, 5], 1, 4)   // [2, 3, 4]
```

### sum(arr)

모든 요소를 더합니다.

```vais
sum([1, 2, 3, 4, 5])        // 15
```

### product(arr)

모든 요소를 곱합니다.

```vais
product([1, 2, 3, 4])       // 24
```

---

## 문자열 함수

### split(str, sep)

구분자로 문자열을 분할합니다.

```vais
split("a,b,c", ",")         // ["a", "b", "c"]
split("hello world", " ")   // ["hello", "world"]
```

### join(arr, sep)

배열 요소를 구분자로 연결합니다.

```vais
join(["a", "b", "c"], "-")  // "a-b-c"
join([1, 2, 3], ", ")       // "1, 2, 3"
```

### trim(str)

앞뒤 공백을 제거합니다.

```vais
trim("  hello  ")           // "hello"
```

### upper(str)

대문자로 변환합니다.

```vais
upper("hello")              // "HELLO"
```

### lower(str)

소문자로 변환합니다.

```vais
lower("HELLO")              // "hello"
```

### contains(str, substr)

문자열에 부분 문자열이 포함되어 있는지 확인합니다.

```vais
contains("hello", "ell")    // true
contains("hello", "xyz")    // false
```

### replace(str, from, to)

모든 발생을 대체합니다.

```vais
replace("hello", "l", "L")  // "heLLo"
```

### starts_with(str, prefix)

문자열이 접두사로 시작하는지 확인합니다.

```vais
starts_with("hello", "he")  // true
```

### ends_with(str, suffix)

문자열이 접미사로 끝나는지 확인합니다.

```vais
ends_with("hello", "lo")    // true
```

### substring(str, start, end)

부분 문자열을 가져옵니다.

```vais
substring("hello", 1, 4)    // "ell"
```

---

## 타입 변환

### str(value)

문자열로 변환합니다.

```vais
str(42)                     // "42"
str(3.14)                   // "3.14"
str(true)                   // "true"
```

### int(value)

정수로 변환합니다.

```vais
int("42")                   // 42
int(3.7)                    // 3
int(true)                   // 1
```

### float(value)

부동소수점으로 변환합니다.

```vais
float("3.14")               // 3.14
float(42)                   // 42.0
```

---

## std.io - 파일 I/O

### read_file(path)

전체 파일을 문자열로 읽습니다.

```vais
content = read_file("data.txt")
```

### write_file(path, content)

문자열을 파일에 씁니다 (덮어쓰기).

```vais
write_file("output.txt", "Hello, World!")
```

### append_file(path, content)

문자열을 파일에 추가합니다.

```vais
append_file("log.txt", "새 항목\n")
```

### read_lines(path)

파일을 줄 배열로 읽습니다.

```vais
lines = read_lines("data.txt")
```

### read_file_bytes(path)

파일을 바이트 배열로 읽습니다.

```vais
bytes = read_file_bytes("image.png")
```

### path_join(parts...)

경로 구성 요소를 연결합니다.

```vais
path_join("dir", "subdir", "file.txt")  // "dir/subdir/file.txt"
```

### path_exists(path)

경로가 존재하는지 확인합니다.

```vais
path_exists("/tmp")         // true
```

### path_is_file(path)

경로가 파일인지 확인합니다.

```vais
path_is_file("data.txt")    // true 또는 false
```

### path_is_dir(path)

경로가 디렉토리인지 확인합니다.

```vais
path_is_dir("/tmp")         // true
```

### path_parent(path)

상위 디렉토리를 가져옵니다.

```vais
path_parent("/home/user/file.txt")  // "/home/user"
```

### path_filename(path)

경로에서 파일명을 가져옵니다.

```vais
path_filename("/home/user/file.txt")  // "file.txt"
```

### path_extension(path)

파일 확장자를 가져옵니다.

```vais
path_extension("file.txt")  // "txt"
```

### list_dir(path)

디렉토리 내용을 나열합니다.

```vais
files = list_dir(".")       // ["file1.txt", "file2.txt", ...]
```

### create_dir(path)

디렉토리를 생성합니다.

```vais
create_dir("new_folder")
```

### create_dir_all(path)

디렉토리와 모든 상위 디렉토리를 생성합니다.

```vais
create_dir_all("a/b/c")
```

### remove_file(path)

파일을 삭제합니다.

```vais
remove_file("temp.txt")
```

### remove_dir(path)

빈 디렉토리를 삭제합니다.

```vais
remove_dir("empty_folder")
```

### copy_file(src, dst)

파일을 복사합니다.

```vais
copy_file("original.txt", "backup.txt")
```

### rename(src, dst)

파일 또는 디렉토리의 이름을 바꾸거나 이동합니다.

```vais
rename("old.txt", "new.txt")
```

### cwd()

현재 작업 디렉토리를 가져옵니다.

```vais
current = cwd()             // "/home/user/project"
```

### chdir(path)

현재 디렉토리를 변경합니다.

```vais
chdir("/tmp")
```

### env_get(key)

환경 변수를 가져옵니다.

```vais
home = env_get("HOME")
```

### env_set(key, value)

환경 변수를 설정합니다.

```vais
env_set("MY_VAR", "value")
```

### readline()

stdin에서 한 줄을 읽습니다.

```vais
print("이름 입력: ")
name = readline()
```

---

## std.json - JSON

### json_parse(str)

JSON 문자열을 값으로 파싱합니다.

```vais
data = json_parse('{"name": "Alice", "age": 30}')
data.name                   // "Alice"
```

### json_stringify(value)

값을 JSON 문자열로 변환합니다.

```vais
json_stringify({a: 1, b: 2})   // '{"a":1,"b":2}'
```

### json_stringify_pretty(value)

값을 포맷된 JSON 문자열로 변환합니다.

```vais
json_stringify_pretty({a: 1, b: 2})
// {
//   "a": 1,
//   "b": 2
// }
```

### json_get(obj, path)

경로로 중첩 값을 가져옵니다.

```vais
data = {user: {name: "Alice"}}
json_get(data, "user.name")    // "Alice"
```

### json_set(obj, path, value)

경로로 중첩 값을 설정합니다.

```vais
data = {user: {name: "Alice"}}
json_set(data, "user.age", 30)
```

### json_keys(obj)

모든 키를 가져옵니다.

```vais
json_keys({a: 1, b: 2})        // ["a", "b"]
```

### json_values(obj)

모든 값을 가져옵니다.

```vais
json_values({a: 1, b: 2})      // [1, 2]
```

### json_has(obj, key)

키가 존재하는지 확인합니다.

```vais
json_has({a: 1}, "a")          // true
json_has({a: 1}, "b")          // false
```

### json_remove(obj, key)

객체에서 키를 제거합니다.

```vais
json_remove({a: 1, b: 2}, "a") // {b: 2}
```

### json_merge(obj1, obj2)

두 객체를 병합합니다.

```vais
json_merge({a: 1}, {b: 2})     // {a: 1, b: 2}
```

### json_type(value)

JSON 타입을 가져옵니다.

```vais
json_type(42)                  // "number"
json_type("hi")                // "string"
json_type([1,2])               // "array"
json_type({})                  // "object"
json_type(null)                // "null"
```

### json_is_null(value)

값이 null인지 확인합니다.

```vais
json_is_null(nil)              // true
```

### json_is_object(value)

값이 객체인지 확인합니다.

```vais
json_is_object({a: 1})         // true
```

### json_is_array(value)

값이 배열인지 확인합니다.

```vais
json_is_array([1, 2])          // true
```

---

## std.net - HTTP

### http_get(url)

HTTP GET 요청, 응답 본문을 반환합니다.

```vais
body = http_get("https://api.example.com/data")
```

### http_get_json(url)

HTTP GET 요청, 파싱된 JSON을 반환합니다.

```vais
data = http_get_json("https://api.example.com/users")
```

### http_post(url, body)

본문과 함께 HTTP POST 요청.

```vais
response = http_post("https://api.example.com/users", '{"name": "Alice"}')
```

### http_post_json(url, data)

JSON 본문으로 HTTP POST, 파싱된 JSON을 반환합니다.

```vais
result = http_post_json("https://api.example.com/users", {name: "Alice"})
```

### http_put(url, body)

HTTP PUT 요청.

```vais
http_put("https://api.example.com/users/1", '{"name": "Bob"}')
```

### http_delete(url)

HTTP DELETE 요청.

```vais
http_delete("https://api.example.com/users/1")
```

### http_head(url)

HTTP HEAD 요청, 헤더를 반환합니다.

```vais
headers = http_head("https://example.com")
```

### http_request(method, url, headers, body)

커스텀 HTTP 요청.

```vais
response = http_request(
    "PATCH",
    "https://api.example.com/users/1",
    {"Authorization": "Bearer token"},
    '{"status": "active"}'
)
```

### url_encode(str)

문자열을 URL 인코딩합니다.

```vais
url_encode("hello world")      // "hello%20world"
```

### url_decode(str)

문자열을 URL 디코딩합니다.

```vais
url_decode("hello%20world")    // "hello world"
```

---

## 관련 문서

- [문법 가이드](syntax.md)
- [예제](examples.md)
- [시작 가이드](getting-started.md)
