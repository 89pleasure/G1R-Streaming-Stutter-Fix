mod hardware;
mod installer;
mod paths;
mod presets;
mod recommendation;

pub use hardware::{detect_hardware, GpuInfo, GpuVendor, HardwareConfidence, HardwareSnapshot};
pub use installer::{
    ini_file_contents, install_preset, install_preset_with_strategy, list_backups, preview_install,
    reset_to_vanilla, restore_backup, BackupInfo, FileInstallReport, FileModificationState,
    FilePreview, IniFileContent, InstallOptions, InstallReport, InstallStrategy, ResetReport,
    RestoreReport,
};
pub use paths::{detect_config_paths, ConfigCandidate};
pub use presets::{find_preset, list_presets, Preset};
pub use recommendation::{recommend_preset_for_hardware, PresetRecommendation};

use std::fmt;

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreError {
    message: String,
}

impl CoreError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn io(action: &str, path: &std::path::Path, source: std::io::Error) -> Self {
        Self::new(format!("{action} '{}': {source}", path.display()))
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CoreError {}
