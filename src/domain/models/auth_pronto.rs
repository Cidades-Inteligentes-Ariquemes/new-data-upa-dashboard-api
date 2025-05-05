use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UserLoginPronto {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct ProfileInfo {
    pub perfil_id: i32,
    pub perfil_nome: String,
    pub login_codigo: String,
    pub usuario_nome: String,
    pub unidade_id: i32,
}


#[derive(Debug, Serialize)]
pub struct UserPronto {
    pub username: String,
    #[serde(skip_serializing)]
    pub password_pronto: String,
    pub userid: String,    
    pub login_id: String, 
    pub fullname: String,
    pub unit_id: i32,
}

#[derive(Debug, Serialize)]
pub struct LoginProntoResponse {
    pub token: String,
    pub user_id: String,
    pub full_name: String,
    pub profile: String,
    pub allowed_applications: Vec<String>,
    pub allowed_health_units: Vec<i64>,
}
