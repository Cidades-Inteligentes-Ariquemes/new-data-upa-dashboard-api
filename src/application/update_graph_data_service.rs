use crate::domain::repositories::data_upa::DataRepository;
use crate::infrastructure::repositories::data_upa_repository::PgDataRepository;
use crate::utils::graph_data_processing::DataProcessingForGraphPlotting;
use crate::utils::process_data::create_dataframe_from_dict;
use crate::AppError;
use actix_web::{web, HttpResponse};
use log::{info, error};
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct UpdateGraphDataService {
    repo: web::Data<PgDataRepository>,
    data_processing: DataProcessingForGraphPlotting,
}

impl UpdateGraphDataService {
    pub fn new(repo: web::Data<PgDataRepository>) -> Self {
        Self { 
            repo, 
            data_processing: DataProcessingForGraphPlotting {},
        }
    }
    
    pub async fn update_data(&self) -> Result<HttpResponse, AppError> {
        info!("Atualizando dados para gráficos");
        
        let informations_to_plot = DataProcessingForGraphPlotting::columns_to_plot_graphs();
        
        // Definir a lista de parâmetros para processamento
        let list_params = vec![
            HashMap::from([
                ("table", Value::String(informations_to_plot["tables"]["bpa"].as_str().unwrap_or("bpa").to_string())),
                ("column", informations_to_plot["columns"]["number_of_appointments_per_month"].clone()),
                ("identifier", Value::String("number_of_appointments_per_month".to_string())),
                ("table_json", Value::String("number_of_appointments_per_month".to_string())),
                ("method", Value::String("create_dict_to_number_of_appointments_per_month".to_string()))
            ]),
            HashMap::from([
                ("table", Value::String(informations_to_plot["tables"]["bpa"].as_str().unwrap_or("bpa").to_string())),
                ("column", informations_to_plot["columns"]["number_of_appointments_per_flow"].clone()),
                ("identifier", Value::String("number_of_appointments_per_flow".to_string())),
                ("table_json", Value::String("number_of_appointments_per_flow".to_string())),
                ("method", Value::String("create_dict_to_number_of_appointments_per_flow".to_string()))
            ]),
            // Continuar com os demais parâmetros...
            HashMap::from([
                ("table", Value::String(informations_to_plot["tables"]["bpa"].as_str().unwrap_or("bpa").to_string())),
                ("column", informations_to_plot["columns"]["distribuition_of_patients_ages"].clone()),
                ("identifier", Value::String("distribuition_of_patients_ages".to_string())),
                ("table_json", Value::String("distribuition_of_patients_ages".to_string())),
                ("method", Value::String("create_dict_to_distribuition_of_patients_ages".to_string()))
            ]),
            HashMap::from([
                ("table", Value::String(informations_to_plot["tables"]["bpa"].as_str().unwrap_or("bpa").to_string())),
                ("column", informations_to_plot["columns"]["number_of_calls_per_day_of_the_week"].clone()),
                ("identifier", Value::String("number_of_calls_per_day_of_the_week".to_string())),
                ("table_json", Value::String("number_of_calls_per_day_of_the_week".to_string())),
                ("method", Value::String("create_dict_to_number_of_calls_per_day_of_the_week".to_string()))
            ]),
            HashMap::from([
                ("table", Value::String(informations_to_plot["tables"]["bpa"].as_str().unwrap_or("bpa").to_string())),
                ("column", informations_to_plot["columns"]["distribution_of_services_by_hour_group"].clone()),
                ("identifier", Value::String("distribution_of_services_by_hour_group".to_string())),
                ("table_json", Value::String("distribution_of_services_by_hour_group".to_string())),
                ("method", Value::String("create_dict_to_distribution_of_services_by_hour_group".to_string()))
            ]),
            HashMap::from([
                ("table", Value::String(informations_to_plot["tables"]["bpa"].as_str().unwrap_or("bpa").to_string())),
                ("column", informations_to_plot["columns"]["number_of_visits_per_nurse"].clone()),
                ("identifier", Value::String("number_of_visits_per_nurse".to_string())),
                ("table_json", Value::String("number_of_visits_per_nurse".to_string())),
                ("method", Value::String("create_dict_to_number_of_visits_per_nurse".to_string()))
            ]),
            HashMap::from([
                ("table", Value::String(informations_to_plot["tables"]["bpa"].as_str().unwrap_or("bpa").to_string())),
                ("column", informations_to_plot["columns"]["number_of_visits_per_doctor"].clone()),
                ("identifier", Value::String("number_of_visits_per_doctor".to_string())),
                ("table_json", Value::String("number_of_visits_per_doctor".to_string())),
                ("method", Value::String("create_dict_to_number_of_visits_per_doctor".to_string()))
            ]),
            HashMap::from([
                ("table", Value::String(informations_to_plot["tables"]["bpa"].as_str().unwrap_or("bpa").to_string())),
                ("column", informations_to_plot["columns"]["average_time_per_doctor"].clone()),
                ("identifier", Value::String("average_time_per_doctor".to_string())),
                ("table_json", Value::String("average_time_per_doctor".to_string())),
                ("method", Value::String("create_dict_to_average_time_in_minutes_per_doctor".to_string()))
            ]),
            HashMap::from([
                ("table", Value::String(informations_to_plot["tables"]["bpa"].as_str().unwrap_or("bpa").to_string())),
                ("column", informations_to_plot["columns"]["heat_map_with_disease_indication"].clone()),
                ("identifier", Value::String("heat_map_with_disease_indication".to_string())),
                ("table_json", Value::String("heat_map_with_disease_indication".to_string())),
                ("method", Value::String("create_dictionary_with_location_and_number_per_disease".to_string()))
            ]),
            HashMap::from([
                ("table", Value::String(informations_to_plot["tables"]["bpa"].as_str().unwrap_or("bpa").to_string())),
                ("column", informations_to_plot["columns"]["heat_map_with_the_number_of_medical_appointments_by_neighborhood"].clone()),
                ("identifier", Value::String("heat_map_with_the_number_of_medical_appointments_by_neighborhood".to_string())),
                ("table_json", Value::String("heat_map_with_the_number_of_medical_appointments_by_neighborhood".to_string())),
                ("method", Value::String("create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood".to_string()))
            ])
        ];

        // Buscar dados da tabela

        let result_dict = match self.repo.fetch_all_data("bpa").await {
            Ok(data) => data,
            Err(e) => {
                error!("Erro ao buscar dados da tabela {}: {}", "bpa", e);
                return Err(AppError::InternalServerError);
            }
        };
        
        // Processar cada parâmetro
        for params in list_params {
            //let table = params["table"].as_str().unwrap();
            let columns = params["column"].as_array().unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect::<Vec<String>>();
            let identifier = params["identifier"].as_str().unwrap();
            let table_json = params["table_json"].as_str().unwrap();
            let method_name = params["method"].as_str().unwrap();
            
            let filtered_data: HashMap<String, Vec<Value>> = result_dict.clone().into_iter()
                .filter(|(key, _)| columns.contains(key))
                .collect();

            if filtered_data.is_empty() {
                error!("Erro atualizando dados: nenhum dado encontrado");
                return Err(AppError::BadRequest("Nenhum dado encontrado".to_string()));
            }
            
            // Converter para DataFrame
            let df = match create_dataframe_from_dict(&filtered_data) {
                Ok(df) => df,
                Err(e) => {
                    error!("Erro ao criar DataFrame: {}", e);
                    return Err(AppError::InternalServerError);
                }
            };
            
            // Executar o método apropriado
            let organized_data = match method_name {
                "create_dict_to_number_of_appointments_per_month" => {
                    self.data_processing.create_dict_to_number_of_appointments_per_month(&df).await
                        .map_err(|e| { error!("Error processing data for {}: {}", method_name, e); AppError::InternalServerError })?
                },
                "create_dict_to_number_of_appointments_per_flow" => {
                    self.data_processing.create_dict_to_number_of_appointments_per_flow(&df).await
                        .map_err(|e| { error!("Error processing data for {}: {}", method_name, e); AppError::InternalServerError })?
                },
                "create_dict_to_distribuition_of_patients_ages" => {
                    self.data_processing.create_dict_to_distribuition_of_patients_ages(&df).await
                        .map_err(|e| { error!("Error processing data for {}: {}", method_name, e); AppError::InternalServerError })?
                },
                "create_dict_to_number_of_calls_per_day_of_the_week" => {
                    self.data_processing.create_dict_to_number_of_calls_per_day_of_the_week(&df).await
                        .map_err(|e| { error!("Error processing data for {}: {}", method_name, e); AppError::InternalServerError })?
                },
                "create_dict_to_distribution_of_services_by_hour_group" => {
                    self.data_processing.create_dict_to_distribution_of_services_by_hour_group(&df).await
                        .map_err(|e| { error!("Error processing data for {}: {}", method_name, e); AppError::InternalServerError })?
                },
                "create_dict_to_number_of_visits_per_nurse" => {
                    // Buscar dados adicionais
                    let non_nurse_table = informations_to_plot["tables"]["non_nurse"].as_str().unwrap();
                    let result_non_nurse = self.repo.fetch_all_data(non_nurse_table).await
                        .map_err(|e| { error!("Error fetching data for non_nurse: {}", e); AppError::InternalServerError })?;
                    let df_non_nurse = create_dataframe_from_dict(&result_non_nurse)
                        .map_err(|e| { error!("Error creating dataframe for non_nurse: {}", e); AppError::InternalServerError })?;
                    
                    self.data_processing.create_dict_to_number_of_visits_per_nurse(&df, &df_non_nurse).await
                        .map_err(|e| { error!("Error processing data for {}: {}", method_name, e); AppError::InternalServerError })?
                }, // Added closing brace and comma
                "create_dict_to_number_of_visits_per_doctor" => {
                    // Buscar dados adicionais
                    let non_doctors_table = informations_to_plot["tables"]["non_doctors"].as_str().unwrap();
                    let result_non_doctors = self.repo.fetch_all_data(non_doctors_table).await
                        .map_err(|e| { error!("Error fetching data for non_doctors: {}", e); AppError::InternalServerError })?;
                    let df_non_doctors = create_dataframe_from_dict(&result_non_doctors)
                        .map_err(|e| { error!("Error creating dataframe for non_doctors: {}", e); AppError::InternalServerError })?;
                    // Removed duplicate map_err line
                    
                    self.data_processing.create_dict_to_number_of_visits_per_doctor(&df, &df_non_doctors).await
                        .map_err(|e| { error!("Error processing data for {}: {}", method_name, e); AppError::InternalServerError })? // Added map_err and ?
                }, // Added closing brace and comma
                "create_dict_to_average_time_in_minutes_per_doctor" => {
                    // Buscar dados adicionais
                    let non_doctors_table = informations_to_plot["tables"]["non_doctors"].as_str().unwrap();
                    let result_non_doctors = self.repo.fetch_all_data(non_doctors_table).await
                        .map_err(|e| { error!("Error fetching data for non_doctors: {}", e); AppError::InternalServerError })?;
                    let df_non_doctors = create_dataframe_from_dict(&result_non_doctors)
                        .map_err(|e| { error!("Error creating dataframe for non_doctors: {}", e); AppError::InternalServerError })?;
                    // Removed duplicate map_err line
                    
                    self.data_processing.create_dict_to_average_time_in_minutes_per_doctor(&df, &df_non_doctors).await
                        .map_err(|e| { error!("Error processing data for {}: {}", method_name, e); AppError::InternalServerError })?
                },
                "create_dictionary_with_location_and_number_per_disease" => {
                    self.data_processing.create_dictionary_with_location_and_number_per_disease(&df).await
                        .map_err(|e| { error!("Error processing data for {}: {}", method_name, e); AppError::InternalServerError })?
                },
                "create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood" => {
                    self.data_processing.create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood(&df).await
                        .map_err(|e| { error!("Error processing data for {}: {}", method_name, e); AppError::InternalServerError })?
                },
                _ => {
                    error!("Método desconhecido: {}", method_name);
                    return Err(AppError::BadRequest(format!("Método desconhecido: {}", method_name)));
                }
            };
            
            if organized_data.is_null() {
                error!("Erro atualizando dados: dados organizados estão vazios para {}", identifier);
                return Err(AppError::InternalServerError);
            }
            
            // Inserir os dados processados
            let identifier_str = params["identifier"].as_str().ok_or_else(|| {
                error!("Identifier is not a string for method {}", method_name);
                AppError::InternalServerError
            })?;
            
        
            let result = match self.repo.insert_nested_json(organized_data, table_json, identifier_str).await {
                Ok(result) => result,
                Err(e) => {
                    error!("Erro ao inserir dados na tabela JSON: {}", e);
                    return Err(AppError::InternalServerError);
                }
                
            };
            
            if !result["added"].as_bool().unwrap_or(false) {
                error!("Erro ao inserir dados na tabela JSON para {}", identifier);
                return Err(AppError::InternalServerError);
            }
        }
        
        // Retornar sucesso
        let response = HttpResponse::Ok().json(json!({
            "detail": {
                "message": "Dados atualizados com sucesso",
                "status_code": 200
            }
        }));
        
        Ok(response)
    }
}