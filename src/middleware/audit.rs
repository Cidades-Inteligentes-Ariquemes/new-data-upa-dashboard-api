// src/middleware/audit.rs
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use chrono::Utc;
use futures::future::{ready, Ready};
use futures::Future;
use log::error;
use std::pin::Pin;
use uuid::Uuid;

use crate::domain::models::audit::CreateAuditDto;
use crate::domain::models::auth::Claims;
use crate::domain::repositories::audit::AuditRepository;
use crate::utils::validators::is_public_route;

pub struct AuditMiddleware<R: AuditRepository> {
    repository: R,
}

impl<R: AuditRepository> AuditMiddleware<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<S, B, R> Transform<S, ServiceRequest> for AuditMiddleware<R>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
    R: AuditRepository + Clone + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuditMiddlewareService<S, R>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuditMiddlewareService {
            service,
            repository: self.repository.clone(),
        }))
    }
}

pub struct AuditMiddlewareService<S, R>
where
    R: AuditRepository,
{
    service: S,
    repository: R,
}

impl<S, B, R> Service<ServiceRequest> for AuditMiddlewareService<S, R>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
    R: AuditRepository + Clone + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().to_string();
        let path = req.path().to_string();
        let ip = req
            .connection_info()
            .peer_addr()
            .unwrap_or("unknown")
            .to_string();

        // Capturar informações do usuário do token, se disponível
        let mut user_email = "unknown".to_string();
        let mut user_profile = "unknown".to_string();

        // Tentar obter as claims do usuário, se estiver autenticado
        if !is_public_route(&path) {
            if let Some(claims) = req.extensions().get::<Claims>() {
                user_email = if !claims.email.is_empty() { 
                    claims.email.clone() 
                } else { 
                    claims.full_name.clone() 
                };
                user_profile = claims.profile.clone();
            }
        }

        // Formatar data e hora
        let now = Utc::now();
        let date_of_request = now.format("%Y-%m-%d").to_string();
        let hour_of_request = now.format("%H:%M:%S").to_string();

        // Criar o DTO de auditoria
        let audit_data = CreateAuditDto {
            id: Uuid::new_v4(),
            user_email,
            user_profile,
            method,
            path,
            ip,
            date_of_request,
            hour_of_request,
        };

        // Clone dos dados para usar na future
        let audit_data_clone = audit_data;
        let fut = self.service.call(req);
        let repository_clone = self.repository.clone();

        Box::pin(async move {
            // Registrar a auditoria em paralelo com o processamento da requisição
            let audit_future = repository_clone.add_information_audit(audit_data_clone);
            
            // Continuar com o fluxo normal da requisição
            let res = fut.await?;
            
            // Aguardar a conclusão do registro de auditoria, mas não bloquear a resposta
            if let Err(e) = audit_future.await {
                error!("Error adding audit information: {:?}", e);
                // Não falhar a requisição se o registro de auditoria falhar
            }
            
            Ok(res)
        })
    }
}