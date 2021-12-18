use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;
// use crate::pgdb::{self, prelude::*};
use super::{burrow::BurrowMetadata, content::Post};

pub static SEND_EMAIL_LIMIT: usize = 3;

#[derive(Serialize, Deserialize)]
pub struct UserData {
    pub id: Uuid,
    pub name: String,
}

#[derive(Deserialize)]
pub struct UserInfo<'r> {
    pub username: &'r str,
    pub password: &'r str,
    pub email: &'r str,
    pub verification_code: &'r str,
}

#[derive(Deserialize)]
pub struct UserResetInfo<'r> {
    pub password: &'r str,
    pub email: &'r str,
    pub verification_code: &'r str,
}

#[derive(Deserialize)]
pub struct UserChangePassword<'r> {
    pub password: &'r str,
    pub new_password: &'r str,
}

#[derive(Serialize, Deserialize)]
pub struct UserEmail {
    pub email: String,
}

#[derive(Deserialize)]
pub struct UserLoginInfo<'r> {
    pub username: &'r str,
    pub password: &'r str,
}

#[derive(Serialize, Deserialize)]
pub struct UserResponse {
    pub default_burrow: i64,
}

#[derive(Serialize, Deserialize)]
pub struct UserGetCollectionResponse {
    pub post: Post,
    pub is_update: bool,
}

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
