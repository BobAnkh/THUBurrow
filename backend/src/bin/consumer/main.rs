extern crate serde;
use backend::req::pulsar::*;
use futures::TryStreamExt;
use lazy_static::lazy_static;
use pulsar::{Consumer, Pulsar, SubType, TokioExecutor};
use serde_json::json;
use std::env;
use tokio::time::sleep;
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
        let res = client.build_post("/collections").json(&each).send().await?;
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
