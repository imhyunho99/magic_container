use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub vram_total: u64, // bytes
    pub vram_used: u64,  // bytes
    pub driver_version: Option<String>,
    pub cuda_version: Option<String>,
}

#[derive(Serialize)]
pub struct SystemSpecs {
    pub os_name: String,
    pub os_version: String,
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub total_memory: u64,
    pub used_memory: u64,
    pub gpus: Vec<GpuInfo>,
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::get_specs;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::get_specs;

// Fallback for other OS (Linux, etc.) - optional
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn get_specs() -> SystemSpecs {
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_all();
    SystemSpecs {
        os_name: System::name().unwrap_or("Unknown".into()),
        os_version: System::os_version().unwrap_or("Unknown".into()),
        cpu_model: "Unsupported OS".into(),
        cpu_cores: 0,
        total_memory: 0,
        used_memory: 0,
        gpus: vec![],
    }
}
