use crate::hardware::{HardwareConfidence, HardwareSnapshot};
use crate::Preset;

const PRESET_VRAM_TOLERANCE_PERCENT: u32 = 95;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetRecommendation {
    pub preset_id: String,
    pub gpu_name: String,
    pub detected_vram_mb: u32,
    pub confidence: HardwareConfidence,
    pub reason: String,
}

pub fn recommend_preset_for_hardware(
    presets: &[Preset],
    snapshot: &HardwareSnapshot,
) -> Option<PresetRecommendation> {
    let gpu = snapshot
        .gpus
        .iter()
        .filter(|gpu| gpu.confidence != HardwareConfidence::Low)
        .filter_map(|gpu| gpu.dedicated_vram_mb.map(|vram_mb| (gpu, vram_mb)))
        .max_by_key(|(_, vram_mb)| *vram_mb)?;

    let preset = presets
        .iter()
        .filter(|preset| preset_is_supported_by_vram(preset, gpu.1))
        .max_by_key(|preset| preset.vram_gb)?;

    Some(PresetRecommendation {
        preset_id: preset.id.clone(),
        gpu_name: gpu.0.name.clone(),
        detected_vram_mb: gpu.1,
        confidence: gpu.0.confidence,
        reason: format!(
            "Detected {} GB VRAM on {}. This preset is recommended.",
            gpu.1 / 1024,
            gpu.0.name
        ),
    })
}

fn preset_is_supported_by_vram(preset: &Preset, detected_vram_mb: u32) -> bool {
    let preset_vram_mb = preset.vram_gb.saturating_mul(1024);
    detected_vram_mb.saturating_mul(100)
        >= preset_vram_mb.saturating_mul(PRESET_VRAM_TOLERANCE_PERCENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::{GpuInfo, GpuVendor, HardwareConfidence, HardwareSnapshot};
    use crate::Preset;
    use std::path::PathBuf;

    #[test]
    fn recommends_exact_vram_preset_for_best_dedicated_gpu() {
        let presets = test_presets(&[4, 8, 12, 20]);
        let snapshot = HardwareSnapshot {
            gpus: vec![
                gpu(
                    "Integrated GPU",
                    GpuVendor::Intel,
                    None,
                    HardwareConfidence::Low,
                ),
                gpu(
                    "Radeon RX 7900 XT",
                    GpuVendor::Amd,
                    Some(20 * 1024),
                    HardwareConfidence::High,
                ),
            ],
            system_ram_mb: Some(32 * 1024),
            cpu_name: Some("AMD Ryzen 7 9800X3D".to_string()),
            logical_cores: Some(16),
            os_runtime: "Linux".to_string(),
        };

        let recommendation = recommend_preset_for_hardware(&presets, &snapshot).unwrap();

        assert_eq!(recommendation.preset_id, "20GB_VRAM_10240MB");
        assert_eq!(recommendation.detected_vram_mb, 20 * 1024);
        assert!(recommendation.reason.contains("20 GB VRAM"));
    }

    #[test]
    fn rounds_down_to_nearest_available_preset() {
        let presets = test_presets(&[4, 8, 10, 12]);
        let snapshot = HardwareSnapshot {
            gpus: vec![gpu(
                "Unknown 11 GB GPU",
                GpuVendor::Unknown,
                Some(11 * 1024),
                HardwareConfidence::Medium,
            )],
            system_ram_mb: None,
            cpu_name: None,
            logical_cores: None,
            os_runtime: "Windows".to_string(),
        };

        let recommendation = recommend_preset_for_hardware(&presets, &snapshot).unwrap();

        assert_eq!(recommendation.preset_id, "10GB_VRAM_5120MB");
        assert_eq!(recommendation.detected_vram_mb, 11 * 1024);
    }

    #[test]
    fn allows_small_reported_vram_gap_for_high_vram_cards() {
        let presets = test_presets(&[12, 16, 20, 24]);
        let snapshot = HardwareSnapshot {
            gpus: vec![gpu(
                "Radeon RX 7900 XT",
                GpuVendor::Amd,
                Some(19 * 1024),
                HardwareConfidence::High,
            )],
            system_ram_mb: None,
            cpu_name: None,
            logical_cores: None,
            os_runtime: "Linux".to_string(),
        };

        let recommendation = recommend_preset_for_hardware(&presets, &snapshot).unwrap();

        assert_eq!(recommendation.preset_id, "20GB_VRAM_10240MB");
        assert_eq!(recommendation.detected_vram_mb, 19 * 1024);
    }

    #[test]
    fn skips_low_confidence_or_missing_vram() {
        let presets = test_presets(&[4, 8, 12]);
        let snapshot = HardwareSnapshot {
            gpus: vec![
                gpu(
                    "Software Adapter",
                    GpuVendor::Unknown,
                    None,
                    HardwareConfidence::Low,
                ),
                gpu(
                    "Shared Memory GPU",
                    GpuVendor::Intel,
                    Some(8 * 1024),
                    HardwareConfidence::Low,
                ),
            ],
            system_ram_mb: Some(16 * 1024),
            cpu_name: None,
            logical_cores: Some(8),
            os_runtime: "Windows".to_string(),
        };

        assert_eq!(recommend_preset_for_hardware(&presets, &snapshot), None);
    }

    fn gpu(
        name: &str,
        vendor: GpuVendor,
        dedicated_vram_mb: Option<u32>,
        confidence: HardwareConfidence,
    ) -> GpuInfo {
        GpuInfo {
            name: name.to_string(),
            vendor,
            dedicated_vram_mb,
            shared_memory_mb: None,
            source: "test".to_string(),
            confidence,
        }
    }

    fn test_presets(vram_values: &[u32]) -> Vec<Preset> {
        vram_values
            .iter()
            .map(|vram_gb| Preset {
                id: format!("{vram_gb:02}GB_VRAM_{}MB", pool_for_vram(*vram_gb)),
                directory: PathBuf::from(format!("{vram_gb:02}GB_VRAM")),
                vram_gb: *vram_gb,
                pool_mb: pool_for_vram(*vram_gb),
                engine_ini: PathBuf::from("Engine.ini"),
                scalability_ini: PathBuf::from("Scalability.ini"),
            })
            .collect()
    }

    fn pool_for_vram(vram_gb: u32) -> u32 {
        match vram_gb {
            4 => 1536,
            _ => vram_gb * 512,
        }
    }
}
