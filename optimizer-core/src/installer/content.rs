use crate::presets::Preset;
use crate::{CoreError, CoreResult};
use std::fs;
use std::path::Path;

use super::InstallOptions;

const STREAMING_FIXES_SETTINGS_COMMENT: &str = "; Streaming Fixes";
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

#[derive(Clone)]
pub(super) struct PlannedFile {
    pub(super) file_name: &'static str,
    pub(super) content: String,
    pub(super) managed_content: String,
    pub(super) read_only: bool,
    pub(super) applies_balanced_performance_tweaks: bool,
    pub(super) applies_disable_volumetric_fog: bool,
    pub(super) applies_low_volumetric_fog: bool,
    pub(super) applies_d3d12_pso_cache: bool,
    pub(super) applies_runtime_pso_precaching: bool,
    pub(super) applies_gc_smoothing: bool,
    pub(super) skips_intro_videos: bool,
}

fn read_preset_content(preset_file: &Path) -> CoreResult<String> {
    fs::read_to_string(preset_file)
        .map_err(|source| CoreError::io("read preset file", preset_file, source))
}

pub(super) fn planned_files(
    preset: &Preset,
    options: InstallOptions,
) -> CoreResult<Vec<PlannedFile>> {
    Ok(managed_files(preset, options)?
        .into_iter()
        .filter(|file| !file.content.trim().is_empty())
        .collect())
}

pub(super) fn managed_files(
    preset: &Preset,
    options: InstallOptions,
) -> CoreResult<Vec<PlannedFile>> {
    let mut files = Vec::new();

    let preset_engine_content = read_preset_content(&preset.engine_ini)?;
    let preset_engine_content =
        streaming_preset_content_with_custom_pool(&preset_engine_content, options.custom_pool_mb);
    let preset_engine_content = streaming_fixes_ini_content(&preset_engine_content);
    let engine_content = if options.apply_streaming_fixes {
        preset_engine_content.clone()
    } else {
        String::new()
    };
    let engine_content = engine_ini_content_with_options(&engine_content, options);
    files.push(PlannedFile {
        file_name: "Engine.ini",
        content: engine_content,
        managed_content: managed_engine_ini_content(&preset_engine_content),
        read_only: options.lock_engine_ini,
        applies_balanced_performance_tweaks: false,
        applies_disable_volumetric_fog: options.apply_disable_volumetric_fog,
        applies_low_volumetric_fog: options.applies_low_volumetric_fog(),
        applies_d3d12_pso_cache: options.apply_d3d12_pso_cache,
        applies_runtime_pso_precaching: options.apply_runtime_pso_precaching,
        applies_gc_smoothing: options.apply_gc_smoothing,
        skips_intro_videos: false,
    });

    let preset_scalability_content = read_preset_content(&preset.scalability_ini)?;
    let preset_scalability_content = streaming_preset_content_with_custom_pool(
        &preset_scalability_content,
        options.custom_pool_mb,
    );
    let preset_scalability_content = streaming_fixes_ini_content(&preset_scalability_content);
    let scalability_content = if options.apply_streaming_fixes {
        preset_scalability_content.clone()
    } else {
        String::new()
    };
    let scalability_content = scalability_ini_content_with_options(&scalability_content, options);

    files.push(PlannedFile {
        file_name: "Scalability.ini",
        content: scalability_content,
        managed_content: managed_scalability_ini_content(&preset_scalability_content),
        read_only: options.lock_scalability_ini,
        applies_balanced_performance_tweaks: options.apply_balanced_performance_tweaks,
        applies_disable_volumetric_fog: false,
        applies_low_volumetric_fog: false,
        applies_d3d12_pso_cache: false,
        applies_runtime_pso_precaching: false,
        applies_gc_smoothing: false,
        skips_intro_videos: false,
    });

    files.push(PlannedFile {
        file_name: "Game.ini",
        content: game_ini_content_with_options(options),
        managed_content: managed_game_ini_content(),
        read_only: options.lock_game_ini,
        applies_balanced_performance_tweaks: false,
        applies_disable_volumetric_fog: false,
        applies_low_volumetric_fog: false,
        applies_d3d12_pso_cache: false,
        applies_runtime_pso_precaching: false,
        applies_gc_smoothing: false,
        skips_intro_videos: options.apply_skip_intro_videos,
    });

    Ok(files)
}

fn streaming_preset_content_with_custom_pool(content: &str, custom_pool_mb: Option<u32>) -> String {
    let Some(pool_mb) = custom_pool_mb else {
        return content.to_string();
    };

    let custom_preset_comment = format!("; Preset: Custom / {pool_mb} MB pool");
    let had_final_newline = content.ends_with('\n');
    let mut has_preset_comment = false;
    let mut lines = content
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("; Preset:") {
                has_preset_comment = true;
                return custom_preset_comment.clone();
            }

            if is_streaming_pool_size_line(trimmed) {
                return format!("r.Streaming.PoolSize={pool_mb}");
            }

            line.to_string()
        })
        .collect::<Vec<_>>();

    if !has_preset_comment {
        let insert_index = lines
            .iter()
            .position(|line| line.trim_start().starts_with('['))
            .unwrap_or(0);
        lines.insert(insert_index, custom_preset_comment);
    }

    let mut content = lines.join("\n");
    if had_final_newline && !content.is_empty() {
        content.push('\n');
    }
    content
}

fn is_streaming_pool_size_line(line: &str) -> bool {
    if line.starts_with(';') || line.starts_with('#') {
        return false;
    }

    line.split_once('=')
        .is_some_and(|(key, _value)| key.trim() == "r.Streaming.PoolSize")
}

fn streaming_fixes_ini_content(content: &str) -> String {
    let content = content.trim_start();
    if content.is_empty() || content.starts_with(STREAMING_FIXES_SETTINGS_COMMENT) {
        return content.to_string();
    }

    let lines = content.lines().map(str::to_string).collect::<Vec<_>>();
    let Some(first_section_index) = lines
        .iter()
        .position(|line| line.trim_start().starts_with('['))
    else {
        return format!("{STREAMING_FIXES_SETTINGS_COMMENT}\n{content}");
    };

    let mut with_comment = Vec::with_capacity(lines.len() + 1);
    with_comment.extend(lines[..first_section_index].iter().cloned());
    with_comment.push(STREAMING_FIXES_SETTINGS_COMMENT.to_string());
    with_comment.extend(lines[first_section_index..].iter().cloned());
    with_comment.join("\n")
}

fn managed_engine_ini_content(preset_content: &str) -> String {
    [
        preset_content,
        DISABLE_VOLUMETRIC_FOG_ENGINE_SETTINGS,
        LOW_VOLUMETRIC_FOG_ENGINE_SETTINGS,
        D3D12_PSO_CACHE_ENGINE_SETTINGS,
        RUNTIME_PSO_PRECACHING_ENGINE_SETTINGS,
        GC_SMOOTHING_ENGINE_SETTINGS,
    ]
    .join("\n")
}

fn managed_scalability_ini_content(preset_content: &str) -> String {
    [preset_content, BALANCED_PERFORMANCE_SCALABILITY_SETTINGS].join("\n")
}

fn managed_game_ini_content() -> String {
    SKIP_INTRO_VIDEOS_GAME_SETTINGS.to_string()
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
        let settings_content =
            if last_section_name(&content).is_some_and(|name| name == "SystemSettings") {
                settings_without_section_header(settings.1, "SystemSettings")
            } else {
                settings.1.trim_start().to_string()
            };
        content.push_str(&settings_content);
    }

    if options.apply_d3d12_pso_cache {
        if !content.is_empty() {
            content.push_str("\n\n");
        }
        content.push_str(D3D12_PSO_CACHE_ENGINE_SETTINGS.trim_start());
    }

    content
}

fn settings_without_section_header(settings: &str, section_name: &str) -> String {
    settings
        .trim_start()
        .lines()
        .filter(|line| line.trim() != format!("[{section_name}]"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn last_section_name(content: &str) -> Option<&str> {
    content.lines().rev().find_map(|line| {
        let trimmed = line.trim();
        if !trimmed.starts_with('[') {
            return None;
        }

        let close_index = trimmed.find(']')?;
        Some(trimmed[1..close_index].trim())
    })
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
