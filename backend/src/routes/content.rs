use std::collections::HashMap;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;

use sea_orm::sea_query::Expr;
use sea_orm::{entity::*, ActiveModelTrait, Condition, ConnectionTrait, DbErr, QueryOrder};
use sea_orm::{PaginatorTrait, QueryFilter};

// use crate::db::user::Model;
use crate::pgdb;
use crate::pgdb::prelude::*;
use crate::pool::PgDb;
use crate::req::content::*;
use crate::utils::auth::Auth;
use crate::utils::burrow_valid::is_valid_burrow;

use chrono::{prelude::*, Duration};

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/content",
        routes![
            create_post,
            read_post,
            read_post_list,
            create_reply,
            update_reply,
            delete_post,
        ],
    )
}

#[post("/post", data = "<post_info>", format = "json")]
pub async fn create_post(
    auth: Auth,
    db: Connection<PgDb>,
    post_info: Json<PostInfo>,
) -> (Status, Result<Json<PostCreateResponse>, String>) {
    let pg_con = db.into_inner();
    // get content info from request
    let content = post_info.into_inner();
    // check if title, author and section is empty
    if content.title.is_empty() {
        return (Status::BadRequest, Err("Empty Title".to_string()));
    }
    if content.section.is_empty() {
        return (Status::BadRequest, Err("Empty Section".to_string()));
    }
    // TODO: check if section is valid
    // check if user has been banned
    match UserStatus::find_by_id(auth.id).one(&pg_con).await {
        Ok(ust) => match ust {
            None => (Status::BadRequest, Err("User not exists".to_string())),
            Some(user_state_info) => {
                if user_state_info.user_state != 0 {
                    (Status::Forbidden, Err("User invalid".to_string()))
                } else if is_valid_burrow(&user_state_info.valid_burrow, &content.burrow_id) {
                    match pg_con
                        .transaction::<_, i64, DbErr>(|txn| {
                            Box::pin(async move {
                                // get timestamp
                                let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
                                // get tag string
                                let section = content.section.join(",");
                                let tag = content.tag.join(",");
                                let content_post = pgdb::content_post::ActiveModel {
                                    title: Set(content.title),
                                    burrow_id: Set(content.burrow_id),
                                    create_time: Set(now.to_owned()),
                                    update_time: Set(now.to_owned()),
                                    section: Set(section),
                                    tag: Set(tag),
                                    ..Default::default()
                                };
                                // insert the row in database
                                let post_res = content_post.insert(txn).await?;
                                let post_id = post_res.post_id.unwrap();
                                log::info!("[CREATE-POST] create post: {}", post_id);
                                // fill the row in content_reply
                                let content_reply = pgdb::content_reply::ActiveModel {
                                    post_id: Set(post_id),
                                    reply_id: Set(0),
                                    burrow_id: Set(content.burrow_id),
                                    create_time: Set(now.to_owned()),
                                    update_time: Set(now),
                                    content: Set(content.content.to_string()),
                                    ..Default::default()
                                };
                                let reply_res = content_reply.insert(txn).await?;
                                log::info!(
                                    "[CREATE-POST] add reply {}",
                                    reply_res.reply_id.unwrap()
                                );
                                let update_res = Burrow::update_many()
                                    .col_expr(
                                        pgdb::burrow::Column::PostNum,
                                        Expr::col(pgdb::burrow::Column::PostNum).add(1),
                                    )
                                    .filter(pgdb::burrow::Column::BurrowId.eq(content.burrow_id))
                                    .exec(txn)
                                    .await?;
                                if update_res.rows_affected != 1 {
                                    return Err(DbErr::RecordNotFound(
                                        "burrow not found".to_string(),
                                    ));
                                }
                                Ok(post_id)
                            })
                        })
                        .await
                    {
                        Ok(post_id) => (Status::Ok, Ok(Json(PostCreateResponse { post_id }))),
                        Err(e) => {
                            log::error!("[CREATE-POST] Database error: {:?}", e);
                            (Status::InternalServerError, Err(String::new()))
                        }
                    }
                } else {
                    (Status::Forbidden, Err("Burrow invalid".to_string()))
                }
            }
        },
        Err(e) => {
            log::error!("[CREATE-POST] Database error: {:?}", e);
            (Status::InternalServerError, Err(String::new()))
        }
    }
}

#[get("/post/<post_id>?<page>")]
pub async fn read_post(
    auth: Auth,
    db: Connection<PgDb>,
    post_id: i64,
    page: Option<usize>,
) -> (Status, Result<Json<PostPage>, String>) {
    let pg_con = db.into_inner();
    let page = page.unwrap_or(0);
    // check if the post not exsits, add corresponding error if so
    match ContentPost::find_by_id(post_id).one(&pg_con).await {
        Ok(r) => match r {
            None => (Status::BadRequest, Err("Post not exists".to_string())),
            Some(post_info) => {
                let reply_pages = ContentReply::find()
                    .filter(pgdb::content_reply::Column::PostId.eq(post_id))
                    .order_by_asc(pgdb::content_reply::Column::ReplyId)
                    .paginate(&pg_con, REPLY_PER_PAGE);
                let reply_info = match reply_pages.fetch_page(page).await {
                    Ok(reply_info) => reply_info,
                    Err(e) => {
                        log::error!("[READ-POST] Database error: {:?}", e);
                        return (Status::InternalServerError, Err(String::new()));
                    }
                };
                // get post metadata
                let post_desc: Post = post_info.into();
                let reply_page: Vec<Reply> = reply_info.iter().map(|r| r.into()).collect();
                // check if the user collect the post, if so, update the state is_update
                let record = pgdb::user_collection::ActiveModel {
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
                            return (Status::InternalServerError, Err(String::new()));
                        }
                    },
                };
                // check if the user like the post
                let like = match UserLike::find_by_id((auth.id, post_id)).one(&pg_con).await {
                    Ok(user_like) => user_like.is_some(),
                    Err(e) => {
                        error!("[READ-POST] Database Error: {:?}", e.to_string());
                        false
                    }
                };
                // return the response
                (
                    Status::Ok,
                    Ok(Json(PostPage {
                        post_desc,
                        reply_page,
                        page,
                        like,
                        collection,
                    })),
                )
            }
        },
        Err(e) => {
            log::error!("[READ-POST] Database error: {:?}", e);
            (Status::InternalServerError, Err(String::new()))
        }
    }
}

#[get("/post/list?<page>")]
pub async fn read_post_list(
    auth: Auth,
    db: Connection<PgDb>,
    page: Option<usize>,
) -> (Status, Result<Json<ListPage>, String>) {
    let pg_con = db.into_inner();
    let page = page.unwrap_or(0);
    let post_pages = ContentPost::find()
        .order_by_desc(pgdb::content_post::Column::PostId)
        .paginate(&pg_con, POST_PER_PAGE);
    let post_info = match post_pages.fetch_page(page).await {
        Ok(post_info) => post_info,
        Err(e) => {
            log::error!("[READ-POST] Database error: {:?}", e);
            return (Status::InternalServerError, Err(String::new()));
        }
    };
    // check if the user collect and like the posts
    // TODO: check if the post is banned?
    let post_ids = post_info.iter().map(|r| r.post_id).collect::<Vec<i64>>();
    let like_res = match pgdb::user_like::Entity::find()
        .filter(
            Condition::all()
                .add(pgdb::user_like::Column::Uid.eq(auth.id))
                .add(pgdb::user_like::Column::PostId.is_in(post_ids.clone())),
        )
        .order_by_desc(pgdb::user_like::Column::PostId)
        .all(&pg_con)
        .await
        .map(|user_likes| {
            let hm: HashMap<i64, bool> = user_likes.iter().map(|r| (r.post_id, true)).collect();
            hm
        }) {
        Ok(hm) => hm,
        Err(e) => {
            log::error!("[READ-POST-LIST] Database Error: {:?}", e.to_string());
            return (Status::InternalServerError, Err(String::new()));
        }
    };
    let collection_res = match pgdb::user_collection::Entity::find()
        .filter(
            Condition::all()
                .add(pgdb::user_collection::Column::Uid.eq(auth.id))
                .add(pgdb::user_collection::Column::PostId.is_in(post_ids)),
        )
        .order_by_desc(pgdb::user_collection::Column::PostId)
        .all(&pg_con)
        .await
        .map(|user_collections| {
            let hm: HashMap<i64, (bool, bool)> = user_collections
                .iter()
                .map(|r| (r.post_id, (true, r.is_update)))
                .collect();
            hm
        }) {
        Ok(hm) => hm,
        Err(e) => {
            log::error!("[READ-POST-LIST] Database Error: {:?}", e.to_string());
            return (Status::InternalServerError, Err(String::new()));
        }
    };
    let post_page: Vec<PostDisplay> = post_info
        .iter()
        .map(|m| {
            let c = *collection_res.get(&m.post_id).unwrap_or(&(false, false));
            PostDisplay {
                post: m.into(),
                like: *like_res.get(&m.post_id).unwrap_or(&false),
                collection: c.0,
                is_update: c.1,
            }
        })
        .collect();
    (Status::Ok, Ok(Json(ListPage { post_page, page })))
}

#[post("/reply", data = "<reply_info>", format = "json")]
pub async fn create_reply(
    auth: Auth,
    db: Connection<PgDb>,
    reply_info: Json<ReplyInfo>,
) -> (Status, Result<Json<ReplyCreateResponse>, String>) {
    let pg_con = db.into_inner();
    // get content info from request
    let content = reply_info.into_inner();
    match UserStatus::find_by_id(auth.id).one(&pg_con).await {
        Ok(ust) => match ust {
            None => (Status::BadRequest, Err("User not exists".to_string())),
            Some(user_state_info) => {
                if user_state_info.user_state != 0 {
                    (Status::Forbidden, Err("User invalid".to_string()))
                } else if is_valid_burrow(&user_state_info.valid_burrow, &content.burrow_id) {
                    match ContentPost::find_by_id(content.post_id).one(&pg_con).await {
                        Ok(r) => match r {
                            None => (Status::Forbidden, Err("Post not exists".to_string())),
                            Some(post_info) => {
                                let post_id = post_info.post_id;
                                match pg_con
                                    .transaction::<_, i32, DbErr>(|txn| {
                                        Box::pin(async move {
                                            // get timestamp
                                            let now = Utc::now()
                                                .with_timezone(&FixedOffset::east(8 * 3600));
                                            // fill the row in content_reply
                                            let content_reply = pgdb::content_reply::ActiveModel {
                                                post_id: Set(post_info.post_id),
                                                reply_id: Set(post_info.post_len),
                                                burrow_id: Set(content.burrow_id),
                                                create_time: Set(now.to_owned()),
                                                update_time: Set(now.to_owned()),
                                                content: Set(content.content.to_string()),
                                                ..Default::default()
                                            };
                                            // insert the row in database
                                            let reply_res = content_reply.insert(txn).await?;
                                            let reply_id = reply_res.reply_id.unwrap();
                                            log::info!("[CREATE-REPLY] create reply {}", reply_id);
                                            let post_update = pgdb::content_post::ActiveModel {
                                                post_id: Set(post_info.post_id),
                                                update_time: Set(now),
                                                post_len: Set(post_info.post_len + 1),
                                                ..Default::default()
                                            };
                                            // update the row in database
                                            let post_res = post_update.update(txn).await?;
                                            log::info!(
                                                "[CREATE-REPLY] update post {}",
                                                post_res.post_id.unwrap()
                                            );
                                            UserCollection::update_many()
                                                .col_expr(
                                                    pgdb::user_collection::Column::IsUpdate,
                                                    Expr::value(true),
                                                )
                                                .filter(
                                                    pgdb::user_collection::Column::PostId
                                                        .eq(post_info.post_id),
                                                )
                                                .exec(txn)
                                                .await?;
                                            Ok(reply_id)
                                        })
                                    })
                                    .await
                                {
                                    Ok(reply_id) => (
                                        Status::Ok,
                                        Ok(Json(ReplyCreateResponse { post_id, reply_id })),
                                    ),
                                    Err(e) => {
                                        log::error!("[CREATE-POST] Database error: {:?}", e);
                                        (Status::InternalServerError, Err(String::new()))
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("[CREATE-POST] Database error: {:?}", e);
                            (Status::InternalServerError, Err(String::new()))
                        }
                    }
                } else {
                    (Status::Forbidden, Err("Burrow invalid".to_string()))
                }
            }
        },
        Err(e) => {
            log::error!("[CREATE-POST] Database error: {:?}", e);
            (Status::InternalServerError, Err(String::new()))
        }
    }
}

#[put("/reply", data = "<reply_update_info>", format = "json")]
pub async fn update_reply(
    auth: Auth,
    db: Connection<PgDb>,
    reply_update_info: Json<ReplyUpdateInfo>,
) -> (Status, String) {
    let pg_con = db.into_inner();
    // get content info from request
    let content = reply_update_info.into_inner();
    match UserStatus::find_by_id(auth.id).one(&pg_con).await {
        Ok(ust) => match ust {
            None => (Status::BadRequest, "User not exists".to_string()),
            Some(user_state_info) => {
                if user_state_info.user_state != 0 {
                    (Status::Forbidden, "User invalid".to_string())
                } else {
                    // if is_valid_burrow(&user_state_info.valid_burrow, &content.burrow_id)
                    match ContentReply::find_by_id((content.post_id, content.reply_id))
                        .one(&pg_con)
                        .await
                    {
                        Ok(r) => match r {
                            None => (Status::Forbidden, "Reply not exists".to_string()),
                            Some(reply_info) => {
                                if is_valid_burrow(
                                    &user_state_info.valid_burrow,
                                    &reply_info.burrow_id,
                                ) {
                                    let now =
                                        Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
                                    // fill the row in content_reply
                                    let mut content_reply: pgdb::content_reply::ActiveModel =
                                        reply_info.into();
                                    content_reply.content = Set(content.content);
                                    content_reply.update_time = Set(now);
                                    match content_reply.update(&pg_con).await {
                                        Ok(_) => (Status::Ok, "Success".to_string()),
                                        Err(e) => {
                                            log::error!("[CREATE-POST] Database error: {:?}", e);
                                            (Status::InternalServerError, String::new())
                                        }
                                    }
                                } else {
                                    (Status::Forbidden, "Burrow invalid".to_string())
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("[CREATE-POST] Database error: {:?}", e);
                            (Status::InternalServerError, String::new())
                        }
                    }
                }
            }
        },
        Err(e) => {
            log::error!("[CREATE-POST] Database error: {:?}", e);
            (Status::InternalServerError, String::new())
        }
    }
}

#[delete("/post/<post_id>")]
pub async fn delete_post(auth: Auth, db: Connection<PgDb>, post_id: i64) -> (Status, String) {
    let pg_con = db.into_inner();
    let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
    // check if the post not exsits, add corresponding error if so
    match ContentPost::find_by_id(post_id).one(&pg_con).await {
        Ok(r) => match r {
            None => (Status::BadRequest, "Post not exsits".to_string()),
            Some(post_info) => {
                // TODO: check if this user create the post
                // TODO: check if time is within limit, if so, allow user to delete
                if post_info
                    .create_time
                    .checked_add_signed(Duration::seconds(135))
                    .unwrap()
                    < now
                {
                    return (
                        Status::Forbidden,
                        "Can only delete post in 2 minutes".to_string(),
                    );
                }
                match pgdb::user_status::Entity::find_by_id(auth.id)
                    .one(&pg_con)
                    .await
                {
                    Ok(opt_state) => match opt_state {
                        Some(state) => {
                            if is_valid_burrow(&state.valid_burrow, &post_info.burrow_id) {
                                // delete data in content_subject
                                let delete_post: pgdb::content_post::ActiveModel = post_info.into();
                                match pg_con
                                    .transaction::<_, (), DbErr>(|txn| {
                                        Box::pin(async move {
                                            delete_post.delete(txn).await?;
                                            ContentReply::delete_many()
                                                .filter(
                                                    pgdb::content_reply::Column::PostId.eq(post_id),
                                                )
                                                .exec(txn)
                                                .await?;
                                            Ok(())
                                        })
                                    })
                                    .await
                                {
                                    Ok(_) => (Status::Ok, "Success".to_string()),
                                    Err(e) => {
                                        log::error!("[DELETE-POST] Database error: {:?}", e);
                                        (Status::InternalServerError, String::new())
                                    }
                                }
                            } else {
                                (
                                    Status::Forbidden,
                                    "Not allowed to delete this post".to_string(),
                                )
                            }
                        }
                        None => {
                            log::info!("[DELETE-POST] Cannot find user_status by uid.");
                            (Status::Forbidden, "".to_string())
                        }
                    },
                    Err(e) => {
                        log::error!("[DELETE-POST] Database error: {:?}", e);
                        (Status::InternalServerError, String::new())
                    }
                }
            }
        },
        Err(e) => {
            log::error!("[DELETE-POST] Database error: {:?}", e);
            (Status::InternalServerError, String::new())
        }
    }
}
