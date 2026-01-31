/// Complete workflow example showing C++ binding generation
///
/// This example demonstrates:
/// 1. Parsing a C++ header
/// 2. Generating Vais FFI bindings
/// 3. Generating C wrapper header
/// 4. Showing the complete integration workflow
use vais_bindgen::Bindgen;

fn main() {
    println!("=== Complete C++ Binding Workflow ===\n");

    // Step 1: Define your C++ header
    println!("Step 1: C++ Header");
    println!("-----------------");
    let cpp_header = r#"
// geometry.hpp - C++ header
#ifndef GEOMETRY_HPP
#define GEOMETRY_HPP

namespace Geometry {
    class Point {
    public:
        Point(double x, double y);
        ~Point();

        double getX() const;
        double getY() const;
        void setX(double x);
        void setY(double y);

        double distanceTo(Point* other);

        static Point* origin();

    private:
        double x;
        double y;
    };

    class Rectangle {
    public:
        Rectangle(Point* topLeft, Point* bottomRight);
        ~Rectangle();

        double area();
        double perimeter();
        bool contains(Point* p);

    private:
        Point* topLeft;
        Point* bottomRight;
    };

    double distance(Point* p1, Point* p2);
}

#endif
    "#;
    println!("{}", cpp_header);

    // Step 2: Parse and generate bindings
    println!("\nStep 2: Generate Vais FFI Bindings");
    println!("-----------------------------------");

    let mut bindgen = Bindgen::new_cpp();
    bindgen
        .parse_header(cpp_header)
        .expect("Failed to parse C++ header");

    let vais_bindings = bindgen.generate().expect("Failed to generate bindings");
    println!("{}", vais_bindings);

    // Step 3: Generate C wrapper header
    println!("\nStep 3: Generate C Wrapper Header");
    println!("---------------------------------");

    let wrapper_header = bindgen
        .generate_cpp_wrapper_header()
        .expect("Failed to generate wrapper header");
    println!("{}", wrapper_header);

    // Step 4: Show implementation template
    println!("\nStep 4: C++ Wrapper Implementation Template");
    println!("-------------------------------------------");

    let wrapper_impl = r#"
// geometry_wrapper.cpp - C wrapper implementation
#include "geometry_wrapper.h"
#include "geometry.hpp"

extern "C" {

// Point wrappers
Geometry_PointHandle Geometry_Point_new(double x, double y) {
    return reinterpret_cast<Geometry_PointHandle>(
        new Geometry::Point(x, y)
    );
}

void Geometry_Point_delete(Geometry_PointHandle ptr) {
    delete reinterpret_cast<Geometry::Point*>(ptr);
}

double Geometry_Point_getX(Geometry_PointHandle ptr) {
    return reinterpret_cast<Geometry::Point*>(ptr)->getX();
}

double Geometry_Point_getY(Geometry_PointHandle ptr) {
    return reinterpret_cast<Geometry::Point*>(ptr)->getY();
}

void Geometry_Point_setX(Geometry_PointHandle ptr, double x) {
    reinterpret_cast<Geometry::Point*>(ptr)->setX(x);
}

void Geometry_Point_setY(Geometry_PointHandle ptr, double y) {
    reinterpret_cast<Geometry::Point*>(ptr)->setY(y);
}

double Geometry_Point_distanceTo(
    Geometry_PointHandle ptr,
    Geometry_PointHandle other
) {
    return reinterpret_cast<Geometry::Point*>(ptr)->distanceTo(
        reinterpret_cast<Geometry::Point*>(other)
    );
}

Geometry_PointHandle Geometry_Point_origin() {
    return reinterpret_cast<Geometry_PointHandle>(
        Geometry::Point::origin()
    );
}

// Rectangle wrappers
Geometry_RectangleHandle Geometry_Rectangle_new(
    Geometry_PointHandle topLeft,
    Geometry_PointHandle bottomRight
) {
    return reinterpret_cast<Geometry_RectangleHandle>(
        new Geometry::Rectangle(
            reinterpret_cast<Geometry::Point*>(topLeft),
            reinterpret_cast<Geometry::Point*>(bottomRight)
        )
    );
}

void Geometry_Rectangle_delete(Geometry_RectangleHandle ptr) {
    delete reinterpret_cast<Geometry::Rectangle*>(ptr);
}

double Geometry_Rectangle_area(Geometry_RectangleHandle ptr) {
    return reinterpret_cast<Geometry::Rectangle*>(ptr)->area();
}

double Geometry_Rectangle_perimeter(Geometry_RectangleHandle ptr) {
    return reinterpret_cast<Geometry::Rectangle*>(ptr)->perimeter();
}

bool Geometry_Rectangle_contains(
    Geometry_RectangleHandle ptr,
    Geometry_PointHandle p
) {
    return reinterpret_cast<Geometry::Rectangle*>(ptr)->contains(
        reinterpret_cast<Geometry::Point*>(p)
    );
}

// Free function wrapper
double Geometry_distance(
    Geometry_PointHandle p1,
    Geometry_PointHandle p2
) {
    return Geometry::distance(
        reinterpret_cast<Geometry::Point*>(p1),
        reinterpret_cast<Geometry::Point*>(p2)
    );
}

} // extern "C"
    "#;
    println!("{}", wrapper_impl);

    // Step 5: Show Vais usage
    println!("\nStep 5: Using the Bindings in Vais");
    println!("----------------------------------");

    let vais_usage = r#"
// example.vais - Using the C++ geometry library from Vais

// Import the generated bindings
import geometry_ffi;

// Safe Vais wrapper around C++ Point class
struct Point {
    handle: geometry_ffi::Geometry_PointHandle,
}

impl Point {
    fn new(x: f64, y: f64) -> Point {
        let handle = geometry_ffi::Geometry_Point_new(x, y);
        Point { handle }
    }

    fn origin() -> Point {
        let handle = geometry_ffi::Geometry_Point_origin();
        Point { handle }
    }

    fn get_x(self: &Point) -> f64 {
        geometry_ffi::Geometry_Point_getX(self.handle)
    }

    fn get_y(self: &Point) -> f64 {
        geometry_ffi::Geometry_Point_getY(self.handle)
    }

    fn set_x(self: &Point, x: f64) {
        geometry_ffi::Geometry_Point_setX(self.handle, x);
    }

    fn set_y(self: &Point, y: f64) {
        geometry_ffi::Geometry_Point_setY(self.handle, y);
    }

    fn distance_to(self: &Point, other: &Point) -> f64 {
        geometry_ffi::Geometry_Point_distanceTo(self.handle, other.handle)
    }
}

impl Drop for Point {
    fn drop(self: &Point) {
        geometry_ffi::Geometry_Point_delete(self.handle);
    }
}

// Example usage
fn main() {
    let p1 = Point::new(0.0, 0.0);
    let p2 = Point::new(3.0, 4.0);

    let dist = p1.distance_to(&p2);
    println("Distance: {}", dist);  // Should print 5.0

    let origin = Point::origin();
    println("Origin: ({}, {})", origin.get_x(), origin.get_y());
}
    "#;
    println!("{}", vais_usage);

    println!("\n=== Workflow Summary ===");
    println!("1. Write or receive C++ headers");
    println!("2. Use vais-bindgen to parse and generate:");
    println!("   - Vais FFI bindings (for your Vais code)");
    println!("   - C wrapper header (defines the C ABI)");
    println!("3. Implement C wrapper functions in C++");
    println!("4. Compile wrapper to a shared library");
    println!("5. Use generated Vais bindings to call C++ code");
    println!("\nNote: Create safe Vais wrappers for better ergonomics!");
}
