use sysinfo::System;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor {
    Amd,
    Intel,
    Nvidia,
    Microsoft,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: GpuVendor,
    pub dedicated_vram_mb: Option<u32>,
    pub shared_memory_mb: Option<u32>,
    pub source: String,
    pub confidence: HardwareConfidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareSnapshot {
    pub gpus: Vec<GpuInfo>,
    pub system_ram_mb: Option<u64>,
    pub cpu_name: Option<String>,
    pub logical_cores: Option<usize>,
    pub os_runtime: String,
}

pub fn detect_hardware() -> HardwareSnapshot {
    let mut system = System::new_all();
    system.refresh_all();

    HardwareSnapshot {
        gpus: detect_gpus(),
        system_ram_mb: Some(bytes_to_mb_u64(system.total_memory())),
        cpu_name: system
            .cpus()
            .first()
            .map(|cpu| cpu.brand().trim().to_string())
            .filter(|name| !name.is_empty()),
        logical_cores: Some(system.cpus().len()),
        os_runtime: os_runtime(),
    }
}

fn os_runtime() -> String {
    let os_name = System::name().unwrap_or_else(|| std::env::consts::OS.to_string());
    let os_version = System::os_version();
    match os_version {
        Some(version) if !version.is_empty() => format!("{os_name} {version}"),
        _ => os_name,
    }
}

#[cfg(windows)]
fn detect_gpus() -> Vec<GpuInfo> {
    windows::detect_windows_gpus()
}

#[cfg(target_os = "linux")]
fn detect_gpus() -> Vec<GpuInfo> {
    linux::detect_linux_gpus()
}

#[cfg(not(any(windows, target_os = "linux")))]
fn detect_gpus() -> Vec<GpuInfo> {
    Vec::new()
}

fn bytes_to_mb_u64(bytes: u64) -> u64 {
    bytes / 1024 / 1024
}

fn bytes_to_mb_u32(bytes: u64) -> Option<u32> {
    u32::try_from(bytes_to_mb_u64(bytes)).ok()
}

#[cfg(target_os = "linux")]
mod linux {
    use super::{bytes_to_mb_u32, GpuInfo, GpuVendor, HardwareConfidence};
    use std::fs;
    use std::path::{Path, PathBuf};

    pub fn detect_linux_gpus() -> Vec<GpuInfo> {
        let mut gpus = detect_drm_gpus();
        let nvml_gpus = detect_nvml_gpus();

        if !nvml_gpus.is_empty() {
            gpus.retain(|gpu| gpu.vendor != GpuVendor::Nvidia);
            gpus.extend(nvml_gpus);
        }

        gpus
    }

    fn detect_drm_gpus() -> Vec<GpuInfo> {
        let Ok(entries) = fs::read_dir("/sys/class/drm") else {
            return Vec::new();
        };

        entries
            .filter_map(Result::ok)
            .filter(|entry| is_drm_card(entry.file_name().to_string_lossy().as_ref()))
            .filter_map(|entry| drm_gpu_from_path(entry.path()))
            .collect()
    }

    fn drm_gpu_from_path(card_path: PathBuf) -> Option<GpuInfo> {
        let device_path = card_path.join("device");
        let vendor = read_vendor(&device_path);
        let dedicated_vram_mb =
            read_u64(&device_path.join("mem_info_vram_total")).and_then(bytes_to_mb_u32);
        let card_name = card_path.file_name()?.to_string_lossy();
        let confidence = if dedicated_vram_mb.is_some() {
            HardwareConfidence::High
        } else {
            HardwareConfidence::Low
        };

        Some(GpuInfo {
            name: format!("{} {}", vendor_label(vendor), card_name),
            vendor,
            dedicated_vram_mb,
            shared_memory_mb: None,
            source: "Linux DRM sysfs".to_string(),
            confidence,
        })
    }

    fn detect_nvml_gpus() -> Vec<GpuInfo> {
        let Ok(nvml) = nvml_wrapper::Nvml::init() else {
            return Vec::new();
        };
        let Ok(device_count) = nvml.device_count() else {
            return Vec::new();
        };

        (0..device_count)
            .filter_map(|index| {
                let device = nvml.device_by_index(index).ok()?;
                let name = device
                    .name()
                    .unwrap_or_else(|_| format!("NVIDIA GPU {index}"));
                let dedicated_vram_mb = device
                    .memory_info()
                    .ok()
                    .and_then(|memory| bytes_to_mb_u32(memory.total));

                Some(GpuInfo {
                    name,
                    vendor: GpuVendor::Nvidia,
                    dedicated_vram_mb,
                    shared_memory_mb: None,
                    source: "NVIDIA NVML".to_string(),
                    confidence: if dedicated_vram_mb.is_some() {
                        HardwareConfidence::High
                    } else {
                        HardwareConfidence::Low
                    },
                })
            })
            .collect()
    }

    fn is_drm_card(name: &str) -> bool {
        name.strip_prefix("card").is_some_and(|suffix| {
            !suffix.is_empty() && suffix.chars().all(|char| char.is_ascii_digit())
        })
    }

    fn read_vendor(device_path: &Path) -> GpuVendor {
        match read_trimmed(&device_path.join("vendor")).as_deref() {
            Some("0x1002") => GpuVendor::Amd,
            Some("0x8086") => GpuVendor::Intel,
            Some("0x10de") => GpuVendor::Nvidia,
            _ => GpuVendor::Unknown,
        }
    }

    fn read_u64(path: &Path) -> Option<u64> {
        read_trimmed(path)?.parse().ok()
    }

    fn read_trimmed(path: &Path) -> Option<String> {
        fs::read_to_string(path)
            .ok()
            .map(|content| content.trim().to_string())
            .filter(|content| !content.is_empty())
    }

    fn vendor_label(vendor: GpuVendor) -> &'static str {
        match vendor {
            GpuVendor::Amd => "AMD",
            GpuVendor::Intel => "Intel",
            GpuVendor::Nvidia => "NVIDIA",
            GpuVendor::Microsoft => "Microsoft",
            GpuVendor::Unknown => "GPU",
        }
    }
}

#[cfg(windows)]
mod windows {
    use super::{bytes_to_mb_u32, GpuInfo, GpuVendor, HardwareConfidence};
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
    };

    pub fn detect_windows_gpus() -> Vec<GpuInfo> {
        let Ok(factory) = create_factory() else {
            return Vec::new();
        };

        let mut gpus = Vec::new();
        let mut index = 0;
        while let Ok(adapter) = unsafe {
            // SAFETY: The factory object is valid for the duration of the call and DXGI owns the
            // returned COM interface lifetime through the windows crate wrapper.
            factory.EnumAdapters1(index)
        } {
            if let Ok(desc) = unsafe {
                // SAFETY: The adapter COM interface came from DXGI enumeration and remains valid
                // for this call. GetDesc1 only writes to an internal output managed by windows-rs.
                adapter.GetDesc1()
            } {
                let software_flag = DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32;
                let is_software = desc.Flags & software_flag != 0;
                let dedicated_vram_mb = bytes_to_mb_u32(desc.DedicatedVideoMemory as u64);

                gpus.push(GpuInfo {
                    name: wide_name(&desc.Description),
                    vendor: vendor_from_id(desc.VendorId),
                    dedicated_vram_mb,
                    shared_memory_mb: bytes_to_mb_u32(desc.SharedSystemMemory as u64),
                    source: "Windows DXGI".to_string(),
                    confidence: if is_software || dedicated_vram_mb.is_none() {
                        HardwareConfidence::Low
                    } else {
                        HardwareConfidence::High
                    },
                });
            }
            index += 1;
        }

        gpus
    }

    fn create_factory() -> windows::core::Result<IDXGIFactory1> {
        unsafe {
            // SAFETY: CreateDXGIFactory1 initializes and returns a COM factory object through the
            // typed windows crate wrapper. No raw pointers are retained by this code.
            CreateDXGIFactory1()
        }
    }

    fn wide_name(value: &[u16]) -> String {
        let len = value
            .iter()
            .position(|char| *char == 0)
            .unwrap_or(value.len());
        String::from_utf16_lossy(&value[..len]).trim().to_string()
    }

    fn vendor_from_id(vendor_id: u32) -> GpuVendor {
        match vendor_id {
            0x1002 => GpuVendor::Amd,
            0x8086 => GpuVendor::Intel,
            0x10de => GpuVendor::Nvidia,
            0x1414 => GpuVendor::Microsoft,
            _ => GpuVendor::Unknown,
        }
    }
}
