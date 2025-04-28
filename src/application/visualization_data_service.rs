use log::{error, info};
use serde_json::json;
use std::collections::HashSet;
use actix_web::{web, HttpResponse};
use crate::domain::repositories::data_upa::DataRepository;
use crate::infrastructure::repositories::data_upa_repository::PgDataRepository;
use crate::utils::response::ApiResponse;
use crate::AppError;

pub struct VisualizationDataService {
    repo: web::Data<PgDataRepository>,
}

impl VisualizationDataService {
    pub fn new(repo: web::Data<PgDataRepository>) -> Self {
        Self { repo }
    }

    pub async fn number_of_appointments_per_month(&self) -> Result<HttpResponse, AppError> {
        info!("Fetching number of appointments per month");
        
        match self.repo.fetch_nested_json("number_of_appointments_per_month", "number_of_appointments_per_month").await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching number of appointments per month. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Number of appointments per month fetched successfully");
                Ok(ApiResponse::success(json!({
                    "message": "Number of appointments per month fetched successfully",
                    "data": corrected_data
                })).into_response())
            },
            Err(e) => {
                error!("Error fetching number of appointments per month: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn number_of_appointments_per_year(&self, year: String) -> Result<HttpResponse, AppError> {
        info!("Fetching number of appointments per year: {}", year);
        
        match self.repo.fetch_nested_json("number_of_appointments_per_month", "number_of_appointments_per_month").await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching number of appointments per year. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                // Filtrar apenas o ano especificado
                let filtered_data: serde_json::Map<String, serde_json::Value> = corrected_data
                    .into_iter()
                    .filter(|(key, _)| key.starts_with(&year))
                    .collect();
                
                // Ordenar por mês (formato: ano-mês)
                let mut sorted_data_vec: Vec<(String, serde_json::Value)> = filtered_data.into_iter().collect();
                sorted_data_vec.sort_by(|(a, _), (b, _)| {
                    let a_month = a.split('-').nth(1).unwrap_or("0").parse::<i32>().unwrap_or(0);
                    let b_month = b.split('-').nth(1).unwrap_or("0").parse::<i32>().unwrap_or(0);
                    a_month.cmp(&b_month)
                });
                
                let sorted_data: serde_json::Map<String, serde_json::Value> = sorted_data_vec.into_iter().collect();
                
                info!("Number of appointments per year fetched successfully");
                Ok(ApiResponse::success(json!({
                    "message": "Number of appointments per year fetched successfully",
                    "data": sorted_data
                })).into_response())
            },
            Err(e) => {
                error!("Error fetching number of appointments per year: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn years_available_for_number_of_appointments_per_month(&self) -> Result<HttpResponse, AppError> {
        info!("Fetching years available for number of appointments per month");
        
        match self.repo.fetch_nested_json("number_of_appointments_per_month", "number_of_appointments_per_month").await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching years available. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                // Extrair os anos únicos
                let mut years = HashSet::new();
                for key in corrected_data.keys() {
                    if let Some(year) = key.split('-').next() {
                        years.insert(year.to_string());
                    }
                }
                
                let years_vec: Vec<String> = years.into_iter().collect();
                
                info!("Years available fetched successfully");
                Ok(ApiResponse::success(json!({
                    "message": "Years available for number of appointments per month fetched successfully",
                    "years_available": years_vec
                })).into_response())
            },
            Err(e) => {
                error!("Error fetching years available: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn number_of_appointments_per_flow(&self) -> Result<HttpResponse, AppError> {
        info!("Fetching number of appointments per flow");
        
        match self.repo.fetch_nested_json("number_of_appointments_per_flow", "number_of_appointments_per_flow").await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching number of appointments per flow. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Number of appointments per flow fetched successfully");
                Ok(ApiResponse::success(json!({
                    "message": "Number of appointments per flow fetched successfully",
                    "data": corrected_data
                })).into_response())
            },
            Err(e) => {
                error!("Error fetching number of appointments per flow: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn distribuition_of_patients_ages(&self) -> Result<HttpResponse, AppError> {
        info!("Fetching distribuition of patients ages");
        
        match self.repo.fetch_nested_json("distribuition_of_patients_ages", "distribuition_of_patients_ages").await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching distribuition of patients ages. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Distribuition of patients ages fetched successfully");
                Ok(ApiResponse::success(json!({
                    "message": "Distribuition of patients ages fetched successfully",
                    "data": corrected_data
                })).into_response())
            },
            Err(e) => {
                error!("Error fetching distribuition of patients ages: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn number_of_calls_per_day_of_the_week(&self) -> Result<HttpResponse, AppError> {
        info!("Fetching number of calls per day of the week");
        
        match self.repo.fetch_nested_json("number_of_calls_per_day_of_the_week", "number_of_calls_per_day_of_the_week").await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching number of calls per day of the week. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Number of calls per day of the week fetched successfully");
                Ok(ApiResponse::success(json!({
                    "message": "Number of calls per day of the week fetched successfully",
                    "data": corrected_data
                })).into_response())
            },
            Err(e) => {
                error!("Error fetching number of calls per day of the week: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn distribution_of_services_by_hour_group(&self) -> Result<HttpResponse, AppError> {
        info!("Fetching distribution of services by hour group");
        
        match self.repo.fetch_nested_json("distribution_of_services_by_hour_group", "distribution_of_services_by_hour_group").await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching distribution of services by hour group. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Distribution of services by hour group fetched successfully");
                Ok(ApiResponse::success(json!({
                    "message": "Distribution of services by hour group fetched successfully",
                    "data": corrected_data
                })).into_response())
            },
            Err(e) => {
                error!("Error fetching distribution of services by hour group: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn number_of_visits_per_nurse(&self) -> Result<HttpResponse, AppError> {
        info!("Fetching number of visits per nurse");
        
        match self.repo.fetch_nested_json("number_of_visits_per_nurse", "number_of_visits_per_nurse").await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching number of visits per nurse. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Number of visits per nurse fetched successfully");
                Ok(ApiResponse::success(json!({
                    "message": "Number of visits per nurse fetched successfully",
                    "data": corrected_data
                })).into_response())
            },
            Err(e) => {
                error!("Error fetching number of visits per nurse: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn number_of_visits_per_doctor(&self) -> Result<HttpResponse, AppError> {
        info!("Fetching number of visits per doctor");
        
        match self.repo.fetch_nested_json("number_of_visits_per_doctor", "number_of_visits_per_doctor").await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching number of visits per doctor. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Number of visits per doctor fetched successfully");
                Ok(ApiResponse::success(json!({
                    "message": "Number of visits per doctor fetched successfully",
                    "data": corrected_data
                })).into_response())
            },
            Err(e) => {
                error!("Error fetching number of visits per doctor: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn average_time_in_minutes_per_doctor(&self) -> Result<HttpResponse, AppError> {
        info!("Fetching average time in minutes per doctor");
        
        match self.repo.fetch_nested_json("average_time_per_doctor", "average_time_per_doctor").await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching average time in minutes per doctor. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Average time in minutes per doctor fetched successfully");
                Ok(ApiResponse::success(json!({
                    "message": "Average time in minutes per doctor fetched successfully",
                    "data": corrected_data
                })).into_response())
            },
            Err(e) => {
                error!("Error fetching average time in minutes per doctor: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn heat_map_with_disease_indication(&self) -> Result<HttpResponse, AppError> {
        info!("Fetching heat map with disease indication");
        
        match self.repo.fetch_nested_json("heat_map_with_disease_indication", "heat_map_with_disease_indication").await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching heat map with disease indication. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Heat map with disease indication fetched successfully");
                Ok(ApiResponse::success(json!({
                    "message": "Heat map with disease indication fetched successfully",
                    "data": corrected_data
                })).into_response())
            },
            Err(e) => {
                error!("Error fetching heat map with disease indication: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn heat_map_with_the_number_of_medical_appointments_by_neighborhood(&self) -> Result<HttpResponse, AppError> {
        info!("Fetching heat map with the number of medical appointments by neighborhood");
        
        match self.repo.fetch_nested_json("heat_map_with_the_number_of_medical_appointments_by_neighborhood", "heat_map_with_the_number_of_medical_appointments_by_neighborhood").await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching heat map with appointments by neighborhood. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Heat map with appointments by neighborhood fetched successfully");
                Ok(ApiResponse::success(json!({
                    "message": "Heat map with the number of medical appointments by neighborhood fetched successfully",
                    "data": corrected_data
                })).into_response())
            },
            Err(e) => {
                error!("Error fetching heat map with appointments by neighborhood: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    // Função auxiliar para corrigir as chaves JSON
    fn correct_keys(&self, data: serde_json::Map<String, serde_json::Value>) -> serde_json::Map<String, serde_json::Value> {
        let mut corrected_data = serde_json::Map::new();
        
        for (key, value) in data {
            let corrected_key = key
                .replace("(", "")
                .replace(")", "")
                .replace("'", "")
                .replace(",", "");
            
            corrected_data.insert(corrected_key, value);
        }
        
        corrected_data
    }
}