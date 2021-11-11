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
