use actix_web::{web, HttpResponse};
use crate::application::update_graph_data_service::UpdateGraphDataService;
use crate::AppError;

pub async fn update_graph_data(
    service: web::Data<UpdateGraphDataService>,
) -> Result<HttpResponse, AppError> {
    service.update_data().await
}