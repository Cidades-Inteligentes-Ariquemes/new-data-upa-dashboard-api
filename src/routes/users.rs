use actix_web::web;
use crate::application::user_service;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(
                web::resource("")
                    .route(web::get().to(user_service::get_users))
                    .route(web::post().to(user_service::create_user))
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(user_service::get_user_by_id))
                    .route(web::put().to(user_service::update_user))
                    .route(web::delete().to(user_service::delete_user))
            )
            .service(
                web::resource("/{id}/password")
                    .route(web::put().to(user_service::update_password))
            )
    );
}