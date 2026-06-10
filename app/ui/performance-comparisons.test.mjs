import assert from "node:assert/strict";
import { access } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

import { performanceComparisonScenes } from "./performance-comparisons.js";

const expectedIds = [
  "swamp",
  "swamp2",
  "old-camp-day",
  "old-camp-night",
  "old-camp-night2",
];

assert.deepEqual(
  performanceComparisonScenes.map((scene) => scene.id),
  expectedIds,
);

for (const scene of performanceComparisonScenes) {
  assert.ok(scene.label.length > 0, `${scene.id} needs a label`);
  assert.match(scene.thumbnail.src, /^\.\/assets\/performance-comparisons\/.+\.webp$/);
  assert.match(scene.before.src, /^\.\/assets\/performance-comparisons\/.+\.webp$/);
  assert.match(scene.after.src, /^\.\/assets\/performance-comparisons\/.+\.webp$/);
  assert.equal(scene.thumbnail.alt, `${scene.label} comparison preview`);
  assert.equal(scene.before.label, "Overdose");
  assert.equal(scene.after.label, "Balanced (Overdose)");

  const moduleDir = path.dirname(fileURLToPath(import.meta.url));
  await access(path.join(moduleDir, scene.thumbnail.src));
  await access(path.join(moduleDir, scene.before.src));
  await access(path.join(moduleDir, scene.after.src));
}
