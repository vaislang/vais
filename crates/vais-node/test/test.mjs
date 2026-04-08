import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

// The native addon is built to the workspace target directory.
// When running `npm test` from crates/vais-node/, the workspace root is two levels up.
const addonPath = path.resolve(__dirname, '..', '..', '..', 'target', 'release', 'vais_node.node');
const vais = require(addonPath);

// Minimal valid Vais source: a function that adds two i64 values
const VALID_SOURCE = 'F add(a:i64,b:i64)->i64=a+b';
// A function returning a boolean literal
const BOOL_SOURCE = 'F is_true()->bool=true';
// Multiple functions
const MULTI_SOURCE = 'F foo()->i64=1 F bar()->i64=2';

// ─── tokenize() ──────────────────────────────────────────────────────────────

test('tokenize: returns an array for valid source', () => {
    const tokens = vais.tokenize(VALID_SOURCE);
    assert.ok(Array.isArray(tokens));
    assert.ok(tokens.length > 0);
});

test('tokenize: each token has tokenType, span.start, span.end', () => {
    const tokens = vais.tokenize(VALID_SOURCE);
    for (const tok of tokens) {
        assert.equal(typeof tok.tokenType, 'string');
        assert.ok(tok.tokenType.length > 0);
        assert.equal(typeof tok.span.start, 'number');
        assert.equal(typeof tok.span.end, 'number');
        assert.ok(tok.span.end >= tok.span.start);
    }
});

test('tokenize: first token of function declaration is Function', () => {
    const tokens = vais.tokenize(VALID_SOURCE);
    assert.equal(tokens[0].tokenType, 'Function');
});

test('tokenize: Ident tokens carry text', () => {
    const tokens = vais.tokenize(VALID_SOURCE);
    const identTokens = tokens.filter(t => t.tokenType === 'Ident');
    assert.ok(identTokens.length > 0);
    for (const t of identTokens) {
        assert.equal(typeof t.text, 'string');
        assert.ok(t.text.length > 0);
    }
});

// ─── parse() ─────────────────────────────────────────────────────────────────

test('parse: returns object with type and itemsCount for valid source', () => {
    const result = vais.parse(VALID_SOURCE);
    assert.equal(typeof result, 'object');
    assert.equal(result.type, 'Module');
    assert.equal(typeof result.itemsCount, 'number');
    assert.ok(result.itemsCount > 0);
});

test('parse: itemsCount matches number of top-level items', () => {
    const single = vais.parse(VALID_SOURCE);
    assert.equal(single.itemsCount, 1);

    const multi = vais.parse(MULTI_SOURCE);
    assert.equal(multi.itemsCount, 2);
});

test('parse: throws on syntax error', () => {
    assert.throws(() => {
        vais.parse('F (');
    }, /error/i);
});

// ─── check() ─────────────────────────────────────────────────────────────────

test('check: returns empty array for valid source', () => {
    const errors = vais.check(VALID_SOURCE);
    assert.ok(Array.isArray(errors));
    assert.equal(errors.length, 0);
});

test('check: returns error array for invalid source', () => {
    const errors = vais.check('F bad( = 1');
    assert.ok(Array.isArray(errors));
    assert.ok(errors.length > 0);
    assert.equal(typeof errors[0].message, 'string');
    assert.equal(typeof errors[0].errorType, 'string');
});

test('check: valid bool function produces no errors', () => {
    const errors = vais.check(BOOL_SOURCE);
    assert.equal(errors.length, 0);
});

// ─── compile() ───────────────────────────────────────────────────────────────

test('compile: returns a non-empty string for valid source', () => {
    const ir = vais.compile(VALID_SOURCE);
    assert.equal(typeof ir, 'string');
    assert.ok(ir.length > 0);
});

test('compile: output contains LLVM IR markers', () => {
    const ir = vais.compile(VALID_SOURCE);
    // LLVM IR always begins with a module-level declaration
    assert.ok(ir.includes('define') || ir.includes('declare') || ir.includes('; ModuleID'));
});

test('compile: accepts optLevel option', () => {
    const ir = vais.compile(VALID_SOURCE, { optLevel: 2 });
    assert.equal(typeof ir, 'string');
    assert.ok(ir.length > 0);
});

test('compile: accepts moduleName option', () => {
    const ir = vais.compile(VALID_SOURCE, { moduleName: 'test_module' });
    assert.equal(typeof ir, 'string');
    assert.ok(ir.length > 0);
});

test('compile: throws on parse error in source', () => {
    assert.throws(() => {
        vais.compile('F broken(');
    }, /error/i);
});
