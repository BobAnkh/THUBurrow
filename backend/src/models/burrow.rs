use crate::{models::content::Post, pgdb::burrow};
use rocket::serde::{Deserialize, Serialize};
use sea_orm::FromQueryResult;

pub static BURROW_PER_PAGE: usize = 10;
pub static BURROW_LIMIT: usize = 5;

#[derive(Debug, FromQueryResult)]
pub struct LastBurrowSeq {
    last_value: i64,
}

#[derive(Serialize, Deserialize)]
pub struct BurrowTotalCount {
    pub total: i64,
}

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

#[derive(Serialize, Deserialize, Clone)]
pub struct BurrowMetadata {
    pub burrow_id: i64,
    pub title: String,
    pub description: String,
    pub post_num: i32,
}

// TODO: According to burrow state to determine whether to show burrow
impl From<burrow::Model> for BurrowMetadata {
    fn from(burrow: burrow::Model) -> BurrowMetadata {
        BurrowMetadata {
            burrow_id: burrow.burrow_id,
            title: burrow.title.clone(),
            description: burrow.description.clone(),
            post_num: burrow.post_num,
        }
    }
}

impl From<&burrow::Model> for BurrowMetadata {
    fn from(burrow: &burrow::Model) -> BurrowMetadata {
        BurrowMetadata {
            burrow_id: burrow.burrow_id,
            title: burrow.title.clone(),
            description: burrow.description.clone(),
            post_num: burrow.post_num,
        }
    }
}

impl From<LastBurrowSeq> for BurrowTotalCount {
    fn from(seq: LastBurrowSeq) -> BurrowTotalCount {
        BurrowTotalCount {
            total: seq.last_value,
        }
    }
}

impl From<&LastBurrowSeq> for BurrowTotalCount {
    fn from(seq: &LastBurrowSeq) -> BurrowTotalCount {
        BurrowTotalCount {
            total: seq.last_value,
        }
    }
}
