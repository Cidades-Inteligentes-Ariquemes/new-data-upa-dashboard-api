use actix_web::web;
use crate::handlers::user::user_handler;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(
                web::resource("/feedback-respiratory-diseases")
                    .route(web::post().to(user_handler::create_feedback_respiratory_diseases))
            )
            .service(
                web::resource("/feedback-tuberculosis")
                    .route(web::post().to(user_handler::create_feedback_tuberculosis))
            )
            .service(
                web::resource("/feedbacks")
                    .route(web::get().to(user_handler::get_feedbacks_respiratory_diseases))
            )
            .service(
                web::resource("")
                    .route(web::get().to(user_handler::get_users))
                    .route(web::post().to(user_handler::create_user))
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(user_handler::get_user_by_id))
                    .route(web::put().to(user_handler::update_user))
                    .route(web::delete().to(user_handler::delete_user))
            )
            .service(
                web::resource("/{id}/update-password-by-admin")
                    .route(web::patch().to(user_handler::update_password_by_admin))
            )
            .service(
                web::resource("/{id}/application/{application_name}")
                    .route(web::delete().to(user_handler::delete_application))
            ) 
            .service(
                web::resource("/{id}/applications")
                    .route(web::post().to(user_handler::add_application))
            )
            .service(
                web::resource("/{id}/update-password-by-user-common")
                    .route(web::patch().to(user_handler::update_password_by_user_common))
            )      
    );
}