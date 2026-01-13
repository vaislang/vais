# AOEL API Reference

Complete reference for all built-in functions in AOEL.

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

```aoel
print("Hello", "World")    // Output: Hello World
print(1, 2, 3)             // Output: 1 2 3
```

### println(...args)
Print values to stdout with newline.

```aoel
println("Line 1")
println("Line 2")
```

### len(value)
Get length of array, string, or map.

```aoel
len([1, 2, 3])      // 3
len("hello")        // 5
len({a: 1, b: 2})   // 2
```

### type(value)
Get type name as string.

```aoel
type(42)            // "int"
type(3.14)          // "float"
type("hello")       // "string"
type([1, 2])        // "array"
type({a: 1})        // "map"
type(true)          // "bool"
```

### range(start, end)
Create array of integers from start to end (exclusive).

```aoel
range(0, 5)         // [0, 1, 2, 3, 4]
range(1, 4)         // [1, 2, 3]
```

---

## Math Functions

### abs(n)
Absolute value.

```aoel
abs(-42)            // 42
abs(42)             // 42
abs(-3.14)          // 3.14
```

### sqrt(n)
Square root.

```aoel
sqrt(16)            // 4.0
sqrt(2)             // 1.4142135623730951
```

### pow(base, exp)
Exponentiation.

```aoel
pow(2, 10)          // 1024.0
pow(3, 3)           // 27.0
```

### sin(x), cos(x), tan(x)
Trigonometric functions (radians).

```aoel
sin(0)              // 0.0
cos(0)              // 1.0
tan(0)              // 0.0
sin(3.14159 / 2)    // ~1.0
```

### log(x)
Natural logarithm (base e).

```aoel
log(1)              // 0.0
log(2.71828)        // ~1.0
```

### exp(x)
Exponential (e^x).

```aoel
exp(0)              // 1.0
exp(1)              // 2.718281828...
```

### floor(x)
Round down to nearest integer.

```aoel
floor(3.7)          // 3.0
floor(-3.2)         // -4.0
```

### ceil(x)
Round up to nearest integer.

```aoel
ceil(3.2)           // 4.0
ceil(-3.7)          // -3.0
```

### round(x)
Round to nearest integer.

```aoel
round(3.5)          // 4.0
round(3.4)          // 3.0
```

### min(a, b)
Minimum of two values.

```aoel
min(3, 7)           // 3
min(-5, -2)         // -5
```

### max(a, b)
Maximum of two values.

```aoel
max(3, 7)           // 7
max(-5, -2)         // -2
```

---

## Collection Functions

### head(arr)
First element of array.

```aoel
head([1, 2, 3])     // 1
```

### tail(arr)
All elements except first.

```aoel
tail([1, 2, 3])     // [2, 3]
```

### init(arr)
All elements except last.

```aoel
init([1, 2, 3])     // [1, 2]
```

### last(arr)
Last element of array.

```aoel
last([1, 2, 3])     // 3
```

### reverse(arr)
Reverse array or string.

```aoel
reverse([1, 2, 3])  // [3, 2, 1]
reverse("hello")    // "olleh"
```

### sort(arr)
Sort array in ascending order.

```aoel
sort([3, 1, 4, 1, 5])    // [1, 1, 3, 4, 5]
sort(["c", "a", "b"])    // ["a", "b", "c"]
```

### unique(arr)
Remove duplicates from array.

```aoel
unique([1, 2, 2, 3, 3, 3])   // [1, 2, 3]
```

### concat(arr1, arr2)
Concatenate two arrays.

```aoel
concat([1, 2], [3, 4])      // [1, 2, 3, 4]
```

### flatten(arr)
Flatten nested array by one level.

```aoel
flatten([[1, 2], [3, 4]])   // [1, 2, 3, 4]
```

### zip(arr1, arr2)
Combine two arrays into pairs.

```aoel
zip([1, 2], ["a", "b"])     // [[1, "a"], [2, "b"]]
```

### enumerate(arr)
Add indices to array elements.

```aoel
enumerate(["a", "b", "c"])  // [[0, "a"], [1, "b"], [2, "c"]]
```

### take(arr, n)
Take first n elements.

```aoel
take([1, 2, 3, 4, 5], 3)    // [1, 2, 3]
```

### drop(arr, n)
Drop first n elements.

```aoel
drop([1, 2, 3, 4, 5], 2)    // [3, 4, 5]
```

### slice(arr, start, end)
Get slice of array.

```aoel
slice([1, 2, 3, 4, 5], 1, 4)   // [2, 3, 4]
```

### sum(arr)
Sum all elements.

```aoel
sum([1, 2, 3, 4, 5])        // 15
```

### product(arr)
Multiply all elements.

```aoel
product([1, 2, 3, 4])       // 24
```

---

## String Functions

### split(str, sep)
Split string by separator.

```aoel
split("a,b,c", ",")         // ["a", "b", "c"]
split("hello world", " ")   // ["hello", "world"]
```

### join(arr, sep)
Join array elements with separator.

```aoel
join(["a", "b", "c"], "-")  // "a-b-c"
join([1, 2, 3], ", ")       // "1, 2, 3"
```

### trim(str)
Remove leading/trailing whitespace.

```aoel
trim("  hello  ")           // "hello"
```

### upper(str)
Convert to uppercase.

```aoel
upper("hello")              // "HELLO"
```

### lower(str)
Convert to lowercase.

```aoel
lower("HELLO")              // "hello"
```

### contains(str, substr)
Check if string contains substring.

```aoel
contains("hello", "ell")    // true
contains("hello", "xyz")    // false
```

### replace(str, from, to)
Replace all occurrences.

```aoel
replace("hello", "l", "L")  // "heLLo"
```

### starts_with(str, prefix)
Check if string starts with prefix.

```aoel
starts_with("hello", "he")  // true
```

### ends_with(str, suffix)
Check if string ends with suffix.

```aoel
ends_with("hello", "lo")    // true
```

### substring(str, start, end)
Get substring.

```aoel
substring("hello", 1, 4)    // "ell"
```

---

## Type Conversion

### str(value)
Convert to string.

```aoel
str(42)                     // "42"
str(3.14)                   // "3.14"
str(true)                   // "true"
```

### int(value)
Convert to integer.

```aoel
int("42")                   // 42
int(3.7)                    // 3
int(true)                   // 1
```

### float(value)
Convert to float.

```aoel
float("3.14")               // 3.14
float(42)                   // 42.0
```

---

## std.io - File I/O

### read_file(path)
Read entire file as string.

```aoel
content = read_file("data.txt")
```

### write_file(path, content)
Write string to file (overwrites).

```aoel
write_file("output.txt", "Hello, World!")
```

### append_file(path, content)
Append string to file.

```aoel
append_file("log.txt", "New entry\n")
```

### read_lines(path)
Read file as array of lines.

```aoel
lines = read_lines("data.txt")
```

### read_file_bytes(path)
Read file as byte array.

```aoel
bytes = read_file_bytes("image.png")
```

### path_join(parts...)
Join path components.

```aoel
path_join("dir", "subdir", "file.txt")  // "dir/subdir/file.txt"
```

### path_exists(path)
Check if path exists.

```aoel
path_exists("/tmp")         // true
```

### path_is_file(path)
Check if path is a file.

```aoel
path_is_file("data.txt")    // true or false
```

### path_is_dir(path)
Check if path is a directory.

```aoel
path_is_dir("/tmp")         // true
```

### path_parent(path)
Get parent directory.

```aoel
path_parent("/home/user/file.txt")  // "/home/user"
```

### path_filename(path)
Get filename from path.

```aoel
path_filename("/home/user/file.txt")  // "file.txt"
```

### path_extension(path)
Get file extension.

```aoel
path_extension("file.txt")  // "txt"
```

### list_dir(path)
List directory contents.

```aoel
files = list_dir(".")       // ["file1.txt", "file2.txt", ...]
```

### create_dir(path)
Create directory.

```aoel
create_dir("new_folder")
```

### create_dir_all(path)
Create directory and all parents.

```aoel
create_dir_all("a/b/c")
```

### remove_file(path)
Delete file.

```aoel
remove_file("temp.txt")
```

### remove_dir(path)
Delete empty directory.

```aoel
remove_dir("empty_folder")
```

### copy_file(src, dst)
Copy file.

```aoel
copy_file("original.txt", "backup.txt")
```

### rename(src, dst)
Rename/move file or directory.

```aoel
rename("old.txt", "new.txt")
```

### cwd()
Get current working directory.

```aoel
current = cwd()             // "/home/user/project"
```

### chdir(path)
Change current directory.

```aoel
chdir("/tmp")
```

### env_get(key)
Get environment variable.

```aoel
home = env_get("HOME")
```

### env_set(key, value)
Set environment variable.

```aoel
env_set("MY_VAR", "value")
```

### readline()
Read line from stdin.

```aoel
print("Enter name: ")
name = readline()
```

---

## std.json - JSON

### json_parse(str)
Parse JSON string to value.

```aoel
data = json_parse('{"name": "Alice", "age": 30}')
data.name                   // "Alice"
```

### json_stringify(value)
Convert value to JSON string.

```aoel
json_stringify({a: 1, b: 2})   // '{"a":1,"b":2}'
```

### json_stringify_pretty(value)
Convert to formatted JSON string.

```aoel
json_stringify_pretty({a: 1, b: 2})
// {
//   "a": 1,
//   "b": 2
// }
```

### json_get(obj, path)
Get nested value by path.

```aoel
data = {user: {name: "Alice"}}
json_get(data, "user.name")    // "Alice"
```

### json_set(obj, path, value)
Set nested value by path.

```aoel
data = {user: {name: "Alice"}}
json_set(data, "user.age", 30)
```

### json_keys(obj)
Get all keys.

```aoel
json_keys({a: 1, b: 2})        // ["a", "b"]
```

### json_values(obj)
Get all values.

```aoel
json_values({a: 1, b: 2})      // [1, 2]
```

### json_has(obj, key)
Check if key exists.

```aoel
json_has({a: 1}, "a")          // true
json_has({a: 1}, "b")          // false
```

### json_remove(obj, key)
Remove key from object.

```aoel
json_remove({a: 1, b: 2}, "a") // {b: 2}
```

### json_merge(obj1, obj2)
Merge two objects.

```aoel
json_merge({a: 1}, {b: 2})     // {a: 1, b: 2}
```

### json_type(value)
Get JSON type.

```aoel
json_type(42)                  // "number"
json_type("hi")                // "string"
json_type([1,2])               // "array"
json_type({})                  // "object"
json_type(null)                // "null"
```

### json_is_null(value)
Check if value is null.

```aoel
json_is_null(nil)              // true
```

### json_is_object(value)
Check if value is object.

```aoel
json_is_object({a: 1})         // true
```

### json_is_array(value)
Check if value is array.

```aoel
json_is_array([1, 2])          // true
```

---

## std.net - HTTP

### http_get(url)
HTTP GET request, returns response body.

```aoel
body = http_get("https://api.example.com/data")
```

### http_get_json(url)
HTTP GET request, returns parsed JSON.

```aoel
data = http_get_json("https://api.example.com/users")
```

### http_post(url, body)
HTTP POST request with body.

```aoel
response = http_post("https://api.example.com/users", '{"name": "Alice"}')
```

### http_post_json(url, data)
HTTP POST with JSON body, returns parsed JSON.

```aoel
result = http_post_json("https://api.example.com/users", {name: "Alice"})
```

### http_put(url, body)
HTTP PUT request.

```aoel
http_put("https://api.example.com/users/1", '{"name": "Bob"}')
```

### http_delete(url)
HTTP DELETE request.

```aoel
http_delete("https://api.example.com/users/1")
```

### http_head(url)
HTTP HEAD request, returns headers.

```aoel
headers = http_head("https://example.com")
```

### http_request(method, url, headers, body)
Custom HTTP request.

```aoel
response = http_request(
    "PATCH",
    "https://api.example.com/users/1",
    {"Authorization": "Bearer token"},
    '{"status": "active"}'
)
```

### url_encode(str)
URL encode string.

```aoel
url_encode("hello world")      // "hello%20world"
```

### url_decode(str)
URL decode string.

```aoel
url_decode("hello%20world")    // "hello world"
```
