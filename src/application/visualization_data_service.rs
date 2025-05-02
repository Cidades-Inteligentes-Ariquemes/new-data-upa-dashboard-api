use log::{error, info};
use std::collections::HashSet;
use actix_web::{web, HttpResponse};
use crate::domain::repositories::data_upa::DataRepository;
use crate::infrastructure::repositories::data_upa_repository::PgDataRepository;
use crate::utils::response::ApiResponse;
use crate::AppError;

pub struct VisualizationDataService {
    repo: web::Data<PgDataRepository>,
}

impl VisualizationDataService {
    pub fn new(repo: web::Data<PgDataRepository>) -> Self {
        Self { repo }
    }

    // Método auxiliar para verificar se existem dados para a unidade solicitada
    async fn verify_unit_data_exists(&self, table: &str, identifier: &str, unidade_id: Option<i32>) -> Result<(), AppError> {
        // Se unidade_id for None, assumimos unidade 2 (padrão)
        let unit_id = unidade_id.unwrap_or(2);
        
        // Verificamos se a unidade é válida (existe no sistema)
        let unidades = match self.repo.fetch_distinct_values("bpa", "ifrounidadeid").await {
            Ok(units) => units,
            Err(e) => {
                error!("Erro ao buscar unidades disponíveis: {}", e);
                return Err(AppError::DatabaseError(format!("Erro ao verificar unidades disponíveis: {}", e)));
            }
        };
        
        // Se a unidade solicitada não existir no sistema
        if !unidades.contains(&unit_id) {
            error!("Unidade {} não encontrada no sistema", unit_id);
            return Err(AppError::NotFound(format!("Unidade {} não encontrada no sistema", unit_id)));
        }
        
        // Verificamos se existem dados processados para esta unidade
        match self.repo.check_unit_data_exists(table, identifier, unit_id).await {
            Ok(exists) => {
                if !exists {
                    error!("Não há dados processados para a unidade {} em {}", unit_id, table);
                    return Err(AppError::NotFound(format!("Não há dados processados para a unidade {} em {}", unit_id, table)));
                }
                Ok(())
            },
            Err(e) => {
                error!("Erro ao verificar dados para unidade {}: {}", unit_id, e);
                Err(AppError::DatabaseError(format!("Erro ao verificar dados da unidade: {}", e)))
            }
        }
    }

    pub async fn number_of_appointments_per_month(&self, unidade_id: Option<i32>) -> Result<HttpResponse, AppError> {
        info!("Fetching number of appointments per month for unit {:?}", unidade_id);
        
        // Verificar se existem dados para esta unidade
        self.verify_unit_data_exists("number_of_appointments_per_month", "number_of_appointments_per_month", unidade_id).await?;
        
        match self.repo.fetch_nested_json("number_of_appointments_per_month", "number_of_appointments_per_month", unidade_id).await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching number of appointments per month for unit {:?}. Organized data is empty", unidade_id);
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Number of appointments per month fetched successfully for unit {:?}", unidade_id);
                Ok(ApiResponse::success(corrected_data).into_response())
            },
            Err(e) => {
                error!("Error fetching number of appointments per month for unit {:?}: {:?}", unidade_id, e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn number_of_appointments_per_year(&self, year: String, unidade_id: Option<i32>) -> Result<HttpResponse, AppError> {
        info!("Fetching number of appointments per year: {} for unit {:?}", year, unidade_id);
        
        // Verificar se existem dados para esta unidade
        self.verify_unit_data_exists("number_of_appointments_per_month", "number_of_appointments_per_month", unidade_id).await?;
        
        match self.repo.fetch_nested_json("number_of_appointments_per_month", "number_of_appointments_per_month", unidade_id).await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching number of appointments per year for unit {:?}. Organized data is empty", unidade_id);
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                // Filtrar apenas o ano especificado
                let filtered_data: serde_json::Map<String, serde_json::Value> = corrected_data
                    .into_iter()
                    .filter(|(key, _)| key.starts_with(&year))
                    .collect();
                
                if filtered_data.is_empty() {
                    error!("No data found for year {} in unit {:?}", year, unidade_id);
                    return Err(AppError::NotFound(format!("No data found for year {} in unit {:?}", year, unidade_id)));
                }
                
                // Ordenar por mês (formato: ano-mês)
                let mut sorted_data_vec: Vec<(String, serde_json::Value)> = filtered_data.into_iter().collect();
                sorted_data_vec.sort_by(|(a, _), (b, _)| {
                    let a_month = a.split('-').nth(1).unwrap_or("0").parse::<i32>().unwrap_or(0);
                    let b_month = b.split('-').nth(1).unwrap_or("0").parse::<i32>().unwrap_or(0);
                    a_month.cmp(&b_month)
                });
                
                let sorted_data: serde_json::Map<String, serde_json::Value> = sorted_data_vec.into_iter().collect();
                
                info!("Number of appointments per year fetched successfully for unit {:?}", unidade_id);
                Ok(ApiResponse::success(sorted_data).into_response())
            },
            Err(e) => {
                error!("Error fetching number of appointments per year for unit {:?}: {:?}", unidade_id, e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn years_available_for_number_of_appointments_per_month(&self, unidade_id: Option<i32>) -> Result<HttpResponse, AppError> {
        info!("Fetching years available for number of appointments per month for unit {:?}", unidade_id);
        
        // Verificar se existem dados para esta unidade
        self.verify_unit_data_exists("number_of_appointments_per_month", "number_of_appointments_per_month", unidade_id).await?;
        
        match self.repo.fetch_nested_json("number_of_appointments_per_month", "number_of_appointments_per_month", unidade_id).await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching years available for unit {:?}. Organized data is empty", unidade_id);
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                // Extrair os anos únicos
                let mut years = HashSet::new();
                for key in corrected_data.keys() {
                    if let Some(year) = key.split('-').next() {
                        years.insert(year.to_string());
                    }
                }
                
                // Verificar se encontramos anos
                if years.is_empty() {
                    error!("No years found for unit {:?}", unidade_id);
                    return Err(AppError::NotFound(format!("No years found for unit {:?}", unidade_id)));
                }
                
                // Ordenar os anos
                let mut years_vec: Vec<String> = years.into_iter().collect();
                years_vec.sort();
                
                info!("Years available fetched successfully for unit {:?}", unidade_id);
                Ok(ApiResponse::success(years_vec).into_response())
            },
            Err(e) => {
                error!("Error fetching years available for unit {:?}: {:?}", unidade_id, e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn number_of_appointments_per_flow(&self, unidade_id: Option<i32>) -> Result<HttpResponse, AppError> {
        info!("Fetching number of appointments per flow for unit {:?}", unidade_id);
        
        // Verificar se existem dados para esta unidade
        self.verify_unit_data_exists("number_of_appointments_per_flow", "number_of_appointments_per_flow", unidade_id).await?;
        
        match self.repo.fetch_nested_json("number_of_appointments_per_flow", "number_of_appointments_per_flow", unidade_id).await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching number of appointments per flow for unit {:?}. Organized data is empty", unidade_id);
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Number of appointments per flow fetched successfully for unit {:?}", unidade_id);
                Ok(ApiResponse::success(corrected_data).into_response())
            },
            Err(e) => {
                error!("Error fetching number of appointments per flow for unit {:?}: {:?}", unidade_id, e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn distribuition_of_patients_ages(&self, unidade_id: Option<i32>) -> Result<HttpResponse, AppError> {
        info!("Fetching distribuition of patients ages for unit {:?}", unidade_id);
        
        // Verificar se existem dados para esta unidade
        self.verify_unit_data_exists("distribuition_of_patients_ages", "distribuition_of_patients_ages", unidade_id).await?;
        
        match self.repo.fetch_nested_json("distribuition_of_patients_ages", "distribuition_of_patients_ages", unidade_id).await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching distribuition of patients ages for unit {:?}. Organized data is empty", unidade_id);
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Distribuition of patients ages fetched successfully for unit {:?}", unidade_id);
                Ok(ApiResponse::success(corrected_data).into_response())
            },
            Err(e) => {
                error!("Error fetching distribuition of patients ages for unit {:?}: {:?}", unidade_id, e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn number_of_calls_per_day_of_the_week(&self, unidade_id: Option<i32>) -> Result<HttpResponse, AppError> {
        info!("Fetching number of calls per day of the week for unit {:?}", unidade_id);
        
        // Verificar se existem dados para esta unidade
        self.verify_unit_data_exists("number_of_calls_per_day_of_the_week", "number_of_calls_per_day_of_the_week", unidade_id).await?;
        
        match self.repo.fetch_nested_json("number_of_calls_per_day_of_the_week", "number_of_calls_per_day_of_the_week", unidade_id).await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching number of calls per day of the week for unit {:?}. Organized data is empty", unidade_id);
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Number of calls per day of the week fetched successfully for unit {:?}", unidade_id);
                Ok(ApiResponse::success(corrected_data).into_response())
            },
            Err(e) => {
                error!("Error fetching number of calls per day of the week for unit {:?}: {:?}", unidade_id, e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn distribution_of_services_by_hour_group(&self, unidade_id: Option<i32>) -> Result<HttpResponse, AppError> {
        info!("Fetching distribution of services by hour group for unit {:?}", unidade_id);
        
        // Verificar se existem dados para esta unidade
        self.verify_unit_data_exists("distribution_of_services_by_hour_group", "distribution_of_services_by_hour_group", unidade_id).await?;
        
        match self.repo.fetch_nested_json("distribution_of_services_by_hour_group", "distribution_of_services_by_hour_group", unidade_id).await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching distribution of services by hour group for unit {:?}. Organized data is empty", unidade_id);
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Distribution of services by hour group fetched successfully for unit {:?}", unidade_id);
                Ok(ApiResponse::success(corrected_data).into_response())
            },
            Err(e) => {
                error!("Error fetching distribution of services by hour group for unit {:?}: {:?}", unidade_id, e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn number_of_visits_per_nurse(&self, unidade_id: Option<i32>) -> Result<HttpResponse, AppError> {
        info!("Fetching number of visits per nurse for unit {:?}", unidade_id);
        
        // Verificar se existem dados para esta unidade
        self.verify_unit_data_exists("number_of_visits_per_nurse", "number_of_visits_per_nurse", unidade_id).await?;
        
        match self.repo.fetch_nested_json("number_of_visits_per_nurse", "number_of_visits_per_nurse", unidade_id).await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching number of visits per nurse for unit {:?}. Organized data is empty", unidade_id);
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Number of visits per nurse fetched successfully for unit {:?}", unidade_id);
                Ok(ApiResponse::success(corrected_data).into_response())
            },
            Err(e) => {
                error!("Error fetching number of visits per nurse for unit {:?}: {:?}", unidade_id, e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn number_of_visits_per_doctor(&self, unidade_id: Option<i32>) -> Result<HttpResponse, AppError> {
        info!("Fetching number of visits per doctor for unit {:?}", unidade_id);
        
        // Verificar se existem dados para esta unidade
        self.verify_unit_data_exists("number_of_visits_per_doctor", "number_of_visits_per_doctor", unidade_id).await?;
        
        match self.repo.fetch_nested_json("number_of_visits_per_doctor", "number_of_visits_per_doctor", unidade_id).await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching number of visits per doctor for unit {:?}. Organized data is empty", unidade_id);
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Number of visits per doctor fetched successfully for unit {:?}", unidade_id);
                Ok(ApiResponse::success(corrected_data).into_response())
            },
            Err(e) => {
                error!("Error fetching number of visits per doctor for unit {:?}: {:?}", unidade_id, e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn average_time_in_minutes_per_doctor(&self, unidade_id: Option<i32>) -> Result<HttpResponse, AppError> {
        info!("Fetching average time in minutes per doctor for unit {:?}", unidade_id);
        
        // Verificar se existem dados para esta unidade
        self.verify_unit_data_exists("average_time_per_doctor", "average_time_per_doctor", unidade_id).await?;
        
        match self.repo.fetch_nested_json("average_time_per_doctor", "average_time_per_doctor", unidade_id).await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching average time in minutes per doctor for unit {:?}. Organized data is empty", unidade_id);
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Average time in minutes per doctor fetched successfully for unit {:?}", unidade_id);
                Ok(ApiResponse::success(corrected_data).into_response())
            },
            Err(e) => {
                error!("Error fetching average time in minutes per doctor for unit {:?}: {:?}", unidade_id, e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn heat_map_with_disease_indication(&self, unidade_id: Option<i32>) -> Result<HttpResponse, AppError> {
        // Para mapas de calor, sempre usar unidade 2
        let forced_unit_id = Some(2);
        info!("Fetching heat map with disease indication for unit {:?} (forcing unit 2)", unidade_id);
        
        // Verificar se existem dados para a unidade 2
        self.verify_unit_data_exists(
            "heat_map_with_disease_indication", 
            "heat_map_with_disease_indication", 
            forced_unit_id
        ).await?;
        
        match self.repo.fetch_nested_json(
            "heat_map_with_disease_indication", 
            "heat_map_with_disease_indication", 
            forced_unit_id
        ).await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching heat map with disease indication. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                // Se a unidade solicitada não é 2, informar que apenas a unidade 2 é suportada
                if unidade_id.is_some() && unidade_id.unwrap() != 2 {
                    info!("Unidade {} solicitada para mapa de calor, mas apenas a unidade 2 (UPA Ariquemes) possui estes dados", 
                           unidade_id.unwrap());
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Heat map with disease indication fetched successfully (unit 2)");
                Ok(ApiResponse::success(corrected_data).into_response())
            },
            Err(e) => {
                error!("Error fetching heat map with disease indication: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
    
    pub async fn heat_map_with_the_number_of_medical_appointments_by_neighborhood(&self, unidade_id: Option<i32>) -> Result<HttpResponse, AppError> {
        // Para mapas de calor, sempre usar unidade 2
        let forced_unit_id = Some(2);
        info!("Fetching heat map with appointments by neighborhood for unit {:?} (forcing unit 2)", unidade_id);
        
        // Verificar se existem dados para a unidade 2
        self.verify_unit_data_exists(
            "heat_map_with_the_number_of_medical_appointments_by_neighborhood", 
            "heat_map_with_the_number_of_medical_appointments_by_neighborhood", 
            forced_unit_id
        ).await?;
        
        match self.repo.fetch_nested_json(
            "heat_map_with_the_number_of_medical_appointments_by_neighborhood", 
            "heat_map_with_the_number_of_medical_appointments_by_neighborhood", 
            forced_unit_id
        ).await {
            Ok(data) => {
                if data.is_empty() {
                    error!("Error fetching heat map with appointments by neighborhood. Organized data is empty");
                    return Err(AppError::BadRequest("No data found".to_string()));
                }
                
                // Se a unidade solicitada não é 2, informar que apenas a unidade 2 é suportada
                if unidade_id.is_some() && unidade_id.unwrap() != 2 {
                    info!("Unidade {} solicitada para mapa de calor, mas apenas a unidade 2 (UPA Ariquemes) possui estes dados", 
                           unidade_id.unwrap());
                }
                
                let corrected_data = self.correct_keys(data);
                
                info!("Heat map with appointments by neighborhood fetched successfully (unit 2)");
                Ok(ApiResponse::success(corrected_data).into_response())
            },
            Err(e) => {
                error!("Error fetching heat map with appointments by neighborhood: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    // Função auxiliar para corrigir as chaves JSON
    fn correct_keys(&self, data: serde_json::Map<String, serde_json::Value>) -> serde_json::Map<String, serde_json::Value> {
        let mut corrected_data = serde_json::Map::new();
        
        for (key, value) in data {
            let corrected_key = key
                .replace("(", "")
                .replace(")", "")
                .replace("'", "")
                .replace(",", "");
            
            corrected_data.insert(corrected_key, value);
        }
        
        corrected_data
    }
}