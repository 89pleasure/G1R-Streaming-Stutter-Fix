import { loadUiPreferences, saveUiPreferences } from "./preferences.js";
import { performanceComparisonScenes } from "./performance-comparisons.js";

const state = {
  presets: [],
  candidates: [],
  presetRoot: "",
  selectedPresetId: "",
  recommendation: null,
  hardware: null,
  targetDir: "",
  preview: [],
  backups: [],
  currentView: "optimizeStreaming",
  selectedComparisonSceneId: performanceComparisonScenes[0]?.id ?? "",
  comparisonPosition: 50,
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
  { id: "24GB_VRAM_12288MB", label: "24 GB VRAM / 12288 MB pool", vram_gb: 24, pool_mb: 12288 },
];

const viewTitles = {
  optimizeStreaming: "Optimize Streaming",
  performance: "Performance Tweaks",
  gameTweaks: "Game Tweaks",
  backups: "Backups",
  diagnostics: "Diagnostics",
  settings: "Settings",
};

const viewsWithPreview = new Set(["optimizeStreaming", "performance", "gameTweaks"]);

const elements = {};
let confirmModalResolve = null;
let confirmModalPreviousFocus = null;
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
    "appStatus",
    "refreshButton",
    "presetPanel",
    "streamingFixesToggle",
    "streamingStatus",
    "presetCount",
    "presetGrid",
    "recommendationSummary",
    "candidateSelect",
    "targetInput",
    "pathStatus",
    "lockEngineToggle",
    "lockGameToggle",
    "lockScalabilityToggle",
    "balancedPerformanceToggle",
    "performanceStatus",
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
    "optimizeButton",
    "optimizeStatus",
    "lastResult",
    "loadBackupsButton",
    "resetVanillaButton",
    "backupList",
    "presetRootValue",
    "selectedPresetValue",
    "recommendedPresetValue",
    "selectedTargetValue",
    "activityLog",
    "confirmModal",
    "confirmModalCancel",
    "confirmModalConfirm",
  ]) {
    elements[id] = document.getElementById(id);
  }
}

function applyStoredPreferences() {
  const preferences = loadUiPreferences();
  state.selectedPresetId = preferences.selectedPresetId;
  state.targetDir = preferences.targetDir;
  elements.streamingFixesToggle.checked = preferences.streamingFixes;
  elements.balancedPerformanceToggle.checked = preferences.balancedPerformance;
  elements.skipIntroVideosToggle.checked = preferences.skipIntroVideos;
  elements.lockEngineToggle.checked = preferences.lockEngine;
  elements.lockGameToggle.checked = preferences.lockGame;
  elements.lockScalabilityToggle.checked = preferences.lockScalability;
}

function persistUiPreferences() {
  saveUiPreferences({
    balancedPerformance: elements.balancedPerformanceToggle.checked,
    skipIntroVideos: elements.skipIntroVideosToggle.checked,
    streamingFixes: elements.streamingFixesToggle.checked,
    lockEngine: elements.lockEngineToggle.checked,
    lockGame: elements.lockGameToggle.checked,
    lockScalability: elements.lockScalabilityToggle.checked,
    selectedPresetId: state.selectedPresetId,
    targetDir: state.targetDir,
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
  elements.optimizeButton.addEventListener("click", optimizeSelectedPreset);
  elements.confirmModalCancel.addEventListener("click", () => closeConfirmModal(false));
  elements.confirmModalConfirm.addEventListener("click", () => closeConfirmModal(true));
  elements.confirmModal.addEventListener("click", (event) => {
    if (event.target === elements.confirmModal) {
      closeConfirmModal(false);
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
    } else if (!elements.confirmModal.hidden) {
      closeConfirmModal(false);
    }
  });

  elements.candidateSelect.addEventListener("change", () => {
    const candidate = state.candidates.find(
      (item) => item.path === elements.candidateSelect.value,
    );
    state.targetDir = candidate?.path ?? elements.candidateSelect.value;
    elements.targetInput.value = state.targetDir;
    persistUiPreferences();
    renderDiagnostics();
    refreshPreview();
    loadBackups();
  });

  elements.targetInput.addEventListener("input", () => {
    state.targetDir = elements.targetInput.value;
    persistUiPreferences();
    renderPathStatus();
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

async function loadAppState() {
  setBusy(true);
  try {
    const appState = await invokeCommand("get_app_state");
    state.presets = appState.presets;
    state.candidates = appState.candidates;
    state.presetRoot = appState.preset_root;
    state.hardware = appState.hardware;
    state.recommendation = appState.recommendation ?? null;
    if (!state.presets.some((preset) => preset.id === state.selectedPresetId)) {
      state.selectedPresetId = pickDefaultPreset(state.presets)?.id || "";
    }

    const bestCandidate = state.candidates.find((candidate) => candidate.exists) ?? state.candidates[0];
    if (!state.targetDir && bestCandidate) {
      state.targetDir = bestCandidate.path;
    }

    elements.appStatus.textContent = "Preset data loaded";
    elements.runtimeStatus.textContent = hasTauriApi() ? "Tauri desktop" : "Static preview";
    renderAll();
    await refreshPreview();
    await loadBackups();
  } catch (error) {
    elements.appStatus.textContent = "Failed to load app state";
    appendLog(`Load failed: ${error}`);
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
  renderPerformanceState();
  renderPerformanceComparison();
  renderGameTweaksState();
  renderPageChrome();
  renderPreview();
  renderBackups();
  renderDiagnostics();
}

function renderPresets() {
  elements.presetCount.textContent = `${state.presets.length} presets`;
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
      <span class="preset-pool">${preset.pool_mb} MB pool</span>
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
}

function renderRecommendationSummary() {
  if (!streamingFixesEnabled()) {
    elements.recommendationSummary.textContent =
      "Streaming fixes are off. The selected preset is kept, but it will not be installed.";
    elements.recommendationSummary.className = "recommendation-summary muted";
    return;
  }

  const recommendation = state.recommendation;
  if (!recommendation) {
    elements.recommendationSummary.textContent =
      "Hardware could not be detected reliably. Choose a preset manually.";
    elements.recommendationSummary.className = "recommendation-summary muted";
    return;
  }

  elements.recommendationSummary.textContent = recommendation.reason;
  elements.recommendationSummary.className = "recommendation-summary";
}

function renderCandidates() {
  elements.candidateSelect.innerHTML = "";

  if (state.candidates.length === 0) {
    const option = document.createElement("option");
    option.value = "";
    option.textContent = "No detected locations";
    elements.candidateSelect.appendChild(option);
  } else {
    state.candidates.forEach((candidate) => {
      const option = document.createElement("option");
      option.value = candidate.path;
      option.textContent = `${candidate.label} - ${candidate.exists ? "found" : "not found"}`;
      option.selected = candidate.path === state.targetDir;
      elements.candidateSelect.appendChild(option);
    });
  }

  elements.targetInput.value = state.targetDir;
}

function renderPathStatus() {
  const candidate = state.candidates.find((item) => item.path === state.targetDir);
  if (!state.targetDir.trim()) {
    elements.pathStatus.textContent = "Missing";
    elements.pathStatus.className = "pill bad";
  } else if (candidate?.exists) {
    elements.pathStatus.textContent = "Found";
    elements.pathStatus.className = "pill good";
  } else if (candidate) {
    elements.pathStatus.textContent = "Can create";
    elements.pathStatus.className = "pill warn";
  } else {
    elements.pathStatus.textContent = "Manual";
    elements.pathStatus.className = "pill";
  }
}

async function refreshPreview() {
  if (!state.selectedPresetId || !state.targetDir.trim()) {
    state.preview = [];
    elements.previewStatus.textContent = "Waiting";
    elements.previewStatus.className = "pill";
    renderPreview();
    return;
  }

  try {
    state.preview = await invokeCommand("preview_install", {
      presetId: state.selectedPresetId,
      targetDir: state.targetDir,
      lockEngine: elements.lockEngineToggle.checked,
      lockGame: elements.lockGameToggle.checked,
      lockScalability: elements.lockScalabilityToggle.checked,
      streamingFixes: streamingFixesEnabled(),
      balancedPerformance: elements.balancedPerformanceToggle.checked,
      skipIntroVideos: elements.skipIntroVideosToggle.checked,
    });
    elements.previewStatus.textContent = "Ready";
    elements.previewStatus.className = "pill good";
  } catch (error) {
    state.preview = [];
    elements.previewStatus.textContent = "Error";
    elements.previewStatus.className = "pill bad";
    appendLog(`Preview failed: ${error}`);
  }

  renderPreview();
}

function renderPreview() {
  elements.previewRows.innerHTML = "";

  if (state.preview.length === 0) {
    const row = document.createElement("tr");
    row.className = "empty-row";
    row.innerHTML = `<td colspan="6">${emptyPreviewMessage()}</td>`;
    elements.previewRows.appendChild(row);
    elements.optimizeButton.disabled = true;
    return;
  }

  state.preview.forEach((file) => {
    const row = document.createElement("tr");
    row.innerHTML = `
      <td>${escapeHtml(file.file_name)}</td>
      <td>${formatPool(file.current_pool_mb)}</td>
      <td>${formatPool(file.preset_pool_mb)}</td>
      <td>${formatTweaks(file)}</td>
      <td>${file.will_backup ? "Yes" : "No"}</td>
      <td>${file.will_set_read_only ? "Yes" : "No"}</td>
    `;
    elements.previewRows.appendChild(row);
  });

  elements.optimizeButton.disabled = state.busy || !hasTauriApi();
}

async function optimizeSelectedPreset() {
  if (!state.selectedPresetId || !state.targetDir.trim()) {
    return;
  }

  setBusy(true);
  try {
    const report = await invokeCommand("install_preset", {
      presetId: state.selectedPresetId,
      targetDir: state.targetDir,
      lockEngine: elements.lockEngineToggle.checked,
      lockGame: elements.lockGameToggle.checked,
      lockScalability: elements.lockScalabilityToggle.checked,
      streamingFixes: streamingFixesEnabled(),
      balancedPerformance: elements.balancedPerformanceToggle.checked,
      skipIntroVideos: elements.skipIntroVideosToggle.checked,
    });
    const fileNames = report.installed_files.map((file) => file.file_name).join(", ");
    showActionResult("success", "Success", `Installed ${fileNames}`, true);
    appendLog(`Installed ${report.preset_id} to ${report.target_dir}`);
    if (report.backup_dir) {
      appendLog(`Backup created at ${report.backup_dir}`);
    }
    await refreshPreview();
    await loadBackups();
  } catch (error) {
    showActionResult("error", "Error", "Install failed", false);
    appendLog(`Install failed: ${error}`);
  } finally {
    setBusy(false);
  }
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
    appendLog(`Backup scan failed: ${error}`);
  }

  renderBackups();
}

function renderBackups() {
  elements.backupList.innerHTML = "";

  if (state.backups.length === 0) {
    const empty = document.createElement("div");
    empty.className = "backup-row";
    empty.innerHTML = `<div><div class="backup-title">No backups</div><div class="backup-meta">Backups appear after an existing config file is replaced.</div></div>`;
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
      <button class="secondary-button compact" type="button">Restore</button>
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
    appendLog(`Restored ${report.restored_files.join(", ")} from ${report.backup_id}`);
    await refreshPreview();
    await loadBackups();
  } catch (error) {
    appendLog(`Restore failed: ${error}`);
  } finally {
    setBusy(false);
  }
}

async function resetToVanilla() {
  if (!state.targetDir.trim()) {
    appendLog("Reset failed: no target folder selected");
    return;
  }

  const confirmed = await openConfirmModal();
  if (!confirmed) {
    return;
  }

  setBusy(true);
  try {
    const report = await invokeCommand("reset_to_vanilla", {
      targetDir: state.targetDir,
    });
    if (report.removed_files.length === 0) {
      showActionResult("neutral", "", "No managed config files found", true);
      appendLog("Reset to Vanilla found no managed config files");
    } else {
      showActionResult("neutral", "", `Removed ${report.removed_files.join(", ")}`, true);
      appendLog(`Reset to Vanilla removed ${report.removed_files.join(", ")}`);
    }
    if (report.backup_dir) {
      appendLog(`Backup created at ${report.backup_dir}`);
    }
    await refreshPreview();
    await loadBackups();
  } catch (error) {
    showActionResult("error", "Error", "Reset failed", false);
    appendLog(`Reset failed: ${error}`);
  } finally {
    setBusy(false);
  }
}

function openConfirmModal() {
  if (confirmModalResolve) {
    return Promise.resolve(false);
  }

  confirmModalPreviousFocus = document.activeElement;
  elements.confirmModal.hidden = false;
  document.body.classList.add("modal-open");

  window.requestAnimationFrame(() => {
    elements.confirmModalCancel.focus();
  });

  return new Promise((resolve) => {
    confirmModalResolve = resolve;
  });
}

function closeConfirmModal(confirmed) {
  if (!confirmModalResolve) {
    return;
  }

  const resolve = confirmModalResolve;
  confirmModalResolve = null;
  elements.confirmModal.hidden = true;
  document.body.classList.remove("modal-open");

  if (confirmModalPreviousFocus instanceof HTMLElement) {
    confirmModalPreviousFocus.focus();
  }
  confirmModalPreviousFocus = null;
  resolve(confirmed);
}

function renderDiagnostics() {
  elements.presetRootValue.textContent = state.presetRoot || "Unknown";
  elements.selectedPresetValue.textContent = state.selectedPresetId || "None";
  elements.recommendedPresetValue.textContent = state.recommendation?.preset_id || "None";
  elements.selectedTargetValue.textContent = state.targetDir || "None";
}

function renderPerformanceState() {
  const enabled = elements.balancedPerformanceToggle.checked;
  elements.performanceStatus.textContent = enabled ? "On" : "Off";
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
          alt="${escapeHtml(comparisonScene.thumbnail.alt)}"
          loading="lazy"
          decoding="async"
        />
        <span class="comparison-thumb-action">Open comparison</span>
      </span>
      <span class="comparison-thumb-meta">
        <span class="comparison-thumb-title">${escapeHtml(comparisonScene.label)}</span>
        <span class="comparison-thumb-subtitle">Cine vs Balanced (Cine)</span>
      </span>
    `;
    button.addEventListener("click", () => openPerformanceComparisonModal(comparisonScene.id));
    elements.performanceComparisonGallery.appendChild(button);
  });
}

function renderPerformanceComparisonModal() {
  const scene = selectedPerformanceComparisonScene();
  elements.comparisonModalTitle.textContent = scene.label;
  elements.comparisonModalDescription.textContent =
    "Cine versus Balanced (Cine), with the FPS overlay left visible.";
  elements.performanceComparisonBeforeImage.src = scene.before.src;
  elements.performanceComparisonBeforeImage.alt = `${scene.label} ${scene.before.label}`;
  elements.performanceComparisonAfterImage.src = scene.after.src;
  elements.performanceComparisonAfterImage.alt = `${scene.label} ${scene.after.label}`;
  elements.performanceComparisonBeforeLabel.textContent = scene.before.label;
  elements.performanceComparisonAfterLabel.textContent = scene.after.label;
  elements.performanceComparisonRange.value = String(state.comparisonPosition);
  updatePerformanceComparisonPosition();
}

function openPerformanceComparisonModal(sceneId) {
  state.selectedComparisonSceneId = sceneId;
  comparisonModalPreviousFocus = document.activeElement;
  renderPerformanceComparisonModal();
  elements.comparisonModal.hidden = false;
  document.body.classList.add("modal-open");

  window.requestAnimationFrame(() => {
    elements.comparisonModalClose.focus();
  });
}

function closeComparisonModal() {
  elements.comparisonModal.hidden = true;
  document.body.classList.remove("modal-open");

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
  elements.gameTweaksStatus.textContent = enabled ? "On" : "Off";
  elements.gameTweaksStatus.className = enabled ? "pill warn" : "pill";
}

function renderStreamingState() {
  const enabled = streamingFixesEnabled();
  elements.streamingStatus.textContent = enabled ? "Streaming On" : "Streaming Off";
  elements.streamingStatus.className = enabled ? "pill good" : "pill";
  elements.presetPanel.classList.toggle("streaming-disabled", !enabled);
}

function renderPageChrome() {
  const previewVisible = viewsWithPreview.has(state.currentView);
  elements.pageTitle.textContent = viewTitles[state.currentView] ?? "Optimize Streaming";
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
  elements.optimizeButton.disabled = busy || state.preview.length === 0 || !hasTauriApi();
  elements.loadBackupsButton.disabled = busy;
  elements.resetVanillaButton.disabled = busy || !hasTauriApi();
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
  return typeof value === "number" ? `${value} MB` : "Not set";
}

function formatTweaks(file) {
  const labels = [];
  if (file.will_apply_balanced_performance_tweaks) {
    labels.push("Balanced (Cine)");
  }
  if (file.will_skip_intro_videos) {
    labels.push("Skip intro");
  }
  return labels.length > 0 ? labels.join(", ") : "Base";
}

function emptyPreviewMessage() {
  if (
    !streamingFixesEnabled() &&
    !elements.balancedPerformanceToggle.checked &&
    !elements.skipIntroVideosToggle.checked
  ) {
    return "No optimizer changes selected.";
  }

  return "Select a preset and target folder.";
}

function streamingFixesEnabled() {
  return elements.streamingFixesToggle.checked;
}

function recommendedBadgeMarkup() {
  return `
    <span class="recommended-badge" title="${escapeHtml(state.recommendation.reason)}">
      <span class="recommended-mark" aria-hidden="true"></span>
      <span>Recommended</span>
    </span>
  `;
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
    };
  }

  if (command === "preview_install") {
    const preset = samplePresets.find((item) => item.id === args.presetId) ?? samplePresets[2];
    const files = [];
    if (args.streamingFixes) {
      files.push({
        file_name: "Engine.ini",
        target_exists: false,
        current_pool_mb: null,
        preset_pool_mb: preset.pool_mb,
        will_backup: false,
        will_set_read_only: args.lockEngine,
        will_apply_balanced_performance_tweaks: false,
        will_skip_intro_videos: false,
      });
    }

    if (args.streamingFixes || args.balancedPerformance) {
      files.push({
        file_name: "Scalability.ini",
        target_exists: false,
        current_pool_mb: null,
        preset_pool_mb: args.streamingFixes ? preset.pool_mb : null,
        will_backup: false,
        will_set_read_only: args.lockScalability,
        will_apply_balanced_performance_tweaks: args.balancedPerformance,
        will_skip_intro_videos: false,
      });
    }

    if (args.skipIntroVideos) {
      files.push({
        file_name: "Game.ini",
        target_exists: false,
        current_pool_mb: null,
        preset_pool_mb: null,
        will_backup: false,
        will_set_read_only: args.lockGame,
        will_apply_balanced_performance_tweaks: false,
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
