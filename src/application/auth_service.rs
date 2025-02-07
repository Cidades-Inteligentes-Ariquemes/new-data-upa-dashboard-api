use actix_web::web;
use actix_web::HttpResponse;
use crate::{
    utils::error::AppError,
    domain::models::auth::{LoginDto, LoginResponse},
    infrastructure::repositories::user_repository::PgUserRepository,
    utils::config_env::Config,
    domain::repositories::user::UserRepository,
    adapters::{
        password::PasswordEncryptorPort,
        token::TokenGeneratorPort,
    },
};

pub struct AuthService {
    repo: web::Data<PgUserRepository>,
    config: web::Data<Config>,
    password_encryptor: Box<dyn PasswordEncryptorPort>,
    token_generator: Box<dyn TokenGeneratorPort>,
}

impl AuthService {
    pub fn new(
        repo: web::Data<PgUserRepository>,
        config: web::Data<Config>,
        password_encryptor: Box<dyn PasswordEncryptorPort>,
        token_generator: Box<dyn TokenGeneratorPort>,
    ) -> Self {
        Self { 
            repo, 
            config, 
            password_encryptor,
            token_generator,
        }
    }

    pub async fn login(&self, credentials: LoginDto) -> Result<HttpResponse, AppError> {

        // Busca o usuário
        let user = match self.repo.find_by_email(&credentials.email).await {
            Ok(Some(user)) => {
                if !user.enabled {
                    return Err(AppError::Unauthorized("User is disabled".into()));
                }
                user
            },
            Ok(None) => {
                return Err(AppError::Unauthorized("Invalid credentials".into()));
            },
            Err(_) => {
                return Err(AppError::InternalServerError);
            }
        };

        // Verifica a senha
        if !self.password_encryptor.verify_password(&user.password, &credentials.password)
            .map_err(|_| {
                AppError::InternalServerError
            })? {
            return Err(AppError::Unauthorized("Invalid credentials".into()));
        }

        // Gera o token JWT
        let token = self.token_generator
            .generate_token(
                user.id.to_string(),
                user.full_name.clone(),
                user.email.to_string(),
                user.profile.clone(),
                user.allowed_applications.clone(),
                &self.config.jwt_secret,
            )
            .map_err(|_| {
                AppError::InternalServerError
            })?;

        let response = LoginResponse {
            token,
            user_id: user.id.to_string(),
            full_name: user.full_name,
            email: user.email,
            profile: user.profile,
            allowed_applications: user.allowed_applications,
        };

        Ok(HttpResponse::Ok().json(response))
    }
}