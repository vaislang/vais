// Compiler interface for Vais
// Supports both server-side compilation (via REST API) and mock mode (fallback)

const DEFAULT_API_URL = window.location.hostname === 'localhost'
  ? 'http://localhost:8080'
  : 'https://api.vaislang.dev';

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
    const lines = sourceCode.split('\n');

    if (!sourceCode.trim()) {
      errors.push({ line: 1, column: 1, message: 'Empty source file' });
      return { success: false, errors, warnings, output: null };
    }

    // Check for main function
    if (!sourceCode.includes('F main')) {
      errors.push({ line: 1, column: 1, message: 'No main function found (expected `F main()`)' });
    }

    // Brace matching
    const openBraces = (sourceCode.match(/\{/g) || []).length;
    const closeBraces = (sourceCode.match(/\}/g) || []).length;
    if (openBraces !== closeBraces) {
      errors.push({
        line: lines.length,
        column: 1,
        message: `Mismatched braces: ${openBraces} opening, ${closeBraces} closing`,
      });
    }

    // Parenthesis matching
    const openParens = (sourceCode.match(/\(/g) || []).length;
    const closeParens = (sourceCode.match(/\)/g) || []).length;
    if (openParens !== closeParens) {
      errors.push({
        line: lines.length,
        column: 1,
        message: `Mismatched parentheses: ${openParens} opening, ${closeParens} closing`,
      });
    }

    // Check for common syntax issues per line
    lines.forEach((line, idx) => {
      const trimmed = line.trim();
      if (trimmed.startsWith('#') || !trimmed) return; // skip comments and empty lines

      // Detect unterminated strings
      const quotes = (trimmed.match(/"/g) || []).length;
      if (quotes % 2 !== 0) {
        errors.push({ line: idx + 1, column: 1, message: 'Unterminated string literal' });
      }
    });

    if (errors.length > 0) {
      return { success: false, errors, warnings, output: null };
    }

    return {
      success: true,
      errors: [],
      warnings,
      ir: null,
      output: null,
    };
  }

  mockSimulateOutput(sourceCode) {
    // Extract expected output by parsing puts/println/putchar calls from source
    const output = [];
    const lines = sourceCode.split('\n');

    for (const line of lines) {
      const trimmed = line.trim();

      // Match puts("...") calls
      const putsMatch = trimmed.match(/puts\("([^"]*)"\)/);
      if (putsMatch) {
        output.push(putsMatch[1]);
        continue;
      }

      // Match println("...") calls (with simple interpolation)
      const printlnMatch = trimmed.match(/println\("([^"]*)"\)/);
      if (printlnMatch) {
        // Replace {expr} with <expr> for display
        const text = printlnMatch[1]
          .replace(/\{\{/g, '{')
          .replace(/\}\}/g, '}')
          .replace(/\{([^}]+)\}/g, '<$1>');
        output.push(text);
        continue;
      }
    }

    return output;
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

    // Simulate output from source code analysis
    const simulatedOutput = this.mockSimulateOutput(sourceCode);

    // Count language constructs for summary
    const funcCount = (sourceCode.match(/\bF\s+\w+/g) || []).length;
    const structCount = (sourceCode.match(/\bS\s+\w+/g) || []).length;
    const enumCount = (sourceCode.match(/\bE\s+\w+/g) || []).length;

    const summary = [];
    if (funcCount > 0) summary.push(`${funcCount} function(s)`);
    if (structCount > 0) summary.push(`${structCount} struct(s)`);
    if (enumCount > 0) summary.push(`${enumCount} enum(s)`);

    let outputText = '';
    if (simulatedOutput.length > 0) {
      outputText = simulatedOutput.join('\n');
    } else {
      outputText = `Program compiled successfully (${summary.join(', ')})`;
    }

    outputText += '\n\n[Preview mode — compile server offline. Install locally: cargo install vaisc]';

    return {
      success: true,
      errors: [],
      warnings: compileResult.warnings,
      output: outputText,
      exitCode: 0,
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
