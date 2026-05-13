// i18n.js - Internationalization module for Vais website

const SUPPORTED_LANGUAGES = ['en', 'ko', 'ja', 'zh'];
const DEFAULT_LANGUAGE = 'en';
const STORAGE_KEY = 'vais-language';

let currentLanguage = DEFAULT_LANGUAGE;
let translations = {};
const translationCache = {};

/**
 * Sanitize HTML string — allow only safe tags for i18n translations
 */
function sanitizeHTML(html) {
  const div = document.createElement('div');
  div.innerHTML = html;
  const allowed = new Set(['SPAN', 'CODE', 'STRONG', 'BR', 'A', 'EM', 'B', 'I']);
  const allowedAttrs = new Set(['href', 'target', 'style', 'class']);

  function clean(node) {
    const children = Array.from(node.childNodes);
    for (const child of children) {
      if (child.nodeType === Node.ELEMENT_NODE) {
        if (!allowed.has(child.tagName)) {
          // Replace disallowed element with its text content
          child.replaceWith(document.createTextNode(child.textContent));
        } else {
          // Remove disallowed attributes
          for (const attr of Array.from(child.attributes)) {
            if (!allowedAttrs.has(attr.name)) {
              child.removeAttribute(attr.name);
            }
          }
          // Strip javascript: from href
          if (child.hasAttribute('href') && child.getAttribute('href').replace(/\s/g, '').toLowerCase().startsWith('javascript:')) {
            child.removeAttribute('href');
          }
          clean(child);
        }
      }
    }
  }

  clean(div);
  return div.innerHTML;
}

/**
 * Detect browser language and map to supported language
 */
function detectLanguage() {
  const browserLang = navigator.language || navigator.userLanguage;
  const langCode = browserLang.split('-')[0]; // e.g., "ko-KR" -> "ko"
  return SUPPORTED_LANGUAGES.includes(langCode) ? langCode : DEFAULT_LANGUAGE;
}

/**
 * Get language from localStorage or detect from browser
 */
function getInitialLanguage() {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored && SUPPORTED_LANGUAGES.includes(stored)) {
    return stored;
  }
  return detectLanguage();
}

/**
 * Load translations JSON for a language
 */
async function loadTranslations(lang) {
  // Return cached translations if available
  if (translationCache[lang]) {
    return translationCache[lang];
  }

  try {
    const response = await fetch(`/locales/${lang}.json`);
    if (!response.ok) {
      throw new Error(`Failed to load ${lang}.json`);
    }
    const data = await response.json();
    translationCache[lang] = data;
    return data;
  } catch (error) {
    console.error(`Error loading translations for ${lang}:`, error);
    // Fallback to default language (with guard against infinite recursion)
    if (lang !== DEFAULT_LANGUAGE) {
      return loadTranslations(DEFAULT_LANGUAGE);
    }
    // If default language also fails, return empty (no recursion)
    return {};
  }
}

/**
 * Get nested translation value by dot-separated key
 * e.g., "nav.compare" -> translations.nav.compare
 */
function getTranslation(key) {
  const keys = key.split('.');
  let value = translations;
  for (const k of keys) {
    if (value && typeof value === 'object') {
      value = value[k];
    } else {
      return key; // Return key if not found
    }
  }
  return value || key;
}

/**
 * Update DOM elements with data-i18n attribute
 */
function updateDOM() {
  // Single DOM traversal for all i18n attributes
  document.querySelectorAll('[data-i18n], [data-i18n-placeholder], [data-i18n-aria]').forEach((element) => {
    // Update text/html content
    if (element.hasAttribute('data-i18n')) {
      const key = element.getAttribute('data-i18n');
      const translation = getTranslation(key);

      if (element.hasAttribute('data-i18n-html')) {
        element.innerHTML = sanitizeHTML(translation);
      } else {
        element.textContent = translation;
      }
    }

    // Update placeholder
    if (element.hasAttribute('data-i18n-placeholder')) {
      const key = element.getAttribute('data-i18n-placeholder');
      element.setAttribute('placeholder', getTranslation(key));
    }

    // Update aria-label
    if (element.hasAttribute('data-i18n-aria')) {
      const key = element.getAttribute('data-i18n-aria');
      element.setAttribute('aria-label', getTranslation(key));
    }
  });

  // Update document title
  const titleKey = document.documentElement.getAttribute('data-i18n-title');
  if (titleKey) {
    document.title = getTranslation(titleKey);
  }

  // Update html lang attribute
  document.documentElement.setAttribute('lang', currentLanguage);
}

/**
 * Change language
 */
export async function changeLanguage(lang) {
  if (!SUPPORTED_LANGUAGES.includes(lang)) {
    console.warn(`Unsupported language: ${lang}`);
    return;
  }

  currentLanguage = lang;
  localStorage.setItem(STORAGE_KEY, lang);

  // Load translations
  translations = await loadTranslations(lang);

  // Update DOM
  updateDOM();

  // Dispatch custom event for other modules
  window.dispatchEvent(new CustomEvent('languageChanged', { detail: { language: lang } }));
}

/**
 * Get current language
 */
export function getCurrentLanguage() {
  return currentLanguage;
}

/**
 * Get all supported languages
 */
export function getSupportedLanguages() {
  return [...SUPPORTED_LANGUAGES];
}

/**
 * Initialize i18n system
 */
export async function initI18n() {
  currentLanguage = getInitialLanguage();
  translations = await loadTranslations(currentLanguage);
  updateDOM();
  return currentLanguage;
}

/**
 * Get translation by key (for programmatic use)
 */
export function t(key) {
  return getTranslation(key);
}
