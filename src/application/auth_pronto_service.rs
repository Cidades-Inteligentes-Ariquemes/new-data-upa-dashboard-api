use actix_web::{web, HttpResponse};
use log::{error, info};
use polars::prelude::PlIndexSet;

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

        if !verify_pronto_password(&credentials.password, &user[0].password_pronto) {
            return Err(AppError::Unauthorized("Incorrect password".into()));
        }


        let mut units_id = Vec::new();

        for unit in &user {
            units_id.push(unit.unit_id as i32);
        }

        let units_id: PlIndexSet<i32> = units_id.into_iter().collect();

        // Busca os perfis do usuário
        let profiles = match self.repo.get_user_profiles_by_login_and_unit_id(&user[0].login_id, user[0].unit_id).await {
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
                user[0].userid.to_string(),
                user[0].fullname.clone(),
                String::from(""),  // Email vazio pois não está no banco Pronto
                String::from("Usuario Comum"),
                vec![String::from("xpredict")],
                units_id.iter().map(|&id| id as i64).collect::<Vec<i64>>(),
                &self.config.jwt_secret,
            )
            .map_err(|e| {
                error!("Error generating token: {:?}", e);
                AppError::InternalServerError
            })?;

        info!("User logged in successfully by Pronto");
        
        let response = LoginProntoResponse {
            token,
            user_id: user[0].userid.to_string(),
            full_name: user[0].fullname.clone(),
            profile: String::from("Usuario Comum"),
            allowed_applications: vec![String::from("xpredict")],
            allowed_health_units: units_id.iter().map(|&id| id as i64).collect(),
        };

        Ok(ApiResponse::success(response).into_response())
    }
}