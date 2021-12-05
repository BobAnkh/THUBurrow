use std::collections::HashMap;

use futures::future;
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
) -> (Status, Json<PostCreateResponse>) {
    let pg_con = db.into_inner();
    // get content info from request
    let content = post_info.into_inner();
    // check if title, author and section is empty
    if content.title.is_empty() {
        return (
            Status::BadRequest,
            Json(PostCreateResponse {
                errors: "Empty Title".to_string(),
                post_id: -1,
            }),
        );
    }
    if content.section.is_empty() {
        return (
            Status::BadRequest,
            Json(PostCreateResponse {
                errors: "Empty Section".to_string(),
                post_id: -1,
            }),
        );
    }
    // TODO: split section and check if it is valid
    // check if user has been banned
    match UserStatus::find_by_id(auth.id).one(&pg_con).await {
        Ok(ust) => match ust {
            None => (
                Status::BadRequest,
                Json(PostCreateResponse {
                    errors: "User not exists".to_string(),
                    post_id: -1,
                }),
            ),
            Some(user_state_info) => {
                if user_state_info.user_state != 0 {
                    (
                        Status::Forbidden,
                        Json(PostCreateResponse {
                            errors: "User invalid".to_string(),
                            post_id: -1,
                        }),
                    )
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
                        Ok(post_id) => (
                            Status::Ok,
                            Json(PostCreateResponse {
                                errors: String::new(),
                                post_id,
                            }),
                        ),
                        Err(e) => {
                            log::error!("[CREATE-POST] Database error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Json(PostCreateResponse {
                                    errors: String::new(),
                                    post_id: -1,
                                }),
                            )
                        }
                    }
                } else {
                    (
                        Status::Forbidden,
                        Json(PostCreateResponse {
                            errors: "Burrow invalid".to_string(),
                            post_id: -1,
                        }),
                    )
                }
            }
        },
        Err(e) => {
            log::error!("[CREATE-POST] Database error: {:?}", e);
            (
                Status::InternalServerError,
                Json(PostCreateResponse {
                    errors: String::new(),
                    post_id: -1,
                }),
            )
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
) -> (Status, Json<ReplyCreateResponse>) {
    let pg_con = db.into_inner();
    let mut errors: Vec<String> = Vec::new();
    // get content info from request
    let content = reply_info.into_inner();
    match Burrow::find_by_id(content.burrow_id)
        .one(&pg_con)
        .await
        .expect("cannot fetch content from pgdb")
    {
        None => {
            errors.push("Burrow not exsits".to_string());
        }
        Some(burrow_info) => {
            // check if this burrow_id belongs to the user
            if burrow_info.uid != auth.id {
                errors.push("Wrong user".to_string());
            }
            // check if burrow has been banned
            if burrow_info.burrow_state == 1 {
                errors.push("Burrow banned".to_string());
            }
            // check if the burrow_id is still valid
            if burrow_info.burrow_state == 2 {
                errors.push("Burrow discarded".to_string());
            }
        }
    };
    // check if user has been banned
    match UserStatus::find_by_id(auth.id)
        .one(&pg_con)
        .await
        .expect("cannot fetch content from pgdb")
    {
        None => {
            errors.push("User not exsits".to_string());
        }
        Some(user_state_info) => {
            if user_state_info.user_state == 1 {
                errors.push("User banned".to_string());
            }
        }
    };
    // if error exists, refuse to create reply
    if !errors.is_empty() {
        (
            Status::BadRequest,
            Json(ReplyCreateResponse {
                errors,
                reply_id: -1,
                post_id: -1,
            }),
        )
    } else {
        match ContentPost::find_by_id(content.post_id)
            .one(&pg_con)
            .await
            .expect("cannot fetch content from pgdb")
        {
            None => {
                errors.push("Post not exsits".to_string());
                (
                    Status::BadRequest,
                    Json(ReplyCreateResponse {
                        errors,
                        reply_id: -1,
                        post_id: -1,
                    }),
                )
            }
            Some(post_info) => {
                // get timestamp
                let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
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
                let res1 = match content_reply.insert(&pg_con).await {
                    Ok(res1) => res1,
                    Err(e) => {
                        errors.push(e.to_string());
                        return (
                            Status::InternalServerError,
                            Json(ReplyCreateResponse {
                                errors,
                                post_id: -1,
                                reply_id: -1,
                            }),
                        );
                    }
                };
                log::info!("create reply {}", res1.reply_id.unwrap());
                // modify the time and the post_len in content_subject
                let post_update = pgdb::content_post::ActiveModel {
                    post_id: Set(post_info.post_id),
                    update_time: Set(now),
                    post_len: Set(post_info.post_len + 1),
                    ..Default::default()
                };
                // update the row in database
                let res2 = match post_update.update(&pg_con).await {
                    Ok(res2) => res2,
                    Err(e) => {
                        errors.push(e.to_string());
                        return (
                            Status::InternalServerError,
                            Json(ReplyCreateResponse {
                                errors,
                                post_id: -1,
                                reply_id: -1,
                            }),
                        );
                    }
                };
                log::info!("update post {}", res2.post_id.unwrap());
                // update the user_collection state
                if let Err(e) = UserCollection::update_many()
                    .col_expr(pgdb::user_collection::Column::IsUpdate, Expr::value(false))
                    .filter(pgdb::user_collection::Column::PostId.eq(post_info.post_id))
                    .exec(&pg_con)
                    .await
                {
                    match e {
                        DbErr::RecordNotFound(_) => {}
                        _ => {
                            errors.push(e.to_string());
                            return (
                                Status::InternalServerError,
                                Json(ReplyCreateResponse {
                                    errors,
                                    post_id: -1,
                                    reply_id: -1,
                                }),
                            );
                        }
                    }
                };
                // return the response
                (
                    Status::Ok,
                    Json(ReplyCreateResponse {
                        errors,
                        post_id: post_info.post_id,
                        reply_id: post_info.post_len,
                    }),
                )
            }
        }
    }
}

#[post("/reply/update", data = "<reply_update_info>", format = "json")]
pub async fn update_reply(
    auth: Auth,
    db: Connection<PgDb>,
    reply_update_info: Json<ReplyUpdateInfo>,
) -> (Status, Json<ReplyCreateResponse>) {
    let pg_con = db.into_inner();
    let mut errors: Vec<String> = Vec::new();
    // get content info from request
    let content = reply_update_info.into_inner();
    // check if user has been banned, add corresponding error if so
    match UserStatus::find_by_id(auth.id)
        .one(&pg_con)
        .await
        .expect("cannot fetch content from pgdb")
    {
        None => {
            errors.push("User not exsits".to_string());
        }
        Some(user_state_info) => {
            if user_state_info.user_state == 1 {
                errors.push("User banned".to_string());
            }
        }
    };
    // if error exists, refuse to create reply
    if !errors.is_empty() {
        (
            Status::BadRequest,
            Json(ReplyCreateResponse {
                errors,
                reply_id: -1,
                post_id: -1,
            }),
        )
    } else {
        match ContentPost::find_by_id(content.post_id)
            .one(&pg_con)
            .await
            .expect("cannot fetch content from pgdb")
        {
            None => {
                errors.push("Post not exsits".to_string());
                (
                    Status::BadRequest,
                    Json(ReplyCreateResponse {
                        errors,
                        reply_id: -1,
                        post_id: -1,
                    }),
                )
            }
            Some(post_info) => {
                match ContentReply::find_by_id((content.post_id, content.reply_id))
                    .one(&pg_con)
                    .await
                    .expect("cannot fetch content from pgdb")
                {
                    None => {
                        errors.push("Reply not exsits".to_string());
                        return (
                            Status::BadRequest,
                            Json(ReplyCreateResponse {
                                errors,
                                reply_id: -1,
                                post_id: -1,
                            }),
                        );
                    }
                    Some(reply) => {
                        // check if this burrow_id belongs to the user
                        match Burrow::find_by_id(reply.burrow_id)
                            .one(&pg_con)
                            .await
                            .expect("cannot fetch content from pgdb")
                        {
                            None => {
                                errors.push("Burrow not exsits".to_string());
                            }
                            Some(burrow_info) => {
                                if burrow_info.uid != auth.id {
                                    errors.push("Wrong user".to_string());
                                }
                                // check if burrow has been banned
                                if burrow_info.burrow_state == 1 {
                                    errors.push("Burrow banned".to_string());
                                }
                                // check if the burrow_id is still valid
                                if burrow_info.burrow_state == 2 {
                                    errors.push("Burrow discarded".to_string());
                                }
                            }
                        };
                        // get timestamp
                        let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
                        let mut reply: pgdb::content_reply::ActiveModel = reply.into();
                        reply.content = Set(content.content.to_owned());
                        reply.update_time = Set(now.to_owned());
                        let res1 = match reply.update(&pg_con).await {
                            Ok(res1) => res1,
                            Err(e) => {
                                errors.push(e.to_string());
                                return (
                                    Status::InternalServerError,
                                    Json(ReplyCreateResponse {
                                        errors,
                                        post_id: -1,
                                        reply_id: -1,
                                    }),
                                );
                            }
                        };
                        log::info!("update reply {}", res1.reply_id.unwrap());
                    }
                }
                // update the user_collection state
                if let Err(e) = UserCollection::update_many()
                    .col_expr(pgdb::user_collection::Column::IsUpdate, Expr::value(false))
                    .filter(pgdb::user_collection::Column::PostId.eq(post_info.post_id))
                    .exec(&pg_con)
                    .await
                {
                    match e {
                        DbErr::RecordNotFound(_) => {}
                        _ => {
                            errors.push(e.to_string());
                            return (
                                Status::InternalServerError,
                                Json(ReplyCreateResponse {
                                    errors,
                                    post_id: -1,
                                    reply_id: -1,
                                }),
                            );
                        }
                    }
                };
                // return the response
                (
                    Status::Ok,
                    Json(ReplyCreateResponse {
                        errors,
                        post_id: post_info.post_id,
                        reply_id: post_info.post_len,
                    }),
                )
            }
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
