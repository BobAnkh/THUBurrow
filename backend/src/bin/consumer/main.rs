#[macro_use]
extern crate serde;
use futures::TryStreamExt;
use futures::io::Read;
use pulsar::{
    Authentication, Consumer, DeserializeMessage, Payload, Pulsar, SubType, TokioExecutor,
};
use rocket::figment::Error;
use std::env;
use tokio::time::sleep;
use tokio::time::Duration;
use reqwest;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct TestData {
    operation_type: String,     //("new","remove", "update“)
    operation_level: String,    //("burrow","post", "reply")
    index: i64,
    operation_time: i64,
    data: String,

    #[serde(default = "default_reply_to_whom")]
    reply_to_whom: i32

}
//set default value
fn default_reply_to_whom() -> i32{-1}

impl DeserializeMessage for TestData {
    type Output = Result<TestData, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}

async fn initialize_typesense() -> Result<reqwest::Client, reqwest::Error> {
        //initialize typesense
        let client = reqwest::Client::new();
        let res = client.post("http://localhost:8108/collections")
        .header("Content-Type", "application/json")
        .header("X-TYPESENSE-API-KEY", "xyz")
        .body(r#"{
            "name": "burrows",
            "fields": [
              {"name": "title", "type": "string" },
              {"name": "index", "type": "int32"},  
            ],
            "default_sorting_field": "title"
          }"#)
        .send()
        .await?;
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

    let mut builder = Pulsar::builder(addr, TokioExecutor);

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

    let mut counter = 0usize;

    let client = match initialize_typesense().await {
        Ok(client) => {
            println!("typesense succesfully");
            client
        },
        Err(e) => {
            println!("initialze_typesense failed: {:?}", e);
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
        println!("Consumer receive: {} operation to {} NO.{} at time{}, data:{}",
        data.operation_type, data.operation_level, data.index, data.operation_time, data.data);
        // if data.data.as_str() != "data" {
        //     println!("Unexpected payload: {}", &data.data);
        //     break;
        // }
        counter += 1;
        println!("got {} messages", counter);
        // sleep(Duration::from_millis(1000)).await;
        // println!("1000ms have elapsed");

        //post to typesense
        match client.post("http://localhost:8108/collections/burrows/documents")
        .header("Content-Type", "application/json")
        .header("X-TYPESENSE-API-KEY", "xyz")
        .body(r#"{
            "title": "First Burrow, motherfucker!",
            "index":1
        }"#)
        .send()
        .await {
            Ok(a) => println!("add new burrow.{:?}",a),
            Err(e) => println!("post new burrow failed {:?}", e)
        }
    }
    Ok(())
}
