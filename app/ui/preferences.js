const PREFERENCES_KEY = "g1r-optimizer.ui-preferences.v1";
const VOLUMETRIC_FOG_MODES = new Set(["normal", "low", "off"]);

const DEFAULT_UI_PREFERENCES = {
  balancedPerformance: false,
  volumetricFogMode: "normal",
  d3d12PsoCache: false,
  runtimePsoPrecaching: false,
  gcSmoothing: false,
  skipIntroVideos: false,
  streamingFixes: true,
  lockEngine: true,
  lockGame: true,
  lockScalability: true,
  selectedPresetId: "",
  targetDir: "",
};

export function loadUiPreferences(storage = window.localStorage) {
  const storedPreferences = readStoredPreferences(storage);
  return {
    balancedPerformance: booleanPreference(
      storedPreferences.balancedPerformance,
      DEFAULT_UI_PREFERENCES.balancedPerformance,
    ),
    volumetricFogMode: volumetricFogModePreference(storedPreferences),
    d3d12PsoCache: booleanPreference(
      storedPreferences.d3d12PsoCache,
      DEFAULT_UI_PREFERENCES.d3d12PsoCache,
    ),
    runtimePsoPrecaching: booleanPreference(
      storedPreferences.runtimePsoPrecaching,
      DEFAULT_UI_PREFERENCES.runtimePsoPrecaching,
    ),
    gcSmoothing: booleanPreference(
      storedPreferences.gcSmoothing,
      DEFAULT_UI_PREFERENCES.gcSmoothing,
    ),
    skipIntroVideos: booleanPreference(
      storedPreferences.skipIntroVideos,
      DEFAULT_UI_PREFERENCES.skipIntroVideos,
    ),
    streamingFixes: booleanPreference(
      storedPreferences.streamingFixes,
      DEFAULT_UI_PREFERENCES.streamingFixes,
    ),
    lockEngine: booleanPreference(
      storedPreferences.lockEngine,
      DEFAULT_UI_PREFERENCES.lockEngine,
    ),
    lockGame: booleanPreference(
      storedPreferences.lockGame,
      DEFAULT_UI_PREFERENCES.lockGame,
    ),
    lockScalability: booleanPreference(
      storedPreferences.lockScalability,
      DEFAULT_UI_PREFERENCES.lockScalability,
    ),
    selectedPresetId: stringPreference(
      storedPreferences.selectedPresetId,
      DEFAULT_UI_PREFERENCES.selectedPresetId,
    ),
    targetDir: stringPreference(
      storedPreferences.targetDir,
      DEFAULT_UI_PREFERENCES.targetDir,
    ),
  };
}

export function saveUiPreferences(preferences, storage = window.localStorage) {
  storage.setItem(
    PREFERENCES_KEY,
    JSON.stringify(loadUiPreferencesFromObject(preferences)),
  );
}

function loadUiPreferencesFromObject(preferences) {
  return {
    balancedPerformance: booleanPreference(
      preferences.balancedPerformance,
      DEFAULT_UI_PREFERENCES.balancedPerformance,
    ),
    volumetricFogMode: volumetricFogModePreference(preferences),
    d3d12PsoCache: booleanPreference(
      preferences.d3d12PsoCache,
      DEFAULT_UI_PREFERENCES.d3d12PsoCache,
    ),
    runtimePsoPrecaching: booleanPreference(
      preferences.runtimePsoPrecaching,
      DEFAULT_UI_PREFERENCES.runtimePsoPrecaching,
    ),
    gcSmoothing: booleanPreference(
      preferences.gcSmoothing,
      DEFAULT_UI_PREFERENCES.gcSmoothing,
    ),
    skipIntroVideos: booleanPreference(
      preferences.skipIntroVideos,
      DEFAULT_UI_PREFERENCES.skipIntroVideos,
    ),
    streamingFixes: booleanPreference(
      preferences.streamingFixes,
      DEFAULT_UI_PREFERENCES.streamingFixes,
    ),
    lockEngine: booleanPreference(
      preferences.lockEngine,
      DEFAULT_UI_PREFERENCES.lockEngine,
    ),
    lockGame: booleanPreference(
      preferences.lockGame,
      DEFAULT_UI_PREFERENCES.lockGame,
    ),
    lockScalability: booleanPreference(
      preferences.lockScalability,
      DEFAULT_UI_PREFERENCES.lockScalability,
    ),
    selectedPresetId: stringPreference(
      preferences.selectedPresetId,
      DEFAULT_UI_PREFERENCES.selectedPresetId,
    ),
    targetDir: stringPreference(preferences.targetDir, DEFAULT_UI_PREFERENCES.targetDir),
  };
}

function readStoredPreferences(storage) {
  try {
    return JSON.parse(storage.getItem(PREFERENCES_KEY) ?? "{}") ?? {};
  } catch {
    return {};
  }
}

function booleanPreference(value, fallback) {
  return typeof value === "boolean" ? value : fallback;
}

function volumetricFogModePreference(preferences) {
  if (VOLUMETRIC_FOG_MODES.has(preferences.volumetricFogMode)) {
    return preferences.volumetricFogMode;
  }

  if (preferences.disableVolumetricFog === true) {
    return "off";
  }

  if (preferences.lowVolumetricFog === true) {
    return "low";
  }

  return DEFAULT_UI_PREFERENCES.volumetricFogMode;
}

function stringPreference(value, fallback) {
  return typeof value === "string" ? value : fallback;
}
