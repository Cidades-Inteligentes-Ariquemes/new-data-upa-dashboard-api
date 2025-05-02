use actix_web::{web, HttpResponse};
use crate::application::visualization_data_service::VisualizationDataService;
use crate::AppError;
use crate::domain::models::visualization_data_graph::UnitQueryParams;



pub async fn number_of_appointments_per_month(
    service: web::Data<VisualizationDataService>,
    query: web::Query<UnitQueryParams>,
) -> Result<HttpResponse, AppError> {
    service.number_of_appointments_per_month(query.unidade_id).await
}

pub async fn number_of_appointments_per_year(
    service: web::Data<VisualizationDataService>,
    path: web::Path<String>,
    query: web::Query<UnitQueryParams>,
) -> Result<HttpResponse, AppError> {
    let year = path.into_inner();
    service.number_of_appointments_per_year(year, query.unidade_id).await
}

pub async fn years_available_for_number_of_appointments_per_month(
    service: web::Data<VisualizationDataService>,
    query: web::Query<UnitQueryParams>,
) -> Result<HttpResponse, AppError> {
    service.years_available_for_number_of_appointments_per_month(query.unidade_id).await
}

pub async fn number_of_appointments_per_flow(
    service: web::Data<VisualizationDataService>,
    query: web::Query<UnitQueryParams>,
) -> Result<HttpResponse, AppError> {
    service.number_of_appointments_per_flow(query.unidade_id).await
}

pub async fn distribuition_of_patients_ages(
    service: web::Data<VisualizationDataService>,
    query: web::Query<UnitQueryParams>,
) -> Result<HttpResponse, AppError> {
    service.distribuition_of_patients_ages(query.unidade_id).await
}

pub async fn number_of_calls_per_day_of_the_week(
    service: web::Data<VisualizationDataService>,
    query: web::Query<UnitQueryParams>,
) -> Result<HttpResponse, AppError> {
    service.number_of_calls_per_day_of_the_week(query.unidade_id).await
}

pub async fn distribution_of_services_by_hour_group(
    service: web::Data<VisualizationDataService>,
    query: web::Query<UnitQueryParams>,
) -> Result<HttpResponse, AppError> {
    service.distribution_of_services_by_hour_group(query.unidade_id).await
}

pub async fn number_of_visits_per_nurse(
    service: web::Data<VisualizationDataService>,
    query: web::Query<UnitQueryParams>,
) -> Result<HttpResponse, AppError> {
    service.number_of_visits_per_nurse(query.unidade_id).await
}

pub async fn number_of_visits_per_doctor(
    service: web::Data<VisualizationDataService>,
    query: web::Query<UnitQueryParams>,
) -> Result<HttpResponse, AppError> {
    service.number_of_visits_per_doctor(query.unidade_id).await
}

pub async fn average_time_in_minutes_per_doctor(
    service: web::Data<VisualizationDataService>,
    query: web::Query<UnitQueryParams>,
) -> Result<HttpResponse, AppError> {
    service.average_time_in_minutes_per_doctor(query.unidade_id).await
}

pub async fn heat_map_with_disease_indication(
    service: web::Data<VisualizationDataService>,
    query: web::Query<UnitQueryParams>,
) -> Result<HttpResponse, AppError> {
    // Para mapas de calor, se a unidade não for especificada ou não for a 2, forçamos para 2
    let unidade_id = match query.unidade_id {
        Some(id) if id == 2 => Some(id),
        _ => Some(2),  // Forçar para unidade 2 (UPA Ariquemes)
    };
    
    service.heat_map_with_disease_indication(unidade_id).await
}

pub async fn heat_map_with_the_number_of_medical_appointments_by_neighborhood(
    service: web::Data<VisualizationDataService>,
    query: web::Query<UnitQueryParams>,
) -> Result<HttpResponse, AppError> {
    // Para mapas de calor, se a unidade não for especificada ou não for a 2, forçamos para 2
    let unidade_id = match query.unidade_id {
        Some(id) if id == 2 => Some(id),
        _ => Some(2),  // Forçar para unidade 2 (UPA Ariquemes)
    };
    
    service.heat_map_with_the_number_of_medical_appointments_by_neighborhood(unidade_id).await
}