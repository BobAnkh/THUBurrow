use lazy_static::lazy_static;
use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;

lazy_static! {
    pub static ref POST_PER_PAGE: i32 = 20;
}

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
    pub verification_code: Option<&'r str>,
}

#[derive(Deserialize)]
pub struct UserLoginInfo<'r> {
    pub username: &'r str,
    pub password: &'r str,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserGetBurrowResponse {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub post_num: i32,
}

#[derive(Serialize, Deserialize)]
pub struct UserGetFollowResponse {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub update: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UserGetFavResponse {
    pub post_id: i64,
    pub title: String,
    pub tags: String,
    pub burrow_id: i64,
    pub burrow_name: String,
}

// struct GetBurrow {}
// impl GetBurrow {
//     async fn get_post(
//         burrow: &pgdb::burrow::Model,
//         inner_conn: DatabaseConnection,
//     ) -> Result<i64, Box<dyn std::error::Error>> {
//         let post_num: i64 = match pgdb::content_post::Entity::find()
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
