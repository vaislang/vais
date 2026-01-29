// Compiler interface for Vais
// Supports both server-side compilation (via REST API) and mock mode (fallback)

const DEFAULT_API_URL = 'http://localhost:8080';

export class VaisCompiler {
  constructor(apiUrl) {
    this.apiUrl = apiUrl || DEFAULT_API_URL;
    this.isReady = false;
    this.serverAvailable = false;
  }

  async initialize() {
    // Check if the server is available
    try {
      const response = await fetch(`${this.apiUrl}/api/health`, {
        signal: AbortSignal.timeout(2000),
      });
      if (response.ok) {
        const data = await response.json();
        this.serverAvailable = true;
        console.log(`Connected to Vais Playground server v${data.version}`);
      }
    } catch {
      this.serverAvailable = false;
      console.warn('Playground server not available, using mock mode');
    }

    this.isReady = true;
    return true;
  }

  async compile(sourceCode) {
    if (!this.isReady) {
      await this.initialize();
    }

    if (this.serverAvailable) {
      return this.serverCompile(sourceCode, { execute: false, emit_ir: true });
    }
    return this.mockCompile(sourceCode);
  }

  async execute(ir) {
    // Mock execution for fallback
    return this.mockExecute(ir);
  }

  async compileAndRun(sourceCode) {
    if (!this.isReady) {
      await this.initialize();
    }

    if (this.serverAvailable) {
      return this.serverCompile(sourceCode, { execute: true, emit_ir: false });
    }
    return this.mockCompileAndRun(sourceCode);
  }

  async serverCompile(sourceCode, options = {}) {
    try {
      const response = await fetch(`${this.apiUrl}/api/compile`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          source: sourceCode,
          optimize: options.optimize || false,
          emit_ir: options.emit_ir || false,
          execute: options.execute !== false,
        }),
        signal: AbortSignal.timeout(30000),
      });

      const data = await response.json();

      return {
        success: data.success,
        errors: data.errors || [],
        warnings: data.warnings || [],
        ir: data.ir || null,
        output: data.output || null,
        exitCode: data.exit_code,
        compileTimeMs: data.compile_time_ms,
      };
    } catch (error) {
      // Server became unavailable, fall back to mock
      if (error.name === 'TimeoutError' || error.name === 'TypeError') {
        this.serverAvailable = false;
        return this.mockCompileAndRun(sourceCode);
      }
      return {
        success: false,
        errors: [{ line: 0, column: 0, message: `Server error: ${error.message}` }],
        warnings: [],
        output: null,
      };
    }
  }

  // --- Mock implementation (fallback when server is unavailable) ---

  mockCompile(sourceCode) {
    const errors = [];
    const warnings = [];

    if (!sourceCode.trim()) {
      errors.push({ line: 1, column: 1, message: 'Empty source file' });
    }

    if (!sourceCode.includes('F main')) {
      warnings.push({ line: 1, column: 1, message: 'No main function found' });
    }

    const openBraces = (sourceCode.match(/\{/g) || []).length;
    const closeBraces = (sourceCode.match(/\}/g) || []).length;
    if (openBraces !== closeBraces) {
      errors.push({
        line: sourceCode.split('\n').length,
        column: 1,
        message: `Mismatched braces: ${openBraces} opening, ${closeBraces} closing`,
      });
    }

    if (errors.length > 0) {
      return { success: false, errors, warnings, output: null };
    }

    const ir = this.generateMockIR(sourceCode);
    return {
      success: true,
      errors: [],
      warnings,
      ir,
      output: 'Compilation successful (mock mode)',
    };
  }

  generateMockIR(sourceCode) {
    return `; ModuleID = 'playground.vais'
source_filename = "playground.vais"

declare i32 @puts(i8*)
declare i32 @putchar(i32)

define i64 @main() {
entry:
  ; Your code would be compiled here
  ret i64 0
}
`;
  }

  mockExecute(ir) {
    const output = [];
    const putsMatches = ir.match(/@puts\("([^"]*)"\)/g);
    if (putsMatches) {
      putsMatches.forEach(match => {
        const str = match.match(/"([^"]*)"/)[1];
        output.push(str);
      });
    }

    return {
      success: true,
      output: output.length > 0 ? output.join('\n') : 'Program executed successfully',
      exitCode: 0,
    };
  }

  async mockCompileAndRun(sourceCode) {
    const compileResult = this.mockCompile(sourceCode);
    if (!compileResult.success) {
      return {
        success: false,
        errors: compileResult.errors,
        warnings: compileResult.warnings,
        output: null,
      };
    }

    const execResult = this.mockExecute(compileResult.ir);
    return {
      success: true,
      errors: [],
      warnings: compileResult.warnings,
      output: execResult.output + ' (mock mode - start server for real compilation)',
      exitCode: execResult.exitCode,
    };
  }

  formatError(error) {
    return `Error at line ${error.line}, column ${error.column}: ${error.message}`;
  }

  formatWarning(warning) {
    return `Warning at line ${warning.line}, column ${warning.column}: ${warning.message}`;
  }
}

export default VaisCompiler;
