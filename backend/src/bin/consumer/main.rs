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
            .unwrap_or_else(|| "http://localhost:8108".to_string())
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
        {"name": "created_time", "type": "string"},
        {"name": "last_modified_time", "type": "string"}
      ]
    });
    let collection_posts = json!({
      "name": "posts",
      "fields": [
        {"name": "id", "type": "int64" },
        {"name": "title", "type": "string" },
        {"name": "burrow_id", "type": "int64" },
        {"name": "created_time", "type": "string"},
        {"name": "last_modified_time", "type": "string"},
        {"name": "post_type", "type": "int32"},
        {"name": "tag", "type": "string[]"}
      ]
    });
    let collection_replies = json!({
      "name": "replies",
      "fields": [
        {"name": "id", "type": "int32" },
        {"name": "post_id", "type": "int64"},
        {"name": "created_time", "type": "string"},
        {"name": "content", "type": "string"}
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
    fn build_post(&self, uri: &str) -> reqwest::RequestBuilder;
    fn build_delete(&self, uri: &str) -> reqwest::RequestBuilder;
    fn build_patch(&self, uri: &str) -> reqwest::RequestBuilder;
}

impl Typesense for reqwest::Client {
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
        println!(
            "Consumer receive: {:?} {:?} at time{}",
            data.operation_type, data.operation_level, data.operation_time
        );
        // if data.data.as_str() != "data" {
        //     println!("Unexpected payload: {}", &data.data);
        //     break;
        // }
        match (data.operation_type, data.operation_level) {
            (SearchOperationType::New, SearchContentType::Burrow) => {
                // TODO: define a struct here, not using direct json
                let operation = json!({
                    "id":data.data["id"],
                    "title":data.data["title"],
                    "introduction":data.data["introduction"],
                    "created_time":data.operation_time,
                    "last_modified_time":data.operation_time
                });

                match client
                    .build_post("/collections/burrows/documents")
                    .body(serde_json::to_string(&operation).unwrap())
                    .send()
                    .await
                {
                    Ok(a) => println!("add new burrow.{:?}", a),
                    Err(e) => println!("add new burrow failed {:?}", e),
                }
            }
            (SearchOperationType::New, SearchContentType::Post) => {
                let operation = json!({
                    "id":data.data["id"],
                    "burrow_id":data.data["burrow_id"],
                    "created_time":data.operation_time,
                    "last_modified_time":data.operation_time,
                    "tags":data.data["tags"]
                });

                match client
                    .build_post("/collections/burrows/documents")
                    .body(serde_json::to_string(&operation).unwrap())
                    .send()
                    .await
                {
                    Ok(a) => println!("add new post.{:?}", a),
                    Err(e) => println!("add new post failed {:?}", e),
                }
            }
            (SearchOperationType::New, SearchContentType::Reply) => {
                let operation = json!({
                    "id":data.data["id"],
                    "post_id":data.data["post_id"],
                    "created_time":data.operation_time,
                    "content":data.data["content"]
                });

                match client
                    .post("/collections/burrows/documents")
                    .body(serde_json::to_string(&operation).unwrap())
                    .send()
                    .await
                {
                    Ok(a) => println!("add new reply.{:?}", a),
                    Err(e) => println!("add new reply failed {:?}", e),
                }
            }
            (SearchOperationType::Update, SearchContentType::Burrow) => {
                let operation = json!({
                    "id":data.data["id"],
                    "title":data.data["title"],
                    "introduction":data.data["introduction"],
                    "last_modified_time":data.operation_time
                });
                let uri: String = format!("/collections/burrows/documents/{}", data.data["id"]);
                match client
                    .build_patch(&uri)
                    .body(serde_json::to_string(&operation).unwrap())
                    .send()
                    .await
                {
                    Ok(a) => println!("a burrow updated.{:?}", a),
                    Err(e) => println!("update burrow failed{:?}", e),
                }
            }
            (SearchOperationType::Update, SearchContentType::Post) => {
                let operation = json!({
                    "id":data.data["id"],
                    "last_modified_time":data.operation_time,
                    "tags":data.data["tags"]
                });

                match client
                    .build_patch("/collections/posts/documents")
                    .body(serde_json::to_string(&operation).unwrap())
                    .send()
                    .await
                {
                    Ok(a) => println!("add new post.{:?}", a),
                    Err(e) => println!("add new post failed {:?}", e),
                }
            }
            // (OperationType::Update, OperationLevel::Reply) => {
            //     json!({});
            // }
            (SearchOperationType::Remove, SearchContentType::Burrow) => {
                let uri: String = format!("/collections/burrows/documents/{}", data.data["id"]);
                match client.delete(&uri).send().await {
                    Ok(a) => println!("a burrow deleted.{:?}", a),
                    Err(e) => println!("delete burrow failed{:?}", e),
                }
            }

            (SearchOperationType::Remove, SearchContentType::Post) => {
                let uri: String = format!("/collections/posts/documents/{}", data.data["id"]);
                match client.delete(&uri).send().await {
                    Ok(a) => println!("a post deleted.{:?}", a),
                    Err(e) => println!("delete post failed{:?}", e),
                }
            }
            (SearchOperationType::Remove, SearchContentType::Reply) => {
                let uri: String = format!("/collections/replies/documents/{}", data.data["id"]);
                match client.delete(&uri).send().await {
                    Ok(a) => println!("a reply deleted.{:?}", a),
                    Err(e) => println!("delete reply failed{:?}", e),
                }
            }
            _ => println!("invalid operation from pulsar"),
        }
        // sleep(Duration::from_millis(10000)).await;
        // println!("10000ms have elapsed");
    }

    Ok(())
}
