use actix_web::web;
use crate::handlers::information::information_handler;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/information")
            .service(
                web::resource("/audits/audit-filters")
                    .route(web::get().to(information_handler::get_available_data))
            )
            .service(
                web::resource("/audits/{page}")
                    .route(web::get().to(information_handler::audits))
            )
    );
}