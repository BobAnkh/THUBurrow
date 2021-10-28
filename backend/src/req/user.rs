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
}

#[derive(Serialize)]
pub struct UserSignupResponse {
    pub success: bool,
    pub error: Vec<String>,
}
