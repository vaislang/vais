#!/usr/bin/env node

import { readFileSync } from 'node:fs';
import { dirname, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

function read(path) {
  return readFileSync(resolve(root, path), 'utf8');
}

const failures = [];

function fail(message) {
  failures.push(message);
}

function requireText(path, expected, reason) {
  const content = read(path);
  if (!content.includes(expected)) {
    fail(`${path}: missing "${expected}" (${reason})`);
  }
}

function requirePattern(path, pattern, reason) {
  const content = read(path);
  if (!pattern.test(content)) {
    fail(`${path}: missing pattern ${pattern} (${reason})`);
  }
}

function forbidPattern(path, pattern, reason) {
  const content = read(path);
  if (pattern.test(content)) {
    fail(`${path}: forbidden pattern ${pattern} (${reason})`);
  }
}

function forbidText(path, forbidden, reason) {
  const content = read(path);
  if (content.includes(forbidden)) {
    fail(`${path}: forbidden "${forbidden}" (${reason})`);
  }
}

const publicStatus = 'PUBLIC_STATUS.md';
requireText(
  publicStatus,
  'The current certified baseline is a core compiler and promoted-runtime',
  'public baseline must not claim product-complete status',
);
requireText(
  publicStatus,
  'It is not a product-complete v1.0 release.',
  'public baseline must not claim product-complete status',
);
requireText(
  publicStatus,
  'Complete browser-only playground compilation and execution',
  'browser-only playground compile/execute remains a public non-claim',
);
requireText(
  publicStatus,
  'Playground web mode/build gate: passed',
  'playground web gate can be claimed only as a mode/build contract',
);
requireText(
  publicStatus,
  'Server-WASM remains API-compiled',
  'playground status must keep the server compile boundary explicit',
);
requireText(
  publicStatus,
  'Browser-JS playground smoke gate: passed',
  'browser JS smoke gate must be named without promoting complete browser-only status',
);
requireText(
  publicStatus,
  'this is not a complete browser-only language',
  'browser JS smoke gate must not imply full browser-only language completion',
);
requireText(
  publicStatus,
  'Vais Server runtime smoke: `20/20`',
  'server public baseline must match the promoted runtime gate',
);
requireText(
  publicStatus,
  'Cross-package schema gate: `15/15`',
  'schema propagation must be named as a gate-backed public claim',
);
requireText(
  publicStatus,
  'Multi-domain product schema gate: `9/9`',
  'shared-schema product propagation must be named as a gate-backed public claim',
);

const playgroundCompiler = 'playground/src/compiler.js';
requireText(
  playgroundCompiler,
  "case MODE_WASM: return 'Server-WASM';",
  'UI mode label must show that WASM compilation is server-backed',
);
requirePattern(
  playgroundCompiler,
  /Server-WASM mode\s+\u2014\s+API compiled/,
  'runtime output must state that the API compiled the WASM binary',
);
requirePattern(
  playgroundCompiler,
  /Preview mode\s+\u2014\s+syntax\/demo fallback only/,
  'preview fallback must not imply certified compilation',
);
forbidPattern(
  playgroundCompiler,
  /case\s+MODE_WASM:\s+return\s+['"]WASM['"]/,
  'plain WASM label hides the server-backed compile dependency',
);
forbidPattern(
  playgroundCompiler,
  /\[WASM mode\s+[\u2014-]\s+compiled/,
  'plain WASM output hides the server-backed compile dependency',
);

const playgroundReadme = 'playground/README.md';
requireText(
  playgroundReadme,
  'Server-WASM mode',
  'playground README must name the server-backed WASM mode',
);
requireText(
  playgroundReadme,
  'does not include browser-only compilation or execution',
  'playground README must keep the browser-only non-claim explicit',
);
requireText(
  playgroundReadme,
  'this is not a certified compile/execute path',
  'preview mode must be documented as non-certified',
);
requireText(
  playgroundReadme,
  'test:contract',
  'playground README must document the local mode contract gate',
);
requireText(
  playgroundReadme,
  'Browser-JS mode',
  'playground README must document the browser JS smoke mode',
);
requireText(
  playgroundReadme,
  'without the playground API',
  'playground README must say Browser-JS smoke does not use the API',
);

const websiteSubtitle =
  'Try Vais syntax and examples in the browser. Real compilation uses the playground API; browser-only compile/execute remains experimental.';
requireText('website/index.html', websiteSubtitle, 'homepage playground copy must match public claim boundary');
requireText('website/public/locales/en.json', websiteSubtitle, 'English locale must match public claim boundary');
requireText('website/index.html', 'server runtime smoke 20/20', 'homepage server claim must match the promoted runtime gate');
requireText('website/index.html', 'shared-schema product gate 9/9', 'homepage web claim must name the shared-schema product gate');
requireText('website/ecosystem/index.html', 'server runtime 20/20', 'ecosystem server claim must match the promoted runtime gate');
requireText('website/ecosystem/index.html', 'shared-schema product 9/9 gate', 'ecosystem web claim must name the shared-schema product gate');
requireText('website/vaisx/index.html', 'shared-schema product gate 9/9', 'VaisX page must name the shared-schema product gate');
requireText('playground/src/examples.js', "'shared-schema-product'", 'playground must expose the shared-schema product example');

for (const locale of ['ko', 'ja', 'zh']) {
  const path = `website/public/locales/${locale}.json`;
  requireText(path, 'playground API', `${locale} locale must name the API dependency`);
  requireText(path, 'browser-only compile/execute', `${locale} locale must keep browser-only status experimental`);
}

for (const path of [
  'website/index.html',
  'website/public/locales/en.json',
  'website/public/locales/ko.json',
  'website/public/locales/ja.json',
  'website/public/locales/zh.json',
]) {
  forbidText(
    path,
    'Write and run Vais code directly in your browser. No installation required.',
    'homepage must not imply certified browser-only execution',
  );
  forbidText(
    path,
    'browser-only JS/WASM paths are experimental',
    'use the clearer compile/execute wording introduced by the claim boundary',
  );
}

if (failures.length > 0) {
  console.error('Public claim guard failed:');
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log(`Public claim guard passed (${relative(root, resolve(root)) || '.'}).`);
