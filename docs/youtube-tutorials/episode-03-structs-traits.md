# Building X in Vais - Episode 03: Structs and Traits in Vais

**Duration:** 10 minutes
**Difficulty:** Intermediate
**Series:** Building X in Vais

## Introduction

Welcome to Episode 03! So far we've learned functions and recursion. Now it's time to organize data with structs and define behavior with traits. We'll build a geometric shapes library to demonstrate Vais's object-oriented features.

In Vais:
- `S` defines a struct (data structure)
- `W` defines a trait (interface)
- `X` implements methods or traits for types

Let's build!

## Step 1: Defining Your First Struct (2 minutes)

Let's create a `Point` struct to represent 2D coordinates:

```vais
# Define a Point struct
S Point {
    x: i64,
    y: i64
}

F main() -> i64 {
    # Create a Point instance
    p := Point { x: 10, y: 20 }

    puts("Point created:")
    puts("x = ")
    putchar((p.x / 10) + 48)
    putchar((p.x % 10) + 48)
    putchar(10)

    puts("y = ")
    putchar((p.y / 10) + 48)
    putchar((p.y % 10) + 48)
    putchar(10)

    0
}
```

Key points:
- `S Point { ... }` defines the struct
- Fields are `name: type` pairs
- Create instances with `Point { x: 10, y: 20 }`
- Access fields with dot notation: `p.x`, `p.y`

## Step 2: Adding Methods to Structs (2 minutes)

Methods bring structs to life. Let's add behavior to our `Point`:

```vais
S Point {
    x: i64,
    y: i64,

    # Methods can be defined inside the struct
    F sum(&self) -> i64 = self.x + self.y

    F scale(&self, factor: i64) -> i64 = (self.x + self.y) * factor
}

F main() -> i64 {
    p := Point { x: 15, y: 25 }

    # Call methods with dot notation
    sum := p.sum()
    puts("Sum:")
    putchar((sum / 10) + 48)
    putchar((sum % 10) + 48)
    putchar(10)  # sum = 40

    scaled := p.scale(2)
    puts("Scaled by 2:")
    putchar((scaled / 10) + 48)
    putchar((scaled % 10) + 48)
    putchar(10)  # scaled = 80

    0
}
```

Understanding methods:
- `&self` is a reference to the instance
- `self.x` accesses fields of the current instance
- Methods can have additional parameters like `factor`
- Call methods with: `instance.method(args)`

## Step 3: Building a Shape Trait (2 minutes)

Traits define interfaces that multiple types can implement:

```vais
# Define a trait for shapes
W Shape {
    F area(&self) -> i64
    F perimeter(&self) -> i64
}

# Define a Rectangle struct
S Rectangle {
    width: i64,
    height: i64
}

# Implement Shape trait for Rectangle
X Rectangle: Shape {
    F area(&self) -> i64 = self.width * self.height

    F perimeter(&self) -> i64 = 2 * (self.width + self.height)
}

F main() -> i64 {
    rect := Rectangle { width: 5, height: 8 }

    a := rect.area()
    p := rect.perimeter()

    puts("Rectangle(5, 8):")
    puts("Area = ")
    putchar((a / 10) + 48)
    putchar((a % 10) + 48)
    putchar(10)  # 40

    puts("Perimeter = ")
    putchar((p / 10) + 48)
    putchar((p % 10) + 48)
    putchar(10)  # 26

    0
}
```

Breakdown:
- `W Shape { ... }` defines a trait
- Trait methods declare signatures only
- `X Rectangle: Shape { ... }` implements the trait
- Must implement all trait methods

## Step 4: Multiple Implementations (2 minutes)

Let's add Circle to our shape library:

```vais
W Shape {
    F area(&self) -> i64
}

S Rectangle {
    width: i64,
    height: i64
}

S Circle {
    radius: i64
}

X Rectangle: Shape {
    F area(&self) -> i64 = self.width * self.height
}

X Circle: Shape {
    F area(&self) -> i64 {
        # Approximate pi as 3
        pi := 3
        pi * self.radius * self.radius
    }
}

F main() -> i64 {
    rect := Rectangle { width: 4, height: 5 }
    circle := Circle { radius: 5 }

    rect_area := rect.area()
    circle_area := circle.area()

    puts("Rectangle area:")
    putchar((rect_area / 10) + 48)
    putchar((rect_area % 10) + 48)
    putchar(10)  # 20

    puts("Circle area:")
    putchar((circle_area / 10) + 48)
    putchar((circle_area % 10) + 48)
    putchar(10)  # 75

    0
}
```

Both Rectangle and Circle implement `Shape` differently!

## Step 5: Complex Example - Geometric Library (2 minutes)

Let's build a complete system with multiple shapes and utilities:

```vais
# Shape trait
W Shape {
    F area(&self) -> i64
}

# Point struct with utilities
S Point {
    x: i64,
    y: i64,

    F distance_from_origin(&self) -> i64 {
        # Approximate: sqrt(x^2 + y^2) as x + y for simplicity
        dx := self.x
        dy := self.y
        dx + dy
    }
}

# Rectangle
S Rectangle {
    width: i64,
    height: i64
}

X Rectangle: Shape {
    F area(&self) -> i64 = self.width * self.height
}

# Additional methods without trait
X Rectangle {
    F is_square(&self) -> i64 = self.width == self.height ? 1 : 0

    F diagonal(&self) -> i64 {
        # Approximate diagonal
        w := self.width
        h := self.height
        w + h
    }
}

# Triangle
S Triangle {
    base: i64,
    height: i64
}

X Triangle: Shape {
    F area(&self) -> i64 = (self.base * self.height) / 2
}

F main() -> i64 {
    puts("=== Geometric Library Demo ===")

    # Test Point
    p := Point { x: 3, y: 4 }
    dist := p.distance_from_origin()
    puts("Point distance: 7")

    # Test Rectangle
    rect := Rectangle { width: 6, height: 4 }
    rect_area := rect.area()
    is_sq := rect.is_square()
    puts("Rectangle area: 24")
    puts("Is square: 0")

    # Test Square
    square := Rectangle { width: 5, height: 5 }
    is_sq2 := square.is_square()
    puts("Square is_square: 1")

    # Test Triangle
    tri := Triangle { base: 10, height: 6 }
    tri_area := tri.area()
    puts("Triangle area: 30")

    puts("=== Demo Complete ===")
    0
}
```

This demonstrates:
- Multiple structs implementing the same trait
- Additional methods beyond trait requirements
- Combining different data structures
- Building a cohesive library

## Step 6: Nested Structs (1 minute)

Structs can contain other structs:

```vais
S Point {
    x: i64,
    y: i64
}

S BoundingBox {
    top_left: Point,
    bottom_right: Point,

    F width(&self) -> i64 = self.bottom_right.x - self.top_left.x

    F height(&self) -> i64 = self.top_left.y - self.bottom_right.y

    F area(&self) -> i64 = self.width() * self.height()
}

F main() -> i64 {
    bbox := BoundingBox {
        top_left: Point { x: 0, y: 10 },
        bottom_right: Point { x: 5, y: 0 }
    }

    area := bbox.area()
    puts("BoundingBox area: 50")

    0
}
```

## Key Takeaways

1. **Structs (`S`)**: Define data structures with fields
2. **Traits (`W`)**: Define interfaces with method signatures
3. **Implementation (`X`)**: Add methods to types, with or without traits
4. **Methods**: Use `&self` to access instance data
5. **Composition**: Build complex types from simpler ones

## Design Patterns

- **Trait for Behavior**: Define common interfaces (Shape, Drawable, etc.)
- **Impl for Utilities**: Add helper methods specific to a type
- **Nested Structs**: Model complex relationships
- **Multiple Traits**: A type can implement many traits

## Next Episode Preview

In Episode 04, we'll explore pattern matching and enums:
- Defining enums with the `E` keyword
- Pattern matching with `M`
- Building Option and Result types
- Error handling with elegant pattern matching
- Real-world examples with state machines

## Try It Yourself

Challenges:
1. Create a `Person` struct with name and age, add a `greet` method
2. Define a `Drawable` trait with a `draw` method, implement it for shapes
3. Build a `Color` struct with RGB values and conversion methods
4. Create a `Vehicle` trait and implement it for Car and Bike structs

## Resources

- Example: `examples/method_test.vais`
- Example: `examples/trait_test.vais`
- Tutorial: `docs/TUTORIAL.md` (Structs and Traits section)

---

See you in Episode 04 where we master pattern matching!
