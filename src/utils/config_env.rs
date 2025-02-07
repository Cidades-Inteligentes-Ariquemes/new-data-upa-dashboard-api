use std::env;

#[derive(Clone)]  // Adiciona esta linha
pub struct Config {
    pub database_url: String,
    pub server_addr: String,
    pub jwt_secret: String,
    pub api_key: String,
    pub email: String,
    pub email_password: String,
    pub app_name: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            server_addr: env::var("SERVER_ADDR").expect("SERVER_ADDR must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            api_key: env::var("API_KEY").expect("API_KEY must be set"),
            email: env::var("EMAIL").expect("EMAIL must be set"),
            email_password: env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD must be set"),
            app_name: env::var("APP_NAME").expect("APP_NAME must be set"),
        }
    }
}