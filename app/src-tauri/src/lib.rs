use optimizer_core::{
    detect_config_paths, detect_hardware, ini_file_contents as core_ini_file_contents,
    install_preset_with_strategy as core_install_preset, list_backups as core_list_backups,
    list_presets, preview_install as core_preview_install, recommend_preset_for_hardware,
    reset_to_vanilla as core_reset_to_vanilla, restore_backup as core_restore_backup, BackupInfo,
    ConfigCandidate, FileInstallReport, FileModificationState, FilePreview, GpuInfo, GpuVendor,
    HardwareConfidence, HardwareSnapshot, IniFileContent, InstallOptions, InstallReport,
    InstallStrategy, Preset, PresetRecommendation, ResetReport, RestoreReport,
};
use serde::Serialize;
use std::env;
use std::path::{Path, PathBuf};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize)]
struct AppStateDto {
    preset_root: String,
    presets: Vec<PresetDto>,
    candidates: Vec<ConfigCandidateDto>,
    hardware: HardwareSnapshotDto,
    recommendation: Option<PresetRecommendationDto>,
}

#[derive(Debug, Serialize)]
struct PresetDto {
    id: String,
    label: String,
    vram_gb: u32,
    pool_mb: u32,
}

#[derive(Debug, Serialize)]
struct ConfigCandidateDto {
    label: String,
    path: String,
    exists: bool,
    source: String,
}

#[derive(Debug, Serialize)]
struct HardwareSnapshotDto {
    gpus: Vec<GpuInfoDto>,
    system_ram_mb: Option<u64>,
    cpu_name: Option<String>,
    logical_cores: Option<usize>,
    os_runtime: String,
}

#[derive(Debug, Serialize)]
struct GpuInfoDto {
    name: String,
    vendor: String,
    dedicated_vram_mb: Option<u32>,
    shared_memory_mb: Option<u32>,
    source: String,
    confidence: String,
}

#[derive(Debug, Serialize)]
struct PresetRecommendationDto {
    preset_id: String,
    gpu_name: String,
    detected_vram_mb: u32,
    confidence: String,
    reason: String,
}

#[derive(Debug, Serialize)]
struct FilePreviewDto {
    file_name: String,
    target_exists: bool,
    modification_state: String,
    has_external_settings: bool,
    current_pool_mb: Option<u32>,
    preset_pool_mb: Option<u32>,
    will_backup: bool,
    will_set_read_only: bool,
    will_apply_balanced_performance_tweaks: bool,
    will_apply_disable_volumetric_fog: bool,
    will_apply_low_volumetric_fog: bool,
    will_apply_d3d12_pso_cache: bool,
    will_apply_runtime_pso_precaching: bool,
    will_apply_gc_smoothing: bool,
    will_skip_intro_videos: bool,
}

#[derive(Debug, Serialize)]
struct IniFileContentDto {
    file_name: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct FileInstallReportDto {
    file_name: String,
    bytes_written: u64,
    read_only: bool,
}

#[derive(Debug, Serialize)]
struct InstallReportDto {
    preset_id: String,
    target_dir: String,
    backup_dir: Option<String>,
    created_target_dir: bool,
    installed_files: Vec<FileInstallReportDto>,
}

#[derive(Debug, Serialize)]
struct BackupInfoDto {
    id: String,
    path: String,
    files: Vec<String>,
    modified_unix_seconds: u64,
}

#[derive(Debug, Serialize)]
struct RestoreReportDto {
    backup_id: String,
    target_dir: String,
    restored_files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ResetReportDto {
    target_dir: String,
    backup_dir: Option<String>,
    removed_files: Vec<String>,
}

#[tauri::command]
fn get_app_state(app: AppHandle) -> Result<AppStateDto, String> {
    let preset_root = resolve_preset_root(&app)?;
    let presets = list_presets(&preset_root).map_err(to_error_string)?;
    let hardware = detect_hardware();
    let recommendation = recommend_preset_for_hardware(&presets, &hardware);
    let preset_dtos = presets.into_iter().map(PresetDto::from).collect();
    let candidates = detect_config_paths()
        .into_iter()
        .map(ConfigCandidateDto::from)
        .collect();

    Ok(AppStateDto {
        preset_root: path_to_string(&preset_root),
        presets: preset_dtos,
        candidates,
        hardware: HardwareSnapshotDto::from(hardware),
        recommendation: recommendation.map(PresetRecommendationDto::from),
    })
}

#[tauri::command]
fn preview_install(
    app: AppHandle,
    preset_id: String,
    target_dir: String,
    lock_engine: bool,
    lock_game: bool,
    lock_scalability: bool,
    custom_pool_mb: Option<u32>,
    streaming_fixes: bool,
    balanced_performance: bool,
    disable_volumetric_fog: bool,
    low_volumetric_fog: bool,
    skip_intro_videos: bool,
    d3d12_pso_cache: bool,
    runtime_pso_precaching: bool,
    gc_smoothing: bool,
) -> Result<Vec<FilePreviewDto>, String> {
    let preset_root = resolve_preset_root(&app)?;
    let target_dir = parse_target_dir(&target_dir)?;
    let previews = core_preview_install(
        &preset_root,
        &preset_id,
        &target_dir,
        InstallOptions {
            lock_engine_ini: lock_engine,
            lock_game_ini: lock_game,
            lock_scalability_ini: lock_scalability,
            custom_pool_mb,
            apply_streaming_fixes: streaming_fixes,
            apply_balanced_performance_tweaks: balanced_performance,
            apply_disable_volumetric_fog: disable_volumetric_fog,
            apply_low_volumetric_fog: low_volumetric_fog,
            apply_skip_intro_videos: skip_intro_videos,
            apply_d3d12_pso_cache: d3d12_pso_cache,
            apply_runtime_pso_precaching: runtime_pso_precaching,
            apply_gc_smoothing: gc_smoothing,
        },
    )
    .map_err(to_error_string)?;

    Ok(previews.into_iter().map(FilePreviewDto::from).collect())
}

#[tauri::command]
fn ini_file_contents(
    app: AppHandle,
    preset_id: String,
    custom_pool_mb: Option<u32>,
    streaming_fixes: bool,
    balanced_performance: bool,
    disable_volumetric_fog: bool,
    low_volumetric_fog: bool,
    skip_intro_videos: bool,
    d3d12_pso_cache: bool,
    runtime_pso_precaching: bool,
    gc_smoothing: bool,
) -> Result<Vec<IniFileContentDto>, String> {
    let preset_root = resolve_preset_root(&app)?;
    let files = core_ini_file_contents(
        &preset_root,
        &preset_id,
        InstallOptions {
            lock_engine_ini: false,
            lock_game_ini: false,
            lock_scalability_ini: false,
            custom_pool_mb,
            apply_streaming_fixes: streaming_fixes,
            apply_balanced_performance_tweaks: balanced_performance,
            apply_disable_volumetric_fog: disable_volumetric_fog,
            apply_low_volumetric_fog: low_volumetric_fog,
            apply_skip_intro_videos: skip_intro_videos,
            apply_d3d12_pso_cache: d3d12_pso_cache,
            apply_runtime_pso_precaching: runtime_pso_precaching,
            apply_gc_smoothing: gc_smoothing,
        },
    )
    .map_err(to_error_string)?;

    Ok(files.into_iter().map(IniFileContentDto::from).collect())
}

#[tauri::command]
fn install_preset(
    app: AppHandle,
    preset_id: String,
    target_dir: String,
    lock_engine: bool,
    lock_game: bool,
    lock_scalability: bool,
    custom_pool_mb: Option<u32>,
    streaming_fixes: bool,
    balanced_performance: bool,
    disable_volumetric_fog: bool,
    low_volumetric_fog: bool,
    skip_intro_videos: bool,
    d3d12_pso_cache: bool,
    runtime_pso_precaching: bool,
    gc_smoothing: bool,
    install_strategy: String,
) -> Result<InstallReportDto, String> {
    let preset_root = resolve_preset_root(&app)?;
    let target_dir = parse_target_dir(&target_dir)?;
    let install_strategy = parse_install_strategy(&install_strategy)?;
    let report = core_install_preset(
        &preset_root,
        &preset_id,
        &target_dir,
        InstallOptions {
            lock_engine_ini: lock_engine,
            lock_game_ini: lock_game,
            lock_scalability_ini: lock_scalability,
            custom_pool_mb,
            apply_streaming_fixes: streaming_fixes,
            apply_balanced_performance_tweaks: balanced_performance,
            apply_disable_volumetric_fog: disable_volumetric_fog,
            apply_low_volumetric_fog: low_volumetric_fog,
            apply_skip_intro_videos: skip_intro_videos,
            apply_d3d12_pso_cache: d3d12_pso_cache,
            apply_runtime_pso_precaching: runtime_pso_precaching,
            apply_gc_smoothing: gc_smoothing,
        },
        install_strategy,
    )
    .map_err(to_error_string)?;

    Ok(InstallReportDto::from(report))
}

#[tauri::command]
fn list_backups(target_dir: String) -> Result<Vec<BackupInfoDto>, String> {
    let target_dir = parse_target_dir(&target_dir)?;
    let backups = core_list_backups(&target_dir).map_err(to_error_string)?;
    Ok(backups.into_iter().map(BackupInfoDto::from).collect())
}

#[tauri::command]
fn restore_backup(target_dir: String, backup_id: String) -> Result<RestoreReportDto, String> {
    let target_dir = parse_target_dir(&target_dir)?;
    let report = core_restore_backup(&target_dir, &backup_id).map_err(to_error_string)?;
    Ok(RestoreReportDto::from(report))
}

#[tauri::command]
fn reset_to_vanilla(target_dir: String) -> Result<ResetReportDto, String> {
    let target_dir = parse_target_dir(&target_dir)?;
    let report = core_reset_to_vanilla(&target_dir).map_err(to_error_string)?;
    Ok(ResetReportDto::from(report))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_app_state,
            preview_install,
            ini_file_contents,
            install_preset,
            list_backups,
            restore_backup,
            reset_to_vanilla
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}

fn resolve_preset_root(app: &AppHandle) -> Result<PathBuf, String> {
    for candidate in preset_root_candidates(app) {
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }

    Err("could not find bundled or repository Presets directory".to_string())
}

fn preset_root_candidates(app: &AppHandle) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(path) = app.path().resolve("Presets", BaseDirectory::Resource) {
        candidates.push(path);
    }

    if let Ok(current_dir) = env::current_dir() {
        candidates.push(current_dir.join("Presets"));
        candidates.push(current_dir.join("..").join("Presets"));
        candidates.push(current_dir.join("..").join("..").join("Presets"));
    }

    candidates
}

fn parse_target_dir(target_dir: &str) -> Result<PathBuf, String> {
    let target_dir = target_dir.trim();
    if target_dir.is_empty() {
        return Err("target config folder is empty".to_string());
    }

    Ok(PathBuf::from(target_dir))
}

fn to_error_string(error: optimizer_core::CoreError) -> String {
    error.to_string()
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

impl From<Preset> for PresetDto {
    fn from(preset: Preset) -> Self {
        let label = preset.label();

        Self {
            id: preset.id,
            label,
            vram_gb: preset.vram_gb,
            pool_mb: preset.pool_mb,
        }
    }
}

impl From<ConfigCandidate> for ConfigCandidateDto {
    fn from(candidate: ConfigCandidate) -> Self {
        Self {
            label: candidate.label,
            path: path_to_string(&candidate.path),
            exists: candidate.exists,
            source: candidate.source,
        }
    }
}

impl From<HardwareSnapshot> for HardwareSnapshotDto {
    fn from(snapshot: HardwareSnapshot) -> Self {
        Self {
            gpus: snapshot.gpus.into_iter().map(GpuInfoDto::from).collect(),
            system_ram_mb: snapshot.system_ram_mb,
            cpu_name: snapshot.cpu_name,
            logical_cores: snapshot.logical_cores,
            os_runtime: snapshot.os_runtime,
        }
    }
}

impl From<GpuInfo> for GpuInfoDto {
    fn from(gpu: GpuInfo) -> Self {
        Self {
            name: gpu.name,
            vendor: vendor_to_string(gpu.vendor),
            dedicated_vram_mb: gpu.dedicated_vram_mb,
            shared_memory_mb: gpu.shared_memory_mb,
            source: gpu.source,
            confidence: confidence_to_string(gpu.confidence),
        }
    }
}

impl From<PresetRecommendation> for PresetRecommendationDto {
    fn from(recommendation: PresetRecommendation) -> Self {
        Self {
            preset_id: recommendation.preset_id,
            gpu_name: recommendation.gpu_name,
            detected_vram_mb: recommendation.detected_vram_mb,
            confidence: confidence_to_string(recommendation.confidence),
            reason: recommendation.reason,
        }
    }
}

impl From<FilePreview> for FilePreviewDto {
    fn from(preview: FilePreview) -> Self {
        Self {
            file_name: preview.file_name,
            target_exists: preview.target_exists,
            modification_state: modification_state_to_string(preview.modification_state),
            has_external_settings: preview.has_external_settings,
            current_pool_mb: preview.current_pool_mb,
            preset_pool_mb: preview.preset_pool_mb,
            will_backup: preview.will_backup,
            will_set_read_only: preview.will_set_read_only,
            will_apply_balanced_performance_tweaks: preview.will_apply_balanced_performance_tweaks,
            will_apply_disable_volumetric_fog: preview.will_apply_disable_volumetric_fog,
            will_apply_low_volumetric_fog: preview.will_apply_low_volumetric_fog,
            will_apply_d3d12_pso_cache: preview.will_apply_d3d12_pso_cache,
            will_apply_runtime_pso_precaching: preview.will_apply_runtime_pso_precaching,
            will_apply_gc_smoothing: preview.will_apply_gc_smoothing,
            will_skip_intro_videos: preview.will_skip_intro_videos,
        }
    }
}

impl From<IniFileContent> for IniFileContentDto {
    fn from(file: IniFileContent) -> Self {
        Self {
            file_name: file.file_name,
            content: file.content,
        }
    }
}

fn vendor_to_string(vendor: GpuVendor) -> String {
    match vendor {
        GpuVendor::Amd => "AMD",
        GpuVendor::Intel => "Intel",
        GpuVendor::Nvidia => "NVIDIA",
        GpuVendor::Microsoft => "Microsoft",
        GpuVendor::Unknown => "Unknown",
    }
    .to_string()
}

fn confidence_to_string(confidence: HardwareConfidence) -> String {
    match confidence {
        HardwareConfidence::High => "high",
        HardwareConfidence::Medium => "medium",
        HardwareConfidence::Low => "low",
    }
    .to_string()
}

fn modification_state_to_string(state: FileModificationState) -> String {
    match state {
        FileModificationState::Missing => "missing",
        FileModificationState::Unchanged => "unchanged",
        FileModificationState::Untracked => "untracked",
        FileModificationState::Modified => "modified",
    }
    .to_string()
}

fn parse_install_strategy(value: &str) -> Result<InstallStrategy, String> {
    match value {
        "replace" => Ok(InstallStrategy::Replace),
        "merge" => Ok(InstallStrategy::Merge),
        _ => Err(format!("invalid install strategy '{value}'")),
    }
}

impl From<FileInstallReport> for FileInstallReportDto {
    fn from(report: FileInstallReport) -> Self {
        Self {
            file_name: report.file_name,
            bytes_written: report.bytes_written,
            read_only: report.read_only,
        }
    }
}

impl From<InstallReport> for InstallReportDto {
    fn from(report: InstallReport) -> Self {
        Self {
            preset_id: report.preset_id,
            target_dir: path_to_string(&report.target_dir),
            backup_dir: report.backup_dir.as_deref().map(path_to_string),
            created_target_dir: report.created_target_dir,
            installed_files: report
                .installed_files
                .into_iter()
                .map(FileInstallReportDto::from)
                .collect(),
        }
    }
}

impl From<BackupInfo> for BackupInfoDto {
    fn from(backup: BackupInfo) -> Self {
        Self {
            id: backup.id,
            path: path_to_string(&backup.path),
            files: backup.files,
            modified_unix_seconds: backup.modified_unix_seconds,
        }
    }
}

impl From<RestoreReport> for RestoreReportDto {
    fn from(report: RestoreReport) -> Self {
        Self {
            backup_id: report.backup_id,
            target_dir: path_to_string(&report.target_dir),
            restored_files: report.restored_files,
        }
    }
}

impl From<ResetReport> for ResetReportDto {
    fn from(report: ResetReport) -> Self {
        Self {
            target_dir: path_to_string(&report.target_dir),
            backup_dir: report.backup_dir.as_deref().map(path_to_string),
            removed_files: report.removed_files,
        }
    }
}
