use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
use sea_orm::sea_query::Expr;
use sea_orm::{entity::*, DatabaseConnection, PaginatorTrait, QueryFilter, QueryOrder};

use crate::models::content::*;
use crate::models::error::*;
use crate::pgdb::{content_post, prelude::*};
use crate::pool::{PgDb, RedisDb};
use crate::utils::auth::Auth;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/", routes![read_trending])
}

/// Get Trending
///
/// ## Parameters
///
/// - `Auth`: Authenticated user
/// - `Connection<PgDb>`: Postgres connection
/// - `Connection<RedisDb>`: Redis connection
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `String`: Json string of posts
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::DatabaseErr`
#[get("/trending")]
pub async fn read_trending(
    _auth: Auth,
    db: Connection<PgDb>,
    kvdb: Connection<RedisDb>,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    let mut kv_conn = kvdb.into_inner();
    let redis_result: Result<Option<String>, redis::RedisError> = redis::cmd("GET")
        .arg("trending")
        .query_async(kv_conn.as_mut())
        .await;
    match redis_result {
        Ok(trend) => match trend {
            None => {
                log::info!("Cannot find trending, generate new one");
                let pg_con = db.into_inner();
                match select_trending(&pg_con, kv_conn.as_mut()).await {
                    Ok(trending) => (Status::Ok, Ok(trending)),
                    Err(e) => {
                        log::error!("[TRENDING] Database Error: {}", e);
                        (
                            Status::InternalServerError,
                            Err(Json(ErrorResponse::default())),
                        )
                    }
                }
            }
            Some(trending) => {
                log::info!("Find trending");
                (Status::Ok, Ok(trending))
            }
        },
        Err(e) => {
            log::error!("[TRENDING] Database Error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

pub async fn select_trending(
    pg_con: &DatabaseConnection,
    kv_conn: &mut redis::aio::Connection,
) -> Result<String, String> {
    let query_formula = Expr::cust(
        r#"(ln("content_post"."post_len")+"content_post"."like_num"+"content_post"."collection_num")/((floor(extract(epoch from (CURRENT_TIMESTAMP - "content_post"."create_time") ) / 60 / 60)/2+floor(extract(epoch from (CURRENT_TIMESTAMP - "content_post"."update_time") ) / 60 / 60)/2+2)^1.2+10)"#,
    );
    let trend_pages = ContentPost::find()
        .filter(content_post::Column::PostState.eq(0))
        .order_by_desc(query_formula)
        .paginate(pg_con, 50);
    match trend_pages.fetch_page(0).await {
        Ok(t) => {
            let trend: Vec<Post> = t.iter().map(|r| r.into()).collect();
            match serde_json::to_string(&trend) {
                Ok(trending) => {
                    let _: Result<String, redis::RedisError> = redis::cmd("SETEX")
                        .arg("trending")
                        .arg(3600i32)
                        .arg(&trending)
                        .query_async(kv_conn)
                        .await;
                    Ok(trending)
                }
                Err(e) => Err(format!("{:?}", e)),
            }
        }
        Err(e) => {
            log::error!("[TRENDING] Database Error: {:?}", e);
            Err(format!("{:?}", e))
        }
    }
}
