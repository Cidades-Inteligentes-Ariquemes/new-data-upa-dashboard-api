use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;
use crate::domain::{
    models::user::{CreateUserDto, UpdateUserDto, UpdatePasswordDto, UserResponse},
    repositories::user::UserRepository,
};
use crate::infrastructure::repositories::user_repository::PgUserRepository;

pub async fn get_users(repo: web::Data<PgUserRepository>) -> impl Responder {
    match repo.find_all().await {
        Ok(users) => {
            let responses: Vec<UserResponse> = users.into_iter()
                .map(UserResponse::from)
                .collect();
            HttpResponse::Ok().json(responses)
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_user_by_id(
    repo: web::Data<PgUserRepository>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match repo.find_by_id(id.into_inner()).await {
        Ok(Some(user)) => HttpResponse::Ok().json(UserResponse::from(user)),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn create_user(
    repo: web::Data<PgUserRepository>,
    user: web::Json<CreateUserDto>,
) -> impl Responder {
    match repo.create(user.into_inner()).await {
        Ok(user) => HttpResponse::Created().json(UserResponse::from(user)),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update_user(
    repo: web::Data<PgUserRepository>,
    id: web::Path<Uuid>,
    user: web::Json<UpdateUserDto>,
) -> impl Responder {
    match repo.update(id.into_inner(), user.into_inner()).await {
        Ok(Some(user)) => HttpResponse::Ok().json(UserResponse::from(user)),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update_password(
    repo: web::Data<PgUserRepository>,
    id: web::Path<Uuid>,
    passwords: web::Json<UpdatePasswordDto>,
) -> impl Responder {
    match repo.update_password(id.into_inner(), passwords.into_inner()).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Password updated successfully"
        })),
        Ok(false) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Current password is incorrect or models not found"
        })),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn delete_user(
    repo: web::Data<PgUserRepository>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match repo.delete(id.into_inner()).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}