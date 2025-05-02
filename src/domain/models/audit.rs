use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow; 
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Audit {
    pub id: Uuid,
    pub user_email: String,
    pub user_profile: String,
    pub method: String,
    pub path: String,
    pub ip: String,
    pub date_of_request: NaiveDate,
    pub hour_of_request: NaiveTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuditDto {
    pub id: Uuid,
    pub user_email: String,
    pub user_profile: String,
    pub method: String,
    pub path: String,
    pub ip: String,
    pub date_of_request: String,
    pub hour_of_request: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditResponse {
    pub audits: Vec<Audit>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub current_page: i32,
    pub total_pages: i32,
    pub total_records: i64,
    pub records_per_page: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableAuditData {
    pub user_email: Vec<String>,
    pub path: Vec<String>,
    pub method: Vec<String>,
    pub date_of_request: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub email: Option<String>,
    pub path: Option<String>,
    pub date_of_request: Option<String>,
}