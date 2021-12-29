//! Models for user

use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;
// use crate::pgdb::{self, prelude::*};
use super::{burrow::BurrowMetadata, content::Post};

/// User data
///
/// ## Fields
///
/// - `id`: Uuid, user's uuid
/// - `name`: String, username
#[derive(Serialize, Deserialize)]
pub struct UserData {
    pub id: Uuid,
    pub name: String,
}

/// User Info
///
/// ## Fields
///
/// - `username`: &str, username
/// - `password`: &str, user's password
/// - `email`: &str, user's email
/// - `verification_code`: &str, verification code
#[derive(Deserialize)]
pub struct UserInfo<'r> {
    pub username: &'r str,
    pub password: &'r str,
    pub email: &'r str,
    pub verification_code: &'r str,
}

/// User Reset Info
///
/// ## Fields
///
/// - `password`: &str, user's password
/// - `email`: &str, user's email
/// - `verification_code`: &str, verification code
#[derive(Deserialize)]
pub struct UserResetInfo<'r> {
    pub password: &'r str,
    pub email: &'r str,
    pub verification_code: &'r str,
}

/// Input struct of `user_change_password`
///
/// ## Fields
///
/// - `password`: &str, user's old password
/// - `new_password`: &str, new password
#[derive(Deserialize)]
pub struct UserChangePassword<'r> {
    pub password: &'r str,
    pub new_password: &'r str,
}

/// User Email
///
/// ## Fields
///
/// - `email`: String, user's email
#[derive(Serialize, Deserialize)]
pub struct UserEmail {
    pub email: String,
}

/// User Login Info
///
/// ## Fields
///
/// - `username`: &str, username
/// - `password`: &str, user's password
#[derive(Deserialize)]
pub struct UserLoginInfo<'r> {
    pub username: &'r str,
    pub password: &'r str,
}

/// Response struct of `user_sign_up`
///
/// ## Fields
///
/// - `default_burrow`: i64, burrow_id of assigned default burrow
#[derive(Serialize, Deserialize)]
pub struct UserResponse {
    pub default_burrow: i64,
}

/// Response struct of `get_collection`
///
/// ## Fields
///
/// - `post`: struct Post, information of post
/// - `is_update`: bool, if post is updated since last view
#[derive(Serialize, Deserialize)]
pub struct UserGetCollectionResponse {
    pub post: Post,
    pub is_update: bool,
}

/// Response struct of `get_follow`
///
/// ## Fields
///
/// - `burrow`: struct BurrowMetadata, information of burrow
/// - `is_update`: bool, if burrow is updated since last view
#[derive(Serialize, Deserialize)]
pub struct UserGetFollowResponse {
    pub burrow: BurrowMetadata,
    pub is_update: bool,
}

// pub struct UserGetFavResponse {
//     pub post_id: i64,
//     pub title: String,
//     pub tags: String,
//     pub burrow_id: i64,
//     pub burrow_name: String,
// }

// pub struct GetBatch {}
// impl GetBatch {
//     async fn get_post(
//         burrow: &pgdb::burrow::Model,
//         inner_conn: DatabaseConnection,
//     ) -> Result<i64, Box<dyn std::error::Error>> {
//         let post_num: i64 = match ContentPost::find()
//             .filter(pgdb::content_post::Column::BurrowId.eq(burrow.burrow_id))
//             .all(&inner_conn)
//             .await
//         {
//             Ok(posts) => match posts.len().try_into() {
//                 Ok(n) => n,
//                 Err(e) => {
//                     error!("[GET-BURROW] TryInto Error: {:?}", e.to_string());
//                     -1
//                 }
//             },
//             Err(e) => {
//                 error!("[GET-BURROW] Database Error: {:?}", e.to_string());
//                 -1
//             }
//         };
//         Ok(post_num)
//     }
// }
