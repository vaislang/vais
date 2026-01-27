use vais_bindgen::{Bindgen, BindgenConfig};

fn main() {
    let header = r#"
        // Graphics library FFI

        typedef struct {
            float x;
            float y;
        } Vec2;

        typedef struct {
            float x;
            float y;
            float z;
        } Vec3;

        typedef struct {
            float r;
            float g;
            float b;
            float a;
        } Color;

        struct Window;
        struct Texture;

        // Window management
        struct Window* create_window(int width, int height, const char* title);
        void destroy_window(struct Window* window);
        int is_window_open(struct Window* window);
        void poll_events(struct Window* window);

        // Drawing
        void clear(struct Window* window, Color color);
        void draw_rectangle(struct Window* window, Vec2 pos, Vec2 size, Color color);
        void draw_circle(struct Window* window, Vec2 pos, float radius, Color color);
        void present(struct Window* window);

        // Texture management
        struct Texture* load_texture(const char* path);
        void destroy_texture(struct Texture* texture);
        void draw_texture(struct Window* window, struct Texture* texture, Vec2 pos);

        // Input
        int is_key_pressed(struct Window* window, int key);
        Vec2 get_mouse_position(struct Window* window);
    "#;

    let mut config = BindgenConfig::default();
    config.set_library_name("graphics");

    let mut bindgen = Bindgen::with_config(config);
    bindgen
        .parse_header(header)
        .expect("Failed to parse header");

    let output = bindgen.generate().expect("Failed to generate bindings");
    println!("{}", output);
}
