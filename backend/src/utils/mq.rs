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

use crate::pgdb::{content_post, prelude::*, user_collection, user_follow, user_like};
use crate::req::pulsar::*;
use crate::routes::trending::select_trending;

lazy_static! {
    static ref TYPESENSE_API_KEY: String = {
        let env_v = "ROCKET_DATABASES=".to_string() + &env::var("ROCKET_DATABASES").ok().unwrap_or_else(|| r#"{search={url="http://127.0.0.1:8108@$8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="}}"#.to_string());
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url: String = match env_v.get("ROCKET_DATABASES") {
            Some(r) => match r.get("search") {
                Some(r) => match r.get("url") {
                    Some(r) => r.to_owned(),
                    None => "http://127.0.0.1:8108@$8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="
                        .to_string(),
                },
                None => "http://127.0.0.1:8108@$8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="
                    .to_string(),
            },
            None => {
                "http://127.0.0.1:8108@$8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string()
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
        let env_v = "ROCKET_DATABASES=".to_string() + &env::var("ROCKET_DATABASES").ok().unwrap_or_else(|| r#"{search={url="http://127.0.0.1:8108@$8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="}}"#.to_string());
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url: String = match env_v.get("ROCKET_DATABASES") {
            Some(r) => match r.get("search") {
                Some(r) => match r.get("url") {
                    Some(r) => r.to_owned(),
                    None => "http://127.0.0.1:8108@$8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="
                        .to_string(),
                },
                None => "http://127.0.0.1:8108@$8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="
                    .to_string(),
            },
            None => {
                "http://127.0.0.1:8108@$8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string()
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

pub async fn create_typesense_collections() -> Result<(), reqwest::Error> {
    //create typesense collections
    let collection_burrows = json!({
      "name": "burrows",
      "fields": [
        {"name": "id", "type": "int64"},
        {"name": "title", "type": "string" },
        {"name": "introduction", "type": "string"},
        {"name": "update_time", "type": "string"}
      ]
    });
    let collection_posts = json!({
      "name": "posts",
      "fields": [
        {"name": "id", "type": "int64" },
        {"name": "title", "type": "string" },
        {"name": "burrow_id", "type": "int64" },
        {"name": "update_time", "type": "string"},
        {"name": "post_type", "type": "int32"},
        {"name": "post_state", "type": "int32"},
        {"name": "section", "type": "string[]"},
        {"name": "tag", "type": "string[]"}
      ]
    });
    let collection_replies = json!({
      "name": "replies",
      "fields": [
        {"name": "id", "type": "int32" },
        {"name": "post_id", "type": "int64"},
        {"name": "burrow_id", "type": "int64"},
        {"name": "update_time", "type": "string"},
        {"name": "content", "type": "string"},
        {"name": "reply_state", "type": "int32"}
      ]
    });
    let client = reqwest::Client::new();
    for each in [collection_burrows, collection_posts, collection_replies].iter() {
        let _res = client.build_post("/collections").json(&each).send().await?;
        // TODO: match the status code of Response here, to see whether it is successfully created or is already created, or failed
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
            .header("Content-Type", "application/json")
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
            .header("Content-Type", "application/json")
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
            println!("Typesense successfully initialized");
        }
        Err(e) => {
            panic!("Failed to initialize typesense: {}", e);
        }
    };
    let client = reqwest::Client::new();
    while let Some(msg) = consumer.try_next().await? {
        consumer.ack(&msg).await?;
        println!("metadata: {:?},id: {:?}", msg.metadata(), msg.message_id());
        let data = match msg.deserialize() {
            Ok(data) => data,
            Err(e) => {
                println!("could not deserialize message: {:?}", e);
                continue;
            }
        };
        // if data.data.as_str() != "data" {
        //     println!("Unexpected payload: {}", &data.data);
        //     break;
        // }
        match data {
            PulsarSearchData::CreateBurrow(burrow) => {
                let data: TypesenseBurrowData = burrow.into();

                match client
                    .build_post("/collections/burrows/documents")
                    .body(serde_json::to_string(&data).unwrap())
                    .send()
                    .await
                {
                    Ok(r) => println!("add new burrow.{:?}", r),
                    Err(e) => println!("add new burrow failed {:?}", e),
                }
            }
            PulsarSearchData::CreatePost(post) => {
                let data: TypesensePostData = post.into();

                match client
                    .build_post("/collections/burrows/documents")
                    .body(serde_json::to_string(&data).unwrap())
                    .send()
                    .await
                {
                    Ok(r) => println!("add new post.{:?}", r),
                    Err(e) => println!("add new post failed {:?}", e),
                }
            }
            PulsarSearchData::CreateReply(reply) => {
                let data: TypesenseReplyData = reply.into();

                match client
                    .post("/collections/burrows/documents")
                    .body(serde_json::to_string(&data).unwrap())
                    .send()
                    .await
                {
                    Ok(r) => println!("add new reply.{:?}", r),
                    Err(e) => println!("add new reply failed {:?}", e),
                }
            }
            PulsarSearchData::UpdateBurrow(burrow) => {
                // TODO: read from typesense first, check the time if it is newer than the one in typesense
                let data: TypesenseBurrowData = burrow.into();
                let uri: String = format!("/collections/burrows/documents/{}", data.id);
                match client
                    .build_patch(&uri)
                    .body(serde_json::to_string(&data).unwrap())
                    .send()
                    .await
                {
                    Ok(r) => println!("a burrow updated.{:?}", r),
                    Err(e) => println!("update burrow failed{:?}", e),
                }
            }
            PulsarSearchData::UpdatePost(post) => {
                // TODO: read from typesense first, check the time if it is newer than the one in typesense
                let data: TypesensePostData = post.into();
                let uri: String = format!("/collections/posts/documents/{}", data.id);
                match client
                    .build_patch(&uri)
                    .body(serde_json::to_string(&data).unwrap())
                    .send()
                    .await
                {
                    Ok(r) => println!("update post.{:?}", r),
                    Err(e) => println!("update post failed {:?}", e),
                }
            }
            PulsarSearchData::UpdateReply(reply) => {
                // TODO: read from typesense first, check the time if it is newer than the one in typesense
                let data: TypesenseReplyData = reply.into();
                let uri: String = format!("/collections/replies/documents/{}", data.id);
                match client
                    .build_patch(&uri)
                    .body(serde_json::to_string(&data).unwrap())
                    .send()
                    .await
                {
                    Ok(r) => println!("update reply.{:?}", r),
                    Err(e) => println!("update reply failed {:?}", e),
                }
            }
            PulsarSearchData::DeleteBurrow(burrow_id) => {
                let uri: String = format!("/collections/burrows/documents/{}", burrow_id);
                match client.delete(&uri).send().await {
                    Ok(r) => println!("a burrow deleted.{:?}", r),
                    Err(e) => println!("delete burrow failed{:?}", e),
                }
            }

            PulsarSearchData::DeletePost(post_id) => {
                let uri: String = format!("/collections/posts/documents/{}", post_id);
                match client.delete(&uri).send().await {
                    Ok(r) => println!("a post deleted.{:?}", r),
                    Err(e) => println!("delete post failed{:?}", e),
                }
            }
            PulsarSearchData::DeleteReply(reply_id) => {
                let uri: String = format!("/collections/replies/documents/{}", reply_id);
                match client.delete(&uri).send().await {
                    Ok(r) => println!("a reply deleted.{:?}", r),
                    Err(e) => println!("delete reply failed{:?}", e),
                }
            }
        }
        // sleep(Duration::from_millis(10000)).await;
        // println!("10000ms have elapsed");
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
