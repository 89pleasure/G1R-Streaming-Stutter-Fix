use crate::{CoreError, CoreResult};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preset {
    pub id: String,
    pub directory: PathBuf,
    pub vram_gb: u32,
    pub pool_mb: u32,
    pub engine_ini: PathBuf,
    pub scalability_ini: PathBuf,
}

impl Preset {
    pub fn label(&self) -> String {
        format!("{} GB VRAM / {} MB pool", self.vram_gb, self.pool_mb)
    }
}

pub fn list_presets(presets_root: &Path) -> CoreResult<Vec<Preset>> {
    let entries = fs::read_dir(presets_root)
        .map_err(|source| CoreError::io("read preset directory", presets_root, source))?;
    let mut presets = Vec::new();

    for entry in entries {
        let entry = entry
            .map_err(|source| CoreError::io("read preset directory entry", presets_root, source))?;
        let directory = entry.path();
        if !directory.is_dir() {
            continue;
        }

        let Some(id) = directory
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_owned)
        else {
            continue;
        };

        let Some((vram_gb, pool_mb)) = parse_preset_id(&id) else {
            continue;
        };

        let engine_ini = directory.join("Engine.ini");
        let scalability_ini = directory.join("Scalability.ini");
        if !engine_ini.is_file() || !scalability_ini.is_file() {
            continue;
        }

        presets.push(Preset {
            id,
            directory,
            vram_gb,
            pool_mb,
            engine_ini,
            scalability_ini,
        });
    }

    presets.sort_by_key(|preset| (preset.vram_gb, preset.pool_mb));
    Ok(presets)
}

pub fn find_preset(presets_root: &Path, preset_id: &str) -> CoreResult<Preset> {
    list_presets(presets_root)?
        .into_iter()
        .find(|preset| preset.id == preset_id)
        .ok_or_else(|| CoreError::new(format!("unknown preset '{preset_id}'")))
}

fn parse_preset_id(id: &str) -> Option<(u32, u32)> {
    let (vram_part, pool_part) = id.split_once("GB_VRAM_")?;
    let pool_part = pool_part.strip_suffix("MB")?;
    let vram_gb = vram_part.parse().ok()?;
    let pool_mb = pool_part.parse().ok()?;
    Some((vram_gb, pool_mb))
}

#[cfg(test)]
mod tests {
    use super::parse_preset_id;

    #[test]
    fn parses_vram_preset_ids() {
        assert_eq!(parse_preset_id("04GB_VRAM_1536MB"), Some((4, 1536)));
        assert_eq!(parse_preset_id("20GB_VRAM_10240MB"), Some((20, 10240)));
        assert_eq!(parse_preset_id("README"), None);
    }
}
