import { runVaisSubset } from './vais-runner.js';

const samples = [
  {
    id: 'return',
    expect: 42,
    path: 'examples/t1.vais',
    title: { en: 'Return an Int', ko: 'Int 반환' },
    body: {
      en: 'Change the arithmetic expression in main and run the program. The result is the process exit code.',
      ko: 'main 안의 산술식을 바꾸고 프로그램을 실행하세요. 결과는 프로세스 종료 코드입니다.',
    },
    code: `# expect: 42
fn main() -> Int {
    return 40 + 2
}`,
  },
  {
    id: 'struct',
    expect: 42,
    path: 'examples/c4.vais',
    title: { en: 'Struct Field', ko: 'Struct 필드' },
    body: {
      en: 'Create a struct value and return one of its fields. Try changing Box.value.',
      ko: 'struct 값을 만들고 필드 하나를 반환합니다. Box.value 값을 바꿔보세요.',
    },
    code: `# expect: 42
struct Box {
    value: Int,
}

fn main() -> Int {
    let b = Box { value: 42 }
    return b.value
}`,
  },
  {
    id: 'enumWildcard',
    expect: 7,
    path: 'examples/e90_enum_wildcard.vais',
    title: { en: 'Enum Match', ko: 'Enum match' },
    body: {
      en: 'Use match with a wildcard arm. Change Color.Blue to Color.Red and compare the result.',
      ko: 'wildcard arm이 있는 match를 사용합니다. Color.Blue를 Color.Red로 바꿔 결과를 비교해보세요.',
    },
    code: `# expect: 7
enum Color { Red, Green, Blue }

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
    id: 'list',
    expect: 23,
    path: 'examples/e75_list_push.vais',
    title: { en: 'List Operations', ko: 'List 연산' },
    body: {
      en: 'Push values into a List<Int>, then combine len() and index access.',
      ko: 'List<Int>에 값을 push한 뒤 len()과 index 접근을 조합합니다.',
    },
    code: `# expect: 23
fn main() -> Int {
    let xs: List<Int> = []
    xs.push(10)
    xs.push(20)
    xs.push(30)
    return xs.len() + xs[1]
}`,
  },
  {
    id: 'loop',
    expect: 10,
    path: 'examples/e3.vais',
    title: { en: 'While Loop', ko: 'While loop' },
    body: {
      en: 'Accumulate values with a while loop. Change the bound and run again.',
      ko: 'while loop로 값을 누적합니다. 조건의 bound를 바꾸고 다시 실행해보세요.',
    },
    code: `# expect: 10
fn main() -> Int {
    let mut s = 0
    let mut i = 0
    while i < 5 {
        s = s + i
        i = i + 1
    }
    return s
}`,
  },
];

const messages = {
  en: {
    'top.home': 'Home',
    'problem.label': 'Problem',
    'problem.expected': 'Expected',
    'problem.file': 'Local file',
    'problem.runner': 'Runner',
    'problem.runnerValue': 'Browser tutorial subset',
    'problem.supportedTitle': 'Supported here',
    'problem.supported1': 'Int arithmetic, comparisons, if, while, return',
    'problem.supported2': 'Functions, simple structs, payload-free enum match',
    'problem.supported3': 'List push, len, index, last, pop, sum',
    'editor.title': 'Editor',
    'editor.run': 'Run',
    'editor.reset': 'Reset',
    'editor.copy': 'Copy',
    'result.label': 'Result',
    'result.idle': 'Ready',
    'result.case': 'Exit-code test',
    'result.expected': 'Expected',
    'result.actual': 'Actual',
    'result.output': 'Output',
    'result.local': 'Full compiler',
    pass: 'Pass',
    fail: 'Fail',
    copied: 'Copied',
    copyFailed: 'Copy failed',
    idleOutput: 'Edit main.vais, then run the program.',
    runCommand: '$ browser Vais runner',
    source: 'source',
    selectedSample: 'selected sample',
    editedSource: 'edited source',
    expected: 'expected',
    actual: 'actual',
    error: 'error',
    note: 'For full Vais behavior, save the code as a .vais file and run scripts/vaisc locally.',
  },
  ko: {
    'top.home': '홈',
    'problem.label': '문제',
    'problem.expected': '기대값',
    'problem.file': '로컬 파일',
    'problem.runner': '실행기',
    'problem.runnerValue': '브라우저 튜토리얼 subset',
    'problem.supportedTitle': '여기서 지원',
    'problem.supported1': 'Int 산술, 비교, if, while, return',
    'problem.supported2': '함수, 단순 struct, payload-free enum match',
    'problem.supported3': 'List push, len, index, last, pop, sum',
    'editor.title': '에디터',
    'editor.run': '실행',
    'editor.reset': '초기화',
    'editor.copy': '복사',
    'result.label': '결과',
    'result.idle': '대기',
    'result.case': '종료 코드 테스트',
    'result.expected': '기대값',
    'result.actual': '실제값',
    'result.output': '출력',
    'result.local': '전체 컴파일러',
    pass: '통과',
    fail: '실패',
    copied: '복사됨',
    copyFailed: '복사 실패',
    idleOutput: 'main.vais를 수정한 뒤 프로그램을 실행하세요.',
    runCommand: '$ browser Vais runner',
    source: '소스',
    selectedSample: '선택 예제',
    editedSource: '수정한 소스',
    expected: '기대값',
    actual: '실제값',
    error: '오류',
    note: '전체 Vais 동작은 코드를 .vais 파일로 저장한 뒤 로컬 scripts/vaisc로 실행하세요.',
  },
};

const storedLang = localStorage.getItem('vais-lang');
let currentLang = ['en', 'ko'].includes(storedLang) ? storedLang : 'en';
let currentSample = samples[0];

const sampleSelect = document.getElementById('pg-sample');
const title = document.getElementById('pg-problem-title');
const body = document.getElementById('pg-problem-body');
const expected = document.getElementById('pg-expected');
const path = document.getElementById('pg-path');
const editor = document.getElementById('pg-editor');
const runButton = document.getElementById('pg-run');
const resetButton = document.getElementById('pg-reset');
const copyButton = document.getElementById('pg-copy');
const status = document.getElementById('pg-status');
const testState = document.getElementById('pg-test-state');
const testExpected = document.getElementById('pg-test-expected');
const testActual = document.getElementById('pg-test-actual');
const output = document.querySelector('#pg-output code');
const localCommand = document.getElementById('pg-local-command');

function t(key) {
  return messages[currentLang][key] || messages.en[key] || key;
}

function renderI18n() {
  document.documentElement.lang = currentLang;
  document.querySelectorAll('[data-i18n]').forEach((node) => {
    node.textContent = t(node.getAttribute('data-i18n'));
  });
  document.querySelectorAll('.lang-btn').forEach((button) => {
    button.classList.toggle('active', button.dataset.lang === currentLang);
  });
  renderSamples();
  renderProblem(false);
  if (status.dataset.state === 'pass') {
    status.textContent = t('pass');
    testState.textContent = t('pass');
  } else if (status.dataset.state === 'fail') {
    status.textContent = t('fail');
    testState.textContent = t('fail');
  } else {
    status.textContent = t('result.idle');
    output.textContent = t('idleOutput');
  }
}

function renderSamples() {
  sampleSelect.innerHTML = '';
  samples.forEach((sample) => {
    const option = document.createElement('option');
    option.value = sample.id;
    option.textContent = sample.title[currentLang];
    sampleSelect.appendChild(option);
  });
  sampleSelect.value = currentSample.id;
}

function renderProblem(resetSource = true) {
  title.textContent = currentSample.title[currentLang];
  body.textContent = currentSample.body[currentLang];
  expected.textContent = String(currentSample.expect);
  testExpected.textContent = String(currentSample.expect);
  path.textContent = currentSample.path;
  localCommand.textContent = `scripts/vaisc run ${currentSample.path}`;
  if (resetSource) editor.value = currentSample.code;
}

function setIdle() {
  status.dataset.state = 'idle';
  status.textContent = t('result.idle');
  testState.textContent = '-';
  testActual.textContent = '-';
  output.textContent = t('idleOutput');
}

function runCurrentSource() {
  const result = runVaisSubset(editor.value);
  const sourceKind = normalizeSource(editor.value) === normalizeSource(currentSample.code)
    ? t('selectedSample')
    : t('editedSource');

  if (!result.ok) {
    status.dataset.state = 'fail';
    status.textContent = t('fail');
    testState.textContent = t('fail');
    testActual.textContent = '-';
    output.textContent = `${t('runCommand')}
${t('source')}: ${sourceKind}
${t('error')}: ${result.error}

${t('note')}`;
    return;
  }

  const passed = result.exitCode === currentSample.expect;
  status.dataset.state = passed ? 'pass' : 'fail';
  status.textContent = passed ? t('pass') : t('fail');
  testState.textContent = status.textContent;
  testActual.textContent = String(result.exitCode);
  output.textContent = `${t('runCommand')}
${t('source')}: ${sourceKind}
${t('actual')}: ${result.exitCode}
${t('expected')}: ${currentSample.expect}

${t('note')}`;
}

function normalizeSource(source) {
  return source.replace(/\s+/g, ' ').trim();
}

document.querySelectorAll('.lang-btn').forEach((button) => {
  button.addEventListener('click', () => {
    currentLang = button.dataset.lang || 'en';
    localStorage.setItem('vais-lang', currentLang);
    renderI18n();
  });
});

sampleSelect.addEventListener('change', () => {
  currentSample = samples.find((sample) => sample.id === sampleSelect.value) || samples[0];
  renderProblem(true);
  setIdle();
});

runButton.addEventListener('click', runCurrentSource);

resetButton.addEventListener('click', () => {
  renderProblem(true);
  setIdle();
});

copyButton.addEventListener('click', async () => {
  try {
    await navigator.clipboard.writeText(editor.value);
    const original = copyButton.textContent;
    copyButton.textContent = t('copied');
    setTimeout(() => {
      copyButton.textContent = original;
    }, 1400);
  } catch {
    copyButton.textContent = t('copyFailed');
    setTimeout(() => {
      copyButton.textContent = t('editor.copy');
    }, 1400);
  }
});

editor.addEventListener('keydown', (event) => {
  if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
    event.preventDefault();
    runCurrentSource();
  }
});

renderSamples();
renderProblem(true);
renderI18n();
setIdle();
