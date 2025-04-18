use actix_web::{web, HttpResponse};
use crate::{
    application::auth_pronto_service::AuthProntoService,
    domain::models::auth_pronto::UserLoginPronto,
    AppError,
};

pub async fn login_pronto(
    service: web::Data<AuthProntoService>,
    credentials: web::Json<UserLoginPronto>,
) -> Result<HttpResponse, AppError> {
    service.login_pronto(credentials.into_inner()).await
}
