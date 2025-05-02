use actix_web::{web, HttpResponse};
use log::debug;
use crate::application::information_service::InformationService;
use crate::utils::error::AppError;
use crate::domain::models::audit::AuditQuery;

pub async fn audits(
    path: web::Path<i32>,
    query: web::Query<AuditQuery>,
    service: web::Data<InformationService<web::Data<crate::infrastructure::repositories::audit_repository::PgAuditRepository>>>,
) -> Result<HttpResponse, AppError> {
    let page = path.into_inner();
    debug!("Acessando endpoint audits com página: {}", page);
    service.audits(
        page,
        query.email.clone(),
        query.path.clone(),
        query.date_of_request.clone()
    ).await
}

pub async fn get_available_data(
    service: web::Data<InformationService<web::Data<crate::infrastructure::repositories::audit_repository::PgAuditRepository>>>,
) -> Result<HttpResponse, AppError> {
    debug!("Acessando endpoint get_available_data");
    service.get_available_data().await
}