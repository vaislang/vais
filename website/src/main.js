import { runVaisSubset } from './vais-runner.js';

const samples = [
  {
    id: 'return',
    expect: 42,
    title: { en: 'Return an Int', ko: 'Int 반환' },
    lesson: {
      en: 'A Vais program starts at fn main() -> Int and returns a process exit code.',
      ko: 'Vais 프로그램은 fn main() -> Int에서 시작하고 프로세스 종료 코드를 반환합니다.',
    },
    path: 'examples/t1.vais',
    code: `# expect: 42
fn main() -> Int {
    return 40 + 2
}`,
  },
  {
    id: 'struct',
    expect: 42,
    title: { en: 'Struct field access', ko: 'Struct 필드 접근' },
    lesson: {
      en: 'Simple structs, literals, and field access are part of the verified release surface.',
      ko: '단순 struct 선언, literal 생성, 필드 접근은 검증된 release surface에 포함됩니다.',
    },
    path: 'examples/c4.vais',
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
    title: { en: 'Enum match wildcard', ko: 'Enum match wildcard' },
    lesson: {
      en: 'Payload-free enum match can use a _ catch-all arm in the release corpus.',
      ko: 'payload-free enum match는 release corpus 안에서 _ catch-all arm을 사용할 수 있습니다.',
    },
    path: 'examples/e90_enum_wildcard.vais',
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
    title: { en: 'List push and index', ko: 'List push와 index' },
    lesson: {
      en: 'List<Int> local operations are verified for push, len, index, and related methods.',
      ko: 'List<Int> local 연산은 push, len, index 및 관련 method 범위에서 검증되어 있습니다.',
    },
    path: 'examples/e75_list_push.vais',
    code: `# expect: 23
fn main() -> Int {
    let xs: List<Int> = []
    xs.push(10)
    xs.push(20)
    xs.push(30)
    return xs.len() + xs[1]
}`,
  },
];

const messages = {
  en: {
    'nav.playground': 'Playground',
    'nav.learn': 'Learn',
    'nav.language': 'Language',
    'nav.compiler': 'Compiler',
    'nav.verification': 'Verification',
    'hero.eyebrow': 'Current Vais',
    'hero.title': 'Learn Vais by running verified examples.',
    'hero.body': 'Vais is a .vais language and self-host compiler workspace. The public compiler command is scripts/vaisc, and public claims are tied to repository gates.',
    'hero.github': 'Open GitHub',
    'hero.start': 'Start learning',
    'signal.ext.label': 'Source',
    'signal.ext.value': '.vais files only',
    'signal.cmd.label': 'Command',
    'signal.gate.label': 'Release corpus',
    'signal.gate.value': '49 verified examples',
    'playground.eyebrow': 'Playground',
    'playground.title': 'Editable browser runner',
    'playground.editorLabel': 'Source',
    'playground.run': 'Run code',
    'playground.reset': 'Reset',
    'playground.copy': 'Copy source',
    'playground.output': 'Output',
    'playground.mode': 'browser subset',
    'learn.eyebrow': 'Learning path',
    'learn.title': 'A shorter path from GitHub to a running program.',
    'learn.body': 'The project has one public compiler path and one public checker path. These are the commands to keep in front of new readers.',
    'learn.clone.title': 'Clone and check the toolchain',
    'learn.run.title': 'Run a verified example',
    'learn.build.title': 'Build native output',
    'language.eyebrow': 'Language',
    'language.title': 'Current syntax is intentionally small and gate-backed.',
    'language.body': 'The public surface uses full-word syntax: fn, let, if, while, for, struct, enum, match, and return. Older compact or Rust-like examples are stale for this mainline.',
    'language.scalar.title': 'Scalars',
    'language.scalar.body': 'Int, Bool, Char, and Str are verified across promoted paths where documented.',
    'language.collections.title': 'Collections',
    'language.collections.body': 'List<Int>, List<Struct>, and local Map<Int,Int> slices are verified. Broader generics, Option, and Result remain future work.',
    'language.match.title': 'Enums and match',
    'language.match.body': 'Payload-free enums, Int payload enums, wildcard match arms, and selected struct payload cases are protected by the release corpus.',
    'language.modules.title': 'Imports',
    'language.modules.body': 'The full engine supports local dotted imports and local dependency package paths through vais.toml.',
    'language.reference': 'Language reference',
    'language.examples': 'Examples corpus',
    'compiler.eyebrow': 'Compiler',
    'compiler.title': 'The public compiler is scripts/vaisc.',
    'compiler.body': 'The native public driver links the reusable self-host compiler core, emits LLVM IR, and uses clang for native binaries. Standalone installs are built from this repository.',
    'verification.eyebrow': 'Verification',
    'verification.title': 'Public claims come from gates, not promises.',
    'verification.body': 'The site and GitHub README should stay synced to these checks. If a feature is not in the gates or parity manifest, it should not be advertised as current Vais.',
    'footer.summary': 'Vais language and self-host compiler workspace.',
    'footer.docs': 'Docs',
    'footer.selfhost': 'Self-host',
    copy: 'Copy',
    copied: 'Copied',
    copyFailed: 'Copy failed',
    outputFor: 'source',
    expected: 'expected exit code',
    exitCode: 'exit code',
    sampleSource: 'selected sample',
    editedSource: 'edited source',
    ready: 'Edit the source, then run code.',
    runnerError: 'runner error',
    command: 'local compiler command',
    note: 'The browser runner executes a small tutorial subset of Vais for immediate feedback. Use scripts/vaisc locally for the full compiler.',
  },
  ko: {
    'nav.playground': '플레이그라운드',
    'nav.learn': '배우기',
    'nav.language': '언어',
    'nav.compiler': '컴파일러',
    'nav.verification': '검증',
    'hero.eyebrow': '현재 Vais',
    'hero.title': '검증된 예제를 실행하며 Vais를 배웁니다.',
    'hero.body': 'Vais는 .vais 언어와 자체 컴파일러 workspace입니다. 공개 컴파일러 명령은 scripts/vaisc이며, 공개 주장은 repository gate로 검증된 범위에 맞춥니다.',
    'hero.github': 'GitHub 열기',
    'hero.start': '학습 시작',
    'signal.ext.label': '소스',
    'signal.ext.value': '.vais 파일만 사용',
    'signal.cmd.label': '명령',
    'signal.gate.label': 'Release corpus',
    'signal.gate.value': '검증 예제 49개',
    'playground.eyebrow': '플레이그라운드',
    'playground.title': '편집 가능한 브라우저 실행기',
    'playground.editorLabel': '소스',
    'playground.run': '코드 실행',
    'playground.reset': '초기화',
    'playground.copy': '소스 복사',
    'playground.output': '출력',
    'playground.mode': '브라우저 subset',
    'learn.eyebrow': '학습 흐름',
    'learn.title': 'GitHub에서 실행까지 더 짧게 갑니다.',
    'learn.body': '프로젝트의 공개 컴파일러 경로와 공개 checker 경로는 각각 하나입니다. 새 사용자가 먼저 볼 명령은 아래 정도로 충분합니다.',
    'learn.clone.title': 'Clone 후 toolchain 확인',
    'learn.run.title': '검증 예제 실행',
    'learn.build.title': 'Native 출력 빌드',
    'language.eyebrow': '언어',
    'language.title': '현재 문법은 작고 gate-backed입니다.',
    'language.body': '공개 surface는 fn, let, if, while, for, struct, enum, match, return 같은 full-word syntax를 사용합니다. 오래된 compact 문법이나 Rust식 예제는 현재 mainline 기준으로 stale입니다.',
    'language.scalar.title': 'Scalar',
    'language.scalar.body': 'Int, Bool, Char, Str은 문서화된 promoted path에서 검증되어 있습니다.',
    'language.collections.title': 'Collection',
    'language.collections.body': 'List<Int>, List<Struct>, local Map<Int,Int> slice가 검증되어 있습니다. 더 넓은 generic, Option, Result는 아직 future work입니다.',
    'language.match.title': 'Enum과 match',
    'language.match.body': 'Payload-free enum, Int payload enum, wildcard match arm, 일부 struct payload case가 release corpus로 보호됩니다.',
    'language.modules.title': 'Import',
    'language.modules.body': 'Full engine은 local dotted import와 vais.toml 기반 local dependency package path를 지원합니다.',
    'language.reference': '언어 reference',
    'language.examples': '예제 corpus',
    'compiler.eyebrow': '컴파일러',
    'compiler.title': '공개 컴파일러는 scripts/vaisc입니다.',
    'compiler.body': 'Native public driver는 재사용 가능한 self-host compiler core를 링크하고 LLVM IR을 생성한 뒤 clang으로 native binary를 만듭니다. Standalone install도 이 repository 기준으로 빌드됩니다.',
    'verification.eyebrow': '검증',
    'verification.title': '공개 주장은 약속이 아니라 gate에서 나옵니다.',
    'verification.body': '사이트와 GitHub README는 이 검증 기준과 동기화되어야 합니다. gate나 parity manifest에 없는 기능은 현재 Vais 기능으로 홍보하지 않습니다.',
    'footer.summary': 'Vais 언어와 자체 컴파일러 workspace.',
    'footer.docs': '문서',
    'footer.selfhost': '자체 컴파일러',
    copy: '복사',
    copied: '복사됨',
    copyFailed: '복사 실패',
    outputFor: '소스',
    expected: '기대 종료 코드',
    exitCode: '종료 코드',
    sampleSource: '선택 예제',
    editedSource: '수정한 소스',
    ready: '소스를 수정한 뒤 코드를 실행하세요.',
    runnerError: '실행 오류',
    command: '로컬 컴파일러 명령',
    note: '브라우저 실행기는 즉시 피드백을 위해 Vais 튜토리얼 subset을 실행합니다. 전체 컴파일러는 scripts/vaisc를 로컬에서 사용하세요.',
  },
};

const storedLang = localStorage.getItem('vais-lang');
let currentLang = ['en', 'ko'].includes(storedLang) ? storedLang : 'en';
let currentSample = samples[0];

const toggle = document.querySelector('.nav-mobile-toggle');
const navLinks = document.querySelector('.nav-links');
const sampleSelect = document.getElementById('sample-select');
const lessonStrip = document.getElementById('lesson-strip');
const editor = document.getElementById('code-editor');
const output = document.querySelector('#playground-output code');

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
  renderLesson(!editor || editor.value.trim() === '');
  renderOutput(false);
}

function renderSamples() {
  if (!sampleSelect) return;
  sampleSelect.innerHTML = '';
  samples.forEach((sample) => {
    const option = document.createElement('option');
    option.value = sample.id;
    option.textContent = sample.title[currentLang];
    sampleSelect.appendChild(option);
  });
  sampleSelect.value = currentSample.id;
}

function renderLesson(resetEditor = true) {
  if (!lessonStrip || !editor) return;
  lessonStrip.innerHTML = '';
  const title = document.createElement('strong');
  title.textContent = currentSample.title[currentLang];
  const text = document.createElement('span');
  text.textContent = currentSample.lesson[currentLang];
  lessonStrip.append(title, text);
  if (resetEditor) editor.value = currentSample.code;
}

function renderOutput(hasRun) {
  if (!output) return;
  if (!hasRun) {
    output.textContent = `$ browser Vais runner
${t('ready')}
${t('expected')}: ${currentSample.expect}
${t('command')}: scripts/vaisc run ${currentSample.path}`;
    return;
  }

  const result = runVaisSubset(editor.value);
  const sourceKind = normalizeSource(editor.value) === normalizeSource(currentSample.code)
    ? t('sampleSource')
    : t('editedSource');

  if (!result.ok) {
    output.textContent = `$ browser Vais runner
${t('outputFor')}: ${sourceKind}
${t('runnerError')}: ${result.error}

${t('note')}`;
    return;
  }

  output.textContent = `$ browser Vais runner
${t('outputFor')}: ${sourceKind}
${t('exitCode')}: ${result.exitCode}
${t('expected')}: ${currentSample.expect}

${t('command')}: scripts/vaisc run ${currentSample.path}
${t('note')}`;
}

function normalizeSource(source) {
  return source.replace(/\s+/g, ' ').trim();
}

if (toggle && navLinks) {
  toggle.addEventListener('click', () => {
    const isOpen = navLinks.classList.toggle('open');
    toggle.classList.toggle('active');
    toggle.setAttribute('aria-expanded', String(isOpen));
  });

  navLinks.querySelectorAll('a').forEach((link) => {
    link.addEventListener('click', () => {
      navLinks.classList.remove('open');
      toggle.classList.remove('active');
      toggle.setAttribute('aria-expanded', 'false');
    });
  });
}

document.querySelectorAll('.lang-btn').forEach((button) => {
  button.addEventListener('click', () => {
    currentLang = button.dataset.lang || 'en';
    localStorage.setItem('vais-lang', currentLang);
    renderI18n();
  });
});

if (sampleSelect) {
  sampleSelect.addEventListener('change', () => {
    currentSample = samples.find((sample) => sample.id === sampleSelect.value) || samples[0];
    renderLesson(true);
    renderOutput(false);
  });
}

const runButton = document.getElementById('run-sample');
if (runButton) {
  runButton.addEventListener('click', () => renderOutput(true));
}

const resetButton = document.getElementById('reset-sample');
if (resetButton) {
  resetButton.addEventListener('click', () => {
    renderLesson(true);
    renderOutput(false);
  });
}

document.querySelectorAll('[data-copy-target]').forEach((button) => {
  button.addEventListener('click', async () => {
    const targetId = button.getAttribute('data-copy-target');
    const target = targetId ? document.getElementById(targetId) : null;
    const text = target ? ('value' in target ? target.value : target.textContent).trim() : '';
    if (!text) return;

    try {
      await navigator.clipboard.writeText(text);
      const original = button.textContent;
      button.textContent = t('copied');
      setTimeout(() => {
        button.textContent = original;
      }, 1400);
    } catch {
      button.textContent = t('copyFailed');
      setTimeout(() => {
        button.textContent = t('copy');
      }, 1400);
    }
  });
});

renderI18n();
