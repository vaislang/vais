#!/usr/bin/env python3
"""
Example usage of vais-python bindings

This demonstrates the Python API for the Vais compiler.
Note: You need to build the extension first with:
    pip install maturin
    cd crates/vais-python && maturin develop --release
"""

import sys
import vais

def print_section(title):
    """Print a section header"""
    print(f"\n{'='*60}")
    print(f"  {title}")
    print('='*60)

def example_tokenize():
    """Demonstrate tokenization"""
    print_section("Tokenization Example")

    source = "F add(a:i64, b:i64)->i64 = a + b"
    print(f"Source: {source}\n")

    try:
        tokens = vais.tokenize(source)
        print("Tokens:")
        for token in tokens:
            print(f"  {token}")
    except Exception as e:
        print(f"Error: {e}")

def example_parse():
    """Demonstrate parsing"""
    print_section("Parsing Example")

    source = "F factorial(n:i64)->i64 = if n == 0 { 1 } else { n * factorial(n - 1) }"
    print(f"Source: {source}\n")

    try:
        ast = vais.parse(source)
        print(f"AST: {ast}")
    except Exception as e:
        print(f"Error: {e}")

def example_check():
    """Demonstrate type checking"""
    print_section("Type Checking Example")

    # Valid code
    valid_source = "F add(a:i64, b:i64)->i64 = a + b"
    print(f"Valid source: {valid_source}")
    errors = vais.check(valid_source)
    if not errors:
        print("  ✓ No type errors!\n")
    else:
        print("  Errors:")
        for err in errors:
            print(f"    {err}")

    # Invalid code (type mismatch)
    invalid_source = "F bad(a:i64, b:bool)->i64 = a + b"
    print(f"Invalid source: {invalid_source}")
    errors = vais.check(invalid_source)
    if not errors:
        print("  ✓ No type errors!")
    else:
        print("  Errors:")
        for err in errors:
            print(f"    {err}")

def example_compile():
    """Demonstrate compilation"""
    print_section("Compilation Example (Function API)")

    source = "F square(x:i64)->i64 = x * x"
    print(f"Source: {source}\n")

    # Using compile() - raises on error
    try:
        ir = vais.compile(source, opt_level=2)
        print("Compiled LLVM IR (first 500 chars):")
        print(ir[:500] + "...")
    except Exception as e:
        print(f"Compilation failed: {e}")

    # Using compile_to_result() - returns result object
    print("\nUsing compile_to_result():")
    result = vais.compile_to_result(source, opt_level=2)
    if result.success:
        print(f"  ✓ Compilation successful!")
        print(f"  IR length: {len(result.ir) if result.ir else 0} chars")
    else:
        print("  Compilation failed:")
        for err in result.errors:
            print(f"    {err}")

def example_compile_error():
    """Demonstrate compilation error handling"""
    print_section("Error Handling Example")

    bad_source = "F broken syntax here"
    print(f"Source: {bad_source}\n")

    result = vais.compile_to_result(bad_source)
    if result.success:
        print("Compilation succeeded (unexpected!)")
    else:
        print("Compilation failed (as expected):")
        for err in result.errors:
            print(f"  {err.error_type}: {err.message}")

def example_compiler_class():
    """Demonstrate VaisCompiler class"""
    print_section("VaisCompiler Class Example")

    # Create compiler instance
    compiler = vais.VaisCompiler(opt_level=2, module_name="example")
    print(f"Created: {compiler}\n")

    # Compile multiple sources with same settings
    sources = [
        "F add(a:i64, b:i64)->i64 = a + b",
        "F multiply(a:i64, b:i64)->i64 = a * b",
        "F power_of_two(n:i64)->i64 = 1 << n",
    ]

    for source in sources:
        print(f"Compiling: {source}")
        result = compiler.compile(source)
        if result.success:
            print(f"  ✓ Success! IR length: {len(result.ir) if result.ir else 0}")
        else:
            print(f"  ✗ Failed:")
            for err in result.errors:
                print(f"    {err.error_type}: {err.message}")

    # Change settings
    print("\nChanging settings:")
    compiler.set_opt_level(3)
    compiler.set_module_name("optimized")
    print(f"  New settings: {compiler}")

    # Access properties
    print("\nCurrent configuration:")
    print(f"  Optimization level: {compiler.opt_level}")
    print(f"  Module name: {compiler.module_name}")
    print(f"  Target: {compiler.target}")

def example_compile_and_run():
    """Demonstrate compile_and_run (currently not implemented)"""
    print_section("Compile and Run Example")

    source = "F main()->i64 = 42"
    print(f"Source: {source}\n")

    result = vais.compile_and_run(source)
    if result.success:
        print(f"Exit code: {result.exit_code}")
        print(f"stdout: {result.stdout}")
        print(f"stderr: {result.stderr}")
    else:
        print("Execution failed (expected - JIT not yet implemented):")
        for err in result.errors:
            print(f"  {err.error_type}: {err.message}")

def main():
    """Run all examples"""
    print(f"Vais Python Bindings - Version {vais.__version__}")
    print(vais.__doc__)

    try:
        example_tokenize()
        example_parse()
        example_check()
        example_compile()
        example_compile_error()
        example_compiler_class()
        example_compile_and_run()

        print_section("All Examples Complete")
        print("✓ All examples completed successfully!")

    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    main()
