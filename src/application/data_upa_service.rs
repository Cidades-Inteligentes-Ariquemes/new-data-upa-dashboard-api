use std::collections::{HashMap, HashSet};
use std::time::Instant;

use actix_web::{web, HttpResponse};
use log::{error, info};
use polars::prelude::*;
use serde_json::{json, Value};

use crate::domain::{models::data_upa::HealthUnit, repositories::data_upa::DataRepository};
use crate::infrastructure::repositories::data_upa_repository::PgDataRepository;
use crate::utils::response::ApiResponse;
use crate::AppError;

use crate::utils::process_data::{
    add_week_day_and_split_date_time_polars, columns_names, create_dataframe,
    drop_column_if_exists, extract_keyword_hybrid, fill_null_strings, get_unique_values,
    normalize_column_names_of_the_df_to_lower_case, normalize_text_to_lower_case_columns_lazy,
    normalize_text_to_upper_case_columns_lazy, read_df_from_bytes, remove_unnecessary_columns,
    standardize_neighborhood_names,
};

pub struct DataUpaService {
    repo: web::Data<PgDataRepository>,
}

impl DataUpaService {
    pub fn new(repo: web::Data<PgDataRepository>) -> Self {
        Self { repo }
    }

    fn stage_start(request_id: &str, stage: &str) -> Instant {
        info!(
            "[request_id={}] data_upload.stage_start stage={}",
            request_id, stage
        );
        Instant::now()
    }

    fn stage_complete(
        request_id: &str,
        stage: &str,
        started_at: Instant,
        df: Option<&DataFrame>,
        details: Option<&str>,
    ) {
        let mut message = format!(
            "[request_id={}] data_upload.stage_complete stage={} duration_ms={}",
            request_id,
            stage,
            started_at.elapsed().as_millis()
        );

        if let Some(df) = df {
            let (rows, cols) = df.shape();
            message.push_str(&format!(" rows={} cols={}", rows, cols));
        }

        if let Some(details) = details {
            message.push(' ');
            message.push_str(details);
        }

        info!("{}", message);
    }

    async fn filter_dataframe_by_table_schema(
        &self,
        df: DataFrame,
        table_name: &str,
        request_id: &str,
    ) -> Result<DataFrame, AppError> {
        let table_columns = self
            .repo
            .fetch_table_columns(table_name)
            .await
            .map_err(|e| {
                error!(
                    "[request_id={}] Erro ao buscar colunas da tabela {} para filtrar o upload: {:?}",
                    request_id, table_name, e
                );
                AppError::DatabaseError(e.to_string())
            })?;

        let allowed_columns: HashSet<&str> = table_columns.iter().map(String::as_str).collect();
        let df_columns: Vec<String> = df
            .get_column_names()
            .iter()
            .map(|column| column.to_string())
            .collect();

        let columns_to_drop: Vec<String> = df_columns
            .iter()
            .filter(|column| !allowed_columns.contains(column.as_str()))
            .cloned()
            .collect();

        if !columns_to_drop.is_empty() {
            info!(
                "[request_id={}] Colunas descartadas do upload por não existirem no schema de {}: {:?}",
                request_id, table_name, columns_to_drop
            );
        }

        if df_columns.len() == columns_to_drop.len() {
            error!(
                "[request_id={}] Nenhuma coluna processada corresponde ao schema final da tabela {}",
                request_id, table_name
            );
            return Err(AppError::DatabaseError(format!(
                "Tabela {} sem colunas compatíveis para o upload",
                table_name
            )));
        }

        Ok(if columns_to_drop.is_empty() {
            df
        } else {
            df.drop_many(&columns_to_drop)
        })
    }

    pub async fn add_data(
        &self,
        file_content: web::Bytes,
        request_id: Option<&str>,
    ) -> Result<HttpResponse, AppError> {
        let request_id = request_id.unwrap_or("unknown");
        let total_started_at = Instant::now();

        info!(
            "[request_id={}] data_upload.start file_size_bytes={}",
            request_id,
            file_content.len()
        );

        let read_stage = Self::stage_start(request_id, "read_csv");
        let df = match read_df_from_bytes(&file_content) {
            Ok(df) => df,
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao ler arquivo CSV: {:?}",
                    request_id, e
                );
                return Err(AppError::BadRequest(
                    "Formato de arquivo inválido".to_string(),
                ));
            }
        };
        Self::stage_complete(request_id, "read_csv", read_stage, Some(&df), None);

        let (rows_before, cols_before) = df.shape();
        info!(
            "[request_id={}] Arquivo lido com sucesso: {} linhas, {} colunas",
            request_id, rows_before, cols_before
        );

        let remove_stage = Self::stage_start(request_id, "remove_unnecessary_columns");
        let colunas_desnecessarias = columns_names();
        let df_reduzido = match remove_unnecessary_columns(df.clone(), &colunas_desnecessarias) {
            Ok(df) => df,
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao remover colunas desnecessárias: {:?}",
                    request_id, e
                );
                return Err(AppError::InternalServerError);
            }
        };
        Self::stage_complete(
            request_id,
            "remove_unnecessary_columns",
            remove_stage,
            Some(&df_reduzido),
            None,
        );

        let transform_stage = Self::stage_start(request_id, "split_date_and_add_weekday");
        let df_transformed = match add_week_day_and_split_date_time_polars(df_reduzido) {
            Ok(df) => df,
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao adicionar dia da semana e separar data/hora: {:?}",
                    request_id, e
                );
                return Err(AppError::InternalServerError);
            }
        };
        Self::stage_complete(
            request_id,
            "split_date_and_add_weekday",
            transform_stage,
            Some(&df_transformed),
            None,
        );

        let normalize_stage = Self::stage_start(request_id, "normalize_text_columns");
        let colunas_para_normalizar = ["IfroConsultaConduta", "IfroPacienteBairro"];
        let df_normalizado = match normalize_text_to_lower_case_columns_lazy(
            df_transformed.lazy(),
            &colunas_para_normalizar,
        )
        .collect()
        {
            Ok(df) => df,
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao normalizar colunas de texto para minúsculas: {:?}",
                    request_id, e
                );
                return Err(AppError::InternalServerError);
            }
        };
        Self::stage_complete(
            request_id,
            "normalize_text_columns",
            normalize_stage,
            Some(&df_normalizado),
            None,
        );

        let classify_stage = Self::stage_start(request_id, "classify_main_complaints");
        let df_com_queixas = match extract_keyword_hybrid(&df_normalizado) {
            Ok(df) => df,
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao extrair e classificar queixas: {:?}",
                    request_id, e
                );
                return Err(AppError::InternalServerError);
            }
        };
        Self::stage_complete(
            request_id,
            "classify_main_complaints",
            classify_stage,
            Some(&df_com_queixas),
            None,
        );

        let drop_conduta_stage = Self::stage_start(request_id, "drop_ifroconsultaconduta");
        let colunas_remover = ["IfroConsultaConduta"];
        let df_com_queixas = match remove_unnecessary_columns(df_com_queixas, &colunas_remover) {
            Ok(df) => df,
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao remover coluna IfroConsultaConduta: {:?}",
                    request_id, e
                );
                return Err(AppError::InternalServerError);
            }
        };
        Self::stage_complete(
            request_id,
            "drop_ifroconsultaconduta",
            drop_conduta_stage,
            Some(&df_com_queixas),
            None,
        );

        let map_table_name = "map_neighbourhoods";
        let neighborhoods_stage = Self::stage_start(request_id, "load_neighborhood_mappings");
        let df_bairros = match self.repo.fetch_all_data(map_table_name).await {
            Ok(data) => {
                if data.is_empty() {
                    info!(
                        "[request_id={}] Tabela {} está vazia. Usando DataFrame vazio.",
                        request_id, map_table_name
                    );
                    DataFrame::default()
                } else {
                    info!(
                        "[request_id={}] Dados encontrados na tabela {}.",
                        request_id, map_table_name
                    );
                    match create_dataframe(&data) {
                        Ok(df) => df,
                        Err(e) => {
                            error!(
                                "[request_id={}] Erro ao criar DataFrame de bairros: {:?}",
                                request_id, e
                            );
                            return Err(AppError::InternalServerError);
                        }
                    }
                }
            }
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao buscar dados de bairros: {:?}",
                    request_id, e
                );
                return Err(AppError::InternalServerError);
            }
        };
        Self::stage_complete(
            request_id,
            "load_neighborhood_mappings",
            neighborhoods_stage,
            Some(&df_bairros),
            None,
        );

        let uppercase_stage = Self::stage_start(request_id, "normalize_neighborhoods_uppercase");
        let colunas_para_normalizar = ["IfroPacienteBairro"];
        let df_com_bairros_normalizados = match normalize_text_to_upper_case_columns_lazy(
            df_com_queixas.lazy(),
            &colunas_para_normalizar,
        )
        .collect()
        {
            Ok(df) => df,
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao normalizar nomes de bairros para maiúsculas: {:?}",
                    request_id, e
                );
                return Err(AppError::InternalServerError);
            }
        };
        Self::stage_complete(
            request_id,
            "normalize_neighborhoods_uppercase",
            uppercase_stage,
            Some(&df_com_bairros_normalizados),
            None,
        );

        let standardize_stage = Self::stage_start(request_id, "standardize_neighborhoods");
        let df_bairros_padronizados =
            match standardize_neighborhood_names(df_com_bairros_normalizados, df_bairros) {
                Ok(df) => df,
                Err(e) => {
                    error!(
                        "[request_id={}] Erro ao padronizar nomes de bairros: {:?}",
                        request_id, e
                    );
                    return Err(AppError::InternalServerError);
                }
            };
        Self::stage_complete(
            request_id,
            "standardize_neighborhoods",
            standardize_stage,
            Some(&df_bairros_padronizados),
            None,
        );

        let fill_null_stage = Self::stage_start(request_id, "fill_null_strings");
        let df_com_substituicoes = match fill_null_strings(df_bairros_padronizados) {
            Ok(df) => df,
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao substituir valores nulos: {:?}",
                    request_id, e
                );
                return Err(AppError::InternalServerError);
            }
        };
        Self::stage_complete(
            request_id,
            "fill_null_strings",
            fill_null_stage,
            Some(&df_com_substituicoes),
            None,
        );

        let drop_stage = Self::stage_start(request_id, "drop_unnamed_column");
        let colunas_excluir = "Unnamed: 0";
        let df_final_com_exclusao =
            match drop_column_if_exists(df_com_substituicoes, colunas_excluir) {
                Ok(df) => df,
                Err(e) => {
                    error!(
                        "[request_id={}] Erro ao excluir colunas desnecessárias: {:?}",
                        request_id, e
                    );
                    return Err(AppError::InternalServerError);
                }
            };
        Self::stage_complete(
            request_id,
            "drop_unnamed_column",
            drop_stage,
            Some(&df_final_com_exclusao),
            None,
        );

        let lowercase_stage = Self::stage_start(request_id, "normalize_column_names");
        let df_final_normalizado =
            match normalize_column_names_of_the_df_to_lower_case(df_final_com_exclusao) {
                Ok(df) => df,
                Err(e) => {
                    error!(
                        "[request_id={}] Erro ao normalizar nomes de colunas para minúsculas: {:?}",
                        request_id, e
                    );
                    return Err(AppError::InternalServerError);
                }
            };
        Self::stage_complete(
            request_id,
            "normalize_column_names",
            lowercase_stage,
            Some(&df_final_normalizado),
            None,
        );

        let table_name = "bpa";
        let schema_filter_stage = Self::stage_start(request_id, "filter_by_bpa_schema");
        let df_final_filtrado = self
            .filter_dataframe_by_table_schema(df_final_normalizado, table_name, request_id)
            .await?;
        Self::stage_complete(
            request_id,
            "filter_by_bpa_schema",
            schema_filter_stage,
            Some(&df_final_filtrado),
            None,
        );

        let duplicate_check_stage =
            Self::stage_start(request_id, "check_ifrocompetencia_duplicates");
        let competencia_values = match get_unique_values(&df_final_filtrado, "ifrocompetencia") {
            Ok(values) => values,
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao obter valores únicos de ifrocompetencia: {:?}",
                    request_id, e
                );
                return Err(AppError::InternalServerError);
            }
        };

        match self
            .repo
            .check_ifrocompetencia_exists(table_name, &competencia_values)
            .await
        {
            Ok(true) => {
                error!(
                    "[request_id={}] Dados com valores 'ifrocompetencia' {:?} já existem na tabela",
                    request_id, competencia_values
                );
                return Err(AppError::BadRequest(format!(
                    "Dados do período {} já existem no banco",
                    competencia_values.join(", ")
                )));
            }
            Ok(false) => {}
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao verificar duplicidade: {:?}",
                    request_id, e
                );
                return Err(AppError::InternalServerError);
            }
        };
        let duplicate_details = format!("competencia_values={:?}", competencia_values);
        Self::stage_complete(
            request_id,
            "check_ifrocompetencia_duplicates",
            duplicate_check_stage,
            None,
            Some(&duplicate_details),
        );

        let insert_stage = Self::stage_start(request_id, "insert_bpa_rows");
        match self
            .repo
            .insert_data(&df_final_filtrado, table_name, Some(request_id))
            .await
        {
            Ok(true) => {
                Self::stage_complete(
                    request_id,
                    "insert_bpa_rows",
                    insert_stage,
                    Some(&df_final_filtrado),
                    Some("table=bpa"),
                );
                let (rows, cols) = df_final_filtrado.shape();
                info!(
                    "[request_id={}] data_upload.finish table={} rows_processed={} columns_processed={} competencias={:?} duration_ms={}",
                    request_id,
                    table_name,
                    rows,
                    cols,
                    competencia_values,
                    total_started_at.elapsed().as_millis()
                );

                Ok(ApiResponse::created(json!({
                    "message": "Dados processados e importados com sucesso",
                    "rows_processed": rows,
                    "columns_processed": cols,
                    "competencia_values": competencia_values
                }))
                .into_response())
            }
            Ok(false) => {
                error!(
                    "[request_id={}] Falha ao inserir dados na tabela {}.",
                    request_id, table_name
                );
                Err(AppError::InternalServerError)
            }
            Err(e) => {
                error!(
                    "[request_id={}] Erro ao inserir dados na tabela {}: {:?}",
                    request_id, table_name, e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_available_health_units(&self) -> Result<HttpResponse, AppError> {
        let table_name = "bpa";
        let columns = vec!["ifrounidadeid".to_string(), "ifrounidadenome".to_string()];

        match self
            .repo
            .fetch_distinct_health_units(table_name, &columns)
            .await
        {
            Ok(data) => {
                let mut units = Vec::new();

                if let Some(ids) = data.get("ifrounidadeid") {
                    if let Some(names) = data.get("ifrounidadenome") {
                        for (id, name) in ids.iter().zip(names.iter()) {
                            let id_value = match id {
                                Value::Number(n) => {
                                    if let Some(i) = n.as_i64() {
                                        Some(i)
                                    } else {
                                        n.as_f64().map(|f| f as i64)
                                    }
                                }
                                _ => None,
                            };

                            let name_value = match name {
                                Value::String(s) => Some(s.clone()),
                                _ => None,
                            };

                            if let (Some(id), Some(name)) = (id_value, name_value) {
                                units.push(HealthUnit { id, name });
                            }
                        }
                    }
                }

                let mut unique_units: HashMap<i64, HealthUnit> = HashMap::new();
                for unit in units {
                    unique_units.insert(unit.id, unit);
                }

                let result: Vec<HealthUnit> = unique_units.into_values().collect();
                info!("Found {} unique health units", result.len());

                Ok(ApiResponse::success(result).into_response())
            }
            Err(e) => {
                error!("Error fetching available health units: {:?}", e);
                Err(AppError::DatabaseError(e.to_string()))
            }
        }
    }
}
