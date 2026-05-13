# C++ Support in vais-bindgen

## Overview

The vais-bindgen tool can generate Vais bindings from C++ header files, enabling you to call C++ code from Vais programs.

## Quick Start

See `CPP_QUICK_START.md` for a step-by-step tutorial.

## Features

### Supported C++ Features

✅ **Classes and Structs**
- Public member variables
- Member functions
- Constructors and destructors
- Static methods
- Const methods

✅ **Functions**
- Global functions
- Function overloading
- Default parameters (limited)

✅ **Types**
- Fundamental types (int, float, double, bool, etc.)
- Pointers and references
- Const qualifiers
- Enums and enum classes

✅ **Namespaces**
- Namespace declarations
- Nested namespaces
- Using declarations

✅ **Templates**
- Simple template classes (with manual instantiation)
- Template functions (basic support)

✅ **Operators**
- Operator overloading
- Conversion operators

### Not Supported

❌ **Complex Templates**
- Variadic templates
- Template specialization
- Template metaprogramming

❌ **Advanced OOP**
- Multiple inheritance
- Virtual functions and vtables
- RTTI (runtime type information)

❌ **Modern C++ Features**
- Move semantics
- Smart pointers (std::unique_ptr, std::shared_ptr)
- Concepts (C++20)
- Ranges (C++20)

❌ **Exception Handling**
- C++ exceptions (use error codes instead)

## Usage

### Basic Command

```bash
vaisc bindgen header.hpp -o bindings.vais --cpp
```

### Options

```bash
vaisc bindgen header.hpp \
    --cpp \                          # Enable C++ mode
    -o bindings.vais \               # Output file
    -I /path/to/includes \           # Include directories
    --std c++17 \                    # C++ standard version
    --namespace MyNamespace \        # Filter by namespace
    --instantiate "vector<int>" \    # Template instantiation
    --no-mangle                      # Use original names (requires extern "C")
```

## Type Mappings

### Fundamental Types

| C++ Type | Vais Type |
|----------|-----------|
| char, int8_t | i8 |
| short, int16_t | i16 |
| int, int32_t | i32 |
| long, long long, int64_t | i64 |
| unsigned char, uint8_t | u8 |
| unsigned short, uint16_t | u16 |
| unsigned int, uint32_t | u32 |
| unsigned long, uint64_t | u64 |
| float | f32 |
| double | f64 |
| bool | bool |
| void | void |

### Compound Types

| C++ Type | Vais Type | Notes |
|----------|-----------|-------|
| T* | *T | Pointer |
| T& | *T | Reference (as pointer) |
| const T | T | Const qualification ignored |
| T[N] | *T | Array (as pointer) |

### Standard Library Types

| C++ Type | Vais Type | Notes |
|----------|-----------|-------|
| std::string | String | UTF-8 string |
| `std::vector<T>` | `Vec<T>` | Dynamic array |
| `std::optional<T>` | `Optional<T>` | Optional value |
| `std::pair<A, B>` | (A, B) | Tuple |

## Examples

### Simple Class

**Input** (point.hpp):
```cpp
class Point {
public:
    double x;
    double y;

    Point(double x, double y);
    double distance(const Point& other) const;
};
```

**Generate Bindings**:
```bash
vaisc bindgen point.hpp -o point.vais --cpp
```

**Generated** (point.vais):
```vais
S Point {
    x: f64,
    y: f64,
}

extern F _ZN5PointC1Edd(self: *Point, x: f64, y: f64) -> void
extern F _ZNK5Point8distanceERKS_(self: *Point, other: *Point) -> f64

impl Point {
    F new(x: f64, y: f64) -> Point {
        p := Point { x: 0.0, y: 0.0 }
        _ZN5PointC1Edd(&p, x, y)
        p
    }

    F distance(self: *Point, other: *Point) -> f64 {
        _ZNK5Point8distanceERKS_(self, other)
    }
}
```

**Use in Vais**:
```vais
U point

F main() -> i64 {
    p1 := Point::new(0.0, 0.0)
    p2 := Point::new(3.0, 4.0)
    dist := p1.distance(&p2)
    printf("Distance: %f\n", dist)
    0
}
```

### Class with Methods

**Input** (calculator.hpp):
```cpp
class Calculator {
private:
    int result;

public:
    Calculator() : result(0) {}
    void add(int x) { result += x; }
    void subtract(int x) { result -= x; }
    int get_result() const { return result; }
};
```

**Generated Bindings**:
```vais
S Calculator {
    result: i32,  # Note: private members still visible
}

extern F _ZN10CalculatorC1Ev(self: *Calculator) -> void
extern F _ZN10Calculator3addEi(self: *Calculator, x: i32) -> void
extern F _ZN10Calculator8subtractEi(self: *Calculator, x: i32) -> void
extern F _ZNK10Calculator10get_resultEv(self: *Calculator) -> i32

impl Calculator {
    F new() -> Calculator {
        c := Calculator { result: 0 }
        _ZN10CalculatorC1Ev(&c)
        c
    }

    F add(self: *Calculator, x: i32) -> void {
        _ZN10Calculator3addEi(self, x)
    }

    F subtract(self: *Calculator, x: i32) -> void {
        _ZN10Calculator8subtractEi(self, x)
    }

    F get_result(self: *Calculator) -> i32 {
        _ZNK10Calculator10get_resultEv(self)
    }
}
```

### Namespaces

**Input** (math.hpp):
```cpp
namespace math {
    double pi = 3.14159;

    double square(double x) {
        return x * x;
    }

    class Circle {
    public:
        double radius;
        double area() const;
    };
}
```

**Generated Bindings**:
```vais
# Namespace: math

extern math_pi: f64

extern F _ZN4math6squareEd(x: f64) -> f64

F square(x: f64) -> f64 {
    _ZN4math6squareEd(x)
}

S Circle {
    radius: f64,
}

extern F _ZNK4math6Circle4areaEv(self: *Circle) -> f64

impl Circle {
    F area(self: *Circle) -> f64 {
        _ZNK4math6Circle4areaEv(self)
    }
}
```

### Template Instantiation

**Input** (container.hpp):
```cpp
template<typename T>
class Container {
public:
    T* data;
    size_t size;

    T get(size_t index) const {
        return data[index];
    }
};
```

**Generate for specific type**:
```bash
vaisc bindgen container.hpp --cpp --instantiate "Container<int>" -o container_int.vais
```

**Generated**:
```vais
S ContainerInt {
    data: *i32,
    size: u64,
}

extern F _ZNK9ContainerIiE3getEm(self: *ContainerInt, index: u64) -> i32

impl ContainerInt {
    F get(self: *ContainerInt, index: u64) -> i32 {
        _ZNK9ContainerIiE3getEm(self, index)
    }
}
```

## Building and Linking

### Compile C++ Library

```bash
# Compile C++ implementation
g++ -std=c++17 -shared -fPIC mylib.cpp -o libmylib.so

# Or on macOS
clang++ -std=c++17 -dynamiclib mylib.cpp -o libmylib.dylib

# Or on Windows
cl /LD /std:c++17 mylib.cpp
```

### Link with Vais

```bash
vaisc build main.vais -L. -lmylib
```

### Complete Example

```bash
# 1. Create C++ header and implementation
cat > vector2.hpp << 'EOF'
class Vector2 {
public:
    double x, y;
    Vector2(double x, double y) : x(x), y(y) {}
    double length() const;
};
EOF

cat > vector2.cpp << 'EOF'
#include "vector2.hpp"
#include <cmath>

double Vector2::length() const {
    return std::sqrt(x*x + y*y);
}
EOF

# 2. Generate bindings
vaisc bindgen vector2.hpp -o vector2.vais --cpp

# 3. Compile C++ library
g++ -shared -fPIC vector2.cpp -o libvector2.so

# 4. Use in Vais
cat > main.vais << 'EOF'
U vector2

F main() -> i64 {
    v := Vector2::new(3.0, 4.0)
    len := v.length()
    printf("Length: %f\n", len)
    0
}
EOF

# 5. Build and run
vaisc build main.vais -L. -lvector2
./main
```

## Name Mangling

### Understanding Mangled Names

C++ uses name mangling to support function overloading:

```cpp
void foo(int x);        // _Z3fooi
void foo(double x);     // _Z3food
void foo(int x, int y); // _Z3fooii
```

The bindgen tool automatically generates these mangled names.

### Viewing Mangled Names

```bash
# View mangled names in library
nm libmylib.so | grep foo

# Demangle names
nm libmylib.so | c++filt
```

### Avoiding Name Mangling

Use `extern "C"` in C++ to avoid mangling:

```cpp
extern "C" {
    void simple_function(int x);
}
```

Then use `--no-mangle` flag:
```bash
vaisc bindgen header.h --no-mangle
```

## Best Practices

### 1. Keep Interfaces Simple

Prefer simple C-like interfaces:

```cpp
// Good
extern "C" {
    void* create_object();
    void destroy_object(void* obj);
    int process(void* obj, int x);
}

// Harder to bind
template<typename T, typename U>
class Complex {
    std::unique_ptr<T> data;
    virtual U process(const T&) = 0;
};
```

### 2. Provide Factory Functions

```cpp
class MyClass {
private:
    MyClass(int x);  // Private constructor

public:
    static MyClass* create(int x) {
        return new MyClass(x);
    }
};
```

### 3. Use Value Types When Possible

```cpp
// Good: simple struct
struct Point {
    double x, y;
};

// Harder: requires memory management
class PointManager {
    std::vector<Point> points;
};
```

### 4. Document Ownership

```cpp
// Returns owned pointer - caller must delete
MyClass* create_object();

// Borrows pointer - do not delete
void process(const MyClass* obj);
```

## Troubleshooting

### Cannot Find Header

```bash
vaisc bindgen myheader.hpp -I /usr/include -I /usr/local/include --cpp
```

### Unsupported Feature

If bindgen fails on complex C++:
1. Simplify the interface
2. Use `extern "C"` wrapper
3. Manually write bindings

### Linker Errors

```bash
# Check that library exists
ls libmylib.so

# Check mangled names match
nm libmylib.so | grep function_name
```

## Advanced Topics

### Custom Type Conversions

For types that don't map directly, create wrapper functions:

```cpp
// C++
std::string get_name();

// Wrapper
extern "C" const char* get_name_wrapper() {
    static std::string result = get_name();
    return result.c_str();
}
```

### Exception Handling

Convert exceptions to error codes:

```cpp
// C++
void may_throw();

// Wrapper
extern "C" int may_throw_wrapper() {
    try {
        may_throw();
        return 0;  // Success
    } catch (...) {
        return -1;  // Error
    }
}
```

## Further Reading

- **Implementation**: `CPP_IMPLEMENTATION_SUMMARY.md`
- **Quick Start**: `CPP_QUICK_START.md`
- **General Bindgen**: `README.md`
- **Design**: `DESIGN.md`

## Getting Help

For issues with C++ bindings:
1. Check that C++ code compiles standalone
2. Try simpler version of the interface
3. Use `extern "C"` when possible
4. Consult implementation summary for limitations
