use crate::presets::find_preset;
use crate::{CoreError, CoreResult};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

mod content;
mod ini_merge;
mod manifest;

use content::{managed_files, planned_files, PlannedFile};
use ini_merge::{has_external_ini_settings, has_managed_ini_settings, merge_ini_content};
pub use manifest::FileModificationState;
use manifest::{
    classify_modification_state, current_unix_seconds, load_manifest, remove_manifest_files,
    update_manifest_files, InstallManifest, ManifestFile,
};

const MANAGED_DIR: &str = ".g1r-streaming-stutter-fix";
const BACKUPS_DIR: &str = "backups";
const MANAGED_FILES: [&str; 3] = ["Engine.ini", "Game.ini", "Scalability.ini"];
const MIN_CUSTOM_POOL_MB: u32 = 512;
const MAX_CUSTOM_POOL_MB: u32 = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallOptions {
    pub lock_engine_ini: bool,
    pub lock_game_ini: bool,
    pub lock_scalability_ini: bool,
    pub custom_pool_mb: Option<u32>,
    pub apply_streaming_fixes: bool,
    pub apply_balanced_performance_tweaks: bool,
    pub apply_disable_volumetric_fog: bool,
    pub apply_low_volumetric_fog: bool,
    pub apply_skip_intro_videos: bool,
    pub apply_d3d12_pso_cache: bool,
    pub apply_runtime_pso_precaching: bool,
    pub apply_gc_smoothing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStrategy {
    Replace,
    Merge,
}

impl InstallOptions {
    fn applies_low_volumetric_fog(self) -> bool {
        self.apply_low_volumetric_fog && !self.apply_disable_volumetric_fog
    }

    fn validate(self) -> CoreResult<()> {
        if let Some(pool_mb) = self.custom_pool_mb {
            if !(MIN_CUSTOM_POOL_MB..=MAX_CUSTOM_POOL_MB).contains(&pool_mb) {
                return Err(CoreError::new(format!(
                    "custom pool size must be between {MIN_CUSTOM_POOL_MB} and {MAX_CUSTOM_POOL_MB} MB"
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePreview {
    pub file_name: String,
    pub target_exists: bool,
    pub modification_state: FileModificationState,
    pub has_external_settings: bool,
    pub current_pool_mb: Option<u32>,
    pub preset_pool_mb: Option<u32>,
    pub will_backup: bool,
    pub will_set_read_only: bool,
    pub will_apply_balanced_performance_tweaks: bool,
    pub will_apply_disable_volumetric_fog: bool,
    pub will_apply_low_volumetric_fog: bool,
    pub will_apply_d3d12_pso_cache: bool,
    pub will_apply_runtime_pso_precaching: bool,
    pub will_apply_gc_smoothing: bool,
    pub will_skip_intro_videos: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IniFileContent {
    pub file_name: String,
    pub content: String,
}

impl FilePreview {
    pub fn has_overwrite_risk(&self) -> bool {
        self.modification_state.has_overwrite_risk()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInstallReport {
    pub file_name: String,
    pub bytes_written: u64,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallReport {
    pub preset_id: String,
    pub target_dir: PathBuf,
    pub backup_dir: Option<PathBuf>,
    pub created_target_dir: bool,
    pub installed_files: Vec<FileInstallReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupInfo {
    pub id: String,
    pub path: PathBuf,
    pub files: Vec<String>,
    pub modified_unix_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreReport {
    pub backup_id: String,
    pub target_dir: PathBuf,
    pub restored_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetReport {
    pub target_dir: PathBuf,
    pub backup_dir: Option<PathBuf>,
    pub removed_files: Vec<String>,
}

pub fn preview_install(
    presets_root: &Path,
    preset_id: &str,
    target_dir: &Path,
    options: InstallOptions,
) -> CoreResult<Vec<FilePreview>> {
    options.validate()?;
    let preset = find_preset(presets_root, preset_id)?;
    let manifest = load_manifest(target_dir);
    planned_files_for_target(&preset, options, target_dir)?
        .into_iter()
        .map(|file| {
            preview_file_content(
                &file.content,
                &file.managed_content,
                &target_dir.join(file.file_name),
                file.file_name,
                file.read_only,
                file.applies_balanced_performance_tweaks,
                file.applies_disable_volumetric_fog,
                file.applies_low_volumetric_fog,
                file.applies_d3d12_pso_cache,
                file.applies_runtime_pso_precaching,
                file.applies_gc_smoothing,
                file.skips_intro_videos,
                manifest.as_ref(),
            )
        })
        .collect()
}

pub fn ini_file_contents(
    presets_root: &Path,
    preset_id: &str,
    options: InstallOptions,
) -> CoreResult<Vec<IniFileContent>> {
    options.validate()?;
    let preset = find_preset(presets_root, preset_id)?;
    planned_files(&preset, options).map(|files| {
        files
            .into_iter()
            .map(|file| IniFileContent {
                file_name: file.file_name.to_string(),
                content: file.content,
            })
            .collect()
    })
}

pub fn install_preset(
    presets_root: &Path,
    preset_id: &str,
    target_dir: &Path,
    options: InstallOptions,
) -> CoreResult<InstallReport> {
    install_preset_with_strategy(
        presets_root,
        preset_id,
        target_dir,
        options,
        InstallStrategy::Replace,
    )
}

pub fn install_preset_with_strategy(
    presets_root: &Path,
    preset_id: &str,
    target_dir: &Path,
    options: InstallOptions,
    strategy: InstallStrategy,
) -> CoreResult<InstallReport> {
    options.validate()?;
    let preset = find_preset(presets_root, preset_id)?;
    let planned_files = planned_files_for_target(&preset, options, target_dir)?;
    let created_target_dir = !target_dir.exists();
    fs::create_dir_all(target_dir)
        .map_err(|source| CoreError::io("create target directory", target_dir, source))?;

    let file_names = planned_files
        .iter()
        .map(|file| file.file_name)
        .collect::<Vec<_>>();
    let installed_unix_seconds = current_unix_seconds();
    let backup_dir = backup_existing_files(target_dir, &file_names)?;
    let installed_files = planned_files
        .into_iter()
        .map(|file| {
            let target_file = target_dir.join(file.file_name);
            let content = install_content_for_strategy(
                &target_file,
                &file.content,
                &file.managed_content,
                strategy,
            )?;
            let manifest_file =
                ManifestFile::from_content(file.file_name, &content, installed_unix_seconds);
            let report =
                install_file_content(&content, &target_file, file.file_name, file.read_only)?;
            Ok((report, manifest_file))
        })
        .collect::<CoreResult<Vec<_>>>()?;
    let (installed_files, manifest_files): (Vec<_>, Vec<_>) = installed_files.into_iter().unzip();
    update_manifest_files(target_dir, manifest_files)?;

    Ok(InstallReport {
        preset_id: preset.id,
        target_dir: target_dir.to_path_buf(),
        backup_dir,
        created_target_dir,
        installed_files,
    })
}

fn planned_files_for_target(
    preset: &crate::presets::Preset,
    options: InstallOptions,
    target_dir: &Path,
) -> CoreResult<Vec<PlannedFile>> {
    let mut files = planned_files(preset, options)?;
    let managed_files = managed_files(preset, options)?;

    for file in managed_files {
        if files
            .iter()
            .any(|active_file| active_file.file_name == file.file_name)
        {
            continue;
        }

        let target_file = target_dir.join(file.file_name);
        if target_has_managed_settings(&target_file, &file.managed_content)? {
            files.push(file);
        }
    }

    Ok(files)
}

fn install_content_for_strategy(
    target: &Path,
    planned_content: &str,
    managed_content: &str,
    strategy: InstallStrategy,
) -> CoreResult<String> {
    match strategy {
        InstallStrategy::Replace => Ok(planned_content.to_string()),
        InstallStrategy::Merge => merged_file_content(target, planned_content, managed_content),
    }
}

fn target_has_managed_settings(target: &Path, managed_content: &str) -> CoreResult<bool> {
    let content = match fs::read_to_string(target) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(CoreError::io("read existing config file", target, error)),
    };

    Ok(has_managed_ini_settings(&content, managed_content))
}

fn merged_file_content(
    target: &Path,
    planned_content: &str,
    managed_content: &str,
) -> CoreResult<String> {
    let current_content = match fs::read_to_string(target) {
        Ok(content) => Some(content),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(CoreError::io("read existing config file", target, error)),
    };

    Ok(merge_ini_content(
        current_content.as_deref(),
        planned_content,
        managed_content,
    ))
}

pub fn list_backups(target_dir: &Path) -> CoreResult<Vec<BackupInfo>> {
    let backups_root = backups_root(target_dir);
    if !backups_root.is_dir() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    let entries = fs::read_dir(&backups_root)
        .map_err(|source| CoreError::io("read backups directory", &backups_root, source))?;

    for entry in entries {
        let entry = entry.map_err(|source| {
            CoreError::io("read backup directory entry", &backups_root, source)
        })?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(id) = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_owned)
        else {
            continue;
        };

        let files = MANAGED_FILES
            .iter()
            .filter(|file_name| path.join(file_name).is_file())
            .map(|file_name| (*file_name).to_string())
            .collect::<Vec<_>>();

        if files.is_empty() {
            continue;
        }

        let modified_unix_seconds = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(system_time_to_unix_seconds)
            .unwrap_or_default();

        backups.push(BackupInfo {
            id,
            path,
            files,
            modified_unix_seconds,
        });
    }

    backups.sort_by(|left, right| {
        right
            .modified_unix_seconds
            .cmp(&left.modified_unix_seconds)
            .then_with(|| right.id.cmp(&left.id))
    });
    Ok(backups)
}

pub fn restore_backup(target_dir: &Path, backup_id: &str) -> CoreResult<RestoreReport> {
    validate_backup_id(backup_id)?;
    let backup_dir = backups_root(target_dir).join(backup_id);
    if !backup_dir.is_dir() {
        return Err(CoreError::new(format!(
            "backup '{backup_id}' does not exist"
        )));
    }

    fs::create_dir_all(target_dir)
        .map_err(|source| CoreError::io("create target directory", target_dir, source))?;

    let mut restored_files = Vec::new();
    for file_name in MANAGED_FILES {
        let source = backup_dir.join(file_name);
        if !source.is_file() {
            continue;
        }

        let target = target_dir.join(file_name);
        make_writable_if_exists(&target)?;
        fs::copy(&source, &target)
            .map_err(|error| CoreError::io("restore backup file", &target, error))?;
        set_read_only(&target, false)?;
        restored_files.push(file_name.to_string());
    }

    if restored_files.is_empty() {
        return Err(CoreError::new(format!(
            "backup '{backup_id}' contains no managed files"
        )));
    }
    remove_manifest_files(target_dir, &restored_files)?;

    Ok(RestoreReport {
        backup_id: backup_id.to_string(),
        target_dir: target_dir.to_path_buf(),
        restored_files,
    })
}

pub fn reset_to_vanilla(target_dir: &Path) -> CoreResult<ResetReport> {
    let backup_dir = backup_existing_files(target_dir, &MANAGED_FILES)?;
    let mut removed_files = Vec::new();

    for file_name in MANAGED_FILES {
        let target = target_dir.join(file_name);
        if !target.is_file() {
            continue;
        }

        make_writable_if_exists(&target)?;
        match fs::remove_file(&target) {
            Ok(()) => removed_files.push(file_name.to_string()),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(CoreError::io("remove managed config file", &target, error)),
        }
    }
    remove_manifest_files(target_dir, &removed_files)?;

    Ok(ResetReport {
        target_dir: target_dir.to_path_buf(),
        backup_dir,
        removed_files,
    })
}

fn preview_file_content(
    preset_content: &str,
    managed_content: &str,
    target_file: &Path,
    file_name: &str,
    will_set_read_only: bool,
    will_apply_balanced_performance_tweaks: bool,
    will_apply_disable_volumetric_fog: bool,
    will_apply_low_volumetric_fog: bool,
    will_apply_d3d12_pso_cache: bool,
    will_apply_runtime_pso_precaching: bool,
    will_apply_gc_smoothing: bool,
    will_skip_intro_videos: bool,
    manifest: Option<&InstallManifest>,
) -> CoreResult<FilePreview> {
    let target_exists = target_file.is_file();
    let current_bytes = if target_exists {
        fs::read(target_file).ok()
    } else {
        None
    };
    let current_pool_mb = current_bytes
        .as_deref()
        .and_then(|content| str::from_utf8(content).ok())
        .and_then(extract_pool_size);
    let has_external_settings = current_bytes
        .as_deref()
        .and_then(|content| str::from_utf8(content).ok())
        .is_some_and(|content| has_external_ini_settings(content, managed_content));
    let modification_state =
        classify_modification_state(file_name, target_exists, current_bytes.as_deref(), manifest);

    Ok(FilePreview {
        file_name: file_name.to_string(),
        target_exists,
        modification_state,
        has_external_settings,
        current_pool_mb,
        preset_pool_mb: extract_pool_size(&preset_content),
        will_backup: target_exists,
        will_set_read_only,
        will_apply_balanced_performance_tweaks,
        will_apply_disable_volumetric_fog,
        will_apply_low_volumetric_fog,
        will_apply_d3d12_pso_cache,
        will_apply_runtime_pso_precaching,
        will_apply_gc_smoothing,
        will_skip_intro_videos,
    })
}

fn install_file_content(
    content: &str,
    target: &Path,
    file_name: &str,
    read_only: bool,
) -> CoreResult<FileInstallReport> {
    make_writable_if_exists(target)?;
    fs::write(target, content)
        .map_err(|error| CoreError::io("write preset file", target, error))?;
    set_read_only(target, read_only)?;

    Ok(FileInstallReport {
        file_name: file_name.to_string(),
        bytes_written: content.len() as u64,
        read_only,
    })
}

fn backup_existing_files(target_dir: &Path, file_names: &[&str]) -> CoreResult<Option<PathBuf>> {
    let existing_files = file_names
        .iter()
        .map(|file_name| target_dir.join(file_name))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();

    if existing_files.is_empty() {
        return Ok(None);
    }

    let backup_dir = create_backup_dir(target_dir)?;

    for source in existing_files {
        let file_name = source
            .file_name()
            .ok_or_else(|| CoreError::new("managed file path has no file name"))?;
        let target = backup_dir.join(file_name);
        fs::copy(&source, &target)
            .map_err(|error| CoreError::io("back up existing file", &target, error))?;
    }

    Ok(Some(backup_dir))
}

fn backups_root(target_dir: &Path) -> PathBuf {
    target_dir.join(MANAGED_DIR).join(BACKUPS_DIR)
}

fn create_backup_dir(target_dir: &Path) -> CoreResult<PathBuf> {
    create_backup_dir_with_timestamp(target_dir, current_unix_millis())
}

fn create_backup_dir_with_timestamp(
    target_dir: &Path,
    timestamp_millis: u128,
) -> CoreResult<PathBuf> {
    let backups_root = backups_root(target_dir);
    fs::create_dir_all(&backups_root)
        .map_err(|source| CoreError::io("create backups directory", &backups_root, source))?;

    for attempt in 0..1000 {
        let backup_dir = backups_root.join(backup_dir_name(timestamp_millis, attempt));
        match fs::create_dir(&backup_dir) {
            Ok(()) => return Ok(backup_dir),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(CoreError::io("create backup directory", &backup_dir, error));
            }
        }
    }

    Err(CoreError::new(
        "could not allocate a unique backup directory",
    ))
}

fn backup_dir_name(timestamp_millis: u128, attempt: u16) -> String {
    if attempt == 0 {
        format!("backup-{timestamp_millis}")
    } else {
        format!("backup-{timestamp_millis}-{attempt}")
    }
}

fn extract_pool_size(content: &str) -> Option<u32> {
    content.lines().find_map(|line| {
        let line = line.trim();
        if line.starts_with(';') || line.starts_with('#') {
            return None;
        }

        let (key, value) = line.split_once('=')?;
        if key.trim() != "r.Streaming.PoolSize" {
            return None;
        }

        value.trim().parse().ok()
    })
}

fn make_writable_if_exists(path: &Path) -> CoreResult<()> {
    if !path.exists() {
        return Ok(());
    }

    set_read_only(path, false)
}

fn set_read_only(path: &Path, read_only: bool) -> CoreResult<()> {
    let metadata =
        fs::metadata(path).map_err(|source| CoreError::io("read file metadata", path, source))?;
    let mut permissions = metadata.permissions();
    permissions.set_readonly(read_only);
    fs::set_permissions(path, permissions)
        .map_err(|source| CoreError::io("set file permissions", path, source))
}

fn validate_backup_id(backup_id: &str) -> CoreResult<()> {
    let valid = !backup_id.is_empty()
        && !backup_id.contains('/')
        && !backup_id.contains('\\')
        && backup_id != "."
        && backup_id != "..";

    if valid {
        Ok(())
    } else {
        Err(CoreError::new("invalid backup id"))
    }
}

fn current_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn system_time_to_unix_seconds(time: SystemTime) -> Option<u64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}

#[cfg(test)]
mod tests;
