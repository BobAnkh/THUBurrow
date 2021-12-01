use crate::req::content::Post;
use rocket::serde::{Deserialize, Serialize};

pub static BURROW_PER_PAGE: usize = 10;
pub static BURROW_LIMIT: usize = 5;

#[derive(Serialize, Deserialize)]
pub struct BurrowInfo {
    pub description: String,
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct BurrowCreateResponse {
    pub burrow_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct BurrowShowResponse {
    pub title: String,
    pub description: String,
    pub posts: Vec<Post>,
}
