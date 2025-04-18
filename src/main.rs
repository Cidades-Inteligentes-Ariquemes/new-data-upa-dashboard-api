use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware::Logger};
use env_logger::{Builder, Env};
use log::info;

use new_data_upa_dashboard_api::{
   adapters::{
       password::encryptor::Argon2PasswordEncryptor,
       token::generate_token::JwtTokenGenerator,
   },
   application::{
       auth_service::AuthService,
       auth_pronto_service::AuthProntoService,
       user_service::UserService,
       machine_information_service::MachineInformationService,
   },
   infrastructure::{
       database::init_database,
       repositories::{
           user_repository::PgUserRepository,
           auth_pronto_repository::SqlServerAuthProntoRepository,
       },
   },
   middleware::{
       auth::AuthMiddleware,
       logging::LoggingMiddleware,
   },
   routes::config::routes::configure_routes,
   utils::config_env::Config,
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

   // Cria os adapters
   let password_encryptor = Box::new(Argon2PasswordEncryptor::new());

   // Cria os repositórios
   let user_repository = web::Data::new(PgUserRepository::new(pool));
   
   info!("Repositórios criados");

   // Cria os services
   let user_service = web::Data::new(UserService::new(
       user_repository.clone(),
       password_encryptor.clone(),
       web::Data::new(config.clone()),
   ));

   let auth_service = web::Data::new(AuthService::new(
       user_repository.clone(),
       web::Data::new(config.clone()),
       Box::new(Argon2PasswordEncryptor::new()),
       Box::new(JwtTokenGenerator::new()),
   ));

   let machine_information_service = web::Data::new(MachineInformationService);

   let server_addr = config.server_addr.clone();
   info!("Server será iniciado em: http://{}", server_addr);

   HttpServer::new(move || {
       // Cria uma nova instância do repositório para cada worker
       let auth_pronto_repository = SqlServerAuthProntoRepository::new(config.clone());
       
       // Cria uma nova instância do serviço para cada worker
       let auth_pronto_service = web::Data::new(AuthProntoService::new(
           Box::new(auth_pronto_repository),
           web::Data::new(config.clone()),
           Box::new(JwtTokenGenerator::new()),
       ));
       
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
           .app_data(user_service.clone())
           .app_data(auth_service.clone())
           .app_data(auth_pronto_service)
           .app_data(web::Data::new(config.clone()))
           .app_data(machine_information_service.clone())
           .configure(configure_routes)
   })
   .bind(server_addr)?
   .run()
   .await
}