use actix_web::{web, HttpResponse};
use crate::application::visualization_data_service::VisualizationDataService;
use crate::AppError;



pub async fn number_of_appointments_per_month(
    service: web::Data<VisualizationDataService>,
    path: web::Path<(String, String)>, // (user_id, unidade_id)
) -> Result<HttpResponse, AppError> {
    let (user_id, unidade_id) = path.into_inner();
    
    let unidade_id: i32 = unidade_id.parse()
        .map_err(|_| AppError::BadRequest("Invalid unit ID format".to_string()))?;
    
    service.number_of_appointments_per_month(user_id, unidade_id).await
}

pub async fn number_of_appointments_per_year(
    service: web::Data<VisualizationDataService>,
    path: web::Path<(String, String, String)>, // (user_id, unidade_id, year)
) -> Result<HttpResponse, AppError> {
    let (user_id, unidade_id, year) = path.into_inner();
    
    let unidade_id: i32 = unidade_id.parse()
        .map_err(|_| AppError::BadRequest("Invalid unit ID format".to_string()))?;
    
    service.number_of_appointments_per_year(user_id, unidade_id, year).await
}

pub async fn years_available_for_number_of_appointments_per_month(
    service: web::Data<VisualizationDataService>,
    path: web::Path<(String, String)>, // (user_id, unidade_id)
) -> Result<HttpResponse, AppError> {
    let (user_id, unidade_id) = path.into_inner();
    
    let unidade_id: i32 = unidade_id.parse()
        .map_err(|_| AppError::BadRequest("Invalid unit ID format".to_string()))?;
    
    service.years_available_for_number_of_appointments_per_month(user_id, unidade_id).await
}

pub async fn number_of_appointments_per_flow(
    service: web::Data<VisualizationDataService>,
    path: web::Path<(String, String)>, // (user_id, unidade_id)
) -> Result<HttpResponse, AppError> {
    let (user_id, unidade_id) = path.into_inner();
    
    let unidade_id: i32 = unidade_id.parse()
        .map_err(|_| AppError::BadRequest("Invalid unit ID format".to_string()))?;
    
    service.number_of_appointments_per_flow(user_id, unidade_id).await
}

pub async fn distribuition_of_patients_ages(
    service: web::Data<VisualizationDataService>,
    path: web::Path<(String, String)>, // (user_id, unidade_id)
) -> Result<HttpResponse, AppError> {
    let (user_id, unidade_id) = path.into_inner();
    
    let unidade_id: i32 = unidade_id.parse()
        .map_err(|_| AppError::BadRequest("Invalid unit ID format".to_string()))?;
    
    service.distribuition_of_patients_ages(user_id, unidade_id).await
}

pub async fn number_of_calls_per_day_of_the_week(
    service: web::Data<VisualizationDataService>,
    path: web::Path<(String, String)>, // (user_id, unidade_id)
) -> Result<HttpResponse, AppError> {
    let (user_id, unidade_id) = path.into_inner();
    
    let unidade_id: i32 = unidade_id.parse()
        .map_err(|_| AppError::BadRequest("Invalid unit ID format".to_string()))?;
    
    service.number_of_calls_per_day_of_the_week(user_id, unidade_id).await
}

pub async fn distribution_of_services_by_hour_group(
    service: web::Data<VisualizationDataService>,
    path: web::Path<(String, String)>, // (user_id, unidade_id)
) -> Result<HttpResponse, AppError> {
    let (user_id, unidade_id) = path.into_inner();
    
    let unidade_id: i32 = unidade_id.parse()
        .map_err(|_| AppError::BadRequest("Invalid unit ID format".to_string()))?;
    
    service.distribution_of_services_by_hour_group(user_id, unidade_id).await
}

pub async fn number_of_visits_per_nurse(
    service: web::Data<VisualizationDataService>,
    path: web::Path<(String, String)>, // (user_id, unidade_id)
) -> Result<HttpResponse, AppError> {
    let (user_id, unidade_id) = path.into_inner();
    
    let unidade_id: i32 = unidade_id.parse()
        .map_err(|_| AppError::BadRequest("Invalid unit ID format".to_string()))?;
    
    service.number_of_visits_per_nurse(user_id, unidade_id).await
}

pub async fn number_of_visits_per_doctor(
    service: web::Data<VisualizationDataService>,
    path: web::Path<(String, String)>, // (user_id, unidade_id)
) -> Result<HttpResponse, AppError> {
    let (user_id, unidade_id) = path.into_inner();
    
    let unidade_id: i32 = unidade_id.parse()
        .map_err(|_| AppError::BadRequest("Invalid unit ID format".to_string()))?;
    
    service.number_of_visits_per_doctor(user_id, unidade_id).await
}

pub async fn average_time_in_minutes_per_doctor(
    service: web::Data<VisualizationDataService>,
    path: web::Path<(String, String)>, // (user_id, unidade_id)
) -> Result<HttpResponse, AppError> {
    let (user_id, unidade_id) = path.into_inner();
    
    let unidade_id: i32 = unidade_id.parse()
        .map_err(|_| AppError::BadRequest("Invalid unit ID format".to_string()))?;
    
    service.average_time_in_minutes_per_doctor(user_id, unidade_id).await
}

pub async fn heat_map_with_disease_indication(
    service: web::Data<VisualizationDataService>,
    path: web::Path<(String, String)>, // (user_id, unidade_id)
) -> Result<HttpResponse, AppError> {
    let (user_id, unidade_id) = path.into_inner();
    
    let unidade_id: i32 = unidade_id.parse()
        .map_err(|_| AppError::BadRequest("Invalid unit ID format".to_string()))?;
    
    service.heat_map_with_disease_indication(user_id, unidade_id).await
}

pub async fn heat_map_with_the_number_of_medical_appointments_by_neighborhood(
    service: web::Data<VisualizationDataService>,
    path: web::Path<(String, String)>, // (user_id, unidade_id)
) -> Result<HttpResponse, AppError> {
    let (user_id, unidade_id) = path.into_inner();
    
    let unidade_id: i32 = unidade_id.parse()
        .map_err(|_| AppError::BadRequest("Invalid unit ID format".to_string()))?;
    
    service.heat_map_with_the_number_of_medical_appointments_by_neighborhood(user_id, unidade_id).await
}