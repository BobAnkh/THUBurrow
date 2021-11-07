use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;

use sea_orm::QueryFilter;
use sea_orm::{entity::*, ActiveModelTrait, Condition, QueryOrder};

// use crate::db::user::Model;
use crate::pgdb;
use crate::pgdb::prelude::*;
use crate::pool::PgDb;
use crate::req::content::*;

use chrono::prelude::*;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/content",
        routes![
            content_check,
            content_post,
            content_read,
            content_reply,
            content_delete_post,
            content_delete_reply,
        ],
    )
}

#[get("/test")]
async fn content_check() -> String {
    "Ok".to_string()
}

#[post("/post", data = "<content_info>", format = "json")]
pub async fn content_post(
    db: Connection<PgDb>,
    content_info: Json<ContentInfo<'_>>,
) -> (Status, Json<PostResponse>) {
    let pg_con = db.into_inner();
    // create a response struct
    let mut post_response = PostResponse {
        success: false,
        error: Vec::new(),
        post_id: 0i32,
    };
    // get content info from request
    let content = content_info.into_inner();
    // check if title, author and section is empty, add corresponding error if so
    if content.title.to_string().is_empty() {
        post_response.error.push("title is empty".to_string());
    }
    if content.author.to_string().is_empty() {
        post_response.error.push("author is empty".to_string());
    }
    if content.anonymous.to_string().is_empty() {
        post_response.error.push("anonymous is empty".to_string());
    }
    if content.section.to_string().is_empty() {
        post_response.error.push("section is empty".to_string());
    }
    // check if user has been banned, add corresponding error if so
    /* TODO */
    if !post_response.error.is_empty() {
        (Status::BadRequest, Json(post_response))
    } else {
        // get timestamp
        let created_time: DateTime<Utc> = Utc::now();
        let modified_time: DateTime<Utc> = Utc::now();
        // get tag string
        let mut tag: Option<String> = None;
        if !content.tag1.to_string().is_empty() {
            if !content.tag2.to_string().is_empty() {
                if !content.tag2.to_string().is_empty() {
                    tag = Some(format!(
                        "{{{},{},{}}}",
                        content.tag1.to_string(),
                        content.tag2.to_string(),
                        content.tag3.to_string(),
                    ));
                } else {
                    tag = Some(format!(
                        "{{{},{}}}",
                        content.tag1.to_string(),
                        content.tag2.to_string(),
                    ));
                }
            } else {
                tag = Some(format!("{{{}}}", content.tag1.to_string()));
            }
        }
        // fill the row in content_subject
        let content_subject = pgdb::content_subject::ActiveModel {
            title: Set(content.title.to_string()),
            author: Set(content.author.to_string()),
            anonymous: Set(content.anonymous),
            created_time: Set(created_time.with_timezone(&FixedOffset::east(8 * 3600))),
            modified_time: Set(modified_time.with_timezone(&FixedOffset::east(8 * 3600))),
            section: Set(content.section.to_string()),
            // tag: Set(tag),
            post_len: Set(1),
            ..Default::default()
        };
        // insert the row in database
        let res1 = content_subject
            .insert(&pg_con)
            .await
            .expect("Cannot save content");
        // println!("{:?}", res1.post_id.unwrap());
        // fill the row in content_reply
        let content_reply = pgdb::content_reply::ActiveModel {
            post_id: Set(res1.post_id.unwrap()),
            reply_id: Set(0),
            author: Set(content.author.to_string()),
            anonymous: Set(content.anonymous),
            created_time: Set(created_time.with_timezone(&FixedOffset::east(8 * 3600))),
            modified_time: Set(modified_time.with_timezone(&FixedOffset::east(8 * 3600))),
            content: Set(Some(content.content.to_string()).to_owned()),
            ..Default::default()
        };
        // insert the row into database
        let res2 = content_reply
            .insert(&pg_con)
            .await
            .expect("Cannot save content");
        // println!("{:?}", res2.reply_id.unwrap());
        // return the response
        post_response.success = true;
        post_response.post_id = res2.post_id.unwrap();
        (Status::Ok, Json(post_response))
    }
}

#[get("/<post_id>/<page>")]
pub async fn content_read(
    db: Connection<PgDb>,
    post_id: i32,
    page: i32,
) -> (Status, Json<ReadResponse>) {
    let pg_con = db.into_inner();
    // create a response struct
    let mut read_response = ReadResponse {
        success: false,
        error: Vec::new(),
        subject_info: Subject {
            post_id: post_id,
            title: String::new(),
            author: String::new(),
            anonymous: false,
            created_time: String::new(),
            modified_time: String::new(),
            section: String::new(),
            tag1: String::new(),
            tag2: String::new(),
            tag3: String::new(),
            post_len: 0i32,
        },
        reply_info: Vec::new(),
    };
    // check if the post not exsits, add corresponding error if so
    let subject_info = ContentSubject::find_by_id(post_id)
        .one(&pg_con)
        .await
        .expect("cannot fetch content from pgdb");
    if subject_info == None {
        read_response.error.push("Post not exsits".to_string());
        (Status::BadRequest, Json(read_response))
    } else {
        // get subject data
        let subject_data = subject_info.unwrap();
        // the number of replies displayed on one page
        let item_display = 10;
        let reply_info: Vec<pgdb::content_reply::Model>;
        if subject_data.post_len < page * item_display {
            reply_info = ContentReply::find()
                .filter(
                    Condition::all()
                        .add(pgdb::content_reply::Column::PostId.eq(post_id))
                        .add(pgdb::content_reply::Column::ReplyId.gte((page - 1) * item_display)),
                )
                .order_by_asc(pgdb::content_reply::Column::ReplyId)
                .all(&pg_con)
                .await
                .expect("cannot fetch content from pgdb");
        } else {
            reply_info = ContentReply::find()
                .filter(
                    Condition::all()
                        .add(pgdb::content_reply::Column::PostId.eq(post_id))
                        .add(pgdb::content_reply::Column::ReplyId.gte((page - 1) * item_display))
                        .add(
                            pgdb::content_reply::Column::ReplyId
                                .lte((page - 1) * item_display + item_display - 1),
                        ),
                )
                .order_by_asc(pgdb::content_reply::Column::ReplyId)
                .all(&pg_con)
                .await
                .expect("cannot fetch content from pgdb");
        }
        /* TODO */
        // split the tag into String tag1, tag2 and tag3
        // let tag_vec: Vec<&str> = subject_data.tag.unwrap().split(",").collect();

        // get subject data
        read_response.success = true;
        read_response.subject_info = Subject {
            post_id: subject_data.post_id,
            title: subject_data.title,
            author: subject_data.author,
            anonymous: subject_data.anonymous,
            created_time: subject_data
                .created_time
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            modified_time: subject_data
                .modified_time
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            section: subject_data.section,
            post_len: subject_data.post_len,
            /* TODO */
            // update tag1, tag2 and tag3
            ..read_response.subject_info
        };
        // get reply data
        for reply in &reply_info {
            let data = Reply {
                post_id: reply.post_id,
                reply_id: reply.reply_id,
                author: String::from(&reply.author),
                anonymous: reply.anonymous,
                created_time: reply.created_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                modified_time: reply.modified_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                content: String::from(reply.content.as_ref().unwrap()),
            };
            read_response.reply_info.push(data);
        }
        /* TODO */
        // process the author string(if anonymous)
        // if the author is anonymous, return an empty string for the "author"
        // if subject_info.anonymous {
        //     subject_info.author = String::new();
        // }else{
        //     subject_info.author = String::new(subject_info.author);
        // }

        // return the response
        (Status::Ok, Json(read_response))
    }
}

#[post("/<post_id>/reply", data = "<reply_info>", format = "json")]
pub async fn content_reply(
    post_id: i32,
    db: Connection<PgDb>,
    reply_info: Json<ReplyInfo<'_>>,
) -> (Status, Json<ReplyResponse>) {
    let pg_con = db.into_inner();
    // create a response struct
    let mut reply_response = ReplyResponse {
        success: false,
        error: Vec::new(),
        post_id: post_id,
        reply_id: 0i32,
    };
    // get content info from request
    let content = reply_info.into_inner();
    // check if author, anonymous and time is empty, add corresponding error if so
    if content.author.to_string().is_empty() {
        reply_response.error.push("author is empty".to_string());
    }
    if content.anonymous.to_string().is_empty() {
        reply_response.error.push("anonymous is empty".to_string());
    }
    // check if user has been banned, add corresponding error if so
    /* TODO */
    // if error exists, refuse to add user
    if !reply_response.error.is_empty() {
        (Status::BadRequest, Json(reply_response))
    } else {
        let subject_info = ContentSubject::find_by_id(post_id)
            .one(&pg_con)
            .await
            .expect("cannot fetch content from pgdb");
        if None == subject_info {
            reply_response.error.push("Post not exsits".to_string());
            (Status::BadRequest, Json(reply_response))
        } else {
            let subject_info = subject_info.unwrap();
            // get timestamp
            let created_time: DateTime<Utc> = Utc::now();
            let modified_time: DateTime<Utc> = Utc::now();
            // fill the row in content_reply
            let content_reply = pgdb::content_reply::ActiveModel {
                post_id: Set(post_id),
                reply_id: Set(subject_info.post_len),
                author: Set(content.author.to_string()),
                anonymous: Set(content.anonymous),
                created_time: Set(created_time.with_timezone(&FixedOffset::east(8 * 3600))),
                modified_time: Set(modified_time.with_timezone(&FixedOffset::east(8 * 3600))),
                content: Set(Some(content.content.to_string()).to_owned()),
                ..Default::default()
            };
            // insert the row in database
            let res1 = content_reply
                .insert(&pg_con)
                .await
                .expect("Cannot save content");
            // println!("{:?}", res1.reply_id.unwrap());
            reply_response.reply_id = subject_info.post_len;
            // modify the time and the post_len in content_subject
            let mut subject_content: pgdb::content_subject::ActiveModel =
                subject_info.into_active_model();
            subject_content.modified_time =
                Set(modified_time.with_timezone(&FixedOffset::east(8 * 3600)));
            subject_content.post_len = Set(reply_response.reply_id + 1);
            // insert the row in database
            let res2 = subject_content
                .update(&pg_con)
                .await
                .expect("Cannot update content");
            println!("{:?}", res2.post_id.unwrap());
            reply_response.success = true;
            // return the response
            (Status::Ok, Json(reply_response))
        }
    }
}

#[delete("/<post_id>/delete")]
pub async fn content_delete_post(
    db: Connection<PgDb>,
    post_id: i32,
) -> (Status, Json<PostResponse>) {
    let pg_con = db.into_inner();
    // create a response struct
    let mut post_response = PostResponse {
        success: false,
        error: Vec::new(),
        post_id: 0i32,
    };
    // check if the post not exsits, add corresponding error if so
    let subject_info = ContentSubject::find_by_id(post_id)
        .one(&pg_con)
        .await
        .expect("cannot fetch content from pgdb");
    if None == subject_info {
        post_response.error.push("Post not exsits".to_string());
        (Status::BadRequest, Json(post_response))
    } else {
        /* TODO */
        // check if time is within limit, if so, allow user to delete
        // delete data in content_subject
        let subject_info: pgdb::content_subject::ActiveModel = subject_info.unwrap().into();
        subject_info
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
        post_response.success = true;
        post_response.post_id = post_id;
        (Status::Ok, Json(post_response))
    }
}

#[delete("/<post_id>/<reply_id>/delete")]
pub async fn content_delete_reply(
    db: Connection<PgDb>,
    post_id: i32,
    reply_id: i32,
) -> (Status, Json<ReplyResponse>) {
    let pg_con = db.into_inner();
    // create a response struct
    let mut reply_response = ReplyResponse {
        success: false,
        error: Vec::new(),
        post_id: 0i32,
        reply_id: 0i32,
    };
    // check if the post not exsits, add corresponding error if so
    let reply_info = ContentReply::find_by_id((post_id, reply_id))
        .one(&pg_con)
        .await
        .expect("cannot fetch content from pgdb");
    if None == reply_info {
        reply_response.error.push("Reply not exsits".to_string());
        (Status::BadRequest, Json(reply_response))
    } else {
        /* TODO */
        // check if time is within limit, if so, allow user to delete
        //delete data in content_reply
        let reply_info: pgdb::content_reply::ActiveModel = reply_info.unwrap().into();
        reply_info
            .delete(&pg_con)
            .await
            .expect("cannot delete content from content_reply");
        /* TODO */
        // modify post_len in content_subject
        // return the response
        reply_response.success = true;
        reply_response.post_id = post_id;
        reply_response.reply_id = reply_id;
        (Status::Ok, Json(reply_response))
    }
}
