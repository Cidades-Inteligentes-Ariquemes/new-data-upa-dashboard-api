use actix_web::web;
use crate::routes::{users, auth};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(auth::configure_routes)
            .configure(users::configure_routes)
    );
}