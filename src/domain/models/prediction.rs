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
    pub negative: f64,
    pub positive: f64,
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


#[derive(Serialize, Deserialize)]
pub struct OsteoporosisProbabilities {
    pub normal: f64,
    pub osteopenia: f64,
    pub osteoporosis: f64,
}

#[derive(Serialize, Deserialize)]
pub struct OsteoporosisPredictionResponse {
    pub class_pred: String,
    pub probabilities: OsteoporosisProbabilities,
}



#[derive(Serialize, Deserialize)]
pub struct OsteoporosisResponse {
    pub prediction_osteoporosis: OsteoporosisPredictionResponse,
}