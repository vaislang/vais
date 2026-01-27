use vais_bindgen::Bindgen;

fn main() {
    let header = r#"
        // Simple math library
        int add(int a, int b);
        int subtract(int a, int b);
        double multiply(double a, double b);
        double divide(double a, double b);

        typedef struct {
            double x;
            double y;
        } Point;

        double distance(Point p1, Point p2);
    "#;

    let mut bindgen = Bindgen::new();
    bindgen
        .parse_header(header)
        .expect("Failed to parse header");

    let output = bindgen.generate().expect("Failed to generate bindings");
    println!("{}", output);
}
