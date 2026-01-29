use vais_bindgen::Bindgen;

fn main() {
    let header = r#"
        // C++ template example
        template<typename T>
        class Vector {
        public:
            void push(T item);
            T pop();
            T get(int index);
            int size();
        private:
            T* data;
            int length;
            int capacity;
        };

        template<typename K, typename V>
        class Map {
        public:
            void insert(K key, V value);
            V get(K key);
            bool contains(K key);
        };

        // Non-template class for comparison
        class IntVector {
        public:
            void push(int item);
            int pop();
            int get(int index);
            int size();
        };
    "#;

    let mut bindgen = Bindgen::new_cpp();
    bindgen
        .parse_header(header)
        .expect("Failed to parse C++ header");

    println!("=== Generated Vais FFI Bindings ===\n");
    let output = bindgen.generate().expect("Failed to generate bindings");
    println!("{}", output);

    println!("\n=== Generated C Wrapper Header ===\n");
    let wrapper = bindgen
        .generate_cpp_wrapper_header()
        .expect("Failed to generate wrapper header");
    println!("{}", wrapper);
}
