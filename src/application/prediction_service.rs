use actix_web::HttpResponse;
use reqwest::Client;
use anyhow::Result;
use log::error;

use crate::{
    domain::models::prediction::{DetectionResponse, OsteoporosisResponse, PredictionResponse, TBResponse}, 
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
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(image_data)
                .file_name("image.jpg")
                .mime_str("image/jpeg").map_err(|e| {
                    error!("Erro ao definir MIME type: {:?}", e);
                    AppError::InternalServerError
                })?);

        let response = self.client.post(format!("{}/predict", self.ml_api_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                error!("Erro ao chamar a API de ML: {:?}", e);
                AppError::InternalServerError
            })?;

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
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(image_data)
                .file_name("image.jpg")
                .mime_str("image/jpeg").map_err(|e| {
                    error!("Erro ao definir MIME type: {:?}", e);
                    AppError::InternalServerError
                })?);

        let response = self.client.post(format!("{}/detect", self.ml_api_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                error!("Erro ao chamar a API de ML: {:?}", e);
                AppError::InternalServerError
            })?;

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
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(image_data)
                .file_name("image.jpg")
                .mime_str("image/jpeg").map_err(|e| {
                    error!("Erro ao definir MIME type: {:?}", e);
                    AppError::InternalServerError
                })?);

        let response = self.client.post(format!("{}/predict_tb", self.ml_api_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                error!("Erro ao chamar a API de ML: {:?}", e);
                AppError::InternalServerError
            })?;

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


    pub async fn predict_osteoporosis(&self, image_data: Vec<u8>) -> Result<HttpResponse, AppError> {
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(image_data)
                .file_name("image.jpg")
                .mime_str("image/jpeg").map_err(|e| {
                    error!("Erro ao definir MIME type: {:?}", e);
                    AppError::InternalServerError
                })?);

        let response = self.client.post(format!("{}/osteoporosis", self.ml_api_url))
                .multipart(form)
                .send()
                .await
                .map_err(|e| {
                    error!("Erro ao chamar a API de ML: {:?}", e);
                    AppError::InternalServerError
                })?;

        let response_body = response.text().await.map_err(|e| {
            error!("Erro ao ler resposta como texto: {:?}", e);
            AppError::InternalServerError
        })?;
    
        // Primeiro convertemos para um valor JSON genérico
        let mut json_value: serde_json::Value = serde_json::from_str(&response_body).map_err(|e| {
            error!("Erro ao deserializar resposta da API de ML: {:?}", e);
            AppError::InternalServerError
        })?;
    
        // Acessamos os campos específicos e arredondamos
        if let Some(probabilities) = json_value
            .get_mut("prediction_osteoporosis")
            .and_then(|pred| pred.get_mut("probabilities")) 
        {
            if let Some(normal) = probabilities.get_mut("normal") {
                if let Some(n) = normal.as_f64() {
                    *normal = serde_json::Value::Number(
                        serde_json::Number::from_f64((n * 100.0).round() / 100.0).unwrap()
                    );
                }
            }
            
            if let Some(osteopenia) = probabilities.get_mut("osteopenia") {
                if let Some(n) = osteopenia.as_f64() {
                    *osteopenia = serde_json::Value::Number(
                        serde_json::Number::from_f64((n * 100.0).round() / 100.0).unwrap()
                    );
                }
            }
            
            if let Some(osteoporosis) = probabilities.get_mut("osteoporosis") {
                if let Some(n) = osteoporosis.as_f64() {
                    *osteoporosis = serde_json::Value::Number(
                        serde_json::Number::from_f64((n * 100.0).round() / 100.0).unwrap()
                    );
                }
            }
        }
    
        // Agora convertemos de volta para nossa estrutura tipada
        let osteoporosis_response: OsteoporosisResponse = 
            serde_json::from_value(json_value).map_err(|e| {
                error!("Erro ao converter JSON para estrutura: {:?}", e);
                AppError::InternalServerError
            })?;
            

        Ok(ApiResponse::success(osteoporosis_response.prediction_osteoporosis).into_response())
    }
}