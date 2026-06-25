const KEYWORDS = new Set([
  'fn',
  'struct',
  'enum',
  'let',
  'mut',
  'return',
  'if',
  'else',
  'while',
  'match',
  'true',
  'false',
  'not',
]);

class RunnerError extends Error {}

export function runVaisSubset(source) {
  try {
    const parser = new Parser(tokenize(source));
    const program = parser.parseProgram();
    const runtime = new Runtime(program);
    const value = runtime.callFunction('main', []);
    return { ok: true, exitCode: toExitCode(value) };
  } catch (error) {
    return {
      ok: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

function tokenize(source) {
  const tokens = [];
  let index = 0;
  let line = 1;
  let column = 1;

  const push = (type, value, startLine = line, startColumn = column) => {
    tokens.push({ type, value, line: startLine, column: startColumn });
  };

  const advance = () => {
    const char = source[index++];
    if (char === '\n') {
      line += 1;
      column = 1;
    } else {
      column += 1;
    }
    return char;
  };

  while (index < source.length) {
    const char = source[index];

    if (char === '#') {
      while (index < source.length && source[index] !== '\n') advance();
      continue;
    }

    if (/\s/.test(char)) {
      advance();
      continue;
    }

    const startLine = line;
    const startColumn = column;

    if (/[A-Za-z_]/.test(char)) {
      let text = '';
      while (index < source.length && /[A-Za-z0-9_]/.test(source[index])) {
        text += advance();
      }
      push(KEYWORDS.has(text) ? 'keyword' : 'identifier', text, startLine, startColumn);
      continue;
    }

    if (/[0-9]/.test(char)) {
      let text = '';
      while (index < source.length && /[0-9]/.test(source[index])) {
        text += advance();
      }
      push('number', text, startLine, startColumn);
      continue;
    }

    if (char === '"' || char === '`') {
      const quote = advance();
      let text = '';
      while (index < source.length && source[index] !== quote) {
        if (source[index] === '\\' && quote === '"') {
          advance();
          const escaped = advance();
          text += escaped === 'n' ? '\n' : escaped === 't' ? '\t' : escaped;
          continue;
        }
        text += advance();
      }
      if (source[index] !== quote) {
        throw new RunnerError(`Unterminated string at ${startLine}:${startColumn}`);
      }
      advance();
      push('string', text, startLine, startColumn);
      continue;
    }

    const two = source.slice(index, index + 2);
    if (['->', '=>', '==', '!=', '<=', '>=', '&&', '||'].includes(two)) {
      advance();
      advance();
      push('symbol', two, startLine, startColumn);
      continue;
    }

    if ('{}()[],:.;+-*/%<>=!.'.includes(char)) {
      push('symbol', advance(), startLine, startColumn);
      continue;
    }

    throw new RunnerError(`Unexpected character '${char}' at ${startLine}:${startColumn}`);
  }

  tokens.push({ type: 'eof', value: '<eof>', line, column });
  return tokens;
}

class Parser {
  constructor(tokens) {
    this.tokens = tokens;
    this.position = 0;
  }

  parseProgram() {
    const program = {
      structs: new Map(),
      enums: new Map(),
      functions: new Map(),
    };

    while (!this.is('<eof>')) {
      if (this.match('struct')) {
        const declaration = this.parseStruct();
        program.structs.set(declaration.name, declaration);
      } else if (this.match('enum')) {
        const declaration = this.parseEnum();
        program.enums.set(declaration.name, declaration);
      } else if (this.match('fn')) {
        const declaration = this.parseFunction();
        program.functions.set(declaration.name, declaration);
      } else {
        this.fail(this.current(), 'Expected struct, enum, or fn declaration');
      }
    }

    if (!program.functions.has('main')) {
      this.fail(this.current(), 'Expected fn main()');
    }

    return program;
  }

  parseStruct() {
    const name = this.expectIdentifier('Expected struct name');
    const fields = [];
    this.expect('{');
    while (!this.match('}')) {
      const field = this.expectIdentifier('Expected struct field name');
      this.expect(':');
      this.skipTypeUntil([',', '}']);
      fields.push(field);
      this.match(',');
    }
    return { name, fields };
  }

  parseEnum() {
    const name = this.expectIdentifier('Expected enum name');
    const variants = [];
    this.expect('{');
    while (!this.match('}')) {
      const variant = this.expectIdentifier('Expected enum variant name');
      variants.push(variant);
      if (this.match('(')) {
        this.skipBalanced('(', ')');
      }
      this.match(',');
    }
    return { name, variants };
  }

  parseFunction() {
    const name = this.expectIdentifier('Expected function name');
    const params = [];
    this.expect('(');
    while (!this.match(')')) {
      const paramName = this.expectIdentifier('Expected parameter name');
      if (this.match(':')) {
        this.skipTypeUntil([',', ')']);
      }
      params.push(paramName);
      if (!this.match(',')) {
        this.expect(')');
        break;
      }
    }

    if (this.match('->')) {
      this.skipTypeUntil(['{']);
    }

    const body = this.parseBlock();
    return { name, params, body };
  }

  parseBlock() {
    const statements = [];
    this.expect('{');
    while (!this.match('}')) {
      if (this.is('<eof>')) this.fail(this.current(), 'Unclosed block');
      statements.push(this.parseStatement());
      this.match(';');
    }
    return statements;
  }

  parseStatement() {
    if (this.match('return')) {
      return { kind: 'return', value: this.parseExpression() };
    }

    if (this.match('let')) {
      this.match('mut');
      const name = this.expectIdentifier('Expected variable name');
      if (this.match(':')) {
        this.skipTypeUntil(['=']);
      }
      this.expect('=');
      return { kind: 'let', name, value: this.parseExpression() };
    }

    if (this.match('while')) {
      const condition = this.parseExpression();
      const body = this.parseBlock();
      return { kind: 'while', condition, body };
    }

    if (this.match('if')) {
      return this.parseIfStatement();
    }

    if (this.match('match')) {
      return this.parseMatchStatement();
    }

    const expression = this.parseExpression();
    if (expression.kind === 'name' && this.match('=')) {
      return { kind: 'assign', name: expression.name, value: this.parseExpression() };
    }
    return { kind: 'expr', expression };
  }

  parseIfStatement() {
    const condition = this.parseExpression();
    const consequent = this.parseBlock();
    let alternate = null;
    if (this.match('else')) {
      alternate = this.match('if') ? [this.parseIfStatement()] : this.parseBlock();
    }
    return { kind: 'if', condition, consequent, alternate };
  }

  parseMatchStatement() {
    const target = this.parseExpression();
    const arms = [];
    this.expect('{');
    while (!this.match('}')) {
      const pattern = this.parsePattern();
      this.expect('=>');
      arms.push({ pattern, statement: this.parseStatement() });
      this.match(',');
    }
    return { kind: 'match', target, arms };
  }

  parsePattern() {
    if (this.match('_')) return { kind: 'wildcard' };
    return { kind: 'value', value: this.parseExpression() };
  }

  parseExpression(minPrecedence = 0) {
    let left = this.parseUnary();

    while (true) {
      const operator = this.current().value;
      const precedence = binaryPrecedence(operator);
      if (precedence < minPrecedence) break;
      this.advance();
      const right = this.parseExpression(precedence + 1);
      left = { kind: 'binary', operator, left, right };
    }

    return left;
  }

  parseUnary() {
    if (this.match('-')) {
      return { kind: 'unary', operator: '-', value: this.parseUnary() };
    }
    if (this.match('!') || this.match('not')) {
      return { kind: 'unary', operator: 'not', value: this.parseUnary() };
    }
    return this.parsePostfix();
  }

  parsePostfix() {
    let expression = this.parsePrimary();

    while (true) {
      if (this.match('(')) {
        const args = this.parseArgumentsAfterOpenParen();
        expression = { kind: 'call', callee: expression, args };
      } else if (this.match('[')) {
        const index = this.parseExpression();
        this.expect(']');
        expression = { kind: 'index', target: expression, index };
      } else if (this.match('.')) {
        const field = this.expectIdentifier('Expected field or method name');
        if (this.match('(')) {
          const args = this.parseArgumentsAfterOpenParen();
          expression = { kind: 'methodCall', target: expression, method: field, args };
        } else {
          expression = { kind: 'field', target: expression, field };
        }
      } else {
        break;
      }
    }

    return expression;
  }

  parsePrimary() {
    const token = this.current();

    if (token.type === 'number') {
      this.advance();
      return { kind: 'number', value: Number(token.value) };
    }

    if (token.type === 'string') {
      this.advance();
      return { kind: 'string', value: token.value };
    }

    if (this.match('true')) return { kind: 'bool', value: true };
    if (this.match('false')) return { kind: 'bool', value: false };

    if (this.match('(')) {
      const expression = this.parseExpression();
      this.expect(')');
      return expression;
    }

    if (this.match('[')) {
      const items = [];
      while (!this.match(']')) {
        items.push(this.parseExpression());
        if (!this.match(',')) {
          this.expect(']');
          break;
        }
      }
      return { kind: 'list', items };
    }

    if (token.type === 'identifier') {
      const name = token.value;
      this.advance();
      if (startsWithUppercase(name) && this.match('{')) {
        const fields = {};
        while (!this.match('}')) {
          const field = this.expectIdentifier('Expected struct field name');
          this.expect(':');
          fields[field] = this.parseExpression();
          if (!this.match(',')) {
            this.expect('}');
            break;
          }
        }
        return { kind: 'structLiteral', name, fields };
      }
      return { kind: 'name', name };
    }

    this.fail(token, 'Expected expression');
  }

  parseArgumentsAfterOpenParen() {
    const args = [];
    while (!this.match(')')) {
      args.push(this.parseExpression());
      if (!this.match(',')) {
        this.expect(')');
        break;
      }
    }
    return args;
  }

  skipTypeUntil(stopValues) {
    let angleDepth = 0;
    let parenDepth = 0;
    let bracketDepth = 0;

    while (!this.is('<eof>')) {
      const value = this.current().value;
      const atBoundary =
        angleDepth === 0 &&
        parenDepth === 0 &&
        bracketDepth === 0 &&
        stopValues.includes(value);
      if (atBoundary) return;

      if (value === '<') angleDepth += 1;
      else if (value === '>' && angleDepth > 0) angleDepth -= 1;
      else if (value === '(') parenDepth += 1;
      else if (value === ')' && parenDepth > 0) parenDepth -= 1;
      else if (value === '[') bracketDepth += 1;
      else if (value === ']' && bracketDepth > 0) bracketDepth -= 1;
      this.advance();
    }
  }

  skipBalanced(open, close) {
    let depth = 1;
    while (depth > 0 && !this.is('<eof>')) {
      if (this.match(open)) depth += 1;
      else if (this.match(close)) depth -= 1;
      else this.advance();
    }
  }

  match(value) {
    if (!this.is(value)) return false;
    this.advance();
    return true;
  }

  is(value) {
    return this.current().value === value;
  }

  expect(value) {
    if (!this.match(value)) this.fail(this.current(), `Expected '${value}'`);
  }

  expectIdentifier(message) {
    const token = this.current();
    if (token.type !== 'identifier') this.fail(token, message);
    this.advance();
    return token.value;
  }

  current() {
    return this.tokens[this.position];
  }

  advance() {
    const token = this.current();
    this.position += 1;
    return token;
  }

  fail(token, message) {
    throw new RunnerError(`${message} at ${token.line}:${token.column}`);
  }
}

class Runtime {
  constructor(program) {
    this.program = program;
    this.callDepth = 0;
  }

  callFunction(name, args) {
    const declaration = this.program.functions.get(name);
    if (!declaration) {
      return this.callBuiltin(name, args);
    }

    if (this.callDepth > 500) {
      throw new RunnerError('Call depth limit exceeded');
    }

    this.callDepth += 1;
    const scope = new Scope();
    declaration.params.forEach((param, index) => {
      scope.define(param, args[index] ?? 0);
    });

    const result = this.executeBlock(declaration.body, scope);
    this.callDepth -= 1;
    return result?.kind === 'return' ? result.value : 0;
  }

  executeBlock(statements, parentScope) {
    const scope = new Scope(parentScope);
    for (const statement of statements) {
      const result = this.executeStatement(statement, scope);
      if (result?.kind === 'return') return result;
    }
    return null;
  }

  executeStatement(statement, scope) {
    switch (statement.kind) {
      case 'return':
        return { kind: 'return', value: this.evaluate(statement.value, scope) };
      case 'let':
        scope.define(statement.name, this.evaluate(statement.value, scope));
        return null;
      case 'assign':
        scope.set(statement.name, this.evaluate(statement.value, scope));
        return null;
      case 'expr':
        this.evaluate(statement.expression, scope);
        return null;
      case 'if': {
        const branch = this.isTruthy(this.evaluate(statement.condition, scope))
          ? statement.consequent
          : statement.alternate;
        return branch ? this.executeBlock(branch, scope) : null;
      }
      case 'while': {
        let iterations = 0;
        while (this.isTruthy(this.evaluate(statement.condition, scope))) {
          if (iterations++ > 100000) {
            throw new RunnerError('Loop iteration limit exceeded');
          }
          const result = this.executeBlock(statement.body, scope);
          if (result?.kind === 'return') return result;
        }
        return null;
      }
      case 'match': {
        const target = this.evaluate(statement.target, scope);
        for (const arm of statement.arms) {
          if (this.matchesPattern(target, arm.pattern, scope)) {
            return this.executeStatement(arm.statement, new Scope(scope));
          }
        }
        throw new RunnerError('No match arm matched');
      }
      default:
        throw new RunnerError(`Unsupported statement '${statement.kind}'`);
    }
  }

  evaluate(expression, scope) {
    switch (expression.kind) {
      case 'number':
      case 'string':
      case 'bool':
        return expression.value;
      case 'list':
        return expression.items.map((item) => this.evaluate(item, scope));
      case 'structLiteral':
        return this.evaluateStructLiteral(expression, scope);
      case 'name':
        return scope.get(expression.name);
      case 'field':
        return this.evaluateField(expression, scope);
      case 'index':
        return this.evaluateIndex(expression, scope);
      case 'call':
        return this.evaluateCall(expression, scope);
      case 'methodCall':
        return this.evaluateMethodCall(expression, scope);
      case 'unary':
        return this.evaluateUnary(expression, scope);
      case 'binary':
        return this.evaluateBinary(expression, scope);
      default:
        throw new RunnerError(`Unsupported expression '${expression.kind}'`);
    }
  }

  evaluateStructLiteral(expression, scope) {
    if (!this.program.structs.has(expression.name)) {
      throw new RunnerError(`Unknown struct '${expression.name}'`);
    }
    const value = { __struct: expression.name };
    Object.entries(expression.fields).forEach(([field, fieldExpression]) => {
      value[field] = this.evaluate(fieldExpression, scope);
    });
    return value;
  }

  evaluateField(expression, scope) {
    if (expression.target.kind === 'name') {
      const enumDeclaration = this.program.enums.get(expression.target.name);
      if (enumDeclaration?.variants.includes(expression.field)) {
        return { __enum: expression.target.name, variant: expression.field };
      }
    }

    const target = this.evaluate(expression.target, scope);
    if (target && typeof target === 'object' && expression.field in target) {
      return target[expression.field];
    }
    throw new RunnerError(`Unknown field '${expression.field}'`);
  }

  evaluateIndex(expression, scope) {
    const target = this.evaluate(expression.target, scope);
    const index = this.evaluate(expression.index, scope);
    if (!Number.isInteger(index)) throw new RunnerError('Index must be an Int');
    if ((Array.isArray(target) || typeof target === 'string') && index >= 0 && index < target.length) {
      return target[index];
    }
    throw new RunnerError('Index out of range');
  }

  evaluateCall(expression, scope) {
    if (expression.callee.kind !== 'name') {
      throw new RunnerError('Only named function calls are supported in the browser runner');
    }
    const args = expression.args.map((arg) => this.evaluate(arg, scope));
    return this.callFunction(expression.callee.name, args);
  }

  evaluateMethodCall(expression, scope) {
    const target = this.evaluate(expression.target, scope);
    const args = expression.args.map((arg) => this.evaluate(arg, scope));
    return this.callMethod(target, expression.method, args);
  }

  evaluateUnary(expression, scope) {
    const value = this.evaluate(expression.value, scope);
    if (expression.operator === '-') return -expectNumber(value, 'Unary -');
    if (expression.operator === 'not') return !this.isTruthy(value);
    throw new RunnerError(`Unsupported unary operator '${expression.operator}'`);
  }

  evaluateBinary(expression, scope) {
    if (expression.operator === '&&') {
      return this.isTruthy(this.evaluate(expression.left, scope)) && this.isTruthy(this.evaluate(expression.right, scope));
    }
    if (expression.operator === '||') {
      return this.isTruthy(this.evaluate(expression.left, scope)) || this.isTruthy(this.evaluate(expression.right, scope));
    }

    const left = this.evaluate(expression.left, scope);
    const right = this.evaluate(expression.right, scope);

    switch (expression.operator) {
      case '+':
        return typeof left === 'string' || typeof right === 'string'
          ? String(left) + String(right)
          : expectNumber(left, '+') + expectNumber(right, '+');
      case '-':
        return expectNumber(left, '-') - expectNumber(right, '-');
      case '*':
        return expectNumber(left, '*') * expectNumber(right, '*');
      case '/':
        return Math.trunc(expectNumber(left, '/') / expectNumber(right, '/'));
      case '%':
        return expectNumber(left, '%') % expectNumber(right, '%');
      case '==':
        return valueEquals(left, right);
      case '!=':
        return !valueEquals(left, right);
      case '<':
        return expectNumber(left, '<') < expectNumber(right, '<');
      case '<=':
        return expectNumber(left, '<=') <= expectNumber(right, '<=');
      case '>':
        return expectNumber(left, '>') > expectNumber(right, '>');
      case '>=':
        return expectNumber(left, '>=') >= expectNumber(right, '>=');
      default:
        throw new RunnerError(`Unsupported binary operator '${expression.operator}'`);
    }
  }

  callBuiltin(name, args) {
    switch (name) {
      case 'list':
        return [];
      case 'parse_int':
        return parseSignedInt(args[0]);
      case 'parse_uint':
        return Math.max(0, parseUnsignedInt(args[0]));
      case 'Str':
        return String(expectNumber(args[0], 'Str'));
      default:
        throw new RunnerError(`Unknown function '${name}'`);
    }
  }

  callMethod(target, method, args) {
    if (Array.isArray(target)) {
      switch (method) {
        case 'push':
          target.push(args[0]);
          return 0;
        case 'len':
          return target.length;
        case 'is_empty':
          return target.length === 0;
        case 'last':
          if (target.length === 0) throw new RunnerError('last() on empty list');
          return target[target.length - 1];
        case 'pop':
          if (target.length === 0) throw new RunnerError('pop() on empty list');
          return target.pop();
        case 'sum':
          return target.reduce((sum, item) => sum + expectNumber(item, 'sum'), 0);
        default:
          break;
      }
    }

    if (typeof target === 'string' && method === 'len') {
      return target.length;
    }

    throw new RunnerError(`Unsupported method '${method}'`);
  }

  matchesPattern(target, pattern, scope) {
    if (pattern.kind === 'wildcard') return true;
    return valueEquals(target, this.evaluate(pattern.value, scope));
  }

  isTruthy(value) {
    return typeof value === 'boolean' ? value : value !== 0;
  }
}

class Scope {
  constructor(parent = null) {
    this.parent = parent;
    this.values = new Map();
  }

  define(name, value) {
    this.values.set(name, value);
  }

  get(name) {
    if (this.values.has(name)) return this.values.get(name);
    if (this.parent) return this.parent.get(name);
    throw new RunnerError(`Unknown variable '${name}'`);
  }

  set(name, value) {
    if (this.values.has(name)) {
      this.values.set(name, value);
      return;
    }
    if (this.parent) {
      this.parent.set(name, value);
      return;
    }
    throw new RunnerError(`Unknown variable '${name}'`);
  }
}

function binaryPrecedence(operator) {
  switch (operator) {
    case '||':
      return 1;
    case '&&':
      return 2;
    case '==':
    case '!=':
      return 3;
    case '<':
    case '<=':
    case '>':
    case '>=':
      return 4;
    case '+':
    case '-':
      return 5;
    case '*':
    case '/':
    case '%':
      return 6;
    default:
      return -1;
  }
}

function startsWithUppercase(name) {
  return /^[A-Z]/.test(name);
}

function expectNumber(value, context) {
  if (typeof value !== 'number' || !Number.isFinite(value)) {
    throw new RunnerError(`${context} expects Int values`);
  }
  return value;
}

function parseSignedInt(value) {
  const match = String(value ?? '').match(/^[+-]?\d+/);
  return match ? Number(match[0]) : 0;
}

function parseUnsignedInt(value) {
  const match = String(value ?? '').match(/^\d+/);
  return match ? Number(match[0]) : 0;
}

function valueEquals(left, right) {
  if (left && right && typeof left === 'object' && typeof right === 'object') {
    if (left.__enum || right.__enum) {
      return left.__enum === right.__enum && left.variant === right.variant;
    }
  }
  return left === right;
}

function toExitCode(value) {
  if (typeof value === 'boolean') return value ? 1 : 0;
  if (!Number.isInteger(value)) {
    throw new RunnerError('main() must return an Int for the browser runner');
  }
  return value;
}
