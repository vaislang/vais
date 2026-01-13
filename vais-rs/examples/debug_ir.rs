use vais_lowering::Lowerer;

fn main() {
    let source = "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2)";
    let program = vais_parser::parse(source).expect("Parse failed");
    let mut lowerer = Lowerer::new();
    let functions = lowerer.lower_program(&program).expect("Lowering failed");
    
    for func in &functions {
        println!("Function: {}", func.name);
        println!("  Params: {:?}", func.params);
        println!("  Local count: {}", func.local_count);
        println!("  Instructions:");
        for (i, instr) in func.instructions.iter().enumerate() {
            println!("    {}: {:?}", i, instr.opcode);
        }
        println!();
    }
}
