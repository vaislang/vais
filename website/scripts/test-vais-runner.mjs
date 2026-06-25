import { runVaisSubset } from '../src/vais-runner.js';

const cases = [
  {
    name: 'edited arithmetic',
    expect: 45,
    source: `fn main() -> Int {
    return 40 + 5
}`,
  },
  {
    name: 'struct field access',
    expect: 42,
    source: `struct Box {
    value: Int,
}

fn main() -> Int {
    let b = Box { value: 42 }
    return b.value
}`,
  },
  {
    name: 'enum wildcard match',
    expect: 7,
    source: `enum Color { Red, Green, Blue }

fn score(c: Color) -> Int {
    match c {
        Color.Red => return 1,
        _ => return 7,
    }
}

fn main() -> Int {
    return score(Color.Blue)
}`,
  },
  {
    name: 'list push and index',
    expect: 23,
    source: `fn main() -> Int {
    let xs: List<Int> = []
    xs.push(10)
    xs.push(20)
    xs.push(30)
    return xs.len() + xs[1]
}`,
  },
  {
    name: 'while loop',
    expect: 10,
    source: `fn main() -> Int {
    let mut s = 0
    let mut i = 0
    while i < 5 {
        s = s + i
        i = i + 1
    }
    return s
}`,
  },
  {
    name: 'parse helpers',
    expect: 42,
    source: `fn main() -> Int {
    return parse_uint("16x") + parse_int("-5") + parse_int(\`31\`)
}`,
  },
  {
    name: 'Str conversion',
    expect: 42,
    source: `fn main() -> Int {
    let a = Str(42)
    let n = 0 - 7
    let b = Str(n)
    if a != "42" {
        return 1
    }
    if b != "-7" {
        return 2
    }
    return a.len() * 20 + b.len()
}`,
  },
];

let failed = false;

for (const testCase of cases) {
  const result = runVaisSubset(testCase.source);
  if (!result.ok || result.exitCode !== testCase.expect) {
    failed = true;
    console.error(`${testCase.name}: expected ${testCase.expect}, got ${JSON.stringify(result)}`);
  }
}

const errorCase = runVaisSubset('fn main() -> Int { return missing }');
if (errorCase.ok || !errorCase.error.includes('Unknown variable')) {
  failed = true;
  console.error(`error case: expected Unknown variable, got ${JSON.stringify(errorCase)}`);
}

if (failed) {
  process.exit(1);
}

console.log(`vais browser runner: ${cases.length + 1} checks passed`);
