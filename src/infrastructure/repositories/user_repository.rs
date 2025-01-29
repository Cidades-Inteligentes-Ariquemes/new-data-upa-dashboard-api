use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::models::user::{CreateUserDto, UpdateUserDto, User};
use crate::domain::repositories::user::UserRepository;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error> {
        let users = sqlx::query!(
            r#"
            SELECT
                id,
                full_name,
                email,
                password,
                profile,
                allowed_applications as "allowed_applications!: Vec<String>",
                enabled
            FROM users_api
            WHERE enabled = true
            ORDER BY full_name
            "#
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(users.into_iter().map(|row| User {
            id: row.id,
            full_name: row.full_name,
            email: row.email,
            password: row.password,
            profile: row.profile,
            allowed_applications: row.allowed_applications,
            enabled: row.enabled,
        }).collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query!(
            r#"
            SELECT
                id,
                full_name,
                email,
                password,
                profile,
                allowed_applications as "allowed_applications!: Vec<String>",
                enabled
            FROM users_api
            WHERE id = $1
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await?;

        Ok(user.map(|row| User {
            id: row.id,
            full_name: row.full_name,
            email: row.email,
            password: row.password,
            profile: row.profile,
            allowed_applications: row.allowed_applications,
            enabled: row.enabled,
        }))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query!(
            r#"
            SELECT
                id,
                full_name,
                email,
                password,
                profile,
                allowed_applications as "allowed_applications!: Vec<String>",
                enabled
            FROM users_api
            WHERE email = $1
            "#,
            email
        )
            .fetch_optional(&self.pool)
            .await?;

        Ok(user.map(|row| User {
            id: row.id,
            full_name: row.full_name,
            email: row.email,
            password: row.password,
            profile: row.profile,
            allowed_applications: row.allowed_applications,
            enabled: row.enabled,
        }))
    }

    async fn create(&self, user: CreateUserDto) -> Result<User, sqlx::Error> {
        let id = Uuid::new_v4();
    
        let user = sqlx::query!(
            r#"
            INSERT INTO users_api (
                id,
                full_name,
                email,
                password,
                profile,
                allowed_applications,
                enabled
            )
            VALUES ($1, $2, $3, $4, $5, $6, true)
            RETURNING
                id,
                full_name,
                email,
                password,
                profile,
                allowed_applications as "allowed_applications!: Vec<String>",
                enabled
            "#,
            id,
            user.full_name,
            user.email,
            user.password, 
            user.profile,
            &user.allowed_applications as &[String],
        )
        .fetch_one(&self.pool)
        .await?;
    
        Ok(User {
            id: user.id,
            full_name: user.full_name,
            email: user.email,
            password: user.password,
            profile: user.profile,
            allowed_applications: user.allowed_applications,
            enabled: user.enabled,
        })
    }

    async fn update(&self, id: Uuid, user: UpdateUserDto) -> Result<Option<User>, sqlx::Error> {
        let current_user = self.find_by_id(id).await?;

        if let Some(_) = current_user {
            let updated_user = sqlx::query!(
                r#"
                UPDATE users_api
                SET
                    full_name = $1,
                    email = $2,
                    profile = $3,
                    allowed_applications = $4,
                    enabled = $5
                WHERE id = $6
                RETURNING
                    id,
                    full_name,
                    email,
                    password,
                    profile,
                    allowed_applications as "allowed_applications!: Vec<String>",
                    enabled
                "#,
                user.full_name,
                user.email,
                user.profile,
                &user.allowed_applications as &[String],
                user.enabled,
                id
            )
                .fetch_one(&self.pool)
                .await?;

            Ok(Some(User {
                id: updated_user.id,
                full_name: updated_user.full_name,
                email: updated_user.email,
                password: updated_user.password,
                profile: updated_user.profile,
                allowed_applications: updated_user.allowed_applications,
                enabled: updated_user.enabled,
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_password(&self, id: Uuid, new_password: String) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE users_api
            SET password = $1
            WHERE id = $2
            "#,
            new_password,
            id
        )
        .execute(&self.pool)
        .await?;
    
        Ok(result.rows_affected() > 0)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM users_api
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
    
        Ok(result.rows_affected() > 0)
    }
}