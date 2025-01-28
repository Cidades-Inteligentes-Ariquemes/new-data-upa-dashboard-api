use actix_web::{web, HttpResponse};
use crate::{
    application::auth_service::AuthService,
    domain::models::auth::LoginDto,
    AppError,
};

pub async fn login(
    service: web::Data<AuthService>,
    credentials: web::Json<LoginDto>,
) -> Result<HttpResponse, AppError> {
    service.login(credentials.into_inner()).await
}