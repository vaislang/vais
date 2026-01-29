use vais_bindgen::Bindgen;

fn main() {
    let header = r#"
        // C++ namespace example
        namespace Math {
            int square(int x);
            int cube(int x);
            double sqrt(double x);

            class Vector {
            public:
                double x;
                double y;
                double z;

                double length();
                void normalize();
            };
        }

        namespace Utils {
            void print(const char* msg);
            int randomInt(int min, int max);
        }
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
