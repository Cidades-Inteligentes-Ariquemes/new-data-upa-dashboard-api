use actix_web::{web, HttpResponse};
use uuid::Uuid;
use crate::{
    application::user_service::UserService,
    domain::models::user::{AddApplicationDto, ApplicationPath, CreateUserDto, UpdatePasswordByAdminDto, UpdateUserDto},
    AppError,
};

pub async fn get_users(service: web::Data<UserService>) -> Result<HttpResponse, AppError> {
    service.get_users().await
}

pub async fn create_user(
    service: web::Data<UserService>,
    user: web::Json<CreateUserDto>,
) -> Result<HttpResponse, AppError> {
    service.create_user(user.into_inner()).await
}

pub async fn get_user_by_id(
    service: web::Data<UserService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    service.get_user_by_id(id.into_inner()).await
}

pub async fn update_user(
    service: web::Data<UserService>,
    id: web::Path<Uuid>,
    user: web::Json<UpdateUserDto>,
) -> Result<HttpResponse, AppError> {
    service.update_user(id.into_inner(), user.into_inner()).await
}

pub async fn delete_user(
    service: web::Data<UserService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    service.delete_user(id.into_inner()).await
}

pub async fn update_password_by_admin(
    service: web::Data<UserService>,
    id: web::Path<Uuid>,
    passwords: web::Json<UpdatePasswordByAdminDto>,
) -> Result<HttpResponse, AppError> {
    service.update_password_by_admin(id.into_inner(), passwords.into_inner()).await
}

pub async fn add_application(
    service: web::Data<UserService>,
    id: web::Path<Uuid>,
    applications: web::Json<AddApplicationDto>,
) -> Result<HttpResponse, AppError> {
    service.add_application(id.into_inner(), applications.into_inner()).await
}

pub async fn delete_application(
    service: web::Data<UserService>,
    path: web::Path<ApplicationPath>,
) -> Result<HttpResponse, AppError> {
    let params = path.into_inner();
    service.delete_application(params.id, params.application_name).await
}