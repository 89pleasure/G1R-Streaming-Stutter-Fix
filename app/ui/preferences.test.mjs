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
  assert.equal(preferences.volumetricFogMode, "normal");
  assert.equal(preferences.d3d12PsoCache, false);
  assert.equal(preferences.runtimePsoPrecaching, false);
  assert.equal(preferences.gcSmoothing, false);
  assert.equal(preferences.skipIntroVideos, false);
  assert.equal(preferences.streamingFixes, true);
  assert.equal(preferences.lockEngine, true);
  assert.equal(preferences.lockGame, true);
  assert.equal(preferences.lockScalability, true);
  assert.equal(preferences.customPoolMb, 12288);
}

{
  const storage = createMemoryStorage();

  saveUiPreferences(
    {
      balancedPerformance: true,
      volumetricFogMode: "off",
      d3d12PsoCache: true,
      runtimePsoPrecaching: false,
      gcSmoothing: true,
      skipIntroVideos: true,
      streamingFixes: false,
      lockEngine: false,
      lockGame: true,
      lockScalability: true,
      customPoolMb: 16384,
      selectedPresetId: "20GB_VRAM_10240MB",
      targetDir: "/tmp/G1R/Config/Windows",
    },
    storage,
  );

  assert.deepEqual(loadUiPreferences(storage), {
    balancedPerformance: true,
    volumetricFogMode: "off",
    d3d12PsoCache: true,
    runtimePsoPrecaching: false,
    gcSmoothing: true,
    skipIntroVideos: true,
    streamingFixes: false,
    lockEngine: false,
    lockGame: true,
    lockScalability: true,
    customPoolMb: 16384,
    selectedPresetId: "20GB_VRAM_10240MB",
    targetDir: "/tmp/G1R/Config/Windows",
  });
}

{
  const storage = createMemoryStorage();
  storage.setItem("g1r-optimizer.ui-preferences.v1", "{not valid json");

  assert.equal(loadUiPreferences(storage).balancedPerformance, false);
}

{
  const storage = createMemoryStorage();
  storage.setItem(
    "g1r-optimizer.ui-preferences.v1",
    JSON.stringify({ disableVolumetricFog: true, lowVolumetricFog: true }),
  );

  assert.equal(loadUiPreferences(storage).volumetricFogMode, "off");
}

{
  const storage = createMemoryStorage();
  storage.setItem(
    "g1r-optimizer.ui-preferences.v1",
    JSON.stringify({ lowVolumetricFog: true }),
  );

  assert.equal(loadUiPreferences(storage).volumetricFogMode, "low");
}

{
  const storage = createMemoryStorage();
  storage.setItem(
    "g1r-optimizer.ui-preferences.v1",
    JSON.stringify({ volumetricFogMode: "unsupported" }),
  );

  assert.equal(loadUiPreferences(storage).volumetricFogMode, "normal");
}
