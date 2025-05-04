use actix_web::{web, HttpResponse};
use log::{error, info};

use crate::{
    utils::error::AppError,
    domain::models::auth_pronto::{UserLoginPronto, LoginProntoResponse},
    domain::repositories::auth_pronto::AuthProntoRepository,
    adapters::token::TokenGeneratorPort,
    adapters::password::pronto_password::{verify_pronto_password, has_doctor_profile},
    utils::config_env::Config,
    utils::response::ApiResponse,
};

pub struct AuthProntoService {
    repo: Box<dyn AuthProntoRepository>,
    config: web::Data<Config>,
    token_generator: Box<dyn TokenGeneratorPort>,
}

impl AuthProntoService {
    pub fn new(
        repo: Box<dyn AuthProntoRepository>,
        config: web::Data<Config>,
        token_generator: Box<dyn TokenGeneratorPort>,
    ) -> Self {
        Self { 
            repo, 
            config, 
            token_generator,
        }
    }

    pub async fn login_pronto(&self, credentials: UserLoginPronto) -> Result<HttpResponse, AppError> {
        let user = match self.repo.get_user_pronto_by_username_with_fullname(&credentials.username).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Err(AppError::BadRequest("User not found".into()));
            },
            Err(e) => {
                error!("Error fetching user from Pronto: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
        if !verify_pronto_password(&credentials.password, &user.password_pronto) {
            return Err(AppError::Unauthorized("Incorrect password".into()));
        }


        // Busca os perfis do usuário
        let profiles = match self.repo.get_user_profiles_by_login_and_unit_id(&user.login_id, 2).await {
            Ok(profiles) if !profiles.is_empty() => profiles,
            Ok(_) => {
                println!("No profiles found for user: {:?}", user);
                return Err(AppError::BadRequest("Profile not found".into()));
            },
            Err(e) => {
                error!("Error fetching profiles: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        // Verifica se tem perfil de médico
        if !has_doctor_profile(&profiles) {
            return Err(AppError::Forbidden("User does not have the required profile".into()));
        }

        // Gera o token JWT
        let token = self.token_generator
            .generate_token(
                user.userid.to_string(),
                user.fullname.clone(),
                String::from(""),  // Email vazio pois não está no banco Pronto
                String::from("Usuario Comum"),
                vec![String::from("xpredict")],
                vec![2],
                &self.config.jwt_secret,
            )
            .map_err(|e| {
                error!("Error generating token: {:?}", e);
                AppError::InternalServerError
            })?;

        info!("User logged in successfully by Pronto");
        
        let response = LoginProntoResponse {
            token,
            user_id: user.userid.to_string(),
            full_name: user.fullname,
            profile: String::from("Usuario Comum"),
            allowed_applications: vec![String::from("xpredict")],
        };

        Ok(ApiResponse::success(response).into_response())
    }
}