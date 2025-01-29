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
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordByAdminDto {
    pub email: String,
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