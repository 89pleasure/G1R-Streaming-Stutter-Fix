import assert from "node:assert/strict";

import {
  DEFAULT_LANGUAGE,
  LANGUAGE_AUTO,
  applyTranslationsToDocument,
  createTranslator,
  languageOptions,
  resolveLanguage,
  translationKeys,
} from "./i18n.js";
import { translations } from "./locales/index.js";

const supportedLanguageValues = [
  "auto",
  "en",
  "de",
  "pl",
  "es",
  "fr",
  "it",
  "ru",
  "ja",
  "zh",
  "pt",
];

assert.equal(DEFAULT_LANGUAGE, "en");
assert.equal(LANGUAGE_AUTO, "auto");
assert.deepEqual(
  languageOptions.map((option) => option.value),
  supportedLanguageValues,
);
assert.equal(resolveLanguage("auto", ["de-DE", "en-US"]), "de");
assert.equal(resolveLanguage("auto", ["pl-PL", "de-DE"]), "pl");
assert.equal(resolveLanguage("auto", ["fr-FR"]), "fr");
assert.equal(resolveLanguage("auto", ["ja-JP", "en-US"]), "ja");
assert.equal(resolveLanguage("auto", ["zh-CN", "en-US"]), "zh");
assert.equal(resolveLanguage("de", ["en-US"]), "de");
assert.equal(resolveLanguage("unsupported", ["pl-PL"]), "pl");

const englishKeys = translationKeys(translations.en);
for (const [language, dictionary] of Object.entries(translations)) {
  assert.deepEqual(
    translationKeys(dictionary),
    englishKeys,
    `${language} translations must match English keys`,
  );
}

{
  const t = createTranslator("de");
  assert.equal(t("nav.optimizeStreaming"), "Streaming-Fix");
  assert.equal(t("preset.count", { count: 3 }), "3 Presets");
  assert.equal(t("missing.key"), "missing.key");
}

{
  const elements = new Map();
  const title = textElement();
  const button = textElement();
  button.dataset.i18nTitle = "actions.refresh";
  elements.set("title", title);
  elements.set("button", button);
  const document = {
    querySelectorAll(selector) {
      if (selector === "[data-i18n]") {
        return [title];
      }
      if (selector === "[data-i18n-title]") {
        return [button];
      }
      return [];
    },
  };

  title.dataset.i18n = "views.settings";
  applyTranslationsToDocument(document, createTranslator("pl"));

  assert.equal(title.textContent, "Ustawienia");
  assert.equal(button.title, "Odśwież");
}

function textElement() {
  return {
    dataset: {},
    textContent: "",
    title: "",
  };
}
