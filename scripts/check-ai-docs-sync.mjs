#!/usr/bin/env node

import { existsSync, readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const registryPath = 'docs/ai/feature-registry.json';
const registry = JSON.parse(readFileSync(resolve(root, registryPath), 'utf8'));
const failures = [];

function fail(message) {
  failures.push(message);
}

function requireString(value, path) {
  if (typeof value !== 'string' || value.trim() === '') {
    fail(`${path} must be a non-empty string`);
  }
}

function requireStringArray(value, path) {
  if (!Array.isArray(value) || value.length === 0 || value.some((item) => typeof item !== 'string' || item.trim() === '')) {
    fail(`${path} must be a non-empty string array`);
  }
}

function pathMatches(path, pattern) {
  if (pattern.endsWith('/**')) {
    return path.startsWith(pattern.slice(0, -3));
  }
  return path === pattern;
}

function run(command, args, options = {}) {
  return spawnSync(command, args, {
    cwd: root,
    encoding: 'utf8',
    stdio: options.stdio ?? 'pipe',
  });
}

function validateRegistry() {
  if (registry.schemaVersion !== 1) {
    fail('schemaVersion must be 1');
  }
  requireString(registry.lastReviewed, 'lastReviewed');
  requireStringArray(registry.sourceOfTruth, 'sourceOfTruth');
  requireStringArray(registry.generatedDocs, 'generatedDocs');

  for (const path of [...registry.sourceOfTruth, ...registry.generatedDocs]) {
    if (!existsSync(resolve(root, path))) {
      fail(`${path} does not exist`);
    }
  }

  if (!Array.isArray(registry.changeTriggers) || registry.changeTriggers.length === 0) {
    fail('changeTriggers must be a non-empty array');
  } else {
    for (const [index, trigger] of registry.changeTriggers.entries()) {
      requireString(trigger.id, `changeTriggers[${index}].id`);
      requireStringArray(trigger.paths, `changeTriggers[${index}].paths`);
      requireString(trigger.reason, `changeTriggers[${index}].reason`);
    }
  }

  const ids = new Set();
  if (!Array.isArray(registry.features) || registry.features.length === 0) {
    fail('features must be a non-empty array');
    return;
  }

  for (const [index, feature] of registry.features.entries()) {
    const prefix = `features[${index}]`;
    requireString(feature.id, `${prefix}.id`);
    requireString(feature.title, `${prefix}.title`);
    requireString(feature.status, `${prefix}.status`);
    requireString(feature.summary, `${prefix}.summary`);
    requireStringArray(feature.aiInstructions, `${prefix}.aiInstructions`);
    requireStringArray(feature.mustUse, `${prefix}.mustUse`);
    requireStringArray(feature.avoid, `${prefix}.avoid`);
    if (ids.has(feature.id)) {
      fail(`duplicate feature id: ${feature.id}`);
    }
    ids.add(feature.id);
  }
}

function validateGeneratedDocs() {
  const generated = run(process.execPath, ['scripts/generate-ai-docs.mjs', '--check']);
  if (generated.status !== 0) {
    fail(generated.stderr.trim() || generated.stdout.trim() || 'generated AI docs are stale');
  }

  for (const feature of registry.features) {
    const found = registry.generatedDocs.some((path) =>
      readFileSync(resolve(root, path), 'utf8').includes(`Feature id: \`${feature.id}\``)
    );
    if (!found) {
      fail(`feature ${feature.id} is not present in generated AI docs`);
    }
  }
}

function changedFiles() {
  const baseRef = process.env.AI_DOCS_BASE_REF || (process.env.GITHUB_BASE_REF ? `origin/${process.env.GITHUB_BASE_REF}` : '');
  if (!baseRef) {
    return [];
  }

  const diff = run('git', ['diff', '--name-only', `${baseRef}...HEAD`]);
  if (diff.status !== 0) {
    console.warn(`AI docs changed-check skipped: could not diff against ${baseRef}`);
    return [];
  }
  return diff.stdout.split('\n').map((line) => line.trim()).filter(Boolean);
}

function validateChangedFiles() {
  if (!process.argv.includes('--changed-check')) {
    return;
  }

  const files = changedFiles();
  if (files.length === 0) {
    return;
  }

  const generatedAndRegistry = new Set([
    registryPath,
    ...registry.generatedDocs,
    'scripts/generate-ai-docs.mjs',
    'scripts/check-ai-docs-sync.mjs',
  ]);

  const aiDocsTouched = files.some((path) => generatedAndRegistry.has(path) || path.startsWith('docs/ai/'));
  const triggered = [];
  for (const trigger of registry.changeTriggers) {
    if (files.some((path) => trigger.paths.some((pattern) => pathMatches(path, pattern)))) {
      triggered.push(trigger);
    }
  }

  if (triggered.length > 0 && !aiDocsTouched) {
    fail(
      [
        'AI docs registry was not updated for an AI-visible behavior change.',
        `Changed files matched: ${triggered.map((trigger) => trigger.id).join(', ')}`,
        `Update ${registryPath}, run node scripts/generate-ai-docs.mjs, and commit the generated docs.`,
      ].join('\n'),
    );
  }
}

validateRegistry();
validateGeneratedDocs();
validateChangedFiles();

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(failure);
  }
  process.exit(1);
}

console.log('AI docs sync guard passed.');
