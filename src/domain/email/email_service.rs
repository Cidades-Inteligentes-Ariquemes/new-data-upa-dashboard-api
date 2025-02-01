use serde::Serialize;
use async_trait::async_trait;

#[derive(Serialize)]
pub struct EmailContext {
    pub user: String,
    pub code_verification: String,
}

#[derive(Serialize)]
pub struct EmailRequest {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub context: EmailContext,
    pub template: String,
}

#[async_trait]
pub trait EmailService {
    async fn send_email(&self, user: String, email_user: String, code_verification: String) -> Result<bool, Box<dyn std::error::Error>>;
}