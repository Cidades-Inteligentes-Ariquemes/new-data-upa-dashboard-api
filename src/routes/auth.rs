use actix_web::web;
use crate::handlers::auth::{auth_handler, pronto_auth_handler};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(
                web::resource("/login")
                    .route(web::post().to(auth_handler::login))
            )
            .service(
                web::resource("/login-pronto")
                    .route(web::post().to(pronto_auth_handler::login_pronto))
            )
    );
}
