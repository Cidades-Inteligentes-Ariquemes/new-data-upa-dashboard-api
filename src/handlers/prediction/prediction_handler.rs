use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use futures::StreamExt;
use std::io::Write;

use crate::AppError;
use crate::application::prediction_service::PredictionService;

pub async fn predict(
    service: web::Data<PredictionService>,
    mut payload: Multipart,
) -> Result<HttpResponse, AppError> {
    // Extrair o arquivo da requisição
    let mut image_data = Vec::new();
    
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_) => return Err(AppError::BadRequest("Erro ao processar o formulário multipart".to_string())),
        };

        if field.name() == Some("file") {
            // Ler os dados da imagem
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(d) => d,
                    Err(_) => return Err(AppError::BadRequest("Erro ao ler o arquivo".to_string())),
                };
                image_data.write_all(&data).unwrap();
            }
            break;
        }
    }

    if image_data.is_empty() {
        return Err(AppError::BadRequest("Nenhuma imagem enviada".to_string()));
    }

    // Chamar o serviço para fazer a predição
    service.predict_image(image_data).await
}

pub async fn detect_breast_cancer(
    service: web::Data<PredictionService>,
    mut payload: Multipart,
) -> Result<HttpResponse, AppError> {
    // Extrair o arquivo da requisição
    let mut image_data = Vec::new();
    
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_) => return Err(AppError::BadRequest("Erro ao processar o formulário multipart".to_string())),
        };

        if field.name() == Some("file") {
            // Ler os dados da imagem
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(d) => d,
                    Err(_) => return Err(AppError::BadRequest("Erro ao ler o arquivo".to_string())),
                };
                image_data.write_all(&data).unwrap();
            }
            break;
        }
    }

    if image_data.is_empty() {
        return Err(AppError::BadRequest("Nenhuma imagem enviada".to_string()));
    }

    // Chamar o serviço para fazer a detecção
    service.detect_breast_cancer(image_data).await
}

pub async fn predict_tuberculosis(
    service: web::Data<PredictionService>,
    mut payload: Multipart,
) -> Result<HttpResponse, AppError> {
    // Extrair o arquivo da requisição
    let mut image_data = Vec::new();
    
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_) => return Err(AppError::BadRequest("Erro ao processar o formulário multipart".to_string())),
        };

        if field.name() == Some("file") {
            // Ler os dados da imagem
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(d) => d,
                    Err(_) => return Err(AppError::BadRequest("Erro ao ler o arquivo".to_string())),
                };
                image_data.write_all(&data).unwrap();
            }
            break;
        }
    }

    if image_data.is_empty() {
        return Err(AppError::BadRequest("Nenhuma imagem enviada".to_string()));
    }

    // Chamar o serviço para fazer a predição de tuberculose
    service.predict_tuberculosis(image_data).await
}