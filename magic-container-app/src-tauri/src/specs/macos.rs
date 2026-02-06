use super::{SystemSpecs, GpuInfo};
use sysinfo::System;

pub fn get_specs() -> SystemSpecs {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    
    let cpu_model = sys.cpus().first().map(|cpu| cpu.brand().to_string()).unwrap_or_else(|| "Unknown CPU".to_string());
    let cpu_cores = System::physical_core_count().unwrap_or(0);
    
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();

    // TODO: Implement Metal API call for GPU info
    let gpus = vec![]; 

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
