use actix_web::{web, HttpResponse};
use log::{error, info};
use serde_json::json;
use polars::prelude::*;

use crate::domain::repositories::data_upa::DataRepository;
use crate::infrastructure::repositories::data_upa_repository::PgDataRepository;
use crate::utils::response::ApiResponse;
use crate::AppError;

// Importe as funções de processamento de dados
use crate::utils::process_data::{
    columns_names,
    remove_unnecessary_columns,
    add_week_day_and_split_date_time_polars,
    normalize_text_to_lower_case_columns_lazy,
    extract_keyword_hybrid,
    normalize_text_to_upper_case_columns_lazy,
    standardize_neighborhood_names,
    fill_null_strings,
    drop_column_if_exists,
    normalize_column_names_of_the_df_to_lower_case,
    get_unique_values,
    create_dataframe,
    read_df_from_bytes,
};

pub struct DataUpaService {
    repo: web::Data<PgDataRepository>,
}

impl DataUpaService {
    pub fn new(repo: web::Data<PgDataRepository>) -> Self {
        Self { repo }
    }
    
    pub async fn add_data(&self, file_content: web::Bytes) -> Result<HttpResponse, AppError> {
        info!("Iniciando processamento do arquivo CSV");
               
        // Lê o DataFrame diretamente dos bytes do arquivo
        let df = match read_df_from_bytes(&file_content) {
            Ok(df) => df,
            Err(e) => {
                error!("Erro ao ler arquivo CSV: {:?}", e);
                return Err(AppError::BadRequest("Formato de arquivo inválido".to_string()));
            }
        };
    
        let (rows_before, cols_before) = df.shape();
        info!("Arquivo lido com sucesso: {} linhas, {} colunas", rows_before, cols_before);
    
        // Remove as colunas desnecessárias
        let colunas_desnecessarias = columns_names();
        let df_reduzido = match remove_unnecessary_columns(df.clone(), &colunas_desnecessarias) {
            Ok(df) => df,
            Err(e) => {
                error!("Erro ao remover colunas desnecessárias: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
    
        // Adiciona o dia da semana e separa a data e hora
        let df_transformed = match add_week_day_and_split_date_time_polars(df_reduzido) {
            Ok(df) => df,
            Err(e) => {
                error!("Erro ao adicionar dia da semana e separar data/hora: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
        
    
        // Normaliza as colunas de texto (aplica strip e lowercase)
        let colunas_para_normalizar = ["IfroConsultaConduta", "IfroPacienteBairro"];
        let df_normalizado = match normalize_text_to_lower_case_columns_lazy(df_transformed.lazy(), &colunas_para_normalizar).collect() {
            Ok(df) => df,
            Err(e) => {
                error!("Erro ao normalizar colunas de texto para minúsculas: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
    
        // Classificação de doenças
        let df_com_queixas = match extract_keyword_hybrid(&df_normalizado) {
            Ok(df) => df,
            Err(e) => {
                error!("Erro ao extrair e classificar queixas: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
    
        // Remover a coluna "IfroConsultaConduta"
        let colunas_remover = ["IfroConsultaConduta"];
        let df_com_queixas = match remove_unnecessary_columns(df_com_queixas, &colunas_remover) {
            Ok(df) => df,
            Err(e) => {
                error!("Erro ao remover coluna IfroConsultaConduta: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
    
        // Busca dados de bairros no banco
        let map_table_name = "map_neighbourhoods";
        let df_bairros = match self.repo.fetch_all_data(map_table_name).await {
            Ok(data) => {
                if data.is_empty() {
                    info!("Tabela {} está vazia. Usando DataFrame vazio.", map_table_name);
                    DataFrame::default()
                } else {
                    info!("Dados encontrados na tabela {}.", map_table_name);
                    match create_dataframe(&data) {
                        Ok(df) => df,
                        Err(e) => {
                            error!("Erro ao criar DataFrame de bairros: {:?}", e);
                            return Err(AppError::InternalServerError);
                        }
                    }
                }
            },
            Err(e) => {
                error!("Erro ao buscar dados de bairros: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
    
        // Normaliza os nomes dos bairros para maiúsculas
        let colunas_para_normalizar = ["IfroPacienteBairro"];
        let df_com_bairros_normalizados = match normalize_text_to_upper_case_columns_lazy(df_com_queixas.lazy(), &colunas_para_normalizar).collect() {
            Ok(df) => df,
            Err(e) => {
                error!("Erro ao normalizar nomes de bairros para maiúsculas: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
         
        // Padroniza os nomes dos bairros
        let df_bairros_padronizados = match standardize_neighborhood_names(df_com_bairros_normalizados, df_bairros) {
            Ok(df) => df,
            Err(e) => {
                error!("Erro ao padronizar nomes de bairros: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
    
        // Fazer replace nos valores nulos
        let df_com_substituicoes = match fill_null_strings(df_bairros_padronizados) {
            Ok(df) => df,
            Err(e) => {
                error!("Erro ao substituir valores nulos: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
    
        // Excluir coluna especificada
        let colunas_excluir = "Unnamed: 0";
        let df_final_com_exclusao = match drop_column_if_exists(df_com_substituicoes, colunas_excluir) {
            Ok(df) => df,
            Err(e) => {
                error!("Erro ao excluir colunas desnecessárias: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
    
        // Normaliza os nomes das colunas para minúsculas
        let df_final_normalizado = match normalize_column_names_of_the_df_to_lower_case(df_final_com_exclusao) {
            Ok(df) => df,
            Err(e) => {
                error!("Erro ao normalizar nomes de colunas para minúsculas: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
        
        // Verificar duplicidade de ifrocompetencia
        let competencia_values = match get_unique_values(&df_final_normalizado, "ifrocompetencia") {
            Ok(values) => values,
            Err(e) => {
                error!("Erro ao obter valores únicos de ifrocompetencia: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        //Nome da tabela que armazenará os dados
        let table_name = "bpa";
        
        // Verifica se já existem esses valores no banco
        match self.repo.check_ifrocompetencia_exists(table_name, &competencia_values).await {
            Ok(true) => {
                error!("Dados com valores 'ifrocompetencia' {:?} já existem na tabela", competencia_values);
                return Err(AppError::BadRequest(format!("Dados do período {} já existem no banco", competencia_values.join(", "))));
            },
            Ok(false) => {
                // Continua a execução
            },
            Err(e) => {
                error!("Erro ao verificar duplicidade: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        
        // Insere os dados na tabela
        match self.repo.insert_data(&df_final_normalizado, table_name).await {
            Ok(true) => {
                info!("Dados inseridos com sucesso na tabela {}.", table_name);
                let (rows, cols) = df_final_normalizado.shape();
                
                Ok(ApiResponse::created(json!({
                    "message": "Dados processados e importados com sucesso",
                    "rows_processed": rows,
                    "columns_processed": cols,
                    "competencia_values": competencia_values
                })).into_response())
            },
            Ok(false) => {
                error!("Falha ao inserir dados na tabela {}.", table_name);
                Err(AppError::InternalServerError)
            },
            Err(e) => {
                error!("Erro ao inserir dados na tabela {}: {:?}", table_name, e);
                Err(AppError::InternalServerError)
            }
        }
    }
}