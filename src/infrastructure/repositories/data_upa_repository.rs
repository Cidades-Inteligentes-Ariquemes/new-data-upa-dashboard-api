use crate::domain::repositories::data_upa::DataRepository;
use crate::utils::process_data::convert_keys_to_str;
use async_trait::async_trait;
use polars::frame::DataFrame;
use serde_json::{Value, json};
use sqlx::{Column, PgPool, Row};
use uuid::Uuid;
use std::collections::HashMap;
use std::error::Error;

pub struct PgDataRepository {
    pool: PgPool,
}

impl PgDataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DataRepository for PgDataRepository {
    async fn fetch_all_data(&self, table: &str) -> Result<HashMap<String, Vec<Value>>, Box<dyn Error + Send + Sync>> {
        // Constrói a query para buscar todos os dados
        let query = format!("SELECT * FROM {}", table);

        // Executa a query
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await?;

        if rows.is_empty() {
            println!("No data found in table {}", table);
            return Ok(HashMap::new());
        }

        // Obtém os nomes das colunas
        let mut result: HashMap<String, Vec<Value>> = HashMap::new();

        // Para cada linha de resultado
        for row in &rows {
            // Para cada coluna na linha
            for (i, column) in row.columns().iter().enumerate() {
                let column_name = column.name();

                // Inicializa o vetor para a coluna se ainda não existir
                if !result.contains_key(column_name) {
                    result.insert(column_name.to_string(), Vec::new());
                }

                // Tenta obter o valor baseado no tipo da coluna
                let value: Value = match column.type_info().to_string().as_str() {
                    "INT4" | "INT8" => {
                        if let Ok(v) = row.try_get::<i64, _>(i) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    },
                    "FLOAT4" | "FLOAT8" => {
                        if let Ok(v) = row.try_get::<f64, _>(i) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    },
                    "VARCHAR" | "TEXT" => {
                        if let Ok(v) = row.try_get::<String, _>(i) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    },
                    "BOOL" => {
                        if let Ok(v) = row.try_get::<bool, _>(i) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    },
                    "TIMESTAMP" | "TIMESTAMPTZ" => {
                        if let Ok(v) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(i) {
                            json!(v.to_string())
                        } else {
                            Value::Null
                        }
                    },
                    "DATE" => {
                        if let Ok(v) = row.try_get::<chrono::NaiveDate, _>(i) {
                            json!(v.to_string())
                        } else {
                            Value::Null
                        }
                    },
                    _ => {
                        // Para outros tipos, tenta obter como string
                        if let Ok(v) = row.try_get::<String, _>(i) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    }
                };

                // Adiciona o valor ao vetor da coluna
                if let Some(column_values) = result.get_mut(column_name) {
                    column_values.push(value);
                }
            }
        }

        println!("Fetched all data from table {}", table);
        Ok(result)
    }

    async fn check_ifrocompetencia_exists(&self, table: &str, competencia_values: &[String]) -> Result<bool, Box<dyn Error + Send + Sync>> {
        // Formatando os valores para a cláusula ANY do PostgreSQL
        let values_string = competencia_values
            .iter()
            .map(|s| format!("'{}'", s))
            .collect::<Vec<String>>()
            .join(", ");
            
        // Construindo a query
        let query = format!(
            "SELECT 1 FROM {} WHERE ifrocompetencia = ANY(ARRAY[{}]) LIMIT 1",
            table, values_string
        );
        
        // Executando a query
        let result = sqlx::query(&query)
            .fetch_optional(&self.pool)
            .await?;
            
        // Se encontrou algum resultado, então existe duplicidade
        Ok(result.is_some())
    }

    async fn create_table_if_not_exists(&self, df: &DataFrame, table: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        // Verifica se a tabela existe
        let query = format!(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name='{}');",
            table
        );
        
        let row = sqlx::query(&query)
            .fetch_one(&self.pool)
            .await?;
            
        let table_exists: bool = row.get(0);
        
        if !table_exists {
            // Constrói o comando SQL para criar a tabela
            let mut sql_command = format!("CREATE TABLE {} (", table);
            
            // Itera sobre as colunas do DataFrame
            for (name, dtype) in df.schema().iter() {
                let postgres_type = match dtype {
                    polars::prelude::DataType::Int32 => "INTEGER",
                    polars::prelude::DataType::Int64 => "BIGINT",
                    polars::prelude::DataType::Float32 => "FLOAT",
                    polars::prelude::DataType::Float64 => "DOUBLE PRECISION",
                    polars::prelude::DataType::Date => "DATE",
                    polars::prelude::DataType::Datetime(_, _) => "TIMESTAMP",
                    _ => "VARCHAR",
                };
                
                sql_command.push_str(&format!("{} {},", name, postgres_type));
            }
            
            // Remove a vírgula final e fecha o parêntese
            sql_command = sql_command.trim_end_matches(',').to_string();
            sql_command.push_str(");");
            
            // Executa o comando para criar a tabela
            sqlx::query(&sql_command)
                .execute(&self.pool)
                .await?;
                
            println!("Tabela {} criada com sucesso!", table);
        } else {
            println!("Tabela {} já existe. Anexando dados.", table);
        }
        
        Ok(true)
    }
    
    async fn insert_data(&self, df: &DataFrame, table: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        // Primeiro cria a tabela se necessário
        match self.create_table_if_not_exists(df, table).await {
            Ok(true) => {
                // Se a tabela foi criada ou já existe, insere os dados
                let batch_size = 5000;
                let total_rows = df.height();
                
                for batch_start in (0..total_rows).step_by(batch_size) {
                    let batch_end = std::cmp::min(batch_start + batch_size, total_rows);
                    let df_batch = df.slice(batch_start as i64, batch_end - batch_start);
                    
                    // Filtrar para remover nomes de colunas vazios
                    let filtered_columns: Vec<String> = df.get_column_names()
                        .into_iter()
                        .filter(|name| !name.is_empty())
                        .map(|name| name.to_string())
                        .collect();
                    
                    // Construir a consulta de inserção para este lote
                    let column_names = filtered_columns.join(", ");
                    
                    // Se não há colunas ou linhas, pula este lote
                    if filtered_columns.is_empty() || df_batch.height() == 0 {
                        println!("Lote vazio ou sem colunas válidas, pulando.");
                        continue;
                    }
                    
                    let mut value_strings = Vec::new();
                    
                    // Adicionar valores para cada linha
                    for row_idx in 0..df_batch.height() {
                        let mut row_values = Vec::new();
                        
                        for col_name in &filtered_columns { // Iterate over references to owned Strings
                            let value = df_batch.column(col_name)?.get(row_idx)?;
                            
                            // Formatar o valor de acordo com seu tipo
                            let formatted_value = match value {
                                polars::prelude::AnyValue::Null => "NULL".to_string(),
                                polars::prelude::AnyValue::String(s) => format!("'{}'", s.replace("'", "''")),
                                _ => format!("{}", value),
                            };
                            
                            row_values.push(formatted_value);
                        }
                        
                        // Junta os valores da linha com vírgulas e envolve em parênteses
                        value_strings.push(format!("({})", row_values.join(", ")));
                    }
                    
                    // Construir a consulta completa
                    let sql_command = format!(
                        "INSERT INTO {} ({}) VALUES {}",
                        table,
                        column_names,
                        value_strings.join(", ")
                    );
                    
                    // Salva parte da query para ver o erro, caso tenha
                    let truncated_query = if sql_command.len() > 5000 {
                        format!("{}...", &sql_command[..5000])
                    } else {
                        sql_command.clone()
                    };
                    println!("SQL Query (truncada): {}", truncated_query);
                    
                    // Executa a consulta de inserção para este lote
                    match sqlx::query(&sql_command).execute(&self.pool).await {
                        Ok(_) => {
                            println!("Inseridos dados de {} na tabela {}", 
                                total_rows, table);
                        },
                        Err(e) => {
                            eprintln!("Erro durante a inserção: {}", e);
                            return Err(e.into());
                        }
                    }
                }
                
                println!("Dados inseridos com sucesso na tabela {}.", table);
                Ok(true)
            },
            Ok(false) => {
                println!("Falha ao criar tabela {}. Os dados não puderam ser inseridos.", table);
                Ok(false)
            },
            Err(e) => {
                eprintln!("Erro ao criar tabela {}: {}", table, e);
                Err(e)
            }
        }
    }

    async fn fetch_columns_by_name(&self, table: &str, columns: &[String]) -> Result<HashMap<String, Vec<Value>>, Box<dyn Error + Send + Sync>> {
        // Constrói a query para buscar colunas específicas
        let query = format!("SELECT {} FROM {}", columns.join(", "), table);

        // Executa a query
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await?;

        if rows.is_empty() {
            println!("No data found in table {} for columns {}", table, columns.join(", "));
            return Ok(HashMap::new());
        }

        // Inicializar o resultado
        let mut result: HashMap<String, Vec<Value>> = HashMap::new();
        for col_name in columns {
            result.insert(col_name.to_string(), Vec::new());
        }

        // Para cada linha de resultado
        for row in &rows {
            // Para cada coluna na linha
            for (i, column_name) in columns.iter().enumerate() {
                // Tentar obter o valor baseado no tipo da coluna
                let column = row.columns().get(i).unwrap();
                let value: Value = match column.type_info().to_string().as_str() {
                    "INT4" | "INT8" => {
                        if let Ok(v) = row.try_get::<i64, _>(i) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    },
                    "FLOAT4" | "FLOAT8" => {
                        if let Ok(v) = row.try_get::<f64, _>(i) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    },
                    "VARCHAR" | "TEXT" => {
                        if let Ok(v) = row.try_get::<String, _>(i) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    },
                    "BOOL" => {
                        if let Ok(v) = row.try_get::<bool, _>(i) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    },
                    "TIMESTAMP" | "TIMESTAMPTZ" => {
                        if let Ok(v) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(i) {
                            json!(v.to_string())
                        } else {
                            Value::Null
                        }
                    },
                    "DATE" => {
                        if let Ok(v) = row.try_get::<chrono::NaiveDate, _>(i) {
                            json!(v.to_string())
                        } else {
                            Value::Null
                        }
                    },
                    _ => {
                        // Para outros tipos, tenta obter como string
                        if let Ok(v) = row.try_get::<String, _>(i) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    }
                };

                // Adiciona o valor ao vetor da coluna
                if let Some(column_values) = result.get_mut(column_name) {
                    column_values.push(value);
                }
            }
        }

        println!("Fetched columns {:?} from table {}", columns, table);
        Ok(result)
    }

    

    async fn insert_nested_json(&self, data: Value, table: &str, identifier: &str) -> Result<HashMap<String, Value>, Box<dyn Error + Send + Sync>> {
        // Habilita a extensão uuid-ossp se necessário
        sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";")
            .execute(&self.pool)
            .await?;

        // Cria a tabela se não existir
        let create_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id UUID DEFAULT uuid_generate_v4(),
                identifier TEXT UNIQUE,
                data JSONB
            );",
            table
        );

        sqlx::query(&create_table_query)
            .execute(&self.pool)
            .await?;

        // Converter chaves para strings
        let processed_data = convert_keys_to_str(data);

        // Serializar o dicionário em JSON
        let data_json = processed_data.to_string();

        // Inserir ou atualizar os dados
        let insert_query = format!(
            "INSERT INTO {} (identifier, data)
            VALUES ($1, $2::jsonb)
            ON CONFLICT (identifier)
            DO UPDATE SET data = EXCLUDED.data
            RETURNING id;",
            table
        );

        let record_id: Uuid = sqlx::query_scalar(&insert_query)
            .bind(identifier)
            .bind(data_json)
            .fetch_one(&self.pool)
            .await?;

        println!("Dados inseridos/atualizados com sucesso na tabela {} com id {}", table, record_id);

        let mut result = HashMap::new();
        result.insert("id".to_string(), json!(record_id.to_string()));
        result.insert("added".to_string(), json!(true));

        Ok(result)
    }

    async fn fetch_nested_json(&self, table: &str, identifier: &str) -> Result<serde_json::Map<String, serde_json::Value>, Box<dyn Error + Send + Sync>> {
        // Consulta SQL para buscar dados JSON
        let query = format!("SELECT data FROM {} WHERE identifier = $1", table);
        
        // Executa a consulta
        let row = sqlx::query(&query)
            .bind(identifier)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            // Extrair o valor JSON da coluna 'data'
            let json_data: serde_json::Value = row.get("data");
            
            // Verificar se é um objeto JSON
            if let serde_json::Value::Object(map) = json_data {
                return Ok(map);
            }
            
            // Se não for um objeto, retornar um mapa vazio
            Ok(serde_json::Map::new())
        } else {
            // Se não encontrou nenhum registro, retornar um mapa vazio
            Ok(serde_json::Map::new())
        }
    }
}