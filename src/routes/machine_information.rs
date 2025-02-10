use actix_web::web;
use crate::handlers::machine_information::machine_information_handler;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/machine-information")
            .service(
                web::resource("")
                    .route(web::get().to(machine_information_handler::get_machine_information))
            )
    );
}