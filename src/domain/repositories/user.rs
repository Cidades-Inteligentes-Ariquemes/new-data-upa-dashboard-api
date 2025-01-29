use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::models::user::{CreateUserDto, UpdateUserDto, User};

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
    async fn create(&self, user: CreateUserDto) -> Result<User, sqlx::Error>;
    async fn update(&self, id: Uuid, user: UpdateUserDto) -> Result<Option<User>, sqlx::Error>;
    async fn update_password(&self, id: Uuid, new_password: String) -> Result<bool, sqlx::Error>;    
    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error>;
    async fn delete_application(&self, id: Uuid, application_name: &str) -> Result<bool, sqlx::Error>;
}