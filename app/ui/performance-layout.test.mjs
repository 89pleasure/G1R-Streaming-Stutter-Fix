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
const overdoseFeatureRow = extractFeatureRow(
  performanceView,
  'id="balancedPerformanceHeading"',
);
const fogFeatureRow = extractFeatureRow(
  performanceView,
  'id="volumetricFogHeading"',
);

assert.match(optimizeStreamingView, /Experimental Stutter Tests/);
assert.match(optimizeStreamingView, /Frame-time stability/);
assert.match(optimizeStreamingView, /id="d3d12PsoCacheToggle"/);
assert.match(optimizeStreamingView, /id="runtimePsoPrecachingToggle"/);
assert.match(optimizeStreamingView, /id="gcSmoothingToggle"/);

assert.match(performanceView, /<div class="layout-grid">/);
assert.doesNotMatch(performanceView, /class="panel performance-panel"/);
assert.equal(countMatches(html, /class="panel feature-card/g), 7);
assert.match(optimizeStreamingView, /class="panel feature-card streaming-fixes-card"/);
assert.match(optimizeStreamingView, /class="feature-card-list experimental-feature-list"/);
assert.equal(countMatches(optimizeStreamingView, /class="panel feature-card experimental-feature-card"/g), 3);
assert.equal(countMatches(performanceView, /class="panel feature-card performance-feature-card"/g), 2);
assert.match(gameTweaksView(), /class="panel feature-card game-feature-card"/);
assert.match(performanceView, /id="openPerformanceComparisonButton"/);
assert.match(overdoseFeatureRow, /id="openPerformanceComparisonButton"/);
assert.match(overdoseFeatureRow, /Visual Comparison/);
assert.match(performanceView, /class="feature-row with-detail"/);
assert.doesNotMatch(performanceView, /class="streaming-detail-title"/);
assert.doesNotMatch(performanceView, /class="streaming-detail-heading"/);
assert.ok(
  performanceView.match(/<div class="streaming-feature-detail">\s*<p(?:\s[^>]*)?>/g)?.length >= 2,
  "Performance detail boxes start directly with explanatory text",
);
assert.match(performanceView, /id="volumetricFogModeControl"/);
assert.match(performanceView, /id="volumetricFogModeNormal"/);
assert.match(performanceView, /id="volumetricFogModeLow"/);
assert.match(performanceView, /id="volumetricFogModeOff"/);
assert.match(performanceView, /class="fog-mode-option recommended"/);
assert.match(performanceView, /data-i18n="feature\.fog\.modeDefault"/);
assert.match(performanceView, /id="volumetricFogHeading"[\s\S]*Volumetric Fog/);
assert.match(performanceView, /data-i18n="feature\.fog\.recommended"/);
assert.match(performanceView, /data-i18n="feature\.fog\.impactBalance"/);
assert.doesNotMatch(fogFeatureRow, /<span>Engine\.ini<\/span>/);
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
assert.match(html, /id="customPoolPanel"/);
assert.match(html, /id="customPoolInput"/);
assert.match(html, /id="customPoolHint"/);
assert.match(html, /id="copyIniButton"/);
assert.match(html, /id="optimizeButton"[\s\S]*id="copyIniButton"/);
assert.match(html, /id="iniCopyModal"/);
assert.match(html, /id="iniCopyFileList"/);
assert.match(html, /data-i18n="preview\.tracking"/);
assert.match(mainJs, /function confirmOverwriteRisks\(\)/);
assert.match(mainJs, /modal\.overwrite\.descriptionMerge/);
assert.match(mainJs, /actions\.useAppSettingsOnly/);
assert.match(mainJs, /function openIniCopyModal\(\)/);
assert.match(mainJs, /function copyIniFileContent\(/);
assert.match(mainJs, /CUSTOM_PRESET_ID/);
assert.match(mainJs, /function selectedCustomPoolMb\(\)/);
assert.match(mainJs, /ini_file_contents/);
assert.match(mainJs, /customPoolMb/);
assert.match(mainJs, /navigator\.clipboard\.writeText/);
assert.match(mainJs, /has_external_settings/);
assert.match(
  mainJs,
  /function overwriteRiskFiles\(\)\s*\{[\s\S]*?file\.has_external_settings[\s\S]*?\n\}/,
);
assert.match(mainJs, /installStrategy/);
assert.match(mainJs, /modification_state/);
assert.match(css, /#performanceView\.view\.active\s*\{[^}]*align-content:\s*stretch;/s);
assert.match(css, /#performanceView\.view\.active\s*\{[^}]*overflow:\s*hidden;/s);
assert.match(css, /\.feature-card-list\s*\{[^}]*display:\s*grid;/s);
assert.match(css, /\.feature-card-list\s*\{[^}]*grid-auto-rows:\s*max-content;/s);
assert.match(css, /\.feature-card\s+\.feature-row\s*\{[^}]*border-bottom:\s*0;/s);
assert.match(css, /#performanceView > \.layout-grid\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\);/s);
assert.match(css, /\.feature-actions\s*\{/);
assert.match(css, /\.segmented-control label\.recommended > span\s*\{[^}]*var\(--app-accent\)/s);
assert.match(css, /\.fog-mode-recommendation\s*\{[^}]*var\(--app-accent-text\)/s);
assert.match(
  css,
  /@media \(max-width: 920px\)\s*\{[\s\S]*?\.feature-row\.with-detail\s*\{[^}]*grid-template-columns:\s*1fr;/s,
);
assert.match(css, /\.modal-dialog\.comparison-gallery-modal-dialog\s*\{/);
assert.match(css, /\.file-state\.warn\s*\{/);
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

function gameTweaksView() {
  return extractSection(html, "gameTweaksView");
}

function countMatches(source, pattern) {
  return source.match(pattern)?.length ?? 0;
}
