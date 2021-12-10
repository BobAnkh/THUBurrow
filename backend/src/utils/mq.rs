extern crate serde;
use futures::TryStreamExt;
use lazy_static::lazy_static;
use pulsar::{Consumer, Pulsar, SubType, TokioExecutor};
use sea_orm::sea_query::Expr;
use sea_orm::{entity::*, ConnectionTrait, Database, DatabaseConnection, DbErr, QueryFilter};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use tokio::time::Duration;

use crate::models::{pulsar::*, search::*, user::SEND_EMAIL_LIMIT};
use crate::pgdb::{content_post, prelude::*, user_collection, user_follow, user_like};
use crate::routes::trending::select_trending;
use super::email_send;
use super::email::check_email_exist;


lazy_static! {
    static ref TYPESENSE_API_KEY: String = {
        let env_v = "ROCKET_DATABASES=".to_string() + &env::var("ROCKET_DATABASES").ok().unwrap_or_else(|| r#"{search={url="http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="}}"#.to_string());
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url: String = match env_v.get("ROCKET_DATABASES") {
            Some(r) => match r.get("search") {
                Some(r) => match r.get("url") {
                    Some(r) => r.to_owned(),
                    None => "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="
                        .to_string(),
                },
                None => {
                    "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string()
                }
            },
            None => {
                "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string()
            }
        };
        let info: Vec<&str> = url.split('@').collect();
        let api_key: String;
        if info.len() == 1 {
            api_key = "8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_owned();
        } else if info.len() == 2 {
            api_key = info[1].to_owned();
        } else {
            panic!("Invalid typesense url.");
        }
        api_key
    };
    static ref TYPESENSE_ADDR: String = {
        let env_v = "ROCKET_DATABASES=".to_string() + &env::var("ROCKET_DATABASES").ok().unwrap_or_else(|| r#"{search={url="http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="}}"#.to_string());
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url: String = match env_v.get("ROCKET_DATABASES") {
            Some(r) => match r.get("search") {
                Some(r) => match r.get("url") {
                    Some(r) => r.to_owned(),
                    None => "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="
                        .to_string(),
                },
                None => {
                    "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string()
                }
            },
            None => {
                "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string()
            }
        };
        let info: Vec<&str> = url.split('@').collect();
        let addr: String;
        if info.len() == 1 || info.len() == 2 {
            addr = info[0].to_owned();
        } else {
            panic!("Invalid typesense url.");
        }
        addr
    };
    static ref POSTGRES_ADDR: String = {
        let env_v = "ROCKET_DATABASES=".to_string()
            + &env::var("ROCKET_DATABASES").ok().unwrap_or_else(|| {
                r#"{pgdb={url="postgres://postgres:postgres@127.0.0.1:5432/pgdb"}}"#.to_string()
            });
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url: String = match env_v.get("ROCKET_DATABASES") {
            Some(r) => match r.get("pgdb") {
                Some(r) => match r.get("url") {
                    Some(r) => r.to_owned(),
                    None => "postgres://postgres:postgres@127.0.0.1:5432/pgdb".to_string(),
                },
                None => "postgres://postgres:postgres@127.0.0.1:5432/pgdb".to_string(),
            },
            None => "postgres://postgres:postgres@127.0.0.1:5432/pgdb".to_string(),
        };
        url
    };
    static ref PULSAR_ADDR: String = {
        let env_v = "ROCKET_DATABASES=".to_string()
            + &env::var("ROCKET_DATABASES")
                .ok()
                .unwrap_or_else(|| r#"{pulsar-mq={url="pulsar://127.0.0.1:6650"}}"#.to_string());
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url: String = match env_v.get("ROCKET_DATABASES") {
            Some(r) => match r.get("pulsar-mq") {
                Some(r) => match r.get("url") {
                    Some(r) => r.to_owned(),
                    None => "pulsar://127.0.0.1:6650".to_string(),
                },
                None => "pulsar://127.0.0.1:6650".to_string(),
            },
            None => "pulsar://127.0.0.1:6650".to_string(),
        };
        url
    };
    static ref REDIS_ADDR: String = {
        let env_v = "ROCKET_DATABASES=".to_string()
            + &env::var("ROCKET_DATABASES").ok().unwrap_or_else(|| {
                r#"{keydb={url="redis://:keypassword@127.0.0.1:6300"}}"#.to_string()
            });
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url: String = match env_v.get("ROCKET_DATABASES") {
            Some(r) => match r.get("keydb") {
                Some(r) => match r.get("url") {
                    Some(r) => r.to_owned(),
                    None => "redis://:keypassword@127.0.0.1:6300".to_string(),
                },
                None => "redis://:keypassword@127.0.0.1:6300".to_string(),
            },
            None => "redis://:keypassword@127.0.0.1:6300".to_string(),
        };
        url
    };
}

async fn create_typesense_collections() -> Result<(), reqwest::Error> {
    //create typesense collections
    let collection_burrows = json!({
        "name": "burrows",
        "fields": [
            {"name": "burrow_id", "type": "int64", "index": false, "optional": true },
            {"name": "title", "type": "string", "locale": "zh"},
            {"name": "description", "type": "string", "locale": "zh"},
        ]
    });
    let collection_posts = json!({
        "name": "posts",
        "fields": [
            {"name": "post_id", "type": "int64", "index": false , "optional": true},
            {"name": "burrow_id", "type": "int64" , "index": false , "optional": true},
            {"name": "title", "type": "string", "locale": "zh"},
            {"name": "section", "type": "string[]", "facet":true},
            {"name": "tag", "type": "string[]", "facet":true},
        ]
    });
    let collection_replies = json!({
        "name": "replies",
        "fields": [
            {"name": "post_id", "type": "int64", "index": false , "optional": true},
            {"name": "reply_id", "type": "int32", "index": false , "optional": true},
            {"name": "burrow_id", "type": "int64", "index": false , "optional": true},
            {"name": "content", "type": "string", "locale": "zh"},
        ]
    });
    let client = reqwest::Client::new();
    for each in [collection_burrows, collection_posts, collection_replies].iter() {
        match client.build_post("/collections").json(&each).send().await {
            Ok(a) => match a.status().as_u16() {
                201 => (),
                400 => panic!(
                    "Bad Request - The request could not be understood due to malformed syntax."
                ),
                401 => panic!("Unauthorized - Your API key is wrong."),
                404 => panic!("Not Found - The requested resource is not found."),
                409 => {
                    log::warn!("Collections already exist.");
                }
                422 => panic!(
                    "Unprocessable Entity - Request is well-formed, but cannot be processed."
                ),
                503 => panic!(
                    "Service Unavailable - We’re temporarily offline. Please try again later."
                ),
                _ => panic!(
                    "Unknown error when creating collections. Status code:{}",
                    a.status().as_u16()
                ),
            },
            Err(e) => panic!("Err when create typesense collections,{:?}", e),
        }
    }
    Ok(())
}

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
                    .post("/collections/replies/documents")
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
                    .delete(&format!("/collections/burrows/documents/{}", burrow_id))
                    .send()
                    .await
                {
                    Ok(_) => log::info!("[PULSAR-SEARCH] Delete burrow successfully"),
                    Err(e) => log::error!("[PULSAR-SEARCH] Delete burrow failed: {:?}", e),
                }
            }
            PulsarSearchData::DeletePost(post_id) => {
                match client
                    .delete(&format!("/collections/posts/documents/{}", post_id))
                    .send()
                    .await
                {
                    Ok(_) => println!("[PULSAR-SEARCH] Delete post successfully"),
                    Err(e) => println!("[PULSAR-SEARCH] Delete post failed: {:?}", e),
                }
            }
            PulsarSearchData::DeleteReply(post_id, reply_id) => {
                match client
                    .delete(&format!(
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
        if check_email_exist(&data.email).await.0 {
            let mut op_times: usize = 1;
            // **TODO: Generate verification code**
            let mut verification_code = "666666";
            let get_redis_result: Result<Option<String>, redis::RedisError> = redis::cmd("GET")
                .arg(&data.email)
                .query_async(&mut kv_conn)
                .await;
            match get_redis_result {
                Ok(res) => {
                    log::info!("[PULSAR-EMAIL] Redis get success");
                    if res.is_some() {
                        let s = res.unwrap();
                        let values: Vec<&str> = s.split(":").collect();
                        op_times = values[1].parse::<usize>().unwrap();
                        // **TODO: Generate verification code**
                        verification_code = "233333";
                        if op_times <= SEND_EMAIL_LIMIT {
                            op_times = op_times + 1;
                            email_send.post(data.email.clone(), verification_code);
                            log::info!("[PULSAR-EMAIL] Email send success");
                        } else {
                            log::info!("[PULSAR-EMAIL] User sent too many emails, refuse to send");
                        }
                    }
                },
                Err(e) => {
                    log::error!("[PULSAR-EMAIL] Redis get failed {:?}", e);
                    continue;
                },
            };
            let set_redis_result: Result<String, redis::RedisError> = redis::cmd("SETEX")
                .arg(&data.email)
                .arg(14400i32)
                .arg(op_times.to_string() + ":" + verification_code)
                .query_async(&mut kv_conn)
                .await;
            match set_redis_result {
                Ok(_) => log::info!("[PULSAR-EMAIL] Redis set success"),
                Err(e) => log::error!("[PULSAR-EMAIL] Redis set failed {:?}", e),
            }
        } else {
            let set_redis_result: Result<String, redis::RedisError> = redis::cmd("SETEX")
                .arg(&data.email)
                .arg(14400i32)
                .arg((SEND_EMAIL_LIMIT + 1).to_string() + ":" + "666666")
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
