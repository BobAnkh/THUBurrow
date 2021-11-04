use pulsar::SerializeMessage;
use pulsar::{producer, Error as PulsarError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TestData {
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

impl SerializeMessage for TestData {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}
