use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub post_num: i64,
}

// impl UserGetBurrowResponse {
//     pub async fn resolve(&mut self, x: Model, pg_con: DatabaseConnection) {
//         self.id = x.burrow_id;
//         self.title = x.title.clone();
//         self.description = x.description.clone();
//         self.post_num = {
//             match pgdb::content_post::Entity::find()
//                 .filter(pgdb::content_post::Column::BurrowId.eq(x.burrow_id))
//                 .all(&pg_con)
//                 .await
//             {
//                 Ok(posts) => {
//                     match posts.len().try_into(){
//                         Ok(n) => n,
//                         Err(e) => {
//                             error!("[GET-BURROW] Database Error: {:?}", e.to_string());
//                             -1
//                         },
//                     }
//                 },
//                 Err(e) => {
//                     error!("[GET-BURROW] Database Error: {:?}", e.to_string());
//                     -1
//                 },
//             }
//         };
//     }
// }

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
