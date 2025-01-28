use actix_web::{web, HttpResponse};
use uuid::Uuid;
use log::error;
use crate::{
    domain::{
        models::user::{CreateUserDto, UpdatePasswordDto, UpdateUserDto, UserResponse},
        repositories::user::UserRepository,
    },
    utils::response::ApiResponse,
    AppError,
};
use crate::infrastructure::repositories::user_repository::PgUserRepository;
use crate::adapters::password::PasswordEncryptorPort;

pub struct UserService {
    repo: web::Data<PgUserRepository>,
    password_encryptor: Box<dyn PasswordEncryptorPort>,
}

impl UserService {
    pub fn new(repo: web::Data<PgUserRepository>, password_encryptor: Box<dyn PasswordEncryptorPort>) -> Self {
        Self { repo, password_encryptor }
    }

    pub async fn get_users(&self) -> Result<HttpResponse, AppError> {
        match self.repo.find_all().await {
            Ok(users) => {
                let responses: Vec<UserResponse> = users.into_iter()
                    .map(UserResponse::from)
                    .collect();
                
                if responses.is_empty() {
                    Ok(ApiResponse::<Vec<UserResponse>>::users_not_found().into_response())
                } else {
                    Ok(ApiResponse::success(responses).into_response())
                }
            },
            Err(e) => {
                error!("Error fetching users: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<HttpResponse, AppError> {
        match self.repo.find_by_id(id).await {
            Ok(Some(user)) => Ok(ApiResponse::success(UserResponse::from(user)).into_response()),
            Ok(None) => Ok(ApiResponse::<UserResponse>::user_not_found().into_response()),
            Err(e) => {
                error!("Error fetching user: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn create_user(&self, user: CreateUserDto) -> Result<HttpResponse, AppError> {
        // Validações de campos vazios
        let validations = [
            ("full_name", user.full_name.is_empty()),
            ("email", user.email.is_empty()),
            ("password", user.password.is_empty()),
            ("profile", user.profile.is_empty()),
            ("allowed_applications", user.allowed_applications.is_empty()),
        ];

        for (field_name, is_empty) in validations {
            if is_empty {
                return Err(AppError::BadRequest(
                    format!("Error adding user: {} cannot be empty", field_name)
                ));
            }
        }

        // Verifica se o usuário já existe
        if let Some(_) = self.repo.find_by_email(&user.email).await.unwrap() {
            return Err(AppError::BadRequest(
                format!("Error adding user: email '{}' already exists", user.email)
            ));
        }

        // Validação de aplicações permitidas
        const ALLOWED_APPS: [&str; 2] = ["xpredict", "upavision"];
        for app in &user.allowed_applications {
            if !ALLOWED_APPS.contains(&app.as_str()) {
                return Err(AppError::BadRequest(
                    format!("Error adding user: '{}' is not a valid application. Allowed values are: xpredict, upavision", app)
                ));
            }
        }

        // Hash da senha
        let mut user_with_hash = user;
        user_with_hash.password = self.password_encryptor
            .hash_password(&user_with_hash.password)
            .map_err(|e| {
                error!("Error hashing password: {:?}", e);
                AppError::InternalServerError
            })?;

        match self.repo.create(user_with_hash).await {
            Ok(user) => Ok(ApiResponse::created(UserResponse::from(user)).into_response()),
            Err(e) => {
                error!("Error creating user: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_user(&self, id: Uuid, user: UpdateUserDto) -> Result<HttpResponse, AppError> {
        match self.repo.update(id, user).await {
            Ok(Some(user)) => Ok(ApiResponse::updated(UserResponse::from(user)).into_response()),
            Ok(None) => Ok(ApiResponse::<UserResponse>::user_not_found().into_response()),
            Err(e) => {
                error!("Error updating user: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_password(&self, id: Uuid, passwords: UpdatePasswordDto) -> Result<HttpResponse, AppError> {
        // Verifica a senha atual e faz o hash da nova senha
        let current_user = match self.repo.find_by_id(id).await {
            Ok(Some(user)) => user,
            Ok(None) => return Ok(ApiResponse::<()>::user_not_found().into_response()),
            Err(e) => {
                error!("Error fetching user: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
    
        // Verifica senha atual
        if !self.password_encryptor.verify_password(&current_user.password, &passwords.current_password)
            .map_err(|_| AppError::InternalServerError)? {
            return Err(AppError::BadRequest("Current password is incorrect".into()));
        }
    
        // Hash da nova senha
        let new_password_hash = self.password_encryptor
            .hash_password(&passwords.new_password)
            .map_err(|_| AppError::InternalServerError)?;
    
        // Atualiza a senha
        match self.repo.update_password(id, new_password_hash).await {
            Ok(true) => Ok(ApiResponse::success("Password updated successfully").into_response()),
            Ok(false) => Ok(ApiResponse::<()>::user_not_found().into_response()),
            Err(e) => {
                error!("Error updating password: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<HttpResponse, AppError> {
        match self.repo.delete(id).await {
            Ok(true) => Ok(ApiResponse::<()>::deleted().into_response()),
            Ok(false) => Ok(ApiResponse::<()>::user_not_found().into_response()),
            Err(e) => {
                error!("Error deleting user: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}