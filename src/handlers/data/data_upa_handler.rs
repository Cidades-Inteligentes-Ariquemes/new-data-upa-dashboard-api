use crate::{
    application::data_upa_service::DataUpaService,
    middleware::logging::request_id_from_http_request, AppError,
};
use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::StreamExt;
use log::error;

pub async fn add_data(
    req: HttpRequest,
    service: web::Data<DataUpaService>,
    mut payload: Multipart,
) -> Result<HttpResponse, AppError> {
    // Extrair arquivo do upload multipart
    let mut file_data = web::BytesMut::new();
    let request_id = request_id_from_http_request(&req);
    let request_id = request_id.as_deref().unwrap_or("unknown");

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao processar campo do multipart: {:?}",
                    request_id, e
                );
                return Err(AppError::BadRequest(
                    "Erro ao processar arquivo enviado".to_string(),
                ));
            }
        };

        // Verificar se é o campo de arquivo
        if field.name() == Some("file") {
            // Ler todos os dados do campo
            while let Some(chunk) = field.next().await {
                match chunk {
                    Ok(data) => file_data.extend_from_slice(&data),
                    Err(e) => {
                        error!(
                            "[request_id={}] Erro ao ler chunk do arquivo: {:?}",
                            request_id, e
                        );
                        return Err(AppError::BadRequest(
                            "Erro ao ler arquivo enviado".to_string(),
                        ));
                    }
                }
            }
            break;
        }
    }

    if file_data.is_empty() {
        error!("[request_id={}] Nenhum arquivo foi enviado", request_id);
        return Err(AppError::BadRequest(
            "Nenhum arquivo foi enviado".to_string(),
        ));
    }

    // Processa e salva os dados
    service.add_data(file_data.freeze(), Some(request_id)).await
}

pub async fn available_health_units(
    service: web::Data<DataUpaService>,
) -> Result<HttpResponse, AppError> {
    service.get_available_health_units().await
}
