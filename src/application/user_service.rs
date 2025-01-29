use actix_web::{web, HttpResponse};
use uuid::Uuid;
use log::error;
use crate::{
    domain::{
        models::user::{CreateUserDto, UpdatePasswordByAdminDto, UpdateUserDto, AddApplicationDto, UserResponse},
        repositories::user::UserRepository,
    },
    utils::response::ApiResponse,
    AppError,
    utils::validators::{is_valid_email, validate_applications, validate_profile},
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

        // Validação de email
        if !is_valid_email(&user.email) {
            return Err(AppError::BadRequest(
                format!("Error adding user: '{}' is not a valid email", user.email)
            ));
        }

        // Verifica se o usuário já existe
        if let Some(_) = self.repo.find_by_email(&user.email).await.unwrap() {
            return Err(AppError::BadRequest(
                format!("Error adding user: email '{}' already exists", user.email)
            ));
        }

        //Verifica se o perfil é válido
        validate_profile(&user.profile)?;

        // Validação de aplicações permitidas
        validate_applications(&user.allowed_applications)?;

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
        // Verifica se o usuario existe
        if self.repo.find_by_id(id).await.unwrap().is_none() {
            return Err(AppError::BadRequest(
                format!("Error updating user: user with id '{}' not found", id)
            ));
        }

        // Validações de campos vazios
        let validations = [
            ("full_name", user.full_name.is_empty()),
            ("email", user.email.is_empty()),
            ("profile", user.profile.is_empty()),
            ("allowed_applications", user.allowed_applications.is_empty()),
            ("enabled", !user.enabled),
        ];

        for (field_name, is_none) in validations {
            if is_none {
                return Err(AppError::BadRequest(
                    format!("Error updating user: {} cannot be empty", field_name)
                ));
            }
        }

        // Validação de email
        if !is_valid_email(&user.email) {
            return Err(AppError::BadRequest(
                format!("Error adding user: '{}' is not a valid email", user.email)
            ));
        }

        //Verifica se o perfil é válido
        validate_profile(&user.profile)?;

        // Validação de aplicações permitidas
        validate_applications(&user.allowed_applications)?;

        match self.repo.update(id, user).await {
            Ok(Some(user)) => Ok(ApiResponse::updated(UserResponse::from(user)).into_response()),
            Ok(None) => Ok(ApiResponse::<UserResponse>::user_not_found().into_response()),
            Err(e) => {
                error!("Error updating user: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_password_by_admin(&self, id: Uuid, data: UpdatePasswordByAdminDto) -> Result<HttpResponse, AppError> {

        // Validações de campos vazios

        let _validations = [
            ("email", data.email.is_empty()),
            ("new_password", data.new_password.is_empty()),
        ];

        for (field_name, is_empty) in _validations {
            if is_empty {
                return Err(AppError::BadRequest(
                    format!("Error updating password: {} cannot be empty", field_name)
                ));
            }
        }
    
        // Hash da nova senha
        let new_password_hash = self.password_encryptor
            .hash_password(&data.new_password)
            .map_err(|_| AppError::InternalServerError)?;
    
        // Atualiza a senha
        match self.repo.update_password(id, new_password_hash).await {
            Ok(true) => Ok(ApiResponse::<()>::updated_password().into_response()),
            Ok(false) => Ok(ApiResponse::<()>::user_not_found().into_response()),
            Err(e) => {
                error!("Error updating password: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<HttpResponse, AppError> {

        // Verifica se o usuario existe
        if self.repo.find_by_id(id).await.unwrap().is_none() {
            return Err(AppError::BadRequest(
                format!("Error deleting user: user with id '{}' not found", id)
            ));
        }

        match self.repo.delete(id).await {
            Ok(true) => Ok(ApiResponse::<()>::deleted().into_response()),
            Ok(false) => Ok(ApiResponse::<()>::user_not_found().into_response()),
            Err(e) => {
                error!("Error deleting user: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn add_application(&self, id: Uuid, applications: AddApplicationDto) -> Result<HttpResponse, AppError> {
        // obtem o usuario
        let user = self.repo.find_by_id(id).await.unwrap();
    
        // Verifica se o usuario existe
        if user.is_none() {
            return Err(AppError::BadRequest(
                format!("Error adding application: user with id '{}' not found", id)
            ));
        }

        // Validação de aplicações permitidas
        validate_applications(&applications.applications_name)?;

         // Validação para ver se a aplicação enviada já existe
        for app in applications.applications_name.iter() {
            if user.as_ref().unwrap().allowed_applications.contains(app) {
                return Err(AppError::BadRequest(
                    format!("Error adding application: application '{}' already exists", app)
                ));
            }
        }
    
        match self.repo.add_application(id, applications).await {
            Ok(Some(user)) => Ok(ApiResponse::success(UserResponse::from(user)).into_response()),
            Ok(None) => Ok(ApiResponse::<UserResponse>::user_not_found().into_response()),
            Err(e) => {
                error!("Error adding application: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_application(&self, id: Uuid, application_name: String) -> Result<HttpResponse, AppError> {

        // obtem o usuario
        let user = self.repo.find_by_id(id).await.unwrap();

        // Verifica se o usuario existe
        if user.is_none() {
            return Err(AppError::BadRequest(
                format!("Error deleting application: user with id '{}' not found", id)
            ));
        }

        // Validação de aplicações permitidas
        validate_applications(&[application_name.clone()])?;

        // Verifica se existe mais de uma aplicação permitida
        if user.unwrap().allowed_applications.len() == 1 {
            return Err(AppError::BadRequest(
                "Error deleting application: user must have at least one allowed application".to_string()
            ));
        }

        match self.repo.delete_application(id, &application_name).await {
            Ok(true) => Ok(ApiResponse::<()>::deleted().into_response()),
            Ok(false) => Ok(ApiResponse::<()>::application_not_found().into_response()),
            Err(e) => {
                error!("Error deleting application: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
    
}

    