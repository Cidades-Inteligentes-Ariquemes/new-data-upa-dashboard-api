use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use log::{error, info};
use crate::domain::models::user::{CreateUserDto, UpdateUserDto, User, UpdatePasswordDto};
use crate::domain::repositories::user::UserRepository;
use crate::domain::models::auth::LoginDto;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string())
    }

    fn verify_password(&self, hash: &str, password: &str) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(hash)?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
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

    async fn create(&self, user: CreateUserDto) -> Result<User, sqlx::Error> {
        let hashed_password = self.hash_password(&user.password)
            .map_err(|_| sqlx::Error::Protocol("Password hashing failed".into()))?;

        let user = sqlx::query!(
            r#"
            INSERT INTO users_api (
                full_name,
                email,
                password,
                profile,
                allowed_applications,
                enabled
            )
            VALUES ($1, $2, $3, $4, $5, true)
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
            hashed_password,
            user.profile,
            &user.allowed_applications,
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

        if let Some(user_data) = current_user {
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
                user.full_name.unwrap_or(user_data.full_name),
                user.email.unwrap_or(user_data.email),
                user.profile.unwrap_or(user_data.profile),
                &user.allowed_applications.unwrap_or(user_data.allowed_applications),
                user.enabled.unwrap_or(user_data.enabled),
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

    async fn update_password(&self, id: Uuid, passwords: UpdatePasswordDto) -> Result<bool, sqlx::Error> {
        let current_user = self.find_by_id(id).await?;

        if let Some(user) = current_user {
            let is_valid = self.verify_password(&user.password, &passwords.current_password)
                .map_err(|_| sqlx::Error::Protocol("Password verification failed".into()))?;

            if !is_valid {
                return Ok(false);
            }

            let new_password_hash = self.hash_password(&passwords.new_password)
                .map_err(|_| sqlx::Error::Protocol("Password hashing failed".into()))?;

            let result = sqlx::query!(
                r#"
                UPDATE users_api
                SET password = $1
                WHERE id = $2
                "#,
                new_password_hash,
                id
            )
                .execute(&self.pool)
                .await?;

            Ok(result.rows_affected() > 0)
        } else {
            Ok(false)
        }
    }

    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE users_api
            SET enabled = false
            WHERE id = $1
            "#,
            id
        )
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn authenticate(&self, credentials: LoginDto) -> Result<Option<User>, sqlx::Error> {
        info!("=== Iniciando autenticação no repositório ===");
        info!("Email recebido: {}", credentials.email);

        let result = sqlx::query!(
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
        WHERE email = $1 AND enabled = true
        "#,
        credentials.email
    )
            .fetch_optional(&self.pool)
            .await;

        match &result {
            Ok(Some(user)) => {
                info!("Usuário encontrado no banco: {}", user.email);
                if user.enabled {
                    info!("Usuário está ativo");
                } else {
                    info!("Usuário está inativo");
                }
            },
            Ok(None) => info!("Usuário não encontrado no banco para o email: {}", credentials.email),
            Err(e) => error!("Erro ao consultar banco: {:?}", e),
        }

        match result {
            Ok(Some(user)) => {
                match self.verify_password(&user.password, &credentials.password) {
                    Ok(true) => {
                        info!("Senha verificada com sucesso");
                        Ok(Some(User {
                            id: user.id,
                            full_name: user.full_name,
                            email: user.email,
                            password: user.password,
                            profile: user.profile,
                            allowed_applications: user.allowed_applications,
                            enabled: user.enabled,
                        }))
                    },
                    Ok(false) => {
                        info!("Senha inválida para o usuário: {}", credentials.email);
                        Ok(None)
                    },
                    Err(e) => {
                        error!("Erro ao verificar senha: {:?}", e);
                        Err(sqlx::Error::Protocol(format!("Password verification failed: {}", e)))
                    }
                }
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}