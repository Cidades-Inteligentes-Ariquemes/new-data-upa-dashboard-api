use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use futures::StreamExt;
use log::error;
use crate::{
    application::data_upa_service::DataUpaService,
    AppError,
};

pub async fn add_data(
    service: web::Data<DataUpaService>,
    mut payload: Multipart,
) -> Result<HttpResponse, AppError> {
    // Extrair arquivo do upload multipart
    let mut file_data = web::BytesMut::new();
    
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => {
                error!("Erro ao processar campo do multipart: {:?}", e);
                return Err(AppError::BadRequest("Erro ao processar arquivo enviado".to_string()));
            }
        };
        
        // Verificar se é o campo de arquivo
        if field.name() == Some("file") {
            // Ler todos os dados do campo
            while let Some(chunk) = field.next().await {
                match chunk {
                    Ok(data) => file_data.extend_from_slice(&data),
                    Err(e) => {
                        error!("Erro ao ler chunk do arquivo: {:?}", e);
                        return Err(AppError::BadRequest("Erro ao ler arquivo enviado".to_string()));
                    }
                }
            }
            break;
        }
    }
    
    if file_data.is_empty() {
        error!("Nenhum arquivo foi enviado");
        return Err(AppError::BadRequest("Nenhum arquivo foi enviado".to_string()));
    }

    // Processa e salva os dados
    service.add_data(file_data.freeze()).await
}