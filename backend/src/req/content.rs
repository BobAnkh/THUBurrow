use crate::pgdb::{content_post, content_reply};
use rocket::serde::{Deserialize, Serialize};
use sea_orm::prelude::DateTimeWithTimeZone;
use std::convert::From;

pub static POST_PER_PAGE: usize = 20;
pub static REPLY_PER_PAGE: usize = 20;

#[derive(Serialize, Deserialize)]
pub struct PostCreateResponse {
    pub post_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct PostUpdateInfo {
    pub title: String,
    pub section: Vec<String>,
    pub tag: Vec<String>,
}

#[derive(Serialize)]
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
    pub section: Vec<String>,
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

#[derive(Serialize, Deserialize)]
pub struct Post {
    pub post_id: i64,
    pub title: String,
    pub burrow_id: i64,
    pub section: Vec<String>,
    pub tag: Vec<String>,
    pub create_time: DateTimeWithTimeZone,
    pub update_time: DateTimeWithTimeZone,
    pub post_state: i32,
    pub post_type: i32,
    pub like_num: i32,
    pub collection_num: i32,
    pub post_len: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Reply {
    pub post_id: i64,
    pub reply_id: i32,
    pub burrow_id: i64,
    pub create_time: DateTimeWithTimeZone,
    pub update_time: DateTimeWithTimeZone,
    pub content: String,
    pub reply_state: i32,
}

// TODO: According to post_state to determine whether the post is visible
impl From<content_post::Model> for Post {
    fn from(post_info: content_post::Model) -> Post {
        Post {
            post_id: post_info.post_id,
            title: post_info.title,
            burrow_id: post_info.burrow_id,
            section: post_info.section.split(',').map(str::to_string).collect(),
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
            title: post_info.title.to_owned(),
            burrow_id: post_info.burrow_id,
            section: post_info.section.split(',').map(str::to_string).collect(),
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
            content: reply_info.content,
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
            content: reply_info.content.to_owned(),
            reply_state: reply_info.reply_state,
        }
    }
}
