use jsonwebtoken::{encode, EncodingKey, Header, errors::Error as JwtError};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::domain::models::auth::Claims;

// Interface para geração de tokens
pub trait TokenGeneratorPort: Send + Sync {
    fn generate_token(&self, user_id: String, profile: String, secret: &str) -> Result<String, JwtError>;
}

// Implementação usando JWT
#[derive(Clone)] // Permite clonagem do adaptador
pub struct JwtTokenGenerator;

impl JwtTokenGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl TokenGeneratorPort for JwtTokenGenerator {
    fn generate_token(&self, user_id: String, profile: String, secret: &str) -> Result<String, JwtError> {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + 24 * 3600; // Expiração de 24 horas

        let claims = Claims {
            sub: user_id,
            exp: expiration,
            profile,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
    }
}