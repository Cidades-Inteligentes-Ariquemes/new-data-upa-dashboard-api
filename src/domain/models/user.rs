use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub profile: String,
    pub allowed_applications: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserDto {
    pub full_name: String,
    pub email: String,
    pub password: String,
    pub profile: String,
    pub allowed_applications: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserDto {
    pub full_name: String,
    pub email: String,
    pub profile: String,
    pub allowed_applications: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordByAdminDto {
    pub email: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordByUserCommonDto {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub profile: String,
    pub allowed_applications: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct AddApplicationDto {
    pub applications_name: Vec<String>,
}

#[derive(Deserialize)]
pub struct ApplicationPath {
    pub id: Uuid,
    pub application_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFeedbackRespiratoryDiseasesDto {
    pub user_name: String,
    pub feedback: String,
    pub prediction_made: String,
    pub correct_prediction: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedbackRespiratoryDiseasesResponse {
    pub id: Uuid,
    pub user_name: String,
    pub feedback: String,
    pub prediction_made: String,
    pub correct_prediction: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFeedbackTuberculosisDto {
    pub user_name: String,
    pub feedback: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedbackTuberculosisResponse {
    pub id: Uuid,
    pub user_name: String,
    pub feedback: String,
}

#[derive(Debug, Serialize)]
pub struct DiseaseStats {
    pub total_quantity: i32,
    pub total_quantity_correct: i32,
}

#[derive(Debug, Serialize)]
pub struct ProcessedFeedbackResponse {
    pub normal: DiseaseStats,
    pub covid_19: DiseaseStats,
    pub pneumonia_viral: DiseaseStats,
    pub pneumonia_bacteriana: DiseaseStats,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            full_name: user.full_name,
            email: user.email,
            profile: user.profile,
            allowed_applications: user.allowed_applications,
            enabled: user.enabled,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}