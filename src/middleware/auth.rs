use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::domain::models::auth::Claims;
use crate::utils::config_env::Config;

// 1. Estrutura principal do Middleware
pub struct AuthMiddleware;

// 2. Implementação do Transform trait para AuthMiddleware
impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
// Restrições de tipo necessarily para o Service
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    // Cria uma nova instância do middleware
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

// 3. Serviço do Middleware que faz o trabalho real
pub struct AuthMiddlewareService<S> {
    service: S,
}

// 4. Implementação do Service trait - Onde a lógica principal acontece
impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    // Helper fornecido pelo actix para lidar com readiness do serviço
    forward_ready!(service);

    // Método principal que processa cada requisição

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // 5. Bypass para a rota de login
        println!("Hi from start. You requested: {}", req.path());
        if req.path() == "/api/auth/login" {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        // 6. Extrai e valida o header de autorização
        let auth_header = req.headers().get("Authorization");
        let config = req.app_data::<actix_web::web::Data<Config>>().unwrap();

        let auth_header = match auth_header {
            Some(header) => header.to_str().unwrap_or_default(),
            None => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("No authorization header"))
                })
            }
        };

        // 7. Verifica se é um token Bearer
        if !auth_header.starts_with("Bearer ") {
            return Box::pin(async move {
                Err(ErrorUnauthorized("Invalid authorization header"))
            });
        }

        // 8. Extrai e valida o token JWT
        let token = &auth_header[7..]; // Remove "Bearer " do início
        let token_data = match decode::<Claims>(
            token,
            &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(data) => data,
            Err(_) => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("Invalid token"))
                })
            }
        };

        // 9. Adiciona os claims ao request para uso posterior
        req.extensions_mut().insert(token_data.claims);

        // 10. Passa a requisição para o próximo handler
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}