#!/usr/bin/env node

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const browserCompilerUrl = pathToFileURL(
  resolve(root, 'playground/public/wasm/vais_browser_compiler.js'),
).href;
const wasmBytes = readFileSync(resolve(root, 'playground/public/wasm/vais_browser_compiler_bg.wasm'));

globalThis.fetch = async (url) => {
  throw new Error(`browser compiler gate must not fetch ${url}`);
};

const { BrowserCompiler } = await import(pathToFileURL(
  resolve(root, 'playground/src/browser-compiler.js'),
).href);

const compiler = new BrowserCompiler({
  moduleUrl: browserCompilerUrl,
  wasmBytes,
});

const compiled = await compiler.compileToJs('fn main() -> i64 = 42');
assert.equal(compiled.success, true);
assert.match(compiled.jsCode, /function main\(\)/);

const executed = await compiler.compileAndRun('fn main() -> i64 = 42');
assert.equal(executed.success, true);
assert.match(executed.output, /^42/);
assert.match(executed.output, /Browser-JS mode\s+\u2014\s+browser compiled and executed JavaScript/);

const invalid = await compiler.compileToJs('fn main( -> i64 = 42');
assert.equal(invalid.success, false);
assert.ok(invalid.errors.length > 0);

console.log('Browser compiler gate passed.');
