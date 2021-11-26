use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BurrowInfo {
    pub description: Option<String>,
    pub title: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct BurrowCreateResponse {
    pub id: i64,
    pub author: i64,
    pub title: String,
    pub description: Option<String>,
}
