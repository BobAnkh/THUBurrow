//! Models of burrow

use crate::{db::burrow, models::content::Post};
use rocket::serde::{Deserialize, Serialize};
use sea_orm::FromQueryResult;

/// Largest burrow_id from query
///
/// ## Fields
///
/// - `last_value`: i64, the largest burrow_id from query
#[derive(Debug, FromQueryResult)]
pub struct LastBurrowSeq {
    last_value: i64,
}

/// Total count of burrows
///
/// ## Fields
///
/// - `total`: i64, total burrow count
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct BurrowTotalCount {
    pub total: i64,
}

/// Burrow Info
///
/// ## Fields
///
/// - `description`: String, description of burrow
/// - `title`: String, title of burrow
#[derive(Serialize, Deserialize)]
pub struct BurrowInfo {
    pub description: String,
    pub title: String,
}

/// Response struct of `create_burrow`
///
/// ## Fields
///
/// - `burrow_id`: i64, burrow_id of created burrow
#[derive(Serialize, Deserialize)]
pub struct BurrowCreateResponse {
    pub burrow_id: i64,
}

/// Response struct of `show_burrow`
///
/// ## Fields
///
/// - `title`: String, title of burrow
/// - `description`: String, description of burrow
/// - `posts`: Vec<Post>, information of posts in burrow
#[derive(Serialize, Deserialize)]
pub struct BurrowShowResponse {
    pub title: String,
    pub description: String,
    pub posts: Vec<Post>,
}

/// Burrow Metadata
///
/// ## Fields
///
/// - `burrow_id`: i64, burrow_id of burrow
/// - `title`: String, title of burrow
/// - `description`: String, description of burrow
/// - `post_num`: i32, post count in burrow
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BurrowMetadata {
    pub burrow_id: i64,
    pub title: String,
    pub description: String,
    pub post_num: i32,
}

// TODO: According to burrow state to determine whether to show burrow
impl From<burrow::Model> for BurrowMetadata {
    fn from(burrow: burrow::Model) -> BurrowMetadata {
        match burrow.burrow_state {
            0 => BurrowMetadata {
                burrow_id: burrow.burrow_id,
                title: burrow.title,
                description: burrow.description,
                post_num: burrow.post_num,
            },
            _ => BurrowMetadata {
                burrow_id: burrow.burrow_id,
                title: "Admin has banned this burrow".to_string(),
                description: "Admin has banned this burrow".to_string(),
                post_num: burrow.post_num,
            },
        }
    }
}

impl From<&burrow::Model> for BurrowMetadata {
    fn from(burrow: &burrow::Model) -> BurrowMetadata {
        match burrow.burrow_state {
            0 => BurrowMetadata {
                burrow_id: burrow.burrow_id,
                title: burrow.title.clone(),
                description: burrow.description.clone(),
                post_num: burrow.post_num,
            },
            _ => BurrowMetadata {
                burrow_id: burrow.burrow_id,
                title: "Admin has banned this burrow".to_string(),
                description: "Admin has banned this burrow".to_string(),
                post_num: burrow.post_num,
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, Utc};

    #[test]
    fn test_burrow_metadata() {
        // get timestamp
        let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let burrow_id: i64 = 666;
        let title = "title".to_string();
        let description = "description".to_string();
        let post_num: i32 = 10;
        let burrow = burrow::Model {
            burrow_id,
            title: title.clone(),
            description: description.clone(),
            uid: 123i64,
            burrow_state: 0i32,
            post_num,
            create_time: now,
            update_time: now,
            credit: 0i32,
            badge: "badge".to_string(),
            avatar: "default.jpg".to_string(),
        };
        // let burrow_banned = burrow::Model {
        //     burrow_id,
        //     title: title.clone(),
        //     description: description.clone(),
        //     uid: 123i64,
        //     burrow_state: 0i32,
        //     post_num,
        //     create_time: now,
        //     update_time: now,
        //     credit: 1i32,
        //     badge: "badge".to_string(),
        // };
        let burrow_ref = &burrow;
        // let burrow_banned_ref = &burrow_banned;
        let burrow_data = BurrowMetadata {
            burrow_id,
            title,
            description: description.clone(),
            post_num,
        };
        // let burrow_banned_data = BurrowMetadata {
        //     burrow_id,
        //     title: "Admin has banned this burrow".to_string(),
        //     description,
        //     post_num,
        // };
        assert_eq!(burrow_data, burrow_ref.into());
        // assert_eq!(burrow_banned_data, burrow_banned_ref.into());
        assert_eq!(burrow_data, burrow.into());
        // assert_eq!(burrow_banned_data, burrow_banned.into());
    }

    #[test]
    fn test_burrow_count() {
        let last_value: i64 = 666;
        let seq = LastBurrowSeq { last_value };
        let seq_ref = &seq;
        let burrow_cnt = BurrowTotalCount { total: last_value };
        assert_eq!(burrow_cnt, seq_ref.into());
        assert_eq!(burrow_cnt, seq.into());
    }
}
