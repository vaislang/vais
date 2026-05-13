const viteEnv = import.meta.env || {};
const DEFAULT_MODULE_URL = `${viteEnv.BASE_URL || '/'}wasm/vais_browser_compiler.js`;

export class BrowserCompiler {
  constructor(options = {}) {
    this.moduleUrl = options.moduleUrl || DEFAULT_MODULE_URL;
    this.wasmBytes = options.wasmBytes;
    this.module = null;
    this._initPromise = null;
  }

  async initialize() {
    if (this.module) {
      return;
    }

    if (!this._initPromise) {
      this._initPromise = this._loadModule();
    }

    await this._initPromise;
  }

  async _loadModule() {
    const module = await import(/* @vite-ignore */ this.moduleUrl);
    const initInput = this.wasmBytes ? { module_or_path: this.wasmBytes } : undefined;
    await module.default(initInput);
    this.module = module;
  }

  async compileToJs(sourceCode) {
    await this.initialize();
    const raw = this.module.compile_to_js_json(sourceCode);
    const result = JSON.parse(raw);

    return {
      success: result.success,
      errors: result.errors || [],
      warnings: result.warnings || [],
      jsCode: result.js_code || null,
      compiler: result.compiler || 'vais-browser-compiler',
    };
  }

  async compileAndRun(sourceCode) {
    const start = performance.now();
    const compiled = await this.compileToJs(sourceCode);

    if (!compiled.success) {
      return {
        success: false,
        errors: compiled.errors,
        warnings: compiled.warnings,
        output: null,
        jsCode: compiled.jsCode,
      };
    }

    try {
      const output = [];
      const result = await executeGeneratedJs(compiled.jsCode, output);
      let outputText = output.join('');

      if (result !== undefined && outputText.length === 0) {
        outputText = String(result);
      }

      const elapsed = Math.round(performance.now() - start);
      outputText += `\n\n[Browser-JS mode — browser compiled and executed JavaScript in ${elapsed}ms]`;

      return {
        success: true,
        errors: [],
        warnings: compiled.warnings,
        output: outputText,
        jsCode: compiled.jsCode,
        exitCode: 0,
        compileTimeMs: elapsed,
      };
    } catch (error) {
      return {
        success: false,
        errors: [{ line: 0, column: 0, message: `Browser-JS execution error: ${error.message}` }],
        warnings: compiled.warnings,
        output: null,
        jsCode: compiled.jsCode,
      };
    }
  }
}

function executeGeneratedJs(jsCode, output) {
  const runnableCode = makeRunnableJs(jsCode);
  const puts = (value) => {
    output.push(`${String(value)}\n`);
    return 0;
  };
  const println = puts;
  const print = (value) => {
    output.push(String(value));
    return 0;
  };
  const putchar = (value) => {
    output.push(String.fromCharCode(Number(value)));
    return Number(value);
  };

  const run = new Function(
    'puts',
    'println',
    'print',
    'putchar',
    'console',
    `${runnableCode}\nreturn typeof main === 'function' ? main() : undefined;`,
  );

  return run(puts, println, print, putchar, console);
}

function makeRunnableJs(jsCode) {
  return jsCode.replace(/\bexport\s+/g, '');
}

export default BrowserCompiler;
