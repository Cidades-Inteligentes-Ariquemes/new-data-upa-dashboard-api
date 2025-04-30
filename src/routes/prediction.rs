use actix_web::web;
use crate::handlers::prediction::prediction_handler;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/prediction")
            .service(
                web::resource("/predict")
                    .route(web::post().to(prediction_handler::predict))
            )
            .service(
                web::resource("/detect")
                    .route(web::post().to(prediction_handler::detect_breast_cancer))
            )
            .service(
                web::resource("/predict_tb")
                    .route(web::post().to(prediction_handler::predict_tuberculosis))
            )
    );
}