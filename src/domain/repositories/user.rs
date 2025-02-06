use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::models::user::{
    AddApplicationDto,
    CreateUserDto,
    UpdateUserDto,
    CreateFeedbackRespiratoryDiseasesDto,
    FeedbackRespiratoryDiseasesResponse,
    CreateFeedbackTuberculosisDto,
    User,
    FeedbackTuberculosisResponse,
    UpdateEnabledUserDto,
    AddVerificationCodeDto,
    AddVerificationCodeResponse,
    UpdateVerificationCodeDto,
};

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
    async fn create(&self, user: CreateUserDto) -> Result<User, sqlx::Error>;
    async fn update(&self, id: Uuid, user: UpdateUserDto) -> Result<Option<User>, sqlx::Error>;
    async fn update_password(&self, id: Uuid, new_password: String) -> Result<bool, sqlx::Error>;
    async fn update_enabled(&self, id: Uuid, new_enabled: UpdateEnabledUserDto) -> Result<bool, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error>;
    async fn delete_application(&self, id: Uuid, application_name: &str) -> Result<bool, sqlx::Error>;
    async fn add_application(&self, id: Uuid, applications: AddApplicationDto) -> Result<Option<User>, sqlx::Error>;
    async fn create_feedback_respiratory_diseases(&self, feedback: CreateFeedbackRespiratoryDiseasesDto) -> Result<Option<FeedbackRespiratoryDiseasesResponse>, sqlx::Error>;
    async fn find_all_feedbacks_respiratory_diseases(&self) -> Result<Vec<FeedbackRespiratoryDiseasesResponse>, sqlx::Error>;
    async fn create_feedback_tuberculosis(&self, feedback_tuberculosis: CreateFeedbackTuberculosisDto) -> Result<Option<FeedbackTuberculosisResponse>, sqlx::Error>;
    async fn find_all_feedbacks_tuberculosis(&self) -> Result<Vec<FeedbackTuberculosisResponse>, sqlx::Error>;
    async fn add_verification_code(&self, data:AddVerificationCodeDto) -> Result<AddVerificationCodeResponse, sqlx::Error>;
    async fn verify_code_exist(&self, id: Uuid) -> Result<AddVerificationCodeResponse, sqlx::Error>;
    async fn update_code_verification(&self, code: UpdateVerificationCodeDto, email: String, id_verification: Uuid,) -> Result<AddVerificationCodeResponse, sqlx::Error>;
    async fn update_used_verification_code(&self, id_verification: Uuid) -> Result<AddVerificationCodeResponse, sqlx::Error>;
    async fn update_password_for_forgetting_user(&self, user_id: Uuid, new_password: String) -> Result<bool, sqlx::Error>;
}