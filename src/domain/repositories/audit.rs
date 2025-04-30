use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::models::audit::CreateAuditDto;

#[async_trait]
pub trait AuditRepository: Send + Sync + 'static {
    async fn add_information_audit(&self, audit_data: CreateAuditDto) -> Result<Uuid, sqlx::Error>;
}