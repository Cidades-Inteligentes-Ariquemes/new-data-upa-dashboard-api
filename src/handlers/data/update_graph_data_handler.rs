use crate::application::update_graph_data_service::UpdateGraphDataService;
use crate::middleware::logging::request_id_from_http_request;
use crate::AppError;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn update_graph_data(
    req: HttpRequest,
    service: web::Data<UpdateGraphDataService>,
) -> Result<HttpResponse, AppError> {
    let request_id = request_id_from_http_request(&req);
    service.update_data(request_id.as_deref()).await
}
