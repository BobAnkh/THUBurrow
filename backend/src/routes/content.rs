use futures::future;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;

use sea_orm::sea_query::Expr;
use sea_orm::{entity::*, ActiveModelTrait, DbErr, QueryOrder};
use sea_orm::{PaginatorTrait, QueryFilter};

// use crate::db::user::Model;
use crate::pgdb;
use crate::pgdb::prelude::*;
use crate::pool::PgDb;
use crate::req::content::*;
use crate::utils::sso::SsoAuth;

use chrono::prelude::*;

static REPLY_PER_PAGE: usize = 20;

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
    auth: SsoAuth,
    db: Connection<PgDb>,
    post_info: Json<PostInfo>,
) -> (Status, Json<PostCreateResponse>) {
    let pg_con = db.into_inner();
    // create a response errors
    let mut errors: Vec<String> = Vec::new();
    // get content info from request
    let content = post_info.into_inner();
    // check if title, author and section is empty
    if content.title.is_empty() {
        errors.push("Empty Title".to_string());
    }
    if content.section.is_empty() {
        errors.push("Empty Section".to_string());
    }
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
            if burrow_info.burrow_status == 1 {
                errors.push("Burrow banned".to_string());
            }
            // check if the burrow_id is still valid
            if burrow_info.status == 2 {
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
            if user_state_info.banned == 1 {
                errors.push("User banned".to_string());
            }
        }
    };
    if !errors.is_empty() {
        (
            Status::BadRequest,
            Json(PostCreateResponse {
                errors,
                post_id: -1,
            }),
        )
    } else {
        // get timestamp
        let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        // get tag string
        let section = content.section.join(",");
        let tag = content.tag.join(",");
        // fill the row in content_subject
        let content_post = pgdb::content_post::ActiveModel {
            title: Set(content.title.to_string()),
            burrow_id: Set(content.burrow_id),
            create_time: Set(now.to_owned()),
            update_time: Set(now.to_owned()),
            section: Set(section),
            tag: Set(tag),
            ..Default::default()
        };
        // insert the row in database
        let res1 = match content_post.insert(&pg_con).await {
            Ok(res1) => res1,
            Err(e) => {
                errors.push(e.to_string());
                return (
                    Status::InternalServerError,
                    Json(PostCreateResponse {
                        errors,
                        post_id: -1,
                    }),
                );
            }
        };
        let post_id = res1.post_id.unwrap();
        log::info!("create post: {}", post_id);
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
        // insert the row into database
        let res2 = match content_reply.insert(&pg_con).await {
            Ok(res2) => res2,
            Err(e) => {
                errors.push(e.to_string());
                return (
                    Status::InternalServerError,
                    Json(PostCreateResponse {
                        errors,
                        post_id: -1,
                    }),
                );
            }
        };
        log::info!("add reply {}", res2.reply_id.unwrap());
        // return the response
        (Status::Ok, Json(PostCreateResponse { errors, post_id }))
    }
}

#[get("/post/<post_id>?<page>")]
pub async fn read_post(
    auth: SsoAuth,
    db: Connection<PgDb>,
    post_id: i64,
    page: usize,
) -> (Status, Json<PostReadResponse>) {
    let pg_con = db.into_inner();
    // check if the post not exsits, add corresponding error if so
    match ContentPost::find_by_id(post_id)
        .one(&pg_con)
        .await
        .expect("cannot fetch content from pgdb")
    {
        None => (
            Status::BadRequest,
            Json(PostReadResponse {
                errors: "Post not exsits".to_string(),
                post_page: None,
            }),
        ),
        Some(post_info) => {
            let reply_pages = ContentReply::find()
                .filter(pgdb::content_reply::Column::PostId.eq(post_id))
                .order_by_asc(pgdb::content_reply::Column::ReplyId)
                .paginate(&pg_con, REPLY_PER_PAGE);
            let reply_info = match reply_pages.fetch_page(page).await {
                Ok(reply_info) => reply_info,
                Err(e) => {
                    return (
                        Status::InternalServerError,
                        Json(PostReadResponse {
                            errors: format!("{}", e),
                            post_page: None,
                        }),
                    );
                }
            };
            // get post metadata
            let post_desc: Post = post_info.into();
            let reply_page: Vec<Reply> = reply_info.iter().map(|r| r.into()).collect();
            let collection;
            // check if the user collect the post, if so, update the state is_update
            let record = pgdb::user_collection::ActiveModel {
                uid: Set(auth.id),
                post_id: Set(post_id),
                is_update: Set(false),
            };
            match record.update(&pg_con).await {
                Ok(_) => {
                    collection = true;
                }
                Err(e) => match e {
                    DbErr::RecordNotFound(_) => {
                        collection = false;
                    }
                    _ => {
                        return (
                            Status::InternalServerError,
                            Json(PostReadResponse {
                                errors: format!("{}", e),
                                post_page: None,
                            }),
                        );
                    }
                },
            };
            // check if the user like the post
            let like = match UserLike::find_by_id((auth.id, post_id)).one(&pg_con).await {
                Ok(user_like) => user_like.is_some(),
                Err(e) => {
                    error!("[GET-BURROW] Database Error: {:?}", e.to_string());
                    false
                }
            };
            let post_page = PostPage {
                post_desc,
                reply_page,
                page,
                like,
                collection,
            };
            // return the response
            (
                Status::Ok,
                Json(PostReadResponse {
                    errors: "".to_string(),
                    post_page: Some(post_page),
                }),
            )
        }
    }
}

#[get("/post/list?<page>")]
pub async fn read_post_list(
    auth: SsoAuth,
    db: Connection<PgDb>,
    page: usize,
) -> (Status, Json<ListReadResponse>) {
    let pg_con = db.into_inner();
    let post_pages = ContentPost::find()
        .order_by_desc(pgdb::content_post::Column::PostId)
        .paginate(&pg_con, REPLY_PER_PAGE);
    let post_info = match post_pages.fetch_page(page).await {
        Ok(post_info) => post_info,
        Err(e) => {
            return (
                Status::InternalServerError,
                Json(ListReadResponse {
                    errors: format!("{}", e),
                    list_page: None,
                }),
            );
        }
    };
    // get total number of posts
    let post_num: i64;
    match ContentPost::find()
        .order_by_desc(pgdb::content_post::Column::PostId)
        .one(&pg_con)
        .await
        .expect("cannot fetch content from pgdb")
    {
        None => {
            return (
                Status::BadRequest,
                Json(ListReadResponse {
                    errors: "No post exsits".to_string(),
                    list_page: None,
                }),
            )
        }
        Some(post_last) => {
            post_num = post_last.post_id;
        }
    }
    // check if the user collect and like the posts
    // TODO: check if the post is banned?
    let post_page: Vec<PostDisplay> = future::try_join_all(post_info.iter().map(move |post| {
        let inner_conn = pg_con.clone();
        GetPostList::get_post_display(post, inner_conn, auth.id)
    }))
    .await
    .unwrap();
    let list_page = ListPage {
        post_page,
        page,
        post_num,
    };
    (
        Status::Ok,
        Json(ListReadResponse {
            errors: "".to_string(),
            list_page: Some(list_page),
        }),
    )
}

#[post("/reply", data = "<reply_info>", format = "json")]
pub async fn create_reply(
    auth: SsoAuth,
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
            if burrow_info.status == 1 {
                errors.push("Burrow banned".to_string());
            }
            // check if the burrow_id is still valid
            if burrow_info.status == 2 {
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
            if user_state_info.banned == 1 {
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
    auth: SsoAuth,
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
            if user_state_info.banned == 1 {
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
                                if burrow_info.status == 1 {
                                    errors.push("Burrow banned".to_string());
                                }
                                // check if the burrow_id is still valid
                                if burrow_info.status == 2 {
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
pub async fn delete_post(
    auth: SsoAuth,
    db: Connection<PgDb>,
    post_id: i64,
) -> (Status, Json<PostDeleteResponse>) {
    let pg_con = db.into_inner();
    let mut errors: Vec<String> = Vec::new();
    // check if the post not exsits, add corresponding error if so
    match ContentPost::find_by_id(post_id)
        .one(&pg_con)
        .await
        .expect("cannot fetch content from pgdb")
    {
        None => {
            errors.push("Post not exsits".to_string());
            (
                Status::BadRequest,
                Json(PostDeleteResponse {
                    errors,
                    post_id: -1,
                }),
            )
        }
        Some(post_info) => {
            // TODO: check if this user create the post
            // TODO: check if time is within limit, if so, allow user to delete
            // delete data in content_subject
            let delete_post: pgdb::content_post::ActiveModel = post_info.into();
            delete_post
                .delete(&pg_con)
                .await
                .expect("cannot delete content from content_subject");
            // delete data in content_reply
            ContentReply::delete_many()
                .filter(pgdb::content_reply::Column::PostId.eq(post_id))
                .exec(&pg_con)
                .await
                .expect("cannot delete content from content_reply");
            // return the response
            (Status::Ok, Json(PostDeleteResponse { errors, post_id }))
        }
    }
}
