use std::collections::HashMap;
use actix_web::{web, HttpResponse};
use log::{error, info};
use serde_json::json;
use uuid::Uuid;

use crate::adapters::password::PasswordEncryptorPort;

use crate::domain::{
    email::email_service::EmailService,
    models::user::{
        AddApplicationDto,
        CreateFeedbackRespiratoryDiseasesDto,
        CreateFeedbackTuberculosisDto,
        CreateUserDto,
        DiseaseStats,
        FeedbackRespiratoryDiseasesResponse,
        UpdateEnabledUserDto,
        UpdatePasswordByAdminDto,
        UpdatePasswordByUserCommonDto,
        UpdateUserDto,
        UpdateVerificationCodeDto,
        AddVerificationCodeDto,
        ConfirmVerificationCodeDto,
        UpdatePasswordForgettingUserDto,
        UserResponse,
    },
    repositories::user::UserRepository,
};

use crate::infrastructure::{
    email::email_service::SmtpEmailService,
    repositories::user_repository::PgUserRepository,
};

use crate::utils::{
    config_env::Config,
    response::ApiResponse,
    validators::{
        is_valid_email,
        validate_applications,
        validate_feedbacks,
        validate_profile,
        validate_respiratory_diseases,
        ALLOWED_RESPIRATORY_DISEASES,
    },
};

use crate::AppError;
use crate::domain::models::user::IdVerificationDto;

pub struct UserService {
    repo: web::Data<PgUserRepository>,
    password_encryptor: Box<dyn PasswordEncryptorPort>,
    config: web::Data<Config>
}

impl UserService {
    pub fn new(repo: web::Data<PgUserRepository>, password_encryptor: Box<dyn PasswordEncryptorPort>, config: web::Data<Config>) -> Self {
        Self { repo, password_encryptor, config }
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
            ("allowed_applications", user.allowed_applications.is_empty())
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

    pub async fn update_enabled(&self, id: Uuid, enabled: UpdateEnabledUserDto) -> Result<HttpResponse, AppError> {

        // Verifica se o usuario existe
        if self.repo.find_by_id(id).await.unwrap().is_none() {
            return Err(AppError::BadRequest(
                format!("Error deleting user: user with id '{}' not found", id)
            ));
        }

        match self.repo.update_enabled(id, enabled).await {
            Ok(true) => Ok(ApiResponse::<()>::updated_enabled().into_response()),
            Ok(false) => Ok(ApiResponse::<()>::user_not_found().into_response()),
            Err(e) => {
                error!("Error updating user: {:?}", e);
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

    pub async fn send_verification_code(&self, email: String) -> Result<HttpResponse, AppError> {

        // Validação de email
        if !is_valid_email(&email) {
            return Err(AppError::BadRequest(
                format!("Error adding user: '{}' is not a valid email", email)
            ));
        }

        let user = match self.repo.find_by_email(&email).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(AppError::BadRequest("User not found".to_string())),
            Err(_) => return Err(AppError::InternalServerError)
        };

        if !user.enabled {
            return Err(AppError::BadRequest("User disabled".to_string()))
        }

        //Gera o codigo e caso tenha zeros à esquerda, retira eles
        let verification_code = format!("{:06}", rand::random::<u32>() % 1000000)
            .trim_start_matches('0')
            .to_string();

        let final_code = if verification_code.is_empty() {
            "0".to_string()
        } else {
            verification_code
        };
        let email_service = SmtpEmailService::new(self.config.clone());

        // Tenta enviar o email primeiro
        if let Err(e) = email_service.send_email(
            user.full_name.clone(),
            email,
            final_code.clone(),
        ).await {
            error!("Failed to send email: {:?}", e);
            return Err(AppError::InternalServerError);
        }

        //Criar o json para enviar para o banco as informaçoes

        let data = AddVerificationCodeDto {
            id: Uuid::new_v4(),
            user_id: user.id,
            user_email: user.email,
            code_verification: final_code.parse().unwrap(),
            used: false,
            created_at: chrono::Utc::now().naive_utc(),
            expiration_at: chrono::Utc::now().naive_utc() + chrono::Duration::minutes(10),
        };

        match self.repo.add_verification_code(data).await {
            Ok(data) => Ok(ApiResponse::created(data).into_response()),
            Err(e) => {
                error!("Error creating verification code: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }

    }

    pub async fn resend_verification_code(&self, email: String, id_verification: IdVerificationDto) -> Result<HttpResponse, AppError> {
        // Validação de email
        if !is_valid_email(&email) {
            return Err(AppError::BadRequest(
                format!("Error adding user: '{}' is not a valid email", email)
            ));
        }

        // Verifica se o usuario existe
        let user = match self.repo.find_by_email(&email).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(AppError::BadRequest("User not found".to_string())),
            Err(_) => return Err(AppError::InternalServerError)
        };

        //Verifica se o código existe no banco
        let code_exists = match self.repo.verify_code_exist(id_verification.id_verification).await {
            Ok(code) => code,
            Err(e) => {
                error!("Database error: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let now = chrono::Utc::now().naive_utc();

        // Se código expirado ou usado, gera novo
        if code_exists.used || code_exists.expiration_at < now {
            info!("Code expired, already used, or not found. Generating a new one.");
            return self.send_verification_code(email).await;
        }

        // Define qual código será usado (existente ou novo)
        let verification_code = if code_exists.expiration_at >= now && !code_exists.used {
            info!("Code valid and not used. Resending the same code.");
            code_exists.verification_code
        } else {
            info!("Generating a new verification code.");
            let new_code = format!("{:06}", rand::random::<u32>() % 1000000)
                .trim_start_matches('0')
                .to_string();

            let final_code = if new_code.is_empty() {
                "0".to_string()
            } else {
                new_code
            };

            final_code.parse().unwrap()
        };

        // Envia o email
        let email_service = SmtpEmailService::new(self.config.clone());
        if let Err(e) = email_service.send_email(
            user.full_name.clone(),
            email.clone(),
            verification_code.clone().to_string(),
        ).await {
            error!("Error sending email: {:?}", e);
            return Err(AppError::InternalServerError);
        }

        // Atualiza o código no banco
        let updated_code = UpdateVerificationCodeDto {
            verification_code,
            expiration_at: now + chrono::Duration::minutes(10),
            used: false,
        };

        match self.repo.update_code_verification(updated_code, email, id_verification.id_verification).await {
            Ok(updated) => Ok(ApiResponse::success(updated).into_response()),
            Err(e) => {
                error!("Error updating verification code: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }

    }

    pub async fn confirm_verification_code(&self, data: ConfirmVerificationCodeDto) -> Result<HttpResponse, AppError> {
        //Verifica se o código existe no banco
        let code_exists = match self.repo.verify_code_exist(data.id_verification).await {
            Ok(code_exists) => code_exists,
            Err(e) => {
                error!("Database error: {:?}", e);
                return Err(AppError::BadRequest("Id verification not found".to_string()));
            }
        };

        if code_exists.verification_code != data.verification_code {
            return Err(AppError::BadRequest("Verification code not matched".to_string()));
        }

        if code_exists.used {
            return Err(AppError::BadRequest("Code already used".to_string()));
        }

        if code_exists.expiration_at < chrono::Utc::now().naive_utc() {
            return Err(AppError::BadRequest("Code expired".to_string()));
        }

        match self.repo.update_used_verification_code(data.id_verification).await {
            Ok(updated_used_code) => Ok(ApiResponse::success(updated_used_code).into_response()),
            Err(e) => {
                error!("Error updating verification code: {:?}", e);
                return Err(AppError::BadRequest(format!("Failed to update verification code: {:?}", e)));
            }
        }
    }

    pub async fn update_password_for_forgetting_user(&self, user_id: Uuid, data: UpdatePasswordForgettingUserDto) -> Result<HttpResponse, AppError> {
        //Verifica se o usuário existe
        match self.repo.find_by_id(user_id.clone()).await {
            Ok(Some(_)) => (),
            Ok(None) => return Err(AppError::BadRequest("User not found".to_string())),
            Err(_) => return Err(AppError::InternalServerError)
        };

        //Verifica se existe codigo para o id verification enviado
        let code_exist = match self.repo.verify_code_exist(data.id_verification.clone()).await {
            Ok(code) => code,
            Err(e) => {
                error!("Database error: {:?}", e);
                return Err(AppError::BadRequest("Id verification not found".to_string()));
            }
        };

        //Verifica se o codigo já foi confirmado pelo usuário
        if !code_exist.used {
            return Err(AppError::BadRequest("Code not verified".to_string()));
        }

        // Faz o encrypt da nova senha
        let new_password_hash = self.password_encryptor
            .hash_password(&data.new_password)
            .map_err(|_| AppError::InternalServerError)?;

        match self.repo.update_password_for_forgetting_user(user_id, new_password_hash.clone()).await {
            Ok(updated_password_hash) => Ok(ApiResponse::success(updated_password_hash).into_response()),
            Err(e) => {
                error!("Error updating verification code: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
    
}

    