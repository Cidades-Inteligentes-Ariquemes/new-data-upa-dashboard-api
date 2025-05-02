use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime};
use log::{error, info};
use sqlx::PgPool;
use uuid::Uuid;
use actix_web::web;

use crate::domain::models::audit::{CreateAuditDto, Audit, AvailableAuditData};
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

    async fn get_audits(
        &self, 
        page: i32, 
        email: Option<String>, 
        path: Option<String>, 
        date_of_request: Option<String>
    ) -> Result<(Vec<Audit>, i64), sqlx::Error> {
        let limit = 15;
        let offset = (page - 1) * limit;
        
        // Conversão condicional da data
        let date_filter = if let Some(date_str) = &date_of_request {
            match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(date) => Some(date),
                Err(_) => return Err(sqlx::Error::ColumnDecode {
                    index: "".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid date format"
                    )),
                }),
            }
        } else {
            None
        };
        
        // Construção das consultas baseadas nos parâmetros
        let (audits, total_records) = match (email.as_ref(), path.as_ref(), date_filter) {
            // Sem filtros
            (None, None, None) => {
                let audits = sqlx::query_as!(Audit,
                    r#"
                    SELECT id, user_email, user_profile, method, path, ip, date_of_request, hour_of_request
                    FROM audit
                    ORDER BY date_of_request DESC, hour_of_request DESC
                    LIMIT $1 OFFSET $2
                    "#,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;
                
                let total: i64 = sqlx::query_scalar!(
                    r#"SELECT COUNT(*) as "count!: i64" FROM audit"#
                )
                .fetch_one(&self.pool)
                .await?;
                
                (audits, total)
            },
            
            // Apenas email
            (Some(email_val), None, None) => {
                let audits = sqlx::query_as!(Audit,
                    r#"
                    SELECT id, user_email, user_profile, method, path, ip, date_of_request, hour_of_request
                    FROM audit
                    WHERE user_email = $1
                    ORDER BY date_of_request DESC, hour_of_request DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    email_val,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;
                
                let total: i64 = sqlx::query_scalar!(
                    r#"SELECT COUNT(*) as "count!: i64" FROM audit WHERE user_email = $1"#,
                    email_val
                )
                .fetch_one(&self.pool)
                .await?;
                
                (audits, total)
            },
            
            // Apenas path
            (None, Some(path_val), None) => {
                let audits = sqlx::query_as!(Audit,
                    r#"
                    SELECT id, user_email, user_profile, method, path, ip, date_of_request, hour_of_request
                    FROM audit
                    WHERE path = $1
                    ORDER BY date_of_request DESC, hour_of_request DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    path_val,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;
                
                let total: i64 = sqlx::query_scalar!(
                    r#"SELECT COUNT(*) as "count!: i64" FROM audit WHERE path = $1"#,
                    path_val
                )
                .fetch_one(&self.pool)
                .await?;
                
                (audits, total)
            },
            
            // Apenas data
            (None, None, Some(date_val)) => {
                let audits = sqlx::query_as!(Audit,
                    r#"
                    SELECT id, user_email, user_profile, method, path, ip, date_of_request, hour_of_request
                    FROM audit
                    WHERE date_of_request = $1
                    ORDER BY date_of_request DESC, hour_of_request DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    date_val,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;
                
                let total: i64 = sqlx::query_scalar!(
                    r#"SELECT COUNT(*) as "count!: i64" FROM audit WHERE date_of_request = $1"#,
                    date_val
                )
                .fetch_one(&self.pool)
                .await?;
                
                (audits, total)
            },
            
            // Email e path
            (Some(email_val), Some(path_val), None) => {
                let audits = sqlx::query_as!(Audit,
                    r#"
                    SELECT id, user_email, user_profile, method, path, ip, date_of_request, hour_of_request
                    FROM audit
                    WHERE user_email = $1 AND path = $2
                    ORDER BY date_of_request DESC, hour_of_request DESC
                    LIMIT $3 OFFSET $4
                    "#,
                    email_val,
                    path_val,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;
                
                let total: i64 = sqlx::query_scalar!(
                    r#"SELECT COUNT(*) as "count!: i64" FROM audit WHERE user_email = $1 AND path = $2"#,
                    email_val,
                    path_val
                )
                .fetch_one(&self.pool)
                .await?;
                
                (audits, total)
            },
            
            // Email e data
            (Some(email_val), None, Some(date_val)) => {
                let audits = sqlx::query_as!(Audit,
                    r#"
                    SELECT id, user_email, user_profile, method, path, ip, date_of_request, hour_of_request
                    FROM audit
                    WHERE user_email = $1 AND date_of_request = $2
                    ORDER BY date_of_request DESC, hour_of_request DESC
                    LIMIT $3 OFFSET $4
                    "#,
                    email_val,
                    date_val,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;
                
                let total: i64 = sqlx::query_scalar!(
                    r#"SELECT COUNT(*) as "count!: i64" FROM audit WHERE user_email = $1 AND date_of_request = $2"#,
                    email_val,
                    date_val
                )
                .fetch_one(&self.pool)
                .await?;
                
                (audits, total)
            },
            
            // Path e data
            (None, Some(path_val), Some(date_val)) => {
                let audits = sqlx::query_as!(Audit,
                    r#"
                    SELECT id, user_email, user_profile, method, path, ip, date_of_request, hour_of_request
                    FROM audit
                    WHERE path = $1 AND date_of_request = $2
                    ORDER BY date_of_request DESC, hour_of_request DESC
                    LIMIT $3 OFFSET $4
                    "#,
                    path_val,
                    date_val,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;
                
                let total: i64 = sqlx::query_scalar!(
                    r#"SELECT COUNT(*) as "count!: i64" FROM audit WHERE path = $1 AND date_of_request = $2"#,
                    path_val,
                    date_val
                )
                .fetch_one(&self.pool)
                .await?;
                
                (audits, total)
            },
            
            // Todos os filtros
            (Some(email_val), Some(path_val), Some(date_val)) => {
                let audits = sqlx::query_as!(Audit,
                    r#"
                    SELECT id, user_email, user_profile, method, path, ip, date_of_request, hour_of_request
                    FROM audit
                    WHERE user_email = $1 AND path = $2 AND date_of_request = $3
                    ORDER BY date_of_request DESC, hour_of_request DESC
                    LIMIT $4 OFFSET $5
                    "#,
                    email_val,
                    path_val,
                    date_val,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;
                
                let total: i64 = sqlx::query_scalar!(
                    r#"SELECT COUNT(*) as "count!: i64" FROM audit 
                    WHERE user_email = $1 AND path = $2 AND date_of_request = $3"#,
                    email_val,
                    path_val,
                    date_val
                )
                .fetch_one(&self.pool)
                .await?;
                
                (audits, total)
            },
        };
        
        Ok((audits, total_records))
    }
    
    // Método get_available_data
    async fn get_available_data(&self) -> Result<AvailableAuditData, sqlx::Error> {
        // Consulta para obter e-mails distintos
        let emails: Vec<String> = sqlx::query!(
            r#"
            SELECT DISTINCT user_email 
            FROM audit 
            WHERE user_email IS NOT NULL AND user_email != ''
            ORDER BY user_email
            "#
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| row.user_email)
        .collect();
        
        // Consulta para obter paths distintos
        let paths: Vec<String> = sqlx::query!(
            r#"
            SELECT DISTINCT path 
            FROM audit 
            WHERE path IS NOT NULL AND path != ''
            ORDER BY path
            "#
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| row.path)
        .collect();
        
        // Consulta para obter métodos distintos
        let methods: Vec<String> = sqlx::query!(
            r#"
            SELECT DISTINCT method 
            FROM audit 
            WHERE method IS NOT NULL AND method != ''
            ORDER BY method
            "#
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| row.method)
        .collect();
        
        // Consulta para obter datas distintas
        let dates: Vec<String> = sqlx::query!(
            r#"
            SELECT DISTINCT date_of_request 
            FROM audit 
            WHERE date_of_request IS NOT NULL
            ORDER BY date_of_request DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| row.date_of_request.format("%Y-%m-%d").to_string())
        .collect();
        
        Ok(AvailableAuditData {
            user_email: emails,
            path: paths,
            method: methods,
            date_of_request: dates,
        })
    }
}

#[async_trait]
impl AuditRepository for web::Data<PgAuditRepository> {
    async fn add_information_audit(&self, audit_data: CreateAuditDto) -> Result<Uuid, sqlx::Error> {
        self.get_ref().add_information_audit(audit_data).await
    }

    async fn get_audits(
        &self, 
        page: i32, 
        email: Option<String>, 
        path: Option<String>, 
        date_of_request: Option<String>
    ) -> Result<(Vec<Audit>, i64), sqlx::Error> {
        self.get_ref().get_audits(page, email, path, date_of_request).await
    }
    
    async fn get_available_data(&self) -> Result<AvailableAuditData, sqlx::Error> {
        self.get_ref().get_available_data().await
    }
}