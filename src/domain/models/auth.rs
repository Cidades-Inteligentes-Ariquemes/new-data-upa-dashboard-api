use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub full_name: String,
    pub email: String,
    pub profile: String,
    pub allowed_applications: Vec<String>,
    pub allowed_health_units: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub exp: usize,
    pub full_name: String,
    pub email: String,
    pub profile: String,
    pub allowed_applications: Vec<String>,
    pub allowed_health_units: Vec<i64>,
}



