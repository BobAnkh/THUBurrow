use crate::pgdb::{content_post, content_reply};
use rocket::serde::{Deserialize, Serialize};
use sea_orm::{prelude::DateTimeWithTimeZone, FromQueryResult};
use std::convert::From;

pub static POST_PER_PAGE: usize = 20;
pub static REPLY_PER_PAGE: usize = 20;
pub static MAX_SECTION: usize = 3;
pub static MAX_TAG: usize = 10;

/// Section of post
///
/// ## Fields
///
/// - `PostSection::Entertainment`: Entertainment activities
/// - `PostSection::Learning`: Learning issues
/// - `PostSection::Life`: Everyday events
/// - `PostSection::NSFW`: No safe for work
///
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum PostSection {
    Entertainment,
    Learning,
    Life,
    NSFW,
}

/// Last Post Sequence Number
///
/// ## Fields
///
/// - `LastPostSeq::last_value`: Last post sequence number
///
#[derive(Debug, FromQueryResult)]
pub struct LastPostSeq {
    last_value: i64,
}

/// Total Number of Posts
///
/// ## Fields
///
/// - `PostTotalCount::total`: Total number of posts
///
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PostTotalCount {
    pub total: i64,
}

/// Post Create Response
///
/// ## Fields
///
/// - `PostCreateResponse::post_id`: Post id of created post
///
#[derive(Serialize, Deserialize)]
pub struct PostCreateResponse {
    pub post_id: i64,
}

/// Reply Create Response
///
/// ## Fields
///
/// - `ReplyCreateResponse::post_id`: Post id of post for created reply
/// - `ReplyCreateResponse::reply_id`: Reply id of created reply
///
#[derive(Serialize, Deserialize)]
pub struct ReplyCreateResponse {
    pub post_id: i64,
    pub reply_id: i32,
}

/// Post detail information
///
/// ## Fields
///
/// - `PostPage::post_desc`: Post information of the post
/// - `PostPage::reply_page`: Reply information vector of the post, with the length up to ten
/// - `PostPage::page`: Page number of the post
/// - `PostPage::like`: Flag indicating whether the user liked the post
/// - `PostPage::collection`: Flag indicating whether the user collected the post
///
#[derive(Serialize, Deserialize)]
pub struct PostPage {
    pub post_desc: Post,
    pub reply_page: Vec<Reply>,
    pub page: usize,
    pub like: bool,
    pub collection: bool,
}

/// Post general information for one page
///
/// ## Fields
///
/// - `ListPage::post_page`: Post general information vector of the posts, with the length up to ten
/// - `ListPage::page`: Page number of the list
///
#[derive(Serialize, Deserialize)]
pub struct ListPage {
    pub post_page: Vec<PostDisplay>,
    pub page: usize,
}

/// Post create information of request
///
/// ## Fields
///
/// - `PostInfo::title`: Title of the post
/// - `PostInfo::burrow_id`: Burrow id of the post
/// - `PostInfo::section`: Section of the post
/// - `PostInfo::tag`: Tag of the post
/// - `PostInfo::content`: Content of the post
///
#[derive(Serialize, Deserialize)]
pub struct PostInfo {
    pub title: String,
    pub burrow_id: i64,
    pub section: Vec<PostSection>,
    pub tag: Vec<String>,
    pub content: String,
}

/// Post updated information of request
///
/// ## Fields
///
/// - `PostUpdateInfo::title`: New title of updated post
/// - `PostUpdateInfo::section`: New section of updated post
/// - `PostUpdateInfo::tag`: New tag of updated post
///
#[derive(Serialize, Deserialize)]
pub struct PostUpdateInfo {
    pub title: String,
    pub section: Vec<PostSection>,
    pub tag: Vec<String>,
}

/// Reply create information of request
///
/// ## Fields
///
/// - `ReplyInfo::post_id`: Post id of the reply
/// - `ReplyInfo::burrow_id`: Burrow id of the reply
/// - `ReplyInfo::content`: Content of the reply
///
#[derive(Deserialize)]
pub struct ReplyInfo {
    pub post_id: i64,
    pub burrow_id: i64,
    pub content: String,
}

/// Reply update information of request
///
/// ## Fields
///
/// - `ReplyUpdateInfo::post_id`: Post id of updated reply
/// - `ReplyUpdateInfo::reply_id`: Reply id of updated reply
/// - `ReplyUpdateInfo::content`: New content of updated reply
///
#[derive(Deserialize)]
pub struct ReplyUpdateInfo {
    pub post_id: i64,
    pub reply_id: i32,
    pub content: String,
}

/// Post general information
///
/// ## Fields
///
/// - `PostDisplay::post`: Post information of the post
/// - `PostDisplay::like`: Flag indicating whether the user liked the post
/// - `PostDisplay::collection`: Flag indicating whether the user collected the post
/// - `PostDisplay::is_update`: Flag indicating whether the post has new reply, in case that the user collected the post
///
#[derive(Serialize, Deserialize)]
pub struct PostDisplay {
    pub post: Post,
    pub like: bool,
    pub collection: bool,
    pub is_update: bool,
}

/// Post information of database
///
/// ## Fields
///
/// - `Post::post_id`: Post id of the post
/// - `Post::title`: Title of the post
/// - `Post::burrow_id`: Burrow id of the post
/// - `Post::section`: Section of the post
/// - `Post::tag`: Tag of the post
/// - `Post::create_time`: Created time of the post
/// - `Post::update_time`: Updated time of the post
/// - `Post::post_state`: State of the post
/// - `Post::post_type`: Type of the post
/// - `Post::like_num`: Total number of likes of the post
/// - `Post::collection_num`: Total number of collections of the post
/// - `Post::post_len`: Total number of replies of the post
///
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

/// Reply information of database
///
/// ## Fields
///
/// - `Reply::post_id`: Post id of the reply
/// - `Reply::reply_id`: Reply id of the reply
/// - `Reply::burrow_id`: Burrow id of the reply
/// - `Reply::create_time`: Created time of the reply
/// - `Reply::update_time`: Updated time of the reply
/// - `Reply::content`: Content of the reply
/// - `Reply::reply_state`: State of the post
///
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
