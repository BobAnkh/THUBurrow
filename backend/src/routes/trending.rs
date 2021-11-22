use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;

use sea_orm::sea_query::Expr;
use sea_orm::{entity::*, ActiveModelTrait, QueryOrder, QuerySelect};
use sea_orm::{PaginatorTrait, QueryFilter};

// use crate::db::user::Model;
use crate::pgdb;
use crate::pgdb::prelude::*;
use crate::pool::{PgDb, RedisDb};
use crate::req::content::*;
use crate::utils::sso::SsoAuth;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/", routes![read_trending])
}

#[get("/trending")]
pub async fn read_trending(
    // _auth: SsoAuth,
    db: Connection<PgDb>,
    kvdb: Connection<RedisDb>,
) -> (Status, Json<Vec<String>>) {
    let mut kv_conn = kvdb.into_inner();
    let redis_result: Result<Vec<String>, redis::RedisError> = redis::cmd("ZRANGE")
        .arg("trending")
        .arg(0i32)
        .arg(-1i32)
        .query_async(kv_conn.as_mut())
        .await;
    match redis_result {
        Ok(trending) => {
            if trending.len() == 0 {
                log::info!("Cannot find trending, generate new one");
                let pg_con = db.into_inner();
                let query_formula = Expr::cust(
                    r#"(ln("content_post"."post_len")+"content_post"."like_num"+"content_post"."collection_num")/((floor(extract(epoch from (CURRENT_TIMESTAMP - "content_post"."create_time") ) / 60 / 60)/2+floor(extract(epoch from (CURRENT_TIMESTAMP - "content_post"."last_modify_time") ) / 60 / 60)/2+2)^1.2+10)"#,
                );
                let trend_pages = ContentPost::find()
                    .column_as(query_formula, "score")
                    .paginate(&pg_con, 50);
                match trend_pages.fetch_page(0).await {
                    Ok(trend) => {
                        log::info!("selected!{:?}", trend);
                    }
                    Err(e) => {
                        log::warn!("err: {}", e);
                    }
                }
                (Status::Ok, Json(Vec::new()))
            } else {
                log::info!("Find trending");
                (Status::Ok, Json(trending))
            }
        }
        _ => {
            log::warn!("InternalServerError");
            (Status::InternalServerError, Json(Vec::new()))
        }
    }
}
