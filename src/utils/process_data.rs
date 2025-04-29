use tempfile::NamedTempFile;
use std::io::Write;
use polars::prelude::*;
use polars::lazy::dsl::{col, lit, when};
use polars::datatypes::DataType;
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use linfa::prelude::*;
use linfa_clustering::KMeans;
use ndarray::Array2;
use rust_stemmers::{Algorithm, Stemmer};


/// Retorna os nomes das colunas a serem removidas
pub fn columns_names() -> Vec<&'static str> {
    vec![
        "IfroId",
        "IfroTabelaId",
        "IfroOrigem",
        "IfroMunicipioId",
        "IfroAlocacaoId",
        "IfroMunicipioIBGE",
        "IfroUnidadeNome",
        "IfroUnidadeCNES",
        "IfroUnidadeCNPJ",
        "IfroProfissionalCBOCd",
        "IfroProcedimentoId",
        "IfroProfissionalCNS",
        "IfroProcedimentoSUSCd",
        "IfroPacienteId",
        "IfroPacienteCNS",
        "IfroPacienteCNSTipo",
        "IfroPacienteCNSValido",
        "IfroPacienteSexoCd",
        "IfroPacienteRacaCorCd",
        "IfroPacienteEtniaCd",
        "IfroPacienteNacionalidadeCd",
        "IfroPacienteEnderecoComp",
        "IfroPacienteTelefone",
        "IfroPacienteEmail",
        "IfroCidId",
        "IfroCidCd",
        "IfroCidDs",
    ]
}

/// Remove as colunas especificadas do DataFrame
pub fn remove_unnecessary_columns(df: DataFrame, columns_to_remove: &[&str]) -> PolarsResult<DataFrame> {
    // Converte &[&str] para Vec<String>
    let columns: Vec<String> = columns_to_remove.iter().map(|&s| s.to_string()).collect();
    Ok(df.drop_many(&columns))
}



/// Verifica se as colunas necessárias existem no DataFrame
fn verify_required_columns(df: &DataFrame, columns: &[&str]) -> PolarsResult<()> {
    for col in columns {
        if !df.schema().iter().any(|(name, _)| name == col) {
            return Err(PolarsError::ComputeError(
                format!("Coluna '{}' não encontrada no DataFrame", col).into(),
            ));
        }
    }
    Ok(())
}

/// Cria uma coluna de competência (ano-mês) no DataFrame
fn add_competencia_column(df: &mut DataFrame) -> PolarsResult<()> {
    let ano = df.column("IfroCompetenciaAno")?.cast(&DataType::String)?;
    let mes = df.column("IfroCompetenciaMes")?.cast(&DataType::String)?;

    let mut competencia_values = Vec::with_capacity(df.height());
    for i in 0..df.height() {
        let ano_val = match ano.get(i) {
            Ok(AnyValue::String(s)) => s,
            _ => "?",
        };

        let mes_val = match mes.get(i) {
            Ok(AnyValue::String(s)) => s,
            _ => "?",
        };

        competencia_values.push(format!("{}-{}", ano_val, mes_val));
    }

    let competencia_series = Series::new("IfroCompetencia".into(), competencia_values);
    df.with_column(competencia_series)?;
    Ok(())
}

/// Separa data e hora da coluna IfroDataAtendimento
fn split_date_and_time(lf: LazyFrame) -> LazyFrame {
    lf.with_columns([
        when(col("IfroDataAtendimento").str().contains(lit(" "), true))
            .then(col("IfroDataAtendimento").str().split(lit(" ")).list().first())
            .otherwise(col("IfroDataAtendimento"))
            .alias("IfroDataAtendimento_temp"),

        when(col("IfroDataAtendimento").str().contains(lit(" "), true))
            .then(col("IfroDataAtendimento").str().split(lit(" ")).list().get(lit(1), false))
            .otherwise(lit(NULL))
            .alias("IfroHoraAtendimento"),
    ])
    .with_column(
        col("IfroDataAtendimento_temp").alias("IfroDataAtendimento")
    )
    .drop(["IfroDataAtendimento_temp"])
}

/// Adiciona coluna com o dia da semana
fn add_day_of_week(lf: LazyFrame) -> LazyFrame {
    lf.with_column(
        col("IfroDataAtendimento")
            .str()
            .strptime(DataType::Date, StrptimeOptions {
                format: Some("%Y-%m-%d".into()),
                strict: true,
                exact: true,
                ..Default::default()
            }, lit("raise"))
            .dt()
            .strftime("%A")
            .alias("IfroDiaSemana")
    )
}

/// Adiciona o dia da semana e separa a data e hora no DataFrame
pub fn add_week_day_and_split_date_time_polars(df: DataFrame) -> PolarsResult<DataFrame> {
    // Verifica se as colunas necessárias existem
    let required_columns = ["IfroCompetenciaAno", "IfroCompetenciaMes", "IfroDataAtendimento"];
    verify_required_columns(&df, &required_columns)?;

    // Cria uma nova coluna 'IfroCompetencia'
    let mut df_com_competencia = df.clone();
    add_competencia_column(&mut df_com_competencia)?;

    // Aplica as transformações restantes
    let lf = df_com_competencia.lazy()
        // Ordenar por 'IfroCompetencia'
        .sort(["IfroCompetencia"], Default::default());

    // Separar data e hora
    let lf = split_date_and_time(lf);

    // Adicionar dia da semana
    let lf = add_day_of_week(lf);

    lf.collect()
}


/// Aplica strip() e lowercase()
pub fn normalize_text_to_lower_case_columns_lazy(lf: LazyFrame, column_names: &[&str]) -> LazyFrame {
    let mut result = lf;

    for &col_name in column_names {
        result = result.with_column(
            col(col_name)
                .str()
                .strip_chars_start(lit(NULL))
                .str()
                .strip_chars_end(lit(NULL))
                .str()
                .to_lowercase()
                .alias(col_name)
        );
    }

    result
}

/// Aplica strip() e uppercase()
pub fn normalize_text_to_upper_case_columns_lazy(lf: LazyFrame, column_names: &[&str]) -> LazyFrame {
    let mut result = lf;

    for &col_name in column_names {
        result = result.with_column(
            col(col_name)
                .str()
                .strip_chars_start(lit(NULL))
                .str()
                .strip_chars_end(lit(NULL))
                .str()
                .to_uppercase()
                .alias(col_name)
        );
    }

    result
}


// Lista de doenças mais comuns
pub fn list_of_most_common_diseases() -> Vec<&'static str> {
    vec![
        "dengue",
        "malaria",
        "covid-19",
        "diarreia",
        "virose",
        "gripe",
        "sarampo",
        "influenza",
        "caxumba",
        "sars-cov-2",
        "meningite",
        "depressão",
        "tuberculose",
        "sinusite",
        "rinite",
        "otite",
        "faringite",
        "laringite",
        "hemorroida",
        "trombose",
        "diabetes",
        "cefaleia",
        "pneumonia",
        "hipertensão",
        "leishmaniose",
        "raiva",
        "zika",
        "chikungunya",
        "leishmaniose ",
        "febre amarela",
        "doença de chagas",
        "esquistossomose",
        "filariose linfática",
        "leptospirose",
        "febre tifoide",
        "hepatite a",
        "hepatite e",
        "paracoccidioidomicose",
        "hantavirose",
        "cisticercose",
        "oncocercose",
        "micoses sistêmicas",
        "febre do nilo ocidental",
        "riquetsioses",
        "não especificado"
    ]
}

/// Verifica se o texto é numérico
pub fn is_numeric_text(text: &str) -> bool {
    let re = Regex::new(r"^\d+(\.\d+)?$").unwrap();
    re.is_match(text.trim())
}

/// Verifica se o texto é "não especificado" ou equivalente
pub fn is_not_specified(text: &str) -> bool {
    text.trim().to_lowercase() == "não especificado" ||
    is_numeric_text(text) ||
    ["encerramento de chamado", "atendimento chamado"].contains(&text.trim().to_lowercase().as_str())
}

/// Aplica regras de decisão para determinar a queixa principal
fn apply_decision_rules(text: &str, original_disease: &str) -> String {
    let text_lower = text.to_lowercase();

    // Lista de mapeamentos de palavras-chave para doenças
    let keyword_map = [
        (("febre", "amarela"), "febre amarela"),
        (("covid", ""), "covid-19"),
        (("sars-cov-2", ""), "covid-19"),
        (("malaria", ""), "malaria"),
        (("dengue", ""), "dengue"),
        (("pneumonia", ""), "pneumonia"),
        (("leptospirose", ""), "leptospirose"),
        (("gripe", ""), "gripe"),
        (("diabetes", ""), "diabetes"),
        (("diarreia", ""), "diarreia"),
        (("virose", ""), "virose"),
        (("sarampo", ""), "sarampo"),
        (("influenza", ""), "influenza"),
        (("caxumba", ""), "caxumba"),
        (("meningite", ""), "meningite"),
        (("depressão", ""), "depressão"),
        (("tuberculose", ""), "tuberculose"),
        (("sinusite", ""), "sinusite"),
        (("rinite", ""), "rinite"),
        (("otite", ""), "otite"),
        (("faringite", ""), "faringite"),
        (("laringite", ""), "laringite"),
        (("hemorroida", ""), "hemorroida"),
        (("trombose", ""), "trombose"),
        (("cefaleia", ""), "cefaleia"),
        (("hipertensão", ""), "hipertensão"),
        (("leishmaniose", ""), "leishmaniose"),
        (("raiva", ""), "raiva"),
        (("zika", ""), "zika"),
        (("chikungunya", ""), "chikungunya"),
        (("doença de chagas", ""), "doença de chagas"),
        (("esquistossomose", ""), "esquistossomose"),
        (("filariose", "linfática"), "filariose linfática"),
        (("febre", "tifoide"), "febre tifoide"),
        (("hepatite a", ""), "hepatite a"),
        (("hepatite e", ""), "hepatite e"),
        (("paracoccidioidomicose", ""), "paracoccidioidomicose"),
        (("hantavirose", ""), "hantavirose"),
        (("cisticercose", ""), "cisticercose"),
        (("oncocercose", ""), "oncocercose"),
        (("micoses sistêmicas", ""), "micoses sistêmicas"),
        (("riquetsioses", ""), "riquetsioses"),
    ];

    // Verifica cada par de palavras-chave
    for ((keyword1, keyword2), disease) in keyword_map.iter() {
        if keyword2.is_empty() {
            // Se não há segunda palavra-chave, verifica apenas a primeira
            if text_lower.contains(keyword1) {
                return disease.to_string();
            }
        } else {
            // Se há duas palavras-chave, verifica ambas
            if text_lower.contains(keyword1) && text_lower.contains(keyword2) {
                return disease.to_string();
            }
        }
    }

    // Se nenhum padrão corresponder, retorna a doença original
    original_disease.to_string()
}



/// Retorna um conjunto de stopwords em português
fn get_portuguese_stopwords() -> HashSet<String> {
    let stopwords_list = vec![
        "a", "ao", "aos", "aquela", "aquelas", "aquele", "aqueles", "aquilo", "as", "até", "com", "como",
        "da", "das", "de", "dela", "delas", "dele", "deles", "depois", "do", "dos", "e", "ela", "elas", "ele",
        "eles", "em", "entre", "era", "eram", "éramos", "essa", "essas", "esse", "esses", "esta", "estas",
        "este", "estes", "eu", "foi", "fomos", "for", "foram", "forem", "formos", "fosse", "fossem", "fôssemos",
        "há", "isso", "isto", "já", "lhe", "lhes", "mais", "mas", "me", "mesmo", "meu", "meus", "minha", "minhas",
        "muito", "na", "nas", "não", "no", "nos", "nós", "nossa", "nossas", "nosso", "nossos", "num", "numa", "o",
        "os", "ou", "para", "pela", "pelas", "pelo", "pelos", "por", "quando", "que", "quem", "são", "se", "seja",
        "sejam", "sejamos", "sem", "seu", "seus", "sua", "suas", "também", "te", "tem", "tém", "temos", "tenha",
        "tenham", "tenhamos", "tenho", "teria", "teriam", "teríamos", "teu", "teus", "tua", "tuas", "um", "uma", "você",
        "vocês", "vos"
    ];

    stopwords_list.into_iter().map(String::from).collect()
}


/// Função para vetorizar textos (TF-IDF com normalização)
fn vectorize_texts(texts: &[String], stopwords: &HashSet<String>, stemmer: &Stemmer) -> (Array2<f64>, Vec<String>) {
    // Extrair termos de todos os textos
    let mut all_terms = HashSet::new();
    let mut doc_terms = Vec::with_capacity(texts.len());

    for text in texts {
        let terms = tokenize_and_stem(text, stopwords, stemmer);
        all_terms.extend(terms.clone());
        doc_terms.push(terms);
    }

    // Converte para vetor ordenado
    let terms_vec: Vec<String> = all_terms.into_iter().collect();

    // Cria matriz documento-termo
    let n_docs = texts.len();
    let n_terms = terms_vec.len();

    // Criar uma matriz preenchida com zeros
    let mut matrix = Array2::<f64>::zeros((n_docs, n_terms));

    // Preenche a matriz com contagens de termos (TF)
    for (doc_idx, terms) in doc_terms.iter().enumerate() {
        for term in terms {
            if let Some(term_idx) = terms_vec.iter().position(|t| t == term) {
                matrix[[doc_idx, term_idx]] += 1.0;
            }
        }
    }

    // Calcula IDF e aplicar TF-IDF
    let n_docs_f64 = n_docs as f64;

    for term_idx in 0..n_terms {
        let term_docs = matrix.column(term_idx).iter().filter(|&&v| v > 0.0).count();

        if term_docs > 0 {
            let idf = (1.0 + n_docs_f64 / term_docs as f64).ln();

            for doc_idx in 0..n_docs {
                if matrix[[doc_idx, term_idx]] > 0.0 {
                    matrix[[doc_idx, term_idx]] *= idf;
                }
            }
        }
    }

    // Normalização  (Euclidiana) por linha
    for doc_idx in 0..n_docs {
        let row = matrix.row(doc_idx);
        let norm: f64 = row.iter().map(|&v| v * v).sum::<f64>().sqrt();

        if norm > 0.0 {
            for term_idx in 0..n_terms {
                matrix[[doc_idx, term_idx]] /= norm;
            }
        }
    }

    (matrix, terms_vec)
}

/// Tokeniza e aplica stemming em um texto
fn tokenize_and_stem(text: &str, stopwords: &HashSet<String>, stemmer: &Stemmer) -> HashSet<String> {
    let text = text.to_lowercase();

    // Tokenização simples (separar por espaços e remover pontuação)
    let tokens: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    // Remove stopwords e aplica stemming
    tokens
        .into_iter()
        .filter(|token| !stopwords.contains(token))
        .map(|token| stemmer.stem(&token).to_string())
        .collect()
}



/// Dicionário com doenças e seus sintomas característicos
pub fn get_disease_symptoms() -> HashMap<&'static str, Vec<&'static str>> {
    let mut dicionario_sintomas = HashMap::new();

    dicionario_sintomas.insert("dengue", vec![
        "febre alta", "febre", "dor de cabeça", "dor atrás dos olhos",
        "dores no corpo", "dores musculares", "dores articulares",
        "manchas vermelhas", "exantema", "prostração", "fraqueza", "náuseas",
        "vômitos", "mal-estar"
    ]);

    dicionario_sintomas.insert("covid-19", vec![
        "febre", "tosse", "falta de ar", "perda de olfato", "perda de paladar",
        "fadiga", "dor de garganta", "congestão nasal", "dor de cabeça"
    ]);

    dicionario_sintomas.insert("gripe", vec![
        "febre", "tosse", "dor de garganta", "congestão nasal",
        "dor de cabeça", "dores musculares", "fadiga"
    ]);

    dicionario_sintomas.insert("influenza", vec![
        "febre", "tosse", "dor de garganta", "congestão nasal",
        "dor de cabeça", "dores musculares", "fadiga"
    ]);

    dicionario_sintomas.insert("zika", vec![
        "febre baixa", "manchas vermelhas", "dor nas articulações",
        "olhos vermelhos", "dor de cabeça", "coceira"
    ]);

    dicionario_sintomas.insert("chikungunya", vec![
        "febre alta", "dor intensa nas articulações", "dor muscular",
        "dor de cabeça", "erupção cutânea", "fadiga"
    ]);

    dicionario_sintomas.insert("malaria", vec![
        "febre alta", "calafrios", "sudorese", "dor de cabeça",
        "dores musculares", "náuseas", "vômitos", "diarreia"
    ]);

    dicionario_sintomas.insert("diarreia", vec![
        "evacuações líquidas", "diarreia", "cólica", "náuseas",
        "vômitos", "dor abdominal", "febre baixa"
    ]);

    dicionario_sintomas.insert("virose", vec![
        "febre", "dor de cabeça", "dores no corpo", "fadiga",
        "mal-estar", "sintomas respiratórios", "sintomas gastrointestinais"
    ]);

    dicionario_sintomas.insert("sarampo", vec![
        "febre alta", "tosse", "coriza", "conjuntivite",
        "manchas brancas na boca", "erupção cutânea vermelha"
    ]);

    dicionario_sintomas.insert("caxumba", vec![
        "febre", "dor de cabeça", "inchaço das glândulas salivares",
        "dificuldade para mastigar", "dor ao engolir", "fadiga"
    ]);

    dicionario_sintomas.insert("meningite", vec![
        "febre alta", "dor de cabeça intensa", "rigidez no pescoço",
        "náuseas", "vômitos", "confusão mental", "sensibilidade à luz"
    ]);

    dicionario_sintomas.insert("tuberculose", vec![
        "tosse persistente", "expectoração com sangue", "dor no peito",
        "fraqueza", "perda de peso", "febre", "suores noturnos"
    ]);

    dicionario_sintomas.insert("sinusite", vec![
        "congestão nasal", "descarga nasal espessa", "dor facial",
        "dor de cabeça", "tosse", "pressão nos ouvidos"
    ]);

    dicionario_sintomas.insert("rinite", vec![
        "espirros", "coceira no nariz", "coriza", "congestão nasal",
        "coceira nos olhos", "lacrimejamento"
    ]);

    dicionario_sintomas.insert("otite", vec![
        "dor de ouvido", "secreção do ouvido", "perda de audição",
        "febre", "irritabilidade", "problemas de equilíbrio"
    ]);

    dicionario_sintomas.insert("faringite", vec![
        "dor de garganta", "dificuldade para engolir", "febre",
        "gânglios inchados no pescoço", "amígdalas inflamadas", "tosse"
    ]);

    dicionario_sintomas.insert("laringite", vec![
        "rouquidão", "perda de voz", "dor na garganta", "tosse seca",
        "sensação de prurito na garganta", "dificuldade respiratória"
    ]);

    dicionario_sintomas.insert("hemorroida", vec![
        "sangramento anal", "dor no ânus", "coceira anal",
        "protuberância no ânus", "desconforto ao evacuar"
    ]);

    dicionario_sintomas.insert("trombose", vec![
        "dor intensa", "inchaço na área afetada", "vermelhidão",
        "calor local", "sensibilidade ao toque", "nódulo endurecido"
    ]);

    dicionario_sintomas.insert("diabetes", vec![
        "sede excessiva", "urinar frequentemente", "fome excessiva", "fadiga",
        "visão embaçada", "perda de peso", "feridas de cicatrização lenta"
    ]);

    dicionario_sintomas.insert("cefaleia", vec![
        "dor de cabeça", "sensibilidade à luz", "sensibilidade ao som",
        "náuseas", "vômitos", "visão turva"
    ]);

    dicionario_sintomas.insert("pneumonia", vec![
        "tosse com expectoração", "falta de ar", "febre", "calafrios",
        "dor torácica", "respiração rápida", "fadiga"
    ]);

    dicionario_sintomas.insert("hipertensão", vec![
        "dor de cabeça", "tontura", "visão turva", "dor no peito",
        "falta de ar", "palpitações"
    ]);

    dicionario_sintomas.insert("leishmaniose", vec![
        "feridas na pele", "febre", "perda de peso", "inchaço do baço",
        "inchaço do fígado", "anemia", "fraqueza"
    ]);

    dicionario_sintomas.insert("raiva", vec![
        "febre", "dor de cabeça", "irritabilidade", "confusão",
        "paralisia", "hidrofobia", "salivação excessiva"
    ]);

    dicionario_sintomas.insert("febre amarela", vec![
        "febre alta", "dor de cabeça", "dores musculares", "náuseas",
        "vômitos", "icterícia", "sangramento", "insuficiência hepática"
    ]);

    dicionario_sintomas.insert("doença de chagas", vec![
        "febre", "mal-estar", "falta de apetite", "inchaço dos olhos",
        "aumento do baço", "aumento do fígado", "alterações cardíacas"
    ]);

    dicionario_sintomas.insert("esquistossomose", vec![
        "coceira na pele", "febre", "dor abdominal", "diarreia",
        "sangue nas fezes", "aumento do fígado", "aumento do baço"
    ]);

    dicionario_sintomas.insert("filariose linfática", vec![
        "febre", "calafrios", "inchaço dos membros", "dor nos gânglios linfáticos",
        "inflamação da pele", "dor nos testículos"
    ]);

    dicionario_sintomas.insert("leptospirose", vec![
        "febre alta", "dor de cabeça", "dores musculares", "olhos vermelhos",
        "icterícia", "náuseas", "vômitos"
    ]);

    dicionario_sintomas.insert("febre tifoide", vec![
        "febre persistente", "dor de cabeça", "mal-estar", "falta de apetite",
        "manchas rosadas no tronco", "constipação ou diarreia"
    ]);

    dicionario_sintomas.insert("hepatite a", vec![
        "fadiga", "náuseas", "vômitos", "dor abdominal", "icterícia",
        "urina escura", "fezes claras"
    ]);

    dicionario_sintomas.insert("hepatite e", vec![
        "fadiga", "náuseas", "vômitos", "dor abdominal", "icterícia",
        "urina escura", "fezes claras"
    ]);

    dicionario_sintomas.insert("paracoccidioidomicose", vec![
        "lesões na pele", "lesões na mucosa", "tosse crônica",
        "perda de peso", "febre", "aumento dos gânglios linfáticos"
    ]);

    dicionario_sintomas.insert("hantavirose", vec![
        "febre", "dor de cabeça", "dores musculares", "náuseas",
        "vômitos", "dificuldade respiratória", "hipotensão"
    ]);

    dicionario_sintomas.insert("cisticercose", vec![
        "dor de cabeça", "convulsões", "visão turva", "confusão mental",
        "náuseas", "vômitos", "alterações comportamentais"
    ]);

    dicionario_sintomas.insert("oncocercose", vec![
        "coceira intensa", "nódulos sob a pele", "lesões na pele",
        "alterações na visão", "cegueira"
    ]);

    dicionario_sintomas.insert("micoses sistêmicas", vec![
        "febre", "tosse", "perda de peso", "lesões na pele",
        "dor no peito", "dificuldade respiratória"
    ]);

    dicionario_sintomas.insert("febre do nilo ocidental", vec![
        "febre", "dor de cabeça", "fadiga", "dores no corpo",
        "erupção cutânea", "gânglios linfáticos inchados"
    ]);

    dicionario_sintomas.insert("riquetsioses", vec![
        "febre", "dor de cabeça", "erupção cutânea", "dores musculares",
        "calafrios", "mal-estar"
    ]);

    dicionario_sintomas
}




/// Função extract_keyword_kmeans
pub fn extract_keyword_hybrid(df: &DataFrame) -> PolarsResult<DataFrame> {
    // Verificar se o DataFrame tem a coluna necessária
    if !df.schema().iter().any(|(name, _)| name == "IfroConsultaConduta") {
        return Err(PolarsError::ComputeError(
            "Coluna 'IfroConsultaConduta' não encontrada no DataFrame".into(),
        ));
    }

    // Usa uma referência ao DataFrame sempre que possível para evitar clones desnecessários
    let mut df_result = df.clone();

    // Obtem a coluna de texto
    let conduta_col = df.column("IfroConsultaConduta")?;

    // Cast to StringChunked to iterate over string values
    let conduta_series_string = conduta_col.cast(&DataType::String)?;
    let conduta_utf8 = conduta_series_string.str()?; 

    // Preenche valores nulos com "não especificado" e converter para lowercase
    let condutas: Vec<String> = conduta_utf8
        .iter()
        .map(|opt_s| {
            match opt_s {
                Some(s) => s.to_lowercase(), // Process valid strings
                None => "não especificado".to_string(), // Handle nulls
            }
        })
        .collect();

    // Vetor para armazenar as queixas diagnosticadas
    let mut queixas = Vec::with_capacity(condutas.len());

    // Coleta índices e condutas que precisam do K-means
    let mut condutas_kmeans = Vec::new();
    let mut indices_kmeans = Vec::new();

    // Primeira passagem: diagnosticar usando sintomas
    for (i, conduta) in condutas.iter().enumerate() {
        if is_not_specified(conduta) {
            queixas.push("não especificado".to_string());
            continue;
        }

        // Tenta diagnosticar com base nos sintomas
        let (doenca, pontuacao) = diagnosticar_doenca_com_pontuacao(conduta);
        let doenca_normalizada = apply_decision_rules(conduta, &doenca);
        if pontuacao >= 50.0 {
            // Alta confiança: usar o diagnóstico diretamente
            //queixas.push(doenca);
            queixas.push(doenca_normalizada);
        } else if pontuacao < 30.0 {
            // Baixa confiança: marcar como não especificado
            queixas.push("não especificado".to_string());
        } else {
            // Confiança média: usar K-means como suporte
            queixas.push("".to_string()); // Placeholder temporário
            condutas_kmeans.push(conduta.clone());
            indices_kmeans.push(i);
        }
    }

    // Se temos casos para o K-means, processá-los
    if !condutas_kmeans.is_empty() {
        // Obtem a lista de doenças
        let diseases = list_of_most_common_diseases();
        let n_clusters = std::cmp::min(diseases.len() - 1, condutas_kmeans.len()); // Excluindo "não especificado"

        // Processa texto: remove stopwords, aplica stemming, etc.
        let stemmer = Stemmer::create(Algorithm::Portuguese);
        let stopwords = get_portuguese_stopwords();

        // Vetorizaros textos (TF-IDF simplificado)
        let (document_term_matrix, terms) = vectorize_texts(&condutas_kmeans, &stopwords, &stemmer);

        // Aplica K-means
        let dataset = Dataset::from(document_term_matrix);

        // Executa K-means - com tratamento de erro adequado
        let kmeans = KMeans::params(n_clusters)
            .max_n_iterations(100)
            .tolerance(1e-5)
            .fit(&dataset)
            .map_err(|e| PolarsError::ComputeError(
                format!("Falha ao executar K-means: {}", e).into()
            ))?;

        // Predizer clusters
        let preds = kmeans.predict(&dataset);

        // Calcula palavras mais importantes em cada cluster
        let mut cluster_important_terms: Vec<Vec<&String>> = Vec::with_capacity(n_clusters);
        let centroids = kmeans.centroids();

        for cluster_idx in 0..n_clusters {
            // Obte o centroide do cluster
            let centroid = centroids.row(cluster_idx);

            // Cria pares (termo, valor) e ordena por valor decrescente
            let mut term_scores: Vec<(&String, f64)> = terms.iter()
                .enumerate()
                .map(|(term_idx, term)| (term, centroid[term_idx]))
                .collect();

            term_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Pega os 5 termos mais importantes (ou menos se não houver 5)
            let top_n = std::cmp::min(5, term_scores.len());
            let top_terms: Vec<&String> = term_scores.iter()
                .take(top_n)
                .map(|(term, _)| *term)
                .collect();

            cluster_important_terms.push(top_terms);
        }

        // Mapea clusters para doenças considerando tanto o nome quanto os sintomas
        let mut cluster_to_disease = HashMap::new();

        for cluster_idx in 0..n_clusters {
            let top_terms = &cluster_important_terms[cluster_idx];

            // Calcula pontuação de similaridade para cada doença
            let mut best_disease = "não especificado";
            let mut best_score = 0.0;

            let dicionario_sintomas = get_disease_symptoms();

            for &disease in diseases.iter() {
                if disease == "não especificado" {
                    continue;
                }

                let disease_lower = disease.to_lowercase();
                let disease_stems: HashSet<String> = disease_lower
                    .split_whitespace()
                    .map(|word| stemmer.stem(word).to_string())
                    .collect();

                // Calcula pontuação baseada no número de termos em comum
                let mut score = 0.0;

                // Pontuação por correspondência de termos
                for term in top_terms {
                    if disease_stems.contains(*term) || disease_lower.contains(*term) {
                        score += 1.0;
                    }
                }

                // Pontuação adicional por correspondência de sintomas
                if let Some(sintomas_doenca) = dicionario_sintomas.get(disease) {
                    for &sintoma in sintomas_doenca {
                        for term in top_terms {
                            // Dereference term once to get &String, which implements Pattern
                            if (*term).contains(sintoma) || sintoma.contains(*term) {
                                score += 0.5;
                                break;
                            }
                        }
                    }
                }

                if score > best_score {
                    best_score = score;
                    best_disease = disease;
                }
            }

            // Se nenhuma doença tem uma boa pontuação, atribuir sequencialmente
            if best_score == 0.0 {
                best_disease = diseases[cluster_idx % (diseases.len() - 1)]; // Evitar "não especificado"
            }

            cluster_to_disease.insert(cluster_idx, best_disease.to_string());
        }

        // Preenche queixas para os textos que precisam do K-means
        for (i_local, &i_global) in indices_kmeans.iter().enumerate() {
            let cluster = preds[i_local];
            let conduta = &condutas_kmeans[i_local];

            // Aplicar as regras de decisão
            let default_disease = "não especificado".to_string();
            let disease = cluster_to_disease.get(&cluster).unwrap_or(&default_disease);

            // Reforçar com as regras de decisão tradicionais
            let queixa = apply_decision_rules(conduta, disease);
            queixas[i_global] = queixa;
        }
    }

    // Adiciona a coluna ao DataFrame
    let queixas_series = Series::new("IfroPacienteQueixaPrincipal".into(), queixas);
    df_result.with_column(queixas_series)?;

    Ok(df_result)
}



/// Função para diagnosticar_doenca_com_pontuacao com detecção mais precisa

fn diagnosticar_doenca_com_pontuacao(sintomas: &str) -> (String, f64) {
    let sintomas_normalizados = sintomas.to_lowercase();

    let mut pontuacao_doencas = HashMap::new();

    let dicionario_sintomas = get_disease_symptoms();

    for (doenca, sintomas_doenca) in dicionario_sintomas.iter() {
        let mut pontos_totais = 0;
        let mut sintomas_encontrados = 0;

        for &sintoma in sintomas_doenca {
            if sintomas_normalizados.contains(sintoma) {
                sintomas_encontrados += 1;

                let peso = match sintoma {
                    // Sintomas muito específicos recebem peso maior
                    "perda de olfato" | "perda de paladar" | "icterícia" | "hidrofobia" => 2,
                    // Sintomas comuns recebem peso menor
                    "febre" | "dor de cabeça" | "tosse" => 1,
                    // Outros sintomas com peso padrão
                    _ => 1,
                };

                pontos_totais += peso;
            }
        }

        // Calcular a pontuação final
        if !sintomas_doenca.is_empty() {
            // Base na porcentagem de sintomas encontrados
            let porcentagem_sintomas = (sintomas_encontrados as f64 / sintomas_doenca.len() as f64) * 100.0;

            // Ajustar com base na especificidade dos sintomas encontrados
            let pontuacao_ajustada = if sintomas_encontrados > 0 {
                // Fator de ajuste baseado na soma ponderada dos sintomas
                let fator_especificidade = pontos_totais as f64 / sintomas_encontrados as f64;
                porcentagem_sintomas * fator_especificidade
            } else {
                0.0
            };

            // Se o nome da doença aparece explicitamente no texto, aumentar a pontuação
            let pontuacao_final = if sintomas_normalizados.contains(&doenca.to_lowercase()) {
                pontuacao_ajustada + 25.0 // Bônus quando o nome da doença está explícito
            } else {
                pontuacao_ajustada
            };

            pontuacao_doencas.insert(*doenca, pontuacao_final);
        }
    }

    // Encontra a doença com maior pontuação
    if !pontuacao_doencas.is_empty() {
        let (doenca_mais_provavel, pontuacao_maxima) = pontuacao_doencas
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        (doenca_mais_provavel.to_string(), *pontuacao_maxima)
    } else {
        ("não especificado".to_string(), 0.0)
    }
}



pub fn standardize_neighborhood_names(df_bpa: DataFrame, df_neighborhood: DataFrame,) -> PolarsResult<DataFrame> {
    let df_neighborhood = df_neighborhood
        .lazy()
        .group_by(["bairro"])
        .agg([all().first()]) // Garante entradas únicas pegando a primeira ocorrência de cada bairro
        .collect()?;

    df_bpa
        .lazy()
        .join(
            df_neighborhood.lazy(),
            [col("IfroPacienteBairro")], 
            [col("bairro")],        
            JoinType::Left.into(),
        )
        .with_columns([
            // Substitui o nome do bairro pelo padrão
            when(col("map").is_not_null())
                .then(col("map"))
                .otherwise(col("IfroPacienteBairro"))
                .alias("IfroPacienteBairro"),
                
            // Adiciona as novas colunas de coordenadas
            col("lat").alias("IfroPacienteLatitude"),
            col("long").alias("IfroPacienteLongitude"),
        ])
        .drop(["map", "lat", "long"]) // Remove colunas temporárias
        .collect()
}


pub fn fill_null_strings(df: DataFrame) -> PolarsResult<DataFrame> {
    df.lazy()
        .select([
            // Converte todas as colunas para string e preenche nulos
            all().cast(DataType::String).fill_null(lit(""))
        ])
        .collect()
}



pub fn drop_column_if_exists(df: DataFrame, column_name: &str) -> PolarsResult<DataFrame> {
    if df.get_column_names().iter().any(|&name| name == column_name) {
        df.drop(column_name)
    } else {
        Ok(df)
    }
}


pub fn normalize_column_names_of_the_df_to_lower_case(mut df: DataFrame) -> PolarsResult<DataFrame> {
    let new_columns: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|col| col.to_lowercase())
        .collect();
    
    df.set_column_names(&new_columns)?;
    Ok(df)
}



/// Função para obter valores únicos de uma coluna específica
pub fn get_unique_values(df: &DataFrame, column_name: &str) -> PolarsResult<Vec<String>> {
    // Verifica se a coluna existe
    if !df.schema().iter().any(|(name, _)| name == column_name) {
        return Err(PolarsError::ComputeError(
            format!("Coluna '{}' não encontrada no DataFrame", column_name).into(),
        ));
    }
    
    // Obtém valores únicos
    let unique_series = df.column(column_name)?.unique()?;
    
    // Converte para Vec<String>
    let unique_values: Vec<String> = unique_series
        .cast(&DataType::String)?
        .str()?
        .into_iter()
        .filter_map(|opt_s| opt_s.map(String::from))
        .collect();
        
    Ok(unique_values)
}

// Função para ler um DataFrame a partir de um arquivo CSV
pub fn read_df_with_read_csv_options(path: &str) -> PolarsResult<DataFrame> {
    CsvReadOptions::default()
        .with_has_header(true)
        .with_infer_schema_length(Some(10_000))
        .with_ignore_errors(true)
        .try_into_reader_with_file_path(Some(path.into()))?
        .finish()
}


pub fn read_df_from_bytes(file_content: &[u8]) -> PolarsResult<DataFrame> {
    // Cria um arquivo temporário que será automaticamente removido quando sair do escopo
    let mut temp_file = NamedTempFile::new()?;
    
    // Escreve os bytes no arquivo temporário
    temp_file.write_all(file_content)?;
    
    // Obtém o caminho do arquivo temporário
    let temp_path = temp_file.path();
    
    // Lê o arquivo usando o método que já funciona
    let result = read_df_with_read_csv_options(temp_path.to_str().unwrap());
    
    // O arquivo será automaticamente removido quando temp_file sair do escopo
    
    result
}

pub fn create_dataframe(data: &HashMap<String, Vec<Value>>) -> PolarsResult<DataFrame> {
    // Converter cada coluna para o tipo apropriado
    let map: Vec<&str> = data["map"]
        .iter()
        .map(|v| v.as_str().unwrap_or_default())
        .collect();

    let bairro: Vec<&str> = data["bairro"]
        .iter()
        .map(|v| v.as_str().unwrap_or_default())
        .collect();

    let long: Vec<Option<f64>> = data["long"]
        .iter()
        .map(|v| v.as_str().and_then(|s| s.parse().ok()))
        .collect();

    let lat: Vec<Option<f64>> = data["lat"]
        .iter()
        .map(|v| v.as_str().and_then(|s| s.parse().ok()))
        .collect();

    // Criar o DataFrame com a macro df!
    df!(
        "map" => map,
        "bairro" => bairro,
        "long" => long,
        "lat" => lat
    )
}


// Função recursiva para converter chaves para string
pub fn convert_keys_to_str(data: Value) -> Value {
    match data {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                new_map.insert(k.to_string(), convert_keys_to_str(v));
            }
            Value::Object(new_map)
        },
        Value::Array(arr) => {
            let new_arr = arr.into_iter()
                .map(|item| convert_keys_to_str(item))
                .collect();
            Value::Array(new_arr)
        },
        _ => data,
    }
}


// Função auxiliar para criar DataFrame a partir de dicionário
pub fn create_dataframe_from_dict(data: &HashMap<String, Vec<Value>>) -> PolarsResult<DataFrame> {
    // Create a vector to store the Series for each column
    let mut series_vec = Vec::new();
    
    // Process each column in the HashMap
    for (column_name, values) in data {
        // Check the first non-null value to determine the column type
        let first_value = values.iter().find(|v| !v.is_null());
        
        match first_value {
            // String type columns
            Some(value) if value.is_string() => {
                let string_values: Vec<&str> = values
                    .iter()
                    .map(|v| v.as_str().unwrap_or_default())
                    .collect();
                series_vec.push(Series::new(column_name.into(), string_values).into());
            },
            // Numeric columns (assuming float for simplicity)
            Some(value) if value.is_number() => {
                let numeric_values: Vec<Option<f64>> = values
                    .iter()
                    .map(|v| v.as_str().and_then(|s| s.parse().ok()))
                    .collect();
                series_vec.push(Series::new(column_name.into(), numeric_values).into());
            },
            // Boolean columns
            Some(value) if value.is_boolean() => {
                let bool_values: Vec<Option<bool>> = values
                    .iter()
                    .map(|v| v.as_bool())
                    .collect();
                series_vec.push(Series::new(column_name.into(), bool_values).into());
            },
            // Default to a string type if type can't be determined
            _ => {
                let string_values: Vec<&str> = values
                    .iter()
                    .map(|v| v.as_str().unwrap_or_default())
                    .collect();
                series_vec.push(Series::new(column_name.into(), string_values).into());
            }
        }
    }
    
    // Create DataFrame from the Series vector
    DataFrame::new(series_vec)
}
