const PREFERENCES_KEY = "g1r-optimizer.ui-preferences.v1";

const DEFAULT_UI_PREFERENCES = {
  balancedPerformance: false,
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

function stringPreference(value, fallback) {
  return typeof value === "string" ? value : fallback;
}
