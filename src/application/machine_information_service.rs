use actix_web::{HttpResponse};
use log::error;
use serde_json::json;
use crate::AppError;
use crate::domain::models::machine_information::SystemMetrics;
use crate::utils::response::ApiResponse;

pub struct MachineInformationService;

impl MachineInformationService {
    pub async fn get_machine_information(&self) -> Result<HttpResponse, AppError> {
        let system_metrics = SystemMetrics::new();

        let ip = SystemMetrics::get_external_ip().await.unwrap_or_else(|e| {
            error!("Error getting external IP: {:?}", e);
            "Unable to obtain IP".to_string()
        });

        let uptime = SystemMetrics::calculate_uptime();

        let information = json!({
            "cpu": system_metrics.cpu,
            "memory": system_metrics.memory,
            "disk": system_metrics.disk,
            "network": {
                "ip": ip
            },
            "uptime": uptime
        });

        Ok(ApiResponse::success(information).into_response())
    }
}