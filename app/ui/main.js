import { loadUiPreferences, saveUiPreferences } from "./preferences.js";
import { performanceComparisonScenes } from "./performance-comparisons.js";
import {
  LANGUAGE_AUTO,
  applyTranslationsToDocument,
  createTranslator,
  languageOptions,
  resolveLanguage,
} from "./i18n.js";

const CUSTOM_PRESET_ID = "CUSTOM_POOL";
const DEFAULT_CUSTOM_POOL_MB = 12288;
const MIN_CUSTOM_POOL_MB = 512;
const MAX_CUSTOM_POOL_MB = 65536;
const CUSTOM_POOL_STEP_MB = 256;

const state = {
  presets: [],
  candidates: [],
  launchCandidates: [],
  presetRoot: "",
  selectedPresetId: "",
  selectedLaunchTargetId: "",
  recommendation: null,
  hardware: null,
  targetDir: "",
  manualExecutablePath: "",
  customPoolMb: DEFAULT_CUSTOM_POOL_MB,
  preview: [],
  iniCopyFiles: [],
  backups: [],
  currentView: "optimizeStreaming",
  selectedComparisonSceneId: performanceComparisonScenes[0]?.id ?? "",
  comparisonPosition: 50,
  languagePreference: LANGUAGE_AUTO,
  language: "en",
  busy: false,
};

const samplePresets = [
  { id: "04GB_VRAM_1536MB", label: "4 GB VRAM / 1536 MB pool", vram_gb: 4, pool_mb: 1536 },
  { id: "06GB_VRAM_3072MB", label: "6 GB VRAM / 3072 MB pool", vram_gb: 6, pool_mb: 3072 },
  { id: "08GB_VRAM_4096MB", label: "8 GB VRAM / 4096 MB pool", vram_gb: 8, pool_mb: 4096 },
  { id: "10GB_VRAM_5120MB", label: "10 GB VRAM / 5120 MB pool", vram_gb: 10, pool_mb: 5120 },
  { id: "12GB_VRAM_6144MB", label: "12 GB VRAM / 6144 MB pool", vram_gb: 12, pool_mb: 6144 },
  { id: "16GB_VRAM_8192MB", label: "16 GB VRAM / 8192 MB pool", vram_gb: 16, pool_mb: 8192 },
  { id: "20GB_VRAM_10240MB", label: "20 GB VRAM / 10240 MB pool", vram_gb: 20, pool_mb: 10240 },
  { id: "24GB_VRAM_12288MB", label: "24+ GB VRAM / 12288 MB pool", vram_gb: 24, pool_mb: 12288 },
];

const viewTitleKeys = {
  optimizeStreaming: "views.optimizeStreaming",
  performance: "views.performance",
  gameTweaks: "views.gameTweaks",
  backups: "views.backups",
  diagnostics: "views.diagnostics",
  settings: "views.settings",
};

const viewSubtitleKeys = {
  optimizeStreaming: "views.optimizeStreaming.subtitle",
  performance: "views.performance.subtitle",
  gameTweaks: "views.gameTweaks.subtitle",
  backups: "views.backups.subtitle",
  diagnostics: "views.diagnostics.subtitle",
  settings: "views.settings.subtitle",
};

const viewsWithPreview = new Set(["optimizeStreaming", "performance", "gameTweaks"]);

const elements = {};
let translate = createTranslator(state.language);
let confirmModalResolve = null;
let confirmModalPreviousFocus = null;
let confirmModalActionButtons = [];
let iniCopyModalPreviousFocus = null;
let comparisonGalleryModalPreviousFocus = null;
let comparisonModalPreviousFocus = null;
let resultClearTimer = null;

window.addEventListener("DOMContentLoaded", () => {
  bindElements();
  applyStoredPreferences();
  bindEvents();
  loadAppState();
});

function bindElements() {
  for (const id of [
    "runtimeStatus",
    "workspace",
    "pageTitle",
    "pageSubtitle",
    "refreshButton",
    "presetPanel",
    "streamingFixesToggle",
    "streamingStatus",
    "presetCount",
    "presetGrid",
    "customPoolPanel",
    "customPoolInput",
    "customPoolHint",
    "recommendationSummary",
    "candidateSelect",
    "targetInput",
    "browseTargetButton",
    "pathStatus",
    "launchTargetSelect",
    "manualExecutableInput",
    "browseExecutableButton",
    "launchStatus",
    "lockEngineToggle",
    "lockGameToggle",
    "lockScalabilityToggle",
    "balancedPerformanceToggle",
    "volumetricFogModeControl",
    "volumetricFogModeNormal",
    "volumetricFogModeLow",
    "volumetricFogModeOff",
    "d3d12PsoCacheToggle",
    "runtimePsoPrecachingToggle",
    "gcSmoothingToggle",
    "performanceStatus",
    "openPerformanceComparisonButton",
    "comparisonGalleryModal",
    "comparisonGalleryModalClose",
    "performanceComparisonGallery",
    "comparisonModal",
    "comparisonModalTitle",
    "comparisonModalDescription",
    "comparisonModalClose",
    "performanceComparisonStage",
    "performanceComparisonBeforeImage",
    "performanceComparisonAfterImage",
    "performanceComparisonBeforeLabel",
    "performanceComparisonAfterLabel",
    "performanceComparisonRange",
    "skipIntroVideosToggle",
    "gameTweaksStatus",
    "previewPanel",
    "previewStatus",
    "previewRows",
    "copyIniButton",
    "playButton",
    "optimizeButton",
    "optimizeStatus",
    "lastResult",
    "loadBackupsButton",
    "resetVanillaButton",
    "backupList",
    "languageSelect",
    "presetRootValue",
    "selectedPresetValue",
    "recommendedPresetValue",
    "selectedTargetValue",
    "activityLog",
    "confirmModal",
    "confirmModalTitle",
    "confirmModalDescription",
    "confirmModalCancel",
    "confirmModalConfirm",
    "iniCopyModal",
    "iniCopyModalClose",
    "iniCopyFileList",
  ]) {
    elements[id] = document.getElementById(id);
  }
}

function populateLanguageSelect() {
  elements.languageSelect.innerHTML = "";
  for (const option of languageOptions) {
    const languageOption = document.createElement("option");
    languageOption.value = option.value;
    elements.languageSelect.appendChild(languageOption);
  }
}

function applyLanguagePreference(preference, options = {}) {
  const { render = true } = options;
  const supportedPreference = languageOptions.some((option) => option.value === preference)
    ? preference
    : LANGUAGE_AUTO;

  state.languagePreference = supportedPreference;
  state.language = resolveLanguage(supportedPreference);
  translate = createTranslator(state.language);
  document.documentElement.lang = state.language;
  refreshLanguageSelect();
  applyTranslationsToDocument(document, translate);

  if (render) {
    renderAll();
  }
}

function refreshLanguageSelect() {
  elements.languageSelect.value = state.languagePreference;
  elements.languageSelect.querySelectorAll("option").forEach((option) => {
    const languageOption = languageOptions.find((item) => item.value === option.value);
    option.textContent = translate(languageOption?.labelKey ?? "language.auto");
  });
}

function applyStoredPreferences() {
  const preferences = loadUiPreferences();
  populateLanguageSelect();
  applyLanguagePreference(preferences.language, { render: false });
  state.selectedPresetId = preferences.selectedPresetId;
  state.targetDir = preferences.targetDir;
  state.selectedLaunchTargetId = preferences.selectedLaunchTargetId;
  state.manualExecutablePath = preferences.manualExecutablePath;
  state.customPoolMb = preferences.customPoolMb;
  elements.streamingFixesToggle.checked = preferences.streamingFixes;
  elements.balancedPerformanceToggle.checked = preferences.balancedPerformance;
  setVolumetricFogMode(preferences.volumetricFogMode);
  elements.d3d12PsoCacheToggle.checked = preferences.d3d12PsoCache;
  elements.runtimePsoPrecachingToggle.checked = preferences.runtimePsoPrecaching;
  elements.gcSmoothingToggle.checked = preferences.gcSmoothing;
  elements.skipIntroVideosToggle.checked = preferences.skipIntroVideos;
  elements.lockEngineToggle.checked = preferences.lockEngine;
  elements.lockGameToggle.checked = preferences.lockGame;
  elements.lockScalabilityToggle.checked = preferences.lockScalability;
}

function persistUiPreferences() {
  saveUiPreferences({
    balancedPerformance: elements.balancedPerformanceToggle.checked,
    volumetricFogMode: selectedVolumetricFogMode(),
    d3d12PsoCache: elements.d3d12PsoCacheToggle.checked,
    runtimePsoPrecaching: elements.runtimePsoPrecachingToggle.checked,
    gcSmoothing: elements.gcSmoothingToggle.checked,
    skipIntroVideos: elements.skipIntroVideosToggle.checked,
    streamingFixes: elements.streamingFixesToggle.checked,
    lockEngine: elements.lockEngineToggle.checked,
    lockGame: elements.lockGameToggle.checked,
    lockScalability: elements.lockScalabilityToggle.checked,
    customPoolMb: state.customPoolMb,
    selectedPresetId: state.selectedPresetId,
    targetDir: state.targetDir,
    selectedLaunchTargetId: state.selectedLaunchTargetId,
    manualExecutablePath: state.manualExecutablePath,
    language: state.languagePreference,
  });
}

function bindEvents() {
  const refreshPreviewDebounced = debounce(refreshPreview, 250);

  document.querySelectorAll(".nav-item[data-view]").forEach((button) => {
    button.addEventListener("click", () => switchView(button.dataset.view));
  });

  elements.refreshButton.addEventListener("click", loadAppState);
  elements.loadBackupsButton.addEventListener("click", loadBackups);
  elements.resetVanillaButton.addEventListener("click", resetToVanilla);
  elements.copyIniButton.addEventListener("click", openIniCopyModal);
  elements.playButton.addEventListener("click", launchGame);
  elements.optimizeButton.addEventListener("click", optimizeSelectedPreset);
  elements.languageSelect.addEventListener("change", () => {
    applyLanguagePreference(elements.languageSelect.value, { render: true });
    persistUiPreferences();
  });
  elements.confirmModalCancel.addEventListener("click", () => closeConfirmModal(false));
  elements.confirmModalConfirm.addEventListener("click", () =>
    closeConfirmModal(elements.confirmModalConfirm.dataset.modalAction ?? true),
  );
  elements.confirmModal.addEventListener("click", (event) => {
    if (event.target === elements.confirmModal) {
      closeConfirmModal(false);
    }
  });
  elements.openPerformanceComparisonButton.addEventListener(
    "click",
    openPerformanceComparisonGalleryModal,
  );
  elements.comparisonGalleryModalClose.addEventListener("click", closeComparisonGalleryModal);
  elements.comparisonGalleryModal.addEventListener("click", (event) => {
    if (event.target === elements.comparisonGalleryModal) {
      closeComparisonGalleryModal();
    }
  });
  elements.iniCopyModalClose.addEventListener("click", closeIniCopyModal);
  elements.iniCopyModal.addEventListener("click", (event) => {
    if (event.target === elements.iniCopyModal) {
      closeIniCopyModal();
    }
  });
  elements.comparisonModalClose.addEventListener("click", closeComparisonModal);
  elements.comparisonModal.addEventListener("click", (event) => {
    if (event.target === elements.comparisonModal) {
      closeComparisonModal();
    }
  });
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") {
      return;
    }

    if (!elements.comparisonModal.hidden) {
      closeComparisonModal();
    } else if (!elements.iniCopyModal.hidden) {
      closeIniCopyModal();
    } else if (!elements.comparisonGalleryModal.hidden) {
      closeComparisonGalleryModal();
    } else if (!elements.confirmModal.hidden) {
      closeConfirmModal(false);
    }
  });

  elements.candidateSelect.addEventListener("change", () => {
    const candidate = state.candidates.find(
      (item) => item.path === elements.candidateSelect.value,
    );
    applyTargetDir(candidate?.path ?? elements.candidateSelect.value);
    refreshPreview();
    loadBackups();
  });

  elements.targetInput.addEventListener("input", () => {
    applyTargetDir(elements.targetInput.value);
    refreshPreviewDebounced();
  });

  elements.browseTargetButton.addEventListener("click", browseTargetFolder);
  elements.browseExecutableButton.addEventListener("click", browseExecutableFile);

  elements.launchTargetSelect.addEventListener("change", () => {
    state.selectedLaunchTargetId = elements.launchTargetSelect.value;
    persistUiPreferences();
    renderLaunchSettings();
    updateActionButtons();
  });

  elements.manualExecutableInput.addEventListener("input", () => {
    state.manualExecutablePath = elements.manualExecutableInput.value;
    if (state.manualExecutablePath.trim()) {
      state.selectedLaunchTargetId = manualLaunchTargetId(state.manualExecutablePath);
    }
    persistUiPreferences();
    renderLaunchSettings({ syncInputValue: false });
    updateActionButtons();
  });

  elements.customPoolInput.addEventListener("input", () => {
    const poolMb = selectedCustomPoolMb();
    if (poolMb !== null) {
      state.customPoolMb = poolMb;
      persistUiPreferences();
    }
    renderCustomPoolState(false);
    renderDiagnostics();
    refreshPreviewDebounced();
  });

  elements.lockEngineToggle.addEventListener("change", () => {
    persistUiPreferences();
    refreshPreview();
  });
  elements.lockGameToggle.addEventListener("change", () => {
    persistUiPreferences();
    refreshPreview();
  });
  elements.lockScalabilityToggle.addEventListener("change", () => {
    persistUiPreferences();
    refreshPreview();
  });
  elements.streamingFixesToggle.addEventListener("change", () => {
    persistUiPreferences();
    renderStreamingState();
    renderPresets();
    renderRecommendationSummary();
    refreshPreview();
  });
  elements.balancedPerformanceToggle.addEventListener("change", () => {
    persistUiPreferences();
    renderPerformanceState();
    refreshPreview();
  });
  for (const toggle of [
    elements.volumetricFogModeNormal,
    elements.volumetricFogModeLow,
    elements.volumetricFogModeOff,
  ]) {
    toggle.addEventListener("change", () => {
      persistUiPreferences();
      renderPerformanceState();
      refreshPreview();
    });
  }
  for (const toggle of [
    elements.d3d12PsoCacheToggle,
    elements.runtimePsoPrecachingToggle,
    elements.gcSmoothingToggle,
  ]) {
    toggle.addEventListener("change", () => {
      persistUiPreferences();
      refreshPreview();
    });
  }
  elements.performanceComparisonRange.addEventListener("input", () => {
    state.comparisonPosition = Number(elements.performanceComparisonRange.value);
    updatePerformanceComparisonPosition();
  });
  elements.skipIntroVideosToggle.addEventListener("change", () => {
    persistUiPreferences();
    renderGameTweaksState();
    refreshPreview();
  });
}

async function browseTargetFolder() {
  if (!hasTauriApi() && !window.__TAURI__?.dialog?.open) {
    appendLog(translate("logs.folderPickerUnavailable"));
    return;
  }

  setBusy(true);
  try {
    const selectedFolder = await openTargetFolderDialog();
    if (!selectedFolder || Array.isArray(selectedFolder)) {
      return;
    }

    applyTargetDir(selectedFolder);
    await refreshPreview();
    await loadBackups();
  } catch (error) {
    appendLog(translate("logs.folderPickerFailed", { error }));
  } finally {
    setBusy(false);
  }
}

async function browseExecutableFile() {
  if (!hasTauriApi() && !window.__TAURI__?.dialog?.open) {
    appendLog(translate("logs.filePickerUnavailable"));
    return;
  }

  setBusy(true);
  try {
    const selectedFile = await openExecutableDialog();
    if (!selectedFile || Array.isArray(selectedFile)) {
      return;
    }

    state.manualExecutablePath = selectedFile;
    state.selectedLaunchTargetId = manualLaunchTargetId(selectedFile);
    persistUiPreferences();
    renderLaunchSettings();
  } catch (error) {
    appendLog(translate("logs.filePickerFailed", { error }));
  } finally {
    setBusy(false);
  }
}

async function loadAppState() {
  setBusy(true);
  try {
    const appState = await invokeCommand("get_app_state");
    state.presets = appState.presets;
    state.candidates = appState.candidates;
    state.launchCandidates = appState.launch_candidates ?? [];
    state.presetRoot = appState.preset_root;
    state.hardware = appState.hardware;
    state.recommendation = appState.recommendation ?? null;
    if (
      state.selectedPresetId !== CUSTOM_PRESET_ID &&
      !state.presets.some((preset) => preset.id === state.selectedPresetId)
    ) {
      state.selectedPresetId = pickDefaultPreset(state.presets)?.id || "";
    }

    const bestCandidate = state.candidates.find((candidate) => candidate.exists) ?? state.candidates[0];
    if (!state.targetDir && bestCandidate) {
      state.targetDir = bestCandidate.path;
    }

    if (!availableLaunchTargets().some((target) => target.id === state.selectedLaunchTargetId)) {
      state.selectedLaunchTargetId = preferredLaunchTarget()?.id ?? "";
    }

    elements.runtimeStatus.textContent = hasTauriApi()
      ? translate("runtime.tauri")
      : translate("runtime.static");
    renderAll();
    await refreshPreview();
    await loadBackups();
  } catch (error) {
    appendLog(translate("logs.loadFailed", { error }));
    renderAll();
  } finally {
    setBusy(false);
  }
}

function renderAll() {
  renderStreamingState();
  renderPresets();
  renderRecommendationSummary();
  renderCandidates();
  renderPathStatus();
  renderLaunchSettings();
  renderPerformanceState();
  renderPerformanceComparison();
  renderGameTweaksState();
  renderPageChrome();
  renderPreview();
  renderBackups();
  renderDiagnostics();
}

function renderPresets() {
  elements.presetCount.textContent = translate("preset.count", { count: state.presets.length + 1 });
  elements.presetGrid.innerHTML = "";

  state.presets.forEach((preset) => {
    const isRecommended = preset.id === state.recommendation?.preset_id;
    const button = document.createElement("button");
    button.type = "button";
    button.className = [
      "preset-option",
      preset.id === state.selectedPresetId ? "active" : "",
      isRecommended ? "recommended" : "",
    ]
      .filter(Boolean)
      .join(" ");
    button.innerHTML = `
      <span class="preset-heading-row">
        <span class="preset-vram">${preset.vram_gb} GB</span>
        ${isRecommended ? recommendedBadgeMarkup() : ""}
      </span>
        <span class="preset-pool">${escapeHtml(poolLabel(preset.pool_mb))}</span>
    `;
    button.disabled = !streamingFixesEnabled();
    button.addEventListener("click", () => {
      state.selectedPresetId = preset.id;
      persistUiPreferences();
      renderPresets();
      renderDiagnostics();
      refreshPreview();
    });

    elements.presetGrid.appendChild(button);
  });

  const customButton = document.createElement("button");
  customButton.type = "button";
  customButton.className = [
    "preset-option",
    "custom-preset-option",
    isCustomPresetSelected() ? "active" : "",
  ]
    .filter(Boolean)
    .join(" ");
  customButton.innerHTML = `
    <span class="preset-heading-row">
      <span class="preset-vram">${escapeHtml(translate("preset.custom"))}</span>
    </span>
    <span class="preset-pool">${customPoolLabel()}</span>
  `;
  customButton.disabled = !streamingFixesEnabled();
  customButton.addEventListener("click", () => {
    state.selectedPresetId = CUSTOM_PRESET_ID;
    persistUiPreferences();
    renderPresets();
    renderRecommendationSummary();
    renderDiagnostics();
    refreshPreview();
  });
  elements.presetGrid.appendChild(customButton);

  renderCustomPoolState(true);
}

function renderRecommendationSummary() {
  if (!streamingFixesEnabled()) {
    elements.recommendationSummary.textContent =
      translate("recommendation.whenOff");
    elements.recommendationSummary.className = "recommendation-summary muted";
    return;
  }

  if (isCustomPresetSelected()) {
    elements.recommendationSummary.textContent =
      translate("recommendation.custom");
    elements.recommendationSummary.className = "recommendation-summary";
    return;
  }

  const recommendation = state.recommendation;
  if (!recommendation) {
    elements.recommendationSummary.textContent =
      translate("recommendation.unknownHardware");
    elements.recommendationSummary.className = "recommendation-summary muted";
    return;
  }

  elements.recommendationSummary.textContent = recommendationMessage(recommendation);
  elements.recommendationSummary.className = "recommendation-summary";
}

function renderCandidates() {
  elements.candidateSelect.innerHTML = "";

  if (state.candidates.length === 0) {
    const option = document.createElement("option");
    option.value = "";
    option.textContent = translate("candidate.noLocations");
    elements.candidateSelect.appendChild(option);
  } else {
    state.candidates.forEach((candidate) => {
      const option = document.createElement("option");
      option.value = candidate.path;
      option.textContent = `${candidate.label} - ${
        candidate.exists ? translate("candidate.found") : translate("candidate.notFound")
      }`;
      option.selected = candidate.path === state.targetDir;
      elements.candidateSelect.appendChild(option);
    });
  }

  elements.targetInput.value = state.targetDir;
}

function applyTargetDir(targetDir) {
  state.targetDir = targetDir;
  elements.targetInput.value = state.targetDir;
  elements.candidateSelect.value = state.targetDir;
  persistUiPreferences();
  renderPathStatus();
  renderDiagnostics();
}

function renderPathStatus() {
  const candidate = state.candidates.find((item) => item.path === state.targetDir);
  if (!state.targetDir.trim()) {
    elements.pathStatus.textContent = translate("pathStatus.missing");
    elements.pathStatus.className = "pill bad";
  } else if (candidate?.exists) {
    elements.pathStatus.textContent = translate("pathStatus.found");
    elements.pathStatus.className = "pill good";
  } else if (candidate) {
    elements.pathStatus.textContent = translate("pathStatus.canCreate");
    elements.pathStatus.className = "pill warn";
  } else {
    elements.pathStatus.textContent = translate("pathStatus.manual");
    elements.pathStatus.className = "pill";
  }
}

function renderLaunchSettings(options = {}) {
  const { syncInputValue = true } = options;
  const targets = availableLaunchTargets();
  elements.launchTargetSelect.innerHTML = "";

  if (targets.length === 0) {
    const option = document.createElement("option");
    option.value = "";
    option.textContent = translate("launch.noTargets");
    elements.launchTargetSelect.appendChild(option);
  } else {
    for (const target of targets) {
      const option = document.createElement("option");
      option.value = target.id;
      option.textContent = launchTargetLabel(target);
      option.selected = target.id === state.selectedLaunchTargetId;
      elements.launchTargetSelect.appendChild(option);
    }
  }

  elements.launchTargetSelect.value = state.selectedLaunchTargetId;
  if (syncInputValue) {
    elements.manualExecutableInput.value = state.manualExecutablePath;
  }
  renderLaunchStatus();
}

function renderLaunchStatus() {
  const target = selectedLaunchTarget();
  if (!target) {
    elements.launchStatus.textContent = translate("pathStatus.missing");
    elements.launchStatus.className = "pill bad";
  } else if (target.source === "manual") {
    elements.launchStatus.textContent = translate("pathStatus.manual");
    elements.launchStatus.className = "pill warn";
  } else if (target.exists) {
    elements.launchStatus.textContent = translate("pathStatus.found");
    elements.launchStatus.className = "pill good";
  } else {
    elements.launchStatus.textContent = translate("pathStatus.missing");
    elements.launchStatus.className = "pill bad";
  }
}

async function refreshPreview() {
  if (!selectedPresetIdForCommand() || !state.targetDir.trim()) {
    state.preview = [];
    elements.previewStatus.textContent = translate("status.waiting");
    elements.previewStatus.className = "pill";
    renderPreview();
    return;
  }

  if (!customPoolSelectionValid()) {
    state.preview = [];
    elements.previewStatus.textContent = translate("status.invalid");
    elements.previewStatus.className = "pill bad";
    renderPreview();
    return;
  }

  try {
    state.preview = await invokeCommand("preview_install", {
      presetId: selectedPresetIdForCommand(),
      targetDir: state.targetDir,
      lockEngine: elements.lockEngineToggle.checked,
      lockGame: elements.lockGameToggle.checked,
      lockScalability: elements.lockScalabilityToggle.checked,
      customPoolMb: selectedCustomPoolMb(),
      streamingFixes: streamingFixesEnabled(),
      balancedPerformance: elements.balancedPerformanceToggle.checked,
      disableVolumetricFog: selectedVolumetricFogMode() === "off",
      lowVolumetricFog: selectedVolumetricFogMode() === "low",
      skipIntroVideos: elements.skipIntroVideosToggle.checked,
      d3d12PsoCache: elements.d3d12PsoCacheToggle.checked,
      runtimePsoPrecaching: elements.runtimePsoPrecachingToggle.checked,
      gcSmoothing: elements.gcSmoothingToggle.checked,
    });
    elements.previewStatus.textContent = translate("status.ready");
    elements.previewStatus.className = "pill good";
  } catch (error) {
    state.preview = [];
    elements.previewStatus.textContent = translate("status.error");
    elements.previewStatus.className = "pill bad";
    appendLog(translate("logs.previewFailed", { error }));
  }

  renderPreview();
}

function renderPreview() {
  elements.previewRows.innerHTML = "";

  if (state.preview.length === 0) {
    const row = document.createElement("tr");
    row.className = "empty-row";
    row.innerHTML = `<td colspan="7">${emptyPreviewMessage()}</td>`;
    elements.previewRows.appendChild(row);
    updateActionButtons();
    return;
  }

  state.preview.forEach((file) => {
    const row = document.createElement("tr");
    row.innerHTML = `
      <td>${escapeHtml(file.file_name)}</td>
      <td>${formatModificationState(file.modification_state)}</td>
      <td>${formatPool(file.current_pool_mb)}</td>
      <td>${formatPool(file.preset_pool_mb)}</td>
      <td>${formatTweaks(file)}</td>
      <td>${yesNo(file.will_backup)}</td>
      <td>${yesNo(file.will_set_read_only)}</td>
    `;
    elements.previewRows.appendChild(row);
  });

  updateActionButtons();
}

async function openIniCopyModal() {
  if (
    !selectedPresetIdForCommand() ||
    !selectedOptimizerChangesEnabled() ||
    !customPoolSelectionValid()
  ) {
    return;
  }

  setBusy(true);
  try {
    state.iniCopyFiles = await invokeCommand("ini_file_contents", selectedIniContentArgs());
    renderIniCopyModal();
    iniCopyModalPreviousFocus = document.activeElement;
    elements.iniCopyModal.hidden = false;
    document.body.classList.add("modal-open");

    window.requestAnimationFrame(() => {
      elements.iniCopyModalClose.focus();
    });
  } catch (error) {
    showActionResult("error", translate("status.error"), translate("logs.copyPreviewFailed"), false);
    appendLog(translate("logs.previewFailed", { error }));
  } finally {
    setBusy(false);
  }
}

function renderIniCopyModal() {
  elements.iniCopyFileList.innerHTML = "";

  if (state.iniCopyFiles.length === 0) {
    const empty = document.createElement("div");
    empty.className = "ini-copy-empty";
    empty.textContent = translate("iniCopy.empty");
    elements.iniCopyFileList.appendChild(empty);
    return;
  }

  for (const file of state.iniCopyFiles) {
    const item = document.createElement("article");
    item.className = "ini-copy-file";

    const header = document.createElement("div");
    header.className = "ini-copy-file-header";

    const title = document.createElement("h3");
    title.textContent = file.file_name;

    const copyButton = document.createElement("button");
    copyButton.className = "secondary-button compact";
    copyButton.type = "button";
    copyButton.textContent = translate("actions.copy");
    copyButton.addEventListener("click", () => copyIniFileContent(file, copyButton));

    const textarea = document.createElement("textarea");
    textarea.className = "ini-copy-content";
    textarea.spellcheck = false;
    textarea.readOnly = true;
    textarea.value = file.content;

    header.append(title, copyButton);
    item.append(header, textarea);
    elements.iniCopyFileList.appendChild(item);
  }
}

async function copyIniFileContent(file, button) {
  try {
    await navigator.clipboard.writeText(file.content);
    button.textContent = translate("actions.copied");
    appendLog(translate("logs.copySuccess", { fileName: file.file_name }));
    window.setTimeout(() => {
      button.textContent = translate("actions.copy");
    }, 1800);
  } catch (error) {
    button.textContent = translate("actions.failed");
    appendLog(translate("logs.copyFailed", { fileName: file.file_name, error }));
  }
}

function closeIniCopyModal() {
  elements.iniCopyModal.hidden = true;
  state.iniCopyFiles = [];
  elements.iniCopyFileList.innerHTML = "";
  if (
    elements.comparisonModal.hidden &&
    elements.comparisonGalleryModal.hidden &&
    elements.confirmModal.hidden
  ) {
    document.body.classList.remove("modal-open");
  }

  if (iniCopyModalPreviousFocus instanceof HTMLElement) {
    iniCopyModalPreviousFocus.focus();
  }
  iniCopyModalPreviousFocus = null;
}

async function optimizeSelectedPreset() {
  if (!selectedPresetIdForCommand() || !state.targetDir.trim() || !customPoolSelectionValid()) {
    return;
  }

  await refreshPreview();
  if (state.preview.length === 0) {
    return;
  }

  const installStrategy = await confirmOverwriteRisks();
  if (!installStrategy) {
    return;
  }

  setBusy(true);
  try {
    const report = await invokeCommand("install_preset", {
      presetId: selectedPresetIdForCommand(),
      targetDir: state.targetDir,
      lockEngine: elements.lockEngineToggle.checked,
      lockGame: elements.lockGameToggle.checked,
      lockScalability: elements.lockScalabilityToggle.checked,
      customPoolMb: selectedCustomPoolMb(),
      streamingFixes: streamingFixesEnabled(),
      balancedPerformance: elements.balancedPerformanceToggle.checked,
      disableVolumetricFog: selectedVolumetricFogMode() === "off",
      lowVolumetricFog: selectedVolumetricFogMode() === "low",
      skipIntroVideos: elements.skipIntroVideosToggle.checked,
      d3d12PsoCache: elements.d3d12PsoCacheToggle.checked,
      runtimePsoPrecaching: elements.runtimePsoPrecachingToggle.checked,
      gcSmoothing: elements.gcSmoothingToggle.checked,
      installStrategy,
    });
    showActionResult(
      "success",
      translate("status.success"),
      translate("logs.installSuccess", {
        presetId: report.preset_id,
        targetDir: report.target_dir,
      }),
      true,
    );
    appendLog(
      translate("logs.installSuccess", { presetId: report.preset_id, targetDir: report.target_dir }),
    );
    if (report.backup_dir) {
      appendLog(translate("logs.backupCreated", { path: report.backup_dir }));
    }
    await refreshPreview();
    await loadBackups();
  } catch (error) {
    showActionResult("error", translate("status.error"), translate("result.installFailed"), false);
    appendLog(translate("logs.installFailed", { error }));
  } finally {
    setBusy(false);
  }
}

function confirmOverwriteRisks() {
  const riskyFiles = overwriteRiskFiles();
  if (riskyFiles.length === 0) {
    return Promise.resolve("replace");
  }

  const hasExternalSettings = riskyFiles.some((file) => file.has_external_settings);
  const actions = hasExternalSettings
    ? [
        { id: "merge", label: translate("actions.merge"), className: "primary-button" },
        {
          id: "replace",
          label: translate("actions.useAppSettingsOnly"),
          className: "danger-button",
        },
      ]
    : [
        {
          id: "replace",
          label: translate("actions.useAppSettingsOnly"),
          className: "danger-button",
        },
      ];

  return openConfirmModal({
    title: translate("modal.overwrite.title"),
    description: hasExternalSettings
      ? translate("modal.overwrite.descriptionMerge")
      : translate("modal.overwrite.descriptionManaged"),
    actions,
  });
}

function overwriteRiskFiles() {
  return state.preview.filter(
    (file) =>
      file.has_external_settings || ["modified", "untracked"].includes(file.modification_state),
  );
}

async function loadBackups() {
  if (!state.targetDir.trim()) {
    state.backups = [];
    renderBackups();
    return;
  }

  try {
    state.backups = await invokeCommand("list_backups", { targetDir: state.targetDir });
  } catch (error) {
    state.backups = [];
    appendLog(translate("logs.backupScanFailed", { error }));
  }

  renderBackups();
}

function renderBackups() {
  elements.backupList.innerHTML = "";

  if (state.backups.length === 0) {
    const empty = document.createElement("div");
    empty.className = "backup-row";
    empty.innerHTML = `<div><div class="backup-title">${escapeHtml(
      translate("backups.emptyTitle"),
    )}</div><div class="backup-meta">${escapeHtml(translate("backups.emptyMeta"))}</div></div>`;
    elements.backupList.appendChild(empty);
    return;
  }

  state.backups.forEach((backup) => {
    const row = document.createElement("div");
    row.className = "backup-row";
    row.innerHTML = `
      <div>
        <div class="backup-title">${escapeHtml(backup.id)}</div>
        <div class="backup-meta">${escapeHtml(backup.files.join(", "))} - ${escapeHtml(backup.path)}</div>
      </div>
      <button class="secondary-button compact" type="button">${escapeHtml(
        translate("actions.restore"),
      )}</button>
    `;
    row.querySelector("button").addEventListener("click", () => restoreBackup(backup.id));
    elements.backupList.appendChild(row);
  });
}

async function restoreBackup(backupId) {
  setBusy(true);
  try {
    const report = await invokeCommand("restore_backup", {
      targetDir: state.targetDir,
      backupId,
    });
    appendLog(
      translate("logs.restoreSuccess", {
        files: report.restored_files.join(", "),
        backupId: report.backup_id,
      }),
    );
    await refreshPreview();
    await loadBackups();
  } catch (error) {
    appendLog(translate("logs.restoreFailed", { error }));
  } finally {
    setBusy(false);
  }
}

async function resetToVanilla() {
  if (!state.targetDir.trim()) {
    appendLog(translate("logs.resetNoTarget"));
    return;
  }

  const confirmed = await openConfirmModal({
    title: translate("modal.reset.title"),
    description: translate("modal.reset.description"),
    actions: [
      { id: "reset", label: translate("actions.resetToVanilla"), className: "danger-button" },
    ],
  });
  if (!confirmed) {
    return;
  }

  setBusy(true);
  try {
    const report = await invokeCommand("reset_to_vanilla", {
      targetDir: state.targetDir,
    });
    if (report.removed_files.length === 0) {
      showActionResult("neutral", "", translate("logs.resetNoFiles"), true);
      appendLog(translate("logs.resetNoFiles"));
    } else {
      showActionResult(
        "neutral",
        "",
        translate("logs.resetRemoved", { files: report.removed_files.join(", ") }),
        true,
      );
      appendLog(translate("logs.resetRemoved", { files: report.removed_files.join(", ") }));
    }
    if (report.backup_dir) {
      appendLog(translate("logs.backupCreated", { path: report.backup_dir }));
    }
    await refreshPreview();
    await loadBackups();
  } catch (error) {
    showActionResult("error", translate("status.error"), translate("result.resetFailed"), false);
    appendLog(translate("logs.resetFailed", { error }));
  } finally {
    setBusy(false);
  }
}

async function launchGame() {
  const request = selectedLaunchRequest();
  if (!request) {
    showActionResult("error", translate("status.error"), translate("logs.launchNoTarget"), false);
    appendLog(translate("logs.launchNoTarget"));
    return;
  }

  setBusy(true);
  try {
    const report = await invokeCommand("launch_game", { request });
    showActionResult("success", translate("status.success"), translate("logs.launchSuccess"), true);
    appendLog(translate("logs.launchStarted", { target: report.path || report.kind }));
  } catch (error) {
    showActionResult("error", translate("status.error"), translate("result.launchFailed"), false);
    appendLog(translate("logs.launchFailed", { error }));
  } finally {
    setBusy(false);
  }
}

function openConfirmModal({ title, description, actions }) {
  if (confirmModalResolve) {
    return Promise.resolve(false);
  }

  confirmModalPreviousFocus = document.activeElement;
  elements.confirmModalTitle.textContent = title;
  elements.confirmModalDescription.textContent = description;
  renderConfirmModalActions(actions);
  elements.confirmModal.hidden = false;
  document.body.classList.add("modal-open");

  window.requestAnimationFrame(() => {
    elements.confirmModalCancel.focus();
  });

  return new Promise((resolve) => {
    confirmModalResolve = resolve;
  });
}

function renderConfirmModalActions(actions) {
  for (const button of confirmModalActionButtons) {
    button.remove();
  }
  confirmModalActionButtons = [];

  const [primaryAction, ...extraActions] = actions;
  elements.confirmModalConfirm.textContent = primaryAction.label;
  elements.confirmModalConfirm.className = primaryAction.className;
  elements.confirmModalConfirm.dataset.modalAction = primaryAction.id;

  for (const action of extraActions) {
    const button = document.createElement("button");
    button.className = action.className;
    button.type = "button";
    button.textContent = action.label;
    button.dataset.modalAction = action.id;
    button.addEventListener("click", () => closeConfirmModal(action.id));
    elements.confirmModalConfirm.before(button);
    confirmModalActionButtons.push(button);
  }
}

function closeConfirmModal(result) {
  if (!confirmModalResolve) {
    return;
  }

  const resolve = confirmModalResolve;
  confirmModalResolve = null;
  elements.confirmModal.hidden = true;
  document.body.classList.remove("modal-open");
  for (const button of confirmModalActionButtons) {
    button.remove();
  }
  confirmModalActionButtons = [];

  if (confirmModalPreviousFocus instanceof HTMLElement) {
    confirmModalPreviousFocus.focus();
  }
  confirmModalPreviousFocus = null;
  resolve(result);
}

function renderDiagnostics() {
  elements.presetRootValue.textContent = state.presetRoot || translate("value.unknown");
  elements.selectedPresetValue.textContent = selectedPresetLabel();
  elements.recommendedPresetValue.textContent =
    state.recommendation?.preset_id || translate("value.none");
  elements.selectedTargetValue.textContent = state.targetDir || translate("value.none");
}

function renderPerformanceState() {
  const enabled =
    elements.balancedPerformanceToggle.checked ||
    selectedVolumetricFogMode() !== "normal";
  elements.performanceStatus.textContent = enabled ? translate("status.on") : translate("status.off");
  elements.performanceStatus.className = enabled ? "pill warn" : "pill";
}

function renderPerformanceComparison() {
  elements.performanceComparisonGallery.innerHTML = "";
  performanceComparisonScenes.forEach((comparisonScene) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "comparison-thumb";
    button.innerHTML = `
      <span class="comparison-thumb-image-wrap">
        <img
          class="comparison-thumb-image"
          src="${escapeHtml(comparisonScene.thumbnail.src)}"
          alt="${escapeHtml(
            translate("comparison.previewAlt", { scene: comparisonSceneLabel(comparisonScene) }),
          )}"
          loading="lazy"
          decoding="async"
        />
        <span class="comparison-thumb-action">${escapeHtml(
          translate("actions.openComparison"),
        )}</span>
      </span>
      <span class="comparison-thumb-meta">
        <span class="comparison-thumb-title">${escapeHtml(comparisonSceneLabel(comparisonScene))}</span>
        <span class="comparison-thumb-subtitle">${escapeHtml(translate("comparison.subtitle"))}</span>
      </span>
    `;
    button.addEventListener("click", () => {
      const returnFocus = comparisonGalleryModalPreviousFocus;
      closeComparisonGalleryModal({ keepBodyOpen: true, restoreFocus: false });
      openPerformanceComparisonModal(comparisonScene.id, returnFocus);
    });
    elements.performanceComparisonGallery.appendChild(button);
  });
}

function renderPerformanceComparisonModal() {
  const scene = selectedPerformanceComparisonScene();
  elements.comparisonModalTitle.textContent = comparisonSceneLabel(scene);
  elements.comparisonModalDescription.textContent =
    translate("comparison.modalDescription");
  elements.performanceComparisonBeforeImage.src = scene.before.src;
  elements.performanceComparisonBeforeImage.alt = `${comparisonSceneLabel(scene)} ${translate(
    "comparison.beforeLabel",
  )}`;
  elements.performanceComparisonAfterImage.src = scene.after.src;
  elements.performanceComparisonAfterImage.alt = `${comparisonSceneLabel(scene)} ${translate(
    "comparison.afterLabel",
  )}`;
  elements.performanceComparisonBeforeLabel.textContent = translate("comparison.beforeLabel");
  elements.performanceComparisonAfterLabel.textContent = translate("comparison.afterLabel");
  elements.performanceComparisonRange.value = String(state.comparisonPosition);
  updatePerformanceComparisonPosition();
}

function openPerformanceComparisonGalleryModal() {
  comparisonGalleryModalPreviousFocus = document.activeElement;
  elements.comparisonGalleryModal.hidden = false;
  document.body.classList.add("modal-open");

  window.requestAnimationFrame(() => {
    elements.comparisonGalleryModalClose.focus();
  });
}

function closeComparisonGalleryModal(options = {}) {
  const { keepBodyOpen = false, restoreFocus = true } = options;
  elements.comparisonGalleryModal.hidden = true;
  if (!keepBodyOpen && elements.comparisonModal.hidden) {
    document.body.classList.remove("modal-open");
  }

  if (restoreFocus && comparisonGalleryModalPreviousFocus instanceof HTMLElement) {
    comparisonGalleryModalPreviousFocus.focus();
  }
  comparisonGalleryModalPreviousFocus = null;
}

function openPerformanceComparisonModal(sceneId, returnFocus = document.activeElement) {
  state.selectedComparisonSceneId = sceneId;
  comparisonModalPreviousFocus = returnFocus;
  renderPerformanceComparisonModal();
  elements.comparisonModal.hidden = false;
  document.body.classList.add("modal-open");

  window.requestAnimationFrame(() => {
    elements.comparisonModalClose.focus();
  });
}

function closeComparisonModal() {
  elements.comparisonModal.hidden = true;
  if (elements.comparisonGalleryModal.hidden) {
    document.body.classList.remove("modal-open");
  }

  if (comparisonModalPreviousFocus instanceof HTMLElement) {
    comparisonModalPreviousFocus.focus();
  }
  comparisonModalPreviousFocus = null;
}

function selectedPerformanceComparisonScene() {
  return (
    performanceComparisonScenes.find((scene) => scene.id === state.selectedComparisonSceneId) ??
    performanceComparisonScenes[0]
  );
}

function updatePerformanceComparisonPosition() {
  elements.performanceComparisonStage.style.setProperty(
    "--comparison-position",
    `${state.comparisonPosition}%`,
  );
}

function renderGameTweaksState() {
  const enabled = elements.skipIntroVideosToggle.checked;
  elements.gameTweaksStatus.textContent = enabled ? translate("status.on") : translate("status.off");
  elements.gameTweaksStatus.className = enabled ? "pill warn" : "pill";
}

function renderStreamingState() {
  const enabled = streamingFixesEnabled();
  elements.streamingStatus.textContent = enabled
    ? translate("status.streamingOn")
    : translate("status.streamingOff");
  elements.streamingStatus.className = enabled ? "pill good" : "pill";
  elements.presetPanel.classList.toggle("streaming-disabled", !enabled);
  renderCustomPoolState(false);
}

function renderPageChrome() {
  const previewVisible = viewsWithPreview.has(state.currentView);
  elements.pageTitle.textContent = translate(
    viewTitleKeys[state.currentView] ?? "views.optimizeStreaming",
  );
  elements.pageSubtitle.textContent = translate(
    viewSubtitleKeys[state.currentView] ?? "views.optimizeStreaming.subtitle",
  );
  elements.previewPanel.hidden = !previewVisible;
  elements.workspace.classList.toggle("preview-visible", previewVisible);
}

function switchView(view) {
  state.currentView = view;
  document.querySelectorAll(".nav-item").forEach((button) => {
    button.classList.toggle("active", button.dataset.view === view);
  });
  document.querySelectorAll(".view").forEach((section) => {
    section.classList.toggle("active", section.id === `${view}View`);
  });
  renderPageChrome();

  if (view === "backups") {
    loadBackups();
  }
}

function pickDefaultPreset(presets) {
  return presets.find((preset) => preset.vram_gb === 8) ?? presets[0];
}

function setBusy(busy) {
  state.busy = busy;
  elements.refreshButton.disabled = busy;
  elements.loadBackupsButton.disabled = busy;
  elements.browseTargetButton.disabled = busy;
  elements.browseExecutableButton.disabled = busy;
  elements.resetVanillaButton.disabled = busy || !hasTauriApi();
  updateActionButtons();
}

function updateActionButtons() {
  elements.optimizeButton.disabled =
    state.busy || state.preview.length === 0 || !hasTauriApi() || !customPoolSelectionValid();
  elements.copyIniButton.disabled =
    state.busy ||
    !hasTauriApi() ||
    !selectedPresetIdForCommand() ||
    !selectedOptimizerChangesEnabled() ||
    !customPoolSelectionValid();
  elements.playButton.disabled = state.busy || !hasTauriApi() || !selectedLaunchRequest();
}

function showActionResult(kind, statusText, detailText, autoHide) {
  clearTimeout(resultClearTimer);
  resultClearTimer = null;

  elements.optimizeStatus.hidden = !statusText;
  elements.optimizeStatus.textContent = statusText;
  elements.optimizeStatus.className = statusText ? `action-status ${kind}` : "action-status";

  elements.lastResult.hidden = !detailText;
  elements.lastResult.textContent = detailText;

  if (autoHide) {
    resultClearTimer = setTimeout(clearActionResult, 5000);
  }
}

function clearActionResult() {
  elements.optimizeStatus.hidden = true;
  elements.optimizeStatus.textContent = "";
  elements.optimizeStatus.className = "action-status";
  elements.lastResult.hidden = true;
  elements.lastResult.textContent = "";
  resultClearTimer = null;
}

function appendLog(message) {
  const line = document.createElement("div");
  line.className = "log-entry";
  line.textContent = `${new Date().toLocaleTimeString()} - ${message}`;
  elements.activityLog.prepend(line);
}

function formatPool(value) {
  return typeof value === "number" ? `${value} MB` : translate("value.notSet");
}

function formatModificationState(stateValue) {
  const className = modificationStateClass(stateValue);
  return `<span class="${className}">${escapeHtml(modificationStateLabel(stateValue))}</span>`;
}

function modificationStateLabel(stateValue) {
  switch (stateValue) {
    case "missing":
      return translate("tracking.new");
    case "unchanged":
      return translate("tracking.clean");
    case "untracked":
      return translate("tracking.untracked");
    case "modified":
      return translate("tracking.modified");
    default:
      return translate("tracking.unknown");
  }
}

function modificationStateClass(stateValue) {
  const tone = ["modified", "untracked"].includes(stateValue) ? "warn" : "neutral";
  return `file-state ${tone}`;
}

function yesNo(value) {
  return value ? translate("table.yes") : translate("table.no");
}

function poolLabel(poolMb) {
  return translate("preset.poolLabel", { poolMb });
}

function presetLabel(preset) {
  return `${preset.vram_gb} GB VRAM / ${poolLabel(preset.pool_mb)}`;
}

function recommendationMessage(recommendation) {
  return translate("recommendation.detected", {
    vramGb: Math.floor(recommendation.detected_vram_mb / 1024),
    gpuName: recommendation.gpu_name,
  });
}

function comparisonSceneLabel(scene) {
  return translate(`scene.${scene.id}`);
}

function formatTweaks(file) {
  const labels = [];
  if (file.will_apply_balanced_performance_tweaks) {
    labels.push(translate("tweaks.balanced"));
  }
  if (file.will_apply_disable_volumetric_fog) {
    labels.push(translate("tweaks.fogOff"));
  }
  if (file.will_apply_low_volumetric_fog) {
    labels.push(translate("tweaks.fogLow"));
  }
  if (file.will_apply_d3d12_pso_cache) {
    labels.push(translate("tweaks.d3d12"));
  }
  if (file.will_apply_runtime_pso_precaching) {
    labels.push(translate("tweaks.runtimePso"));
  }
  if (file.will_apply_gc_smoothing) {
    labels.push(translate("tweaks.gc"));
  }
  if (file.will_skip_intro_videos) {
    labels.push(translate("tweaks.skipIntro"));
  }
  return labels.length > 0 ? labels.join(", ") : translate("tweaks.base");
}

function emptyPreviewMessage() {
  if (!customPoolSelectionValid()) {
    return customPoolValidationMessage();
  }

  if (
    !streamingFixesEnabled() &&
    !elements.balancedPerformanceToggle.checked &&
    selectedVolumetricFogMode() === "normal" &&
    !experimentalEngineTweaksEnabled() &&
    !elements.skipIntroVideosToggle.checked
  ) {
    return translate("emptyPreview.noChanges");
  }

  return translate("emptyPreview.selectPreset");
}

function streamingFixesEnabled() {
  return elements.streamingFixesToggle.checked;
}

function selectedVolumetricFogMode() {
  if (elements.volumetricFogModeOff.checked) {
    return "off";
  }

  if (elements.volumetricFogModeLow.checked) {
    return "low";
  }

  return "normal";
}

function setVolumetricFogMode(mode) {
  elements.volumetricFogModeOff.checked = mode === "off";
  elements.volumetricFogModeLow.checked = mode === "low";
  elements.volumetricFogModeNormal.checked = mode !== "off" && mode !== "low";
}

function experimentalEngineTweaksEnabled() {
  return (
    elements.d3d12PsoCacheToggle.checked ||
    elements.runtimePsoPrecachingToggle.checked ||
    elements.gcSmoothingToggle.checked
  );
}

function selectedOptimizerChangesEnabled() {
  return (
    streamingFixesEnabled() ||
    elements.balancedPerformanceToggle.checked ||
    selectedVolumetricFogMode() !== "normal" ||
    experimentalEngineTweaksEnabled() ||
    elements.skipIntroVideosToggle.checked
  );
}

function isCustomPresetSelected() {
  return state.selectedPresetId === CUSTOM_PRESET_ID;
}

function selectedPresetIdForCommand() {
  if (!isCustomPresetSelected()) {
    return state.selectedPresetId;
  }

  return customPresetTemplate()?.id ?? "";
}

function customPresetTemplate() {
  if (state.presets.length === 0) {
    return null;
  }

  const poolMb = selectedCustomPoolMb() ?? state.customPoolMb;
  return state.presets.reduce((closest, preset) => {
    const currentDistance = Math.abs(preset.pool_mb - poolMb);
    const closestDistance = Math.abs(closest.pool_mb - poolMb);
    return currentDistance < closestDistance ? preset : closest;
  }, state.presets[0]);
}

function selectedCustomPoolMb() {
  if (!isCustomPresetSelected() || !streamingFixesEnabled()) {
    return null;
  }

  const value = Number(elements.customPoolInput.value);
  if (!Number.isInteger(value)) {
    return null;
  }

  if (value < MIN_CUSTOM_POOL_MB || value > MAX_CUSTOM_POOL_MB) {
    return null;
  }

  if (value % CUSTOM_POOL_STEP_MB !== 0) {
    return null;
  }

  return value;
}

function customPoolSelectionValid() {
  return !isCustomPresetSelected() || !streamingFixesEnabled() || selectedCustomPoolMb() !== null;
}

function customPoolValidationMessage() {
  if (!isCustomPresetSelected() || !streamingFixesEnabled()) {
    return "";
  }

  const inputValue = elements.customPoolInput.value.trim();
  if (!inputValue) {
    return translate("validation.poolEnter");
  }

  const value = Number(inputValue);
  if (!Number.isInteger(value)) {
    return translate("validation.poolWhole");
  }

  if (value < MIN_CUSTOM_POOL_MB || value > MAX_CUSTOM_POOL_MB) {
    return translate("validation.poolRange", {
      min: MIN_CUSTOM_POOL_MB,
      max: MAX_CUSTOM_POOL_MB,
    });
  }

  if (value % CUSTOM_POOL_STEP_MB !== 0) {
    return translate("validation.poolStep", { step: CUSTOM_POOL_STEP_MB });
  }

  return "";
}

function renderCustomPoolState(syncInputValue) {
  const customSelected = isCustomPresetSelected();
  elements.customPoolPanel.hidden = !customSelected;
  elements.customPoolInput.disabled = !streamingFixesEnabled();

  if (syncInputValue) {
    elements.customPoolInput.value = String(state.customPoolMb);
  }

  const validationMessage = customPoolValidationMessage();
  elements.customPoolHint.textContent =
    validationMessage ||
    translate("customPool.hint", {
      min: MIN_CUSTOM_POOL_MB,
      max: MAX_CUSTOM_POOL_MB,
      step: CUSTOM_POOL_STEP_MB,
    });
  elements.customPoolHint.className = validationMessage
    ? "custom-pool-hint bad"
    : "custom-pool-hint";
}

function customPoolLabel() {
  const poolMb = isCustomPresetSelected()
    ? (selectedCustomPoolMb() ?? state.customPoolMb)
    : state.customPoolMb;
  return poolLabel(poolMb);
}

function selectedPresetLabel() {
  if (isCustomPresetSelected()) {
    return translate("preset.customLabel", { poolLabel: customPoolLabel() });
  }

  const preset = state.presets.find((item) => item.id === state.selectedPresetId);
  return preset ? presetLabel(preset) : state.selectedPresetId || translate("value.none");
}

function availableLaunchTargets() {
  const targets = [...state.launchCandidates];
  const manualPath = state.manualExecutablePath.trim();
  if (manualPath) {
    targets.push({
      id: manualLaunchTargetId(manualPath),
      kind: "executable",
      label: translate("launch.manualLabel"),
      path: manualPath,
      exists: true,
      source: "manual",
    });
  }

  const seen = new Set();
  return targets.filter((target) => {
    if (seen.has(target.id)) {
      return false;
    }

    seen.add(target.id);
    return true;
  });
}

function preferredLaunchTarget() {
  const targets = availableLaunchTargets();
  return (
    targets.find((target) => target.kind === "steam" && target.exists) ??
    targets.find((target) => target.kind === "executable" && target.exists) ??
    targets[0] ??
    null
  );
}

function selectedLaunchTarget() {
  return (
    availableLaunchTargets().find((target) => target.id === state.selectedLaunchTargetId) ??
    null
  );
}

function selectedLaunchRequest() {
  const target = selectedLaunchTarget();
  if (!target || !target.path || (target.exists === false && target.source !== "manual")) {
    return null;
  }

  return {
    kind: target.kind,
    path: target.path,
  };
}

function manualLaunchTargetId(path) {
  return `manual:${path.trim()}`;
}

function launchTargetLabel(target) {
  const status = target.source === "manual" ? translate("pathStatus.manual") : translate("pathStatus.found");
  return `${target.label} - ${status}`;
}

function selectedIniContentArgs() {
  return {
    presetId: selectedPresetIdForCommand(),
    customPoolMb: selectedCustomPoolMb(),
    streamingFixes: streamingFixesEnabled(),
    balancedPerformance: elements.balancedPerformanceToggle.checked,
    disableVolumetricFog: selectedVolumetricFogMode() === "off",
    lowVolumetricFog: selectedVolumetricFogMode() === "low",
    skipIntroVideos: elements.skipIntroVideosToggle.checked,
    d3d12PsoCache: elements.d3d12PsoCacheToggle.checked,
    runtimePsoPrecaching: elements.runtimePsoPrecachingToggle.checked,
    gcSmoothing: elements.gcSmoothingToggle.checked,
  };
}

function recommendedBadgeMarkup() {
  return `
    <span class="recommended-badge" title="${escapeHtml(
      recommendationMessage(state.recommendation),
    )}">
      <span class="recommended-mark" aria-hidden="true"></span>
      <span>${escapeHtml(translate("recommended.badge"))}</span>
    </span>
  `;
}

async function openTargetFolderDialog() {
  const options = {
    title: translate("settings.browseTargetFolder"),
    directory: true,
    multiple: false,
    defaultPath: state.targetDir || undefined,
  };

  if (window.__TAURI__?.dialog?.open) {
    return window.__TAURI__.dialog.open(options);
  }

  return invokeCommand("plugin:dialog|open", { options });
}

async function openExecutableDialog() {
  const options = {
    title: translate("settings.browseExecutable"),
    directory: false,
    multiple: false,
    defaultPath: state.manualExecutablePath || undefined,
    filters: [{ name: translate("settings.executableFilter"), extensions: ["exe"] }],
  };

  if (window.__TAURI__?.dialog?.open) {
    return window.__TAURI__.dialog.open(options);
  }

  return invokeCommand("plugin:dialog|open", { options });
}

function hasTauriApi() {
  return Boolean(window.__TAURI__?.core?.invoke);
}

async function invokeCommand(command, args = {}) {
  if (hasTauriApi()) {
    return window.__TAURI__.core.invoke(command, args);
  }

  return demoInvoke(command, args);
}

async function demoInvoke(command, args) {
  await new Promise((resolve) => window.setTimeout(resolve, 80));

  if (command === "get_app_state") {
    return {
      preset_root: "../../Presets",
      presets: samplePresets,
      hardware: {
        gpus: [
          {
            name: "Radeon RX 7900 XT",
            vendor: "AMD",
            dedicated_vram_mb: 20480,
            shared_memory_mb: null,
            source: "Static preview",
            confidence: "high",
          },
        ],
        system_ram_mb: 32768,
        cpu_name: "AMD Ryzen 7 9800X3D",
        logical_cores: 16,
        os_runtime: "Static preview",
      },
      recommendation: {
        preset_id: "20GB_VRAM_10240MB",
        gpu_name: "Radeon RX 7900 XT",
        detected_vram_mb: 20480,
        confidence: "high",
        reason: "Detected 20 GB VRAM on Radeon RX 7900 XT. This preset is recommended.",
      },
      candidates: [
        {
          label: "Linux Steam Proton",
          path: "/home/user/.steam/steam/steamapps/compatdata/1297900/pfx/drive_c/users/steamuser/AppData/Local/G1R/Saved/Config/Windows",
          exists: false,
          source: "Static preview",
        },
      ],
      launch_candidates: [
        {
          id: "steam:/home/user/.steam/steam/steamapps/appmanifest_1297900.acf",
          kind: "steam",
          label: "Steam: Gothic 1 Remake",
          path: "/home/user/.steam/steam/steamapps/appmanifest_1297900.acf",
          exists: true,
          source: "Static preview",
        },
      ],
    };
  }

  if (command === "preview_install") {
    const preset = samplePresets.find((item) => item.id === args.presetId) ?? samplePresets[2];
    const previewPoolMb = args.customPoolMb ?? preset.pool_mb;
    const files = [];
    if (
      args.streamingFixes ||
      args.disableVolumetricFog ||
      args.lowVolumetricFog ||
      args.d3d12PsoCache ||
      args.runtimePsoPrecaching ||
      args.gcSmoothing
    ) {
      files.push({
        file_name: "Engine.ini",
        target_exists: false,
        modification_state: "missing",
        has_external_settings: false,
        current_pool_mb: null,
        preset_pool_mb: args.streamingFixes ? previewPoolMb : null,
        will_backup: false,
        will_set_read_only: args.lockEngine,
        will_apply_balanced_performance_tweaks: false,
        will_apply_disable_volumetric_fog: Boolean(args.disableVolumetricFog),
        will_apply_low_volumetric_fog: Boolean(args.lowVolumetricFog) && !args.disableVolumetricFog,
        will_apply_d3d12_pso_cache: Boolean(args.d3d12PsoCache),
        will_apply_runtime_pso_precaching: Boolean(args.runtimePsoPrecaching),
        will_apply_gc_smoothing: Boolean(args.gcSmoothing),
        will_skip_intro_videos: false,
      });
    }

    if (args.streamingFixes || args.balancedPerformance) {
      files.push({
        file_name: "Scalability.ini",
        target_exists: false,
        modification_state: "missing",
        has_external_settings: false,
        current_pool_mb: null,
        preset_pool_mb: args.streamingFixes ? previewPoolMb : null,
        will_backup: false,
        will_set_read_only: args.lockScalability,
        will_apply_balanced_performance_tweaks: args.balancedPerformance,
        will_apply_disable_volumetric_fog: false,
        will_apply_low_volumetric_fog: false,
        will_apply_d3d12_pso_cache: false,
        will_apply_runtime_pso_precaching: false,
        will_apply_gc_smoothing: false,
        will_skip_intro_videos: false,
      });
    }

    if (args.skipIntroVideos) {
      files.push({
        file_name: "Game.ini",
        target_exists: false,
        modification_state: "missing",
        has_external_settings: false,
        current_pool_mb: null,
        preset_pool_mb: null,
        will_backup: false,
        will_set_read_only: args.lockGame,
        will_apply_balanced_performance_tweaks: false,
        will_apply_disable_volumetric_fog: false,
        will_apply_low_volumetric_fog: false,
        will_apply_d3d12_pso_cache: false,
        will_apply_runtime_pso_precaching: false,
        will_apply_gc_smoothing: false,
        will_skip_intro_videos: true,
      });
    }

    return files;
  }

  if (command === "list_backups") {
    return [];
  }

  if (command === "reset_to_vanilla") {
    return {
      target_dir: args.targetDir,
      backup_dir: null,
      removed_files: [],
    };
  }

  if (command === "launch_game") {
    return {
      kind: args.request.kind,
      path: args.request.path,
    };
  }

  throw new Error("This command requires the Tauri desktop runtime.");
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function debounce(callback, delay) {
  let timer = 0;
  return (...args) => {
    window.clearTimeout(timer);
    timer = window.setTimeout(() => callback(...args), delay);
  };
}
