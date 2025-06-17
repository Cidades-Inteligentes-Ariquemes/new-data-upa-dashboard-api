use actix_web::HttpResponse;
use log::{error, info};
use serde_json::json;
use std::f64;

use crate::domain::models::audit::{AuditResponse, Pagination};
use crate::domain::repositories::audit::AuditRepository;
use crate::utils::error::AppError;
use crate::utils::response::ApiResponse;

pub struct InformationService<R: AuditRepository> {
    audit_repository: R,
}

impl<R: AuditRepository> InformationService<R> {
    pub fn new(audit_repository: R) -> Self {
        Self { audit_repository }
    }

    pub async fn audits(
        &self,
        page: i32,
        email: Option<String>,
        path: Option<String>,
        date_of_request: Option<String>,
    ) -> Result<HttpResponse, AppError> {
        
        // Obter os registros de auditoria com filtros
        let (audits, total_records) = match self.audit_repository.get_audits(page, email, path, date_of_request).await {
            Ok(result) => result,
            Err(err) => {
                error!("Error retrieving audits: {}", err);
                return Err(AppError::InternalServerError);
            }
        };

        // Verificar se foram encontrados registros
        if audits.is_empty() {
            info!("No audits found.");
            return Err(AppError::NotFound("No audits found".into()));
        }

        // Calcular a paginação
        let records_per_page = 15;
        let total_pages = (total_records as f64 / records_per_page as f64).ceil() as i32;

        let pagination = Pagination {
            current_page: page,
            total_pages,
            total_records,
            records_per_page,
        };

        let response = AuditResponse {
            audits,
            pagination,
        };

        // Retornar resposta formatada usando ApiResponse
        Ok(ApiResponse::success(json!({
            "audits": response.audits,
            "pagination": response.pagination,
        })).into_response())
    }

    pub async fn get_available_data(&self) -> Result<HttpResponse, AppError> {
        // Obter dados disponíveis para filtros
        let available_data = match self.audit_repository.get_available_data().await {
            Ok(data) => data,
            Err(err) => {
                error!("Error retrieving available data: {}", err);
                return Err(AppError::InternalServerError);
            }
        };

        // Retornar resposta formatada usando ApiResponse
        Ok(ApiResponse::success(json!({
            "available_data": {
                "user_email": available_data.user_email,
                "path": available_data.path,
                "method": available_data.method,
                "date_of_request": available_data.date_of_request
            }
        })).into_response())
    }

    pub async fn get_all_audits(&self) -> Result<HttpResponse, AppError> {
        // Obter todos os registros de auditoria
        let audits = match self.audit_repository.get_all_audits().await {
            Ok(audits) => audits,
            Err(err) => {
                error!("Error retrieving all audits: {}", err);
                return Err(AppError::InternalServerError);
            }
        };

        // Verificar se foram encontrados registros
        if audits.is_empty() {
            info!("No audits found.");
            return Err(AppError::NotFound("No audits found".into()));
        }

        // Retornar resposta formatada usando ApiResponse
        Ok(ApiResponse::success(audits).into_response())
    }
}