use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::burrow::BurrowMetadata;

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
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserGetFollowResponse {
    pub burrow: BurrowMetadata,
    pub is_update: bool,
}
