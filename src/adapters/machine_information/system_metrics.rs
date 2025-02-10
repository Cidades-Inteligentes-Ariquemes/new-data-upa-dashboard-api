use sysinfo::System;
use std::time::SystemTime;
use reqwest;
use crate::domain::models::machine_information::{
    DiskInfo,
    SystemMetrics,
    MemoryInfo,
    CpuInfo,
};

impl SystemMetrics {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        SystemMetrics {
            cpu: Self::get_cpu_info(&sys),
            memory: Self::get_memory_info(&sys),
            disk: Self::get_disk_info(&sys),
        }
    }

    fn bytes_to_gb(bytes: u64) -> f64 {
        (bytes as f64) / (1024.0 * 1024.0 * 1024.0)
    }

    // Função auxiliar para arredondar valores para duas casas decimais
    fn round_two(value: f64) -> f64 {
        (value * 100.0).round() / 100.0
    }

    fn get_cpu_info(sys: &System) -> CpuInfo {
        let cpu = sys.cpus().first().unwrap();

        CpuInfo {
            name: cpu.brand().to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            physical_cores: num_cpus::get_physical(),
            logical_cores: num_cpus::get(),
            percent: Self::round_two(cpu.cpu_usage() as f64) as f32,
            temperature: Self::get_cpu_temperature(),
        }
    }

    fn get_cpu_temperature() -> String {
        use sysinfo::Components;
        let components = Components::new_with_refreshed_list();
        for component in components.iter() {
            if component.label().to_lowercase().contains("cpu") {
                if let Some(temp) = component.temperature() {
                    return format!("{:.2}", temp);
                }
            }
        }
        "Not available".to_string()
    }

    fn get_memory_info(sys: &System) -> MemoryInfo {
        let total = sys.total_memory();
        let available = sys.available_memory();
        let used = total - available;

        MemoryInfo {
            total_gb: Self::round_two(Self::bytes_to_gb(total)),
            available_gb: Self::round_two(Self::bytes_to_gb(available)),
            used_gb: Self::round_two(Self::bytes_to_gb(used)),
            percent: Self::round_two((used as f64 / total as f64) * 100.0),
            free_percent: Self::round_two((available as f64 / total as f64) * 100.0),
        }
    }

    fn get_disk_info(_sys: &System) -> DiskInfo {
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        let disk = disks.first().unwrap();

        let total = disk.total_space();
        let free = disk.available_space();
        let used = total - free;

        DiskInfo {
            total_gb: Self::round_two(Self::bytes_to_gb(total)),
            used_gb: Self::round_two(Self::bytes_to_gb(used)),
            free_gb: Self::round_two(Self::bytes_to_gb(free)),
            percent: Self::round_two((used as f64 / total as f64) * 100.0),
            free_percent: Self::round_two((free as f64 / total as f64) * 100.0),
        }
    }

    pub async fn get_external_ip() -> Result<String, reqwest::Error> {
        let response = reqwest::get("https://api.ipify.org?format=json").await?;
        let json: serde_json::Value = response.json().await?;
        Ok(json["ip"].as_str().unwrap_or("Unable to obtain IP").to_string())
    }

    pub fn calculate_uptime() -> u64 {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs() * 1000,
            Err(_) => 0,
        }
    }
}
