// Compiler interface for Vais
// Supports server-side compilation, WASM execution, and mock mode (fallback)

import { WasmRunner } from './wasm-runner.js';

const DEFAULT_API_URL = window.location.hostname === 'localhost'
  ? 'http://localhost:8080'
  : 'https://api.vaislang.dev';

// Compilation modes
const MODE_SERVER = 'server';
const MODE_WASM = 'wasm';
const MODE_MOCK = 'mock';

export class VaisCompiler {
  constructor(apiUrl) {
    this.apiUrl = apiUrl || DEFAULT_API_URL;
    this.isReady = false;
    this.serverAvailable = false;
    this.wasmAvailable = false;
    this.wasmRunner = new WasmRunner();
    this.mode = MODE_MOCK;
  }

  async initialize() {
    // Try server first
    try {
      const response = await fetch(`${this.apiUrl}/api/health`, {
        signal: AbortSignal.timeout(2000),
      });
      if (response.ok) {
        const data = await response.json();
        this.serverAvailable = true;
        this.mode = MODE_SERVER;
        console.log(`Connected to Vais Playground server v${data.version}`);
      }
    } catch {
      this.serverAvailable = false;
      console.warn('Playground server not available');
    }

    // If no server, check if WASM compilation endpoint is available
    if (!this.serverAvailable) {
      try {
        const response = await fetch(`${this.apiUrl}/api/compile-wasm`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ source: 'F main()->i64{0}', target: 'wasm32' }),
          signal: AbortSignal.timeout(5000),
        });
        if (response.ok) {
          this.wasmAvailable = true;
          this.mode = MODE_WASM;
          console.log('WASM compilation available');
        }
      } catch {
        // WASM compilation not available either
      }
    }

    if (!this.serverAvailable && !this.wasmAvailable) {
      this.mode = MODE_MOCK;
      console.warn('Using mock mode (server and WASM both unavailable)');
    }

    this.isReady = true;
    return true;
  }

  /** Get current compilation mode */
  getMode() {
    return this.mode;
  }

  /** Get mode display string */
  getModeLabel() {
    switch (this.mode) {
      case MODE_SERVER: return 'Server';
      case MODE_WASM: return 'WASM';
      case MODE_MOCK: return 'Preview';
      default: return 'Unknown';
    }
  }

  async compile(sourceCode, target = 'native') {
    if (!this.isReady) {
      await this.initialize();
    }

    if (this.serverAvailable) {
      return this.serverCompile(sourceCode, { execute: false, emit_ir: true, target });
    }
    return this.mockCompile(sourceCode);
  }

  async execute(ir) {
    // Mock execution for fallback
    return this.mockExecute(ir);
  }

  async compileAndRun(sourceCode, target = 'native') {
    if (!this.isReady) {
      await this.initialize();
    }

    // Handle JS target - compile only, no execution
    if (target === 'js') {
      return this.compileToJs(sourceCode);
    }

    // Handle WASM target
    if (target === 'wasm') {
      if (this.wasmAvailable) {
        return this.wasmCompileAndRun(sourceCode);
      }
      if (this.serverAvailable) {
        return this.serverCompile(sourceCode, { execute: false, emit_ir: false, target: 'wasm32' });
      }
      return this.mockCompileAndRun(sourceCode);
    }

    // Handle native target (default)
    if (this.serverAvailable) {
      return this.serverCompile(sourceCode, { execute: true, emit_ir: false, target: 'native' });
    }
    if (this.wasmAvailable) {
      return this.wasmCompileAndRun(sourceCode);
    }
    return this.mockCompileAndRun(sourceCode);
  }

  // --- WASM compilation mode ---
  // Server compiles to WASM, then we execute in browser

  async wasmCompileAndRun(sourceCode) {
    try {
      // Step 1: Server-side compilation to WASM binary
      const response = await fetch(`${this.apiUrl}/api/compile-wasm`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          source: sourceCode,
          target: 'wasm32',
          optimize: false,
        }),
        signal: AbortSignal.timeout(30000),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        return {
          success: false,
          errors: errorData.errors || [{ line: 0, column: 0, message: `Server error: ${response.status}` }],
          warnings: [],
          output: null,
        };
      }

      const data = await response.json();

      if (!data.success) {
        return {
          success: false,
          errors: data.errors || [],
          warnings: data.warnings || [],
          output: null,
          compileTimeMs: data.compile_time_ms,
        };
      }

      // Step 2: Decode the WASM binary (base64 encoded from server)
      const wasmBinary = base64ToArrayBuffer(data.wasm_binary);
      const compileTimeMs = data.compile_time_ms || 0;

      // Step 3: Execute in browser
      const execStart = performance.now();
      const result = await this.wasmRunner.execute(wasmBinary, {
        timeout: 10000,
      });
      const execTimeMs = Math.round(performance.now() - execStart);

      let output = result.output || '';
      output += `\n\n[WASM mode — compiled in ${compileTimeMs}ms, executed in ${execTimeMs}ms]`;

      return {
        success: result.success,
        errors: [],
        warnings: data.warnings || [],
        output: output,
        exitCode: result.exitCode,
        compileTimeMs: compileTimeMs + execTimeMs,
      };
    } catch (error) {
      // If WASM mode fails, fall back to mock
      if (error.name === 'TimeoutError' || error.name === 'TypeError') {
        this.wasmAvailable = false;
        this.mode = MODE_MOCK;
        return this.mockCompileAndRun(sourceCode);
      }
      return {
        success: false,
        errors: [{ line: 0, column: 0, message: `WASM error: ${error.message}` }],
        warnings: [],
        output: null,
      };
    }
  }

  async serverCompile(sourceCode, options = {}) {
    try {
      const requestBody = {
        source: sourceCode,
        optimize: options.optimize || false,
        emit_ir: options.emit_ir || false,
        execute: options.execute !== false,
      };

      // Add target if specified
      if (options.target && options.target !== 'native') {
        requestBody.target = options.target;
      }

      const response = await fetch(`${this.apiUrl}/api/compile`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(requestBody),
        signal: AbortSignal.timeout(30000),
      });

      const data = await response.json();

      return {
        success: data.success,
        errors: data.errors || [],
        warnings: data.warnings || [],
        ir: data.ir || null,
        output: data.output || null,
        jsCode: data.js_code || null,
        exitCode: data.exit_code,
        compileTimeMs: data.compile_time_ms,
      };
    } catch (error) {
      // Server became unavailable, fall back
      if (error.name === 'TimeoutError' || error.name === 'TypeError') {
        this.serverAvailable = false;
        if (this.wasmAvailable) {
          this.mode = MODE_WASM;
          return this.wasmCompileAndRun(sourceCode);
        }
        this.mode = MODE_MOCK;
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

  // --- JavaScript target compilation ---
  async compileToJs(sourceCode) {
    if (!this.serverAvailable) {
      return {
        success: false,
        errors: [{ line: 0, column: 0, message: 'JavaScript compilation requires server (not available in preview mode)' }],
        warnings: [],
        output: null,
      };
    }

    try {
      const response = await fetch(`${this.apiUrl}/api/compile`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          source: sourceCode,
          target: 'js',
          optimize: false,
          emit_ir: false,
          execute: false,
        }),
        signal: AbortSignal.timeout(30000),
      });

      const data = await response.json();

      if (!data.success) {
        return {
          success: false,
          errors: data.errors || [],
          warnings: data.warnings || [],
          output: null,
        };
      }

      // Format the JS code output
      const jsCode = data.js_code || data.output || '// No code generated';
      let output = '=== Generated JavaScript (ESM) ===\n\n';
      output += jsCode;
      output += '\n\n';
      output += `[JavaScript target — compiled in ${data.compile_time_ms || 0}ms]`;

      return {
        success: true,
        errors: [],
        warnings: data.warnings || [],
        output: output,
        jsCode: jsCode,
        compileTimeMs: data.compile_time_ms,
      };
    } catch (error) {
      return {
        success: false,
        errors: [{ line: 0, column: 0, message: `JS compilation error: ${error.message}` }],
        warnings: [],
        output: null,
      };
    }
  }
}

// Utility: decode base64 to ArrayBuffer
function base64ToArrayBuffer(base64) {
  const binaryString = atob(base64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes.buffer;
}

export default VaisCompiler;
