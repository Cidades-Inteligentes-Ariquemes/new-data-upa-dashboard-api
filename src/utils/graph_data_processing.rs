use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use log::info;
use polars::prelude::*;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;

type DiseaseLocationMap = HashMap<String, HashMap<String, HashMap<String, (f64, f64, i64)>>>;
#[derive(Clone)]
struct DoctorAttendanceRecord {
    doctor_id: String,
    doctor_name: String,
    competencia: String,
    date: NaiveDate,
    timestamp: NaiveDateTime,
}

const DATE_FMT: &str = "%Y-%m-%d";
const TIME_FMT_FRAC: &str = "%H:%M:%S%.f";
const TIME_FMT: &str = "%H:%M:%S";
const TIME_FMT_SHORT: &str = "%H:%M";
const KEY_60: &str = "ultimos_60_dias";
const KEY_90: &str = "ultimos_90_dias";
const KEY_TODOS: &str = "todos";
const TABLE_ACOLHIMENTO: &str = "Acolhimento";
const TABLE_CONSULTA_MEDICA: &str = "ConsultaMedica";
const ROLE_MEDICO_CLINICO: &str = "MEDICO CLINICO";
const ROLE_MEDICO_CIRURGIAO_GERAL: &str = "MEDICO CIRURGIAO GERAL";
const ROLE_ENFERMEIRO: &str = "ENFERMEIRO";
const BUCKET_SEM_ENDERECO: &str = "SEM ENDERECO";
const BUCKET_SEM_BAIRRO: &str = "SEM BAIRRO";
const BUCKET_SEM_COORDENADA: &str = "SEM COORDENADA";
const INVALID_ADDRESS_DO_IPE: &str = "DO IPE";
const BUCKET_INCORRETO: &str = "INCORRETO";
const KEY_DOENCA_NAO_INFERIDA: &str = "doenca_nao_inferida";

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

fn finalize_summary_counts(mut counts: HashMap<String, i64>) -> HashMap<String, i64> {
    let total = counts
        .iter()
        .filter(|(key, _)| key.as_str() != KEY_TODOS)
        .map(|(_, value)| *value)
        .sum();
    counts.insert(KEY_TODOS.to_string(), total);
    counts
}

fn sum_grouped_counts(
    grouped_counts: &HashMap<String, HashMap<String, i64>>,
) -> HashMap<String, i64> {
    let mut totals = HashMap::new();

    for counts in grouped_counts.values() {
        for (key, value) in counts {
            *totals.entry(key.clone()).or_insert(0) += *value;
        }
    }

    totals
}

fn build_difference_counts(
    total_real_counts: &HashMap<String, i64>,
    counted_totals: &HashMap<String, i64>,
) -> HashMap<String, i64> {
    let mut competencias = HashSet::new();
    competencias.extend(
        total_real_counts
            .keys()
            .filter(|key| key.as_str() != KEY_TODOS)
            .cloned(),
    );
    competencias.extend(
        counted_totals
            .keys()
            .filter(|key| key.as_str() != KEY_TODOS)
            .cloned(),
    );

    let mut differences = HashMap::new();
    let mut total = 0_i64;

    for competencia in competencias {
        let difference = total_real_counts.get(&competencia).copied().unwrap_or(0)
            - counted_totals.get(&competencia).copied().unwrap_or(0);
        total += difference;
        differences.insert(competencia, difference);
    }

    differences.insert(KEY_TODOS.to_string(), total);
    differences
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

fn build_visits_response_with_extra_data(
    grouped_counts: HashMap<String, HashMap<String, i64>>,
    total_real_counts: HashMap<String, i64>,
) -> Value {
    let counted_totals = sum_grouped_counts(&grouped_counts);
    let non_counted = build_difference_counts(&total_real_counts, &counted_totals);

    let mut result = serde_json::Map::new();
    for (professional_name, counts) in grouped_counts {
        result.insert(professional_name, json!(counts));
    }

    result.insert(
        "dados_extras".to_string(),
        json!({
            "atendimentos_nao_contabilizados": non_counted,
            "quantidade_total_real": total_real_counts,
        }),
    );

    Value::Object(result)
}

fn parse_attendance_time(time_str: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(time_str, TIME_FMT_FRAC)
        .ok()
        .or_else(|| NaiveTime::parse_from_str(time_str, TIME_FMT).ok())
        .or_else(|| NaiveTime::parse_from_str(time_str, TIME_FMT_SHORT).ok())
}

fn round_to_two_decimals(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn daily_average_interval_minutes(day_records: &[DoctorAttendanceRecord]) -> Option<f64> {
    if day_records.len() < 2 {
        return None;
    }

    let mut sorted = day_records.to_vec();
    sorted.sort_by_key(|r| r.timestamp);

    let mut total = 0.0_f64;
    let mut valid_pairs = 0_usize;

    for pair in sorted.windows(2) {
        let minutes = (pair[1].timestamp - pair[0].timestamp).num_seconds() as f64 / 60.0;
        if minutes > 60.0 {
            continue;
        }
        total += minutes;
        valid_pairs += 1;
    }

    if valid_pairs == 0 {
        return None;
    }

    Some(round_to_two_decimals(total / valid_pairs as f64))
}

fn mean_of_daily_averages(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let sum: f64 = values.iter().sum();
    Some(round_to_two_decimals(sum / values.len() as f64))
}

fn pick_canonical_name(records: &[DoctorAttendanceRecord]) -> String {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for r in records {
        *counts.entry(r.doctor_name.as_str()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(name, _)| name.to_string())
        .unwrap_or_default()
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

fn is_special_heat_map_bucket(bucket_name: &str) -> bool {
    matches!(
        bucket_name,
        BUCKET_INCORRETO | BUCKET_SEM_ENDERECO | BUCKET_SEM_BAIRRO | BUCKET_SEM_COORDENADA
    )
}

pub struct DataProcessingForGraphPlotting;

impl DataProcessingForGraphPlotting {
    fn build_total_real_counts(
        &self,
        df: &DataFrame,
        table_name: &str,
        required_role: Option<&str>,
    ) -> HashMap<String, i64> {
        let mut total_real_counts = HashMap::new();
        let mut seen_total_real = HashSet::new();

        for row_idx in 0..df.height() {
            if !row_matches_table_name(df, row_idx, table_name) {
                continue;
            }

            if let Some(role_expected) = required_role {
                let Some(role) = get_non_empty_cell_string(df, "ifroprofissionalcbods", row_idx)
                else {
                    continue;
                };

                if role != role_expected {
                    continue;
                }
            }

            let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", row_idx)
            else {
                continue;
            };

            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", row_idx) else {
                continue;
            };

            increment_competencia_unique_count(
                &mut seen_total_real,
                &mut total_real_counts,
                &competencia,
                &ifrotabelaid,
            );
        }

        finalize_summary_counts(total_real_counts)
    }

    fn build_grouped_visits_counts(
        &self,
        df: &DataFrame,
        blocked_names: &HashSet<String>,
        table_name: &str,
        required_role: &str,
    ) -> HashMap<String, HashMap<String, i64>> {
        let mut grouped_counts = HashMap::new();
        let mut seen = HashSet::new();

        for row_idx in 0..df.height() {
            if !row_matches_table_name(df, row_idx, table_name) {
                continue;
            }

            let Some(role) = get_non_empty_cell_string(df, "ifroprofissionalcbods", row_idx) else {
                continue;
            };

            if role != required_role {
                continue;
            }

            let Some(professional_name) =
                get_non_empty_cell_string(df, "ifroprofissionalnome", row_idx)
            else {
                continue;
            };

            if blocked_names.contains(&professional_name) {
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
                &mut grouped_counts,
                &professional_name,
                &competencia,
                &ifrotabelaid,
            );
        }

        grouped_counts
    }

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
                "ifrodataatendimento",
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

            increment_competencia_unique_count(&mut seen, &mut counts, &competencia, &ifrotabelaid);
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

            let Some(tabela_nome) = get_non_empty_cell_string(df, "ifrotabelanome", row_idx) else {
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
                        && seen.insert((
                            group.to_string(),
                            competencia.clone(),
                            ifrotabelaid.clone(),
                        ))
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
            "00h-02h", "02h-04h", "04h-06h", "06h-08h", "08h-10h", "10h-12h", "12h-14h", "14h-16h",
            "16h-18h", "18h-20h", "20h-22h", "22h-24h",
        ];
        let mut hour_group_data: HashMap<String, HashMap<String, i64>> = HashMap::new();
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
        let organized_data = self.build_grouped_visits_counts(
            df,
            &non_nurse_names,
            TABLE_ACOLHIMENTO,
            ROLE_ENFERMEIRO,
        );
        let total_real_counts = self.build_total_real_counts(df, TABLE_ACOLHIMENTO, None);

        Ok(build_visits_response_with_extra_data(
            organized_data,
            total_real_counts,
        ))
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
        let organized_data = self.build_grouped_visits_counts(
            df,
            &non_doctor_names,
            TABLE_CONSULTA_MEDICA,
            ROLE_MEDICO_CLINICO,
        );
        let total_real_counts =
            self.build_total_real_counts(df, TABLE_CONSULTA_MEDICA, Some(ROLE_MEDICO_CLINICO));

        Ok(build_visits_response_with_extra_data(
            organized_data,
            total_real_counts,
        ))
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
        let non_doctor_names: HashSet<String> = df_non_doctors
            .column("ifroprofissionalnome")?
            .str()?
            .into_iter()
            .filter_map(|opt_s| opt_s.map(String::from))
            .collect();

        let mut records_by_doctor: BTreeMap<String, Vec<DoctorAttendanceRecord>> = BTreeMap::new();

        for row_idx in 0..df.height() {
            if !row_matches_table_name(df, row_idx, TABLE_CONSULTA_MEDICA) {
                continue;
            }

            let Some(role) = get_non_empty_cell_string(df, "ifroprofissionalcbods", row_idx) else {
                continue;
            };

            if role != ROLE_MEDICO_CLINICO && role != ROLE_MEDICO_CIRURGIAO_GERAL {
                continue;
            }

            let Some(doctor_id) = get_non_empty_cell_string(df, "ifroprofissionalid", row_idx)
            else {
                continue;
            };

            let Some(doctor_name) = get_non_empty_cell_string(df, "ifroprofissionalnome", row_idx)
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

            let Some(date_str) = get_non_empty_cell_string(df, "ifrodataatendimento", row_idx)
            else {
                continue;
            };
            let Some(time_str) = get_non_empty_cell_string(df, "ifrohoraatendimento", row_idx)
            else {
                continue;
            };

            let Ok(date) = NaiveDate::parse_from_str(&date_str, DATE_FMT) else {
                continue;
            };
            let Some(time) = parse_attendance_time(&time_str) else {
                continue;
            };

            records_by_doctor
                .entry(doctor_id.clone())
                .or_default()
                .push(DoctorAttendanceRecord {
                    doctor_id,
                    doctor_name,
                    competencia,
                    date,
                    timestamp: NaiveDateTime::new(date, time),
                });
        }

        // Resolve nome canônico (mais frequente, tiebreak alfabético) para cada ID
        // e detecta colisões (IDs distintos com mesmo nome canônico).
        let mut canonical_name_by_id: BTreeMap<String, String> = BTreeMap::new();
        for (id, records) in &records_by_doctor {
            canonical_name_by_id.insert(id.clone(), pick_canonical_name(records));
        }
        let mut ids_by_canonical: HashMap<String, Vec<String>> = HashMap::new();
        for (id, name) in &canonical_name_by_id {
            ids_by_canonical
                .entry(name.clone())
                .or_default()
                .push(id.clone());
        }

        let mut organized_data: HashMap<String, HashMap<String, Value>> = HashMap::new();

        for (doctor_id, mut records) in records_by_doctor {
            // Etapa 2 — dedup por (doctor_id, timestamp). Mantém a primeira ocorrência.
            let mut seen: HashSet<(String, NaiveDateTime)> = HashSet::new();
            records.retain(|r| seen.insert((r.doctor_id.clone(), r.timestamp)));

            // Agrupa registros por dia (NaiveDate). Cada dia carrega sua competência.
            let mut records_by_day: BTreeMap<NaiveDate, Vec<DoctorAttendanceRecord>> =
                BTreeMap::new();
            for record in records {
                records_by_day
                    .entry(record.date)
                    .or_default()
                    .push(record);
            }

            // Calcula média diária para cada dia (descarta dias sem par válido).
            let mut all_daily_means: Vec<f64> = Vec::new();
            let mut daily_means_by_competencia: HashMap<String, Vec<f64>> = HashMap::new();

            for (_day, day_records) in records_by_day {
                let Some(daily_mean) = daily_average_interval_minutes(&day_records) else {
                    continue;
                };
                let competencia = day_records[0].competencia.clone();
                all_daily_means.push(daily_mean);
                daily_means_by_competencia
                    .entry(competencia)
                    .or_default()
                    .push(daily_mean);
            }

            // Médico sem nenhum dia com par válido é omitido do JSON.
            let Some(todos) = mean_of_daily_averages(&all_daily_means) else {
                continue;
            };

            let mut doctor_data: HashMap<String, Value> = HashMap::new();
            doctor_data.insert(KEY_TODOS.to_string(), json!(todos));
            for (competencia, daily_means) in daily_means_by_competencia {
                if let Some(monthly_mean) = mean_of_daily_averages(&daily_means) {
                    doctor_data.insert(competencia, json!(monthly_mean));
                }
            }

            // Define a chave do médico no JSON: nome puro se o nome canônico é único entre IDs;
            // caso contrário, anexa "(#<id>)" para diferenciar cadastros distintos com mesmo nome.
            let canonical_name = canonical_name_by_id
                .get(&doctor_id)
                .cloned()
                .unwrap_or_default();
            let collides = ids_by_canonical
                .get(&canonical_name)
                .map(|ids| ids.len() > 1)
                .unwrap_or(false);
            let display_key = if collides {
                info!(
                    "average_time.name_collision id={} canonical_name={}",
                    doctor_id, canonical_name
                );
                format!("{} (#{})", canonical_name, doctor_id)
            } else {
                canonical_name
            };

            organized_data.insert(display_key, doctor_data);
        }

        Ok(json!(organized_data))
    }

    pub async fn create_dictionary_with_location_and_number_per_disease(
        &self,
        df: &DataFrame,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let mut dados_por_queixa: DiseaseLocationMap = HashMap::new();
        let mut seen = HashSet::new();
        let mut total_atendimentos: i64 = 0;
        let mut total_inferidos: i64 = 0;
        let mut atendimentos_base: HashSet<(String, String)> = HashSet::new();

        for i in 0..df.height() {
            if !row_matches_table_name(df, i, TABLE_ACOLHIMENTO) {
                continue;
            }

            let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", i) else {
                continue;
            };
            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", i) else {
                continue;
            };

            if atendimentos_base.insert((competencia.clone(), ifrotabelaid.clone())) {
                total_atendimentos += 1;
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

            total_inferidos += 1;

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

        final_dict.insert(
            KEY_DOENCA_NAO_INFERIDA.to_string(),
            json!({
                "quantidade": total_atendimentos.saturating_sub(total_inferidos)
            }),
        );

        Ok(json!(final_dict))
    }

    pub async fn create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood(
        &self,
        df: &DataFrame,
        _unidade_id: i32,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let mut bairro_dados: HashMap<String, (Option<f64>, Option<f64>, i64)> = HashMap::new();
        let mut seen: HashSet<(String, String)> = HashSet::new();

        for i in 0..df.height() {
            if !row_matches_table_name(df, i, TABLE_ACOLHIMENTO) {
                continue;
            }

            let Some(competencia) = get_non_empty_cell_string(df, "ifrocompetencia", i) else {
                continue;
            };
            let Some(ifrotabelaid) = get_non_empty_cell_string(df, "ifrotabelaid", i) else {
                continue;
            };

            if !seen.insert((competencia, ifrotabelaid)) {
                continue;
            }

            let endereco =
                get_non_empty_cell_string(df, "ifropacienteendereco", i).unwrap_or_default();
            let bairro = get_non_empty_cell_string(df, "ifropacientebairro", i).unwrap_or_default();
            let latitude =
                get_non_empty_cell_string(df, "ifropacientelatitude", i).unwrap_or_default();
            let longitude =
                get_non_empty_cell_string(df, "ifropacientelongitude", i).unwrap_or_default();

            let (bucket_name, lat, long) = if endereco.is_empty() {
                (BUCKET_SEM_ENDERECO.to_string(), None, None)
            } else if endereco == INVALID_ADDRESS_DO_IPE {
                (BUCKET_INCORRETO.to_string(), None, None)
            } else if bairro.is_empty() {
                (BUCKET_SEM_BAIRRO.to_string(), None, None)
            } else {
                match (latitude.parse::<f64>().ok(), longitude.parse::<f64>().ok()) {
                    (Some(lat), Some(long)) => (bairro, Some(lat), Some(long)),
                    _ => (BUCKET_SEM_COORDENADA.to_string(), None, None),
                }
            };

            let entry = bairro_dados.entry(bucket_name).or_insert((lat, long, 0));
            entry.2 += 1;
        }

        let mut organized_data = HashMap::new();
        let mut dados_extras = HashMap::new();

        for (bairro, (lat, long, quantidade)) in bairro_dados {
            let mut neighborhood_data = HashMap::new();
            if is_special_heat_map_bucket(&bairro) {
                neighborhood_data.insert("latitude".to_string(), Value::Null);
                neighborhood_data.insert("longitude".to_string(), Value::Null);
                neighborhood_data.insert("quantidade".to_string(), json!(quantidade));
                dados_extras.insert(bairro, json!(neighborhood_data));
                continue;
            }

            neighborhood_data.insert("latitude".to_string(), json!(lat));
            neighborhood_data.insert("longitude".to_string(), json!(long));
            neighborhood_data.insert("quantidade".to_string(), json!(quantidade));

            organized_data.insert(bairro, json!(neighborhood_data));
        }

        organized_data.insert("dados_extras".to_string(), json!(dados_extras));

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
                cids_por_atendimento
                    .entry(event_key)
                    .or_default()
                    .insert(cid);
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
        Ok(aggregate_by_classification(df, TABLE_CONSULTA_MEDICA, true))
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
            DataProcessingForGraphPlotting.create_dict_to_number_of_appointments_per_month(&df),
        )
        .unwrap();

        assert_eq!(result, json!({ "2026-3": 2 }));
    }

    #[test]
    fn number_of_visits_per_nurse_includes_dados_extras_with_real_and_missing_counts() {
        let df = df!(
            "ifrocompetencia" => ["2026-3", "2026-3", "2026-3", "2026-4", "2026-4", "2026-4"],
            "ifroprofissionalcbods" => [
                "ENFERMEIRO",
                "ENFERMEIRO",
                "TECNICO DE ENFERMAGEM",
                "ENFERMEIRO",
                "ENFERMEIRO",
                "ENFERMEIRO"
            ],
            "ifroprofissionalnome" => ["ANA", "RENAN", "TEC", "ANA", "ANA", "RENAN"],
            "ifrotabelanome" => [
                "Acolhimento",
                "Acolhimento",
                "Acolhimento",
                "Acolhimento",
                "Acolhimento",
                "Acolhimento"
            ],
            "ifrotabelaid" => [1i64, 2, 3, 4, 4, 5]
        )
        .unwrap();
        let df_non_nurse = df!(
            "ifroprofissionalnome" => ["RENAN"]
        )
        .unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_number_of_visits_per_nurse(&df, &df_non_nurse),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "ANA": {
                    "2026-3": 1,
                    "2026-4": 1,
                    "todos": 2
                },
                "dados_extras": {
                    "atendimentos_nao_contabilizados": {
                        "2026-3": 2,
                        "2026-4": 1,
                        "todos": 3
                    },
                    "quantidade_total_real": {
                        "2026-3": 3,
                        "2026-4": 2,
                        "todos": 5
                    }
                }
            })
        );
    }

    #[test]
    fn number_of_visits_per_doctor_includes_dados_extras_with_real_and_missing_counts() {
        let df = df!(
            "ifrocompetencia" => ["2026-3", "2026-3", "2026-3", "2026-4", "2026-4", "2026-4"],
            "ifroprofissionalcbods" => [
                "MEDICO CLINICO",
                "MEDICO CLINICO",
                "MEDICO CIRURGIAO GERAL",
                "MEDICO CLINICO",
                "MEDICO CLINICO",
                "MEDICO CLINICO"
            ],
            "ifroprofissionalnome" => [
                "ALICE",
                "GILLIARD",
                "CIRURGIAO",
                "ALICE",
                "ALICE",
                "GILLIARD"
            ],
            "ifrotabelanome" => [
                "ConsultaMedica",
                "ConsultaMedica",
                "ConsultaMedica",
                "ConsultaMedica",
                "ConsultaMedica",
                "ConsultaMedica"
            ],
            "ifrotabelaid" => [11i64, 12, 13, 14, 14, 15]
        )
        .unwrap();
        let df_non_doctors = df!(
            "ifroprofissionalnome" => ["GILLIARD"]
        )
        .unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_number_of_visits_per_doctor(&df, &df_non_doctors),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "ALICE": {
                    "2026-3": 1,
                    "2026-4": 1,
                    "todos": 2
                },
                "dados_extras": {
                    "atendimentos_nao_contabilizados": {
                        "2026-3": 1,
                        "2026-4": 1,
                        "todos": 2
                    },
                    "quantidade_total_real": {
                        "2026-3": 2,
                        "2026-4": 2,
                        "todos": 4
                    }
                }
            })
        );
    }

    #[test]
    fn average_time_per_doctor_uses_daily_intervals_and_returns_numeric_minutes() {
        let df = df!(
            "ifrocompetencia" => ["2026-1", "2026-1", "2026-1", "2026-2", "2026-2"],
            "ifrodataatendimento" => [
                "2026-01-01",
                "2026-01-01",
                "2026-01-01",
                "2026-02-01",
                "2026-02-01"
            ],
            "ifrohoraatendimento" => ["08:30:00", "08:55:00", "09:05:00", "16:00:00", "16:20:00"],
            "ifroprofissionalcbods" => [
                "MEDICO CLINICO",
                "MEDICO CLINICO",
                "MEDICO CLINICO",
                "MEDICO CLINICO",
                "MEDICO CLINICO"
            ],
            "ifroprofissionalid" => ["100", "100", "100", "100", "100"],
            "ifroprofissionalnome" => ["ALICE", "ALICE", "ALICE", "ALICE", "ALICE"],
            "ifrotabelanome" => [
                "ConsultaMedica",
                "ConsultaMedica",
                "ConsultaMedica",
                "ConsultaMedica",
                "ConsultaMedica"
            ]
        )
        .unwrap();
        let df_non_doctors = df!(
            "ifroprofissionalnome" => ["GILLIARD"]
        )
        .unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_average_time_in_minutes_per_doctor(&df, &df_non_doctors),
        )
        .unwrap();

        // Dia 2026-01-01: pares (08:30→08:55)=25, (08:55→09:05)=10 → diária = (25+10)/2 = 17.5
        // Dia 2026-02-01: par (16:00→16:20)=20 → diária = 20.0
        // "2026-1" = mean({17.5}) = 17.5
        // "2026-2" = mean({20.0}) = 20.0
        // "todos"  = mean({17.5, 20.0}) = 18.75
        assert_eq!(
            result,
            json!({
                "ALICE": {
                    "todos": 18.75,
                    "2026-1": 17.5,
                    "2026-2": 20.0
                }
            })
        );
    }

    #[test]
    fn average_time_per_doctor_resets_each_day_and_filters_excluded_or_invalid_rows() {
        let df = df!(
            "ifrocompetencia" => [
                "2026-3", "2026-3",
                "2026-3", "2026-3",
                "2026-3", "2026-3",
                "2026-3"
            ],
            "ifrodataatendimento" => [
                "2026-03-01", "2026-03-01",
                "2026-03-01", "2026-03-01",
                "2026-03-02", "2026-03-02",
                "2026-03-01"
            ],
            "ifrohoraatendimento" => [
                "10:00:00", "10:30:00",
                "08:00:00", "08:20:00",
                "00:00:00", "00:30:00",
                "invalid"
            ],
            "ifroprofissionalcbods" => [
                "MEDICO CIRURGIAO GERAL", "MEDICO CIRURGIAO GERAL",
                "MEDICO CLINICO", "MEDICO CLINICO",
                "MEDICO CLINICO", "MEDICO CLINICO",
                "MEDICO CLINICO"
            ],
            "ifroprofissionalid" => [
                "200", "200",
                "300", "300",
                "400", "400",
                "500"
            ],
            "ifroprofissionalnome" => [
                "BOB", "BOB",
                "GILLIARD", "GILLIARD",
                "DAVE", "DAVE",
                "CAROL"
            ],
            "ifrotabelanome" => [
                "ConsultaMedica", "ConsultaMedica",
                "ConsultaMedica", "ConsultaMedica",
                "ConsultaMedica", "ConsultaMedica",
                "ConsultaMedica"
            ]
        )
        .unwrap();
        let df_non_doctors = df!(
            "ifroprofissionalnome" => ["GILLIARD"]
        )
        .unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_average_time_in_minutes_per_doctor(&df, &df_non_doctors),
        )
        .unwrap();

        // BOB dia 2026-03-01: par (10:00→10:30)=30 → diária = 30.0 → "2026-3" e "todos" = 30.0
        // DAVE dia 2026-03-02: par (00:00→00:30)=30 → diária = 30.0 → "2026-3" e "todos" = 30.0
        // GILLIARD em non_doctors / CAROL hora inválida → ambos descartados.
        assert_eq!(
            result,
            json!({
                "BOB": {
                    "todos": 30.0,
                    "2026-3": 30.0
                },
                "DAVE": {
                    "todos": 30.0,
                    "2026-3": 30.0
                }
            })
        );
    }

    #[test]
    fn average_time_per_doctor_aggregates_each_day_with_equal_weight() {
        // Dia A: 3 atendimentos, pares 10 e 20 → diária = 15
        // Dia B: 2 atendimentos, par 60 → diária = 60
        // Mensal "2026-4" = mean({15, 60}) = 37.5  (NÃO 30, que seria pool de pares)
        let df = df!(
            "ifrocompetencia" => ["2026-4", "2026-4", "2026-4", "2026-4", "2026-4"],
            "ifrodataatendimento" => [
                "2026-04-10", "2026-04-10", "2026-04-10",
                "2026-04-11", "2026-04-11"
            ],
            "ifrohoraatendimento" => [
                "08:00:00", "08:10:00", "08:30:00",
                "09:00:00", "10:00:00"
            ],
            "ifroprofissionalcbods" => [
                "MEDICO CLINICO", "MEDICO CLINICO", "MEDICO CLINICO",
                "MEDICO CLINICO", "MEDICO CLINICO"
            ],
            "ifroprofissionalid" => ["999", "999", "999", "999", "999"],
            "ifroprofissionalnome" => ["Dr. JOAO", "Dr. JOAO", "Dr. JOAO", "Dr. JOAO", "Dr. JOAO"],
            "ifrotabelanome" => [
                "ConsultaMedica", "ConsultaMedica", "ConsultaMedica",
                "ConsultaMedica", "ConsultaMedica"
            ]
        )
        .unwrap();
        let df_non_doctors = df!("ifroprofissionalnome" => Vec::<String>::new()).unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_average_time_in_minutes_per_doctor(&df, &df_non_doctors),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "Dr. JOAO": {
                    "todos": 37.5,
                    "2026-4": 37.5
                }
            })
        );
    }

    #[test]
    fn average_time_per_doctor_drops_outlier_pairs_above_60_minutes_keeps_60_exactly() {
        // Dia: 08:00, 09:00 (par 60min, mantém), 10:30 (par 90min, descarta), 11:00 (par 30min, mantém).
        // Pares válidos: {60, 30} → diária = 45.0.
        let df = df!(
            "ifrocompetencia" => ["2026-5", "2026-5", "2026-5", "2026-5"],
            "ifrodataatendimento" => [
                "2026-05-01", "2026-05-01", "2026-05-01", "2026-05-01"
            ],
            "ifrohoraatendimento" => ["08:00:00", "09:00:00", "10:30:00", "11:00:00"],
            "ifroprofissionalcbods" => [
                "MEDICO CLINICO", "MEDICO CLINICO", "MEDICO CLINICO", "MEDICO CLINICO"
            ],
            "ifroprofissionalid" => ["111", "111", "111", "111"],
            "ifroprofissionalnome" => ["EDU", "EDU", "EDU", "EDU"],
            "ifrotabelanome" => [
                "ConsultaMedica", "ConsultaMedica", "ConsultaMedica", "ConsultaMedica"
            ]
        )
        .unwrap();
        let df_non_doctors = df!("ifroprofissionalnome" => Vec::<String>::new()).unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_average_time_in_minutes_per_doctor(&df, &df_non_doctors),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "EDU": {
                    "todos": 45.0,
                    "2026-5": 45.0
                }
            })
        );
    }

    #[test]
    fn average_time_per_doctor_deduplicates_same_id_and_timestamp() {
        // 3 linhas: 08:00, 08:00 (duplicata exata), 08:20.
        // Após dedup → 2 records distintos → 1 par 20min → diária 20.0.
        let df = df!(
            "ifrocompetencia" => ["2026-6", "2026-6", "2026-6"],
            "ifrodataatendimento" => ["2026-06-01", "2026-06-01", "2026-06-01"],
            "ifrohoraatendimento" => ["08:00:00", "08:00:00", "08:20:00"],
            "ifroprofissionalcbods" => ["MEDICO CLINICO", "MEDICO CLINICO", "MEDICO CLINICO"],
            "ifroprofissionalid" => ["222", "222", "222"],
            "ifroprofissionalnome" => ["FRAN", "FRAN", "FRAN"],
            "ifrotabelanome" => ["ConsultaMedica", "ConsultaMedica", "ConsultaMedica"]
        )
        .unwrap();
        let df_non_doctors = df!("ifroprofissionalnome" => Vec::<String>::new()).unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_average_time_in_minutes_per_doctor(&df, &df_non_doctors),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "FRAN": {
                    "todos": 20.0,
                    "2026-6": 20.0
                }
            })
        );
    }

    #[test]
    fn average_time_per_doctor_picks_most_frequent_name_per_id_no_collision() {
        // Mesmo ID="777" com nomes ["Dr. JOAO","Dr. Joao","Dr. JOAO","Dr. JOAO"].
        // Canônico = "Dr. JOAO" (3 ocorrências). 4 atendimentos, todos no mesmo dia.
        // Pares: 10, 10, 10 → diária 10.0.
        let df = df!(
            "ifrocompetencia" => ["2026-7", "2026-7", "2026-7", "2026-7"],
            "ifrodataatendimento" => [
                "2026-07-01", "2026-07-01", "2026-07-01", "2026-07-01"
            ],
            "ifrohoraatendimento" => ["08:00:00", "08:10:00", "08:20:00", "08:30:00"],
            "ifroprofissionalcbods" => [
                "MEDICO CLINICO", "MEDICO CLINICO", "MEDICO CLINICO", "MEDICO CLINICO"
            ],
            "ifroprofissionalid" => ["777", "777", "777", "777"],
            "ifroprofissionalnome" => ["Dr. JOAO", "Dr. Joao", "Dr. JOAO", "Dr. JOAO"],
            "ifrotabelanome" => [
                "ConsultaMedica", "ConsultaMedica", "ConsultaMedica", "ConsultaMedica"
            ]
        )
        .unwrap();
        let df_non_doctors = df!("ifroprofissionalnome" => Vec::<String>::new()).unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_average_time_in_minutes_per_doctor(&df, &df_non_doctors),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "Dr. JOAO": {
                    "todos": 10.0,
                    "2026-7": 10.0
                }
            })
        );
    }

    #[test]
    fn average_time_per_doctor_distinguishes_distinct_ids_with_same_canonical_name() {
        // Dois IDs distintos ("100" e "200") cujos nomes resolvem ambos para "Dr. JOAO".
        // Cada um deve aparecer com sua média separada e sufixo (#id).
        let df = df!(
            "ifrocompetencia" => ["2026-8", "2026-8", "2026-8", "2026-8"],
            "ifrodataatendimento" => [
                "2026-08-01", "2026-08-01",
                "2026-08-02", "2026-08-02"
            ],
            "ifrohoraatendimento" => ["08:00:00", "08:10:00", "09:00:00", "09:30:00"],
            "ifroprofissionalcbods" => [
                "MEDICO CLINICO", "MEDICO CLINICO",
                "MEDICO CLINICO", "MEDICO CLINICO"
            ],
            "ifroprofissionalid" => ["100", "100", "200", "200"],
            "ifroprofissionalnome" => ["Dr. JOAO", "Dr. JOAO", "Dr. JOAO", "Dr. JOAO"],
            "ifrotabelanome" => [
                "ConsultaMedica", "ConsultaMedica",
                "ConsultaMedica", "ConsultaMedica"
            ]
        )
        .unwrap();
        let df_non_doctors = df!("ifroprofissionalnome" => Vec::<String>::new()).unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_average_time_in_minutes_per_doctor(&df, &df_non_doctors),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "Dr. JOAO (#100)": { "todos": 10.0, "2026-8": 10.0 },
                "Dr. JOAO (#200)": { "todos": 30.0, "2026-8": 30.0 }
            })
        );
    }

    #[test]
    fn average_time_per_doctor_drops_rows_with_empty_doctor_id() {
        // Linha com ID vazio é descartada e não influencia o resultado.
        // KIM tem 2 atendimentos válidos com ID; uma 3ª linha sem ID é ignorada.
        let df = df!(
            "ifrocompetencia" => ["2026-9", "2026-9", "2026-9"],
            "ifrodataatendimento" => ["2026-09-01", "2026-09-01", "2026-09-01"],
            "ifrohoraatendimento" => ["08:00:00", "08:30:00", "09:00:00"],
            "ifroprofissionalcbods" => ["MEDICO CLINICO", "MEDICO CLINICO", "MEDICO CLINICO"],
            "ifroprofissionalid" => ["555", "555", ""],
            "ifroprofissionalnome" => ["KIM", "KIM", "KIM"],
            "ifrotabelanome" => ["ConsultaMedica", "ConsultaMedica", "ConsultaMedica"]
        )
        .unwrap();
        let df_non_doctors = df!("ifroprofissionalnome" => Vec::<String>::new()).unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_average_time_in_minutes_per_doctor(&df, &df_non_doctors),
        )
        .unwrap();

        // Só os 2 records com ID válido entram → 1 par 30min → diária 30.0.
        // (Se a linha sem ID entrasse, haveria um 2º par e a diária ficaria diferente.)
        assert_eq!(
            result,
            json!({
                "KIM": {
                    "todos": 30.0,
                    "2026-9": 30.0
                }
            })
        );
    }

    #[test]
    fn average_time_per_doctor_omits_doctor_with_no_valid_pair() {
        // Médico LEO tem só 1 atendimento (nenhum par possível) → não aparece no JSON.
        // ZOE tem 2 atendimentos do mesmo dia → entra com diária 15.0.
        let df = df!(
            "ifrocompetencia" => ["2026-10", "2026-10", "2026-10"],
            "ifrodataatendimento" => ["2026-10-01", "2026-10-01", "2026-10-01"],
            "ifrohoraatendimento" => ["08:00:00", "08:00:00", "08:15:00"],
            "ifroprofissionalcbods" => ["MEDICO CLINICO", "MEDICO CLINICO", "MEDICO CLINICO"],
            "ifroprofissionalid" => ["888", "999", "999"],
            "ifroprofissionalnome" => ["LEO", "ZOE", "ZOE"],
            "ifrotabelanome" => ["ConsultaMedica", "ConsultaMedica", "ConsultaMedica"]
        )
        .unwrap();
        let df_non_doctors = df!("ifroprofissionalnome" => Vec::<String>::new()).unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_average_time_in_minutes_per_doctor(&df, &df_non_doctors),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "ZOE": {
                    "todos": 15.0,
                    "2026-10": 15.0
                }
            })
        );
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
            DataProcessingForGraphPlotting.create_dict_to_number_of_appointments_per_cid(&df),
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

    #[test]
    fn heat_map_with_disease_indication_adds_doenca_nao_inferida_for_missing_rows() {
        let df = df!(
            "ifrocompetencia" => ["2026-1", "2026-1", "2026-1", "2026-2", "2026-2"],
            "ifrotabelaid" => [1i64, 2, 3, 4, 5],
            "ifrotabelanome" => ["Acolhimento", "Acolhimento", "Acolhimento", "Acolhimento", "Acolhimento"],
            "ifropacienteendereco" => ["RUA 1", "RUA 2", "DO IPE", "RUA 4", "RUA 5"],
            "ifropacientebairro" => ["SETOR 01", "SETOR 02", "SETOR 03", "SETOR 04", "SETOR 05"],
            "ifropacientequeixaprincipal" => ["gripe", "", "dengue", "dengue", "febre"],
            "ifropacientelatitude" => ["-9.1", "-9.2", "-9.3", "-9.4", ""],
            "ifropacientelongitude" => ["-63.1", "-63.2", "-63.3", "-63.4", "-63.5"]
        )
        .unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dictionary_with_location_and_number_per_disease(&df),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "gripe": {
                    "2026-1": {
                        "SETOR 01": {
                            "latitude": -9.1,
                            "longitude": -63.1,
                            "quantidade": 1
                        }
                    },
                    "todos": {
                        "SETOR 01": {
                            "latitude": -9.1,
                            "longitude": -63.1,
                            "quantidade": 1
                        }
                    }
                },
                "dengue": {
                    "2026-2": {
                        "SETOR 04": {
                            "latitude": -9.4,
                            "longitude": -63.4,
                            "quantidade": 1
                        }
                    },
                    "todos": {
                        "SETOR 04": {
                            "latitude": -9.4,
                            "longitude": -63.4,
                            "quantidade": 1
                        }
                    }
                },
                "doenca_nao_inferida": {
                    "quantidade": 3
                }
            })
        );
    }

    #[test]
    fn heat_map_with_disease_indication_sets_doenca_nao_inferida_to_zero_when_complete() {
        let df = df!(
            "ifrocompetencia" => ["2026-1", "2026-2"],
            "ifrotabelaid" => [1i64, 2],
            "ifrotabelanome" => ["Acolhimento", "Acolhimento"],
            "ifropacienteendereco" => ["RUA 1", "RUA 2"],
            "ifropacientebairro" => ["SETOR 01", "SETOR 02"],
            "ifropacientequeixaprincipal" => ["gripe", "dengue"],
            "ifropacientelatitude" => ["-9.1", "-9.2"],
            "ifropacientelongitude" => ["-63.1", "-63.2"]
        )
        .unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dictionary_with_location_and_number_per_disease(&df),
        )
        .unwrap();

        assert_eq!(result["doenca_nao_inferida"], json!({ "quantidade": 0 }));
        assert_eq!(
            result["gripe"]["todos"]["SETOR 01"],
            json!({
                "latitude": -9.1,
                "longitude": -63.1,
                "quantidade": 1
            })
        );
        assert_eq!(
            result["dengue"]["todos"]["SETOR 02"],
            json!({
                "latitude": -9.2,
                "longitude": -63.2,
                "quantidade": 1
            })
        );
    }

    #[test]
    fn distribution_of_services_by_hour_group_uses_acolhimento_and_unique_ids() {
        let df = df!(
            "ifrocompetencia" => ["2026-3", "2026-3", "2026-3", "2026-3", "2026-3"],
            "ifrohoraatendimento" => ["00:15:00", "00:45:00", "09:30:00", "18:10:00", "08:00:00"],
            "ifroprofissionalcbods" => ["ENFERMEIRO", "MEDICO CLINICO", "ENFERMEIRO", "ENFERMEIRO", "MEDICO CLINICO"],
            "ifrotabelaid" => [1i64, 1, 2, 3, 4],
            "ifrotabelanome" => ["Acolhimento", "Acolhimento", "Acolhimento", "Acolhimento", "ConsultaMedica"]
        )
        .unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_distribution_of_services_by_hour_group(&df),
        )
        .unwrap();

        let object = result.as_object().unwrap();
        let zero_only_groups = [
            "02h-04h", "04h-06h", "06h-08h", "10h-12h", "12h-14h", "14h-16h", "16h-18h", "20h-22h",
            "22h-24h",
        ];

        assert_eq!(object.len(), 12);
        assert_eq!(object["00h-02h"], json!({ "2026-3": 1, "todos": 1 }));
        assert_eq!(object["08h-10h"], json!({ "2026-3": 1, "todos": 1 }));
        assert_eq!(object["18h-20h"], json!({ "2026-3": 1, "todos": 1 }));

        for group in zero_only_groups {
            assert_eq!(object[group], json!({ "todos": 0 }));
        }
    }

    #[test]
    fn heat_map_by_neighborhood_uses_acolhimento_and_extra_buckets() {
        let df = df!(
            "ifrocompetencia" => ["2026-1", "2026-1", "2026-1", "2026-1", "2026-1", "2026-1", "2026-1"],
            "ifrotabelaid" => [1i64, 1, 2, 3, 4, 5, 6],
            "ifrotabelanome" => [
                "Acolhimento",
                "Acolhimento",
                "Acolhimento",
                "Acolhimento",
                "Acolhimento",
                "Acolhimento",
                "ConsultaMedica"
            ],
            "ifropacienteendereco" => [
                "RUA 1",
                "RUA 1",
                "",
                "DO IPE",
                "RUA 4",
                "RUA 5",
                "RUA 6"
            ],
            "ifropacientebairro" => [
                "SETOR 01",
                "SETOR 01",
                "SETOR 02",
                "SETOR 03",
                "",
                "SETOR 05",
                "SETOR 06"
            ],
            "ifropacientelatitude" => [
                "-9.1",
                "-9.1",
                "-9.2",
                "-9.3",
                "-9.4",
                "",
                "-9.6"
            ],
            "ifropacientelongitude" => [
                "-63.1",
                "-63.1",
                "-63.2",
                "-63.3",
                "-63.4",
                "",
                "-63.6"
            ]
        )
        .unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood(
                    &df, 2,
                ),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "SETOR 01": {
                    "latitude": -9.1,
                    "longitude": -63.1,
                    "quantidade": 1
                },
                "dados_extras": {
                    "SEM ENDERECO": {
                        "latitude": null,
                        "longitude": null,
                        "quantidade": 1
                    },
                    "INCORRETO": {
                        "latitude": null,
                        "longitude": null,
                        "quantidade": 1
                    },
                    "SEM BAIRRO": {
                        "latitude": null,
                        "longitude": null,
                        "quantidade": 1
                    },
                    "SEM COORDENADA": {
                        "latitude": null,
                        "longitude": null,
                        "quantidade": 1
                    }
                }
            })
        );
    }

    #[test]
    fn heat_map_by_neighborhood_keeps_extra_buckets_only_inside_dados_extras() {
        let df = df!(
            "ifrocompetencia" => ["2026-1", "2026-1", "2026-1", "2026-1"],
            "ifrotabelaid" => [1i64, 2, 3, 4],
            "ifrotabelanome" => ["Acolhimento", "Acolhimento", "Acolhimento", "Acolhimento"],
            "ifropacienteendereco" => ["", "DO IPE", "RUA 3", "RUA 4"],
            "ifropacientebairro" => ["SETOR 01", "SETOR 02", "", "SETOR 04"],
            "ifropacientelatitude" => ["-9.1", "-9.2", "-9.3", ""],
            "ifropacientelongitude" => ["-63.1", "-63.2", "-63.3", ""]
        )
        .unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood(
                    &df, 1,
                ),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "dados_extras": {
                    "SEM ENDERECO": {
                        "latitude": null,
                        "longitude": null,
                        "quantidade": 1
                    },
                    "INCORRETO": {
                        "latitude": null,
                        "longitude": null,
                        "quantidade": 1
                    },
                    "SEM BAIRRO": {
                        "latitude": null,
                        "longitude": null,
                        "quantidade": 1
                    },
                    "SEM COORDENADA": {
                        "latitude": null,
                        "longitude": null,
                        "quantidade": 1
                    }
                }
            })
        );
    }

    #[test]
    fn heat_map_by_neighborhood_always_includes_empty_dados_extras() {
        let df = df!(
            "ifrocompetencia" => ["2026-1", "2026-1"],
            "ifrotabelaid" => [1i64, 2],
            "ifrotabelanome" => ["Acolhimento", "Acolhimento"],
            "ifropacienteendereco" => ["RUA 1", "RUA 2"],
            "ifropacientebairro" => ["SETOR 01", "SETOR 02"],
            "ifropacientelatitude" => ["-9.1", "-9.2"],
            "ifropacientelongitude" => ["-63.1", "-63.2"]
        )
        .unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood(
                    &df, 2,
                ),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "SETOR 01": {
                    "latitude": -9.1,
                    "longitude": -63.1,
                    "quantidade": 1
                },
                "SETOR 02": {
                    "latitude": -9.2,
                    "longitude": -63.2,
                    "quantidade": 1
                },
                "dados_extras": {}
            })
        );
    }

    #[test]
    fn heat_map_by_neighborhood_counts_same_ifrotabelaid_once_per_competencia() {
        let df = df!(
            "ifrocompetencia" => ["2026-1", "2026-1", "2026-2", "2026-2"],
            "ifrotabelaid" => [1i64, 1, 1, 2],
            "ifrotabelanome" => ["Acolhimento", "Acolhimento", "Acolhimento", "Acolhimento"],
            "ifropacienteendereco" => ["RUA 1", "RUA 1", "RUA 2", "DO IPE"],
            "ifropacientebairro" => ["SETOR 01", "SETOR 01", "SETOR 02", "SETOR 03"],
            "ifropacientelatitude" => ["-9.1", "-9.1", "-9.2", "-9.3"],
            "ifropacientelongitude" => ["-63.1", "-63.1", "-63.2", "-63.3"]
        )
        .unwrap();

        let result = block_on(
            DataProcessingForGraphPlotting
                .create_dict_to_heat_map_with_the_number_of_medical_appointments_by_neighborhood(
                    &df, 2,
                ),
        )
        .unwrap();

        assert_eq!(
            result,
            json!({
                "SETOR 01": {
                    "latitude": -9.1,
                    "longitude": -63.1,
                    "quantidade": 1
                },
                "SETOR 02": {
                    "latitude": -9.2,
                    "longitude": -63.2,
                    "quantidade": 1
                },
                "dados_extras": {
                    "INCORRETO": {
                        "latitude": null,
                        "longitude": null,
                        "quantidade": 1
                    }
                }
            })
        );
    }
}
