use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const GOTHIC_1_REMAKE_STEAM_APP_ID: &str = "1297900";
pub const GOTHIC_1_REMAKE_STEAM_URI: &str = "steam://rungameid/1297900";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigCandidate {
    pub label: String,
    pub path: PathBuf,
    pub exists: bool,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchCandidateKind {
    Steam,
    Executable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchCandidate {
    pub id: String,
    pub kind: LaunchCandidateKind,
    pub label: String,
    pub path: PathBuf,
    pub exists: bool,
    pub source: String,
}

pub fn detect_config_paths() -> Vec<ConfigCandidate> {
    let mut candidates = Vec::new();

    add_windows_local_app_data_candidates(&mut candidates);
    add_linux_steam_candidates(&mut candidates);

    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter(|candidate| seen.insert(normalize_for_dedup(&candidate.path)))
        .collect()
}

pub fn detect_launch_candidates(manual_executable_path: Option<&Path>) -> Vec<LaunchCandidate> {
    launch_candidates_from_steamapps_roots(&steamapps_roots(), manual_executable_path)
}

pub fn launch_candidates_from_steamapps_roots(
    steamapps_roots: &[PathBuf],
    manual_executable_path: Option<&Path>,
) -> Vec<LaunchCandidate> {
    let mut candidates = Vec::new();

    for steamapps in steamapps_roots {
        push_steam_launch_candidate(&mut candidates, steamapps);
        push_steam_executable_candidates(&mut candidates, steamapps);
    }
    push_manual_launch_candidate(&mut candidates, manual_executable_path);

    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter(|candidate| seen.insert(candidate.id.clone()))
        .collect()
}

fn add_windows_local_app_data_candidates(candidates: &mut Vec<ConfigCandidate>) {
    let Some(local_app_data) = env::var_os("LOCALAPPDATA").map(PathBuf::from) else {
        return;
    };

    push_candidate(
        candidates,
        "Windows Steam",
        local_app_data
            .join("G1R")
            .join("Saved")
            .join("Config")
            .join("Windows"),
        "LOCALAPPDATA",
    );
    push_candidate(
        candidates,
        "Legacy WindowsNoEditor",
        local_app_data
            .join("Gothic")
            .join("Saved")
            .join("Config")
            .join("WindowsNoEditor"),
        "LOCALAPPDATA",
    );
    push_candidate(
        candidates,
        "Demo Windows",
        local_app_data
            .join("G1R_NyrasDemo")
            .join("Saved")
            .join("Config")
            .join("Windows"),
        "LOCALAPPDATA",
    );
}

fn add_linux_steam_candidates(candidates: &mut Vec<ConfigCandidate>) {
    let steamapps_roots = steamapps_roots();
    for steamapps in steamapps_roots {
        let label = if steamapps
            .to_string_lossy()
            .contains(".var/app/com.valvesoftware.Steam")
        {
            "Linux Flatpak Steam Proton"
        } else {
            "Linux Steam Proton"
        };

        push_candidate(
            candidates,
            label,
            steamapps
                .join("compatdata")
                .join("1297900")
                .join("pfx")
                .join("drive_c")
                .join("users")
                .join("steamuser")
                .join("AppData")
                .join("Local")
                .join("G1R")
                .join("Saved")
                .join("Config")
                .join("Windows"),
            "Steam library",
        );
    }
}

fn steamapps_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return roots;
    };

    roots.push(home.join(".steam").join("steam").join("steamapps"));
    roots.push(
        home.join(".local")
            .join("share")
            .join("Steam")
            .join("steamapps"),
    );
    roots.push(
        home.join(".var")
            .join("app")
            .join("com.valvesoftware.Steam")
            .join(".local")
            .join("share")
            .join("Steam")
            .join("steamapps"),
    );

    let mut extra_roots = Vec::new();
    for root in &roots {
        extra_roots.extend(read_steam_libraryfolders(root));
    }
    roots.extend(extra_roots);

    let mut seen = BTreeSet::new();
    roots
        .into_iter()
        .filter(|root| seen.insert(normalize_for_dedup(root)))
        .collect()
}

fn read_steam_libraryfolders(steamapps_root: &Path) -> Vec<PathBuf> {
    let libraryfolders = steamapps_root.join("libraryfolders.vdf");
    let Ok(content) = fs::read_to_string(&libraryfolders) else {
        return Vec::new();
    };

    content
        .lines()
        .filter_map(parse_steam_library_path)
        .map(|path| PathBuf::from(path).join("steamapps"))
        .collect()
}

fn parse_steam_library_path(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.trim().split('"').collect();
    if parts.len() < 4 || parts[1] != "path" {
        return None;
    }

    Some(parts[3].replace("\\\\", "\\"))
}

fn push_steam_launch_candidate(candidates: &mut Vec<LaunchCandidate>, steamapps_root: &Path) {
    let manifest = steam_appmanifest_path(steamapps_root);
    if !manifest.is_file() {
        return;
    }

    candidates.push(LaunchCandidate {
        id: launch_candidate_id("steam", &manifest),
        kind: LaunchCandidateKind::Steam,
        label: "Steam: Gothic 1 Remake".to_string(),
        path: manifest,
        exists: true,
        source: "Steam library".to_string(),
    });
}

fn push_steam_executable_candidates(candidates: &mut Vec<LaunchCandidate>, steamapps_root: &Path) {
    let manifest = steam_appmanifest_path(steamapps_root);
    if !manifest.is_file() {
        return;
    }

    let install_dir = steam_install_dir(&manifest)
        .unwrap_or_else(|| "Gothic 1 Remake".to_string())
        .replace("\\\\", "\\");
    let game_root = steamapps_root.join("common").join(install_dir);

    for executable in executable_candidates_for_game_root(&game_root) {
        if !executable.is_file() {
            continue;
        }

        candidates.push(LaunchCandidate {
            id: launch_candidate_id("executable", &executable),
            kind: LaunchCandidateKind::Executable,
            label: "Executable: Gothic 1 Remake".to_string(),
            path: executable,
            exists: true,
            source: "Steam library".to_string(),
        });
    }
}

fn push_manual_launch_candidate(
    candidates: &mut Vec<LaunchCandidate>,
    manual_executable_path: Option<&Path>,
) {
    let Some(path) = manual_executable_path else {
        return;
    };

    let path = path.to_path_buf();
    candidates.push(LaunchCandidate {
        id: launch_candidate_id("manual", &path),
        kind: LaunchCandidateKind::Executable,
        label: "Manual executable".to_string(),
        exists: path.is_file(),
        path,
        source: "manual".to_string(),
    });
}

fn steam_appmanifest_path(steamapps_root: &Path) -> PathBuf {
    steamapps_root.join(format!("appmanifest_{GOTHIC_1_REMAKE_STEAM_APP_ID}.acf"))
}

fn steam_install_dir(manifest: &Path) -> Option<String> {
    let content = fs::read_to_string(manifest).ok()?;
    content.lines().find_map(parse_steam_installdir)
}

fn parse_steam_installdir(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.trim().split('"').collect();
    if parts.len() < 4 || parts[1] != "installdir" {
        return None;
    }

    Some(parts[3].replace("\\\\", "\\"))
}

fn executable_candidates_for_game_root(game_root: &Path) -> Vec<PathBuf> {
    [
        game_root.join("G1R.exe"),
        game_root.join("Gothic1Remake.exe"),
        game_root.join("Gothic 1 Remake.exe"),
        game_root
            .join("G1R")
            .join("Binaries")
            .join("Win64")
            .join("G1R-Win64-Shipping.exe"),
    ]
    .into()
}

fn launch_candidate_id(kind: &str, path: &Path) -> String {
    format!("{kind}:{}", normalize_for_dedup(path))
}

fn push_candidate(
    candidates: &mut Vec<ConfigCandidate>,
    label: impl Into<String>,
    path: PathBuf,
    source: impl Into<String>,
) {
    let exists = path.is_dir();
    candidates.push(ConfigCandidate {
        label: label.into(),
        path,
        exists,
        source: source.into(),
    });
}

fn normalize_for_dedup(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::{
        launch_candidates_from_steamapps_roots, parse_steam_library_path, LaunchCandidateKind,
    };
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn parses_steam_library_path_lines() {
        assert_eq!(
            parse_steam_library_path(r#""path" "/gaming/SteamLibrary""#),
            Some("/gaming/SteamLibrary".to_string())
        );
        assert_eq!(parse_steam_library_path(r#""apps" "1297900""#), None);
    }

    #[test]
    fn detects_steam_launch_candidate_from_appmanifest() {
        let root = unique_temp_dir("steam-launch-candidate");
        let steamapps = root.join("SteamLibrary").join("steamapps");
        fs::create_dir_all(&steamapps).expect("create steamapps");
        fs::write(
            steamapps.join("appmanifest_1297900.acf"),
            r#""appid" "1297900""#,
        )
        .expect("write manifest");

        let candidates = launch_candidates_from_steamapps_roots(&[steamapps], None);

        assert_eq!(candidates[0].kind, LaunchCandidateKind::Steam);
        assert_eq!(candidates[0].label, "Steam: Gothic 1 Remake");
        assert!(candidates[0].exists);
    }

    #[test]
    fn steam_launch_candidate_is_preferred_over_executable() {
        let root = unique_temp_dir("steam-launch-preferred");
        let steamapps = root.join("SteamLibrary").join("steamapps");
        let game_dir = steamapps.join("common").join("Gothic 1 Remake");
        fs::create_dir_all(&game_dir).expect("create game dir");
        fs::write(
            steamapps.join("appmanifest_1297900.acf"),
            r#""appid" "1297900""#,
        )
        .expect("write manifest");
        fs::write(game_dir.join("G1R.exe"), "").expect("write exe");

        let candidates = launch_candidates_from_steamapps_roots(&[steamapps], None);

        assert_eq!(candidates[0].kind, LaunchCandidateKind::Steam);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.kind == LaunchCandidateKind::Executable));
    }

    #[test]
    fn includes_valid_manual_executable_candidate() {
        let root = unique_temp_dir("manual-launch-candidate");
        let executable = root.join("G1R.exe");
        fs::write(&executable, "").expect("write executable");

        let candidates = launch_candidates_from_steamapps_roots(&[], Some(&executable));

        assert_eq!(candidates[0].kind, LaunchCandidateKind::Executable);
        assert_eq!(candidates[0].source, "manual");
        assert!(candidates[0].exists);
    }

    #[test]
    fn marks_missing_manual_executable_candidate() {
        let root = unique_temp_dir("missing-manual-launch-candidate");
        let executable = root.join("Missing.exe");

        let candidates = launch_candidates_from_steamapps_roots(&[], Some(&executable));

        assert_eq!(candidates[0].kind, LaunchCandidateKind::Executable);
        assert_eq!(candidates[0].source, "manual");
        assert!(!candidates[0].exists);
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("g1r-optimizer-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }
}
