(function () {
  'use strict';

  var LANGUAGES = [
    { code: 'ko', label: '한국어' },
    { code: 'en', label: 'English' },
    { code: 'ja', label: '日本語' },
    { code: 'zh', label: '中文' }
  ];

  // Detect current language from URL path
  function getCurrentLang() {
    var path = window.location.pathname;
    // /docs/en/... → en, /docs/ja/... → ja, /docs/zh/... → zh
    var match = path.match(/\/docs\/(en|ja|zh)\//);
    if (match) return match[1];
    // /docs/... (no lang prefix) → ko (default)
    return 'ko';
  }

  // Build the target URL for a given language
  function buildLangUrl(targetLang) {
    var path = window.location.pathname;
    var currentLang = getCurrentLang();

    if (currentLang === targetLang) return path;

    if (currentLang === 'ko') {
      // /docs/foo.html → /docs/{lang}/foo.html
      return path.replace(/\/docs\//, '/docs/' + targetLang + '/');
    } else if (targetLang === 'ko') {
      // /docs/{lang}/foo.html → /docs/foo.html
      return path.replace('/docs/' + currentLang + '/', '/docs/');
    } else {
      // /docs/{from}/foo.html → /docs/{to}/foo.html
      return path.replace('/docs/' + currentLang + '/', '/docs/' + targetLang + '/');
    }
  }

  function createSelector() {
    var currentLang = getCurrentLang();

    var wrapper = document.createElement('div');
    wrapper.className = 'lang-selector';

    var select = document.createElement('select');
    select.className = 'lang-select';
    select.setAttribute('aria-label', 'Language');

    for (var i = 0; i < LANGUAGES.length; i++) {
      var lang = LANGUAGES[i];
      var option = document.createElement('option');
      option.value = lang.code;
      option.textContent = lang.label;
      if (lang.code === currentLang) option.selected = true;
      select.appendChild(option);
    }

    select.addEventListener('change', function () {
      var url = buildLangUrl(this.value);
      window.location.href = url;
    });

    wrapper.appendChild(select);
    return wrapper;
  }

  function inject() {
    // mdBook right-buttons area (theme switcher, search, etc.)
    var rightButtons = document.querySelector('.right-buttons');
    if (rightButtons) {
      var selector = createSelector();
      rightButtons.insertBefore(selector, rightButtons.firstChild);
      return;
    }
    // Fallback: menu-bar
    var menuBar = document.querySelector('.menu-bar');
    if (menuBar) {
      var selector = createSelector();
      menuBar.appendChild(selector);
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', inject);
  } else {
    inject();
  }
})();
