use crate::domain::repositories::data_upa::DataRepository;
use crate::infrastructure::repositories::data_upa_repository::PgDataRepository;
use crate::utils::graph_data_processing::DataProcessingForGraphPlotting;
use crate::utils::process_data::create_dataframe_from_dict;
use crate::{ApiResponse, AppError};
use actix_web::{web, HttpResponse};
use log::{error, info};
use polars::frame::DataFrame;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Instant;

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

    pub async fn update_data(&self, request_id: Option<&str>) -> Result<HttpResponse, AppError> {
        let request_id = request_id.unwrap_or("unknown");
        let total_started_at = Instant::now();

        info!("[request_id={}] graph_update.start", request_id);

        // Buscar todas as unidades disponíveis na tabela bpa
        let fetch_units_started_at = Instant::now();
        let unidades = match self
            .repo
            .fetch_distinct_values("bpa", "ifrounidadeid")
            .await
        {
            Ok(unidades) => {
                if unidades.is_empty() {
                    info!(
                        "[request_id={}] Nenhuma unidade encontrada, usando unidade padrão (2)",
                        request_id
                    );
                    vec![2] // Unidade padrão (UPA Ariquemes)
                } else {
                    unidades
                }
            }
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao buscar unidades disponíveis: {}",
                    request_id, e
                );
                // Fallback para unidade padrão
                vec![2]
            }
        };

        info!(
            "[request_id={}] graph_update.units_loaded count={} duration_ms={} units={:?}",
            request_id,
            unidades.len(),
            fetch_units_started_at.elapsed().as_millis(),
            unidades
        );

        // Para cada unidade, processar todos os gráficos
        for unidade_id in unidades {
            let unit_started_at = Instant::now();
            info!(
                "[request_id={}] graph_update.unit_start unit_id={}",
                request_id, unidade_id
            );

            // Lista base de parâmetros
            let mut list_params = vec![
                // Agendamentos por mês
                HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!(["ifrocompetencia", "ifrotabelaid", "ifrotabelanome"]),
                    ),
                    (
                        "identifier",
                        Value::String("number_of_appointments_per_month".to_string()),
                    ),
                    (
                        "table_json",
                        Value::String("number_of_appointments_per_month".to_string()),
                    ),
                    (
                        "method",
                        Value::String(
                            "create_dict_to_number_of_appointments_per_month".to_string(),
                        ),
                    ),
                ]),
                // Agendamentos por fluxo
                HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!(["ifrocompetencia", "ifrotabelanome", "ifrotabelaid"]),
                    ),
                    (
                        "identifier",
                        Value::String("number_of_appointments_per_flow".to_string()),
                    ),
                    (
                        "table_json",
                        Value::String("number_of_appointments_per_flow".to_string()),
                    ),
                    (
                        "method",
                        Value::String("create_dict_to_number_of_appointments_per_flow".to_string()),
                    ),
                ]),
                // Distribuição de idades
                HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!([
                            "ifrocompetencia",
                            "ifropacienteidade",
                            "ifrotabelaid",
                            "ifrotabelanome"
                        ]),
                    ),
                    (
                        "identifier",
                        Value::String("distribuition_of_patients_ages".to_string()),
                    ),
                    (
                        "table_json",
                        Value::String("distribuition_of_patients_ages".to_string()),
                    ),
                    (
                        "method",
                        Value::String("create_dict_to_distribuition_of_patients_ages".to_string()),
                    ),
                ]),
                // Chamadas por dia da semana
                HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!([
                            "ifrocompetencia",
                            "ifrodiasemana",
                            "ifrotabelaid",
                            "ifrotabelanome"
                        ]),
                    ),
                    (
                        "identifier",
                        Value::String("number_of_calls_per_day_of_the_week".to_string()),
                    ),
                    (
                        "table_json",
                        Value::String("number_of_calls_per_day_of_the_week".to_string()),
                    ),
                    (
                        "method",
                        Value::String(
                            "create_dict_to_number_of_calls_per_day_of_the_week".to_string(),
                        ),
                    ),
                ]),
                // Serviços por grupo horário
                HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!([
                            "ifrocompetencia",
                            "ifrohoraatendimento",
                            "ifroprofissionalcbods",
                            "ifrotabelaid",
                            "ifrotabelanome"
                        ]),
                    ),
                    (
                        "identifier",
                        Value::String("distribution_of_services_by_hour_group".to_string()),
                    ),
                    (
                        "table_json",
                        Value::String("distribution_of_services_by_hour_group".to_string()),
                    ),
                    (
                        "method",
                        Value::String(
                            "create_dict_to_distribution_of_services_by_hour_group".to_string(),
                        ),
                    ),
                ]),
                // Visitas por enfermeiro
                HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!([
                            "ifrocompetencia",
                            "ifroprofissionalid",
                            "ifroprofissionalcbods",
                            "ifroprofissionalnome",
                            "ifrotabelanome",
                            "ifrotabelaid"
                        ]),
                    ),
                    (
                        "identifier",
                        Value::String("number_of_visits_per_nurse".to_string()),
                    ),
                    (
                        "table_json",
                        Value::String("number_of_visits_per_nurse".to_string()),
                    ),
                    (
                        "method",
                        Value::String("create_dict_to_number_of_visits_per_nurse".to_string()),
                    ),
                ]),
                // Atendimentos por médico
                HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!([
                            "ifrocompetencia",
                            "ifroprofissionalid",
                            "ifroprofissionalcbods",
                            "ifroprofissionalnome",
                            "ifrotabelanome",
                            "ifrotabelaid"
                        ]),
                    ),
                    (
                        "identifier",
                        Value::String("number_of_visits_per_doctor".to_string()),
                    ),
                    (
                        "table_json",
                        Value::String("number_of_visits_per_doctor".to_string()),
                    ),
                    (
                        "method",
                        Value::String("create_dict_to_number_of_visits_per_doctor".to_string()),
                    ),
                ]),
                // Atendimentos sem consulta médica
                HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!(["ifrocompetencia", "ifrotabelaid", "ifrotabelanome"]),
                    ),
                    (
                        "identifier",
                        Value::String(
                            "number_of_appointments_without_medical_consultation".to_string(),
                        ),
                    ),
                    (
                        "table_json",
                        Value::String(
                            "number_of_appointments_without_medical_consultation".to_string(),
                        ),
                    ),
                    (
                        "method",
                        Value::String(
                            "create_dict_to_number_of_appointments_without_medical_consultation"
                                .to_string(),
                        ),
                    ),
                ]),
                // Tempo médio por médico
                HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!([
                            "ifrocompetencia",
                            "ifrodataatendimento",
                            "ifrohoraatendimento",
                            "ifroprofissionalid",
                            "ifroprofissionalcbods",
                            "ifroprofissionalnome",
                            "ifrotabelanome"
                        ]),
                    ),
                    (
                        "identifier",
                        Value::String("average_time_per_doctor".to_string()),
                    ),
                    (
                        "table_json",
                        Value::String("average_time_per_doctor".to_string()),
                    ),
                    (
                        "method",
                        Value::String(
                            "create_dict_to_average_time_in_minutes_per_doctor".to_string(),
                        ),
                    ),
                ]),
                // Atendimentos por CID
                HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!([
                            "ifrocompetencia",
                            "ifrocidcd",
                            "ifrodataatendimento",
                            "ifrotabelaid",
                            "ifrotabelanome"
                        ]),
                    ),
                    (
                        "identifier",
                        Value::String("number_of_appointments_per_cid".to_string()),
                    ),
                    (
                        "table_json",
                        Value::String("number_of_appointments_per_cid".to_string()),
                    ),
                    (
                        "method",
                        Value::String("create_dict_to_number_of_appointments_per_cid".to_string()),
                    ),
                ]),
                // Atendimentos por classificação
                HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!([
                            "ifrocompetencia",
                            "ifroclassificacao",
                            "ifrodataatendimento",
                            "ifrotabelaid",
                            "ifrotabelanome"
                        ]),
                    ),
                    (
                        "identifier",
                        Value::String("number_of_appointments_per_classification".to_string()),
                    ),
                    (
                        "table_json",
                        Value::String("number_of_appointments_per_classification".to_string()),
                    ),
                    (
                        "method",
                        Value::String(
                            "create_dict_to_number_of_appointments_per_classification".to_string(),
                        ),
                    ),
                ]),
                // Atendimentos médicos por classificação
                HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!([
                            "ifrocompetencia",
                            "ifroclassificacao",
                            "ifroprofissionalcbods",
                            "ifrotabelanome",
                            "ifrodataatendimento",
                            "ifrotabelaid"
                        ]),
                    ),
                    (
                        "identifier",
                        Value::String(
                            "number_of_medical_appointments_per_classification".to_string(),
                        ),
                    ),
                    (
                        "table_json",
                        Value::String(
                            "number_of_medical_appointments_per_classification".to_string(),
                        ),
                    ),
                    (
                        "method",
                        Value::String(
                            "create_dict_to_number_of_medical_appointments_per_classification"
                                .to_string(),
                        ),
                    ),
                ]),
            ];

            // Adiciona os parâmetros de mapas de calor se a unidade for diferente de 3
            if unidade_id != 3 {
                // Mapa de calor por doença
                list_params.push(HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!([
                            "ifrocompetencia",
                            "ifropacienteendereco",
                            "ifropacientebairro",
                            "ifropacientequeixaprincipal",
                            "ifropacientelatitude",
                            "ifropacientelongitude",
                            "ifrotabelaid",
                            "ifrotabelanome"
                        ]),
                    ),
                    (
                        "identifier",
                        Value::String("heat_map_with_disease_indication".to_string()),
                    ),
                    (
                        "table_json",
                        Value::String("heat_map_with_disease_indication".to_string()),
                    ),
                    (
                        "method",
                        Value::String(
                            "create_dictionary_with_location_and_number_per_disease".to_string(),
                        ),
                    ),
                ]));

                // Mapa de calor por bairro
                list_params.push(HashMap::from([
                    ("table", Value::String("bpa".to_string())),
                    (
                        "column",
                        json!([
                            "ifrocompetencia",
                            "ifropacienteendereco",
                            "ifropacientebairro",
                            "ifropacientelatitude",
                            "ifropacientelongitude",
                            "ifrotabelaid",
                            "ifrotabelanome"
                        ]),
                    ),
                    ("identifier", Value::String("heat_map_with_the_number_of_medical_appointments_by_neighborhood".to_string())),
                    ("table_json", Value::String("heat_map_with_the_number_of_medical_appointments_by_neighborhood".to_string())),
                    ("method", Value::String("create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood".to_string()))
                ]));

                info!(
                    "[request_id={}] graph_update.heat_maps_enabled unit_id={}",
                    request_id, unidade_id
                );
            } else {
                info!(
                    "[request_id={}] graph_update.heat_maps_skipped unit_id={}",
                    request_id, unidade_id
                );
            }

            // Processa cada parâmetro para a unidade atual
            for params in &list_params {
                let table = params["table"].as_str().unwrap();
                let columns = params["column"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect::<Vec<String>>();
                let identifier = params["identifier"].as_str().unwrap();
                let table_json = params["table_json"].as_str().unwrap();
                let method_name = params["method"].as_str().unwrap();
                let metric_started_at = Instant::now();

                info!(
                    "[request_id={}] graph_update.metric_start unit_id={} identifier={} source_table={} columns={:?}",
                    request_id, unidade_id, identifier, table, columns
                );

                // Busca dados específicos para esta unidade
                let result_dict = match self
                    .repo
                    .fetch_columns_by_name_with_filter(table, &columns, "ifrounidadeid", unidade_id)
                    .await
                {
                    Ok(data) => data,
                    Err(e) => {
                        error!(
                            "[request_id={}] Erro ao buscar {} em {} para unidade {}: {}",
                            request_id, identifier, table, unidade_id, e
                        );
                        // Continua com o próximo parâmetro em vez de falhar completamente
                        continue;
                    }
                };

                if result_dict.is_empty() {
                    error!(
                        "[request_id={}] Dados vazios para {} na unidade {}",
                        request_id, identifier, unidade_id
                    );
                    continue; // Passa para o próximo parâmetro
                }

                if identifier == "distribuition_of_patients_ages" {
                    let organized_data = self.data_processing.create_dict_to_distribuition_of_patients_ages_from_raw(
                        &result_dict
                    ).await.map_err(|e| {
                        error!(
                            "[request_id={}] Erro no método create_dict_to_distribuition_of_patients_ages_from_raw: {}",
                            request_id, e
                        );
                        AppError::DataProcessingError(e.to_string())
                    })?;

                    if let Err(e) = self
                        .save_processed_data_with_unit(
                            organized_data,
                            table_json,
                            identifier,
                            unidade_id,
                        )
                        .await
                    {
                        error!(
                            "[request_id={}] Falha ao salvar {} para unidade {}: {}",
                            request_id, identifier, unidade_id, e
                        );
                        // Continua com o próximo parâmetro
                        continue;
                    }
                } else {
                    // Criar DataFrame e processar outros casos normalmente
                    let df = match create_dataframe_from_dict(&result_dict) {
                        Ok(df) => df,
                        Err(e) => {
                            error!(
                                "[request_id={}] Erro ao criar DataFrame para {} na unidade {}: {}",
                                request_id, identifier, unidade_id, e
                            );
                            continue;
                        }
                    };

                    // Processamento condicional
                    let organized_data = match identifier {
                        "number_of_visits_per_doctor" | "average_time_per_doctor" => {
                            let non_doctors = self.get_additional_data("non_doctors").await?;
                            self.call_processing_method(
                                method_name,
                                &df,
                                Some(&non_doctors),
                                unidade_id,
                            )
                            .await?
                        }
                        "number_of_visits_per_nurse" => {
                            let non_nurse = self.get_additional_data("non_nurse").await?;
                            self.call_processing_method(
                                method_name,
                                &df,
                                Some(&non_nurse),
                                unidade_id,
                            )
                            .await?
                        }
                        _ => {
                            self.call_processing_method(method_name, &df, None, unidade_id)
                                .await?
                        }
                    };

                    // Salva dados incluindo o id da unidade
                    if let Err(e) = self
                        .save_processed_data_with_unit(
                            organized_data,
                            table_json,
                            identifier,
                            unidade_id,
                        )
                        .await
                    {
                        error!(
                            "[request_id={}] Falha ao salvar {} para unidade {}: {}",
                            request_id, identifier, unidade_id, e
                        );
                        // Continua com o próximo parâmetro
                        continue;
                    }
                }

                info!(
                    "[request_id={}] graph_update.metric_complete unit_id={} identifier={} duration_ms={}",
                    request_id,
                    unidade_id,
                    identifier,
                    metric_started_at.elapsed().as_millis()
                );
            }

            info!(
                "[request_id={}] graph_update.unit_complete unit_id={} duration_ms={}",
                request_id,
                unidade_id,
                unit_started_at.elapsed().as_millis()
            );
        }

        info!(
            "[request_id={}] graph_update.finish duration_ms={}",
            request_id,
            total_started_at.elapsed().as_millis()
        );

        Ok(ApiResponse::updated(()).into_response())
    }

    // Funções Auxiliares
    async fn get_additional_data(&self, table: &str) -> Result<DataFrame, AppError> {
        let data = self.repo.fetch_all_data(table).await.map_err(|e| {
            error!("Erro ao buscar dados auxiliares da tabela {}: {}", table, e);
            AppError::DatabaseError(e.to_string())
        })?;

        create_dataframe_from_dict(&data).map_err(|e| {
            error!(
                "Erro ao criar DataFrame para dados auxiliares da tabela {}: {}",
                table, e
            );
            AppError::DataProcessingError(e.to_string())
        })
    }

    async fn call_processing_method(
        &self,
        method: &str,
        main_df: &DataFrame,
        additional_df: Option<&DataFrame>,
        unidade_id: i32,
    ) -> Result<Value, AppError> {
        match (method, additional_df) {
            ("create_dict_to_number_of_appointments_per_month", None) => self
                .data_processing
                .create_dict_to_number_of_appointments_per_month(main_df)
                .await
                .map_err(|e| {
                    error!("Erro no método {}: {}", method, e);
                    AppError::DataProcessingError(e.to_string())
                }),
            ("create_dict_to_number_of_appointments_per_flow", None) => self
                .data_processing
                .create_dict_to_number_of_appointments_per_flow(main_df)
                .await
                .map_err(|e| {
                    error!("Erro no método {}: {}", method, e);
                    AppError::DataProcessingError(e.to_string())
                }),
            // ("create_dict_to_distribuition_of_patients_ages", None) =>
            //     self.data_processing.create_dict_to_distribuition_of_patients_ages_from_raw(main_df).await
            //         .map_err(|e| { error!("Erro no método {}: {}", method, e); AppError::DataProcessingError(e.to_string()) }),
            ("create_dict_to_number_of_calls_per_day_of_the_week", None) => self
                .data_processing
                .create_dict_to_number_of_calls_per_day_of_the_week(main_df)
                .await
                .map_err(|e| {
                    error!("Erro no método {}: {}", method, e);
                    AppError::DataProcessingError(e.to_string())
                }),
            ("create_dict_to_distribution_of_services_by_hour_group", None) => self
                .data_processing
                .create_dict_to_distribution_of_services_by_hour_group(main_df)
                .await
                .map_err(|e| {
                    error!("Erro no método {}: {}", method, e);
                    AppError::DataProcessingError(e.to_string())
                }),
            ("create_dict_to_number_of_visits_per_nurse", Some(df)) => self
                .data_processing
                .create_dict_to_number_of_visits_per_nurse(main_df, df)
                .await
                .map_err(|e| {
                    error!("Erro no método {}: {}", method, e);
                    AppError::DataProcessingError(e.to_string())
                }),
            ("create_dict_to_number_of_visits_per_doctor", Some(df)) => self
                .data_processing
                .create_dict_to_number_of_visits_per_doctor(main_df, df)
                .await
                .map_err(|e| {
                    error!("Erro no método {}: {}", method, e);
                    AppError::DataProcessingError(e.to_string())
                }),
            ("create_dict_to_number_of_appointments_without_medical_consultation", None) => self
                .data_processing
                .create_dict_to_number_of_appointments_without_medical_consultation(main_df)
                .await
                .map_err(|e| {
                    error!("Erro no método {}: {}", method, e);
                    AppError::DataProcessingError(e.to_string())
                }),
            ("create_dict_to_average_time_in_minutes_per_doctor", Some(df)) => self
                .data_processing
                .create_dict_to_average_time_in_minutes_per_doctor(main_df, df)
                .await
                .map_err(|e| {
                    error!("Erro no método {}: {}", method, e);
                    AppError::DataProcessingError(e.to_string())
                }),
            ("create_dictionary_with_location_and_number_per_disease", None) => self
                .data_processing
                .create_dictionary_with_location_and_number_per_disease(main_df)
                .await
                .map_err(|e| {
                    error!("Erro no método {}: {}", method, e);
                    AppError::DataProcessingError(e.to_string())
                }),
            (
                "create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood",
                None,
            ) => self
                .data_processing
                .create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood(
                    main_df, unidade_id,
                )
                .await
                .map_err(|e| {
                    error!("Erro no método {}: {}", method, e);
                    AppError::DataProcessingError(e.to_string())
                }),
            ("create_dict_to_number_of_appointments_per_cid", None) => self
                .data_processing
                .create_dict_to_number_of_appointments_per_cid(main_df)
                .await
                .map_err(|e| {
                    error!("Erro no método {}: {}", method, e);
                    AppError::DataProcessingError(e.to_string())
                }),
            ("create_dict_to_number_of_appointments_per_classification", None) => self
                .data_processing
                .create_dict_to_number_of_appointments_per_classification(main_df)
                .await
                .map_err(|e| {
                    error!("Erro no método {}: {}", method, e);
                    AppError::DataProcessingError(e.to_string())
                }),
            ("create_dict_to_number_of_medical_appointments_per_classification", None) => self
                .data_processing
                .create_dict_to_number_of_medical_appointments_per_classification(main_df)
                .await
                .map_err(|e| {
                    error!("Erro no método {}: {}", method, e);
                    AppError::DataProcessingError(e.to_string())
                }),
            _ => Err(AppError::InvalidMethodError(format!(
                "Método '{}' inválido ou dados adicionais incorretos",
                method
            ))),
        }
    }

    // Novo método para salvar com unidade
    async fn save_processed_data_with_unit(
        &self,
        data: Value,
        table: &str,
        identifier: &str,
        unidade_id: i32,
    ) -> Result<(), AppError> {
        self.repo
            .insert_nested_json_with_unit(data, table, identifier, unidade_id)
            .await
            .map(|_| ())
            .map_err(|e| {
                error!(
                    "Erro ao salvar em {} para unidade {}: {}",
                    table, unidade_id, e
                );
                AppError::DatabaseError(e.to_string())
            })
    }
}
