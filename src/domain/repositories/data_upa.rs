use async_trait::async_trait;
use polars::frame::DataFrame;
use std::collections::HashMap;
use std::error::Error;
use serde_json::Value;

#[async_trait]
pub trait DataRepository: Send + Sync + 'static {
    async fn fetch_all_data(&self, table: &str) -> Result<HashMap<String, Vec<Value>>, Box<dyn Error + Send + Sync>>;
    async fn check_ifrocompetencia_exists(&self, table: &str, competencia_values: &[String]) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn create_table_if_not_exists(&self, df: &DataFrame, table: &str) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn insert_data(&self, df: &DataFrame, table: &str) -> Result<bool, Box<dyn Error + Send + Sync>>;
}