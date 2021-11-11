extern crate serde;
use backend::req::pulsar_msg::*;
use futures::TryStreamExt;
use log::*;
use pulsar::{Consumer, Pulsar, SubType, TokioExecutor};
use reqwest;
use serde_json::json;
use std::env;
use tokio::time::sleep;
use tokio::time::Duration;

async fn initialize_typesense() -> Result<reqwest::Client, reqwest::Error> {
    //initialize typesense tables
    let collection_burrows = json!({
      "name": "burrows",
      "fields": [
        {"name": "id", "type": "int64"},
        {"name": "title", "type": "string" },
        {"name": "introduction", "type": "string"},
        {"name": "owner_id", "type": "int32"},
        {"name": "created_time", "type": "int64"},
        {"name": "last_modified_time", "type": "int64"}
      ]
    });
    let collection_posts = json!({
      "name": "posts",
      "fields": [
        {"name": "id", "type": "int64" },
        {"name": "burrow_id", "type": "int64" },
        {"name": "owner_id", "type": "int32"},
        {"name": "created_time", "type": "int64"},
        {"name": "last_modified_time", "type": "int64"},
        {"name": "tag", "type": "string[]"}
      ]
    });
    let collection_replies = json!({
      "name": "replies",
      "fields": [
        {"name": "id", "type": "int32" },
        {"name": "post_id", "type": "int64"},
        {"name": "owner_id", "type": "int32"},
        {"name": "to_whom", "type": "int32"},
        {"name": "created_time", "type": "int64"},
        {"name": "content", "type": "string"}
      ]
    });
    let collection_tags = json!({
      "name": "tags",
      "fields": [
        {"name": "tag_name", "type": "string" },
        {"name": "posts", "type": "int64[]" },
      ]
    });
    let client = reqwest::Client::new();
    for each in [
        collection_burrows,
        collection_posts,
        collection_replies,
        collection_tags,
    ]
    .iter()
    {
        match client
            .post("http://localhost:8108/collections")
            // .header("Content-Type", "application/json")
            .header("X-TYPESENSE-API-KEY", "xyz")
            .json(&each)
            .send()
            .await
        {
            Ok(a) => info!("Initialized collection_{}, {:?}", each["names"], a),
            Err(e) => info!("Err when initialzing collection_{},{:?}", each["names"], e),
        };
    }

    Ok(client)
}
#[tokio::main]
async fn main() -> Result<(), pulsar::Error> {
    let addr = env::var("PULSAR_ADDRESS")
        .ok()
        .unwrap_or("pulsar://127.0.0.1:6650".to_string());
    let topic = env::var("PULSAR_TOPIC")
        .ok()
        .unwrap_or("persistent://public/default/search".to_string());

    let builder = Pulsar::builder(addr, TokioExecutor);

    // if let Ok(token) = env::var("PULSAR_TOKEN") {
    //     let authentication = Authentication {
    //         name: "token".to_string(),
    //         data: token.into_bytes(),
    //     };

    //     builder = builder.with_auth(authentication);
    // }

    let pulsar: Pulsar<_> = builder.build().await?;

    let mut consumer: Consumer<TestData, _> = pulsar
        .consumer()
        .with_topic(topic)
        .with_consumer_name("test_consumer")
        .with_subscription_type(SubType::Exclusive)
        .with_subscription("test_subscription")
        .build()
        .await?;

    let client = match initialize_typesense().await {
        Ok(client) => {
            info!("typesense succesfully initialize");
            client
        }
        Err(e) => {
            info!("initialze_typesense failed to initialize: {:?}", e);
            return Ok(());
        }
    };

    while let Some(msg) = consumer.try_next().await? {
        consumer.ack(&msg).await?;
        info!("metadata: {:?},id: {:?}", msg.metadata(), msg.message_id());
        let data = match msg.deserialize() {
            Ok(data) => data,
            Err(e) => {
                info!("could not deserialize message: {:?}", e);
                continue;
            }
        };
        info!(
            "Consumer receive: {:?} {:?} at time{}",
            data.operation_type, data.operation_level, data.operation_time
        );
        // if data.data.as_str() != "data" {
        //     info!("Unexpected payload: {}", &data.data);
        //     break;
        // }
        match (data.operation_type, data.operation_level) {
            (OperationType::New, OperationLevel::Burrow) => {
                let operation = json!({
                    "id":data.data["id"],
                    "title":data.data["title"],
                    "introduction":data.data["introduction"],
                    "owner_id":data.data["owner_id"],
                    "created_time":data.operation_time,
                    "last_modified_time":data.operation_time
                });

                match client
                    .post("http://localhost:8108/collections/burrows/documents")
                    .header("Content-Type", "application/json")
                    .header("X-TYPESENSE-API-KEY", "xyz")
                    .body(serde_json::to_string(&operation).unwrap())
                    .send()
                    .await
                {
                    Ok(a) => info!("add new burrow.{:?}", a),
                    Err(e) => info!("add new burrow failed {:?}", e),
                }
            }
            (OperationType::New, OperationLevel::Post) => {
                let operation = json!({
                    "id":data.data["id"],
                    "burrow_id":data.data["burrow_id"],
                    "owner_id":data.data["owner_id"],
                    "created_time":data.operation_time,
                    "last_modified_time":data.operation_time,
                    "tags":data.data["tags"]
                });

                match client
                    .post("http://localhost:8108/collections/burrows/documents")
                    .header("Content-Type", "application/json")
                    .header("X-TYPESENSE-API-KEY", "xyz")
                    .body(serde_json::to_string(&operation).unwrap())
                    .send()
                    .await
                {
                    Ok(a) => info!("add new post.{:?}", a),
                    Err(e) => info!("add new post failed {:?}", e),
                }
            }
            (OperationType::New, OperationLevel::Reply) => {
                let operation = json!({
                    "id":data.data["id"],
                    "post_id":data.data["post_id"],
                    "owner_id":data.data["owner_id"],
                    "to_whom":data.data["to_whom"],
                    "created_time":data.operation_time,
                    "content":data.data["content"]
                });

                match client
                    .post("http://localhost:8108/collections/burrows/documents")
                    .header("Content-Type", "application/json")
                    .header("X-TYPESENSE-API-KEY", "xyz")
                    .body(serde_json::to_string(&operation).unwrap())
                    .send()
                    .await
                {
                    Ok(a) => info!("add new reply.{:?}", a),
                    Err(e) => info!("add new reply failed {:?}", e),
                }
            }
            (OperationType::Update, OperationLevel::Burrow) => {
                let operation = json!({
                    "id":data.data["id"],
                    "title":data.data["title"],
                    "introduction":data.data["introduction"],
                    "last_modified_time":data.operation_time
                });
                match client
                    .patch(format!(
                        "http://localhost:8108/collections/burrows/documents/{}",
                        data.data["id"]
                    ))
                    .header("Content-Type", "application/json")
                    .header("X-TYPESENSE-API-KEY", "xyz")
                    .body(serde_json::to_string(&operation).unwrap())
                    .send()
                    .await
                {
                    Ok(a) => info!("a burrow updated.{:?}", a),
                    Err(e) => info!("update burrow failed{:?}", e),
                }
            }
            (OperationType::Update, OperationLevel::Post) => {
                json!({});
            }
            // (OperationType::Update, OperationLevel::Reply) => {
            //     json!({});
            // }
            (OperationType::Remove, OperationLevel::Burrow) => {
                json!({});
            }
            (OperationType::Remove, OperationLevel::Post) => {
                json!({});
            }
            (OperationType::Remove, OperationLevel::Reply) => {
                json!({});
            }
            _ => info!("invalid operation from pulsar")
        }
        // sleep(Duration::from_millis(10000)).await;
        // info!("10000ms have elapsed");
    }
    Ok(())
}
