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
  'VaisDB aggregate main full-build smoke',
  'VaisDB aggregate main full-build smoke must be named as main-reproducible evidence',
);
requireText(
  publicStatus,
  '`36/36` LLVM/object cache artifacts',
  'VaisDB aggregate main full-build smoke must disclose the measured module/cache scope',
);
requirePattern(
  publicStatus,
  /not yet\s+reproducible as a single `origin\/main` gate/,
  'remaining integration evidence must not be presented as one full ecosystem main gate',
);
requireText(
  publicStatus,
  'Main-scoped integrity runner:',
  'main-scoped integrity runner must be named without promoting full ecosystem runtime scope',
);
requireText(
  publicStatus,
  'Full ecosystem runtime aggregate runner: still pending a single',
  'full ecosystem runtime aggregate must remain explicitly pending as one main gate',
);
requireText(
  publicStatus,
  'Vais Server runtime smoke: `20/20`',
  'server public evidence count must remain explicit',
);
requireText(
  publicStatus,
  'Cross-package schema gate: `15/15`',
  'schema propagation must be named as an evidence-scoped public claim',
);
requireText(
  publicStatus,
  'Multi-domain product schema gate: `9/9`',
  'shared-schema product propagation must be named as evidence-scoped public claim',
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

const quickstart = 'docs/QUICKSTART.md';
requireText(quickstart, 'Functions use the canonical `fn` keyword', 'quickstart must present current function syntax');
requireText(quickstart, 'struct Point', 'quickstart must present current struct syntax');
forbidText(quickstart, 'F fib(n: i64)', 'quickstart must not use legacy single-letter function examples');
forbidText(quickstart, 'S Point', 'quickstart must not use legacy single-letter struct examples');

const fibonacciEpisode = 'docs/youtube-tutorials/episode-02-fibonacci.md';
requireText(fibonacciEpisode, 'fn fib(n: i64)', 'fibonacci tutorial must use current function syntax');
forbidText(fibonacciEpisode, 'F fib(n: i64)', 'fibonacci tutorial must not use legacy function syntax');

const websiteSubtitle =
  'Try Vais syntax and examples in the browser. Real compilation uses the playground API; browser-only compile/execute remains experimental.';
requireText('website/index.html', websiteSubtitle, 'homepage playground copy must match public claim boundary');
requireText('website/public/locales/en.json', websiteSubtitle, 'English locale must match public claim boundary');
requireText('website/index.html', 'Evidence Snapshot', 'homepage must scope gate counts as evidence');
requireText('website/index.html', 'VaisDB aggregate main full-build smoke', 'homepage must disclose the promoted aggregate main gate');
requireText('website/index.html', 'other runtime/package counts remain scoped evidence', 'homepage must preserve the DB/server/web scope boundary');
requireText('website/index.html', 'main-fixture/local-workspace reproducible', 'homepage must disclose schema gate main-fixture status');
requireText('website/index.html', 'server runtime integration evidence 20/20', 'homepage server claim must be evidence-scoped');
requireText('website/index.html', 'shared-schema product evidence 9/9', 'homepage web claim must be evidence-scoped');
requireText(
  'website/index.html',
  'Updated 2026-05-13',
  'homepage compile-speed benchmark date must reflect the current refresh',
);
requireText(
  'website/public/locales/en.json',
  'Updated 2026-05-13',
  'English locale compile-speed benchmark date must reflect the current refresh',
);
requireText(
  'website/index.html',
  '9.3x faster than C/clang and 15.6x faster than Rust',
  'homepage compile-speed ratios must match the refreshed benchmark',
);
requireText('website/ecosystem/index.html', 'server runtime 20/20', 'ecosystem server evidence count must remain explicit');
requireText('website/ecosystem/index.html', 'scoped evidence', 'ecosystem page must disclose evidence scope');
requireText('website/vaisx/index.html', 'shared-schema product evidence 9/9', 'VaisX page must be evidence-scoped');
requireText('playground/src/examples.js', "'shared-schema-product'", 'playground must expose the shared-schema product example');

requireText(
  'website/blog/performance-comparison.html',
  'Archive note (2026-05-13)',
  'historical performance article must not read as a current benchmark claim',
);
requireText(
  'website/blog/performance-comparison.html',
  'large-scale throughput numbers below should be rerun before citing them as current',
  'historical throughput data must be scoped as stale until rerun',
);
requireText(
  'website/blog/why-single-char-keywords.html',
  'Archive note, 2026-05-13',
  'single-character keyword rationale must remain explicitly archived',
);
requireText(
  'website/blog/why-vais.html',
  'The refreshed 2026-05-13 token benchmark reports',
  'why-vais token section must use the current scoped benchmark claim',
);
forbidText(
  'website/blog/why-vais.html',
  'Vais uses single-character keywords throughout',
  'why-vais page must not present historical single-character declarations as current syntax',
);
forbidText(
  'website/blog/index.html',
  'How Vais achieves 800K lines/sec',
  'blog index must not promote archived throughput as a current claim',
);

for (const locale of ['ko', 'ja', 'zh']) {
  const path = `website/public/locales/${locale}.json`;
  requireText(path, 'playground API', `${locale} locale must name the API dependency`);
  requireText(path, 'browser-only compile/execute', `${locale} locale must keep browser-only status experimental`);
  requireText(path, '2026-05-13', `${locale} locale compile-speed note must carry the current benchmark date`);
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
  forbidText(
    path,
    '2026-02-11',
    'homepage compile-speed note must not use the stale February benchmark date',
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
