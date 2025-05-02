use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::models::audit::{CreateAuditDto, Audit, AvailableAuditData};

#[async_trait]
pub trait AuditRepository: Send + Sync + 'static {
    async fn add_information_audit(&self, audit_data: CreateAuditDto) -> Result<Uuid, sqlx::Error>;
    async fn get_audits(
        &self, 
        page: i32, 
        email: Option<String>, 
        path: Option<String>, 
        date_of_request: Option<String>
    ) -> Result<(Vec<Audit>, i64), sqlx::Error>;
    
    async fn get_available_data(&self) -> Result<AvailableAuditData, sqlx::Error>;
}