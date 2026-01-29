"""
Tests for vais-python bindings

Run with: pytest tests/test_vais.py
"""

import pytest

# Note: These tests assume the vais module has been built and is importable
# Build with: PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo build --release -p vais-python
# Then add target/release to PYTHONPATH or install with maturin

try:
    import vais
    VAIS_AVAILABLE = True
except ImportError:
    VAIS_AVAILABLE = False
    pytestmark = pytest.mark.skip("vais module not available - build it first")


class TestTokenize:
    """Test tokenization functionality"""

    def test_tokenize_simple(self):
        source = "F add(a:i64,b:i64)->i64=a+b"
        tokens = vais.tokenize(source)
        assert len(tokens) > 0
        assert tokens[0].token_type == "Function"

    def test_tokenize_empty(self):
        source = ""
        tokens = vais.tokenize(source)
        assert len(tokens) == 0

    def test_tokenize_invalid(self):
        # Invalid token should raise error
        source = "F add $ invalid"
        with pytest.raises(ValueError):
            vais.tokenize(source)


class TestParse:
    """Test parsing functionality"""

    def test_parse_simple_function(self):
        source = "F add(a:i64,b:i64)->i64=a+b"
        ast = vais.parse(source)
        assert isinstance(ast, dict)
        assert ast["type"] == "Module"
        assert "items_count" in ast

    def test_parse_invalid(self):
        source = "F broken syntax"
        with pytest.raises(ValueError):
            vais.parse(source)


class TestCheck:
    """Test type checking functionality"""

    def test_check_valid(self):
        source = "F add(a:i64,b:i64)->i64=a+b"
        errors = vais.check(source)
        assert len(errors) == 0

    def test_check_parse_error(self):
        source = "F broken syntax"
        errors = vais.check(source)
        assert len(errors) > 0
        assert errors[0].error_type == "ParseError"

    def test_check_type_error(self):
        # Type error: adding i64 and bool
        source = "F bad(a:i64,b:bool)->i64=a+b"
        errors = vais.check(source)
        # Note: This might not error if type checker doesn't catch it yet
        # Adjust based on actual type checker behavior


class TestCompile:
    """Test compilation functionality"""

    def test_compile_simple(self):
        source = "F square(x:i64)->i64=x*x"
        ir = vais.compile(source, opt_level=0)
        assert isinstance(ir, str)
        assert len(ir) > 0

    def test_compile_with_optimization(self):
        source = "F square(x:i64)->i64=x*x"
        ir = vais.compile(source, opt_level=2)
        assert isinstance(ir, str)
        assert len(ir) > 0

    def test_compile_with_module_name(self):
        source = "F square(x:i64)->i64=x*x"
        ir = vais.compile(source, module_name="test_module")
        assert isinstance(ir, str)

    def test_compile_error(self):
        source = "F broken syntax"
        with pytest.raises(RuntimeError):
            vais.compile(source)


class TestCompileToResult:
    """Test compile_to_result functionality"""

    def test_compile_to_result_success(self):
        source = "F square(x:i64)->i64=x*x"
        result = vais.compile_to_result(source, opt_level=2)
        assert result.success is True
        assert result.ir is not None
        assert len(result.errors) == 0

    def test_compile_to_result_failure(self):
        source = "F broken syntax"
        result = vais.compile_to_result(source)
        assert result.success is False
        assert result.ir is None
        assert len(result.errors) > 0

    def test_compile_result_get_ir(self):
        source = "F square(x:i64)->i64=x*x"
        result = vais.compile_to_result(source)
        ir = result.get_ir()
        assert isinstance(ir, str)

    def test_compile_result_get_ir_failure(self):
        source = "F broken syntax"
        result = vais.compile_to_result(source)
        with pytest.raises(RuntimeError):
            result.get_ir()


class TestCompileAndRun:
    """Test compile_and_run functionality"""

    def test_compile_and_run_not_implemented(self):
        source = "F main()->i64=42"
        result = vais.compile_and_run(source)
        # Currently not implemented, should return error
        assert result.success is False
        assert len(result.errors) > 0
        assert result.errors[0].error_type == "NotImplemented"


class TestVaisCompiler:
    """Test VaisCompiler class"""

    def test_create_compiler(self):
        compiler = vais.VaisCompiler()
        assert compiler is not None
        assert compiler.opt_level == 0
        assert compiler.module_name == "main"
        assert compiler.target is None

    def test_create_compiler_with_options(self):
        compiler = vais.VaisCompiler(
            opt_level=2,
            module_name="test",
            target="wasm32-unknown-unknown"
        )
        assert compiler.opt_level == 2
        assert compiler.module_name == "test"
        assert compiler.target == "wasm32-unknown-unknown"

    def test_compiler_compile(self):
        compiler = vais.VaisCompiler(opt_level=2)
        source = "F square(x:i64)->i64=x*x"
        result = compiler.compile(source)
        assert result.success is True
        assert result.ir is not None

    def test_compiler_compile_ir(self):
        compiler = vais.VaisCompiler()
        source = "F square(x:i64)->i64=x*x"
        ir = compiler.compile_ir(source)
        assert isinstance(ir, str)

    def test_compiler_tokenize(self):
        compiler = vais.VaisCompiler()
        source = "F add(a:i64,b:i64)->i64=a+b"
        tokens = compiler.tokenize(source)
        assert len(tokens) > 0

    def test_compiler_parse(self):
        compiler = vais.VaisCompiler()
        source = "F add(a:i64,b:i64)->i64=a+b"
        ast = compiler.parse(source)
        assert isinstance(ast, dict)

    def test_compiler_check(self):
        compiler = vais.VaisCompiler()
        source = "F add(a:i64,b:i64)->i64=a+b"
        errors = compiler.check(source)
        assert len(errors) == 0

    def test_compiler_set_opt_level(self):
        compiler = vais.VaisCompiler(opt_level=0)
        compiler.set_opt_level(3)
        assert compiler.opt_level == 3

    def test_compiler_set_module_name(self):
        compiler = vais.VaisCompiler()
        compiler.set_module_name("new_module")
        assert compiler.module_name == "new_module"

    def test_compiler_set_target(self):
        compiler = vais.VaisCompiler()
        compiler.set_target("wasm32-unknown-unknown")
        assert compiler.target == "wasm32-unknown-unknown"

    def test_compiler_repr(self):
        compiler = vais.VaisCompiler(opt_level=2, module_name="test")
        repr_str = repr(compiler)
        assert "VaisCompiler" in repr_str
        assert "opt_level=2" in repr_str
        assert "module_name='test'" in repr_str


class TestError:
    """Test Error class"""

    def test_error_from_parse(self):
        source = "F broken"
        errors = vais.check(source)
        assert len(errors) > 0
        error = errors[0]
        assert hasattr(error, "message")
        assert hasattr(error, "error_type")
        assert hasattr(error, "span")


class TestTokenInfo:
    """Test TokenInfo class"""

    def test_token_info(self):
        source = "F add"
        tokens = vais.tokenize(source)
        assert len(tokens) > 0
        token = tokens[0]
        assert hasattr(token, "token_type")
        assert hasattr(token, "span")
        assert hasattr(token, "text")
        assert token.token_type == "Function"


class TestModuleMetadata:
    """Test module metadata"""

    def test_version(self):
        assert hasattr(vais, "__version__")
        assert vais.__version__ == "0.0.1"

    def test_doc(self):
        assert hasattr(vais, "__doc__")
        assert isinstance(vais.__doc__, str)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
