# C++ Bindgen Quick Start

This guide will get you started with using C++ libraries from Vais in 5 minutes.

## Prerequisites

- Vais compiler installed
- C++ compiler (g++, clang++, or MSVC)
- Basic knowledge of C++ and Vais

## Step 1: Create a Simple C++ Library

Create a file `math.hpp`:

```cpp
// math.hpp
#ifndef MATH_HPP
#define MATH_HPP

class Calculator {
public:
    int add(int a, int b) {
        return a + b;
    }

    int multiply(int a, int b) {
        return a * b;
    }
};

double square(double x) {
    return x * x;
}

#endif
```

## Step 2: Generate Vais Bindings

Run the bindgen tool:

```bash
vaisc bindgen math.hpp -o math.vais --cpp
```

This creates `math.vais` with the bindings.

## Step 3: Create a C++ Implementation File

Create `math.cpp`:

```cpp
// math.cpp
#include "math.hpp"

// If you have additional implementation, put it here
// For this example, everything is in the header
```

## Step 4: Compile the C++ Library

### On Linux:
```bash
g++ -shared -fPIC math.cpp -o libmath.so
```

### On macOS:
```bash
clang++ -dynamiclib math.cpp -o libmath.dylib
```

### On Windows:
```cmd
cl /LD math.cpp
```

## Step 5: Use the Library in Vais

Create `main.vais`:

```vais
# Import the generated bindings
U math

F main() -> i64 {
    # Create a Calculator object
    calc := Calculator { }

    # Call methods (note: need to pass self pointer)
    result1 := calc.add(&calc, 10, 20)
    printf("10 + 20 = %d\n", result1)

    result2 := calc.multiply(&calc, 5, 7)
    printf("5 * 7 = %d\n", result2)

    # Call standalone function
    sq := square(4.5)
    printf("4.5^2 = %f\n", sq)

    0
}
```

## Step 6: Build and Run

```bash
# Build the Vais program and link with the C++ library
vaisc build main.vais -L. -lmath

# Run it
./main
```

Expected output:
```
10 + 20 = 30
5 * 7 = 35
4.5^2 = 20.250000
```

## Complete Example with Separate Implementation

Let's do a more realistic example with separate header and implementation.

### vector2.hpp
```cpp
#ifndef VECTOR2_HPP
#define VECTOR2_HPP

class Vector2 {
public:
    double x;
    double y;

    Vector2(double x, double y);
    double length() const;
    Vector2 add(const Vector2& other) const;
};

#endif
```

### vector2.cpp
```cpp
#include "vector2.hpp"
#include <cmath>

Vector2::Vector2(double x, double y) : x(x), y(y) {}

double Vector2::length() const {
    return std::sqrt(x * x + y * y);
}

Vector2 Vector2::add(const Vector2& other) const {
    return Vector2(x + other.x, y + other.y);
}
```

### Generate and build:
```bash
# Generate bindings
vaisc bindgen vector2.hpp -o vector2.vais --cpp

# Compile C++ library
g++ -shared -fPIC vector2.cpp -o libvector2.so
```

### Use in Vais (vec_main.vais):
```vais
U vector2

F main() -> i64 {
    # Create vectors
    v1 := Vector2::new(3.0, 4.0)
    v2 := Vector2::new(1.0, 2.0)

    # Call methods
    len1 := v1.length()
    printf("Length of v1: %f\n", len1)

    # Add vectors
    v3 := v1.add(&v2)
    printf("v1 + v2 = (%f, %f)\n", v3.x, v3.y)

    0
}
```

### Build and run:
```bash
vaisc build vec_main.vais -L. -lvector2
./vec_main
```

## Common Patterns

### 1. Calling Methods

C++ methods require passing `this` pointer explicitly:

```vais
# Wrong:
result := obj.method(arg)

# Correct:
result := obj.method(&obj, arg)

# Or use the impl wrapper:
impl MyClass {
    F method(self: *MyClass, arg: i32) -> i32 {
        _ZN7MyClass6methodEi(self, arg)
    }
}
# Then you can call:
result := obj.method(&obj, arg)
```

### 2. Creating Objects

Constructor calls via helper functions:

```vais
# The bindgen creates wrapper like this:
impl MyClass {
    F new(arg: i32) -> MyClass {
        obj := MyClass { /* fields */ }
        _ZN7MyClassC1Ei(&obj, arg)
        obj
    }
}

# Use it:
obj := MyClass::new(42)
```

### 3. Working with Pointers

```vais
# Pass address with &
method_taking_pointer(&obj)

# Dereference with *
value := *ptr
```

## Troubleshooting

### Problem: "cannot find -lmylib"

**Solution**: Make sure library is in current directory or add path:
```bash
vaisc build main.vais -L/path/to/lib -lmylib
```

### Problem: Undefined symbol errors

**Solution**: Check mangled names match:
```bash
nm libmylib.so | grep ClassName
```

### Problem: Segmentation fault

**Solution**:
- Make sure you're passing pointers correctly
- Check that objects are initialized
- Verify library is compatible (same compiler/ABI)

## Next Steps

- Read `CPP_SUPPORT.md` for detailed feature documentation
- See `CPP_IMPLEMENTATION_SUMMARY.md` for advanced usage
- Check `DESIGN.md` for architecture details

## Tips

1. **Start Simple**: Begin with simple structs and functions
2. **Use extern "C"**: Avoid name mangling when possible
3. **Check Bindings**: Review generated `.vais` file
4. **Test Incrementally**: Add features one at a time
5. **Handle Errors**: Use Result types for fallible operations

## Full Working Example

Here's everything in one place:

**simple.hpp**:
```cpp
class Counter {
private:
    int count;
public:
    Counter() : count(0) {}
    void increment() { count++; }
    int get() const { return count; }
};
```

**Commands**:
```bash
# Generate bindings
vaisc bindgen simple.hpp -o simple.vais --cpp

# Compile library
g++ -shared -fPIC simple.hpp -o libsimple.so

# Create Vais program
cat > test.vais << 'EOF'
U simple

F main() -> i64 {
    counter := Counter::new()
    counter.increment(&counter)
    counter.increment(&counter)
    counter.increment(&counter)
    value := counter.get(&counter)
    printf("Count: %d\n", value)
    0
}
EOF

# Build and run
vaisc build test.vais -L. -lsimple
./test
```

Expected output: `Count: 3`

## Success!

You've now successfully:
- Generated bindings from C++ headers
- Compiled a C++ library
- Called C++ code from Vais
- Linked everything together

Happy coding!
