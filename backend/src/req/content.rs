use crate::pgdb::{content_post, content_reply};
use rocket::serde::{Deserialize, Serialize};
use sea_orm::prelude::DateTimeWithTimeZone;
use std::convert::From;

#[derive(Serialize)]
pub struct PostCreateResponse {
    pub errors: Vec<String>,
    pub post_id: i64,
}

#[derive(Serialize)]
pub struct PostDeleteResponse {
    pub errors: Vec<String>,
    pub post_id: i64,
}

#[derive(Serialize)]
pub struct ReplyCreateResponse {
    pub errors: Vec<String>,
    pub post_id: i64,
    pub reply_id: i32,
}

#[derive(Serialize)]
pub struct ReplyUpdateResponse {
    pub errors: Vec<String>,
    pub post_id: i64,
    pub reply_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct PostReadResponse {
    pub errors: String,
    pub post_page: Option<PostPage>,
    pub like: bool,
    pub collection: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ListReadResponse {
    pub errors: String,
    pub list_page: Option<ListPage>,
}

#[derive(Serialize, Deserialize)]
pub struct PostPage {
    pub post_desc: Post,
    pub reply_page: Vec<Reply>,
    pub page: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ListPage {
    pub post_page: Vec<Post>,
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
pub struct Post {
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

impl From<content_post::Model> for Post {
    fn from(post_info: content_post::Model) -> Post {
        Post {
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
        }
    }
}

impl From<&content_post::Model> for Post {
    fn from(post_info: &content_post::Model) -> Post {
        Post {
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
