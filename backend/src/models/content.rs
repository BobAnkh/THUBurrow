use crate::pgdb::{content_post, content_reply};
use rocket::serde::{Deserialize, Serialize};
use sea_orm::{prelude::DateTimeWithTimeZone, FromQueryResult};
use std::convert::From;

pub static POST_PER_PAGE: usize = 20;
pub static REPLY_PER_PAGE: usize = 20;
pub static MAX_SECTION: usize = 3;
pub static MAX_TAG: usize = 10;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum PostSection {
    Learning,
    Life,
    NSFW,
    XXG,
}

#[derive(Debug, FromQueryResult)]
pub struct LastPostSeq {
    last_value: i64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PostTotalCount {
    pub total: i64,
}

#[derive(Serialize, Deserialize)]
pub struct PostCreateResponse {
    pub post_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct PostUpdateInfo {
    pub title: String,
    pub section: Vec<PostSection>,
    pub tag: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ReplyCreateResponse {
    pub post_id: i64,
    pub reply_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct PostPage {
    pub post_desc: Post,
    pub reply_page: Vec<Reply>,
    pub page: usize,
    pub like: bool,
    pub collection: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ListPage {
    pub post_page: Vec<PostDisplay>,
    pub page: usize,
}

#[derive(Serialize, Deserialize)]
pub struct PostInfo {
    pub title: String,
    pub burrow_id: i64,
    pub section: Vec<PostSection>,
    pub tag: Vec<String>,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ReplyInfo {
    pub post_id: i64,
    pub burrow_id: i64,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ReplyUpdateInfo {
    pub post_id: i64,
    pub reply_id: i32,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct PostDisplay {
    pub post: Post,
    pub like: bool,
    pub collection: bool,
    pub is_update: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Post {
    pub post_id: i64,
    pub title: String,
    pub burrow_id: i64,
    pub section: Vec<PostSection>,
    pub tag: Vec<String>,
    pub create_time: DateTimeWithTimeZone,
    pub update_time: DateTimeWithTimeZone,
    pub post_state: i32,
    pub post_type: i32,
    pub like_num: i32,
    pub collection_num: i32,
    pub post_len: i32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Reply {
    pub post_id: i64,
    pub reply_id: i32,
    pub burrow_id: i64,
    pub create_time: DateTimeWithTimeZone,
    pub update_time: DateTimeWithTimeZone,
    pub content: String,
    pub reply_state: i32,
}

// pub struct GetPostList {}
// impl GetPostList {
//     pub async fn get_post_display(
//         post: &pgdb::content_post::Model,
//         inner_conn: DatabaseConnection,
//         uid: i64,
//     ) -> Result<PostDisplay, Box<dyn std::error::Error>> {
//         let like: bool = match pgdb::user_like::Entity::find_by_id((uid, post.post_id))
//             .one(&inner_conn)
//             .await
//         {
//             Ok(user_like) => user_like.is_some(),
//             Err(e) => {
//                 error!("[GET-BURROW] Database Error: {:?}", e.to_string());
//                 false
//             }
//         };
//         let collection: bool = match pgdb::user_collection::Entity::find_by_id((uid, post.post_id))
//             .one(&inner_conn)
//             .await
//         {
//             Ok(user_collection) => user_collection.is_some(),
//             Err(e) => {
//                 error!("[GET-BURROW] Database Error: {:?}", e.to_string());
//                 false
//             }
//         };
//         Ok(PostDisplay {
//             post: post.into(),
//             like,
//             collection,
//             is_update: false,
//         })
//     }
// }

impl std::fmt::Display for PostSection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

// TODO: According to post_state to determine whether the post is visible
impl From<content_post::Model> for Post {
    fn from(post_info: content_post::Model) -> Post {
        Post {
            post_id: post_info.post_id,
            title: {
                match post_info.post_state {
                    0 => post_info.title,
                    _ => "Admin has banned this post".to_string(),
                }
            },
            burrow_id: post_info.burrow_id,
            section: serde_json::from_str(&post_info.section).unwrap(),
            tag: post_info.tag.split(',').map(str::to_string).collect(),
            create_time: post_info.create_time,
            update_time: post_info.update_time,
            post_state: post_info.post_state,
            post_type: post_info.post_type,
            like_num: post_info.like_num,
            collection_num: post_info.collection_num,
            post_len: post_info.post_len,
        }
    }
}

impl From<&content_post::Model> for Post {
    fn from(post_info: &content_post::Model) -> Post {
        Post {
            post_id: post_info.post_id,
            title: {
                match post_info.post_state {
                    0 => post_info.title.to_owned(),
                    _ => "Admin has banned this post".to_string(),
                }
            },
            burrow_id: post_info.burrow_id,
            section: serde_json::from_str(&post_info.section).unwrap(),
            tag: post_info.tag.split(',').map(str::to_string).collect(),
            create_time: post_info.create_time,
            update_time: post_info.update_time,
            post_state: post_info.post_state,
            post_type: post_info.post_type,
            like_num: post_info.like_num,
            collection_num: post_info.collection_num,
            post_len: post_info.post_len,
        }
    }
}

impl From<content_reply::Model> for Reply {
    fn from(reply_info: content_reply::Model) -> Reply {
        Reply {
            post_id: reply_info.post_id,
            reply_id: reply_info.reply_id,
            burrow_id: reply_info.burrow_id,
            create_time: reply_info.create_time,
            update_time: reply_info.update_time,
            content: {
                match reply_info.reply_state {
                    0 => reply_info.content,
                    _ => "Admin has banned this reply".to_string(),
                }
            },
            reply_state: reply_info.reply_state,
        }
    }
}

impl From<&content_reply::Model> for Reply {
    fn from(reply_info: &content_reply::Model) -> Reply {
        Reply {
            post_id: reply_info.post_id,
            reply_id: reply_info.reply_id,
            burrow_id: reply_info.burrow_id,
            create_time: reply_info.create_time,
            update_time: reply_info.update_time,
            content: {
                match reply_info.reply_state {
                    0 => reply_info.content.to_owned(),
                    _ => "Admin has banned this reply".to_string(),
                }
            },
            reply_state: reply_info.reply_state,
        }
    }
}

impl From<LastPostSeq> for PostTotalCount {
    fn from(seq: LastPostSeq) -> PostTotalCount {
        PostTotalCount {
            total: seq.last_value,
        }
    }
}

impl From<&LastPostSeq> for PostTotalCount {
    fn from(seq: &LastPostSeq) -> PostTotalCount {
        PostTotalCount {
            total: seq.last_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, Utc};

    #[test]
    fn test_content_post() {
        // get timestamp
        let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let post_id: i64 = 1;
        let title = "title".to_string();
        let burrow_id: i64 = 666;
        let section = vec![PostSection::Learning, PostSection::Life, PostSection::NSFW];
        let tag = vec!["TestTag".to_string()];
        let post_state: i32 = 0;
        let post_banned_state: i32 = 1;
        let post_type: i32 = 0;
        let like_num: i32 = 99;
        let collection_num: i32 = 99;
        let post_len: i32 = 10;
        let post_data = Post {
            post_id,
            title: title.clone(),
            burrow_id,
            section: section.clone(),
            tag: tag.clone(),
            create_time: now,
            update_time: now,
            post_state,
            post_type,
            like_num,
            collection_num,
            post_len,
        };
        let post_banned_data = Post {
            post_id,
            title: "Admin has banned this post".to_string(),
            burrow_id,
            section: section.clone(),
            tag: tag.clone(),
            create_time: now,
            update_time: now,
            post_state: post_banned_state,
            post_type,
            like_num,
            collection_num,
            post_len,
        };
        let post_info = content_post::Model {
            post_id,
            title: title.clone(),
            burrow_id,
            create_time: now,
            update_time: now,
            section: serde_json::to_string(&section).unwrap(),
            tag: tag.join(","),
            post_state,
            post_type,
            like_num,
            collection_num,
            post_len,
        };
        let post_banned_info = content_post::Model {
            post_id,
            title: title.clone(),
            burrow_id,
            create_time: now,
            update_time: now,
            section: serde_json::to_string(&section).unwrap(),
            tag: tag.join(","),
            post_state: post_banned_state,
            post_type,
            like_num,
            collection_num,
            post_len,
        };
        let post_info_ref = &post_info;
        let post_banned_info_ref = &post_banned_info;
        assert_eq!(post_data, post_info_ref.into());
        assert_eq!(post_data, post_info.into());
        assert_eq!(post_banned_data, post_banned_info_ref.into());
        assert_eq!(post_banned_data, post_banned_info.into());
    }

    #[test]
    fn test_content_reply() {
        // get timestamp
        let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let post_id: i64 = 1;
        let reply_id: i32 = 2;
        let burrow_id: i64 = 666;
        let content = "content".to_string();
        let reply_state: i32 = 0;
        let reply_banned_state: i32 = 1;
        let reply_data = Reply {
            post_id,
            reply_id,
            burrow_id,
            create_time: now,
            update_time: now,
            content: content.clone(),
            reply_state,
        };
        let reply_banned_data = Reply {
            post_id,
            reply_id,
            burrow_id,
            create_time: now,
            update_time: now,
            content: "Admin has banned this reply".to_string(),
            reply_state: reply_banned_state,
        };
        let reply_info = content_reply::Model {
            post_id,
            reply_id,
            burrow_id,
            create_time: now,
            update_time: now,
            content: content.clone(),
            reply_state,
        };
        let reply_banned_info = content_reply::Model {
            post_id,
            reply_id,
            burrow_id,
            create_time: now,
            update_time: now,
            content: content.clone(),
            reply_state: reply_banned_state,
        };
        let reply_info_ref = &reply_info;
        let reply_banned_info_ref = &reply_banned_info;
        assert_eq!(reply_data, reply_info_ref.into());
        assert_eq!(reply_data, reply_info_ref.into());
        assert_eq!(reply_banned_data, reply_banned_info_ref.into());
        assert_eq!(reply_banned_data, reply_banned_info.into());
    }

    #[test]
    fn test_post_count() {
        let last_value: i64 = 666;
        let seq = LastPostSeq { last_value };
        let seq_ref = &seq;
        let post_cnt = PostTotalCount { total: last_value };
        assert_eq!(post_cnt, seq_ref.into());
        assert_eq!(post_cnt, seq.into());
    }
}
