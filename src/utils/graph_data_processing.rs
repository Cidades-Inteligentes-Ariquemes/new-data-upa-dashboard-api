use chrono::{Duration, NaiveDate};
use polars::lazy::dsl::{col, lit};
use polars::prelude::*;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::error::Error;

type DiseaseLocationMap = HashMap<String, HashMap<String, HashMap<String, (f64, f64, i64)>>>;

const DATE_FMT: &str = "%Y-%m-%d";
const KEY_60: &str = "ultimos_60_dias";
const KEY_90: &str = "ultimos_90_dias";
const KEY_TODOS: &str = "todos";
const TABLE_ACOLHIMENTO: &str = "Acolhimento";
const TABLE_CONSULTA_MEDICA: &str = "ConsultaMedica";
const ROLE_MEDICO_CLINICO: &str = "MEDICO CLINICO";
const ROLE_MEDICO_CIRURGIAO_GERAL: &str = "MEDICO CIRURGIAO GERAL";
const ROLE_ENFERMEIRO: &str = "ENFERMEIRO";

/// Calcula `(cutoff_60, cutoff_90)` a partir da maior data presente no vetor.
/// Datas inválidas/vazias são ignoradas. Retorna `None` se nenhuma data for parseável.
fn compute_recent_cutoffs(dates: &[String]) -> Option<(NaiveDate, NaiveDate)> {
    let max_date = dates
        .iter()
        .filter_map(|s| NaiveDate::parse_from_str(s, DATE_FMT).ok())
        .max()?;
    Some((max_date - Duration::days(60), max_date - Duration::days(90)))
}

/// Retorna `(within_60, within_90)` para uma string de data; `(false, false)` se não parsear.
fn date_within(date_str: &str, cutoff_60: &NaiveDate, cutoff_90: &NaiveDate) -> (bool, bool) {
    match NaiveDate::parse_from_str(date_str, DATE_FMT) {
        Ok(d) => (d >= *cutoff_60, d >= *cutoff_90),
        Err(_) => (false, false),
    }
}

/// Garante que `KEY_60` e `KEY_90` existam no mapa (insere 0 se ausentes).
/// Idempotente — pode ser chamado múltiplas vezes sem efeito colateral.
fn ensure_recent_keys(map: &mut HashMap<String, i64>) {
    map.entry(KEY_60.to_string()).or_insert(0);
    map.entry(KEY_90.to_string()).or_insert(0);
}

fn any_value_to_string(value: AnyValue<'_>) -> Option<String> {
    match value {
        AnyValue::Null => None,
        _ => Some(
            value
                .get_str()
                .map(ToString::to_string)
                .unwrap_or_else(|| value.str_value().into_owned()),
        ),
    }
}

fn get_non_empty_cell_string(df: &DataFrame, column_name: &str, row_idx: usize) -> Option<String> {
    df.column(column_name)
        .ok()?
        .get(row_idx)
        .ok()
        .and_then(any_value_to_string)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn row_matches_table_name(df: &DataFrame, row_idx: usize, expected: &str) -> bool {
    matches!(
        get_non_empty_cell_string(df, "ifrotabelanome", row_idx).as_deref(),
        Some(table_name) if table_name == expected
    )
}

fn increment_competencia_unique_count(
    seen: &mut HashSet<(String, String)>,
    counts: &mut HashMap<String, i64>,
    competencia: &str,
    unique_id: &str,
) {
    if seen.insert((competencia.to_string(), unique_id.to_string())) {
        *counts.entry(competencia.to_string()).or_insert(0) += 1;
    }
}

fn increment_grouped_unique_count(
    seen: &mut HashSet<(String, String, String)>,
    counts: &mut HashMap<String, HashMap<String, i64>>,
    group: &str,
    competencia: &str,
    unique_id: &str,
) {
    if seen.insert((
        group.to_string(),
        competencia.to_string(),
        unique_id.to_string(),
    )) {
        let group_counts = counts.entry(group.to_string()).or_default();
        *group_counts.entry(KEY_TODOS.to_string()).or_insert(0) += 1;
        *group_counts.entry(competencia.to_string()).or_insert(0) += 1;
    }
}

fn increment_summary_with_recent_keys(
    counts: &mut HashMap<String, i64>,
    competencia: &str,
    in_60: bool,
    in_90: bool,
) {
    *counts.entry(competencia.to_string()).or_insert(0) += 1;
    *counts.entry(KEY_TODOS.to_string()).or_insert(0) += 1;
    if in_60 {
        *counts.entry(KEY_60.to_string()).or_insert(0) += 1;
    }
    if in_90 {
        *counts.entry(KEY_90.to_string()).or_insert(0) += 1;
    }
}

fn increment_group_with_recent_keys(
    counts: &mut HashMap<String, HashMap<String, i64>>,
    group: &str,
    competencia: &str,
    in_60: bool,
    in_90: bool,
) {
    let group_counts = counts.entry(group.to_string()).or_default();
    ensure_recent_keys(group_counts);
    increment_summary_with_recent_keys(group_counts, competencia, in_60, in_90);
}

fn translate_day_of_week(day_name: &str) -> Option<&'static str> {
    match day_name {
        "Monday" => Some("segunda-feira"),
        "Tuesday" => Some("terça-feira"),
        "Wednesday" => Some("quarta-feira"),
        "Thursday" => Some("quinta-feira"),
        "Friday" => Some("sexta-feira"),
        "Saturday" => Some("sábado"),
        "Sunday" => Some("domingo"),
        _ => None,
    }
}

fn hour_group_from_time(time_str: &str) -> Option<&'static str> {
    if time_str.len() < 2 {
        return None;
    }

    let hour = time_str[0..2].parse::<i32>().ok()?;

    match hour {
        0..=1 => Some("00h-02h"),
        2..=3 => Some("02h-04h"),
        4..=5 => Some("04h-06h"),
        6..=7 => Some("06h-08h"),
        8..=9 => Some("08h-10h"),
        10..=11 => Some("10h-12h"),
        12..=13 => Some("12h-14h"),
        14..=15 => Some("14h-16h"),
        16..=17 => Some("16h-18h"),
        18..=19 => Some("18h-20h"),
        20..=21 => Some("20h-22h"),
        22..=23 => Some("22h-24h"),
        _ => None,
    }
}

pub struct DataProcessingForGraphPlotting;

impl DataProcessingForGraphPlotting {
    // Função para obter colunas para plotagem
    pub fn columns_to_plot_graphs() -> HashMap<String, Value> {
        let mut result = HashMap::new();

        // Tabelas
        let mut tables = HashMap::new();
        tables.insert("bpa".to_string(), json!("bpa"));
        tables.insert(
            "map_neighbourhoods".to_string(),
            json!("map_neighbourhoods"),
        );
        tables.insert("non_doctors".to_string(), json!("non_doctors"));
        tables.insert("non_nurse".to_string(), json!("non_nurse"));

        // Colunas
        let mut columns = HashMap::new();
        columns.insert(
            "number_of_appointments_per_month".to_string(),
            json!(["ifrocompetencia", "ifrotabelaid", "ifrotabelanome"]),
        );
        columns.insert(
            "number_of_appointments_per_flow".to_string(),
            json!(["ifrocompetencia", "ifrotabelanome", "ifrotabelaid"]),
        );
        columns.insert(
            "distribuition_of_patients_ages".to_string(),
            json!([
                "ifrocompetencia",
                "ifropacienteidade",
                "ifrotabelaid",
                "ifrotabelanome"
            ]),
        );
        columns.insert(
            "number_of_calls_per_day_of_the_week".to_string(),
            json!([
                "ifrocompetencia",
                "ifrodiasemana",
                "ifrotabelaid",
                "ifrotabelanome"
            ]),
        );
        columns.insert(
            "distribution_of_services_by_hour_group".to_string(),
            json!([
                "ifrocompetencia",
                "ifrohoraatendimento",
                "ifroprofissionalcbods",
                "ifrotabelaid",
                "ifrotabelanome"
            ]),
        );
        columns.insert(
            "number_of_visits_per_nurse".to_string(),
            json!([
                "ifrocompetencia",
                "ifroprofissionalid",
                "ifroprofissionalcbods",
                "ifroprofissionalnome",
                "ifrotabelanome",
                "ifrotabelaid"
            ]),
        );
        columns.insert(
            "number_of_visits_per_doctor".to_string(),
            json!([
                "ifrocompetencia",
                "ifroprofissionalid",
                "ifroprofissionalcbods",
                "ifroprofissionalnome",
                "ifrotabelanome",
                "ifrotabelaid"
            ]),
        );
        columns.insert(
            "number_of_appointments_without_medical_consultation".to_string(),
            json!(["ifrocompetencia", "ifrotabelaid", "ifrotabelanome"]),
        );
        columns.insert(
            "average_time_per_doctor".to_string(),
            json!([
                "ifrocompetencia",
                "ifrohoraatendimento",
                "ifroprofissionalid",
                "ifroprofissionalcbods",
                "ifroprofissionalnome",
                "ifrotabelanome"
            ]),
        );
        columns.insert(
            "heat_map_with_disease_indication".to_string(),
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
        );
        columns.insert(
            "heat_map_with_the_number_of_medical_appointments_by_neighborhood".to_string(),
            json!([
                "ifrocompetencia",
                "ifropacienteendereco",
                "ifropacientebairro",
                "ifropacientelatitude",
                "ifropacientelongitude",
                "ifrotabelaid",
                "ifrotabelanome"
            ]),
        );

        result.insert("tables".to_string(), json!(tables));
        result.insert("columns".to_string(), json!(columns));

        json!(result)
            .as_object()
            .unwrap()
            .clone()
            .into_iter()
            .collect()
    }

    // Implementação das funções de processamento
    pub async fn create_dict_to_number_of_appointments_per_month(
        &self,
        df: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let mut counts: HashMap<String, i64> = HashMap::new();
        let mut seen = HashSet::new();

        for row_idx in 0..df.height() {
            if !row_matches_table_name(df, row_idx, TABLE_ACOLHIMENTO) {
                continue;
            }

            let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", row_idx)
            else {
                continue;
            };

            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", row_idx) else {
                continue;
            };

            increment_competencia_unique_count(
                &mut seen,
                &mut counts,
                &competencia,
                &ifrotabelaid,
            );
        }

        Ok(json!(counts))
    }

    pub async fn create_dict_to_number_of_appointments_per_flow(
        &self,
        df: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let mut counts: HashMap<String, HashMap<String, i64>> = HashMap::new();
        let mut seen = HashSet::new();

        for row_idx in 0..df.height() {
            let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", row_idx)
            else {
                continue;
            };

            let Some(tabela_nome) = get_non_empty_cell_string(df, "ifrotabelanome", row_idx)
            else {
                continue;
            };

            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", row_idx) else {
                continue;
            };

            increment_grouped_unique_count(
                &mut seen,
                &mut counts,
                &tabela_nome,
                &competencia,
                &ifrotabelaid,
            );
        }

        Ok(json!(counts))
    }

    pub async fn create_dict_to_distribuition_of_patients_ages_from_raw(
        &self,
        raw_data: &HashMap<String, Vec<Value>>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let competencias = match raw_data.get("ifrocompetencia") {
            Some(comp_values) => comp_values,
            None => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Coluna ifrocompetencia não encontrada",
                )));
            }
        };

        let idades = match raw_data.get("ifropacienteidade") {
            Some(age_values) => age_values,
            None => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Coluna ifropacienteidade não encontrada",
                )));
            }
        };

        let ifrotabelaids = match raw_data.get("ifrotabelaid") {
            Some(values) => values,
            None => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Coluna ifrotabelaid não encontrada",
                )));
            }
        };

        let tabelas = match raw_data.get("ifrotabelanome") {
            Some(values) => values,
            None => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Coluna ifrotabelanome não encontrada",
                )));
            }
        };

        if competencias.len() != idades.len()
            || competencias.len() != ifrotabelaids.len()
            || competencias.len() != tabelas.len()
        {
            return Err(Box::<dyn Error + Send + Sync>::from(
                "Tamanhos de colunas incompatíveis",
            ));
        }

        // Definir faixas etárias
        let age_groups = vec![
            ("0 a 19", 0..20),
            ("20 a 39", 20..40),
            ("40 a 59", 40..60),
            ("60 a 79", 60..80),
            ("80 a 100", 80..101),
            ("+ de 100", 101..i32::MAX),
        ];

        // Contadores para cada faixa etária
        let mut age_data: HashMap<&str, HashMap<String, i64>> = HashMap::new();
        for (group, _) in &age_groups {
            age_data.insert(group, HashMap::new());
        }

        let mut seen = HashSet::new();

        for i in 0..competencias.len() {
            let tabela_nome = match &tabelas[i] {
                Value::String(s) => s.trim(),
                _ => "",
            };

            if tabela_nome != TABLE_ACOLHIMENTO {
                continue;
            }

            let ifrotabelaid = match &ifrotabelaids[i] {
                Value::Number(n) => n.to_string(),
                Value::String(s) if !s.trim().is_empty() => s.trim().to_string(),
                _ => continue,
            };

            let competencia = match &competencias[i] {
                Value::String(s) if !s.trim().is_empty() => s.trim().to_string(),
                _ => continue,
            };

            let idade = match &idades[i] {
                Value::Number(n) => {
                    if let Some(num) = n.as_i64() {
                        num as i32
                    } else if let Some(num) = n.as_f64() {
                        num as i32
                    } else {
                        -1
                    }
                }
                Value::String(s) => s.parse::<i32>().unwrap_or(-1),
                _ => -1,
            };

            if idade >= 0 {
                for (group, range) in &age_groups {
                    if range.contains(&idade)
                        && seen.insert((group.to_string(), competencia.clone(), ifrotabelaid.clone()))
                    {
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

        let mut result = HashMap::new();
        for (group, _) in &age_groups {
            let counts = &age_data[group];
            let total: i64 = counts.values().sum();

            let mut group_data = HashMap::new();
            group_data.insert(KEY_TODOS.to_string(), json!(total));

            for (comp, count) in counts {
                group_data.insert(comp.clone(), json!(count));
            }

            result.insert(group.to_string(), json!(group_data));
        }

        Ok(json!(result))
    }

    pub async fn create_dict_to_number_of_calls_per_day_of_the_week(
        &self,
        df: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let ordered_days = [
            "segunda-feira",
            "terça-feira",
            "quarta-feira",
            "quinta-feira",
            "sexta-feira",
            "sábado",
            "domingo",
        ];
        let mut organized_data = HashMap::new();
        let mut seen = HashSet::new();

        for row_idx in 0..df.height() {
            if !row_matches_table_name(df, row_idx, TABLE_ACOLHIMENTO) {
                continue;
            }

            let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", row_idx)
            else {
                continue;
            };

            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", row_idx) else {
                continue;
            };

            let Some(day_name) = get_non_empty_cell_string(df, "ifrodiasemana", row_idx) else {
                continue;
            };

            let Some(day_name_pt) = translate_day_of_week(&day_name) else {
                continue;
            };

            increment_grouped_unique_count(
                &mut seen,
                &mut organized_data,
                day_name_pt,
                &competencia,
                &ifrotabelaid,
            );
        }

        for day in ordered_days {
            organized_data
                .entry(day.to_string())
                .or_insert_with(|| HashMap::from([(KEY_TODOS.to_string(), 0_i64)]));
        }

        Ok(json!(organized_data))
    }

    pub async fn create_dict_to_distribution_of_services_by_hour_group(
        &self,
        df: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let all_groups = [
            "00h-02h", "02h-04h", "04h-06h", "06h-08h", "08h-10h", "10h-12h", "12h-14h",
            "14h-16h", "16h-18h", "18h-20h", "20h-22h", "22h-24h",
        ];
        let mut hour_group_data: HashMap<String, HashMap<String, i64>> = HashMap::new();
        let mut seen = HashSet::new();

        for row_idx in 0..df.height() {
            if !row_matches_table_name(df, row_idx, TABLE_CONSULTA_MEDICA) {
                continue;
            }

            let Some(especialidade) =
                get_non_empty_cell_string(df, "ifroprofissionalcbods", row_idx)
            else {
                continue;
            };

            if especialidade != ROLE_MEDICO_CLINICO {
                continue;
            }

            let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", row_idx)
            else {
                continue;
            };

            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", row_idx) else {
                continue;
            };

            let Some(time_str) = get_non_empty_cell_string(df, "ifrohoraatendimento", row_idx)
            else {
                continue;
            };

            let Some(group) = hour_group_from_time(&time_str) else {
                continue;
            };

            increment_grouped_unique_count(
                &mut seen,
                &mut hour_group_data,
                group,
                &competencia,
                &ifrotabelaid,
            );
        }

        for group in all_groups {
            hour_group_data
                .entry(group.to_string())
                .or_insert_with(|| HashMap::from([(KEY_TODOS.to_string(), 0_i64)]));
        }

        Ok(json!(hour_group_data))
    }

    pub async fn create_dict_to_number_of_visits_per_nurse(
        &self,
        df: &DataFrame,
        df_non_nurse: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let non_nurse_names: HashSet<String> = df_non_nurse
            .column("ifroprofissionalnome")?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect();

        let mut organized_data = HashMap::new();
        let mut seen = HashSet::new();

        for row_idx in 0..df.height() {
            if !row_matches_table_name(df, row_idx, TABLE_ACOLHIMENTO) {
                continue;
            }

            let Some(role) = get_non_empty_cell_string(df, "ifroprofissionalcbods", row_idx) else {
                continue;
            };

            if role != ROLE_ENFERMEIRO {
                continue;
            }

            let Some(nurse_name) =
                get_non_empty_cell_string(df, "ifroprofissionalnome", row_idx)
            else {
                continue;
            };

            if non_nurse_names.contains(&nurse_name) {
                continue;
            }

            let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", row_idx)
            else {
                continue;
            };

            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", row_idx) else {
                continue;
            };

            increment_grouped_unique_count(
                &mut seen,
                &mut organized_data,
                &nurse_name,
                &competencia,
                &ifrotabelaid,
            );
        }

        Ok(json!(organized_data))
    }

    pub async fn create_dict_to_number_of_visits_per_doctor(
        &self,
        df: &DataFrame,
        df_non_doctors: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let non_doctor_names: HashSet<String> = df_non_doctors
            .column("ifroprofissionalnome")?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect();

        let mut organized_data = HashMap::new();
        let mut seen = HashSet::new();

        for row_idx in 0..df.height() {
            if !row_matches_table_name(df, row_idx, TABLE_CONSULTA_MEDICA) {
                continue;
            }

            let Some(role) = get_non_empty_cell_string(df, "ifroprofissionalcbods", row_idx) else {
                continue;
            };

            if role != ROLE_MEDICO_CLINICO {
                continue;
            }

            let Some(doctor_name) =
                get_non_empty_cell_string(df, "ifroprofissionalnome", row_idx)
            else {
                continue;
            };

            if non_doctor_names.contains(&doctor_name) {
                continue;
            }

            let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", row_idx)
            else {
                continue;
            };

            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", row_idx) else {
                continue;
            };

            increment_grouped_unique_count(
                &mut seen,
                &mut organized_data,
                &doctor_name,
                &competencia,
                &ifrotabelaid,
            );
        }

        Ok(json!(organized_data))
    }

    pub async fn create_dict_to_number_of_appointments_without_medical_consultation(
        &self,
        df: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let mut acolhimentos = HashSet::new();
        let mut consultas = HashSet::new();

        for row_idx in 0..df.height() {
            let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", row_idx)
            else {
                continue;
            };

            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", row_idx) else {
                continue;
            };

            if row_matches_table_name(df, row_idx, TABLE_ACOLHIMENTO) {
                acolhimentos.insert((competencia.clone(), ifrotabelaid.clone()));
            }

            if row_matches_table_name(df, row_idx, TABLE_CONSULTA_MEDICA) {
                consultas.insert((competencia, ifrotabelaid));
            }
        }

        let mut acolhimentos_por_competencia = HashMap::new();
        for (competencia, _) in &acolhimentos {
            *acolhimentos_por_competencia
                .entry(competencia.clone())
                .or_insert(0_i64) += 1;
        }

        let mut consultas_por_competencia = HashMap::new();
        for (competencia, _) in &consultas {
            *consultas_por_competencia
                .entry(competencia.clone())
                .or_insert(0_i64) += 1;
        }

        let mut competencias = HashSet::new();
        competencias.extend(acolhimentos_por_competencia.keys().cloned());
        competencias.extend(consultas_por_competencia.keys().cloned());

        let mut result = HashMap::new();
        let mut total = 0_i64;

        for competencia in competencias {
            let acolhimentos_total = acolhimentos_por_competencia
                .get(&competencia)
                .copied()
                .unwrap_or(0);
            let consultas_total = consultas_por_competencia
                .get(&competencia)
                .copied()
                .unwrap_or(0);
            let without_consultation = acolhimentos_total - consultas_total;

            total += without_consultation;
            result.insert(competencia, json!(without_consultation));
        }

        result.insert(KEY_TODOS.to_string(), json!(total));

        Ok(json!(result))
    }

    pub async fn create_dict_to_average_time_in_minutes_per_doctor(
        &self,
        df: &DataFrame,
        df_non_doctors: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        // Obter lista de médicos a excluir
        let non_doctor_names: Vec<String> = df_non_doctors
            .column("ifroprofissionalnome")?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect();

        // Filtrar DataFrame para médicos e consulta médica com hora de atendimento
        let df_doctor_consulta = df
            .clone()
            .lazy()
            .filter(
                (col("ifroprofissionalcbods")
                    .eq(lit("MEDICO CLINICO"))
                    .or(col("ifroprofissionalcbods").eq(lit("MEDICO CIRURGIAO GERAL"))))
                .and(col("ifrotabelanome").eq(lit("ConsultaMedica")))
                .and(col("ifrohoraatendimento").is_not_null()),
            )
            .collect()?;

        // Filtrar nomes não desejados
        let mut keep_rows = Vec::with_capacity(df_doctor_consulta.height());

        for i in 0..df_doctor_consulta.height() {
            let nome = df_doctor_consulta
                .column("ifroprofissionalnome")?
                .str()?
                .get(i)
                .unwrap_or("");
            let keep = !non_doctor_names.contains(&nome.to_string());
            keep_rows.push(keep);
        }

        // Converter para Series e filtrar
        let mask = BooleanChunked::new("mask".into(), keep_rows);
        let df_filtered = df_doctor_consulta.filter(&mask)?;

        // Calcular tempo em minutos
        let mut minutes_values = Vec::with_capacity(df_filtered.height());

        for i in 0..df_filtered.height() {
            let time_str = df_filtered
                .column("ifrohoraatendimento")?
                .str()?
                .get(i)
                .unwrap_or("");
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
        let valid_times_mask = df_with_time.column("time_minutes")?.f64()?.gt(0.0);

        let df_valid_times = df_with_time.filter(&valid_times_mask)?;

        // Obter nomes únicos de médicos
        let unique_doctors = df_valid_times
            .column("ifroprofissionalnome")?
            .unique()?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect::<Vec<String>>();

        // Criar dicionário organizado para cada médico
        let mut organized_data = HashMap::new();

        for doctor_name in unique_doctors {
            // Filtrar para este médico
            let doctor_mask = df_valid_times
                .column("ifroprofissionalnome")?
                .str()?
                .equal(doctor_name.as_str());

            let df_doctor = df_valid_times.filter(&doctor_mask)?;

            // Calcular média total do médico
            let time_values = df_doctor.column("time_minutes")?.f64()?;
            let sum: f64 = time_values
                .iter()
                .fold(0.0, |acc, opt_val| acc + opt_val.unwrap_or(0.0));

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
            let competencias = df_doctor
                .column("ifrocompetencia")?
                .unique()?
                .str()?
                .into_iter()
                .filter_map(|opt_s| opt_s.map(String::from))
                .collect::<Vec<String>>();

            for competencia in competencias {
                let comp_mask = df_doctor
                    .column("ifrocompetencia")?
                    .str()?
                    .equal(competencia.as_str());

                let df_comp = df_doctor.filter(&comp_mask)?;

                // Calcular média para esta competência
                let comp_time_values = df_comp.column("time_minutes")?.f64()?;
                let comp_sum: f64 = comp_time_values
                    .iter()
                    .fold(0.0, |acc, opt_val| acc + opt_val.unwrap_or(0.0));

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

    pub async fn create_dictionary_with_location_and_number_per_disease(
        &self,
        df: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let mut dados_por_queixa: DiseaseLocationMap = HashMap::new();
        let mut seen = HashSet::new();

        for i in 0..df.height() {
            if !row_matches_table_name(df, i, TABLE_ACOLHIMENTO) {
                continue;
            }

            let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", i) else {
                continue;
            };
            let Some(endereco) = get_non_empty_cell_string(df, "ifropacienteendereco", i) else {
                continue;
            };
            if endereco == "DO IPE" {
                continue;
            }
            let Some(bairro) = get_non_empty_cell_string(df, "ifropacientebairro", i) else {
                continue;
            };
            let Some(queixa) = get_non_empty_cell_string(df, "ifropacientequeixaprincipal", i)
            else {
                continue;
            };
            let Some(latitude) = get_non_empty_cell_string(df, "ifropacientelatitude", i) else {
                continue;
            };
            let Some(longitude) = get_non_empty_cell_string(df, "ifropacientelongitude", i) else {
                continue;
            };
            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", i) else {
                continue;
            };

            let (Ok(lat), Ok(long)) = (latitude.parse::<f64>(), longitude.parse::<f64>()) else {
                continue;
            };

            if !seen.insert((
                queixa.clone(),
                competencia.clone(),
                bairro.clone(),
                ifrotabelaid,
            )) {
                continue;
            }

            if !dados_por_queixa.contains_key(&queixa) {
                dados_por_queixa.insert(queixa.clone(), HashMap::new());
            }

            let comp_map = dados_por_queixa.get_mut(&queixa).unwrap();

            if !comp_map.contains_key(&competencia) {
                comp_map.insert(competencia.clone(), HashMap::new());
            }

            let bairro_map = comp_map.get_mut(&competencia).unwrap();

            let entry = bairro_map.entry(bairro.clone()).or_insert((lat, long, 0));
            entry.2 += 1;

            if !comp_map.contains_key(KEY_TODOS) {
                comp_map.insert(KEY_TODOS.to_string(), HashMap::new());
            }

            let todos_map = comp_map.get_mut(KEY_TODOS).unwrap();
            let todos_entry = todos_map.entry(bairro.clone()).or_insert((lat, long, 0));
            todos_entry.2 += 1;
        }

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

    pub async fn create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood(
        &self,
        df: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let mut bairro_dados: HashMap<String, (f64, f64, i64)> = HashMap::new();
        let mut seen = HashSet::new();

        for i in 0..df.height() {
            if !row_matches_table_name(df, i, TABLE_CONSULTA_MEDICA) {
                continue;
            }

            let Some(endereco) = get_non_empty_cell_string(df, "ifropacienteendereco", i) else {
                continue;
            };
            if endereco == "DO IPE" {
                continue;
            }
            let Some(bairro) = get_non_empty_cell_string(df, "ifropacientebairro", i) else {
                continue;
            };
            let Some(latitude) = get_non_empty_cell_string(df, "ifropacientelatitude", i) else {
                continue;
            };
            let Some(longitude) = get_non_empty_cell_string(df, "ifropacientelongitude", i) else {
                continue;
            };
            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", i) else {
                continue;
            };

            let (Ok(lat), Ok(long)) = (latitude.parse::<f64>(), longitude.parse::<f64>()) else {
                continue;
            };

            if !seen.insert((bairro.clone(), ifrotabelaid)) {
                continue;
            }

            let entry = bairro_dados.entry(bairro).or_insert((lat, long, 0));
            entry.2 += 1;
        }

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

    pub async fn create_dict_to_number_of_appointments_per_cid(
        &self,
        df: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let dates = df
            .column("ifrodataatendimento")?
            .str()?
            .into_iter()
            .enumerate()
            .filter(|(row_idx, _)| row_matches_table_name(df, *row_idx, TABLE_CONSULTA_MEDICA))
            .map(|(_, opt_s)| opt_s.unwrap_or("").to_string())
            .collect::<Vec<String>>();

        let cutoffs = compute_recent_cutoffs(&dates);

        let mut por_cid: HashMap<String, HashMap<String, i64>> = HashMap::new();
        let mut informado: HashMap<String, i64> = HashMap::new();
        let mut nao_informado: HashMap<String, i64> = HashMap::new();
        informado.insert(KEY_TODOS.to_string(), 0);
        informado.insert(KEY_60.to_string(), 0);
        informado.insert(KEY_90.to_string(), 0);
        nao_informado.insert(KEY_TODOS.to_string(), 0);
        nao_informado.insert(KEY_60.to_string(), 0);
        nao_informado.insert(KEY_90.to_string(), 0);

        let mut consultas_base: HashMap<(String, String), (bool, bool)> = HashMap::new();
        let mut cids_por_atendimento: HashMap<(String, String), HashSet<String>> = HashMap::new();

        for row_idx in 0..df.height() {
            if !row_matches_table_name(df, row_idx, TABLE_CONSULTA_MEDICA) {
                continue;
            }

            let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", row_idx)
            else {
                continue;
            };
            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", row_idx) else {
                continue;
            };
            let date_str =
                get_non_empty_cell_string(df, "ifrodataatendimento", row_idx).unwrap_or_default();
            let (in_60, in_90) = match &cutoffs {
                Some((c60, c90)) => date_within(&date_str, c60, c90),
                None => (false, false),
            };

            let event_entry = consultas_base
                .entry((competencia.clone(), ifrotabelaid.clone()))
                .or_insert((false, false));
            event_entry.0 |= in_60;
            event_entry.1 |= in_90;
        }

        for row_idx in 0..df.height() {
            let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", row_idx)
            else {
                continue;
            };
            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", row_idx) else {
                continue;
            };

            let event_key = (competencia, ifrotabelaid);
            if !consultas_base.contains_key(&event_key) {
                continue;
            }

            if let Some(cid) = get_non_empty_cell_string(df, "ifrocidcd", row_idx) {
                cids_por_atendimento.entry(event_key).or_default().insert(cid);
            }
        }

        for ((competencia, ifrotabelaid), (in_60, in_90)) in consultas_base {
            let event_cids = cids_por_atendimento
                .get(&(competencia.clone(), ifrotabelaid))
                .cloned()
                .unwrap_or_default();

            if event_cids.is_empty() {
                increment_summary_with_recent_keys(&mut nao_informado, &competencia, in_60, in_90);
            } else {
                increment_summary_with_recent_keys(&mut informado, &competencia, in_60, in_90);

                for cid in event_cids {
                    increment_group_with_recent_keys(
                        &mut por_cid,
                        &cid,
                        &competencia,
                        in_60,
                        in_90,
                    );
                }
            }
        }

        let mut result = HashMap::new();
        result.insert("por_cid".to_string(), json!(por_cid));
        result.insert("informado".to_string(), json!(informado));
        result.insert("nao_informado".to_string(), json!(nao_informado));

        Ok(json!(result))
    }

    pub async fn create_dict_to_number_of_appointments_per_classification(
        &self,
        df: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        Ok(aggregate_by_classification(df, TABLE_ACOLHIMENTO, false))
    }

    pub async fn create_dict_to_number_of_medical_appointments_per_classification(
        &self,
        df: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        Ok(aggregate_by_classification(
            df,
            TABLE_CONSULTA_MEDICA,
            true,
        ))
    }
}

fn aggregate_by_classification(
    df: &DataFrame,
    expected_table_name: &str,
    medical_roles_only: bool,
) -> Value {
    let known_classes = [
        "NaoUrgente",
        "PoucoUrgente",
        "Urgente",
        "MuitoUrgente",
        "Emergencia",
    ];

    let dates = df
        .column("ifrodataatendimento")
        .ok()
        .and_then(|column| column.str().ok())
        .map(|column| {
            column
                .into_iter()
                .map(|opt_s| opt_s.unwrap_or("").to_string())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    let cutoffs = compute_recent_cutoffs(&dates);

    let mut by_class: HashMap<String, HashMap<String, i64>> = HashMap::new();
    let mut todos: HashMap<String, i64> = HashMap::new();
    todos.insert(KEY_TODOS.to_string(), 0);
    todos.insert(KEY_60.to_string(), 0);
    todos.insert(KEY_90.to_string(), 0);

    for class in &known_classes {
        let mut inner = HashMap::new();
        inner.insert(KEY_TODOS.to_string(), 0_i64);
        inner.insert(KEY_60.to_string(), 0_i64);
        inner.insert(KEY_90.to_string(), 0_i64);
        by_class.insert((*class).to_string(), inner);
    }

    let mut eventos_por_classificacao: HashMap<(String, String, String), (bool, bool)> =
        HashMap::new();
    let mut eventos_totais: HashMap<(String, String), (bool, bool)> = HashMap::new();

    for row_idx in 0..df.height() {
        if !row_matches_table_name(df, row_idx, expected_table_name) {
            continue;
        }

        if medical_roles_only {
            let Some(role) = get_non_empty_cell_string(df, "ifroprofissionalcbods", row_idx) else {
                continue;
            };

            if role != ROLE_MEDICO_CLINICO && role != ROLE_MEDICO_CIRURGIAO_GERAL {
                continue;
            }
        }

        let Some(classification) = get_non_empty_cell_string(df, "ifroclassificacao", row_idx)
        else {
            continue;
        };
        let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", row_idx) else {
            continue;
        };
        let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", row_idx) else {
            continue;
        };
        let date_str =
            get_non_empty_cell_string(df, "ifrodataatendimento", row_idx).unwrap_or_default();
        let (in_60, in_90) = match &cutoffs {
            Some((c60, c90)) => date_within(&date_str, c60, c90),
            None => (false, false),
        };

        let class_entry = eventos_por_classificacao
            .entry((classification, competencia.clone(), ifrotabelaid.clone()))
            .or_insert((false, false));
        class_entry.0 |= in_60;
        class_entry.1 |= in_90;

        let total_entry = eventos_totais
            .entry((competencia, ifrotabelaid))
            .or_insert((false, false));
        total_entry.0 |= in_60;
        total_entry.1 |= in_90;
    }

    for ((class, competencia, _), (in_60, in_90)) in eventos_por_classificacao {
        increment_group_with_recent_keys(&mut by_class, &class, &competencia, in_60, in_90);
    }

    for ((competencia, _), (in_60, in_90)) in eventos_totais {
        increment_summary_with_recent_keys(&mut todos, &competencia, in_60, in_90);
    }

    let mut result = HashMap::new();
    result.insert(KEY_TODOS.to_string(), json!(todos));
    for (class, comp_counts) in by_class {
        result.insert(class, json!(comp_counts));
    }

    json!(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    #[test]
    fn get_non_empty_cell_string_returns_unquoted_utf8_values() {
        let df = df!(
            "ifrotabelanome" => ["Acolhimento"],
            "ifrocompetencia" => ["2026-3"]
        )
        .unwrap();

        assert_eq!(
            get_non_empty_cell_string(&df, "ifrotabelanome", 0),
            Some("Acolhimento".to_string())
        );
        assert_eq!(
            get_non_empty_cell_string(&df, "ifrocompetencia", 0),
            Some("2026-3".to_string())
        );
    }

    #[test]
    fn number_of_appointments_per_month_counts_unique_acolhimento_ids() {
        let df = df!(
            "ifrocompetencia" => ["2026-3", "2026-3", "2026-3", "2026-3"],
            "ifrotabelaid" => [1i64, 1, 2, 3],
            "ifrotabelanome" => ["Acolhimento", "Acolhimento", "Acolhimento", "Procedimento"]
        )
        .unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_number_of_appointments_per_month(&df),
        )
        .unwrap();

        assert_eq!(result, json!({ "2026-3": 2 }));
    }

    #[test]
    fn number_of_appointments_per_cid_uses_all_rows_from_the_same_event() {
        let df = df!(
            "ifrocompetencia" => [
                "2026-3", "2026-3", "2026-3",
                "2026-3", "2026-3",
                "2026-3"
            ],
            "ifrodataatendimento" => [
                "2026-03-10", "2026-03-10", "2026-03-10",
                "2026-03-11", "2026-03-11",
                "2026-03-12"
            ],
            "ifrotabelaid" => [1i64, 1, 1, 2, 2, 3],
            "ifrotabelanome" => [
                "ConsultaMedica", "Procedimento", "Procedimento",
                "ConsultaMedica", "Procedimento",
                "ConsultaMedica"
            ],
            "ifrocidcd" => ["", "L029", "R05", "", "", "J00"]
        )
        .unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_number_of_appointments_per_cid(&df),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "por_cid": {
                    "J00": {
                        "2026-3": 1,
                        "todos": 1,
                        "ultimos_60_dias": 1,
                        "ultimos_90_dias": 1
                    },
                    "L029": {
                        "2026-3": 1,
                        "todos": 1,
                        "ultimos_60_dias": 1,
                        "ultimos_90_dias": 1
                    },
                    "R05": {
                        "2026-3": 1,
                        "todos": 1,
                        "ultimos_60_dias": 1,
                        "ultimos_90_dias": 1
                    }
                },
                "informado": {
                    "2026-3": 2,
                    "todos": 2,
                    "ultimos_60_dias": 2,
                    "ultimos_90_dias": 2
                },
                "nao_informado": {
                    "2026-3": 1,
                    "todos": 1,
                    "ultimos_60_dias": 1,
                    "ultimos_90_dias": 1
                }
            })
        );
    }
}
