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

// fn log_init() {
//     match log4rs::init_file("conf/log4rs.yml", Default::default()) {
//         Ok(_) => (),
//         Err(e) => panic!("Error initial logger: {}", e),
//     }
// }

async fn initialize_typesense() -> Result<reqwest::Client, reqwest::Error> {
    //initialize typesense tables
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
    for each in [
        collection_burrows,
        collection_posts,
        collection_replies
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
            Ok(a) => println!("Initialized collection_{}, {:?}", each["names"], a),
            Err(e) => println!("Err when initialzing collection_{},{:?}", each["names"], e),
        };
    }

    Ok(client)
}
#[tokio::main]
async fn main() -> Result<(), pulsar::Error> {
    // log_init();
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

    let mut consumer: Consumer<PulsarData, _> = pulsar
        .consumer()
        .with_topic(topic)
        .with_consumer_name("test_consumer")
        .with_subscription_type(SubType::Exclusive)
        .with_subscription("test_subscription")
        .build()
        .await?;

    let client = match initialize_typesense().await {
        Ok(client) => {
            println!("typesense succesfully initialize");
            client
        }
        Err(e) => {
            println!("initialze_typesense failed to initialize: {:?}", e);
            return Ok(());
        }
    };

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
            (OperationType::New, OperationLevel::Burrow) => {
                let operation = json!({
                    "id":data.data["id"],
                    "title":data.data["title"],
                    "introduction":data.data["introduction"],
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
                    Ok(a) => println!("add new burrow.{:?}", a),
                    Err(e) => println!("add new burrow failed {:?}", e),
                }
            }
            (OperationType::New, OperationLevel::Post) => {
                let operation = json!({
                    "id":data.data["id"],
                    "burrow_id":data.data["burrow_id"],
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
                    Ok(a) => println!("add new post.{:?}", a),
                    Err(e) => println!("add new post failed {:?}", e),
                }
            }
            (OperationType::New, OperationLevel::Reply) => {
                let operation = json!({
                    "id":data.data["id"],
                    "post_id":data.data["post_id"],
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
                    Ok(a) => println!("add new reply.{:?}", a),
                    Err(e) => println!("add new reply failed {:?}", e),
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
                    Ok(a) => println!("a burrow updated.{:?}", a),
                    Err(e) => println!("update burrow failed{:?}", e),
                }
            }
            (OperationType::Update, OperationLevel::Post) => {
                let operation = json!({
                    "id":data.data["id"],
                    "last_modified_time":data.operation_time,
                    "tags":data.data["tags"]
                });

                match client
                    .patch("http://localhost:8108/collections/posts/documents")
                    .header("Content-Type", "application/json")
                    .header("X-TYPESENSE-API-KEY", "xyz")
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
            (OperationType::Remove, OperationLevel::Burrow) => {
                let operation = json!({
                    "id":data.data["id"],
                });
                match client
                    .delete(format!(
                        "http://localhost:8108/collections/burrows/documents/{}",
                        data.data["id"]
                    ))
                    .header("X-TYPESENSE-API-KEY", "xyz")
                    .send()
                    .await
                {
                    Ok(a) => println!("a burrow deleted.{:?}", a),
                    Err(e) => println!("delete burrow failed{:?}", e),
                }
            }
            
            (OperationType::Remove, OperationLevel::Post) => {
                let operation = json!({
                    "id":data.data["id"],
                });
                match client
                    .delete(format!(
                        "http://localhost:8108/collections/posts/documents/{}",
                        data.data["id"]
                    ))
                    .header("X-TYPESENSE-API-KEY", "xyz")
                    .send()
                    .await
                {
                    Ok(a) => println!("a post deleted.{:?}", a),
                    Err(e) => println!("delete post failed{:?}", e),
                }
            }
            (OperationType::Remove, OperationLevel::Reply) => {
                let operation = json!({
                    "id":data.data["id"],
                });
                match client
                    .delete(format!(
                        "http://localhost:8108/collections/replies/documents/{}",
                        data.data["id"]
                    ))
                    .header("X-TYPESENSE-API-KEY", "xyz")
                    .send()
                    .await
                {
                    Ok(a) => println!("a reply deleted.{:?}", a),
                    Err(e) => println!("delete reply failed{:?}", e),
                }
            }
            _ => println!("invalid operation from pulsar")
        }
        // sleep(Duration::from_millis(10000)).await;
        // println!("10000ms have elapsed");
    }
    Ok(())
}

