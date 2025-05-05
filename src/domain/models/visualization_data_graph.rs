#[derive(serde::Deserialize)]
pub struct DataAccessParams {
    pub user_id: String, 
    pub unidade_id: i32,
}