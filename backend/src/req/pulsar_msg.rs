use pulsar::{producer, Error as PulsarError};
use pulsar::{DeserializeMessage, Payload, SerializeMessage};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum OperationType {
    New,
    Remove,
    Update,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OperationLevel {
    Burrow,
    Post,
    Reply,
}

#[derive(Serialize, Deserialize)]
pub struct TestData {
    pub operation_type: OperationType,   //("new","remove", "update“)
    pub operation_level: OperationLevel, //("burrow","post", "reply")
    pub operation_time: i64,
    pub data: serde_json::Value,
    /*
    Json format for Testdata.data:

        new burrow:
                {
                    "id": i64,
                    "title": string,
                    "introduction": string,
                    "owner_id": i32
                }
        new post:
                {
                    "id": i64,
                    "burrow_id": i64,
                    "owner_id": i32,
                    "tags": string[]
                }
        new reply:
                {
                    "id": i64
                    "post_id": i64,
                    "owner_id": i32,
                    "to_whom": i32,
                    "content": string
                }

    */
}

impl SerializeMessage for TestData {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

impl DeserializeMessage for TestData {
    type Output = Result<TestData, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}
