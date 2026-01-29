use vais_bindgen::Bindgen;

fn main() {
    let header = r#"
        // Simple C++ class example
        class Calculator {
        public:
            Calculator();
            ~Calculator();
            int add(int a, int b);
            int subtract(int a, int b);
            int multiply(int a, int b);
            double divide(double a, double b);
            int getLastResult();
        private:
            int lastResult;
        };

        class ScientificCalculator : public Calculator {
        public:
            double power(double base, double exp);
            double sqrt(double x);
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
