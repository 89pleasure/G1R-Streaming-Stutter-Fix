import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const html = await readFile(new URL("./index.html", import.meta.url), "utf8");
const css = await readFile(new URL("./styles.css", import.meta.url), "utf8");
const main = await readFile(new URL("./main.js", import.meta.url), "utf8");
const optimizeStreamingView = extractSection(html, "optimizeStreamingView");

assert.match(optimizeStreamingView, /class="panel preset-panel"/);
assert.doesNotMatch(optimizeStreamingView, /class="panel explanation-panel"/);
assert.doesNotMatch(optimizeStreamingView, /id="streamingInfoTitle"/);
assert.doesNotMatch(optimizeStreamingView, /id="streamingInfoTag"/);
assert.doesNotMatch(optimizeStreamingView, /id="streamingInfoBody"/);
assert.doesNotMatch(optimizeStreamingView, /id="streamingInfoDetails"/);
assert.doesNotMatch(optimizeStreamingView, /class="info-button/);
assert.doesNotMatch(optimizeStreamingView, /data-streaming-info=/);

assert.doesNotMatch(optimizeStreamingView, /class="streaming-detail-title"/);
assert.doesNotMatch(optimizeStreamingView, /class="streaming-detail-heading"/);
assert.ok(
  optimizeStreamingView.match(/<div class="streaming-feature-detail">\s*<p(?:\s[^>]*)?>/g)?.length >= 4,
  "Optimize Streaming detail boxes start directly with explanatory text",
);

assert.doesNotMatch(main, /const streamingInfoContent = \{/);
assert.doesNotMatch(main, /selectedStreamingInfo/);
assert.doesNotMatch(main, /function showStreamingInfo\(/);
assert.doesNotMatch(main, /querySelectorAll\("\[data-streaming-info\]"\)/);
assert.doesNotMatch(main, /renderSelectedPresetInfo/);

assert.doesNotMatch(css, /\.info-button\s*\{/);
assert.doesNotMatch(css, /\.preset-option-wrap\s*\{/);
assert.match(css, /#optimizeStreamingView > \.layout-grid\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\);/s);
assert.match(css, /\.feature-row\.with-detail\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\)\s+minmax\(280px,\s*0\.85fr\);/s);
assert.match(css, /\.streaming-feature-detail\s*\{/);
assert.match(css, /\.preset-option\s*\{[^}]*min-height:\s*62px;/s);
assert.match(css, /\.preset-grid\s*\{[^}]*minmax\(128px,\s*1fr\)/s);

function extractSection(source, id) {
  const sectionPattern = new RegExp(`<section class="view(?: active)?" id="${id}">`);
  const match = sectionPattern.exec(source);
  const start = match?.index ?? -1;
  assert.notEqual(start, -1, `${id} section exists`);

  const nextView = source.indexOf('<section class="view"', start + match[0].length);
  assert.notEqual(nextView, -1, `${id} has a following view`);
  return source.slice(start, nextView);
}
