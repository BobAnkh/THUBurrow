extern crate serde;
use backend::req::pulsar::*;
use futures::TryStreamExt;
use lazy_static::lazy_static;
use pulsar::{Consumer, Pulsar, SubType, TokioExecutor};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde_json::json;
use std::env;
use tokio::time::Duration;

lazy_static! {
    static ref TYPESENSE_API_KEY: String = {
        env::var("TYPESENSE_API_KEY")
            .ok()
            .unwrap_or_else(|| "8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string())
    };
    static ref TYPESENSE_ADDR: String = {
        env::var("TYPESENSE_ADDR")
            .ok()
            .unwrap_or_else(|| "http://127.0.0.1:8108".to_string())
    };
}

// fn log_init() {
//     match log4rs::init_file("conf/log4rs.yml", Default::default()) {
//         Ok(_) => (),
//         Err(e) => panic!("Error initial logger: {}", e),
//     }
// }

async fn create_typesense_collections() -> Result<(), reqwest::Error> {
    //create typesense collections
    let collection_burrows = json!({
      "name": "burrows",
      "fields": [
        {"name": "id", "type": "string"},
        {"name": "burrow_id", "type": "int64"},
        {"name": "title", "type": "string" , "locale": "zh"},
        {"name": "introduction", "type": "string", "locale": "zh"},
        {"name": "update_time", "type": "string"}
      ]
    });
    let collection_posts = json!({
      "name": "posts",
      "fields": [
        {"name": "id", "type": "string" },
        {"name": "post_id", "type": "int64"},
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
        {"name": "id", "type": "string" },
        {"name": "reply_id", "type": "int32" },
        {"name": "post_id", "type": "int64", "facet":true},
        {"name": "burrow_id", "type": "int64"},
        {"name": "update_time", "type": "string"},
        {"name": "content", "type": "string"},
        {"name": "reply_state", "type": "int32"}
      ]
    });
    let client = reqwest::Client::new();
    for each in [collection_burrows, collection_posts, collection_replies].iter() {
        // TODO: match the status code of Response here, to see whether it is successfully created or is already created, or failed
        match client.build_post("/collections").json(&each).send().await {
            Ok(a) => match a.status().as_u16() {
                201 => (),
                400 => panic!(
                    "Bad Request - The request could not be understood due to malformed syntax."
                ),
                401 => panic!("Unauthorized - Your API key is wrong."),
                404 => panic!("Not Found - The requested resource is not found."),
                409 => println!("Conflict - When a resource already exists."),
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

#[tokio::main]
async fn main() {
    // log_init();
    let handles = vec![tokio::spawn(pulsar_typesense())];
    futures::future::join_all(handles).await;
    std::thread::sleep(Duration::from_millis(1000));
}

async fn pulsar_typesense() -> Result<(), pulsar::Error> {
    // setup pulsar consumer
    let addr = env::var("PULSAR_ADDRESS")
        .ok()
        .unwrap_or_else(|| "pulsar://127.0.0.1:6650".to_string());
    let topic = env::var("PULSAR_TOPIC")
        .ok()
        .unwrap_or_else(|| "persistent://public/default/search".to_string());
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
        // println!("metadata: {:?},id: {:?}", msg.metadata(), msg.message_id());
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
                    Ok(r) => match r.status().as_u16() {
                        201 => println!("Add new burrow."),
                        409 => println!("409, burrow already exist."),
                        _ => panic!(
                            "Status:{} Failed to add new burrow. {}",
                            r.status().as_u16(),
                            r.text().await.unwrap()
                        ),
                    },
                    Err(e) => println!("add new burrow failed {:?}", e),
                }
            }
            PulsarSearchData::CreatePost(post) => {
                let data: TypesensePostData = post.into();

                match client
                    .build_post("/collections/posts/documents")
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
                    .post("/collections/replies/documents")
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
                let search_result:SearchResult = serde_json::from_str(&client
                .build_get(&format!("/collections/burrows/documents/search?q={}&query_by=burrow_id&filter_by=&sort_by=",burrow.burrow_id))
                .send()
                .await.unwrap().text().await.unwrap()).unwrap();
                match search_result.found {
                    0 => println!("Burrow to update does not exist!"),
                    _ => {
                        let present_burrow: TypesenseBurrowData =
                            serde_json::from_value(search_result.hits[0].clone()).unwrap();
                        match DateTimeWithTimeZone::parse_from_rfc3339(&present_burrow.update_time)
                            .unwrap()
                            .timestamp()
                            < burrow.update_time.timestamp()
                        {
                            true => {
                                let data: TypesenseBurrowData = burrow.into();
                                let uri: String =
                                    format!("/collections/burrows/documents/{}", data.id);
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
                            false => println!("The burrow is already up to date."),
                        }
                    }
                }
            }
            PulsarSearchData::UpdatePost(post) => {
                // TODO: read from typesense first, check the time if it is newer than the one in typesense
                let search_result:SearchResult = serde_json::from_str(&client
                    .build_get(&format!("/collections/posts/documents/search?q={}&query_by=post_id&filter_by=&sort_by=",post.post_id))
                    .send()
                    .await.unwrap().text().await.unwrap()).unwrap();
                match search_result.found {
                    0 => println!("Post to update does not exist!"),
                    _ => {
                        let present_post: TypesenseBurrowData =
                            serde_json::from_value(search_result.hits[0].clone()).unwrap();
                        match DateTimeWithTimeZone::parse_from_rfc3339(&present_post.update_time)
                            .unwrap()
                            < post.update_time
                        {
                            true => {
                                let data: TypesensePostData = post.into();
                                let uri: String =
                                    format!("/collections/posts/documents/{}", data.id);
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
                            false => println!("The burrow is already up to date."),
                        }
                    }
                }
            }
            PulsarSearchData::UpdateReply(reply) => {
                // TODO: read from typesense first, check the time if it is newer than the one in typesense
                let search_result:SearchResult = serde_json::from_str(&client
                    .build_get(&format!("/collections/replies/documents/search?q={}&query_by=reply_id&filter_by=&sort_by=",reply.reply_id))
                    .send()
                    .await.unwrap().text().await.unwrap()).unwrap();
                match search_result.found {
                    0 => println!("Reply to update does not exist!"),
                    _ => {
                        let present_reply: TypesenseBurrowData =
                            serde_json::from_value(search_result.hits[0].clone()).unwrap();
                        match DateTimeWithTimeZone::parse_from_rfc3339(&present_reply.update_time)
                            .unwrap()
                            .timestamp()
                            < reply.update_time.timestamp()
                        {
                            true => {
                                let data: TypesenseReplyData = reply.into();
                                let uri: String =
                                    format!("/collections/replies/documents/{}", data.id);
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
                            false => println!("The burrow is already up to date."),
                        }
                    }
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
            PulsarSearchData::DeleteReply(post_id, reply_id) => {
                let uri: String =
                    format!("/collections/replies/documents/{}-{}", post_id, reply_id);
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