const messages = {
  en: {
    'nav.playground': 'Playground',
    'nav.start': 'Start',
    'nav.language': 'Language',
    'nav.compiler': 'Compiler',
    'nav.verification': 'Verification',
    'hero.eyebrow': 'Official current baseline',
    'hero.body': 'A .vais language and self-host compiler workspace. The public compiler command is scripts/vaisc, and public claims are limited to repository gates.',
    'hero.playground': 'Open playground',
    'hero.reference': 'Language reference',
    'hero.release': 'v1.0.1 release',
    'panel.title': 'Current toolchain',
    'status.source.label': 'Source',
    'status.compiler.label': 'Compiler',
    'status.checker.label': 'Checker',
    'status.release.label': 'Release',
    'status.examples.label': 'Release corpus',
    'status.examples.value': '141 examples',
    'start.eyebrow': 'Start',
    'start.title': 'Compile and check a Vais file.',
    'start.body': 'The repository keeps one public compiler path and one public checker path. Use the playground for quick editing, then use scripts/vaisc for full local compilation.',
    'start.play.title': 'Use the playground',
    'start.play.body': 'Open the separate coding workspace, edit a tutorial program, and run the browser subset for immediate feedback.',
    'start.play.link': 'Open /playground/',
    'start.local.title': 'Run locally',
    'start.local.body': 'Clone the repository and run the same examples through the native public compiler.',
    'start.build.title': 'Build output',
    'start.build.body': 'Emit LLVM IR or build a native binary with clang through the public driver.',
    'language.eyebrow': 'Language',
    'language.title': 'Small, explicit, gate-backed.',
    'language.body': 'Current public syntax uses full-word forms such as fn, let, if, while, for, struct, enum, match, and return. Older compact syntax is not part of this mainline.',
    'language.reference': 'Language reference',
    'language.examples': 'Examples',
    'compiler.eyebrow': 'Compiler',
    'compiler.title': 'The public driver is scripts/vaisc.',
    'compiler.body': 'The native driver links the reusable self-host compiler core, emits LLVM IR, and uses clang for native binaries. The v1.0.1 release publishes standalone archives for Linux x64, macOS arm64, and macOS x64.',
    'verification.eyebrow': 'Verification',
    'verification.title': 'The site follows the gates.',
    'verification.body': 'If a feature is not protected by a script gate or parity manifest entry, it should not be advertised as current Vais.',
    'footer.summary': 'Vais language and self-host compiler workspace.',
    'footer.playground': 'Playground',
    'footer.docs': 'Docs',
    'footer.selfhost': 'Self-host',
    copy: 'Copy',
    copied: 'Copied',
    copyFailed: 'Copy failed',
  },
  ko: {
    'nav.playground': '플레이그라운드',
    'nav.start': '시작',
    'nav.language': '언어',
    'nav.compiler': '컴파일러',
    'nav.verification': '검증',
    'hero.eyebrow': '공식 현재 기준',
    'hero.body': 'Vais는 .vais 언어와 자체 컴파일러 workspace입니다. 공개 컴파일러 명령은 scripts/vaisc이며, 공개 주장은 repository gate 범위로 제한합니다.',
    'hero.playground': '플레이그라운드 열기',
    'hero.reference': '언어 reference',
    'hero.release': 'v1.0.1 릴리스',
    'panel.title': '현재 toolchain',
    'status.source.label': '소스',
    'status.compiler.label': '컴파일러',
    'status.checker.label': '체커',
    'status.release.label': '릴리스',
    'status.examples.label': 'Release corpus',
    'status.examples.value': '예제 141개',
    'start.eyebrow': '시작',
    'start.title': 'Vais 파일을 컴파일하고 확인합니다.',
    'start.body': '이 repository의 공개 컴파일러 경로와 checker 경로는 각각 하나입니다. 빠른 편집은 플레이그라운드에서 하고, 전체 컴파일은 로컬 scripts/vaisc를 사용합니다.',
    'start.play.title': '플레이그라운드 사용',
    'start.play.body': '별도 코딩 workspace를 열어 튜토리얼 프로그램을 수정하고 브라우저 subset으로 즉시 실행합니다.',
    'start.play.link': '/playground/ 열기',
    'start.local.title': '로컬 실행',
    'start.local.body': 'repository를 clone한 뒤 같은 예제를 native public compiler로 실행합니다.',
    'start.build.title': '출력 빌드',
    'start.build.body': '공개 driver를 통해 LLVM IR을 생성하거나 clang 기반 native binary를 빌드합니다.',
    'language.eyebrow': '언어',
    'language.title': '작고 명시적이며 gate-backed입니다.',
    'language.body': '현재 공개 문법은 fn, let, if, while, for, struct, enum, match, return 같은 full-word form을 사용합니다. 오래된 compact syntax는 이 mainline의 일부가 아닙니다.',
    'language.reference': '언어 reference',
    'language.examples': '예제',
    'compiler.eyebrow': '컴파일러',
    'compiler.title': '공개 driver는 scripts/vaisc입니다.',
    'compiler.body': 'Native driver는 재사용 가능한 self-host compiler core를 링크하고 LLVM IR을 생성한 뒤 clang으로 native binary를 만듭니다. v1.0.1 릴리스는 Linux x64, macOS arm64, macOS x64 standalone archive를 게시합니다.',
    'verification.eyebrow': '검증',
    'verification.title': '사이트는 gate를 따릅니다.',
    'verification.body': 'script gate나 parity manifest entry로 보호되지 않는 기능은 현재 Vais 기능으로 홍보하지 않습니다.',
    'footer.summary': 'Vais 언어와 자체 컴파일러 workspace.',
    'footer.playground': '플레이그라운드',
    'footer.docs': '문서',
    'footer.selfhost': '자체 컴파일러',
    copy: '복사',
    copied: '복사됨',
    copyFailed: '복사 실패',
  },
};

const storedLang = localStorage.getItem('vais-lang');
let currentLang = ['en', 'ko'].includes(storedLang) ? storedLang : 'en';

const toggle = document.querySelector('.nav-mobile-toggle');
const navLinks = document.querySelector('.nav-links');

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
