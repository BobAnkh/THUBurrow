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

#[derive(Serialize, Deserialize)]
pub struct PostReadResponse {
    pub errors: String,
    pub post_page: Option<PostPage>,
}

#[derive(Serialize, Deserialize)]
pub struct PostPage {
    pub post_desc: Post,
    pub reply_page: Vec<Reply>,
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

#[derive(Serialize, Deserialize)]
pub struct Post {
    pub title: String,
    pub burrow_id: i64,
    pub section: Vec<String>,
    pub tag: Vec<String>,
    pub create_time: DateTimeWithTimeZone,
    pub last_modify_time: DateTimeWithTimeZone,
    pub post_state: i16,
}

#[derive(Serialize, Deserialize)]
pub struct Reply {
    pub post_id: i64,
    pub reply_id: i32,
    pub burrow_id: i64,
    pub create_time: DateTimeWithTimeZone,
    pub last_modify_time: DateTimeWithTimeZone,
    pub content: String,
    pub reply_state: i16,
}

impl From<content_post::Model> for Post {
    fn from(post_info: content_post::Model) -> Post {
        Post {
            title: post_info.title,
            burrow_id: post_info.burrow_id,
            section: post_info.section.split(',').map(str::to_string).collect(),
            tag: post_info.tag.split(',').map(str::to_string).collect(),
            create_time: post_info.create_time,
            last_modify_time: post_info.last_modify_time,
            post_state: post_info.post_state,
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
            last_modify_time: reply_info.last_modify_time,
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
            create_time: reply_info.create_time.to_owned(),
            last_modify_time: reply_info.last_modify_time.to_owned(),
            content: reply_info.content.to_owned(),
            reply_state: reply_info.reply_state,
        }
    }
}
