use actix_web::HttpResponse;
use reqwest::Client;
use anyhow::Result;
use log::error;

use crate::{
    domain::models::prediction::{DetectionResponse, PredictionResponse, TBResponse}, 
    utils::response::ApiResponse, AppError
};


pub struct PredictionService {
    client: Client,
    ml_api_url: String,
}

impl PredictionService {
    pub fn new(ml_api_url: String) -> Self {
        Self {
            client: Client::new(),
            ml_api_url,
        }
    }

    pub async fn predict_image(&self, image_data: Vec<u8>) -> Result<HttpResponse, AppError> {
        // Criar o formulário com a imagem
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(image_data)
                .file_name("image.jpg")
                .mime_str("image/jpeg").map_err(|e| {
                    error!("Erro ao definir MIME type: {:?}", e);
                    AppError::InternalServerError
                })?);

        // Fazer a requisição para a API Python
        let response = self.client.post(format!("{}/predict", self.ml_api_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                error!("Erro ao chamar a API de ML: {:?}", e);
                AppError::InternalServerError
            })?;

        // Analisar a resposta
        if response.status().is_success() {
            let predict_response: PredictionResponse = response.json().await.map_err(|e| {
                error!("Erro ao deserializar resposta da API de ML: {:?}", e);
                AppError::InternalServerError
            })?;

            Ok(ApiResponse::success(predict_response).into_response())
        } else {
            error!("API ML retornou erro: {}", response.status());
            Err(AppError::InternalServerError)
        }
    }

    pub async fn detect_breast_cancer(&self, image_data: Vec<u8>) -> Result<HttpResponse, AppError> {
        // Criar o formulário com a imagem
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(image_data)
                .file_name("image.jpg")
                .mime_str("image/jpeg").map_err(|e| {
                    error!("Erro ao definir MIME type: {:?}", e);
                    AppError::InternalServerError
                })?);

        // Fazer a requisição para a API Python
        let response = self.client.post(format!("{}/detect", self.ml_api_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                error!("Erro ao chamar a API de ML: {:?}", e);
                AppError::InternalServerError
            })?;

        // Analisar a resposta
        if response.status().is_success() {
            let detect_response: DetectionResponse = response.json().await.map_err(|e| {
                error!("Erro ao deserializar resposta da API de ML: {:?}", e);
                AppError::InternalServerError
            })?;

            Ok(ApiResponse::success(detect_response).into_response())
        } else {
            error!("API ML retornou erro: {}", response.status());
            Err(AppError::InternalServerError)
        }
    }

    pub async fn predict_tuberculosis(&self, image_data: Vec<u8>) -> Result<HttpResponse, AppError> {
        // Criar o formulário com a imagem
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(image_data)
                .file_name("image.jpg")
                .mime_str("image/jpeg").map_err(|e| {
                    error!("Erro ao definir MIME type: {:?}", e);
                    AppError::InternalServerError
                })?);

        // Fazer a requisição para a API Python
        let response = self.client.post(format!("{}/predict_tb", self.ml_api_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                error!("Erro ao chamar a API de ML: {:?}", e);
                AppError::InternalServerError
            })?;

        // Analisar a resposta
        if response.status().is_success() {
            let tb_response: TBResponse = response.json().await.map_err(|e| {
                error!("Erro ao deserializar resposta da API de ML: {:?}", e);
                AppError::InternalServerError
            })?;

            Ok(ApiResponse::success(tb_response.prediction_tb).into_response())
        } else {
            error!("API ML retornou erro: {}", response.status());
            Err(AppError::InternalServerError)
        }
    }
}