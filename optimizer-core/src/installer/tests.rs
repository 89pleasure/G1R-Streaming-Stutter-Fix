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
        "[SystemSettings]\nr.TextureStreaming=1\nr.Streaming.PoolSize=4096\n",
    );
    write_file(
        &preset_dir.join("Scalability.ini"),
        concat!(
            "[TextureQuality@0]\n",
            "r.Streaming.PoolSize=4096\n",
            "\n",
            "[TextureQuality@Cine]\n",
            "r.Streaming.PoolSize=4096\n",
        ),
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
            custom_pool_mb: None,
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
            custom_pool_mb: None,
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
fn preview_marks_existing_files_without_manifest_as_untracked() {
    let root = test_dir("preview_untracked_existing_file");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");
    fs::create_dir_all(&target_dir).unwrap();
    write_file(
        &target_dir.join("Engine.ini"),
        "[SystemSettings]\nr.Streaming.PoolSize=2048\n",
    );

    let preview = preview_install(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        streaming_options(),
    )
    .unwrap();
    let engine_preview = preview
        .iter()
        .find(|file| file.file_name == "Engine.ini")
        .unwrap();

    assert_eq!(
        engine_preview.modification_state,
        FileModificationState::Untracked
    );
    assert!(engine_preview.has_overwrite_risk());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preview_marks_app_written_files_as_unchanged() {
    let root = test_dir("preview_unchanged_managed_file");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");

    install_preset(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        streaming_options(),
    )
    .unwrap();

    let preview = preview_install(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        streaming_options(),
    )
    .unwrap();

    assert!(preview
        .iter()
        .filter(|file| file.target_exists)
        .all(|file| file.modification_state == FileModificationState::Unchanged));
    assert!(!preview.iter().any(FilePreview::has_overwrite_risk));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preview_marks_app_written_files_as_modified_after_external_change() {
    let root = test_dir("preview_modified_managed_file");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");

    install_preset(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        streaming_options(),
    )
    .unwrap();
    set_read_only(&target_dir.join("Engine.ini"), false).unwrap();
    write_file(
        &target_dir.join("Engine.ini"),
        "[SystemSettings]\nr.Streaming.PoolSize=2048\n",
    );

    let preview = preview_install(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        streaming_options(),
    )
    .unwrap();
    let engine_preview = preview
        .iter()
        .find(|file| file.file_name == "Engine.ini")
        .unwrap();
    let scalability_preview = preview
        .iter()
        .find(|file| file.file_name == "Scalability.ini")
        .unwrap();

    assert_eq!(
        engine_preview.modification_state,
        FileModificationState::Modified
    );
    assert!(engine_preview.has_overwrite_risk());
    assert_eq!(
        scalability_preview.modification_state,
        FileModificationState::Unchanged
    );
    assert!(!scalability_preview.has_overwrite_risk());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preview_marks_merge_available_only_for_external_settings() {
    let root = test_dir("preview_external_settings");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");
    fs::create_dir_all(&target_dir).unwrap();
    write_file(
        &target_dir.join("Engine.ini"),
        "[SystemSettings]\nr.Streaming.PoolSize=1024\nr.TextureStreaming=0\n",
    );
    write_file(
        &target_dir.join("Scalability.ini"),
        "[TextureQuality@Cine]\nr.Streaming.PoolSize=1024\nCustomTextureSetting=5\n",
    );

    let preview = preview_install(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        streaming_options(),
    )
    .unwrap();
    let engine_preview = preview
        .iter()
        .find(|file| file.file_name == "Engine.ini")
        .unwrap();
    let scalability_preview = preview
        .iter()
        .find(|file| file.file_name == "Scalability.ini")
        .unwrap();

    assert!(engine_preview.has_overwrite_risk());
    assert!(!engine_preview.has_external_settings);
    assert!(scalability_preview.has_overwrite_risk());
    assert!(scalability_preview.has_external_settings);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preview_does_not_treat_disabled_app_features_as_external_settings() {
    let root = test_dir("preview_disabled_app_feature");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");

    install_preset_with_strategy(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        InstallOptions {
            apply_balanced_performance_tweaks: true,
            ..streaming_options()
        },
        InstallStrategy::Merge,
    )
    .unwrap();

    let preview = preview_install(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        streaming_options(),
    )
    .unwrap();
    let scalability_preview = preview
        .iter()
        .find(|file| file.file_name == "Scalability.ini")
        .unwrap();

    assert_eq!(
        scalability_preview.modification_state,
        FileModificationState::Unchanged
    );
    assert!(!scalability_preview.has_external_settings);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn install_merge_removes_disabled_app_managed_settings() {
    let root = test_dir("merge_removes_disabled_app_settings");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");
    fs::create_dir_all(&target_dir).unwrap();
    write_file(
        &target_dir.join("Scalability.ini"),
        concat!(
            "[ShadowQuality@Cine]\n",
            "CustomShadowSetting=1\n",
            "\n",
            "[EffectsQuality@Cine]\n",
            "ModOnly=1\n",
        ),
    );

    install_preset_with_strategy(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        InstallOptions {
            apply_balanced_performance_tweaks: true,
            ..streaming_options()
        },
        InstallStrategy::Merge,
    )
    .unwrap();
    install_preset_with_strategy(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        streaming_options(),
        InstallStrategy::Merge,
    )
    .unwrap();

    let scalability_ini = fs::read_to_string(target_dir.join("Scalability.ini")).unwrap();
    assert!(scalability_ini.contains("CustomShadowSetting=1"));
    assert!(scalability_ini.contains("ModOnly=1"));
    assert!(!scalability_ini.contains("r.Shadow.MaxResolution=2048"));
    assert!(!scalability_ini.contains("r.VolumetricFog.GridPixelSize=8"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn install_merge_removes_disabled_app_managed_settings_from_inactive_files() {
    let root = test_dir("merge_removes_disabled_inactive_file");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");
    fs::create_dir_all(&target_dir).unwrap();
    write_file(
        &target_dir.join("Game.ini"),
        concat!(
            "[/Script/AsyncLoadingScreen.LoadingScreenSettings]\n",
            "ModLoadingScreenSetting=True\n",
        ),
    );

    install_preset_with_strategy(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        InstallOptions {
            apply_skip_intro_videos: true,
            ..streaming_options()
        },
        InstallStrategy::Merge,
    )
    .unwrap();
    install_preset_with_strategy(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        streaming_options(),
        InstallStrategy::Merge,
    )
    .unwrap();

    let game_ini = fs::read_to_string(target_dir.join("Game.ini")).unwrap();
    assert!(game_ini.contains("ModLoadingScreenSetting=True"));
    assert!(!game_ini.contains("StartupLoadingScreen="));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn install_merge_removes_disabled_app_block_scaffolding() {
    let root = test_dir("merge_removes_disabled_app_block_scaffolding");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");

    install_preset_with_strategy(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        InstallOptions {
            apply_low_volumetric_fog: true,
            apply_d3d12_pso_cache: true,
            apply_runtime_pso_precaching: true,
            apply_gc_smoothing: true,
            ..streaming_options()
        },
        InstallStrategy::Merge,
    )
    .unwrap();
    install_preset_with_strategy(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        InstallOptions {
            apply_low_volumetric_fog: true,
            ..streaming_options()
        },
        InstallStrategy::Merge,
    )
    .unwrap();

    let engine_ini = fs::read_to_string(target_dir.join("Engine.ini")).unwrap();
    assert!(engine_ini.contains("; Low Volumetric Fog (visual impact opt-in)"));
    assert!(engine_ini.contains("r.VolumetricFog.GridPixelSize=16"));
    assert!(!engine_ini.contains("; D3D12 PSO Disk Cache (experimental opt-in)"));
    assert!(!engine_ini.contains("[/Script/D3D12RHI.D3D12Options]"));
    assert!(!engine_ini.contains("; Runtime PSO Precaching (experimental opt-in)"));
    assert!(!engine_ini.contains("; GC Smoothing (experimental opt-in)"));
    assert!(!engine_ini.contains("\n\n\n"));
    assert_eq!(count_occurrences(&engine_ini, "[SystemSettings]"), 1);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn install_merge_adds_feature_comment_to_existing_section() {
    let root = test_dir("merge_adds_feature_comment");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");
    fs::create_dir_all(&target_dir).unwrap();
    write_file(
        &target_dir.join("Engine.ini"),
        concat!("[SystemSettings]\n", "ModOnly=1\n", "r.VolumetricFog=0\n"),
    );

    install_preset_with_strategy(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        InstallOptions {
            apply_low_volumetric_fog: true,
            ..streaming_options()
        },
        InstallStrategy::Merge,
    )
    .unwrap();

    let engine_ini = fs::read_to_string(target_dir.join("Engine.ini")).unwrap();
    let comment_index = engine_ini
        .find("; Low Volumetric Fog (visual impact opt-in)")
        .unwrap();
    let setting_index = engine_ini.find("r.VolumetricFog=1").unwrap();
    assert!(comment_index < setting_index);
    assert!(engine_ini.contains("ModOnly=1"));
    assert_eq!(count_occurrences(&engine_ini, "r.VolumetricFog="), 1);
    assert!(!engine_ini.contains("\n\n\n"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn install_replace_discards_existing_custom_settings() {
    let root = test_dir("replace_discards_custom_settings");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");
    fs::create_dir_all(&target_dir).unwrap();
    write_file(
        &target_dir.join("Engine.ini"),
        concat!(
            "[SystemSettings]\n",
            "r.Streaming.PoolSize=1024\n",
            "r.CustomMod.Setting=1\n",
        ),
    );

    install_preset_with_strategy(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        streaming_options(),
        InstallStrategy::Replace,
    )
    .unwrap();

    let engine_ini = fs::read_to_string(target_dir.join("Engine.ini")).unwrap();
    assert!(engine_ini.contains("r.Streaming.PoolSize=4096"));
    assert!(engine_ini.contains("r.TextureStreaming=1"));
    assert!(!engine_ini.contains("r.CustomMod.Setting=1"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn install_merges_streaming_preset_into_existing_ini_files() {
    let root = test_dir("merge_streaming_preset");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");
    fs::create_dir_all(&target_dir).unwrap();
    write_file(
        &target_dir.join("Engine.ini"),
        concat!(
            "; User comment stays\n",
            "[SystemSettings]\n",
            "; Mod setting stays\n",
            "r.Streaming.PoolSize=1024\n",
            "r.Streaming.PoolSize=2048\n",
            "r.CustomMod.Setting=1\n",
            "\n",
            "[/Script/Other.Mod]\n",
            "bEnabled=True\n",
        ),
    );
    write_file(
        &target_dir.join("Scalability.ini"),
        concat!(
            "[TextureQuality@Cine]\n",
            "; User texture comment stays\n",
            "r.Streaming.PoolSize=1024\n",
            "CustomTextureSetting=5\n",
            "\n",
            "[EffectsQuality@Cine]\n",
            "r.VolumetricFog.GridPixelSize=32\n",
            "ModOnly=1\n",
        ),
    );

    install_preset_with_strategy(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        streaming_options(),
        InstallStrategy::Merge,
    )
    .unwrap();

    let engine_ini = fs::read_to_string(target_dir.join("Engine.ini")).unwrap();
    assert!(engine_ini.contains("; User comment stays"));
    assert!(engine_ini.contains("; Mod setting stays"));
    assert!(engine_ini.contains("r.CustomMod.Setting=1"));
    assert!(engine_ini.contains("[/Script/Other.Mod]"));
    assert!(engine_ini.contains("bEnabled=True"));
    assert!(engine_ini.contains("; Streaming Fixes"));
    assert!(engine_ini.contains("r.TextureStreaming=1"));
    assert!(engine_ini.contains("r.Streaming.PoolSize=4096"));
    assert!(!engine_ini.contains("r.Streaming.PoolSize=1024"));
    assert!(!engine_ini.contains("r.Streaming.PoolSize=2048"));
    assert_eq!(count_occurrences(&engine_ini, "r.Streaming.PoolSize="), 1);

    let scalability_ini = fs::read_to_string(target_dir.join("Scalability.ini")).unwrap();
    assert!(scalability_ini.contains("; User texture comment stays"));
    assert!(scalability_ini.contains("; Streaming Fixes"));
    assert!(scalability_ini.contains("CustomTextureSetting=5"));
    assert!(scalability_ini.contains("ModOnly=1"));
    assert!(scalability_ini.contains("[TextureQuality@0]"));
    assert!(scalability_ini.contains("[TextureQuality@Cine]"));
    assert!(scalability_ini.contains("r.Streaming.PoolSize=4096"));
    assert!(!scalability_ini.contains("r.Streaming.PoolSize=1024"));

    let preview = preview_install(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        streaming_options(),
    )
    .unwrap();
    assert!(preview
        .iter()
        .filter(|file| file.target_exists)
        .all(|file| file.modification_state == FileModificationState::Unchanged));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn install_merges_balanced_performance_tweaks_without_duplicate_keys() {
    let root = test_dir("merge_balanced_performance");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");
    fs::create_dir_all(&target_dir).unwrap();
    write_file(
        &target_dir.join("Scalability.ini"),
        concat!(
            "[ShadowQuality@Cine]\n",
            "r.Shadow.MaxResolution=4096\n",
            "CustomShadowSetting=1\n",
            "\n",
            "[EffectsQuality@Cine]\n",
            "r.VolumetricFog.GridPixelSize=32\n",
            "ModOnly=1\n",
        ),
    );

    install_preset_with_strategy(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        InstallOptions {
            apply_streaming_fixes: false,
            apply_balanced_performance_tweaks: true,
            ..streaming_options()
        },
        InstallStrategy::Merge,
    )
    .unwrap();

    let scalability_ini = fs::read_to_string(target_dir.join("Scalability.ini")).unwrap();
    assert!(scalability_ini.contains("CustomShadowSetting=1"));
    assert!(scalability_ini.contains("ModOnly=1"));
    assert!(scalability_ini.contains("r.Shadow.MaxResolution=2048"));
    assert!(scalability_ini.contains("r.VolumetricFog.GridPixelSize=8"));
    assert!(scalability_ini.contains("[GlobalIlluminationQuality@Cine]"));
    assert!(!scalability_ini.contains("r.Shadow.MaxResolution=4096"));
    assert!(!scalability_ini.contains("r.VolumetricFog.GridPixelSize=32"));
    assert_eq!(
        count_occurrences(&scalability_ini, "r.Shadow.MaxResolution="),
        1
    );
    assert_eq!(
        count_occurrences(&scalability_ini, "r.VolumetricFog.GridPixelSize="),
        1
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn install_merges_skip_intro_settings_into_existing_game_ini() {
    let root = test_dir("merge_skip_intro");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");
    fs::create_dir_all(&target_dir).unwrap();
    write_file(
        &target_dir.join("Game.ini"),
        concat!(
            "[/Script/AsyncLoadingScreen.LoadingScreenSettings]\n",
            "StartupLoadingScreen=(MoviePaths=(\"OldIntro\"))\n",
            "ModLoadingScreenSetting=True\n",
        ),
    );

    install_preset_with_strategy(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        InstallOptions {
            apply_streaming_fixes: false,
            apply_skip_intro_videos: true,
            ..streaming_options()
        },
        InstallStrategy::Merge,
    )
    .unwrap();

    let game_ini = fs::read_to_string(target_dir.join("Game.ini")).unwrap();
    assert!(game_ini.contains("MoviePaths=(\"LoopingEngineLoadScreen\")"));
    assert!(game_ini.contains("ModLoadingScreenSetting=True"));
    assert!(!game_ini.contains("OldIntro"));
    assert_eq!(count_occurrences(&game_ini, "StartupLoadingScreen="), 1);

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
        "[SystemSettings]\nr.TextureStreaming=1\nr.Streaming.PoolSize=4096\n",
    );
    write_file(
        &preset_dir.join("Scalability.ini"),
        concat!(
            "[TextureQuality@0]\n",
            "r.Streaming.PoolSize=4096\n",
            "\n",
            "[TextureQuality@Cine]\n",
            "r.Streaming.PoolSize=4096\n",
        ),
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
            custom_pool_mb: None,
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
            custom_pool_mb: None,
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
            custom_pool_mb: None,
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
            custom_pool_mb: None,
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
            custom_pool_mb: None,
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
            custom_pool_mb: None,
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
    assert_eq!(count_occurrences(&engine_ini, "[SystemSettings]"), 1);
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
            custom_pool_mb: None,
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
            custom_pool_mb: None,
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
            custom_pool_mb: None,
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
            custom_pool_mb: None,
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
            custom_pool_mb: None,
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
            custom_pool_mb: None,
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

#[test]
fn prepares_ini_file_contents_for_manual_copy() {
    let root = test_dir("manual_copy_contents");
    let presets_root = create_test_preset(&root);

    let files = ini_file_contents(
        &presets_root,
        "08GB_VRAM_4096MB",
        InstallOptions {
            apply_skip_intro_videos: true,
            ..streaming_options()
        },
    )
    .unwrap();

    let engine_ini = files
        .iter()
        .find(|file| file.file_name == "Engine.ini")
        .unwrap();
    let scalability_ini = files
        .iter()
        .find(|file| file.file_name == "Scalability.ini")
        .unwrap();
    let game_ini = files
        .iter()
        .find(|file| file.file_name == "Game.ini")
        .unwrap();

    assert!(engine_ini.content.contains("; Streaming Fixes"));
    assert!(engine_ini.content.contains("r.Streaming.PoolSize=4096"));
    assert!(scalability_ini.content.contains("; Streaming Fixes"));
    assert!(game_ini.content.contains("; Skip Intro Videos (opt-in)"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn custom_pool_size_overrides_streaming_pool_in_generated_ini_contents() {
    let root = test_dir("custom_pool_contents");
    let presets_root = create_test_preset(&root);

    let files = ini_file_contents(
        &presets_root,
        "08GB_VRAM_4096MB",
        InstallOptions {
            custom_pool_mb: Some(12_288),
            ..streaming_options()
        },
    )
    .unwrap();

    let engine_ini = files
        .iter()
        .find(|file| file.file_name == "Engine.ini")
        .unwrap();
    let scalability_ini = files
        .iter()
        .find(|file| file.file_name == "Scalability.ini")
        .unwrap();

    assert!(engine_ini
        .content
        .contains("; Preset: Custom / 12288 MB pool"));
    assert!(engine_ini.content.contains("r.Streaming.PoolSize=12288"));
    assert!(!engine_ini.content.contains("r.Streaming.PoolSize=4096"));
    assert!(scalability_ini
        .content
        .contains("; Preset: Custom / 12288 MB pool"));
    assert_eq!(
        count_occurrences(&scalability_ini.content, "r.Streaming.PoolSize=12288"),
        2
    );
    assert!(!scalability_ini
        .content
        .contains("r.Streaming.PoolSize=4096"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preview_reports_custom_pool_size() {
    let root = test_dir("custom_pool_preview");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");

    let preview = preview_install(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        InstallOptions {
            custom_pool_mb: Some(6144),
            ..streaming_options()
        },
    )
    .unwrap();

    let engine_preview = preview
        .iter()
        .find(|file| file.file_name == "Engine.ini")
        .unwrap();
    let scalability_preview = preview
        .iter()
        .find(|file| file.file_name == "Scalability.ini")
        .unwrap();

    assert_eq!(engine_preview.preset_pool_mb, Some(6144));
    assert_eq!(scalability_preview.preset_pool_mb, Some(6144));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_custom_pool_size_outside_supported_range() {
    let root = test_dir("custom_pool_range");
    let presets_root = create_test_preset(&root);
    let target_dir = root.join("Config").join("Windows");

    let error = preview_install(
        &presets_root,
        "08GB_VRAM_4096MB",
        &target_dir,
        InstallOptions {
            custom_pool_mb: Some(256),
            ..streaming_options()
        },
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("custom pool size must be between 512 and 65536 MB"));

    fs::remove_dir_all(root).unwrap();
}

fn test_dir(name: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("g1r_optimizer_core_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn create_test_preset(root: &Path) -> PathBuf {
    let presets_root = root.join("Presets");
    let preset_dir = presets_root.join("08GB_VRAM_4096MB");
    fs::create_dir_all(&preset_dir).unwrap();
    write_file(
        &preset_dir.join("Engine.ini"),
        "[SystemSettings]\nr.TextureStreaming=1\nr.Streaming.PoolSize=4096\n",
    );
    write_file(
        &preset_dir.join("Scalability.ini"),
        concat!(
            "[TextureQuality@0]\n",
            "r.Streaming.PoolSize=4096\n",
            "\n",
            "[TextureQuality@Cine]\n",
            "r.Streaming.PoolSize=4096\n",
        ),
    );
    presets_root
}

fn streaming_options() -> InstallOptions {
    InstallOptions {
        lock_engine_ini: false,
        lock_game_ini: false,
        lock_scalability_ini: false,
        custom_pool_mb: None,
        apply_streaming_fixes: true,
        apply_balanced_performance_tweaks: false,
        apply_disable_volumetric_fog: false,
        apply_low_volumetric_fog: false,
        apply_skip_intro_videos: false,
        apply_d3d12_pso_cache: false,
        apply_runtime_pso_precaching: false,
        apply_gc_smoothing: false,
    }
}

fn write_file(path: &Path, content: &str) {
    let mut file = fs::File::create(path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}

fn count_occurrences(content: &str, needle: &str) -> usize {
    content.matches(needle).count()
}
