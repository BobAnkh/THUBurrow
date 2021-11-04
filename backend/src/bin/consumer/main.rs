#[macro_use]
extern crate serde;
use futures::TryStreamExt;
use pulsar::{
    Authentication, Consumer, DeserializeMessage, Payload, Pulsar, SubType, TokioExecutor,
};
use std::env;
use tokio::time::sleep;
use tokio::time::Duration;

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
    while let Some(msg) = consumer.try_next().await? {
        consumer.ack(&msg).await?;
        log::info!("metadata: {:?},id: {:?}", msg.metadata(), msg.message_id());
        let data = match msg.deserialize() {
            Ok(data) => data,
            Err(e) => {
                log::error!("could not deserialize message: {:?}", e);
                continue;
            }
        };
        println!("Consumer receive: {} operation to {} NO.{} at time{}, data:{}",
        data.operation_type, data.operation_level, data.index, data.operation_time, data.data);
        // if data.data.as_str() != "data" {
        //     log::error!("Unexpected payload: {}", &data.data);
        //     break;
        // }
        counter += 1;
        log::info!("got {} messages", counter);
        sleep(Duration::from_millis(1000)).await;
        println!("1000ms have elapsed");
    }
    Ok(())
}
