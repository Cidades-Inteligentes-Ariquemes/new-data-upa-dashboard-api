use reqwest;
use async_trait::async_trait;
use crate::domain::email::email_service::{EmailService, EmailRequest, EmailContext};
use crate::utils::config_env::Config;
use actix_web::web;
use log::error;

pub struct SmtpEmailService {
    client: reqwest::Client,
    email_from: String,
    api_url: String,
}

impl SmtpEmailService {
    pub fn new(
        config: web::Data<Config>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            email_from: config.email.clone(),
            api_url: String::from("https://simple-mail-compose-simple-mail.o3luz9.easypanel.host/send-email"),
        }
    }
}

#[async_trait]
impl EmailService for SmtpEmailService {
    async fn send_email(
        &self,
        user: String,
        email_user: String,
        code_verification: String,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let request = EmailRequest {
            from: self.email_from.clone(),
            to: email_user,
            subject: format!("Recuperação de senha {}", Config::from_env().app_name),
            context: EmailContext {
                user,
                code_verification,
            },
            template: String::from("main"),
        };

        match self.client
            .post(&self.api_url)
            .json(&request)
            .send()
            .await
        {
            Ok(_response) => {
                Ok(true)
            }
            Err(e) => {
                error!("Error sending email: {:?}", e);
                Ok(false)
            }
        }
    }
}