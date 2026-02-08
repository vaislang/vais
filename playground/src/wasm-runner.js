// WASM Runner for Vais Playground
// Executes pre-compiled Vais WebAssembly modules in the browser

const WASM_PAGE_SIZE = 65536; // 64KB per WASM page
const INITIAL_PAGES = 16;     // 1MB initial memory
const MAX_PAGES = 256;        // 16MB max memory

export class WasmRunner {
  constructor() {
    this.memory = null;
    this.instance = null;
    this.outputBuffer = [];
    this.decoder = new TextDecoder('utf-8');
    this.encoder = new TextEncoder();
  }

  /**
   * Execute a compiled WASM binary from bytes.
   * @param {ArrayBuffer} wasmBytes - The compiled .wasm file
   * @param {object} options - Execution options
   * @returns {Promise<{success: boolean, output: string, exitCode: number}>}
   */
  async execute(wasmBytes, options = {}) {
    this.outputBuffer = [];
    const timeout = options.timeout || 10000; // 10s default

    try {
      this.memory = new WebAssembly.Memory({
        initial: INITIAL_PAGES,
        maximum: MAX_PAGES,
      });

      const importObject = this.createImports();
      const module = await WebAssembly.compile(wasmBytes);
      this.instance = await WebAssembly.instantiate(module, importObject);

      // Run with timeout
      const result = await this.runWithTimeout(timeout);

      return {
        success: true,
        output: this.outputBuffer.join(''),
        exitCode: result,
      };
    } catch (error) {
      if (error.name === 'WasmTrap' || error instanceof WebAssembly.RuntimeError) {
        return {
          success: true,
          output: this.outputBuffer.join(''),
          exitCode: error.message.includes('unreachable') ? 1 : -1,
        };
      }
      if (error.message === 'execution_timeout') {
        return {
          success: false,
          output: this.outputBuffer.join('') + '\n[Execution timed out]',
          exitCode: -1,
        };
      }
      return {
        success: false,
        output: `WASM execution error: ${error.message}`,
        exitCode: -1,
      };
    }
  }

  /**
   * Execute a WASM module from a URL (fetch + execute).
   * @param {string} url - URL to the .wasm file
   * @param {object} options - Execution options
   */
  async executeFromUrl(url, options = {}) {
    const response = await fetch(url);
    if (!response.ok) {
      return {
        success: false,
        output: `Failed to fetch WASM: ${response.status} ${response.statusText}`,
        exitCode: -1,
      };
    }
    const bytes = await response.arrayBuffer();
    return this.execute(bytes, options);
  }

  /**
   * Run the WASM _start function with a timeout.
   */
  runWithTimeout(timeoutMs) {
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        reject(new Error('execution_timeout'));
      }, timeoutMs);

      try {
        const exports = this.instance.exports;
        let exitCode = 0;

        if (exports._start) {
          exitCode = exports._start() || 0;
        } else if (exports.main) {
          exitCode = exports.main() || 0;
        } else {
          reject(new Error('No _start or main export found'));
          clearTimeout(timer);
          return;
        }

        clearTimeout(timer);
        resolve(exitCode);
      } catch (error) {
        clearTimeout(timer);
        // Trap on exit is normal (unreachable instruction used for exit)
        if (error instanceof WebAssembly.RuntimeError &&
            error.message.includes('unreachable')) {
          resolve(0);
        } else {
          reject(error);
        }
      }
    });
  }

  /**
   * Create WASM import objects for the Vais runtime.
   */
  createImports() {
    const self = this;

    return {
      env: {
        memory: this.memory,

        // Output: __wasm_write(ptr, len)
        __wasm_write: (ptr, len) => {
          const bytes = new Uint8Array(self.memory.buffer, ptr, len);
          const text = self.decoder.decode(bytes);
          self.outputBuffer.push(text);
        },

        // puts equivalent
        puts: (ptr) => {
          const bytes = new Uint8Array(self.memory.buffer, ptr);
          let end = 0;
          while (bytes[end] !== 0 && end < bytes.length) end++;
          const text = self.decoder.decode(bytes.subarray(0, end));
          self.outputBuffer.push(text + '\n');
          return 0;
        },

        // printf simplified (just prints the format string, no args)
        printf: (fmtPtr) => {
          const bytes = new Uint8Array(self.memory.buffer, fmtPtr);
          let end = 0;
          while (bytes[end] !== 0 && end < bytes.length) end++;
          const text = self.decoder.decode(bytes.subarray(0, end));
          self.outputBuffer.push(text);
          return text.length;
        },

        // putchar
        putchar: (ch) => {
          self.outputBuffer.push(String.fromCharCode(ch));
          return ch;
        },

        // exit via trap
        __wasm_trap: () => {
          throw new WebAssembly.RuntimeError('unreachable');
        },

        // Time
        __time_now_ms: () => {
          return Date.now();
        },

        // Memory intrinsics
        __wasm_memory_size: () => {
          return self.memory.buffer.byteLength / WASM_PAGE_SIZE;
        },

        __wasm_memory_grow: (pages) => {
          try {
            return self.memory.grow(pages);
          } catch {
            return -1;
          }
        },
      },

      // WASI preview1 imports (for wasm32-wasi target)
      wasi_snapshot_preview1: {
        fd_write: (fd, iovsPtr, iovsLen, nwrittenPtr) => {
          const view = new DataView(self.memory.buffer);
          let totalWritten = 0;

          for (let i = 0; i < iovsLen; i++) {
            const offset = iovsPtr + i * 8;
            const bufPtr = view.getUint32(offset, true);
            const bufLen = view.getUint32(offset + 4, true);
            const bytes = new Uint8Array(self.memory.buffer, bufPtr, bufLen);
            const text = self.decoder.decode(bytes);

            if (fd === 1) {
              self.outputBuffer.push(text); // stdout
            } else if (fd === 2) {
              self.outputBuffer.push(`[stderr] ${text}`); // stderr
            }

            totalWritten += bufLen;
          }

          view.setUint32(nwrittenPtr, totalWritten, true);
          return 0; // success
        },

        fd_read: (fd, iovsPtr, iovsLen, nreadPtr) => {
          const view = new DataView(self.memory.buffer);
          view.setUint32(nreadPtr, 0, true);
          return 0;
        },

        fd_close: () => 0,
        fd_seek: () => 0,
        fd_fdstat_get: () => 0,
        fd_prestat_get: () => 8, // EBADF - no preopened directories
        fd_prestat_dir_name: () => 8,
        environ_sizes_get: (countPtr, sizePtr) => {
          const view = new DataView(self.memory.buffer);
          view.setUint32(countPtr, 0, true);
          view.setUint32(sizePtr, 0, true);
          return 0;
        },
        environ_get: () => 0,
        args_sizes_get: (countPtr, sizePtr) => {
          const view = new DataView(self.memory.buffer);
          view.setUint32(countPtr, 0, true);
          view.setUint32(sizePtr, 0, true);
          return 0;
        },
        args_get: () => 0,
        clock_time_get: (clockId, precision, timePtr) => {
          const view = new DataView(self.memory.buffer);
          const now = BigInt(Date.now()) * 1000000n; // ns
          view.setBigUint64(timePtr, now, true);
          return 0;
        },
        proc_exit: (code) => {
          throw new WebAssembly.RuntimeError(`unreachable: exit(${code})`);
        },
        random_get: (bufPtr, bufLen) => {
          const buf = new Uint8Array(self.memory.buffer, bufPtr, bufLen);
          crypto.getRandomValues(buf);
          return 0;
        },
      },
    };
  }

  /**
   * Read a null-terminated string from WASM memory.
   */
  readCString(ptr) {
    const bytes = new Uint8Array(this.memory.buffer, ptr);
    let end = 0;
    while (bytes[end] !== 0 && end < 65536) end++;
    return this.decoder.decode(bytes.subarray(0, end));
  }

  /**
   * Write a string to WASM memory at the given offset.
   */
  writeCString(ptr, str) {
    const encoded = this.encoder.encode(str);
    const bytes = new Uint8Array(this.memory.buffer, ptr);
    bytes.set(encoded);
    bytes[encoded.length] = 0;
    return encoded.length;
  }
}

export default WasmRunner;
