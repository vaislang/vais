use vais_bindgen::{Bindgen, BindgenConfig};

fn main() {
    let header = r#"
        // Advanced FFI example with custom types

        typedef unsigned char uint8_t;
        typedef unsigned short uint16_t;
        typedef unsigned int uint32_t;
        typedef unsigned long uint64_t;
        typedef long ssize_t;

        typedef struct {
            uint8_t* data;
            uint64_t length;
            uint64_t capacity;
        } Buffer;

        enum Status {
            OK = 0,
            ERROR_INVALID_ARGUMENT = 1,
            ERROR_OUT_OF_MEMORY = 2,
            ERROR_IO = 3
        };

        struct Context;

        // Context management
        struct Context* create_context(void);
        void destroy_context(struct Context* ctx);

        // Buffer operations
        Buffer create_buffer(uint64_t capacity);
        void destroy_buffer(Buffer* buf);
        enum Status buffer_append(Buffer* buf, const uint8_t* data, uint64_t len);
        enum Status buffer_resize(Buffer* buf, uint64_t new_capacity);

        // I/O operations
        ssize_t read_file(const char* path, Buffer* buf);
        ssize_t write_file(const char* path, const Buffer* buf);

        // String operations
        char* string_duplicate(const char* str);
        void string_free(char* str);
        int string_compare(const char* a, const char* b);

        // Callback support
        typedef void (*event_callback_t)(void* user_data, int event_type);
        void register_callback(struct Context* ctx, event_callback_t callback, void* user_data);

        // Variadic functions
        int format_string(char* buffer, uint64_t size, const char* format, ...);
    "#;

    let mut config = BindgenConfig::default();
    config.set_library_name("advanced");

    // Add custom type mappings
    config.add_type_mapping("ssize_t", "isize");

    let mut bindgen = Bindgen::with_config(config);
    bindgen
        .parse_header(header)
        .expect("Failed to parse header");

    let output = bindgen.generate().expect("Failed to generate bindings");
    println!("{}", output);
}
