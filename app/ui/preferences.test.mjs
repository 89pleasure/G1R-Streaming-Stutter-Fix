import assert from "node:assert/strict";
import { loadUiPreferences, saveUiPreferences } from "./preferences.js";

function createMemoryStorage() {
  const values = new Map();
  return {
    getItem(key) {
      return values.has(key) ? values.get(key) : null;
    },
    setItem(key, value) {
      values.set(key, value);
    },
  };
}

{
  const storage = createMemoryStorage();
  const preferences = loadUiPreferences(storage);

  assert.equal(preferences.balancedPerformance, false);
  assert.equal(preferences.skipIntroVideos, false);
  assert.equal(preferences.streamingFixes, true);
  assert.equal(preferences.lockEngine, true);
  assert.equal(preferences.lockGame, true);
  assert.equal(preferences.lockScalability, true);
}

{
  const storage = createMemoryStorage();

  saveUiPreferences(
    {
      balancedPerformance: true,
      skipIntroVideos: true,
      streamingFixes: false,
      lockEngine: false,
      lockGame: true,
      lockScalability: true,
      selectedPresetId: "20GB_VRAM_10240MB",
      targetDir: "/tmp/G1R/Config/Windows",
    },
    storage,
  );

  assert.deepEqual(loadUiPreferences(storage), {
    balancedPerformance: true,
    skipIntroVideos: true,
    streamingFixes: false,
    lockEngine: false,
    lockGame: true,
    lockScalability: true,
    selectedPresetId: "20GB_VRAM_10240MB",
    targetDir: "/tmp/G1R/Config/Windows",
  });
}

{
  const storage = createMemoryStorage();
  storage.setItem("g1r-optimizer.ui-preferences.v1", "{not valid json");

  assert.equal(loadUiPreferences(storage).balancedPerformance, false);
}
