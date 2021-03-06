//! Tasks for task_executor
//!
//! This module contains a bunch of functions, each of which represents a background
//! task behind Message Queue executed by task_executor.

use futures::TryStreamExt;
use pulsar::{Consumer, Pulsar, SubType, TokioExecutor};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sea_orm::sea_query::Expr;
use sea_orm::{entity::*, Database, DatabaseConnection, DbErr, QueryFilter, TransactionTrait};
use serde_json::json;
use tokio::time::Duration;

use super::email::{self, check_email_exist};
use crate::config::mq::*;
use crate::config::user::SEND_EMAIL_LIMIT;
use crate::config::BACKEND_TEST_MODE;
use crate::db::{content_post, prelude::*, user_collection, user_follow, user_like};
use crate::models::{pulsar::*, search::*};
use crate::routes::trending::select_trending;

pub trait Typesense {
    fn build_get(&self, uri: &str) -> reqwest::RequestBuilder;
    fn build_post(&self, uri: &str) -> reqwest::RequestBuilder;
    fn build_delete(&self, uri: &str) -> reqwest::RequestBuilder;
    fn build_patch(&self, uri: &str) -> reqwest::RequestBuilder;
}

impl Typesense for reqwest::Client {
    fn build_get(&self, uri: &str) -> reqwest::RequestBuilder {
        let typesense_api_key: &str = &TYPESENSE_API_KEY;
        let typesense_addr: String = TYPESENSE_ADDR.to_owned();
        self.get(typesense_addr + uri)
            .header("X-TYPESENSE-API-KEY", typesense_api_key)
    }

    fn build_post(&self, uri: &str) -> reqwest::RequestBuilder {
        let typesense_api_key: &str = &TYPESENSE_API_KEY;
        let typesense_addr: String = TYPESENSE_ADDR.to_owned();
        self.post(typesense_addr + uri)
            .header("X-TYPESENSE-API-KEY", typesense_api_key)
    }

    fn build_delete(&self, uri: &str) -> reqwest::RequestBuilder {
        let typesense_api_key: &str = &TYPESENSE_API_KEY;
        let typesense_addr: String = TYPESENSE_ADDR.to_owned();
        self.delete(typesense_addr + uri)
            .header("X-TYPESENSE-API-KEY", typesense_api_key)
    }

    fn build_patch(&self, uri: &str) -> reqwest::RequestBuilder {
        let typesense_api_key: &str = &TYPESENSE_API_KEY;
        let typesense_addr: String = TYPESENSE_ADDR.to_owned();
        self.patch(typesense_addr + uri)
            .header("X-TYPESENSE-API-KEY", typesense_api_key)
    }
}

async fn create_typesense_collections() -> Result<(), reqwest::Error> {
    //create typesense collections
    let collection_burrows = json!({
        "name": "burrows",
        "fields": [
            {"name": "burrow_id", "type": "int64"},
            {"name": "title", "type": "string", "locale": "zh"},
            {"name": "description", "type": "string", "locale": "zh"},
        ]
    });
    let collection_posts = json!({
        "name": "posts",
        "fields": [
            {"name": "post_id", "type": "int64"},
            {"name": "burrow_id", "type": "int64"},
            {"name": "title", "type": "string", "locale": "zh"},
            {"name": "section", "type": "string[]", "facet": true},
            {"name": "tag", "type": "string[]", "facet": true},
        ]
    });
    let collection_replies = json!({
        "name": "replies",
        "fields": [
            {"name": "post_id", "type": "int64", "facet": true},
            {"name": "reply_id", "type": "int32", "index": false , "optional": true},
            {"name": "burrow_id", "type": "int64"},
            {"name": "content", "type": "string", "locale": "zh"},
        ]
    });
    let client = reqwest::Client::new();
    for each in [collection_burrows, collection_posts, collection_replies].iter() {
        match client.build_post("/collections").json(&each).send().await {
            Ok(r) => match r.status().as_u16() {
                201 => {
                    log::warn!(
                        "Collection {} created successfully.",
                        each["name"].as_str().unwrap()
                    );
                }
                400 => {
                    log::warn!(
                        "Create collection {} failed with bad request. {}",
                        each["name"].as_str().unwrap(),
                        r.text().await.unwrap()
                    );
                    panic!(
                        "Bad Request - The request could not be understood due to malformed syntax."
                    )
                }
                409 => {
                    log::warn!(
                        "Collection {} already exists. Skip creation.",
                        each["name"].as_str().unwrap()
                    );
                }
                _ => {
                    log::warn!(
                        "{} Create collection {} failed with response. {}",
                        r.status().as_u16(),
                        each["name"].as_str().unwrap(),
                        r.text().await.unwrap()
                    );
                    panic!("Unknown error when creating collections.")
                }
            },
            Err(e) => panic!("Err when create typesense collections,{:?}", e),
        }
    }
    Ok(())
}

pub async fn pulsar_typesense() -> Result<(), pulsar::Error> {
    // setup pulsar consumer
    let pulsar_addr: String = PULSAR_ADDR.to_owned();
    let addr = pulsar_addr;
    let topic = "persistent://public/default/search".to_string();
    let builder = Pulsar::builder(addr, TokioExecutor);
    let pulsar: Pulsar<_> = builder.build().await?;
    let mut consumer: Consumer<PulsarSearchData, _> = pulsar
        .consumer()
        .with_topic(topic)
        .with_subscription_type(SubType::Exclusive)
        .build()
        .await?;

    match create_typesense_collections().await {
        Ok(_) => {
            log::warn!("Search engine successfully initialized");
        }
        Err(e) => {
            panic!("Failed to initialize search engine: {:?}", e);
        }
    };
    let client = reqwest::Client::new();
    while let Some(msg) = consumer.try_next().await? {
        consumer.ack(&msg).await?;
        // println!("metadata: {:?},id: {:?}", msg.metadata(), msg.message_id());
        let data = match msg.deserialize() {
            Ok(data) => data,
            Err(e) => {
                log::error!("could not deserialize message: {:?}", e);
                continue;
            }
        };
        match data {
            PulsarSearchData::CreateBurrow(burrow) => {
                let data: TypesenseBurrowData = burrow.into();
                match client
                    .build_post("/collections/burrows/documents")
                    .json(&data)
                    .send()
                    .await
                {
                    Ok(r) => match r.status().as_u16() {
                        201 => log::info!("[PULSAR-SEARCH] 201: Add new burrow."),
                        409 => log::info!("[PULSAR-SEARCH] 409: Burrow already exist."),
                        _ => log::error!(
                            "[PULSAR-SEARCH] {}:  Failed to add new burrow. {}",
                            r.status().as_u16(),
                            r.text().await.unwrap()
                        ),
                    },
                    Err(e) => log::error!("add new burrow failed {:?}", e),
                }
            }
            PulsarSearchData::CreatePost(post) => {
                let data: TypesensePostData = post.into();

                match client
                    .build_post("/collections/posts/documents")
                    .json(&data)
                    .send()
                    .await
                {
                    Ok(r) => match r.status().as_u16() {
                        201 => log::info!("[PULSAR-SEARCH] 201: Add new post."),
                        409 => log::info!("[PULSAR-SEARCH] 409: Post already exist."),
                        _ => log::error!(
                            "[PULSAR-SEARCH] {}:  Failed to add new post. {}",
                            r.status().as_u16(),
                            r.text().await.unwrap()
                        ),
                    },
                    Err(e) => log::error!("add new post failed {:?}", e),
                }
            }
            PulsarSearchData::CreateReply(reply) => {
                let data: TypesenseReplyData = reply.into();

                match client
                    .build_post("/collections/replies/documents")
                    .json(&data)
                    .send()
                    .await
                {
                    Ok(r) => match r.status().as_u16() {
                        201 => log::info!("[PULSAR-SEARCH] 201: Add new reply."),
                        409 => log::info!("[PULSAR-SEARCH] 409: Reply already exist."),
                        _ => log::error!(
                            "[PULSAR-SEARCH] {}:  Failed to add new reply. {}",
                            r.status().as_u16(),
                            r.text().await.unwrap()
                        ),
                    },
                    Err(e) => log::error!("add new reply failed {:?}", e),
                }
            }
            PulsarSearchData::UpdateBurrow(burrow) => {
                let burrow_id = burrow.burrow_id;
                let data: TypesenseBurrowData = burrow.into();
                match client
                    .build_get(&format!("/collections/burrows/documents/{}", burrow_id))
                    .send()
                    .await
                {
                    Ok(r) => match r.status().as_u16() {
                        200 => {
                            let response = r.json::<TypesenseBurrowData>().await.unwrap();
                            if response.update_time < data.update_time {
                                match client
                                    .build_patch(&format!(
                                        "/collections/burrows/documents/{}",
                                        burrow_id
                                    ))
                                    .json(&data)
                                    .send()
                                    .await
                                {
                                    Ok(r) => match r.status().as_u16() {
                                        201 => log::info!("[PULSAR-SEARCH] 201: Update burrow."),
                                        _ => log::error!(
                                            "[PULSAR-SEARCH] {}:  Failed to update burrow. {}",
                                            r.status().as_u16(),
                                            r.text().await.unwrap()
                                        ),
                                    },
                                    Err(e) => {
                                        log::error!("[PULSAR-SEARCH] Update burrow failed {:?}", e)
                                    }
                                }
                            } else {
                                log::info!("[PULSAR-SEARCH] Burrow is up to date.");
                            }
                        }
                        404 => log::info!("[PULSAR-SEARCH] 404: Burrow does not exist."),
                        _ => log::error!(
                            "[PULSAR-SEARCH] {}:  Failed to update burrow. {}",
                            r.status().as_u16(),
                            r.text().await.unwrap()
                        ),
                    },
                    Err(e) => log::error!("[PULSAR-SEARCH] Update burrow failed {:?}", e),
                }
            }
            PulsarSearchData::UpdatePost(post) => {
                let post_id = post.post_id;
                let data: TypesensePostData = post.into();
                match client
                    .build_get(&format!("/collections/posts/documents/{}", post_id))
                    .send()
                    .await
                {
                    Ok(r) => match r.status().as_u16() {
                        200 => {
                            let response = r.json::<TypesensePostData>().await.unwrap();
                            if response.update_time < data.update_time {
                                match client
                                    .build_patch(&format!(
                                        "/collections/posts/documents/{}",
                                        post_id
                                    ))
                                    .json(&data)
                                    .send()
                                    .await
                                {
                                    Ok(r) => match r.status().as_u16() {
                                        201 => log::info!("[PULSAR-SEARCH] 201: Update post."),
                                        _ => log::error!(
                                            "[PULSAR-SEARCH] {}:  Failed to update post. {}",
                                            r.status().as_u16(),
                                            r.text().await.unwrap()
                                        ),
                                    },
                                    Err(e) => {
                                        log::error!("[PULSAR-SEARCH] Update post failed {:?}", e)
                                    }
                                }
                            } else {
                                log::info!("[PULSAR-SEARCH] Post is up to date.");
                            }
                        }
                        404 => log::info!("[PULSAR-SEARCH] 404: post does not exist."),
                        _ => log::error!(
                            "[PULSAR-SEARCH] {}:  Failed to update post. {}",
                            r.status().as_u16(),
                            r.text().await.unwrap()
                        ),
                    },
                    Err(e) => log::error!("[PULSAR-SEARCH] Update post failed {:?}", e),
                }
            }
            PulsarSearchData::UpdateReply(reply) => {
                let post_id = reply.post_id;
                let reply_id = reply.reply_id;
                let data: TypesenseReplyData = reply.into();
                match client
                    .build_get(&format!(
                        "/collections/replies/documents/{}-{}",
                        post_id, reply_id
                    ))
                    .send()
                    .await
                {
                    Ok(r) => match r.status().as_u16() {
                        200 => {
                            let response = r.json::<TypesenseReplyData>().await.unwrap();
                            if response.update_time < data.update_time {
                                match client
                                    .build_patch(&format!(
                                        "/collections/replies/documents/{}-{}",
                                        post_id, reply_id
                                    ))
                                    .json(&data)
                                    .send()
                                    .await
                                {
                                    Ok(r) => match r.status().as_u16() {
                                        201 => log::info!("[PULSAR-SEARCH] 201: Update reply."),
                                        _ => log::error!(
                                            "[PULSAR-SEARCH] {}:  Failed to update reply. {}",
                                            r.status().as_u16(),
                                            r.text().await.unwrap()
                                        ),
                                    },
                                    Err(e) => {
                                        log::error!("[PULSAR-SEARCH] Update reply failed {:?}", e)
                                    }
                                }
                            } else {
                                log::info!("[PULSAR-SEARCH] Reply is up to date.");
                            }
                        }
                        404 => log::info!("[PULSAR-SEARCH] 404: reply does not exist."),
                        _ => log::error!(
                            "[PULSAR-SEARCH] {}:  Failed to update reply. {}",
                            r.status().as_u16(),
                            r.text().await.unwrap()
                        ),
                    },
                    Err(e) => log::error!("[PULSAR-SEARCH] Update reply failed {:?}", e),
                }
            }
            PulsarSearchData::DeleteBurrow(burrow_id) => {
                match client
                    .build_delete(&format!("/collections/burrows/documents/{}", burrow_id))
                    .send()
                    .await
                {
                    Ok(_) => log::info!("[PULSAR-SEARCH] Delete burrow successfully"),
                    Err(e) => log::error!("[PULSAR-SEARCH] Delete burrow failed: {:?}", e),
                }
            }
            PulsarSearchData::DeletePost(post_id) => {
                match client
                    .build_delete(&format!("/collections/posts/documents/{}", post_id))
                    .send()
                    .await
                {
                    Ok(_) => println!("[PULSAR-SEARCH] Delete post successfully"),
                    Err(e) => println!("[PULSAR-SEARCH] Delete post failed: {:?}", e),
                }
            }
            PulsarSearchData::DeleteReply(post_id, reply_id) => {
                match client
                    .build_delete(&format!(
                        "/collections/replies/documents/{}-{}",
                        post_id, reply_id
                    ))
                    .send()
                    .await
                {
                    Ok(_) => println!("[PULSAR-SEARCH] Delete reply successfully"),
                    Err(e) => println!("[PULSAR-SEARCH] Delete reply failed: {:?}", e),
                }
            }
        }
    }
    Ok(())
}

pub async fn pulsar_relation() -> Result<(), pulsar::Error> {
    // setup pulsar consumer
    let pulsar_addr: String = PULSAR_ADDR.to_owned();
    let addr = pulsar_addr;
    let topic = "persistent://public/default/relation".to_string();
    let builder = Pulsar::builder(addr, TokioExecutor);
    let pulsar: Pulsar<_> = builder.build().await?;
    let mut consumer: Consumer<PulsarRelationData, _> = pulsar
        .consumer()
        .with_topic(topic)
        .with_subscription_type(SubType::Exclusive)
        .build()
        .await?;
    let postgres_addr: &str = &POSTGRES_ADDR;
    let db: DatabaseConnection = match Database::connect(postgres_addr).await {
        Ok(db) => db,
        Err(e) => {
            log::error!("[PULSAR-RELATION] Database Error {:?}", e);
            panic!("pulsar relation database connection failed");
        }
    };
    while let Some(msg) = consumer.try_next().await? {
        consumer.ack(&msg).await?;
        let data = match msg.deserialize() {
            Ok(data) => data,
            Err(e) => {
                log::error!("[PULSAR-RELATION] Could not deserialize message: {:?}", e);
                continue;
            }
        };
        match data {
            PulsarRelationData::ActivateLike(uid, post_id) => {
                let like = user_like::ActiveModel {
                    uid: Set(uid),
                    post_id: Set(post_id),
                };
                match db
                    .transaction::<_, (), DbErr>(|txn| {
                        Box::pin(async move {
                            like.insert(txn).await?;
                            let update_res = ContentPost::update_many()
                                .col_expr(
                                    content_post::Column::LikeNum,
                                    Expr::col(content_post::Column::LikeNum).add(1),
                                )
                                .filter(content_post::Column::PostId.eq(post_id))
                                .exec(txn)
                                .await?;
                            if update_res.rows_affected != 1 {
                                return Err(DbErr::RecordNotFound("post not found".to_string()));
                            }
                            Ok(())
                        })
                    })
                    .await
                {
                    Ok(_) => log::info!("[PULSAR-RELATION] Insert like success"),
                    Err(e) => log::error!("[PULSAR-RELATION] Insert like failed {:?}", e),
                }
            }
            PulsarRelationData::DeactivateLike(uid, post_id) => {
                let like = user_like::ActiveModel {
                    uid: Set(uid),
                    post_id: Set(post_id),
                };
                match like.delete(&db).await {
                    Ok(res) => {
                        log::info!(
                            "[PULSAR-RELATION] Delete like success {}",
                            res.rows_affected
                        );
                        if res.rows_affected == 1 {
                            let _ = ContentPost::update_many()
                                .col_expr(
                                    content_post::Column::LikeNum,
                                    Expr::col(content_post::Column::LikeNum).sub(1),
                                )
                                .filter(content_post::Column::PostId.eq(post_id))
                                .exec(&db)
                                .await;
                        }
                    }
                    Err(e) => log::error!("[PULSAR-RELATION] Delete like failed {:?}", e),
                }
            }
            PulsarRelationData::ActivateCollection(uid, post_id) => {
                let collection = user_collection::ActiveModel {
                    uid: Set(uid),
                    post_id: Set(post_id),
                    ..Default::default()
                };
                match db
                    .transaction::<_, (), DbErr>(|txn| {
                        Box::pin(async move {
                            collection.insert(txn).await?;
                            let update_res = ContentPost::update_many()
                                .col_expr(
                                    content_post::Column::CollectionNum,
                                    Expr::col(content_post::Column::CollectionNum).add(1),
                                )
                                .filter(content_post::Column::PostId.eq(post_id))
                                .exec(txn)
                                .await?;
                            if update_res.rows_affected != 1 {
                                return Err(DbErr::RecordNotFound("post not found".to_string()));
                            }
                            Ok(())
                        })
                    })
                    .await
                {
                    Ok(_) => log::info!("[PULSAR-RELATION] Insert collection success"),
                    Err(e) => log::error!("[PULSAR-RELATION] Insert collection failed {:?}", e),
                }
            }
            PulsarRelationData::DeactivateCollection(uid, post_id) => {
                let collection = user_collection::ActiveModel {
                    uid: Set(uid),
                    post_id: Set(post_id),
                    ..Default::default()
                };
                match collection.delete(&db).await {
                    Ok(res) => {
                        log::info!(
                            "[PULSAR-RELATION] Delete collection success {}",
                            res.rows_affected
                        );
                        if res.rows_affected == 1 {
                            let _ = ContentPost::update_many()
                                .col_expr(
                                    content_post::Column::CollectionNum,
                                    Expr::col(content_post::Column::CollectionNum).sub(1),
                                )
                                .filter(content_post::Column::PostId.eq(post_id))
                                .exec(&db)
                                .await;
                        }
                    }
                    Err(e) => log::error!("[PULSAR-RELATION] Delete collection failed {:?}", e),
                }
            }
            PulsarRelationData::ActivateFollow(uid, burrow_id) => {
                let follow = user_follow::ActiveModel {
                    uid: Set(uid),
                    burrow_id: Set(burrow_id),
                    ..Default::default()
                };
                match db
                    .transaction::<_, (), DbErr>(|txn| {
                        Box::pin(async move {
                            follow.insert(txn).await?;
                            let res = Burrow::find_by_id(burrow_id).one(txn).await?;
                            match res {
                                Some(_) => Ok(()),
                                None => Err(DbErr::RecordNotFound("burrow not found".to_string())),
                            }
                        })
                    })
                    .await
                {
                    Ok(_) => log::info!("[PULSAR-RELATION] Insert follow success"),
                    Err(e) => log::error!("[PULSAR-RELATION] Insert follow failed {:?}", e),
                }
            }
            PulsarRelationData::DeactivateFollow(uid, burrow_id) => {
                let follow = user_follow::ActiveModel {
                    uid: Set(uid),
                    burrow_id: Set(burrow_id),
                    ..Default::default()
                };
                match follow.delete(&db).await {
                    Ok(res) => {
                        log::info!(
                            "[PULSAR-RELATION] Delete follow success {}",
                            res.rows_affected
                        );
                    }
                    Err(e) => log::error!("[PULSAR-RELATION] Delete follow failed {:?}", e),
                }
            }
        }
    }

    Ok(())
}

pub async fn generate_trending() -> redis::RedisResult<()> {
    // setup pulsar consumer
    let redis_addr: String = REDIS_ADDR.to_owned();
    let addr = redis_addr;
    let client = redis::Client::open(addr)?;
    let mut kv_conn = client.get_async_connection().await?;
    let postgres_addr: &str = &POSTGRES_ADDR;
    let pg_con: DatabaseConnection = match Database::connect(postgres_addr).await {
        Ok(db) => db,
        Err(e) => {
            log::error!("[PULSAR-TRENDING] Database Error{:?}", e);
            panic!("pulsar trending database connection failed");
        }
    };
    let seconds = 900;
    let mut interval = tokio::time::interval(Duration::from_secs(seconds));
    interval.tick().await;
    loop {
        interval.tick().await;
        match select_trending(&pg_con, &mut kv_conn).await {
            Ok(trending) => {
                log::info!("[PULSAR-TRENDING] Get Trending: {}", trending);
            }
            Err(e) => {
                log::error!("[PULSAR-TRENDING] Error: {}", e);
            }
        }
    }
}

async fn get_set_redis(
    kv_conn: &mut redis::aio::Connection,
    email: &str,
    verification_code: &str,
) -> Result<String, redis::RedisError> {
    let get_res: Option<String> = redis::cmd("GET").arg(email).query_async(kv_conn).await?;
    let op_times = 1 + match get_res {
        Some(res) => {
            let values: Vec<&str> = res.split(':').collect();
            values[0].parse::<usize>().unwrap()
        }
        None => 0,
    };
    // check request rate
    if op_times > SEND_EMAIL_LIMIT {
        let e = (redis::ErrorKind::ExtensionError, "RateLimit").into();
        return Err(e);
    } else {
        let _: String = redis::cmd("SETEX")
            .arg(email)
            .arg(EMAIL_TOKEN_EX)
            .arg(op_times.to_string() + ":" + verification_code)
            .query_async(kv_conn)
            .await?;
    }
    Ok("Success".to_string())
}

pub async fn pulsar_email() -> Result<(), pulsar::Error> {
    // setup pulsar consumer
    let redis_addr: String = REDIS_ADDR.to_owned();
    let addr = redis_addr;
    let client = match redis::Client::open(addr) {
        Ok(c) => c,
        Err(e) => {
            panic!("[PULSAR-EMAIL] Redis Error: {:?}", e);
        }
    };
    let mut kv_conn = match client.get_async_connection().await {
        Ok(c) => c,
        Err(e) => {
            panic!("[PULSAR-EMAIL] Redis Error: {:?}", e);
        }
    };
    let pulsar_addr: String = PULSAR_ADDR.to_owned();
    let addr = pulsar_addr;
    let topic = "persistent://public/default/email".to_string();
    let builder = Pulsar::builder(addr, TokioExecutor);
    let pulsar: Pulsar<_> = builder.build().await?;
    let mut consumer: Consumer<PulsarSendEmail, _> = pulsar
        .consumer()
        .with_topic(topic)
        .with_subscription_type(SubType::Exclusive)
        .build()
        .await?;
    while let Some(msg) = consumer.try_next().await? {
        consumer.ack(&msg).await?;
        let data = match msg.deserialize() {
            Ok(data) => data,
            Err(e) => {
                log::error!("[PULSAR-RELATION] Could not deserialize message: {:?}", e);
                continue;
            }
        };
        let (email, repeat_times) = match data {
            PulsarSendEmail::Sign { email } => (email, 6),
            PulsarSendEmail::Reset { email } => (email, 10),
        };
        if *BACKEND_TEST_MODE {
            let verification_code = "6".repeat(repeat_times);
            match get_set_redis(&mut kv_conn, &email, &verification_code).await {
                Ok(_) => log::info!("[PULSAR-EMAIL] Redis get & set success, Email send success"),
                Err(e) => match e.kind() {
                    redis::ErrorKind::ExtensionError => {
                        log::info!("[PULSAR-EMAIL] User sent too many emails, refuse to send");
                        continue;
                    }
                    _ => {
                        log::error!("[PULSAR-EMAIL] Redis get/set failed {:?}", e);
                        continue;
                    }
                },
            }
        } else if check_email_exist(&email).await.0 {
            // generate verification code
            let verification_code: String = std::iter::repeat(())
                .map(|()| thread_rng().sample(Alphanumeric))
                .map(char::from)
                .take(repeat_times)
                .collect();
            // println!("{}", verification_code);
            match get_set_redis(&mut kv_conn, &email, &verification_code).await {
                Ok(_) => {
                    log::info!("[PULSAR-EMAIL] Redis get & set success");
                    match email::send(email, verification_code).await {
                        Ok(res) => {
                            log::info!("[PULSAR-EMAIL] Email send success, response: {}", res);
                            // println!("{}", res);
                        }
                        Err(e) => {
                            log::error!("[PULSAR-EMAIL] Email send failed: {}", e);
                        }
                    };
                }
                Err(e) => match e.kind() {
                    redis::ErrorKind::ExtensionError => {
                        log::info!("[PULSAR-EMAIL] User sent too many emails, refuse to send");
                        continue;
                    }
                    _ => {
                        log::error!("[PULSAR-EMAIL] Redis get/set failed {:?}", e);
                        continue;
                    }
                },
            }
        } else {
            let verification_code = "6".repeat(repeat_times);
            let set_redis_result: Result<String, redis::RedisError> = redis::cmd("SETEX")
                .arg(&email)
                .arg(EMAIL_TOKEN_EX)
                .arg((SEND_EMAIL_LIMIT + 1).to_string() + ":" + &verification_code)
                .query_async(&mut kv_conn)
                .await;
            match set_redis_result {
                Ok(_) => log::info!("[PULSAR-EMAIL] Redis set success"),
                Err(e) => log::error!("[PULSAR-EMAIL] Redis set failed {:?}", e),
            }
        }
    }
    Ok(())
}
