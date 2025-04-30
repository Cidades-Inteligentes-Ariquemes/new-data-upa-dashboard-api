use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
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