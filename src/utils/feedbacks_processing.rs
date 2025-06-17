use std::collections::HashMap;

use serde_json::json;

use crate::{domain::models::user::{DiseaseStats, FeedbackOsteoporosisResponse, FeedbackRespiratoryDiseasesResponse, FeedbackTuberculosisResponse, OsteoporosisStats}, utils::validators::{ALLOWED_FEEDBACKS_OSTEOPOROSIS, ALLOWED_RESPIRATORY_DISEASES}};

// Função auxiliar para processar estatísticas de tuberculose
pub fn process_tuberculosis_stats(feedbacks: &[FeedbackTuberculosisResponse]) -> TuberculosisStats {
    let total_quantity = feedbacks.len();
    let total_quantity_correct = feedbacks
        .iter()
        .filter(|f| f.feedback.to_lowercase() == "sim")
        .count();

    TuberculosisStats {
        total_quantity,
        total_quantity_correct,
    }
}

// Função auxiliar para processar estatísticas de doenças respiratórias
pub fn process_respiratory_stats(feedbacks: &[FeedbackRespiratoryDiseasesResponse]) -> HashMap<&'static str, DiseaseStats> {
    let mut stats = HashMap::new();
    
    // Inicializa contadores
    for disease in ALLOWED_RESPIRATORY_DISEASES {
        stats.insert(disease, DiseaseStats {
            total_quantity: 0,
            total_quantity_correct: 0,
        });
    }

    // Processa cada feedback
    for feedback in feedbacks {
        if let Some(stat) = stats.get_mut(feedback.prediction_made.as_str()) {
            stat.total_quantity += 1;
            if feedback.feedback.to_lowercase() == "sim" {
                stat.total_quantity_correct += 1;
            }
        }
    }

    stats
}

// Função auxiliar para processar estatísticas de osteoporose
pub fn process_osteoporosis_stats(feedbacks: &[FeedbackOsteoporosisResponse]) -> HashMap<&'static str, OsteoporosisStats> {
    let mut stats = HashMap::new();
    
    // Inicializa contadores para cada tipo de predição
    for prediction_type in ALLOWED_FEEDBACKS_OSTEOPOROSIS {
        stats.insert(prediction_type, OsteoporosisStats {
            total_quantity: 0,
            total_quantity_correct: 0,
        });
    }

    // Processa cada feedback
    for feedback in feedbacks {
        // CORREÇÃO: Agrupa por prediction_made, não por feedback
        if let Some(stat) = stats.get_mut(feedback.prediction_made.as_str()) {
            stat.total_quantity += 1;
            // Conta como correto se o feedback for "sim"
            if feedback.feedback.to_lowercase() == "sim" {
                stat.total_quantity_correct += 1;
            }
        }
    }

    stats
}

// Função auxiliar para construir a resposta final
pub fn build_final_response(
    respiratory_stats: HashMap<&'static str, DiseaseStats>,
    tuberculosis_stats: TuberculosisStats,
    osteoporosis_stats: HashMap<&'static str, OsteoporosisStats>,
) -> serde_json::Value {
    json!({
        "feedbacks_respiratory_diseases": {
            "normal": respiratory_stats.get("normal").unwrap_or(&DiseaseStats::default()),
            "covid-19": respiratory_stats.get("covid-19").unwrap_or(&DiseaseStats::default()),
            "pneumonia viral": respiratory_stats.get("pneumonia viral").unwrap_or(&DiseaseStats::default()),
            "pneumonia bacteriana": respiratory_stats.get("pneumonia bacteriana").unwrap_or(&DiseaseStats::default()),
        },
        "feedbacks_tuberculosis": {
            "total_quantity": tuberculosis_stats.total_quantity,
            "total_quantity_correct": tuberculosis_stats.total_quantity_correct
        },
        "feedbacks_osteoporosis": {
            "osteopenia": osteoporosis_stats.get("osteopenia").unwrap_or(&OsteoporosisStats::default()),
            "osteoporosis": osteoporosis_stats.get("osteoporosis").unwrap_or(&OsteoporosisStats::default()),
            "normal": osteoporosis_stats.get("normal").unwrap_or(&OsteoporosisStats::default()),
        }
    })
}

// Structs auxiliares (adicione estas ao seu código se não existirem)
#[derive(Debug, Clone)]
pub struct TuberculosisStats {
    pub total_quantity: usize,
    pub total_quantity_correct: usize,
}

impl Default for DiseaseStats {
    fn default() -> Self {
        Self {
            total_quantity: 0,
            total_quantity_correct: 0,
        }
    }
}

impl Default for OsteoporosisStats {
    fn default() -> Self {
        Self {
            total_quantity: 0,
            total_quantity_correct: 0,
        }
    }
}