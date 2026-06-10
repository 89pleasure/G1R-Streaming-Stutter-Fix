use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigCandidate {
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
    use super::parse_steam_library_path;

    #[test]
    fn parses_steam_library_path_lines() {
        assert_eq!(
            parse_steam_library_path(r#""path" "/gaming/SteamLibrary""#),
            Some("/gaming/SteamLibrary".to_string())
        );
        assert_eq!(parse_steam_library_path(r#""apps" "1297900""#), None);
    }
}
