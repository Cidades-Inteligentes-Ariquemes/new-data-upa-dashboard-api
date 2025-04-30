use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct PredictionResponse {
    pub prediction: HashMap<String, f32>,
}

#[derive(Serialize, Deserialize)]
pub struct DetectionResult {
    pub class_id: i32,
    pub confidence: f32,
    pub bbox: [i32; 4],
}

#[derive(Serialize, Deserialize)]
pub struct DetectionResponse {
    pub detections: Vec<DetectionResult>,
    pub image: String,
}

#[derive(Serialize, Deserialize)]
pub struct TuberculosisProbabilities {
    pub negative: f32,
    pub positive: f32,
}

#[derive(Serialize, Deserialize)]
pub struct TuberculosisPredictionResponse {
    pub class_pred: String,
    pub probabilities: TuberculosisProbabilities,
}

#[derive(Serialize, Deserialize)]
pub struct TBResponse {
    pub prediction_tb: TuberculosisPredictionResponse,
}