use crate::pgdb;
use sea_orm::{entity::*, DatabaseConnection};

pub async fn get_valid_burrow(conn: DatabaseConnection, id: i64) -> Result<Vec<i64>, String> {
    match pgdb::user_status::Entity::find_by_id(id).one(&conn).await {
        Ok(res) => match res {
            Some(user) => {
                let vec_str: Vec<&str> = user.valid_burrow.split(',').collect();
                println!("{:?}", vec_str);
                let vec_i64: Vec<i64> = vec_str.iter().map(|x| x.parse::<i64>().unwrap()).collect();
                Ok(vec_i64)
            }
            None => Err("User not found.".to_string()),
        },
        Err(e) => Err("Database Error:".to_string() + &e.to_string()),
    }
}
