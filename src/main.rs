use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware::Logger};
use env_logger::{Builder, Env};
use log::info;

use new_data_upa_dashboard_api::{
    utils::config_env::Config,
    infrastructure::{
        database::init_database,
        repositories::user_repository::PgUserRepository,
    },
    routes::config::routes::configure_routes,
    middleware::{
        auth::AuthMiddleware,
        logging::LoggingMiddleware
    },
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let env = Env::default()
        .filter_or("RUST_LOG", "debug,actix_web=debug");

    Builder::from_env(env)
        .format_timestamp_millis()
        .format_module_path(true)
        .init();

    dotenv::dotenv().ok();

    let config = Config::from_env();

    let pool = init_database(&config.database_url).await;
    info!("Conexão com o banco de dados estabelecida");

    let user_repository = web::Data::new(PgUserRepository::new(pool.clone()));
    info!("Repositório de usuários criado");

    let server_addr = config.server_addr.clone();
    info!("Server será iniciado em: http://{}", server_addr);

    HttpServer::new(move || {

        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(LoggingMiddleware)
            .wrap(AuthMiddleware)
            .app_data(user_repository.clone())
            .app_data(web::Data::new(config.clone()))
            .configure(configure_routes)
    })
        .bind(server_addr)?
        .run()
        .await
}