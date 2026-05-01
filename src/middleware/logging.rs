use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error, HttpMessage, HttpRequest,
};
use futures::future::{ready, Ready};
use log::{error, info, warn};
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;
use uuid::Uuid;

const SLOW_REQUEST_THRESHOLD_MS: u128 = 5_000;
const USER_AGENT_MAX_LEN: usize = 120;

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

pub fn request_id_from_http_request(req: &HttpRequest) -> Option<String> {
    req.extensions().get::<RequestId>().map(|id| id.0.clone())
}

fn truncate_value(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        value.to_string()
    } else {
        let truncated: String = value.chars().take(max_len).collect();
        format!("{}...", truncated)
    }
}

pub struct LoggingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for LoggingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LoggingMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggingMiddlewareService { service }))
    }
}

pub struct LoggingMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for LoggingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().clone();
        let path = req.path().to_owned();
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();
        let user_agent = req
            .headers()
            .get(header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(|value| truncate_value(value, USER_AGENT_MAX_LEN))
            .unwrap_or_else(|| "unknown".to_string());
        let request_id = Uuid::new_v4().to_string();

        req.extensions_mut().insert(RequestId(request_id.clone()));

        info!(
            "[request_id={}] request.start method={} path={} ip={} user_agent=\"{}\"",
            request_id, method, path, ip, user_agent
        );

        let started_at = Instant::now();
        let fut = self.service.call(req);

        Box::pin(async move {
            match fut.await {
                Ok(res) => {
                    let duration_ms = started_at.elapsed().as_millis();
                    let status = res.status().as_u16();

                    if status >= 500 {
                        error!(
                            "[request_id={}] request.finish method={} path={} status={} duration_ms={}",
                            request_id, method, path, status, duration_ms
                        );
                    } else if duration_ms >= SLOW_REQUEST_THRESHOLD_MS {
                        warn!(
                            "[request_id={}] request.finish method={} path={} status={} duration_ms={}",
                            request_id, method, path, status, duration_ms
                        );
                    } else {
                        info!(
                            "[request_id={}] request.finish method={} path={} status={} duration_ms={}",
                            request_id, method, path, status, duration_ms
                        );
                    }

                    Ok(res)
                }
                Err(err) => {
                    let duration_ms = started_at.elapsed().as_millis();
                    error!(
                        "[request_id={}] request.failed method={} path={} duration_ms={} error={}",
                        request_id, method, path, duration_ms, err
                    );
                    Err(err)
                }
            }
        })
    }
}
