use actix_web::web;
use crate::routes::{users, auth, machine_information, data_upa, prediction, information, swagger};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(auth::configure_routes)
            .configure(users::configure_routes)
            .configure(machine_information::configure_routes)
            .configure(data_upa::configure_routes)
            .configure(prediction::configure_routes)
            .configure(information::configure_routes)
            .configure(swagger::configure_routes)
    );
}