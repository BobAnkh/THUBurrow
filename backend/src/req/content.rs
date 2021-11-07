use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct PostResponse {
    pub success: bool,
    pub error: Vec<String>,
    pub post_id: i32,
}

#[derive(Serialize)]
pub struct ReplyResponse {
    pub success: bool,
    pub error: Vec<String>,
    pub post_id: i32,
    pub reply_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ReadResponse {
    pub success: bool,
    pub error: Vec<String>,
    pub subject_info: Subject,
    pub reply_info: Vec<Reply>,
}

#[derive(Deserialize)]
pub struct ContentInfo<'r> {
    pub title: &'r str,
    pub author: &'r str,
    pub anonymous: bool,
    pub section: &'r str,
    pub tag1: &'r str,
    pub tag2: &'r str,
    pub tag3: &'r str,
    pub content: &'r str,
}

#[derive(Deserialize)]
pub struct ReplyInfo<'r> {
    pub author: &'r str,
    pub anonymous: bool,
    pub content: &'r str,
}

#[derive(Serialize, Deserialize)]
pub struct Subject {
    pub post_id: i32,
    pub title: String,
    pub author: String,
    pub anonymous: bool,
    pub created_time: String,
    pub modified_time: String,
    pub section: String,
    pub tag1: String,
    pub tag2: String,
    pub tag3: String,
    pub post_len: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Reply {
    pub post_id: i32,
    pub reply_id: i32,
    pub author: String,
    pub anonymous: bool,
    pub created_time: String,
    pub modified_time: String,
    pub content: String,
}
