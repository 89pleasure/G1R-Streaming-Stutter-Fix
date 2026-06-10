use crate::presets::{find_preset, Preset};
use crate::{CoreError, CoreResult};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MANAGED_DIR: &str = ".g1r-streaming-stutter-fix";
const BACKUPS_DIR: &str = "backups";
const MANAGED_FILES: [&str; 3] = ["Engine.ini", "Game.ini", "Scalability.ini"];
const SKIP_INTRO_VIDEOS_GAME_SETTINGS: &str = r#"
; Skip Intro Videos (opt-in)
[/Script/AsyncLoadingScreen.LoadingScreenSettings]
StartupLoadingScreen=(MinimumLoadingScreenDisplayTime=0.000000,bAutoCompleteWhenLoadingCompletes=True,bMoviesAreSkippable=True,bWaitForManualStop=False,bAllowInEarlyStartup=False,bAllowEngineTick=False,PlaybackType=MT_LoadingLoop,MoviePaths=("LoopingEngineLoadScreen"),bShuffle=False,bSetDisplayMovieIndexManually=False,bShowWidgetOverlay=False,bShowLoadingCompleteText=False,bShowLoadingWidget=True,bUseRenderedFrameAsBackground=False,OverrideSyncIntervalForMovies=-1)
"#;
const D3D12_PSO_CACHE_ENGINE_SETTINGS: &str = r"
; D3D12 PSO Disk Cache (experimental opt-in)
[/Script/D3D12RHI.D3D12Options]
D3D12.PSO.DiskCache=1
D3D12.PSO.DriverOptimizedDiskCache=1
";
const RUNTIME_PSO_PRECACHING_ENGINE_SETTINGS: &str = r"
; Runtime PSO Precaching (experimental opt-in)
[SystemSettings]
r.PSOPrecaching=1
r.AsyncPipelineCompile=1
";
const GC_SMOOTHING_ENGINE_SETTINGS: &str = r"
; GC Smoothing (experimental opt-in)
[SystemSettings]
gc.TimeBetweenPurgingPendingKillObjects=60.0
gc.NumRetriesBeforeForcingGC=5
gc.MinDesiredObjectsPerSubTask=20
gc.AllowParallelGC=1
gc.MultithreadedDestructionEnabled=1
";
const DISABLE_VOLUMETRIC_FOG_ENGINE_SETTINGS: &str = r"
; Disable Volumetric Fog (visual impact opt-in)
[SystemSettings]
r.VolumetricFog=0
";
const LOW_VOLUMETRIC_FOG_ENGINE_SETTINGS: &str = r"
; Low Volumetric Fog (visual impact opt-in)
[SystemSettings]
r.VolumetricFog=1
r.VolumetricFog.GridPixelSize=16
r.VolumetricFog.GridSizeZ=64
r.VolumetricFog.HistoryMissSupersampleCount=4
";
const BALANCED_PERFORMANCE_SCALABILITY_SETTINGS: &str = r"
; Balanced Performance Tweaks (opt-in)
[ShadowQuality@Cine]
r.Shadow.MaxResolution=2048
r.Shadow.MaxCSMResolution=2048
r.Shadow.RadiusThreshold=0.01
r.Shadow.Virtual.MaxPhysicalPages=4096
r.Shadow.Virtual.ResolutionLodBiasDirectional=0
r.Shadow.Virtual.ResolutionLodBiasDirectionalMoving=0
r.Shadow.Virtual.ResolutionLodBiasLocal=1
r.Shadow.Virtual.ResolutionLodBiasLocalMoving=1.0
r.Shadow.Virtual.SMRT.RayCountDirectional=4
r.Shadow.Virtual.SMRT.SamplesPerRayDirectional=4
r.Shadow.Virtual.SMRT.RayCountLocal=8
r.Shadow.Virtual.SMRT.SamplesPerRayLocal=4
r.Shadow.Virtual.Clipmap.WPODisableDistance.LodBias=-1
Gothic.Sky.Light.UpdatePeriod.MinFramesBetween=5

[GlobalIlluminationQuality@Cine]
r.Lumen.TraceMeshSDFs.Allow=1
r.Lumen.ScreenProbeGather.RadianceCache.NumProbesToTraceBudget=300
r.Lumen.ScreenProbeGather.DownsampleFactor=16
r.Lumen.ScreenProbeGather.FullResolutionJitterWidth=1
r.Lumen.TranslucencyVolume.TracingOctahedronResolution=3
r.Lumen.TranslucencyVolume.RadianceCache.ProbeResolution=8
r.Lumen.TranslucencyVolume.RadianceCache.NumProbesToTraceBudget=200

[ReflectionQuality@Cine]
r.SSR.Quality=3
r.Lumen.TranslucencyReflections.FrontLayer.Enable=0

[PostProcessQuality@Cine]
r.DepthOfFieldQuality=2
r.RenderTargetPoolMin=400
r.LensFlareQuality=2
r.Bloom.ScreenPercentage=50.000
r.DOF.Gather.EnableBokehSettings=0
r.DOF.Gather.RingCount=4
r.DOF.Scatter.MaxSpriteRatio=0.1
r.DOF.Recombine.Quality=1
r.DOF.Recombine.EnableBokehSettings=0

[EffectsQuality@Cine]
r.VolumetricFog.GridPixelSize=8
r.VolumetricFog.HistoryMissSupersampleCount=4
r.SSGI.Quality=3
fx.Niagara.QualityLevel=3
r.SkyAtmosphere.AerialPerspectiveLUT.FastApplyOnOpaque=1
r.SkyAtmosphere.AerialPerspectiveLUT.SampleCountMaxPerSlice=4
r.SkyAtmosphere.AerialPerspectiveLUT.DepthResolution=16.0
r.SkyAtmosphere.FastSkyLUT=1
r.SkyAtmosphere.FastSkyLUT.SampleCountMax=128.0
r.SkyAtmosphere.SampleCountMin=4.0
r.SkyAtmosphere.SampleCountMax=128.0
r.SkyAtmosphere.TransmittanceLUT.SampleCount=10.0
r.SkyAtmosphere.MultiScatteringLUT.SampleCount=15.0

[LandscapeQuality@Cine]
r.Nanite.MaxPixelsPerEdge=2
";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallOptions {
    pub lock_engine_ini: bool,
    pub lock_game_ini: bool,
    pub lock_scalability_ini: bool,
    pub apply_streaming_fixes: bool,
    pub apply_balanced_performance_tweaks: bool,
    pub apply_disable_volumetric_fog: bool,
    pub apply_low_volumetric_fog: bool,
    pub apply_skip_intro_videos: bool,
    pub apply_d3d12_pso_cache: bool,
    pub apply_runtime_pso_precaching: bool,
    pub apply_gc_smoothing: bool,
}

impl InstallOptions {
    fn applies_engine_tweaks(self) -> bool {
        self.apply_disable_volumetric_fog
            || self.applies_low_volumetric_fog()
            || self.apply_d3d12_pso_cache
            || self.apply_runtime_pso_precaching
            || self.apply_gc_smoothing
    }

    fn applies_low_volumetric_fog(self) -> bool {
        self.apply_low_volumetric_fog && !self.apply_disable_volumetric_fog
    }
}

struct PlannedFile {
    file_name: &'static str,
    content: String,
    read_only: bool,
    applies_balanced_performance_tweaks: bool,
    applies_disable_volumetric_fog: bool,
    applies_low_volumetric_fog: bool,
    applies_d3d12_pso_cache: bool,
    applies_runtime_pso_precaching: bool,
    applies_gc_smoothing: bool,
    skips_intro_videos: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePreview {
    pub file_name: String,
    pub target_exists: bool,
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
    let preset = find_preset(presets_root, preset_id)?;
    planned_files(&preset, options)?
        .into_iter()
        .map(|file| {
            preview_file_content(
                &file.content,
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
            )
        })
        .collect()
}

pub fn install_preset(
    presets_root: &Path,
    preset_id: &str,
    target_dir: &Path,
    options: InstallOptions,
) -> CoreResult<InstallReport> {
    let preset = find_preset(presets_root, preset_id)?;
    let planned_files = planned_files(&preset, options)?;
    let created_target_dir = !target_dir.exists();
    fs::create_dir_all(target_dir)
        .map_err(|source| CoreError::io("create target directory", target_dir, source))?;

    let file_names = planned_files
        .iter()
        .map(|file| file.file_name)
        .collect::<Vec<_>>();
    let backup_dir = backup_existing_files(target_dir, &file_names)?;
    let installed_files = planned_files
        .into_iter()
        .map(|file| {
            install_file_content(
                &file.content,
                &target_dir.join(file.file_name),
                file.file_name,
                file.read_only,
            )
        })
        .collect::<CoreResult<Vec<_>>>()?;

    Ok(InstallReport {
        preset_id: preset.id,
        target_dir: target_dir.to_path_buf(),
        backup_dir,
        created_target_dir,
        installed_files,
    })
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

    Ok(ResetReport {
        target_dir: target_dir.to_path_buf(),
        backup_dir,
        removed_files,
    })
}

fn read_preset_content(preset_file: &Path) -> CoreResult<String> {
    fs::read_to_string(preset_file)
        .map_err(|source| CoreError::io("read preset file", preset_file, source))
}

fn planned_files(preset: &Preset, options: InstallOptions) -> CoreResult<Vec<PlannedFile>> {
    let mut files = Vec::new();

    if options.apply_streaming_fixes || options.applies_engine_tweaks() {
        let engine_content = if options.apply_streaming_fixes {
            read_preset_content(&preset.engine_ini)?
        } else {
            String::new()
        };
        let engine_content = engine_ini_content_with_options(&engine_content, options);
        files.push(PlannedFile {
            file_name: "Engine.ini",
            content: engine_content,
            read_only: options.lock_engine_ini,
            applies_balanced_performance_tweaks: false,
            applies_disable_volumetric_fog: options.apply_disable_volumetric_fog,
            applies_low_volumetric_fog: options.applies_low_volumetric_fog(),
            applies_d3d12_pso_cache: options.apply_d3d12_pso_cache,
            applies_runtime_pso_precaching: options.apply_runtime_pso_precaching,
            applies_gc_smoothing: options.apply_gc_smoothing,
            skips_intro_videos: false,
        });
    }

    if options.apply_streaming_fixes || options.apply_balanced_performance_tweaks {
        let scalability_content = if options.apply_streaming_fixes {
            read_preset_content(&preset.scalability_ini)?
        } else {
            String::new()
        };
        let scalability_content =
            scalability_ini_content_with_options(&scalability_content, options);

        files.push(PlannedFile {
            file_name: "Scalability.ini",
            content: scalability_content,
            read_only: options.lock_scalability_ini,
            applies_balanced_performance_tweaks: options.apply_balanced_performance_tweaks,
            applies_disable_volumetric_fog: false,
            applies_low_volumetric_fog: false,
            applies_d3d12_pso_cache: false,
            applies_runtime_pso_precaching: false,
            applies_gc_smoothing: false,
            skips_intro_videos: false,
        });
    }

    if options.apply_skip_intro_videos {
        files.push(PlannedFile {
            file_name: "Game.ini",
            content: game_ini_content_with_options(options),
            read_only: options.lock_game_ini,
            applies_balanced_performance_tweaks: false,
            applies_disable_volumetric_fog: false,
            applies_low_volumetric_fog: false,
            applies_d3d12_pso_cache: false,
            applies_runtime_pso_precaching: false,
            applies_gc_smoothing: false,
            skips_intro_videos: true,
        });
    }

    Ok(files)
}

fn engine_ini_content_with_options(content: &str, options: InstallOptions) -> String {
    let mut content = content.trim_end().to_string();

    for settings in [
        (
            options.apply_disable_volumetric_fog,
            DISABLE_VOLUMETRIC_FOG_ENGINE_SETTINGS,
        ),
        (
            options.applies_low_volumetric_fog(),
            LOW_VOLUMETRIC_FOG_ENGINE_SETTINGS,
        ),
        (
            options.apply_d3d12_pso_cache,
            D3D12_PSO_CACHE_ENGINE_SETTINGS,
        ),
        (
            options.apply_runtime_pso_precaching,
            RUNTIME_PSO_PRECACHING_ENGINE_SETTINGS,
        ),
        (options.apply_gc_smoothing, GC_SMOOTHING_ENGINE_SETTINGS),
    ] {
        if !settings.0 {
            continue;
        }

        if !content.is_empty() {
            content.push_str("\n\n");
        }
        content.push_str(settings.1.trim_start());
    }

    content
}

fn scalability_ini_content_with_options(content: &str, options: InstallOptions) -> String {
    if !options.apply_balanced_performance_tweaks {
        return content.to_string();
    }

    let mut content = content.trim_end().to_string();
    if !content.is_empty() {
        content.push_str("\n\n");
    }
    content.push_str(BALANCED_PERFORMANCE_SCALABILITY_SETTINGS.trim_start());
    content
}

fn game_ini_content_with_options(options: InstallOptions) -> String {
    if options.apply_skip_intro_videos {
        return SKIP_INTRO_VIDEOS_GAME_SETTINGS.trim_start().to_string();
    }

    String::new()
}

fn preview_file_content(
    preset_content: &str,
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
) -> CoreResult<FilePreview> {
    let target_exists = target_file.is_file();
    let current_pool_mb = if target_exists {
        fs::read_to_string(target_file)
            .ok()
            .and_then(|content| extract_pool_size(&content))
    } else {
        None
    };

    Ok(FilePreview {
        file_name: file_name.to_string(),
        target_exists,
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
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn installs_preset_and_backups_existing_files() {
        let root = test_dir("install");
        let presets_root = root.join("Presets");
        let preset_dir = presets_root.join("08GB_VRAM_4096MB");
        fs::create_dir_all(&preset_dir).unwrap();
        write_file(
            &preset_dir.join("Engine.ini"),
            "[SystemSettings]\nr.Streaming.PoolSize=4096\n",
        );
        write_file(
            &preset_dir.join("Scalability.ini"),
            "[TextureQuality@Cine]\nr.Streaming.PoolSize=4096\n",
        );

        let target_dir = root.join("Config").join("Windows");
        fs::create_dir_all(&target_dir).unwrap();
        write_file(
            &target_dir.join("Engine.ini"),
            "r.Streaming.PoolSize=1000\n",
        );

        let preview = preview_install(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: false,
                lock_game_ini: false,
                lock_scalability_ini: false,
                apply_streaming_fixes: true,
                apply_balanced_performance_tweaks: false,
                apply_disable_volumetric_fog: false,
                apply_low_volumetric_fog: false,
                apply_skip_intro_videos: false,
                apply_d3d12_pso_cache: false,
                apply_runtime_pso_precaching: false,
                apply_gc_smoothing: false,
            },
        )
        .unwrap();
        assert_eq!(preview[0].current_pool_mb, Some(1000));
        assert_eq!(preview[0].preset_pool_mb, Some(4096));
        assert!(!preview[0].will_apply_balanced_performance_tweaks);

        let report = install_preset(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: false,
                lock_game_ini: false,
                lock_scalability_ini: false,
                apply_streaming_fixes: true,
                apply_balanced_performance_tweaks: false,
                apply_disable_volumetric_fog: false,
                apply_low_volumetric_fog: false,
                apply_skip_intro_videos: false,
                apply_d3d12_pso_cache: false,
                apply_runtime_pso_precaching: false,
                apply_gc_smoothing: false,
            },
        )
        .unwrap();

        assert!(report
            .backup_dir
            .as_ref()
            .unwrap()
            .join("Engine.ini")
            .is_file());
        assert!(target_dir.join("Scalability.ini").is_file());
        assert_eq!(list_backups(&target_dir).unwrap().len(), 1);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn appends_balanced_performance_tweaks_only_when_requested() {
        let root = test_dir("balanced_performance");
        let presets_root = root.join("Presets");
        let preset_dir = presets_root.join("08GB_VRAM_4096MB");
        fs::create_dir_all(&preset_dir).unwrap();
        write_file(
            &preset_dir.join("Engine.ini"),
            "[SystemSettings]\nr.Streaming.PoolSize=4096\n",
        );
        write_file(
            &preset_dir.join("Scalability.ini"),
            "[TextureQuality@Cine]\nr.Streaming.PoolSize=4096\n",
        );

        let target_dir = root.join("Config").join("Windows");
        let preview = preview_install(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: true,
                lock_game_ini: true,
                lock_scalability_ini: true,
                apply_streaming_fixes: true,
                apply_balanced_performance_tweaks: true,
                apply_disable_volumetric_fog: false,
                apply_low_volumetric_fog: false,
                apply_skip_intro_videos: false,
                apply_d3d12_pso_cache: false,
                apply_runtime_pso_precaching: false,
                apply_gc_smoothing: false,
            },
        )
        .unwrap();
        assert!(!preview[0].will_apply_balanced_performance_tweaks);
        assert!(preview[1].will_apply_balanced_performance_tweaks);

        install_preset(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: false,
                lock_game_ini: false,
                lock_scalability_ini: false,
                apply_streaming_fixes: true,
                apply_balanced_performance_tweaks: true,
                apply_disable_volumetric_fog: false,
                apply_low_volumetric_fog: false,
                apply_skip_intro_videos: false,
                apply_d3d12_pso_cache: false,
                apply_runtime_pso_precaching: false,
                apply_gc_smoothing: false,
            },
        )
        .unwrap();

        let engine_ini = fs::read_to_string(target_dir.join("Engine.ini")).unwrap();
        assert!(!engine_ini.contains("; Balanced Performance Tweaks (opt-in)"));

        let scalability_ini = fs::read_to_string(target_dir.join("Scalability.ini")).unwrap();
        assert!(scalability_ini.contains("; Balanced Performance Tweaks (opt-in)"));
        assert!(scalability_ini.contains("r.Shadow.Virtual.MaxPhysicalPages=4096"));
        assert!(scalability_ini.contains("r.Lumen.TraceMeshSDFs.Allow=1"));
        assert!(scalability_ini.contains("r.SSR.Quality=3"));
        assert!(!scalability_ini.contains("r.VT.PoolSizeScale"));
        assert!(!scalability_ini.contains("r.MipMapLODBias"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn can_apply_balanced_performance_without_streaming_fixes() {
        let root = test_dir("performance_without_streaming");
        let presets_root = root.join("Presets");
        let preset_dir = presets_root.join("08GB_VRAM_4096MB");
        fs::create_dir_all(&preset_dir).unwrap();
        write_file(
            &preset_dir.join("Engine.ini"),
            "[SystemSettings]\nr.Streaming.PoolSize=4096\n",
        );
        write_file(
            &preset_dir.join("Scalability.ini"),
            "[TextureQuality@Cine]\nr.Streaming.PoolSize=4096\n",
        );

        let target_dir = root.join("Config").join("Windows");
        let preview = preview_install(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: false,
                lock_game_ini: false,
                lock_scalability_ini: false,
                apply_streaming_fixes: false,
                apply_balanced_performance_tweaks: true,
                apply_disable_volumetric_fog: false,
                apply_low_volumetric_fog: false,
                apply_skip_intro_videos: false,
                apply_d3d12_pso_cache: false,
                apply_runtime_pso_precaching: false,
                apply_gc_smoothing: false,
            },
        )
        .unwrap();

        assert_eq!(preview.len(), 1);
        assert_eq!(preview[0].file_name, "Scalability.ini");
        assert_eq!(preview[0].preset_pool_mb, None);
        assert!(preview[0].will_apply_balanced_performance_tweaks);

        let report = install_preset(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: false,
                lock_game_ini: false,
                lock_scalability_ini: false,
                apply_streaming_fixes: false,
                apply_balanced_performance_tweaks: true,
                apply_disable_volumetric_fog: false,
                apply_low_volumetric_fog: false,
                apply_skip_intro_videos: false,
                apply_d3d12_pso_cache: false,
                apply_runtime_pso_precaching: false,
                apply_gc_smoothing: false,
            },
        )
        .unwrap();

        assert_eq!(report.installed_files.len(), 1);
        assert!(!target_dir.join("Engine.ini").exists());
        assert!(target_dir.join("Scalability.ini").is_file());
        let scalability_ini = fs::read_to_string(target_dir.join("Scalability.ini")).unwrap();
        assert!(scalability_ini.contains("; Balanced Performance Tweaks (opt-in)"));
        assert!(scalability_ini.contains("r.Lumen.TraceMeshSDFs.Allow=1"));
        assert!(!scalability_ini.contains("r.Streaming.PoolSize"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn can_apply_experimental_engine_tweaks_separately() {
        let root = test_dir("experimental_engine_tweaks");
        let presets_root = root.join("Presets");
        let preset_dir = presets_root.join("08GB_VRAM_4096MB");
        fs::create_dir_all(&preset_dir).unwrap();
        write_file(
            &preset_dir.join("Engine.ini"),
            "[SystemSettings]\nr.Streaming.PoolSize=4096\n",
        );
        write_file(
            &preset_dir.join("Scalability.ini"),
            "[TextureQuality@Cine]\nr.Streaming.PoolSize=4096\n",
        );

        let target_dir = root.join("Config").join("Windows");
        let preview = preview_install(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: true,
                lock_game_ini: false,
                lock_scalability_ini: false,
                apply_streaming_fixes: false,
                apply_balanced_performance_tweaks: false,
                apply_disable_volumetric_fog: false,
                apply_low_volumetric_fog: false,
                apply_skip_intro_videos: false,
                apply_d3d12_pso_cache: true,
                apply_runtime_pso_precaching: false,
                apply_gc_smoothing: true,
            },
        )
        .unwrap();

        assert_eq!(preview.len(), 1);
        assert_eq!(preview[0].file_name, "Engine.ini");
        assert_eq!(preview[0].preset_pool_mb, None);
        assert!(preview[0].will_set_read_only);
        assert!(preview[0].will_apply_d3d12_pso_cache);
        assert!(!preview[0].will_apply_runtime_pso_precaching);
        assert!(preview[0].will_apply_gc_smoothing);

        install_preset(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: false,
                lock_game_ini: false,
                lock_scalability_ini: false,
                apply_streaming_fixes: false,
                apply_balanced_performance_tweaks: false,
                apply_disable_volumetric_fog: false,
                apply_low_volumetric_fog: false,
                apply_skip_intro_videos: false,
                apply_d3d12_pso_cache: true,
                apply_runtime_pso_precaching: true,
                apply_gc_smoothing: true,
            },
        )
        .unwrap();

        assert!(target_dir.join("Engine.ini").is_file());
        assert!(!target_dir.join("Scalability.ini").exists());
        let engine_ini = fs::read_to_string(target_dir.join("Engine.ini")).unwrap();
        assert!(engine_ini.contains("; D3D12 PSO Disk Cache (experimental opt-in)"));
        assert!(engine_ini.contains("D3D12.PSO.DiskCache=1"));
        assert!(engine_ini.contains("; Runtime PSO Precaching (experimental opt-in)"));
        assert!(engine_ini.contains("r.PSOPrecaching=1"));
        assert!(engine_ini.contains("; GC Smoothing (experimental opt-in)"));
        assert!(engine_ini.contains("gc.AllowParallelGC=1"));
        assert!(!engine_ini.contains("r.Streaming.PoolSize"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn can_disable_volumetric_fog_without_streaming_fixes() {
        let root = test_dir("disable_volumetric_fog");
        let presets_root = root.join("Presets");
        let preset_dir = presets_root.join("08GB_VRAM_4096MB");
        fs::create_dir_all(&preset_dir).unwrap();
        write_file(
            &preset_dir.join("Engine.ini"),
            "[SystemSettings]\nr.Streaming.PoolSize=4096\n",
        );
        write_file(
            &preset_dir.join("Scalability.ini"),
            "[TextureQuality@Cine]\nr.Streaming.PoolSize=4096\n",
        );

        let target_dir = root.join("Config").join("Windows");
        let preview = preview_install(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: true,
                lock_game_ini: false,
                lock_scalability_ini: false,
                apply_streaming_fixes: false,
                apply_balanced_performance_tweaks: false,
                apply_disable_volumetric_fog: true,
                apply_low_volumetric_fog: false,
                apply_skip_intro_videos: false,
                apply_d3d12_pso_cache: false,
                apply_runtime_pso_precaching: false,
                apply_gc_smoothing: false,
            },
        )
        .unwrap();

        assert_eq!(preview.len(), 1);
        assert_eq!(preview[0].file_name, "Engine.ini");
        assert_eq!(preview[0].preset_pool_mb, None);
        assert!(preview[0].will_set_read_only);
        assert!(preview[0].will_apply_disable_volumetric_fog);

        install_preset(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: false,
                lock_game_ini: false,
                lock_scalability_ini: false,
                apply_streaming_fixes: false,
                apply_balanced_performance_tweaks: false,
                apply_disable_volumetric_fog: true,
                apply_low_volumetric_fog: false,
                apply_skip_intro_videos: false,
                apply_d3d12_pso_cache: false,
                apply_runtime_pso_precaching: false,
                apply_gc_smoothing: false,
            },
        )
        .unwrap();

        assert!(target_dir.join("Engine.ini").is_file());
        assert!(!target_dir.join("Scalability.ini").exists());
        let engine_ini = fs::read_to_string(target_dir.join("Engine.ini")).unwrap();
        assert!(engine_ini.contains("; Disable Volumetric Fog (visual impact opt-in)"));
        assert!(engine_ini.contains("[SystemSettings]"));
        assert!(engine_ini.contains("r.VolumetricFog=0"));
        assert!(!engine_ini.contains("r.Streaming.PoolSize"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn can_apply_low_volumetric_fog_without_streaming_fixes() {
        let root = test_dir("low_volumetric_fog_without_streaming");
        let presets_root = root.join("Presets");
        let preset_dir = presets_root.join("08GB_VRAM_4096MB");
        fs::create_dir_all(&preset_dir).unwrap();
        write_file(
            &preset_dir.join("Engine.ini"),
            "[SystemSettings]\nr.Streaming.PoolSize=4096\n",
        );
        write_file(
            &preset_dir.join("Scalability.ini"),
            "[TextureQuality@Cine]\nr.Streaming.PoolSize=4096\n",
        );

        let target_dir = root.join("Config").join("Windows");
        let preview = preview_install(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: true,
                lock_game_ini: false,
                lock_scalability_ini: false,
                apply_streaming_fixes: false,
                apply_balanced_performance_tweaks: false,
                apply_disable_volumetric_fog: false,
                apply_low_volumetric_fog: true,
                apply_skip_intro_videos: false,
                apply_d3d12_pso_cache: false,
                apply_runtime_pso_precaching: false,
                apply_gc_smoothing: false,
            },
        )
        .unwrap();

        assert_eq!(preview.len(), 1);
        assert_eq!(preview[0].file_name, "Engine.ini");
        assert_eq!(preview[0].preset_pool_mb, None);
        assert!(preview[0].will_set_read_only);
        assert!(preview[0].will_apply_low_volumetric_fog);

        install_preset(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: false,
                lock_game_ini: false,
                lock_scalability_ini: false,
                apply_streaming_fixes: false,
                apply_balanced_performance_tweaks: false,
                apply_disable_volumetric_fog: false,
                apply_low_volumetric_fog: true,
                apply_skip_intro_videos: false,
                apply_d3d12_pso_cache: false,
                apply_runtime_pso_precaching: false,
                apply_gc_smoothing: false,
            },
        )
        .unwrap();

        assert!(target_dir.join("Engine.ini").is_file());
        assert!(!target_dir.join("Scalability.ini").exists());
        let engine_ini = fs::read_to_string(target_dir.join("Engine.ini")).unwrap();
        assert!(engine_ini.contains("; Low Volumetric Fog (visual impact opt-in)"));
        assert!(engine_ini.contains("[SystemSettings]"));
        assert!(engine_ini.contains("r.VolumetricFog=1"));
        assert!(engine_ini.contains("r.VolumetricFog.GridPixelSize=16"));
        assert!(engine_ini.contains("r.VolumetricFog.GridSizeZ=64"));
        assert!(engine_ini.contains("r.VolumetricFog.HistoryMissSupersampleCount=4"));
        assert!(!engine_ini.contains("r.Streaming.PoolSize"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn can_apply_skip_intro_videos_without_streaming_fixes() {
        let root = test_dir("skip_intro_without_streaming");
        let presets_root = root.join("Presets");
        let preset_dir = presets_root.join("08GB_VRAM_4096MB");
        fs::create_dir_all(&preset_dir).unwrap();
        write_file(
            &preset_dir.join("Engine.ini"),
            "[SystemSettings]\nr.Streaming.PoolSize=4096\n",
        );
        write_file(
            &preset_dir.join("Scalability.ini"),
            "[TextureQuality@Cine]\nr.Streaming.PoolSize=4096\n",
        );

        let target_dir = root.join("Config").join("Windows");
        let preview = preview_install(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: false,
                lock_game_ini: true,
                lock_scalability_ini: false,
                apply_streaming_fixes: false,
                apply_balanced_performance_tweaks: false,
                apply_disable_volumetric_fog: false,
                apply_low_volumetric_fog: false,
                apply_skip_intro_videos: true,
                apply_d3d12_pso_cache: false,
                apply_runtime_pso_precaching: false,
                apply_gc_smoothing: false,
            },
        )
        .unwrap();

        assert_eq!(preview.len(), 1);
        assert_eq!(preview[0].file_name, "Game.ini");
        assert_eq!(preview[0].preset_pool_mb, None);
        assert!(preview[0].will_set_read_only);
        assert!(preview[0].will_skip_intro_videos);

        let report = install_preset(
            &presets_root,
            "08GB_VRAM_4096MB",
            &target_dir,
            InstallOptions {
                lock_engine_ini: false,
                lock_game_ini: true,
                lock_scalability_ini: false,
                apply_streaming_fixes: false,
                apply_balanced_performance_tweaks: false,
                apply_disable_volumetric_fog: false,
                apply_low_volumetric_fog: false,
                apply_skip_intro_videos: true,
                apply_d3d12_pso_cache: false,
                apply_runtime_pso_precaching: false,
                apply_gc_smoothing: false,
            },
        )
        .unwrap();

        assert_eq!(report.installed_files.len(), 1);
        assert!(!target_dir.join("Engine.ini").exists());
        assert!(!target_dir.join("Scalability.ini").exists());
        assert!(target_dir.join("Game.ini").is_file());
        assert!(target_dir
            .join("Game.ini")
            .metadata()
            .unwrap()
            .permissions()
            .readonly());

        let game_ini = fs::read_to_string(target_dir.join("Game.ini")).unwrap();
        assert!(game_ini.contains("[/Script/AsyncLoadingScreen.LoadingScreenSettings]"));
        assert!(game_ini.contains("MoviePaths=(\"LoopingEngineLoadScreen\")"));
        assert!(!game_ini.contains("Alkimia_Logo"));
        assert!(!game_ini.contains("THQNordic_Logo"));
        assert!(!game_ini.contains("V_LegalScreen"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reset_to_vanilla_backs_up_and_removes_managed_files() {
        let root = test_dir("reset_to_vanilla");
        let target_dir = root.join("Config").join("Windows");
        fs::create_dir_all(&target_dir).unwrap();
        let engine_ini = target_dir.join("Engine.ini");
        let game_ini = target_dir.join("Game.ini");
        let scalability_ini = target_dir.join("Scalability.ini");
        write_file(&engine_ini, "[SystemSettings]\nr.Streaming.PoolSize=4096\n");
        write_file(
            &game_ini,
            "[/Script/AsyncLoadingScreen.LoadingScreenSettings]\n",
        );
        write_file(
            &scalability_ini,
            "[TextureQuality@Cine]\nr.Streaming.PoolSize=4096\n",
        );
        set_read_only(&engine_ini, true).unwrap();
        set_read_only(&game_ini, true).unwrap();
        set_read_only(&scalability_ini, true).unwrap();

        let report = reset_to_vanilla(&target_dir).unwrap();

        assert_eq!(
            report.removed_files,
            vec![
                "Engine.ini".to_string(),
                "Game.ini".to_string(),
                "Scalability.ini".to_string()
            ]
        );
        let backup_dir = report.backup_dir.unwrap();
        assert!(backup_dir.join("Engine.ini").is_file());
        assert!(backup_dir.join("Game.ini").is_file());
        assert!(backup_dir.join("Scalability.ini").is_file());
        assert!(!engine_ini.exists());
        assert!(!game_ini.exists());
        assert!(!scalability_ini.exists());
        assert_eq!(list_backups(&target_dir).unwrap().len(), 1);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_path_like_backup_ids() {
        assert!(validate_backup_id("../x").is_err());
        assert!(validate_backup_id(r"..\x").is_err());
        assert!(validate_backup_id("backup-123").is_ok());
    }

    #[test]
    fn creates_unique_backup_directory_when_timestamp_collides() {
        let root = test_dir("backup_collision");
        let target_dir = root.join("Config").join("Windows");
        fs::create_dir_all(backups_root(&target_dir)).unwrap();

        let existing = backups_root(&target_dir).join(backup_dir_name(1234, 0));
        fs::create_dir(&existing).unwrap();

        let backup_dir = create_backup_dir_with_timestamp(&target_dir, 1234).unwrap();

        assert_eq!(backup_dir.file_name().unwrap(), "backup-1234-1");
        assert!(existing.is_dir());
        assert!(backup_dir.is_dir());

        fs::remove_dir_all(root).unwrap();
    }

    fn test_dir(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("g1r_optimizer_core_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write_file(path: &Path, content: &str) {
        let mut file = fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }
}
