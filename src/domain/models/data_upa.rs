use serde::Serialize;


#[derive(Debug, Serialize)]
pub struct HealthUnit {
    #[serde(rename = "ifrounidadeid")]
    pub id: i64,
    #[serde(rename = "ifrounidadenome")]
    pub name: String,
}