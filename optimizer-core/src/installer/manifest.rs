use crate::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::MANAGED_DIR;

const MANIFEST_FILE: &str = "manifest.json";
const MANIFEST_VERSION: u32 = 1;
const CHECKSUM_ALGORITHM: &str = "fnv1a64";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileModificationState {
    Missing,
    Unchanged,
    Untracked,
    Modified,
}

impl FileModificationState {
    pub fn has_overwrite_risk(self) -> bool {
        matches!(self, Self::Untracked | Self::Modified)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct InstallManifest {
    version: u32,
    checksum_algorithm: String,
    files: Vec<ManifestFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ManifestFile {
    file_name: String,
    checksum: String,
    byte_len: u64,
    installed_unix_seconds: u64,
}

impl ManifestFile {
    pub(super) fn from_content(
        file_name: &str,
        content: &str,
        installed_unix_seconds: u64,
    ) -> Self {
        Self {
            file_name: file_name.to_string(),
            checksum: checksum_bytes(content.as_bytes()),
            byte_len: content.len() as u64,
            installed_unix_seconds,
        }
    }
}

impl InstallManifest {
    fn empty() -> Self {
        Self {
            version: MANIFEST_VERSION,
            checksum_algorithm: CHECKSUM_ALGORITHM.to_string(),
            files: Vec::new(),
        }
    }

    fn find_file(&self, file_name: &str) -> Option<&ManifestFile> {
        if self.version != MANIFEST_VERSION || self.checksum_algorithm != CHECKSUM_ALGORITHM {
            return None;
        }

        self.files.iter().find(|file| file.file_name == file_name)
    }

    fn update_file(&mut self, file: ManifestFile) {
        if let Some(existing_file) = self
            .files
            .iter_mut()
            .find(|existing_file| existing_file.file_name == file.file_name)
        {
            *existing_file = file;
        } else {
            self.files.push(file);
        }

        self.files
            .sort_by(|left, right| left.file_name.cmp(&right.file_name));
    }

    fn remove_files(&mut self, file_names: &[String]) {
        self.files.retain(|file| {
            !file_names
                .iter()
                .any(|file_name| file_name == &file.file_name)
        });
    }
}

pub(super) fn classify_modification_state(
    file_name: &str,
    target_exists: bool,
    current_bytes: Option<&[u8]>,
    manifest: Option<&InstallManifest>,
) -> FileModificationState {
    if !target_exists {
        return FileModificationState::Missing;
    }

    let Some(current_bytes) = current_bytes else {
        return FileModificationState::Untracked;
    };

    let Some(manifest_file) = manifest.and_then(|manifest| manifest.find_file(file_name)) else {
        return FileModificationState::Untracked;
    };

    if manifest_file.checksum == checksum_bytes(current_bytes) {
        FileModificationState::Unchanged
    } else {
        FileModificationState::Modified
    }
}

fn managed_root(target_dir: &Path) -> PathBuf {
    target_dir.join(MANAGED_DIR)
}

fn manifest_path(target_dir: &Path) -> PathBuf {
    managed_root(target_dir).join(MANIFEST_FILE)
}

pub(super) fn load_manifest(target_dir: &Path) -> Option<InstallManifest> {
    let content = fs::read_to_string(manifest_path(target_dir)).ok()?;
    serde_json::from_str(&content).ok()
}

pub(super) fn update_manifest_files(target_dir: &Path, files: Vec<ManifestFile>) -> CoreResult<()> {
    let mut manifest = load_manifest(target_dir).unwrap_or_else(InstallManifest::empty);
    for file in files {
        manifest.update_file(file);
    }
    write_manifest(target_dir, &manifest)
}

pub(super) fn remove_manifest_files(target_dir: &Path, file_names: &[String]) -> CoreResult<()> {
    let Some(mut manifest) = load_manifest(target_dir) else {
        return Ok(());
    };

    manifest.remove_files(file_names);
    if manifest.files.is_empty() {
        let path = manifest_path(target_dir);
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(CoreError::io("remove manifest", &path, error)),
        }
    } else {
        write_manifest(target_dir, &manifest)
    }
}

fn write_manifest(target_dir: &Path, manifest: &InstallManifest) -> CoreResult<()> {
    let managed_root = managed_root(target_dir);
    fs::create_dir_all(&managed_root)
        .map_err(|source| CoreError::io("create managed directory", &managed_root, source))?;
    let content = serde_json::to_string_pretty(manifest)
        .map_err(|source| CoreError::new(format!("serialize manifest: {source}")))?;
    let path = manifest_path(target_dir);
    fs::write(&path, content).map_err(|source| CoreError::io("write manifest", &path, source))
}

pub(super) fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn checksum_bytes(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
