import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const html = await readFile(new URL("./index.html", import.meta.url), "utf8");
const mainJs = await readFile(new URL("./main.js", import.meta.url), "utf8");
const comparisonJs = await readFile(
  new URL("./performance-comparisons.js", import.meta.url),
  "utf8",
);
const css = await readFile(new URL("./styles.css", import.meta.url), "utf8");
const optimizeStreamingView = extractSection(html, "optimizeStreamingView");
const performanceView = extractSection(html, "performanceView");
const performanceHeader = extractBetween(
  performanceView,
  '<div class="panel-header">',
  '<div class="feature-row with-detail">',
);
const overdoseFeatureRow = extractFeatureRow(
  performanceView,
  '<span class="performance-title">Opt-in Overdose quality caps</span>',
);
const fogFeatureRow = extractFeatureRow(
  performanceView,
  '<span class="performance-title">Volumetric Fog</span>',
);

assert.match(optimizeStreamingView, /Experimental Stutter Tests/);
assert.match(optimizeStreamingView, /Frame-time stability/);
assert.match(optimizeStreamingView, /id="d3d12PsoCacheToggle"/);
assert.match(optimizeStreamingView, /id="runtimePsoPrecachingToggle"/);
assert.match(optimizeStreamingView, /id="gcSmoothingToggle"/);

assert.match(performanceView, /<div class="layout-grid">/);
assert.match(performanceView, /class="panel performance-panel"/);
assert.match(performanceView, /id="openPerformanceComparisonButton"/);
assert.doesNotMatch(performanceHeader, /id="openPerformanceComparisonButton"/);
assert.doesNotMatch(performanceHeader, /Overdose only/);
assert.match(overdoseFeatureRow, /id="openPerformanceComparisonButton"/);
assert.match(overdoseFeatureRow, /Visual Comparison/);
assert.match(overdoseFeatureRow, /Overdose only/);
assert.match(performanceView, /class="feature-row with-detail"/);
assert.match(performanceView, /<span class="streaming-detail-title">Overdose profile caps<\/span>/);
assert.match(performanceView, /id="volumetricFogModeControl"/);
assert.match(performanceView, /id="volumetricFogModeNormal"/);
assert.match(performanceView, /id="volumetricFogModeLow"/);
assert.match(performanceView, /id="volumetricFogModeOff"/);
assert.match(performanceView, /class="fog-mode-option recommended"/);
assert.match(performanceView, /<span class="fog-mode-label">Default<\/span>/);
assert.match(performanceView, /<span class="performance-title">Volumetric Fog<\/span>/);
assert.match(performanceView, /<span class="fog-mode-recommendation">Recommended<\/span>/);
assert.match(performanceView, /<span>Best balance<\/span>/);
assert.doesNotMatch(fogFeatureRow, /<span>Engine\.ini<\/span>/);
assert.match(performanceView, /<span class="streaming-detail-title">Volumetric Fog mode<\/span>/);
assert.doesNotMatch(performanceView, /id="disableVolumetricFogToggle"/);
assert.doesNotMatch(performanceView, /id="lowVolumetricFogToggle"/);
assert.doesNotMatch(performanceView, /class="panel comparison-panel"/);
assert.doesNotMatch(performanceView, /id="performanceComparisonGallery"/);
assert.doesNotMatch(performanceView, /id="performanceInfoHeading"/);
assert.doesNotMatch(performanceView, /<\/div>\s*<section\s+class="panel comparison-panel"/);
assert.doesNotMatch(performanceView, /id="performanceOptimizationDetails"/);
assert.doesNotMatch(performanceView, /id="d3d12PsoCacheToggle"/);
assert.doesNotMatch(performanceView, /id="runtimePsoPrecachingToggle"/);
assert.doesNotMatch(performanceView, /id="gcSmoothingToggle"/);
assert.match(html, /id="comparisonGalleryModal"/);
assert.match(html, /id="comparisonGalleryModalClose"/);
assert.match(html, /id="performanceComparisonGallery"/);
assert.match(css, /#performanceView\.view\.active\s*\{[^}]*align-content:\s*stretch;/s);
assert.match(css, /#performanceView\.view\.active\s*\{[^}]*overflow:\s*hidden;/s);
assert.match(css, /#performanceView \.performance-panel\s*\{[^}]*align-self:\s*stretch;/s);
assert.match(css, /#performanceView \.performance-panel\s*\{[^}]*overflow:\s*auto;/s);
assert.match(css, /#performanceView \.performance-panel\s*\{[^}]*scrollbar-gutter:\s*stable;/s);
assert.match(css, /#performanceView > \.layout-grid\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\);/s);
assert.match(css, /\.feature-actions\s*\{/);
assert.match(css, /\.segmented-control label\.recommended > span\s*\{[^}]*#2c8f6d/s);
assert.match(css, /\.fog-mode-recommendation\s*\{[^}]*#1d7658/s);
assert.match(css, /\.modal-dialog\.comparison-gallery-modal-dialog\s*\{/);
assert.match(css, /\.comparison-gallery\s*\{[^}]*display:\s*grid;/s);
assert.match(css, /\.comparison-gallery\s*\{[^}]*grid-template-columns:\s*repeat\(auto-fit,\s*minmax\(240px,\s*1fr\)\);/s);
assert.match(css, /\.comparison-gallery\s*\{[^}]*overflow:\s*auto;/s);
assert.match(css, /\.comparison-thumb\s*\{[^}]*flex:\s*0 0 auto;/s);
assert.match(css, /\.comparison-thumb-image-wrap\s*\{[^}]*display:\s*block;/s);
assert.doesNotMatch(html, /Cine/);
assert.doesNotMatch(mainJs, /Cine/);
assert.doesNotMatch(comparisonJs, /Cine/);

function extractSection(source, id) {
  const sectionPattern = new RegExp(`<section class="view(?: active)?" id="${id}">`);
  const match = sectionPattern.exec(source);
  const start = match?.index ?? -1;
  assert.notEqual(start, -1, `${id} section exists`);

  const nextView = source.indexOf('<section class="view"', start + match[0].length);
  assert.notEqual(nextView, -1, `${id} has a following view`);
  return source.slice(start, nextView);
}

function extractBetween(source, startMarker, endMarker) {
  const start = source.indexOf(startMarker);
  assert.notEqual(start, -1, `${startMarker} exists`);
  const end = source.indexOf(endMarker, start + startMarker.length);
  assert.notEqual(end, -1, `${endMarker} exists after ${startMarker}`);
  return source.slice(start, end);
}

function extractFeatureRow(source, marker) {
  const markerIndex = source.indexOf(marker);
  assert.notEqual(markerIndex, -1, `${marker} exists`);

  const rowStart = source.lastIndexOf('<div class="feature-row with-detail">', markerIndex);
  assert.notEqual(rowStart, -1, `${marker} has a feature row`);

  const rowEnd = source.indexOf('<div class="feature-row with-detail">', markerIndex);
  if (rowEnd === -1) {
    return source.slice(rowStart);
  }

  return source.slice(rowStart, rowEnd);
}
