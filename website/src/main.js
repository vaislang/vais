// Copy to clipboard
document.querySelectorAll('.copy-btn').forEach((btn) => {
  btn.addEventListener('click', async () => {
    const text = btn.dataset.copy;
    try {
      await navigator.clipboard.writeText(text);
      btn.innerHTML =
        '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#34d399" stroke-width="2"><path d="M20 6L9 17l-5-5"/></svg>';
      setTimeout(() => {
        btn.innerHTML =
          '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/></svg>';
      }, 2000);
    } catch {
      // Fallback: do nothing
    }
  });
});

// Mobile nav toggle
const toggle = document.querySelector('.nav-mobile-toggle');
const navLinks = document.querySelector('.nav-links');

if (toggle && navLinks) {
  toggle.addEventListener('click', () => {
    const isOpen = navLinks.classList.toggle('open');
    toggle.classList.toggle('active');
    toggle.setAttribute('aria-expanded', isOpen);
  });

  // Close on link click
  navLinks.querySelectorAll('a').forEach((link) => {
    link.addEventListener('click', () => {
      navLinks.classList.remove('open');
      toggle.classList.remove('active');
      toggle.setAttribute('aria-expanded', 'false');
    });
  });

  // Close on outside click
  document.addEventListener('click', (e) => {
    if (!toggle.contains(e.target) && !navLinks.contains(e.target)) {
      navLinks.classList.remove('open');
      toggle.classList.remove('active');
      toggle.setAttribute('aria-expanded', 'false');
    }
  });
}

// Inline playground loader
const playgroundBtn = document.getElementById('playground-load-btn');
if (playgroundBtn) {
  playgroundBtn.addEventListener('click', () => {
    const container = document.getElementById('playground-container');
    const placeholder = document.getElementById('playground-placeholder');
    if (container && placeholder) {
      const iframe = document.createElement('iframe');
      iframe.src = '/playground/';
      iframe.title = 'Vais Playground';
      iframe.loading = 'lazy';
      placeholder.remove();
      container.appendChild(iframe);
    }
  });
}

// Code tabs
document.querySelectorAll('.code-tab').forEach((tab) => {
  tab.addEventListener('click', () => {
    const idx = tab.dataset.tab;
    document.querySelectorAll('.code-tab').forEach((t) => t.classList.remove('active'));
    document.querySelectorAll('.code-panel').forEach((p) => p.classList.remove('active'));
    tab.classList.add('active');
    document.querySelector(`.code-panel[data-panel="${idx}"]`)?.classList.add('active');
  });
});

// Compare language tabs
const tokenCounts = { rust: '97 tokens', python: '112 tokens', go: '108 tokens', c: '115 tokens' };
document.querySelectorAll('.compare-tab').forEach((tab) => {
  tab.addEventListener('click', () => {
    const lang = tab.dataset.lang;
    document.querySelectorAll('.compare-tab').forEach((t) => t.classList.remove('active'));
    document.querySelectorAll('.compare-lang-panel').forEach((p) => p.classList.remove('active'));
    tab.classList.add('active');
    document.querySelector(`.compare-lang-panel[data-lang="${lang}"]`)?.classList.add('active');
    const counter = document.getElementById('compare-token-count');
    if (counter && tokenCounts[lang]) counter.textContent = tokenCounts[lang];
  });
});

// Smooth scroll for anchor links
document.querySelectorAll('a[href^="#"]').forEach((anchor) => {
  anchor.addEventListener('click', (e) => {
    const target = document.querySelector(anchor.getAttribute('href'));
    if (target) {
      e.preventDefault();
      target.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  });
});

// Animate bars on scroll
const observeElements = () => {
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add('visible');
          observer.unobserve(entry.target);
        }
      });
    },
    { threshold: 0.2 }
  );

  document.querySelectorAll('.bar-fill').forEach((el) => {
    const width = el.style.width;
    el.style.width = '0%';
    el.dataset.width = width;
    observer.observe(el);
  });

  document.querySelectorAll('.feature-card, .compare-card').forEach((el) => {
    observer.observe(el);
  });
};

// Trigger bar animation when visible
const style = document.createElement('style');
style.textContent = `
  .bar-fill { transition: width 1s ease-out; }
  .bar-fill.visible { width: var(--target-width) !important; }
  .feature-card, .compare-card {
    opacity: 0;
    transform: translateY(20px);
    transition: opacity 0.5s, transform 0.5s;
  }
  .feature-card.visible, .compare-card.visible {
    opacity: 1;
    transform: translateY(0);
  }
`;
document.head.appendChild(style);

// Apply target widths and start observing
document.addEventListener('DOMContentLoaded', () => {
  document.querySelectorAll('.bar-fill').forEach((el) => {
    el.style.setProperty('--target-width', el.style.width);
  });
  observeElements();
});
