use vais_bindgen::{Bindgen, BindgenConfig};

#[test]
fn test_basic_c_functions() {
    let header = r#"
        int add(int a, int b);
        void print_message(const char* msg);
        double calculate(double x, double y);
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    assert!(result.contains("fn add(a: i32, b: i32) -> i32;"));
    assert!(result.contains("fn print_message(msg: *const i8);"));
    assert!(result.contains("fn calculate(x: f64, y: f64) -> f64;"));
    assert!(result.contains("extern \"C\""));
}

#[test]
fn test_struct_generation() {
    let header = r#"
        typedef struct {
            int x;
            int y;
            float z;
        } Point3D;

        Point3D create_point(int x, int y, float z);
        void destroy_point(Point3D* p);
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    assert!(result.contains("struct Point3D"));
    assert!(result.contains("x: i32"));
    assert!(result.contains("y: i32"));
    assert!(result.contains("z: f32"));
    assert!(result.contains("fn create_point"));
    assert!(result.contains("fn destroy_point"));
}

#[test]
fn test_opaque_struct() {
    let header = r#"
        struct FileHandle;

        struct FileHandle* open_file(const char* path);
        void close_file(struct FileHandle* handle);
        int read_file(struct FileHandle* handle, char* buffer, int size);
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    assert!(result.contains("type FileHandle = *mut ()"));
    assert!(result.contains("fn open_file"));
    assert!(result.contains("fn close_file"));
    assert!(result.contains("fn read_file"));
}

#[test]
fn test_enum_generation() {
    let header = r#"
        enum Color {
            RED = 0,
            GREEN = 1,
            BLUE = 2
        };

        void set_color(enum Color c);
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    assert!(result.contains("enum Color"));
    assert!(result.contains("RED = 0"));
    assert!(result.contains("GREEN = 1"));
    assert!(result.contains("BLUE = 2"));
}

#[test]
fn test_pointer_types() {
    let header = r#"
        void* allocate_memory(unsigned long size);
        void free_memory(void* ptr);
        const char* get_error_message(void);
        int* get_array(int size);
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    assert!(result.contains("*mut ()"));
    assert!(result.contains("*const i8"));
    assert!(result.contains("*mut i32"));
}

#[test]
fn test_custom_library_name() {
    let header = "int test_func(void);";

    let mut config = BindgenConfig::default();
    config.set_library_name("mylib");

    let mut bindgen = Bindgen::with_config(config);
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    assert!(result.contains("extern \"mylib\""));
}

#[test]
fn test_custom_type_mappings() {
    let header = r#"
        typedef unsigned long size_t;
        size_t get_size(void);
        void set_size(size_t s);
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.configure(|config| {
        config.add_type_mapping("size_t", "u64");
    });
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    // The type mapping should apply
    assert!(result.contains("u64") || result.contains("usize"));
}

#[test]
fn test_variadic_functions() {
    let header = r#"
        int printf(const char* format, ...);
        void log_message(int level, const char* format, ...);
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    assert!(result.contains("..."));
    assert!(result.contains("fn printf"));
}

#[test]
fn test_complex_example() {
    let header = r#"
        // Graphics library example

        typedef struct {
            float x;
            float y;
        } Vec2;

        typedef struct {
            float r;
            float g;
            float b;
            float a;
        } Color;

        struct Window;

        struct Window* create_window(int width, int height, const char* title);
        void destroy_window(struct Window* window);
        void draw_rectangle(struct Window* window, Vec2 pos, Vec2 size, Color color);
        void present(struct Window* window);
        int is_window_open(struct Window* window);
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    // Check structs
    assert!(result.contains("struct Vec2"));
    assert!(result.contains("struct Color"));
    assert!(result.contains("type Window = *mut ()"));

    // Check functions
    assert!(result.contains("fn create_window"));
    assert!(result.contains("fn destroy_window"));
    assert!(result.contains("fn draw_rectangle"));
    assert!(result.contains("fn present"));
    assert!(result.contains("fn is_window_open"));
}

#[test]
fn test_unsigned_types() {
    let header = r#"
        unsigned int get_count(void);
        unsigned char get_byte(void);
        unsigned short get_word(void);
        unsigned long get_long(void);
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    assert!(result.contains("u32"));
    assert!(result.contains("u8"));
    assert!(result.contains("u16"));
    assert!(result.contains("u64"));
}

#[test]
fn test_stdint_types() {
    let header = r#"
        typedef unsigned char uint8_t;
        typedef unsigned short uint16_t;
        typedef unsigned int uint32_t;
        typedef unsigned long uint64_t;

        uint8_t get_u8(void);
        uint16_t get_u16(void);
        uint32_t get_u32(void);
        uint64_t get_u64(void);
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    assert!(result.contains("u8"));
    assert!(result.contains("u16"));
    assert!(result.contains("u32"));
    assert!(result.contains("u64"));
}

#[test]
fn test_preprocessor_directives_ignored() {
    let header = r#"
        #ifndef MY_HEADER_H
        #define MY_HEADER_H

        #include <stdio.h>
        #include <stdlib.h>

        int my_function(int x);

        #endif
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    assert!(result.contains("fn my_function"));
    assert!(!result.contains("#ifndef"));
    assert!(!result.contains("#include"));
}

#[test]
fn test_comments_ignored() {
    let header = r#"
        // This is a comment
        /* This is a
           multi-line comment */
        int test(void); // inline comment
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    assert!(result.contains("fn test"));
}

#[test]
fn test_void_pointers() {
    let header = r#"
        void* malloc(unsigned long size);
        void free(void* ptr);
        void memcpy(void* dest, const void* src, unsigned long n);
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    assert!(result.contains("*mut ()"));
    assert!(result.contains("*const ()"));
}

#[test]
fn test_function_pointers() {
    let header = r#"
        typedef int (*callback_t)(int x, int y);
        void register_callback(callback_t cb);
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    // Function pointers should be handled
    assert!(result.contains("fn register_callback"));
}

#[test]
fn test_nested_structs() {
    let header = r#"
        typedef struct {
            int x;
            int y;
        } Point;

        typedef struct {
            Point position;
            int width;
            int height;
        } Rectangle;
    "#;

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    assert!(result.contains("struct Point"));
    assert!(result.contains("struct Rectangle"));
}

#[test]
fn test_empty_input() {
    let header = "";

    let mut bindgen = Bindgen::new();
    bindgen.parse_header(header).unwrap();
    let result = bindgen.generate().unwrap();

    // Should still generate valid output with header comment and extern block
    assert!(result.contains("Auto-generated"));
    assert!(result.contains("extern"));
}
