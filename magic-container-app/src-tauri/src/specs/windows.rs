use super::{SystemSpecs, GpuInfo};
use sysinfo::System;
use nvml_wrapper::Nvml;

pub fn get_specs() -> SystemSpecs {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    
    let cpu_model = sys.cpus().first().map(|cpu| cpu.brand().to_string()).unwrap_or_else(|| "Unknown CPU".to_string());
    let cpu_cores = System::physical_core_count().unwrap_or(0);
    
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();

    let mut gpus = Vec::new();

    // Try to initialize NVML for NVIDIA GPUs
    if let Ok(nvml) = Nvml::init() {
        if let Ok(count) = nvml.device_count() {
            for i in 0..count {
                if let Ok(device) = nvml.device_by_index(i) {
                    let name = device.name().unwrap_or_else(|_| "Unknown NVIDIA GPU".into());
                    let memory_info = device.memory_info().ok();
                    let (vram_total, vram_used) = match memory_info {
                        Some(mem) => (mem.total, mem.used),
                        None => (0, 0),
                    };

                    // Driver/CUDA info is global in NVML, but let's put it per GPU for consistency
                    let driver_version = nvml.sys_driver_version().ok();
                    let cuda_version = nvml.sys_cuda_driver_version().ok().map(|v| format!("{}.{}", v / 1000, (v % 1000) / 10));

                    gpus.push(GpuInfo {
                        name,
                        vram_total,
                        vram_used,
                        driver_version,
                        cuda_version,
                    });
                }
            }
        }
    }

    // TODO: Add DXGI fallback for AMD/Intel GPUs

    SystemSpecs {
        os_name,
        os_version,
        cpu_model,
        cpu_cores,
        total_memory,
        used_memory,
        gpus,
    }
}
