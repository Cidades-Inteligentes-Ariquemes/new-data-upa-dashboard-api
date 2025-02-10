use serde::{Serialize};


#[derive(Debug, Serialize)]
pub struct CpuInfo {
    pub name: String,
    pub architecture: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub percent: f32,
    pub temperature: String,
}

#[derive(Debug, Serialize)]
pub struct MemoryInfo {
    pub total_gb: f64,
    pub available_gb: f64,
    pub used_gb: f64,
    pub percent: f64,
    pub free_percent: f64,
}

#[derive(Debug, Serialize)]
pub struct DiskInfo {
    pub total_gb: f64,
    pub used_gb: f64,
    pub free_gb: f64,
    pub percent: f64,
    pub free_percent: f64,
}

#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disk: DiskInfo,
}