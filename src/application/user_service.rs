use std::collections::HashMap;
use actix_web::{web, HttpResponse};
use uuid::Uuid;
use log::error;
use serde_json::json;
use crate::{
    domain::{
        models::user::{
            CreateUserDto, 
            UpdatePasswordByAdminDto, 
            UpdatePasswordByUserCommonDto, 
            UpdateUserDto, AddApplicationDto, 
            UserResponse,
            CreateFeedbackRespiratoryDiseasesDto,
            CreateFeedbackTuberculosisDto,
            DiseaseStats
        },
        repositories::user::UserRepository,
    },
    utils::response::ApiResponse,
    AppError,
    utils::validators::{is_valid_email, validate_applications, validate_profile, validate_respiratory_diseases, validate_feedbacks},
};
use crate::infrastructure::repositories::user_repository::PgUserRepository;
use crate::adapters::password::PasswordEncryptorPort;
use crate::domain::models::user::{FeedbackRespiratoryDiseasesResponse};
use crate::utils::validators::{ALLOWED_RESPIRATORY_DISEASES};

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

    pub async fn update_password_by_user_common(&self, id: Uuid, data: UpdatePasswordByUserCommonDto) -> Result<HttpResponse, AppError> {
        // Validações de campos vazios
        let validations = [
            ("current_password", data.current_password.is_empty()),
            ("new_password", data.new_password.is_empty()),
        ];

        for (field_name, is_empty) in validations {
            if is_empty {
                return Err(AppError::BadRequest(
                    format!("Error updating password: {} cannot be empty", field_name)
                ));
            }
        }

        // Obtem o usuario
        let user = self.repo.find_by_id(id).await.unwrap();

        // Verifica se o usuario existe
        if user.is_none() {
            return Err(AppError::BadRequest(
                format!("Error updating password: user with id '{}' not found", id)
            ));
        }

        // Verifica se a senha atual está correta
        if !self.password_encryptor.verify_password(&user.as_ref().unwrap().password, &data.current_password).map_err(|e| {
            error!("Error verifying password: {:?}", e);
            AppError::InternalServerError
        })? {
            return Err(AppError::BadRequest(
                "Error updating password: current password is incorrect".to_string()
            ));
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

    pub async fn create_feedback_respiratory_diseases(&self, feedback: CreateFeedbackRespiratoryDiseasesDto) -> Result<HttpResponse, AppError> {
        // Validações de campos vazios
        let validations = [
            ("user_name", feedback.user_name.is_empty()),
            ("feedback", feedback.feedback.is_empty()),
            ("prediction_made", feedback.prediction_made.is_empty()),
            ("correct_prediction", feedback.correct_prediction.is_empty()),
        ];

        for (field_name, is_empty) in validations {
            if is_empty {
                return Err(AppError::BadRequest(
                    format!("Error creating feedback: {} cannot be empty", field_name)
                ));
            }
        }

        // Validaçao de feedback permitido
        validate_feedbacks(&feedback.feedback)?;

        // Validação de doenças respiratórias permitidas
        validate_respiratory_diseases(&[feedback.prediction_made.clone(), feedback.correct_prediction.clone()])?;

        match self.repo.create_feedback_respiratory_diseases(feedback).await {
            Ok(feedback) => Ok(ApiResponse::created(feedback).into_response()),
            Err(e) => {
                error!("Error creating feedback: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn create_feedback_tuberculosis(&self, feedback_tuberculosis: CreateFeedbackTuberculosisDto ) -> Result<HttpResponse, AppError> {

        //Validação de campos vazios
        let validations = [
            ("user_name", feedback_tuberculosis.user_name.is_empty()),
            ("feedback", feedback_tuberculosis.feedback.is_empty()),
        ];

        for (field_name, is_empty) in validations {
            if is_empty {
                return Err(AppError::BadRequest(
                    format!("Error creating feedback: {} cannot be empty", field_name)
                ));
            }
        }

        // Validação de feedbacks permitidos
        validate_feedbacks(&feedback_tuberculosis.feedback)?;

        match self.repo.create_feedback_tuberculosis(feedback_tuberculosis).await {
            Ok(feedback_tuberculosis) => Ok(ApiResponse::created(feedback_tuberculosis).into_response()),
            Err(e) => {
                error!("Error creating feedback: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_feedbacks(&self) -> Result<HttpResponse, AppError> {
        match self.repo.find_all_feedbacks_respiratory_diseases().await {
            Ok(feedbacks) => {
                let responses: Vec<FeedbackRespiratoryDiseasesResponse> = feedbacks.into_iter()
                    .map(FeedbackRespiratoryDiseasesResponse::from)
                    .collect();

                if responses.is_empty() {
                    Ok(ApiResponse::<Vec<FeedbackRespiratoryDiseasesResponse>>::feedbacks_not_found().into_response())
                } else {
                    let feedbacks_tuberculosis = self.repo.find_all_feedbacks_tuberculosis().await.unwrap();

                    if feedbacks_tuberculosis.is_empty() {
                        Ok(ApiResponse::<Vec<FeedbackRespiratoryDiseasesResponse>>::feedbacks_not_found().into_response())
                    } else {
                        // Processa feedbacks de tuberculose
                        let total_tuberculosis = feedbacks_tuberculosis.len();
                        let total_correct_tuberculosis = feedbacks_tuberculosis
                            .iter()
                            .filter(|f| f.feedback.to_lowercase() == "sim")
                            .count();

                        // Inicializa contadores para doenças respiratórias
                        let mut stats = HashMap::new();
                        for disease in ALLOWED_RESPIRATORY_DISEASES {
                            stats.insert(disease, DiseaseStats {
                                total_quantity: 0,
                                total_quantity_correct: 0,
                            });
                        }

                        // Processa cada feedback de doenças respiratórias
                        for feedback in responses {
                            if let Some(stat) = stats.get_mut(feedback.prediction_made.as_str()) {
                                stat.total_quantity += 1;
                                if feedback.feedback.to_lowercase() == "sim" {
                                    stat.total_quantity_correct += 1;
                                }
                            }
                        }

                        // Monta a resposta final
                        let final_response = json!({
                        "feedbacks_respiratory_diseases": {
                            "normal": stats.get("normal").unwrap_or(&DiseaseStats { total_quantity: 0, total_quantity_correct: 0 }),
                            "covid-19": stats.get("covid-19").unwrap_or(&DiseaseStats { total_quantity: 0, total_quantity_correct: 0 }),
                            "pneumonia viral": stats.get("pneumonia viral").unwrap_or(&DiseaseStats { total_quantity: 0, total_quantity_correct: 0 }),
                            "pneumonia bacteriana": stats.get("pneumonia bacteriana").unwrap_or(&DiseaseStats { total_quantity: 0, total_quantity_correct: 0 }),
                        },
                        "feedbacks_tuberculosis": {
                            "total_quantity": total_tuberculosis,
                            "total_quantity_correct": total_correct_tuberculosis
                        }
                    });

                        Ok(ApiResponse::success(final_response).into_response())
                    }
                }
            },
            Err(e) => {
                error!("Error fetching feedbacks: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
    
}

    