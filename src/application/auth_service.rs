use actix_web::{web, HttpResponse};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};
use log::{error, info};
use crate::utils::error::AppError;
use crate::domain::models::auth::{LoginDto, LoginResponse, Claims};
use crate::infrastructure::repositories::user_repository::PgUserRepository;
use crate::utils::config_env::Config;
use crate::domain::repositories::user::UserRepository;

pub async fn login(
    repo: web::Data<PgUserRepository>,
    config: web::Data<Config>,
    credentials: web::Json<LoginDto>,
) -> Result<HttpResponse, AppError> {
    info!("=== LOGIN INICIADO ===");
    info!("Recebido request de login");
    info!("Credenciais recebidas: {:?}", credentials);
    info!("JWT Secret configurado: {}", if config.jwt_secret.is_empty() { "VAZIO" } else { "CONFIGURADO" });

    let user = match repo.authenticate(credentials.into_inner()).await {
        Ok(Some(user)) => {
            info!("Usuário autenticado com sucesso: {}", user.email);
            user
        }
        Ok(None) => {
            info!("Credenciais inválidas");
            return Err(AppError::Unauthorized("Invalid credentials".into()));
        }
        Err(e) => {
            error!("Erro na autenticação: {:?}", e);
            return Err(AppError::InternalServerError);
        }
    };

    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 24 * 3600; // 24 horas

    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration,
        profile: user.profile.clone(),
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    ) {
        Ok(token) => {
            info!("Token JWT gerado com sucesso");
            token
        }
        Err(e) => {
            error!("Erro ao gerar token JWT: {:?}", e);
            return Err(AppError::InternalServerError);
        }
    };

    let response = LoginResponse {
        token,
        user_id: user.id.to_string(),
        full_name: user.full_name,
        email: user.email,
        profile: user.profile,
        allowed_applications: user.allowed_applications,
    };

    info!("Login concluído com sucesso para o usuário: {}", response.email);
    Ok(HttpResponse::Ok().json(response))
}