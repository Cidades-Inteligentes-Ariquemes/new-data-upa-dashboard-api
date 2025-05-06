use polars::prelude::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::error::Error;


pub struct DataProcessingForGraphPlotting;

impl DataProcessingForGraphPlotting {
    // Função para obter colunas para plotagem
    pub fn columns_to_plot_graphs() -> HashMap<String, Value> {
        let mut result = HashMap::new();
        
        // Tabelas
        let mut tables = HashMap::new();
        tables.insert("bpa".to_string(), json!("bpa"));
        tables.insert("map_neighbourhoods".to_string(), json!("map_neighbourhoods"));
        tables.insert("non_doctors".to_string(), json!("non_doctors"));
        tables.insert("non_nurse".to_string(), json!("non_nurse"));
        
        // Colunas
        let mut columns = HashMap::new();
        columns.insert("number_of_appointments_per_month".to_string(), json!(["ifrocompetencia"]));
        columns.insert("number_of_appointments_per_flow".to_string(), json!(["ifrocompetencia", "ifrotabelanome"]));
        columns.insert("distribuition_of_patients_ages".to_string(), json!(["ifrocompetencia", "ifropacienteidade"]));
        columns.insert("number_of_calls_per_day_of_the_week".to_string(), json!(["ifrocompetencia", "ifrodiasemana"]));
        columns.insert("distribution_of_services_by_hour_group".to_string(), json!(["ifrocompetencia", "ifrohoraatendimento", "ifroprofissionalcbods"]));
        columns.insert("number_of_visits_per_nurse".to_string(), json!(["ifrocompetencia", "ifroprofissionalid","ifroprofissionalcbods", "ifroprofissionalnome", "ifrotabelanome"]));
        columns.insert("number_of_visits_per_doctor".to_string(), json!(["ifrocompetencia", "ifroprofissionalid","ifroprofissionalcbods", "ifroprofissionalnome", "ifrotabelanome"]));
        columns.insert("average_time_per_doctor".to_string(), json!(["ifrocompetencia", "ifrohoraatendimento","ifroprofissionalid","ifroprofissionalcbods", "ifroprofissionalnome", "ifrotabelanome"]));
        columns.insert("heat_map_with_disease_indication".to_string(), json!(["ifrocompetencia", "ifropacienteendereco", "ifropacientebairro", "ifropacientequeixaprincipal", "ifropacientelatitude", "ifropacientelongitude"]));
        columns.insert("heat_map_with_the_number_of_medical_appointments_by_neighborhood".to_string(), json!(["ifrocompetencia", "ifropacienteendereco", "ifropacientebairro", "ifropacientelatitude", "ifropacientelongitude"]));
        
        result.insert("tables".to_string(), json!(tables));
        result.insert("columns".to_string(), json!(columns));
        
        json!(result).as_object().unwrap().clone().into_iter().collect()
    }

    // Implementação das funções de processamento
    pub async fn create_dict_to_number_of_appointments_per_month(&self, df: &DataFrame) -> Result<Value, Box<dyn Error + Send + Sync>> {
        // Imprimir colunas originais
        println!("Colunas originais: {:?}", df.get_column_names());
        
        // Em vez de usar .lazy() e groupby, vamos fazer contagem manual
        let comp_values = df.column("ifrocompetencia")?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect::<Vec<String>>();
        
        // Fazer contagem
        let mut counts: HashMap<String, i64> = HashMap::new();
        for comp in comp_values {
            *counts.entry(comp).or_insert(0) += 1;
        }
        
        // Converter para o formato final
        let mut result = HashMap::new();
        for (category, count) in counts {
            result.insert(category, json!(count));
        }
        
        Ok(json!(result))
    }

   
    pub async fn create_dict_to_number_of_appointments_per_flow(&self, df: &DataFrame) -> Result<Value, Box<dyn Error + Send + Sync>> {
        // Imprimir colunas originais
        println!("Colunas originais: {:?}", df.get_column_names());
        
        // Extrair dados das colunas necessárias
        let competencias = df.column("ifrocompetencia")?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect::<Vec<String>>();
        
        let tabelas = df.column("ifrotabelanome")?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect::<Vec<String>>();
        
        // Garantir que os vetores têm o mesmo tamanho
        if competencias.len() != tabelas.len() {
            return Err(Box::<dyn Error + Send + Sync>::from("Tamanhos de colunas incompatíveis"));
        }
        
        // Fazer contagem manual com agrupamento duplo
        let mut counts: HashMap<String, HashMap<String, i64>> = HashMap::new();
        
        for i in 0..competencias.len() {
            let categoria = &tabelas[i];
            let competencia = &competencias[i];
            
            // Inicializar categoria se ainda não existe
            if !counts.contains_key(categoria) {
                counts.insert(categoria.clone(), HashMap::new());
            }
            
            // Incrementar contagem para esta competência
            let comp_counts = counts.get_mut(categoria).unwrap();
            *comp_counts.entry(competencia.clone()).or_insert(0) += 1;
        }
        
        // Converter para o formato final
        let mut organized_data = HashMap::new();
        
        for (categoria, comp_counts) in counts {
            let mut category_data = HashMap::new();
            
            // Calcular total
            let total: i64 = comp_counts.values().sum();
            category_data.insert("todos".to_string(), json!(total));
            
            // Adicionar contagens por competência
            for (competencia, count) in comp_counts {
                category_data.insert(competencia, json!(count));
            }
            
            organized_data.insert(categoria, category_data);
        }
        
        Ok(json!(organized_data))
    }


    pub async fn create_dict_to_distribuition_of_patients_ages_from_raw(
        &self, 
        raw_data: &HashMap<String, Vec<Value>>
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        println!("Processando distribuição de idades dos pacientes a partir de dados brutos");
        
        // Extrai vetores de competência e idade diretamente do HashMap
        let competencias = match raw_data.get("ifrocompetencia") {
            Some(comp_values) => comp_values,
            None => {
                println!("Competência não encontrada no mapa de dados");
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Coluna ifrocompetencia não encontrada"
                )));
            }
        };
        
        let idades = match raw_data.get("ifropacienteidade") {
            Some(age_values) => age_values,
            None => {
                println!("Idade não encontrada no mapa de dados");
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Coluna ifropacienteidade não encontrada"
                )));
            }
        };
        
        let n_rows = competencias.len();
        println!("Número de linhas: {}", n_rows);
        
        // Amostra de dados para debug
        println!("Amostra de competências: {:?}", &competencias[0..5.min(competencias.len())]);
        println!("Amostra de idades: {:?}", &idades[0..5.min(idades.len())]);
        
        // Definir faixas etárias
        let age_groups = vec![
            ("0 a 19", 0..20),
            ("20 a 39", 20..40),
            ("40 a 59", 40..60),
            ("60 a 79", 60..80),
            ("80 a 100", 80..101),
            ("+ de 100", 101..i32::MAX)
        ];
        
        // Contadores para cada faixa etária
        let mut age_data: HashMap<&str, HashMap<String, i64>> = HashMap::new();
        for (group, _) in &age_groups {
            age_data.insert(group, HashMap::new());
        }
        
        // Processar cada linha
        let mut valid_ages = 0;
        let mut invalid_ages = 0;
        
        for i in 0..n_rows {
            // Extrair idade
            let idade = match &idades[i] {
                Value::Number(n) => {
                    if let Some(num) = n.as_i64() {
                        valid_ages += 1;
                        num as i32
                    } else if let Some(num) = n.as_f64() {
                        valid_ages += 1;
                        num as i32
                    } else {
                        invalid_ages += 1;
                        -1
                    }
                },
                Value::String(s) => {
                    match s.parse::<i32>() {
                        Ok(num) => {
                            valid_ages += 1;
                            num
                        },
                        Err(_) => {
                            invalid_ages += 1;
                            -1
                        }
                    }
                },
                _ => {
                    invalid_ages += 1;
                    -1
                }
            };
            
            // Extrai competência
            let competencia = match &competencias[i] {
                Value::String(s) => s.clone(),
                _ => {
                    // Pular se não conseguir extrair competência
                    continue;
                }
            };
            
            // Log para debug das primeiras linhas
            if i < 10 {
                println!("Linha {}: idade={}, competencia={}", i, idade, competencia);
            }
            
            // Se idade válida, classifica na faixa etária correspondente
            if idade >= 0 {
                for (group, range) in &age_groups {
                    if range.contains(&idade) {
                        *age_data
                            .get_mut(group)
                            .unwrap()
                            .entry(competencia.clone())
                            .or_insert(0) += 1;
                        break;
                    }
                }
            }
        }
        
        println!("Idades válidas: {}, Idades inválidas: {}", valid_ages, invalid_ages);
        
        // Construir o resultado final
        let mut result = HashMap::new();
        for (group, _) in &age_groups {
            let counts = &age_data[group];
            let total: i64 = counts.values().sum();
            
            let mut group_data = HashMap::new();
            group_data.insert("todos".to_string(), json!(total));
            
            for (comp, count) in counts {
                group_data.insert(comp.clone(), json!(count));
            }
            
            result.insert(group.to_string(), json!(group_data));
        }
        
        // Debug do resultado
        println!("Resultado: {}", json!(result));
        
        Ok(json!(result))
    }



    pub async fn create_dict_to_number_of_calls_per_day_of_the_week(&self, df: &DataFrame) -> Result<Value, Box<dyn Error + Send + Sync>> {
        // Mapeamento de dias da semana
        let day_mappings = [
            ("Monday", "segunda-feira"),
            ("Tuesday", "terça-feira"),
            ("Wednesday", "quarta-feira"),
            ("Thursday", "quinta-feira"),
            ("Friday", "sexta-feira"),
            ("Saturday", "sábado"),
            ("Sunday", "domingo")
        ];
        
        // Cria Series com mapeamento de dias
        let mut day_mapping_expr: Expr = when(col("ifrodiasemana").eq(lit("")))
            .then(lit(""))
            .otherwise(lit("")); 
        
        for (en_day, pt_day) in day_mappings.iter() {
            day_mapping_expr = when(col("ifrodiasemana").eq(lit(*en_day)))
                .then(lit(*pt_day))
                .otherwise(day_mapping_expr); 
        }
        
        // Aplica mapeamento
        let df_with_pt_days = df.clone().lazy()
            .with_column(day_mapping_expr.alias("dia_semana_pt"))
            .collect()?;
        
        // Cria dicionário organizado para cada dia da semana
        let mut organized_data = HashMap::new();
        
        for (_, pt_day) in day_mappings.iter() {
            // Filtra por este dia da semana
            let pt_day_owned = (*pt_day).to_string();
            
            let df_day = df_with_pt_days.clone().lazy()
                .filter(col("dia_semana_pt").eq(lit(pt_day_owned.clone())))
                .collect()?;
            
            // Extrai competências
            let competencias = df_day.column("ifrocompetencia")?
                .str()?
                .into_iter()
                .filter_map(|opt_s| opt_s.map(String::from))
                .collect::<Vec<String>>();
            
            // Faz a contagem manual
            let mut comp_counts: HashMap<String, i64> = HashMap::new();
            for comp in competencias {
                *comp_counts.entry(comp).or_insert(0) += 1;
            }
            
            let mut day_data = HashMap::new();
            day_data.insert("todos".to_string(), json!(df_day.height() as i64));
            
            // Adiciona contagens por competência
            for (competencia, count) in comp_counts {
                day_data.insert(competencia, json!(count));
            }
            
            organized_data.insert(pt_day_owned, day_data);
        }
        
        Ok(json!(organized_data))
    }



    pub async fn create_dict_to_distribution_of_services_by_hour_group(&self, df: &DataFrame) -> Result<Value, Box<dyn Error + Send + Sync>> {
        println!("Processando distribuição de serviços por grupo de hora");
        
        // Filtrar para médicos clínicos manualmente
        let mut medicos_indices = Vec::new();
        for i in 0..df.height() {
            let especialidade = df.column("ifroprofissionalcbods")?.str()?.get(i).unwrap_or("");
            if especialidade == "MEDICO CLINICO" {
                medicos_indices.push(i);
            }
        }
        
        // Extrair horas e competências apenas dos médicos clínicos
        let mut horas = Vec::new();
        let mut competencias = Vec::new();
        
        for &idx in &medicos_indices {
            let time_str = df.column("ifrohoraatendimento")?.str()?.get(idx).unwrap_or("");
            let competencia = df.column("ifrocompetencia")?.str()?.get(idx).unwrap_or("").to_string();
            
            let hour = if time_str.len() >= 2 {
                // Tentar extrair as primeiras 2 posições como hora
                time_str[0..2].parse::<i32>().unwrap_or(-1)
            } else {
                -1 // Valor inválido
            };
            
            if hour >= 0 { // Ignorar valores inválidos
                horas.push(hour);
                competencias.push(competencia);
            }
        }
        
        // Definir os grupos de horas
        let hour_groups = [
            "00h-02h", "02h-04h", "04h-06h", "06h-08h", "08h-10h", 
            "10h-12h", "12h-14h", "14h-16h", "16h-18h", "18h-20h", 
            "20h-22h", "22h-24h"
        ];
        
        // Preparar contadores para cada grupo de hora
        let mut hour_group_data: HashMap<String, HashMap<String, i64>> = HashMap::new();
        for &group in &hour_groups {
            hour_group_data.insert(group.to_string(), HashMap::new());
        }
        
        // Classificar cada hora em seu grupo e contar
        for i in 0..horas.len() {
            let hour = horas[i];
            let competencia = &competencias[i];
            
            let group = if hour >= 0 && hour < 2 {
                "00h-02h"
            } else if hour >= 2 && hour < 4 {
                "02h-04h"
            } else if hour >= 4 && hour < 6 {
                "04h-06h"
            } else if hour >= 6 && hour < 8 {
                "06h-08h"
            } else if hour >= 8 && hour < 10 {
                "08h-10h"
            } else if hour >= 10 && hour < 12 {
                "10h-12h"
            } else if hour >= 12 && hour < 14 {
                "12h-14h"
            } else if hour >= 14 && hour < 16 {
                "14h-16h"
            } else if hour >= 16 && hour < 18 {
                "16h-18h"
            } else if hour >= 18 && hour < 20 {
                "18h-20h"
            } else if hour >= 20 && hour < 22 {
                "20h-22h"
            } else {
                "22h-24h"
            };
            
            // Incrementar contagem para esta competência e grupo de hora
            let comp_counts = hour_group_data.get_mut(group).unwrap();
            *comp_counts.entry(competencia.clone()).or_insert(0) += 1;
        }
        
        // Criar o dicionário organizado final
        let mut organized_data = HashMap::new();
        
        for group in hour_groups {
            let comp_counts = hour_group_data.get(group).unwrap();
            
            // Calcular total para este grupo de hora
            let total: i64 = comp_counts.values().sum();
            
            let mut hour_data = HashMap::new();
            hour_data.insert("todos".to_string(), json!(total));
            
            // Adicionar contagens por competência
            for (competencia, count) in comp_counts {
                hour_data.insert(competencia.clone(), json!(count));
            }
            
            organized_data.insert(group.to_string(), hour_data);
        }
        
        Ok(json!(organized_data))
    }



    pub async fn create_dict_to_number_of_visits_per_nurse(&self, df: &DataFrame, df_non_nurse: &DataFrame) -> Result<Value, Box<dyn Error + Send + Sync>> {
        // Obter lista de enfermeiras a excluir
        let non_nurse_names: Vec<String> = df_non_nurse.column("ifroprofissionalnome")?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect();
        
        // Filtrar DataFrame para enfermeiros e acolhimento
        let df_nurse_acolhimento = df.clone().lazy()
            .filter(
                col("ifroprofissionalcbods").eq(lit("ENFERMEIRO"))
                .and(col("ifrotabelanome").eq(lit("Acolhimento")))
            )
            .collect()?;
        
        // Filtrar nomes não desejados
        let mut keep_rows = Vec::with_capacity(df_nurse_acolhimento.height());
        
        for i in 0..df_nurse_acolhimento.height() {
            let nome = df_nurse_acolhimento.column("ifroprofissionalnome")?.str()?.get(i).unwrap_or("");
            let keep = !non_nurse_names.contains(&nome.to_string());
            keep_rows.push(keep);
        }
        
        // Converter para Series e filtrar
        let mask = BooleanChunked::new("mask".into(), keep_rows);
        let df_filtered = df_nurse_acolhimento.filter(&mask)?;
        
        // Obter nomes únicos de enfermeiras
        let unique_nurses = df_filtered.column("ifroprofissionalnome")?
            .unique()?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect::<Vec<String>>();
        
        // Criar dicionário organizado para cada enfermeira
        let mut organized_data = HashMap::new();
        
        for nurse_name in unique_nurses {
            // Filtrar para esta enfermeira
            let nurse_name_owned = nurse_name.clone();
            
            let df_nurse = df_filtered.clone().lazy()
                .filter(col("ifroprofissionalnome").eq(lit(nurse_name_owned)))
                .collect()?;
            
            // Extrair competências
            let competencias = df_nurse.column("ifrocompetencia")?
                .str()?
                .into_iter()
                .filter_map(|opt_s| opt_s.map(String::from))
                .collect::<Vec<String>>();
            
            // Fazer contagem manual
            let mut comp_counts: HashMap<String, i64> = HashMap::new();
            for comp in competencias {
                *comp_counts.entry(comp).or_insert(0) += 1;
            }
            
            let mut nurse_data = HashMap::new();
            nurse_data.insert("todos".to_string(), json!(df_nurse.height() as i64));
            
            // Adicionar contagens por competência
            for (competencia, count) in comp_counts {
                nurse_data.insert(competencia, json!(count));
            }
            
            organized_data.insert(nurse_name, nurse_data);
        }
        
        Ok(json!(organized_data))
    }


    pub async fn create_dict_to_number_of_visits_per_doctor(&self, df: &DataFrame, df_non_doctors: &DataFrame) -> Result<Value, Box<dyn Error + Send + Sync>> {
        // Obter lista de médicos a excluir
        let non_doctor_names: Vec<String> = df_non_doctors.column("ifroprofissionalnome")?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect();
        
        // Filtrar DataFrame para médicos e consulta médica
        let df_doctor_consulta = df.clone().lazy()
            .filter(
                col("ifroprofissionalcbods").eq(lit("MEDICO CLINICO"))
                .and(col("ifrotabelanome").eq(lit("ConsultaMedica")))
            )
            .collect()?;
        
        // Filtrar nomes não desejados
        let mut keep_rows = Vec::with_capacity(df_doctor_consulta.height());
        
        for i in 0..df_doctor_consulta.height() {
            let nome = df_doctor_consulta.column("ifroprofissionalnome")?.str()?.get(i).unwrap_or("");
            let keep = !non_doctor_names.contains(&nome.to_string());
            keep_rows.push(keep);
        }
        
        // Converter para Series e filtrar
        let mask = BooleanChunked::new("mask".into(), keep_rows);
        let df_filtered = df_doctor_consulta.filter(&mask)?;
        
        // Obter nomes únicos de médicos
        let unique_doctors = df_filtered.column("ifroprofissionalnome")?
            .unique()?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect::<Vec<String>>();
        
        // Criar dicionário organizado para cada médico
        let mut organized_data = HashMap::new();
        
        for doctor_name in unique_doctors {
            // Filtrar para este médico
            let doctor_name_owned = doctor_name.clone();
            
            let df_doctor = df_filtered.clone().lazy()
                .filter(col("ifroprofissionalnome").eq(lit(doctor_name_owned)))
                .collect()?;
            
            // Extrair competências
            let competencias = df_doctor.column("ifrocompetencia")?
                .str()?
                .into_iter()
                .filter_map(|opt_s| opt_s.map(String::from))
                .collect::<Vec<String>>();
            
            // Fazer contagem manual
            let mut comp_counts: HashMap<String, i64> = HashMap::new();
            for comp in competencias {
                *comp_counts.entry(comp).or_insert(0) += 1;
            }
            
            let mut doctor_data = HashMap::new();
            doctor_data.insert("todos".to_string(), json!(df_doctor.height() as i64));
            
            // Adicionar contagens por competência
            for (competencia, count) in comp_counts {
                doctor_data.insert(competencia, json!(count));
            }
            
            organized_data.insert(doctor_name, doctor_data);
        }
        
        Ok(json!(organized_data))
    }


    pub async fn create_dict_to_average_time_in_minutes_per_doctor(&self, df: &DataFrame, df_non_doctors: &DataFrame) -> Result<Value, Box<dyn Error + Send + Sync>> {
        // Obter lista de médicos a excluir
        let non_doctor_names: Vec<String> = df_non_doctors.column("ifroprofissionalnome")?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect();
        
        // Filtrar DataFrame para médicos e consulta médica com hora de atendimento
        let df_doctor_consulta = df.clone().lazy()
            .filter(
                (
                    col("ifroprofissionalcbods").eq(lit("MEDICO CLINICO"))
                    .or(col("ifroprofissionalcbods").eq(lit("MEDICO CIRURGIAO GERAL")))
                )
                .and(col("ifrotabelanome").eq(lit("ConsultaMedica")))
                .and(col("ifrohoraatendimento").is_not_null())
            )
            .collect()?;
        
        // Filtrar nomes não desejados
        let mut keep_rows = Vec::with_capacity(df_doctor_consulta.height());
        
        for i in 0..df_doctor_consulta.height() {
            let nome = df_doctor_consulta.column("ifroprofissionalnome")?.str()?.get(i).unwrap_or("");
            let keep = !non_doctor_names.contains(&nome.to_string());
            keep_rows.push(keep);
        }
        
        // Converter para Series e filtrar
        let mask = BooleanChunked::new("mask".into(), keep_rows);
        let df_filtered = df_doctor_consulta.filter(&mask)?;
        
        // Calcular tempo em minutos
        let mut minutes_values = Vec::with_capacity(df_filtered.height());
        
        for i in 0..df_filtered.height() {
            let time_str = df_filtered.column("ifrohoraatendimento")?.str()?.get(i).unwrap_or("");
            let parts: Vec<&str> = time_str.split(':').collect();
            
            let minutes = if parts.len() >= 3 {
                let hours = parts[0].parse::<f64>().unwrap_or(0.0);
                let mins = parts[1].parse::<f64>().unwrap_or(0.0);
                let secs = parts[2].parse::<f64>().unwrap_or(0.0);
                
                hours * 60.0 + mins + secs / 60.0
            } else {
                0.0
            };
            
            minutes_values.push(minutes);
        }
        
        // Criar Series com os minutos
        let minutes_series = Series::new("time_minutes".into(), minutes_values);
        let mut df_with_time = df_filtered.clone();
        df_with_time.with_column(minutes_series)?;
        
        // Filtrar tempos inválidos
        let valid_times_mask = df_with_time.column("time_minutes")?
            .f64()?
            .gt(0.0);
        
        let df_valid_times = df_with_time.filter(&valid_times_mask)?;
        
        // Obter nomes únicos de médicos
        let unique_doctors = df_valid_times.column("ifroprofissionalnome")?
            .unique()?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect::<Vec<String>>();
        
        // Criar dicionário organizado para cada médico
        let mut organized_data = HashMap::new();
        
        for doctor_name in unique_doctors {
            // Filtrar para este médico
            let doctor_mask = df_valid_times.column("ifroprofissionalnome")?
                .str()?
                .equal(doctor_name.as_str());
            
            let df_doctor = df_valid_times.filter(&doctor_mask)?;
            
            // Calcular média total do médico
            let time_values = df_doctor.column("time_minutes")?.f64()?;
            let sum: f64 = time_values.iter().fold(0.0, |acc, opt_val| {
                acc + opt_val.unwrap_or(0.0)
            });
            
            let avg_total = if df_doctor.height() > 0 {
                sum / df_doctor.height() as f64
            } else {
                0.0
            };
            
            // Formatar tempo médio total
            let total_hours = (avg_total / 60.0).floor() as i32;
            let total_mins = (avg_total % 60.0).round() as i32;
            let total_formatted = format!("{:02}:{:02}", total_hours, total_mins);
            
            // Iniciar dados do médico
            let mut doctor_data = HashMap::new();
            doctor_data.insert("todos".to_string(), json!(total_formatted));
            
            // Calcular médias por competência
            let competencias = df_doctor.column("ifrocompetencia")?
                .unique()?
                .str()?
                .into_iter()
                .filter_map(|opt_s| opt_s.map(String::from))
                .collect::<Vec<String>>();
            
            for competencia in competencias {
                let comp_mask = df_doctor.column("ifrocompetencia")?
                    .str()?
                    .equal(competencia.as_str());
                
                let df_comp = df_doctor.filter(&comp_mask)?;
                
                // Calcular média para esta competência
                let comp_time_values = df_comp.column("time_minutes")?.f64()?;
                let comp_sum: f64 = comp_time_values.iter().fold(0.0, |acc, opt_val| {
                    acc + opt_val.unwrap_or(0.0)
                });
                
                let avg_comp = if df_comp.height() > 0 {
                    comp_sum / df_comp.height() as f64
                } else {
                    0.0
                };

                // Formatar tempo médio por competência
                let comp_hours = (avg_comp / 60.0).floor() as i32;
                let comp_mins = (avg_comp % 60.0).round() as i32;
                let comp_formatted = format!("{:02}:{:02}", comp_hours, comp_mins);
                
                doctor_data.insert(competencia, json!(comp_formatted));
            }
            
            organized_data.insert(doctor_name, doctor_data);
        }
        
        Ok(json!(organized_data))
    }


    pub async fn create_dictionary_with_location_and_number_per_disease(&self, df: &DataFrame) -> Result<Value, Box<dyn Error + Send + Sync>> {
        println!("Processando mapa de calor com indicação de doenças");
        
        // Extrair todos os dados relevantes
        let mut dados_validos = Vec::new();
        
        for i in 0..df.height() {
            let competencia = df.column("ifrocompetencia")?.str()?.get(i);
            let endereco = df.column("ifropacienteendereco")?.str()?.get(i);
            let bairro = df.column("ifropacientebairro")?.str()?.get(i);
            let queixa = df.column("ifropacientequeixaprincipal")?.str()?.get(i);
            let latitude_opt = df.column("ifropacientelatitude")?.str()?.get(i);
            let longitude_opt = df.column("ifropacientelongitude")?.str()?.get(i);
            
            // Verificar se todos os campos estão preenchidos
            if let (Some(comp), Some(end), Some(b), Some(q), Some(lat_str), Some(long_str)) = 
                   (competencia, endereco, bairro, queixa, latitude_opt, longitude_opt) {
                
                // Filtrar endereços "DO IPE"
                if end != "DO IPE" {
                    // Converter latitude e longitude para números
                    if let (Ok(lat), Ok(long)) = (lat_str.parse::<f64>(), long_str.parse::<f64>()) {
                        // Adicionar aos dados válidos
                        dados_validos.push((
                            comp.to_string(),
                            b.to_string(),
                            q.to_string(),
                            lat,
                            long
                        ));
                    }
                }
            }
        }
        
        // Estrutura para armazenar dados por queixa, competência e bairro
        // Uma abordagem alternativa sem usar f64 como chave
        let mut dados_por_queixa: HashMap<String, HashMap<String, HashMap<String, (f64, f64, i64)>>> = HashMap::new();
        
        // Processar os dados
        for (comp, bairro, queixa, lat, long) in dados_validos {
            // Garantir que a queixa exista no mapa
            if !dados_por_queixa.contains_key(&queixa) {
                dados_por_queixa.insert(queixa.clone(), HashMap::new());
            }
            
            // Obter o mapa de competências para esta queixa
            let comp_map = dados_por_queixa.get_mut(&queixa).unwrap();
            
            // Garantir que a competência exista no mapa
            if !comp_map.contains_key(&comp) {
                comp_map.insert(comp.clone(), HashMap::new());
            }
            
            // Obter o mapa de bairros para esta competência
            let bairro_map = comp_map.get_mut(&comp).unwrap();
            
            // Adicionar ou atualizar dados do bairro
            let entry = bairro_map.entry(bairro.clone()).or_insert((lat, long, 0));
            entry.2 += 1;
            
            // Também adicionar à competência "todos"
            if !comp_map.contains_key("todos") {
                comp_map.insert("todos".to_string(), HashMap::new());
            }
            
            let todos_map = comp_map.get_mut("todos").unwrap();
            let todos_entry = todos_map.entry(bairro.clone()).or_insert((lat, long, 0));
            todos_entry.2 += 1;
        }
        
        // Construir o JSON final
        let mut final_dict = HashMap::new();
        
        for (queixa, comp_map) in dados_por_queixa {
            let mut illness_dict = HashMap::new();
            
            for (comp, bairro_map) in comp_map {
                let mut comp_dict = HashMap::new();
                
                for (bairro, (lat, long, quantidade)) in bairro_map {
                    let mut neighborhood_data = HashMap::new();
                    neighborhood_data.insert("latitude".to_string(), json!(lat));
                    neighborhood_data.insert("longitude".to_string(), json!(long));
                    neighborhood_data.insert("quantidade".to_string(), json!(quantidade));
                    
                    comp_dict.insert(bairro, json!(neighborhood_data));
                }
                
                illness_dict.insert(comp, comp_dict);
            }
            
            final_dict.insert(queixa, json!(illness_dict));
        }
        
        Ok(json!(final_dict))
    }



    pub async fn create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood(&self, df: &DataFrame) -> Result<Value, Box<dyn Error + Send + Sync>> {
        println!("Processando mapa de calor com o número de atendimentos médicos por bairro");
        
        // Extrai todos os dados relevantes
        let mut dados_validos = Vec::new();
        
        for i in 0..df.height() {
            let bairro = df.column("ifropacientebairro")?.str()?.get(i);
            let latitude_opt = df.column("ifropacientelatitude")?.str()?.get(i);
            let longitude_opt = df.column("ifropacientelongitude")?.str()?.get(i);
            let endereco = df.column("ifropacienteendereco")?.str()?.get(i);
            
            // Verificar se todos os campos necessários estão preenchidos
            if let (Some(b), Some(lat_str), Some(long_str), Some(end)) = (bairro, latitude_opt, longitude_opt, endereco) {
                // Filtrar endereços "DO IPE"
                if end != "DO IPE" {
                    // Converter latitude e longitude para números
                    if let (Ok(lat), Ok(long)) = (lat_str.parse::<f64>(), long_str.parse::<f64>()) {
                        // Adicionar aos dados válidos
                        dados_validos.push((b.to_string(), lat, long));
                    }
                }
            }
        }
        
        // Agrupa e contar por bairro
        let mut bairro_dados: HashMap<String, (f64, f64, i64)> = HashMap::new();
        
        for (bairro, lat, long) in dados_validos {
            let entry = bairro_dados.entry(bairro).or_insert((lat, long, 0));
            entry.2 += 1;
        }
        
        // Constroi o dicionário organizado final
        let mut organized_data = HashMap::new();
        
        for (bairro, (lat, long, quantidade)) in bairro_dados {
            let mut neighborhood_data = HashMap::new();
            neighborhood_data.insert("latitude".to_string(), json!(lat));
            neighborhood_data.insert("longitude".to_string(), json!(long));
            neighborhood_data.insert("quantidade".to_string(), json!(quantidade));
            
            organized_data.insert(bairro, json!(neighborhood_data));
        }
        
        Ok(json!(organized_data))
    }
}