import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import { translations } from "./locales/index.js";

const html = await readFile(new URL("./index.html", import.meta.url), "utf8");
const mainJs = await readFile(new URL("./main.js", import.meta.url), "utf8");
const settingsView = extractSection(html, "settingsView");

assert.match(html, /<html lang="en">/);
assert.match(html, /data-i18n="nav\.optimizeStreaming"/);
assert.match(html, /data-i18n-aria-label="nav\.viewsAria"/);
assert.match(html, /data-i18n="feature\.streaming\.title"/);
assert.match(html, /data-i18n="feature\.streaming\.description"/);
assert.match(html, /data-i18n="feature\.streaming\.detailText"/);

assert.match(settingsView, /id="languageSelect"/);
assert.match(settingsView, /data-i18n="settings\.languageHeading"/);
assert.match(settingsView, /data-i18n="settings\.languageLabel"/);
assert.match(settingsView, /data-i18n="settings\.languageHint"/);

assert.match(mainJs, /from "\.\/i18n\.js"/);
assert.match(mainJs, /applyTranslationsToDocument/);
assert.match(mainJs, /languageOptions/);
assert.match(mainJs, /resolveLanguage/);
assert.match(mainJs, /language: state\.languagePreference/);
assert.match(mainJs, /elements\.languageSelect\.addEventListener\("change"/);

for (const key of i18nKeys(html)) {
  assert.ok(Object.hasOwn(translations.en, key), `${key} must exist in English translations`);
}

function extractSection(source, id) {
  const sectionPattern = new RegExp(`<section class="view(?: active)?" id="${id}">`);
  const match = sectionPattern.exec(source);
  const start = match?.index ?? -1;
  assert.notEqual(start, -1, `${id} section exists`);

  const nextView = source.indexOf('<section class="view"', start + match[0].length);
  return nextView === -1 ? source.slice(start) : source.slice(start, nextView);
}

function i18nKeys(source) {
  const keys = [];
  const keyPattern = /data-i18n(?:-[a-z-]+)?="([^"]+)"/g;
  for (const match of source.matchAll(keyPattern)) {
    keys.push(match[1]);
  }
  return keys;
}
