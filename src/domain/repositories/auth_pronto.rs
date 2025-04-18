use async_trait::async_trait;
use crate::domain::models::auth_pronto::{UserPronto, ProfileInfo};

#[async_trait]
pub trait AuthProntoRepository: Send + Sync + 'static {
    async fn get_user_pronto_by_username_with_fullname(&self, username: &str) -> Result<Option<UserPronto>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_user_profiles_by_login_and_unit_id(&self, login_id: &str, unit_id: i32) -> Result<Vec<ProfileInfo>, Box<dyn std::error::Error + Send + Sync>>;
}
