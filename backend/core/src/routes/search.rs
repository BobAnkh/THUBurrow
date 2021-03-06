//! Routes for search

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
use sea_orm::{entity::*, ColumnTrait, DbErr, PaginatorTrait, QueryFilter, QueryOrder};

use crate::config::content::REPLY_PER_PAGE;
use crate::db::{self, prelude::*};
use crate::models::{burrow::*, content::*, error::*, search::*};
use crate::pool::{PgDb, Search, TypesenseSearch};
use crate::utils::auth::Auth;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/", routes![search,])
}

/// Search
///
/// ## Parameters
///
/// - `Auth`: Authenticated User
/// - `Connection<PgDb>`: Postgres connection
/// - `Connection<TypesenseSearch>`: Search engine connection
/// - `Json<SearchRequest>`: Search request struct in Json
/// - `Option<usize>`: Result page
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `String`: Json string of search result
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///     - `ErrorCode::DatabaseErr`
///     - `ErrorCode::BurrowNotExist`
///     - `ErrorCode::PostNotExist`
///     - `ErrorCode::EmptyField`
///
#[post("/search?<page>", data = "<data>", format = "json")]
async fn search(
    auth: Auth,
    db: Connection<PgDb>,
    conn: Connection<TypesenseSearch>,
    data: Json<SearchRequest>,
    page: Option<usize>,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    let page = page.unwrap_or(0);
    let pg_con = db.into_inner();
    let client = conn.into_inner();
    match data.into_inner() {
        SearchRequest::RetrieveBurrow { burrow_id } => {
            match db::burrow::Entity::find_by_id(burrow_id).one(&pg_con).await {
                Ok(opt_burrow) => match opt_burrow {
                    Some(burrow) => {
                        match db::content_post::Entity::find()
                            .filter(db::content_post::Column::BurrowId.eq(burrow_id))
                            .order_by_desc(db::content_post::Column::PostId)
                            .paginate(&pg_con, REPLY_PER_PAGE)
                            .fetch_page(page)
                            .await
                        {
                            Ok(posts) => (
                                Status::Ok,
                                Ok(serde_json::to_string(&BurrowShowResponse {
                                    title: burrow.title,
                                    description: burrow.description,
                                    posts: {
                                        let posts_info: Vec<Post> =
                                            posts.iter().map(|post| post.into()).collect();
                                        posts_info
                                    },
                                })
                                .unwrap()),
                            ),
                            Err(e) => {
                                error!("[SEARCH-BURROW] Database Error: {:?}", e);
                                (
                                    Status::InternalServerError,
                                    Err(Json(ErrorResponse::default())),
                                )
                            }
                        }
                    }
                    None => (
                        Status::NotFound,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::BurrowNotExist,
                            format!("Cannot find burrow {}", burrow_id),
                        ))),
                    ),
                },
                Err(e) => {
                    error!("[SEARCH-BURROW] Database Error: {:?}", e);
                    (
                        Status::InternalServerError,
                        Err(Json(ErrorResponse::default())),
                    )
                }
            }
        }
        SearchRequest::RetrievePost { post_id } => {
            match ContentPost::find_by_id(post_id).one(&pg_con).await {
                Ok(r) => match r {
                    None => (
                        Status::NotFound,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::PostNotExist,
                            format!("Cannot find post {}", post_id),
                        ))),
                    ),
                    Some(post_info) => {
                        let reply_pages = ContentReply::find()
                            .filter(db::content_reply::Column::PostId.eq(post_id))
                            .order_by_asc(db::content_reply::Column::ReplyId)
                            .paginate(&pg_con, REPLY_PER_PAGE);
                        let reply_info = match reply_pages.fetch_page(page).await {
                            Ok(reply_info) => reply_info,
                            Err(e) => {
                                log::error!("[READ-POST] Database error: {:?}", e);
                                return (
                                    Status::InternalServerError,
                                    Err(Json(ErrorResponse::default())),
                                );
                            }
                        };
                        // get post metadata
                        let post_desc: Post = post_info.into();
                        let reply_page: Vec<Reply> = reply_info.iter().map(|r| r.into()).collect();
                        // check if the user collect the post, if so, update the state is_update
                        let record = db::user_collection::ActiveModel {
                            uid: Set(auth.id),
                            post_id: Set(post_id),
                            is_update: Set(false),
                        };
                        let collection = match record.update(&pg_con).await {
                            Ok(_) => true,
                            Err(e) => match e {
                                DbErr::RecordNotFound(_) => false,
                                _ => {
                                    log::error!("[READ-POST] Database error: {:?}", e);
                                    return (
                                        Status::InternalServerError,
                                        Err(Json(ErrorResponse::default())),
                                    );
                                }
                            },
                        };
                        // check if the user like the post
                        let like = match UserLike::find_by_id((auth.id, post_id)).one(&pg_con).await
                        {
                            Ok(user_like) => user_like.is_some(),
                            Err(e) => {
                                error!("[READ-POST] Database Error: {:?}", e.to_string());
                                false
                            }
                        };
                        // return the response
                        (
                            Status::Ok,
                            Ok(serde_json::to_string(&PostPage {
                                post_desc,
                                reply_page,
                                page,
                                like,
                                collection,
                            })
                            .unwrap()),
                        )
                    }
                },
                Err(e) => {
                    log::error!("[READ-POST] Database error: {:?}", e);
                    (
                        Status::InternalServerError,
                        Err(Json(ErrorResponse::default())),
                    )
                }
            }
        }
        SearchRequest::SearchBurrowKeyword { keywords } => {
            let page = page + 1;
            let keyword = keywords.join(" ");
            let uri = format!("/collections/burrows/documents/search?q={}&query_by=title,description&prefix=false&sort_by=_text_match:desc,burrow_id:desc&page={}&per_page=20&highlight_fields=title,description",keyword,page);
            let response = match client.build_get(&uri).send().await {
                Ok(r) => match r.json::<SearchBurrowData>().await {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("[SEARCH-BURROW] Database error: {:?}", e);
                        return (
                            Status::InternalServerError,
                            Err(Json(ErrorResponse::default())),
                        );
                    }
                },
                Err(e) => {
                    log::error!("[SEARCH-BURROW] Database error: {:?}", e);
                    return (
                        Status::InternalServerError,
                        Err(Json(ErrorResponse::default())),
                    );
                }
            };
            let response: SearchBurrowResponse = response.into();
            match serde_json::to_string(&response) {
                Ok(r) => (Status::Ok, Ok(r)),
                Err(e) => {
                    log::error!("[SEARCH-BURROW] Database error: {:?}", e);
                    (
                        Status::InternalServerError,
                        Err(Json(ErrorResponse::default())),
                    )
                }
            }
        }
        SearchRequest::SearchPostKeyword { keywords } => {
            let page = page + 1;
            let keyword = keywords.join(" ");
            let search_post = SearchParam {
                collection: "posts".to_string(),
                q: keyword.to_owned(),
                query_by: "title".to_string(),
                filter_by: "".to_string(),
                sort_by: "_text_match:desc,post_id:desc".to_string(),
                group_by: "".to_string(),
                highlight_fields: "title".to_string(),
            };
            let search_reply = SearchParam {
                collection: "replies".to_string(),
                q: keyword,
                query_by: "content".to_string(),
                filter_by: "".to_string(),
                sort_by: "_text_match:desc,reply_id:asc".to_string(),
                group_by: "post_id".to_string(),
                highlight_fields: "content".to_string(),
            };
            let multi_search = MultiSearch {
                searches: vec![search_post, search_reply],
            };
            let uri = format!("/multi_search?prefix=false&page={}&per_page=15", page);
            let response = match client.build_post(&uri).json(&multi_search).send().await {
                Ok(r) => match r.json::<SearchMixResult>().await {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("[SEARCH-MIX] Database error: {:?}", e);
                        return (
                            Status::InternalServerError,
                            Err(Json(ErrorResponse::default())),
                        );
                    }
                },
                Err(e) => {
                    log::error!("[SEARCH-MIX] Database error: {:?}", e);
                    return (
                        Status::InternalServerError,
                        Err(Json(ErrorResponse::default())),
                    );
                }
            };
            let response: SearchMixResponse = response.into();
            match serde_json::to_string(&response) {
                Ok(r) => (Status::Ok, Ok(r)),
                Err(e) => {
                    log::error!("[SEARCH-MIX] Database error: {:?}", e);
                    (
                        Status::InternalServerError,
                        Err(Json(ErrorResponse::default())),
                    )
                }
            }
        }
        SearchRequest::SearchPostTag { tag } => {
            let page = page + 1;
            if tag.is_empty() {
                return (
                    Status::BadRequest,
                    Err(Json(ErrorResponse::build(
                        ErrorCode::EmptyField,
                        "Tags should not be empty",
                    ))),
                );
            }
            let tags = serde_json::to_string(&tag).unwrap();
            let uri = format!(
                "/collections/posts/documents/search?q=*&query_by=title&filter_by=tag:={}&sort_by=post_id:desc&page={}&per_page=20",
                tags,page
            );
            let response = match client.build_get(&uri).send().await {
                Ok(r) => match r.json::<SearchPostData>().await {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("[SEARCH-BURROW] Database error: {:?}", e);
                        return (
                            Status::InternalServerError,
                            Err(Json(ErrorResponse::default())),
                        );
                    }
                },
                Err(e) => {
                    log::error!("[SEARCH-BURROW] Database error: {:?}", e);
                    return (
                        Status::InternalServerError,
                        Err(Json(ErrorResponse::default())),
                    );
                }
            };
            let response: SearchPostResponse = response.into();
            match serde_json::to_string(&response) {
                Ok(r) => (Status::Ok, Ok(r)),
                Err(e) => {
                    log::error!("[SEARCH-BURROW] Database error: {:?}", e);
                    (
                        Status::InternalServerError,
                        Err(Json(ErrorResponse::default())),
                    )
                }
            }
        }
    }
}
