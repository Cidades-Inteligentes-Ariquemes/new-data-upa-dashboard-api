use actix_web::web;
use crate::handlers::data::data_upa_handler;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/data")
            .service(
                web::resource("/add-file")
                    .route(web::post().to(data_upa_handler::add_data))
            )

    );
}
