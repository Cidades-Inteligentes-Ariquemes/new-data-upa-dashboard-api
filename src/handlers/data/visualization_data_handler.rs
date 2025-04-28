use actix_web::{web, HttpResponse};
use crate::application::visualization_data_service::VisualizationDataService;
use crate::AppError;

pub async fn number_of_appointments_per_month(
    service: web::Data<VisualizationDataService>,
) -> Result<HttpResponse, AppError> {
    service.number_of_appointments_per_month().await
}

pub async fn number_of_appointments_per_year(
    service: web::Data<VisualizationDataService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let year = path.into_inner();
    service.number_of_appointments_per_year(year).await
}

pub async fn years_available_for_number_of_appointments_per_month(
    service: web::Data<VisualizationDataService>,
) -> Result<HttpResponse, AppError> {
    service.years_available_for_number_of_appointments_per_month().await
}

pub async fn number_of_appointments_per_flow(
    service: web::Data<VisualizationDataService>,
) -> Result<HttpResponse, AppError> {
    service.number_of_appointments_per_flow().await
}

pub async fn distribuition_of_patients_ages(
    service: web::Data<VisualizationDataService>,
) -> Result<HttpResponse, AppError> {
    service.distribuition_of_patients_ages().await
}

pub async fn number_of_calls_per_day_of_the_week(
    service: web::Data<VisualizationDataService>,
) -> Result<HttpResponse, AppError> {
    service.number_of_calls_per_day_of_the_week().await
}

pub async fn distribution_of_services_by_hour_group(
    service: web::Data<VisualizationDataService>,
) -> Result<HttpResponse, AppError> {
    service.distribution_of_services_by_hour_group().await
}

pub async fn number_of_visits_per_nurse(
    service: web::Data<VisualizationDataService>,
) -> Result<HttpResponse, AppError> {
    service.number_of_visits_per_nurse().await
}

pub async fn number_of_visits_per_doctor(
    service: web::Data<VisualizationDataService>,
) -> Result<HttpResponse, AppError> {
    service.number_of_visits_per_doctor().await
}

pub async fn average_time_in_minutes_per_doctor(
    service: web::Data<VisualizationDataService>,
) -> Result<HttpResponse, AppError> {
    service.average_time_in_minutes_per_doctor().await
}

pub async fn heat_map_with_disease_indication(
    service: web::Data<VisualizationDataService>,
) -> Result<HttpResponse, AppError> {
    service.heat_map_with_disease_indication().await
}

pub async fn heat_map_with_the_number_of_medical_appointments_by_neighborhood(
    service: web::Data<VisualizationDataService>,
) -> Result<HttpResponse, AppError> {
    service.heat_map_with_the_number_of_medical_appointments_by_neighborhood().await
}