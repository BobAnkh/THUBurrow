use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BurrowInfo {
    pub description: String,
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct BurrowCreateResponse {
    pub burrow_id: i64,
    pub uid: i64,
    pub title: String,
    pub description: String,
}
