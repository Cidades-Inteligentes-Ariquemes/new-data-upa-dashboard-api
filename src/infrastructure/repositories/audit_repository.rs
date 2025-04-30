// src/infrastructure/repositories/audit_repository.rs
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime};
use log::{error, info};
use sqlx::PgPool;
use uuid::Uuid;
use actix_web::web;

use crate::domain::models::audit::CreateAuditDto;
use crate::domain::repositories::audit::AuditRepository;

#[derive(Clone)]
pub struct PgAuditRepository {
    pool: PgPool,
}

impl PgAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for PgAuditRepository {
    async fn add_information_audit(&self, audit_data: CreateAuditDto) -> Result<Uuid, sqlx::Error> {
        // Converter as strings de data/hora para objetos NaiveDate/NaiveTime
        let date_of_request = NaiveDate::parse_from_str(&audit_data.date_of_request, "%Y-%m-%d")
            .map_err(|e| {
                error!("Error parsing date: {:?}", e);
                sqlx::Error::ColumnDecode {
                    index: "".to_string(),
                    source: Box::new(e),
                }
            })?;

        let hour_of_request = NaiveTime::parse_from_str(&audit_data.hour_of_request, "%H:%M:%S")
            .map_err(|e| {
                error!("Error parsing time: {:?}", e);
                sqlx::Error::ColumnDecode {
                    index: "".to_string(),
                    source: Box::new(e),
                }
            })?;

        let result = sqlx::query!(
            r#"
            INSERT INTO audit (id, user_email, user_profile, method, path, ip, date_of_request, hour_of_request)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
            audit_data.id,
            audit_data.user_email,
            audit_data.user_profile,
            audit_data.method,
            audit_data.path,
            audit_data.ip,
            date_of_request,
            hour_of_request
        )
        .fetch_one(&self.pool)
        .await?;

        info!("Log added with id: {}", result.id);
        Ok(result.id)
    }
}

#[async_trait]
impl AuditRepository for web::Data<PgAuditRepository> {
    async fn add_information_audit(&self, audit_data: CreateAuditDto) -> Result<Uuid, sqlx::Error> {
        self.get_ref().add_information_audit(audit_data).await
    }
}