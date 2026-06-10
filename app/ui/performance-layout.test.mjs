import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const html = await readFile(new URL("./index.html", import.meta.url), "utf8");
const css = await readFile(new URL("./styles.css", import.meta.url), "utf8");
const performanceView = extractSection(html, "performanceView");

assert.match(performanceView, /<div class="layout-grid">/);
assert.match(performanceView, /class="panel performance-panel"/);
assert.match(performanceView, /class="panel comparison-panel"/);
assert.doesNotMatch(performanceView, /id="performanceInfoHeading"/);
assert.doesNotMatch(performanceView, /<\/div>\s*<section\s+class="panel comparison-panel"/);
assert.match(performanceView, /id="performanceOptimizationDetails"/);
assert.match(css, /\.comparison-gallery\s*\{[^}]*display:\s*flex;/s);
assert.match(css, /\.comparison-gallery\s*\{[^}]*flex-direction:\s*column;/s);
assert.match(css, /\.comparison-gallery\s*\{[^}]*overflow:\s*auto;/s);
assert.match(css, /\.comparison-thumb\s*\{[^}]*flex:\s*0 0 auto;/s);
assert.match(css, /\.comparison-thumb-image-wrap\s*\{[^}]*display:\s*block;/s);

function extractSection(source, id) {
  const marker = `<section class="view" id="${id}">`;
  const start = source.indexOf(marker);
  assert.notEqual(start, -1, `${id} section exists`);

  const nextView = source.indexOf('<section class="view"', start + marker.length);
  assert.notEqual(nextView, -1, `${id} has a following view`);
  return source.slice(start, nextView);
}
