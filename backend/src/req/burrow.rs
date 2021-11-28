use crate::req::content::Post;
use lazy_static::lazy_static;
use rocket::serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref BURROW_LIMIT: i32 = 5;
    pub static ref BURROW_PER_PAGE: usize = 10;
}

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

#[derive(Serialize, Deserialize)]
pub struct BurrowShowResponse {
    pub title: String,
    pub description: String,
    pub posts: Vec<Post>,
}
