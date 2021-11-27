use crate::pgdb;
use sea_orm::{entity::*, DatabaseConnection};

pub async fn get_valid_burrow(conn: &DatabaseConnection, id: i64) -> Result<Vec<i64>, String> {
    match pgdb::user_status::Entity::find_by_id(id).one(conn).await {
        Ok(res) => match res {
            Some(user) => {
                let mut vec_str: Vec<&str> = user.valid_burrow.split(',').collect();
                match vec_str.pop() {
                    Some(_) => Ok(vec_str.iter().map(|x| x.parse::<i64>().unwrap()).collect()),
                    None => Ok(Vec::new()),
                }
            }
            None => Err("User not found.".to_string()),
        },
        Err(e) => Err("Database Error:".to_string() + &e.to_string()),
    }
}

pub async fn get_banned_burrow(conn: &DatabaseConnection, id: i64) -> Result<Vec<i64>, String> {
    match pgdb::user_status::Entity::find_by_id(id).one(conn).await {
        Ok(res) => match res {
            Some(user) => {
                let mut vec_str: Vec<&str> = user.banned_burrow.split(',').collect();
                match vec_str.pop() {
                    Some(_) => Ok(vec_str.iter().map(|x| x.parse::<i64>().unwrap()).collect()),
                    None => Ok(Vec::new()),
                }
            }
            None => Err("User not found.".to_string()),
        },
        Err(e) => Err("Database Error:".to_string() + &e.to_string()),
    }
}
