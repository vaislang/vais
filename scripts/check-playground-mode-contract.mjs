#!/usr/bin/env node

import assert from 'node:assert/strict';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const compilerUrl = pathToFileURL(resolve(root, 'playground/src/compiler.js')).href;

// Minimal module:
// (module (func $main (result i32) i32.const 0) (export "main" (func $main)))
const MINIMAL_WASM_BASE64 = 'AGFzbQEAAAABBQFgAAF/AwIBAAcIAQRtYWluAAAKBgEEAEEACw==';

globalThis.window = {
  location: {
    hostname: 'localhost',
  },
};

const { VaisCompiler } = await import(compilerUrl);

function jsonResponse(body, status = 200) {
  return {
    ok: status >= 200 && status < 300,
    status,
    statusText: status === 200 ? 'OK' : 'Error',
    json: async () => body,
  };
}

async function testPreviewFallbackIsExplicitlyNonCertified() {
  globalThis.fetch = async () => {
    throw new TypeError('offline');
  };

  const compiler = new VaisCompiler('http://localhost:8080');
  const result = await compiler.compileAndRun('F main(){ puts("hello") }');

  assert.equal(compiler.getModeLabel(), 'Preview');
  assert.equal(result.success, true);
  assert.match(result.output, /hello/);
  assert.match(result.output, /Preview mode\s+\u2014\s+syntax\/demo fallback only/);
  assert.doesNotMatch(result.output, /compiled in \d+ms/);
}

async function testServerWasmModeNamesApiCompileBoundary() {
  const calls = [];

  globalThis.fetch = async (url) => {
    calls.push(String(url));

    if (String(url).endsWith('/api/health')) {
      throw new TypeError('health offline');
    }

    if (String(url).endsWith('/api/compile-wasm')) {
      return jsonResponse({
        success: true,
        errors: [],
        warnings: [],
        wasm_binary: MINIMAL_WASM_BASE64,
        compile_time_ms: 7,
      });
    }

    throw new Error(`unexpected fetch URL: ${url}`);
  };

  const compiler = new VaisCompiler('http://localhost:8080');
  const result = await compiler.compileAndRun('F main()->i64{0}', 'wasm');

  assert.equal(compiler.getModeLabel(), 'Server-WASM');
  assert.equal(result.success, true);
  assert.match(result.output, /Server-WASM mode\s+\u2014\s+API compiled in 7ms, browser executed in \d+ms/);
  assert.ok(calls.some((url) => url.endsWith('/api/compile-wasm')));
  assert.ok(!calls.some((url) => url.endsWith('/api/compile')));
}

async function testBrowserJsModeDoesNotUseApi() {
  const calls = [];
  globalThis.fetch = async (url) => {
    calls.push(String(url));
    throw new TypeError('offline');
  };

  const browserCompiler = {
    initialize: async () => {},
    compileAndRun: async () => ({
      success: true,
      errors: [],
      warnings: [],
      output: '5\n\n[Browser-JS mode — browser compiled and executed JavaScript in 1ms]',
      exitCode: 0,
    }),
  };

  const compiler = new VaisCompiler('http://localhost:8080', { browserCompiler });
  const result = await compiler.compileAndRun('F main() -> i64 = 5', 'browser-js');

  assert.equal(compiler.getModeLabel(), 'Browser-JS');
  assert.equal(result.success, true);
  assert.match(result.output, /Browser-JS mode\s+\u2014\s+browser compiled and executed JavaScript/);
  assert.deepEqual(calls, ['http://localhost:8080/api/health']);
}

await testPreviewFallbackIsExplicitlyNonCertified();
await testServerWasmModeNamesApiCompileBoundary();
await testBrowserJsModeDoesNotUseApi();

console.log('Playground mode contract passed.');
