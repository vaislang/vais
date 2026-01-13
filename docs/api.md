# Vais API Reference

Complete reference for all built-in functions in Vais.

## Table of Contents

- [Core Functions](#core-functions)
- [Math Functions](#math-functions)
- [Collection Functions](#collection-functions)
- [String Functions](#string-functions)
- [Type Conversion](#type-conversion)
- [std.io - File I/O](#stdio---file-io)
- [std.json - JSON](#stdjson---json)
- [std.net - HTTP](#stdnet---http)

---

## Core Functions

### print(...args)
Print values to stdout (space-separated).

```vais
print("Hello", "World")    // Output: Hello World
print(1, 2, 3)             // Output: 1 2 3
```

### println(...args)
Print values to stdout with newline.

```vais
println("Line 1")
println("Line 2")
```

### len(value)
Get length of array, string, or map.

```vais
len([1, 2, 3])      // 3
len("hello")        // 5
len({a: 1, b: 2})   // 2
```

### type(value)
Get type name as string.

```vais
type(42)            // "int"
type(3.14)          // "float"
type("hello")       // "string"
type([1, 2])        // "array"
type({a: 1})        // "map"
type(true)          // "bool"
```

### range(start, end)
Create array of integers from start to end (exclusive).

```vais
range(0, 5)         // [0, 1, 2, 3, 4]
range(1, 4)         // [1, 2, 3]
```

---

## Math Functions

### abs(n)
Absolute value.

```vais
abs(-42)            // 42
abs(42)             // 42
abs(-3.14)          // 3.14
```

### sqrt(n)
Square root.

```vais
sqrt(16)            // 4.0
sqrt(2)             // 1.4142135623730951
```

### pow(base, exp)
Exponentiation.

```vais
pow(2, 10)          // 1024.0
pow(3, 3)           // 27.0
```

### sin(x), cos(x), tan(x)
Trigonometric functions (radians).

```vais
sin(0)              // 0.0
cos(0)              // 1.0
tan(0)              // 0.0
sin(3.14159 / 2)    // ~1.0
```

### log(x)
Natural logarithm (base e).

```vais
log(1)              // 0.0
log(2.71828)        // ~1.0
```

### exp(x)
Exponential (e^x).

```vais
exp(0)              // 1.0
exp(1)              // 2.718281828...
```

### floor(x)
Round down to nearest integer.

```vais
floor(3.7)          // 3.0
floor(-3.2)         // -4.0
```

### ceil(x)
Round up to nearest integer.

```vais
ceil(3.2)           // 4.0
ceil(-3.7)          // -3.0
```

### round(x)
Round to nearest integer.

```vais
round(3.5)          // 4.0
round(3.4)          // 3.0
```

### min(a, b)
Minimum of two values.

```vais
min(3, 7)           // 3
min(-5, -2)         // -5
```

### max(a, b)
Maximum of two values.

```vais
max(3, 7)           // 7
max(-5, -2)         // -2
```

---

## Collection Functions

### head(arr)
First element of array.

```vais
head([1, 2, 3])     // 1
```

### tail(arr)
All elements except first.

```vais
tail([1, 2, 3])     // [2, 3]
```

### init(arr)
All elements except last.

```vais
init([1, 2, 3])     // [1, 2]
```

### last(arr)
Last element of array.

```vais
last([1, 2, 3])     // 3
```

### reverse(arr)
Reverse array or string.

```vais
reverse([1, 2, 3])  // [3, 2, 1]
reverse("hello")    // "olleh"
```

### sort(arr)
Sort array in ascending order.

```vais
sort([3, 1, 4, 1, 5])    // [1, 1, 3, 4, 5]
sort(["c", "a", "b"])    // ["a", "b", "c"]
```

### unique(arr)
Remove duplicates from array.

```vais
unique([1, 2, 2, 3, 3, 3])   // [1, 2, 3]
```

### concat(arr1, arr2)
Concatenate two arrays.

```vais
concat([1, 2], [3, 4])      // [1, 2, 3, 4]
```

### flatten(arr)
Flatten nested array by one level.

```vais
flatten([[1, 2], [3, 4]])   // [1, 2, 3, 4]
```

### zip(arr1, arr2)
Combine two arrays into pairs.

```vais
zip([1, 2], ["a", "b"])     // [[1, "a"], [2, "b"]]
```

### enumerate(arr)
Add indices to array elements.

```vais
enumerate(["a", "b", "c"])  // [[0, "a"], [1, "b"], [2, "c"]]
```

### take(arr, n)
Take first n elements.

```vais
take([1, 2, 3, 4, 5], 3)    // [1, 2, 3]
```

### drop(arr, n)
Drop first n elements.

```vais
drop([1, 2, 3, 4, 5], 2)    // [3, 4, 5]
```

### slice(arr, start, end)
Get slice of array.

```vais
slice([1, 2, 3, 4, 5], 1, 4)   // [2, 3, 4]
```

### sum(arr)
Sum all elements.

```vais
sum([1, 2, 3, 4, 5])        // 15
```

### product(arr)
Multiply all elements.

```vais
product([1, 2, 3, 4])       // 24
```

---

## String Functions

### split(str, sep)
Split string by separator.

```vais
split("a,b,c", ",")         // ["a", "b", "c"]
split("hello world", " ")   // ["hello", "world"]
```

### join(arr, sep)
Join array elements with separator.

```vais
join(["a", "b", "c"], "-")  // "a-b-c"
join([1, 2, 3], ", ")       // "1, 2, 3"
```

### trim(str)
Remove leading/trailing whitespace.

```vais
trim("  hello  ")           // "hello"
```

### upper(str)
Convert to uppercase.

```vais
upper("hello")              // "HELLO"
```

### lower(str)
Convert to lowercase.

```vais
lower("HELLO")              // "hello"
```

### contains(str, substr)
Check if string contains substring.

```vais
contains("hello", "ell")    // true
contains("hello", "xyz")    // false
```

### replace(str, from, to)
Replace all occurrences.

```vais
replace("hello", "l", "L")  // "heLLo"
```

### starts_with(str, prefix)
Check if string starts with prefix.

```vais
starts_with("hello", "he")  // true
```

### ends_with(str, suffix)
Check if string ends with suffix.

```vais
ends_with("hello", "lo")    // true
```

### substring(str, start, end)
Get substring.

```vais
substring("hello", 1, 4)    // "ell"
```

---

## Type Conversion

### str(value)
Convert to string.

```vais
str(42)                     // "42"
str(3.14)                   // "3.14"
str(true)                   // "true"
```

### int(value)
Convert to integer.

```vais
int("42")                   // 42
int(3.7)                    // 3
int(true)                   // 1
```

### float(value)
Convert to float.

```vais
float("3.14")               // 3.14
float(42)                   // 42.0
```

---

## std.io - File I/O

### read_file(path)
Read entire file as string.

```vais
content = read_file("data.txt")
```

### write_file(path, content)
Write string to file (overwrites).

```vais
write_file("output.txt", "Hello, World!")
```

### append_file(path, content)
Append string to file.

```vais
append_file("log.txt", "New entry\n")
```

### read_lines(path)
Read file as array of lines.

```vais
lines = read_lines("data.txt")
```

### read_file_bytes(path)
Read file as byte array.

```vais
bytes = read_file_bytes("image.png")
```

### path_join(parts...)
Join path components.

```vais
path_join("dir", "subdir", "file.txt")  // "dir/subdir/file.txt"
```

### path_exists(path)
Check if path exists.

```vais
path_exists("/tmp")         // true
```

### path_is_file(path)
Check if path is a file.

```vais
path_is_file("data.txt")    // true or false
```

### path_is_dir(path)
Check if path is a directory.

```vais
path_is_dir("/tmp")         // true
```

### path_parent(path)
Get parent directory.

```vais
path_parent("/home/user/file.txt")  // "/home/user"
```

### path_filename(path)
Get filename from path.

```vais
path_filename("/home/user/file.txt")  // "file.txt"
```

### path_extension(path)
Get file extension.

```vais
path_extension("file.txt")  // "txt"
```

### list_dir(path)
List directory contents.

```vais
files = list_dir(".")       // ["file1.txt", "file2.txt", ...]
```

### create_dir(path)
Create directory.

```vais
create_dir("new_folder")
```

### create_dir_all(path)
Create directory and all parents.

```vais
create_dir_all("a/b/c")
```

### remove_file(path)
Delete file.

```vais
remove_file("temp.txt")
```

### remove_dir(path)
Delete empty directory.

```vais
remove_dir("empty_folder")
```

### copy_file(src, dst)
Copy file.

```vais
copy_file("original.txt", "backup.txt")
```

### rename(src, dst)
Rename/move file or directory.

```vais
rename("old.txt", "new.txt")
```

### cwd()
Get current working directory.

```vais
current = cwd()             // "/home/user/project"
```

### chdir(path)
Change current directory.

```vais
chdir("/tmp")
```

### env_get(key)
Get environment variable.

```vais
home = env_get("HOME")
```

### env_set(key, value)
Set environment variable.

```vais
env_set("MY_VAR", "value")
```

### readline()
Read line from stdin.

```vais
print("Enter name: ")
name = readline()
```

---

## std.json - JSON

### json_parse(str)
Parse JSON string to value.

```vais
data = json_parse('{"name": "Alice", "age": 30}')
data.name                   // "Alice"
```

### json_stringify(value)
Convert value to JSON string.

```vais
json_stringify({a: 1, b: 2})   // '{"a":1,"b":2}'
```

### json_stringify_pretty(value)
Convert to formatted JSON string.

```vais
json_stringify_pretty({a: 1, b: 2})
// {
//   "a": 1,
//   "b": 2
// }
```

### json_get(obj, path)
Get nested value by path.

```vais
data = {user: {name: "Alice"}}
json_get(data, "user.name")    // "Alice"
```

### json_set(obj, path, value)
Set nested value by path.

```vais
data = {user: {name: "Alice"}}
json_set(data, "user.age", 30)
```

### json_keys(obj)
Get all keys.

```vais
json_keys({a: 1, b: 2})        // ["a", "b"]
```

### json_values(obj)
Get all values.

```vais
json_values({a: 1, b: 2})      // [1, 2]
```

### json_has(obj, key)
Check if key exists.

```vais
json_has({a: 1}, "a")          // true
json_has({a: 1}, "b")          // false
```

### json_remove(obj, key)
Remove key from object.

```vais
json_remove({a: 1, b: 2}, "a") // {b: 2}
```

### json_merge(obj1, obj2)
Merge two objects.

```vais
json_merge({a: 1}, {b: 2})     // {a: 1, b: 2}
```

### json_type(value)
Get JSON type.

```vais
json_type(42)                  // "number"
json_type("hi")                // "string"
json_type([1,2])               // "array"
json_type({})                  // "object"
json_type(null)                // "null"
```

### json_is_null(value)
Check if value is null.

```vais
json_is_null(nil)              // true
```

### json_is_object(value)
Check if value is object.

```vais
json_is_object({a: 1})         // true
```

### json_is_array(value)
Check if value is array.

```vais
json_is_array([1, 2])          // true
```

---

## std.net - HTTP

### http_get(url)
HTTP GET request, returns response body.

```vais
body = http_get("https://api.example.com/data")
```

### http_get_json(url)
HTTP GET request, returns parsed JSON.

```vais
data = http_get_json("https://api.example.com/users")
```

### http_post(url, body)
HTTP POST request with body.

```vais
response = http_post("https://api.example.com/users", '{"name": "Alice"}')
```

### http_post_json(url, data)
HTTP POST with JSON body, returns parsed JSON.

```vais
result = http_post_json("https://api.example.com/users", {name: "Alice"})
```

### http_put(url, body)
HTTP PUT request.

```vais
http_put("https://api.example.com/users/1", '{"name": "Bob"}')
```

### http_delete(url)
HTTP DELETE request.

```vais
http_delete("https://api.example.com/users/1")
```

### http_head(url)
HTTP HEAD request, returns headers.

```vais
headers = http_head("https://example.com")
```

### http_request(method, url, headers, body)
Custom HTTP request.

```vais
response = http_request(
    "PATCH",
    "https://api.example.com/users/1",
    {"Authorization": "Bearer token"},
    '{"status": "active"}'
)
```

### url_encode(str)
URL encode string.

```vais
url_encode("hello world")      // "hello%20world"
```

### url_decode(str)
URL decode string.

```vais
url_decode("hello%20world")    // "hello world"
```
