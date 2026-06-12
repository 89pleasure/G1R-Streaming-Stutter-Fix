import { translations } from "./locales/index.js";

export const DEFAULT_LANGUAGE = "en";
export const LANGUAGE_AUTO = "auto";

export const languageOptions = [
  { value: LANGUAGE_AUTO, labelKey: "language.auto" },
  { value: "en", labelKey: "language.en" },
  { value: "de", labelKey: "language.de" },
  { value: "pl", labelKey: "language.pl" },
  { value: "es", labelKey: "language.es" },
  { value: "fr", labelKey: "language.fr" },
  { value: "it", labelKey: "language.it" },
  { value: "ru", labelKey: "language.ru" },
  { value: "ja", labelKey: "language.ja" },
  { value: "zh", labelKey: "language.zh" },
  { value: "pt", labelKey: "language.pt" },
];

const supportedLanguages = new Set(Object.keys(translations));

export function resolveLanguage(preference = LANGUAGE_AUTO, languages = browserLanguages()) {
  if (supportedLanguages.has(preference)) {
    return preference;
  }

  for (const language of languages) {
    const normalizedLanguage = normalizeLanguage(language);
    if (supportedLanguages.has(normalizedLanguage)) {
      return normalizedLanguage;
    }
  }

  return DEFAULT_LANGUAGE;
}

export function createTranslator(language) {
  const activeDictionary = translations[language] ?? translations[DEFAULT_LANGUAGE];
  const fallbackDictionary = translations[DEFAULT_LANGUAGE];

  return (key, values = {}) => {
    const template = activeDictionary[key] ?? fallbackDictionary[key] ?? key;
    return interpolate(template, values);
  };
}

export function applyTranslationsToDocument(documentObject, translate) {
  documentObject.querySelectorAll("[data-i18n]").forEach((element) => {
    element.textContent = translate(element.dataset.i18n);
  });

  for (const [attributeName, dataKey] of [
    ["aria-label", "i18nAriaLabel"],
    ["placeholder", "i18nPlaceholder"],
    ["title", "i18nTitle"],
  ]) {
    documentObject.querySelectorAll(`[data-${datasetNameToAttribute(dataKey)}]`).forEach((element) => {
      element.setAttribute?.(attributeName, translate(element.dataset[dataKey]));
      element[attributeName] = translate(element.dataset[dataKey]);
    });
  }
}

export function translationKeys(dictionary) {
  return Object.keys(dictionary).sort();
}

function interpolate(template, values) {
  return template.replaceAll(/\{([a-zA-Z0-9_]+)\}/g, (match, key) => {
    if (!Object.hasOwn(values, key)) {
      return match;
    }

    return String(values[key]);
  });
}

function normalizeLanguage(language) {
  return String(language).toLowerCase().split("-")[0];
}

function browserLanguages() {
  if (typeof navigator === "undefined") {
    return [];
  }

  if (Array.isArray(navigator.languages) && navigator.languages.length > 0) {
    return navigator.languages;
  }

  return navigator.language ? [navigator.language] : [];
}

function datasetNameToAttribute(name) {
  return name.replaceAll(/[A-Z]/g, (character) => `-${character.toLowerCase()}`);
}
