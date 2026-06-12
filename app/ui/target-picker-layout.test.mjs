import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const html = await readFile(new URL("./index.html", import.meta.url), "utf8");
const css = await readFile(new URL("./styles.css", import.meta.url), "utf8");
const mainJs = await readFile(new URL("./main.js", import.meta.url), "utf8");
const cargoToml = await readFile(new URL("../src-tauri/Cargo.toml", import.meta.url), "utf8");
const tauriLib = await readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
const defaultCapability = await readFile(
  new URL("../src-tauri/capabilities/default.json", import.meta.url),
  "utf8",
);

assert.match(html, /<div class="path-input-row">[\s\S]*id="targetInput"[\s\S]*id="browseTargetButton"/);
assert.match(html, /id="browseTargetButton"/);
assert.match(html, /class="[^"]*\bpath-picker-button\b[^"]*"/);
assert.match(html, /data-i18n-title="settings\.browseTargetFolder"/);
assert.match(html, /data-i18n-aria-label="settings\.browseTargetFolder"/);

assert.match(css, /\.path-input-row\s*\{/);
assert.match(css, /\.path-input-row input\[type="text"\]\s*\{/);
assert.match(css, /\.path-picker-button\s*\{/);

assert.match(mainJs, /"browseTargetButton"/);
assert.match(mainJs, /elements\.browseTargetButton\.addEventListener\("click", browseTargetFolder\)/);
assert.match(mainJs, /async function browseTargetFolder\(\)/);
assert.match(mainJs, /window\.__TAURI__\?\.dialog\?\.open/);
assert.match(mainJs, /directory:\s*true/);
assert.match(mainJs, /multiple:\s*false/);
assert.match(mainJs, /defaultPath:\s*state\.targetDir \|\| undefined/);
assert.match(mainJs, /applyTargetDir\(selectedFolder\)/);

assert.match(cargoToml, /tauri-plugin-dialog\s*=/);
assert.match(tauriLib, /\.plugin\(tauri_plugin_dialog::init\(\)\)/);
assert.match(defaultCapability, /dialog:allow-open/);
