use crate::domain::repositories::data_upa::DataRepository;
use crate::infrastructure::repositories::data_upa_repository::PgDataRepository;
use crate::utils::graph_data_processing::DataProcessingForGraphPlotting;
use crate::utils::process_data::create_dataframe_from_dict;
use crate::{ApiResponse, AppError};
use actix_web::{web, HttpResponse};
use log::{info, error};
use polars::frame::DataFrame;
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
        info!("Iniciando atualização de dados para gráficos");
        
        let list_params = vec![
            // Agendamentos por mês
            HashMap::from([
                ("table", Value::String("bpa".to_string())),
                ("column", json!(["ifrocompetencia"])),
                ("identifier", Value::String("number_of_appointments_per_month".to_string())),
                ("table_json", Value::String("number_of_appointments_per_month".to_string())),
                ("method", Value::String("create_dict_to_number_of_appointments_per_month".to_string()))
            ]),
            // Agendamentos por fluxo
            HashMap::from([
                ("table", Value::String("bpa".to_string())),
                ("column", json!(["ifrocompetencia", "ifrotabelanome"])),
                ("identifier", Value::String("number_of_appointments_per_flow".to_string())),
                ("table_json", Value::String("number_of_appointments_per_flow".to_string())),
                ("method", Value::String("create_dict_to_number_of_appointments_per_flow".to_string()))
            ]),
            // Distribuição de idades
            HashMap::from([
                ("table", Value::String("bpa".to_string())),
                ("column", json!(["ifrocompetencia", "ifropacienteidade"])),
                ("identifier", Value::String("distribuition_of_patients_ages".to_string())),
                ("table_json", Value::String("distribuition_of_patients_ages".to_string())),
                ("method", Value::String("create_dict_to_distribuition_of_patients_ages".to_string()))
            ]),
            // Chamadas por dia da semana
            HashMap::from([
                ("table", Value::String("bpa".to_string())),
                ("column", json!(["ifrocompetencia", "ifrodiasemana"])),
                ("identifier", Value::String("number_of_calls_per_day_of_the_week".to_string())),
                ("table_json", Value::String("number_of_calls_per_day_of_the_week".to_string())),
                ("method", Value::String("create_dict_to_number_of_calls_per_day_of_the_week".to_string()))
            ]),
            // Serviços por grupo horário
            HashMap::from([
                ("table", Value::String("bpa".to_string())),
                ("column", json!(["ifrocompetencia", "ifrohoraatendimento", "ifroprofissionalcbods"])),
                ("identifier", Value::String("distribution_of_services_by_hour_group".to_string())),
                ("table_json", Value::String("distribution_of_services_by_hour_group".to_string())),
                ("method", Value::String("create_dict_to_distribution_of_services_by_hour_group".to_string()))
            ]),
            // Visitas por enfermeiro
            HashMap::from([
                ("table", Value::String("bpa".to_string())),
                ("column", json!(["ifrocompetencia", "ifroprofissionalid", "ifroprofissionalcbods", "ifroprofissionalnome", "ifrotabelanome"])),
                ("identifier", Value::String("number_of_visits_per_nurse".to_string())),
                ("table_json", Value::String("number_of_visits_per_nurse".to_string())),
                ("method", Value::String("create_dict_to_number_of_visits_per_nurse".to_string()))
            ]),
            // Atendimentos por médico
            HashMap::from([
                ("table", Value::String("bpa".to_string())),
                ("column", json!(["ifrocompetencia", "ifroprofissionalid", "ifroprofissionalcbods", "ifroprofissionalnome", "ifrotabelanome"])),
                ("identifier", Value::String("number_of_visits_per_doctor".to_string())),
                ("table_json", Value::String("number_of_visits_per_doctor".to_string())),
                ("method", Value::String("create_dict_to_number_of_visits_per_doctor".to_string()))
            ]),
            // Tempo médio por médico
            HashMap::from([
                ("table", Value::String("bpa".to_string())),
                ("column", json!(["ifrocompetencia", "ifrohoraatendimento", "ifroprofissionalid", "ifroprofissionalcbods", "ifroprofissionalnome", "ifrotabelanome"])),
                ("identifier", Value::String("average_time_per_doctor".to_string())),
                ("table_json", Value::String("average_time_per_doctor".to_string())),
                ("method", Value::String("create_dict_to_average_time_in_minutes_per_doctor".to_string()))
            ]),
            // Mapa de calor por doença
            HashMap::from([
                ("table", Value::String("bpa".to_string())),
                ("column", json!(["ifrocompetencia", "ifropacienteendereco", "ifropacientebairro", "ifropacientequeixaprincipal", "ifropacientelatitude", "ifropacientelongitude"])),
                ("identifier", Value::String("heat_map_with_disease_indication".to_string())),
                ("table_json", Value::String("heat_map_with_disease_indication".to_string())),
                ("method", Value::String("create_dictionary_with_location_and_number_per_disease".to_string()))
            ]),
            // Mapa de calor por bairro
            HashMap::from([
                ("table", Value::String("bpa".to_string())),
                ("column", json!(["ifrocompetencia", "ifropacienteendereco", "ifropacientebairro", "ifropacientelatitude", "ifropacientelongitude"])),
                ("identifier", Value::String("heat_map_with_the_number_of_medical_appointments_by_neighborhood".to_string())),
                ("table_json", Value::String("heat_map_with_the_number_of_medical_appointments_by_neighborhood".to_string())),
                ("method", Value::String("create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood".to_string()))
            ])
        ];

        // Processa cada parâmetro
        for params in list_params {
            let table = params["table"].as_str().unwrap();
            let columns = params["column"].as_array().unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect::<Vec<String>>();
            let identifier = params["identifier"].as_str().unwrap();
            let table_json = params["table_json"].as_str().unwrap();
            let method_name = params["method"].as_str().unwrap();
            
            // Busca dados específicos
            let result_dict = match self.repo.fetch_columns_by_name(table, &columns).await {
                Ok(data) => data,
                Err(e) => {
                    error!("Erro ao buscar {} em {}: {}", identifier, table, e);
                    return Err(AppError::DatabaseError(e.to_string()));
                }
            };

            if result_dict.is_empty() {
                error!("Dados vazios para {}", identifier);
                return Err(AppError::NotFound(format!("Dados não encontrados para {}", identifier)));
            }

            // Cria DataFrame
            let df = match create_dataframe_from_dict(&result_dict) {
                Ok(df) => df,
                Err(e) => {
                    error!("Erro ao criar DataFrame para {}: {}", identifier, e);
                    return Err(AppError::DataProcessingError(e.to_string()));
                }
            };

            // Processamento condicional, pois necessita de outros dados além da tabela bpa
            let organized_data = match identifier {
                "number_of_visits_per_doctor" | "average_time_per_doctor" => {
                    let non_doctors = self.get_additional_data("non_doctors").await?;
                    self.call_processing_method(method_name, &df, Some(&non_doctors)).await?
                },
                "number_of_visits_per_nurse" => {
                    let non_nurse = self.get_additional_data("non_nurse").await?;
                    self.call_processing_method(method_name, &df, Some(&non_nurse)).await?
                },
                _ => self.call_processing_method(method_name, &df, None).await?,
            };

            // Salva dados
            if let Err(e) = self.save_processed_data(organized_data, table_json, identifier).await {
                error!("Falha ao salvar {}: {}", identifier, e);
                return Err(e);
            }
        }

        Ok(ApiResponse::updated(()).into_response())
    }

    // Funções Auxiliares 
    async fn get_additional_data(&self, table: &str) -> Result<DataFrame, AppError> {
        let data = self.repo.fetch_all_data(table)
            .await
            .map_err(|e| {
                error!("Erro ao buscar dados auxiliares da tabela {}: {}", table, e);
                AppError::DatabaseError(e.to_string())
            })?;

        create_dataframe_from_dict(&data)
            .map_err(|e| {
                error!("Erro ao criar DataFrame para dados auxiliares da tabela {}: {}", table, e);
                AppError::DataProcessingError(e.to_string())
            })
    }

    async fn call_processing_method(&self, method: &str, main_df: &DataFrame, additional_df: Option<&DataFrame>) -> Result<Value, AppError> {
        match (method, additional_df) {
            ("create_dict_to_number_of_appointments_per_month", None) =>
                self.data_processing.create_dict_to_number_of_appointments_per_month(main_df).await
                    .map_err(|e| { error!("Erro no método {}: {}", method, e); AppError::DataProcessingError(e.to_string()) }),
            ("create_dict_to_number_of_appointments_per_flow", None) =>
                self.data_processing.create_dict_to_number_of_appointments_per_flow(main_df).await
                    .map_err(|e| { error!("Erro no método {}: {}", method, e); AppError::DataProcessingError(e.to_string()) }),
            ("create_dict_to_distribuition_of_patients_ages", None) =>
                self.data_processing.create_dict_to_distribuition_of_patients_ages(main_df).await
                    .map_err(|e| { error!("Erro no método {}: {}", method, e); AppError::DataProcessingError(e.to_string()) }),
            ("create_dict_to_number_of_calls_per_day_of_the_week", None) =>
                self.data_processing.create_dict_to_number_of_calls_per_day_of_the_week(main_df).await
                    .map_err(|e| { error!("Erro no método {}: {}", method, e); AppError::DataProcessingError(e.to_string()) }),
            ("create_dict_to_distribution_of_services_by_hour_group", None) =>
                self.data_processing.create_dict_to_distribution_of_services_by_hour_group(main_df).await
                    .map_err(|e| { error!("Erro no método {}: {}", method, e); AppError::DataProcessingError(e.to_string()) }),
            ("create_dict_to_number_of_visits_per_nurse", Some(df)) =>
                self.data_processing.create_dict_to_number_of_visits_per_nurse(main_df, df).await
                    .map_err(|e| { error!("Erro no método {}: {}", method, e); AppError::DataProcessingError(e.to_string()) }),
            ("create_dict_to_number_of_visits_per_doctor", Some(df)) =>
                self.data_processing.create_dict_to_number_of_visits_per_doctor(main_df, df).await
                    .map_err(|e| { error!("Erro no método {}: {}", method, e); AppError::DataProcessingError(e.to_string()) }),
            ("create_dict_to_average_time_in_minutes_per_doctor", Some(df)) =>
                self.data_processing.create_dict_to_average_time_in_minutes_per_doctor(main_df, df).await
                    .map_err(|e| { error!("Erro no método {}: {}", method, e); AppError::DataProcessingError(e.to_string()) }),
            ("create_dictionary_with_location_and_number_per_disease", None) =>
                self.data_processing.create_dictionary_with_location_and_number_per_disease(main_df).await
                    .map_err(|e| { error!("Erro no método {}: {}", method, e); AppError::DataProcessingError(e.to_string()) }),
            ("create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood", None) =>
                self.data_processing.create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood(main_df).await
                    .map_err(|e| { error!("Erro no método {}: {}", method, e); AppError::DataProcessingError(e.to_string()) }),
            _ => Err(AppError::InvalidMethodError(format!("Método '{}' inválido ou dados adicionais incorretos", method)))
        }
    }

    async fn save_processed_data(&self, data: Value, table: &str, identifier: &str) -> Result<(), AppError> {
        self.repo.insert_nested_json(data, table, identifier)
            .await
            .map(|_| ())
            .map_err(|e| {
                error!("Erro ao salvar em {}: {}", table, e);
                AppError::DatabaseError(e.to_string()) 
            })
    }
}