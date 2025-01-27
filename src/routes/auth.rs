use actix_web::web;
use log::info;
use crate::application::auth_service;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    info!("Configurando rotas de autenticação"); // Adicione este log
    cfg.service(
        web::scope("/auth")
            .service(
                web::resource("/login")
                    .route(web::post().to(auth_service::login))
            )
    );
}