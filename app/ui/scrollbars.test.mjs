import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const css = await readFile(new URL("./styles.css", import.meta.url), "utf8");

for (const selector of [".view.active", ".comparison-gallery", ".table-wrap", ".log"]) {
  assert.match(
    css,
    new RegExp(`${escapeRegExp(selector)}\\s*\\{[^}]*scrollbar-gutter:\\s*stable;`, "s"),
    `${selector} keeps a visible scrollbar gutter when it can scroll`,
  );
}

assert.match(css, /scrollbar-width:\s*thin;/);
assert.match(css, /::-webkit-scrollbar-thumb/);

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
