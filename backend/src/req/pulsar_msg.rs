use pulsar::{producer, Error as PulsarError};
use pulsar::{DeserializeMessage, Payload, SerializeMessage};
use sea_orm::prelude::DateTimeWithTimeZone;
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
pub struct PulsarData {
    pub operation_type: OperationType,   //("new","remove", "update“)
    pub operation_level: OperationLevel, //("burrow","post", "reply")
    pub operation_time: DateTimeWithTimeZone,
    pub data: serde_json::Value,
    // Json format for PulsarData.data:

    //     new burrow:
    //             {
    //                 "id": i64,
    //                 "title": string,
    //                 "introduction": string,
    //             }
    //     new post:
    //             {
    //                 "id": i64,
    //                 "title": string,
    //                 "burrow_id": i64,
    //                 "tags": string[],
    //                 "post_type": int32
    //             }
    //     new reply:
    //             {
    //                 "id": i64
    //                 "post_id": i64,
    //                 "content": string
    //             }
    //     update burrow:
    //             {
    //                 "id": i64,
    //                 "title": string,
    //                 "introduction": string,
    //             }
    //     update post:
    //             {
    //                 "id": i64,
    //                 "tags": string[],
    //             }
    //     remove burrow:
    //             {
    //                 "id": i64
    //             }
    //     remove post:
    //             {
    //                 "id": i64,
    //             }
    //     remove reply:
    //             {
    //                 "id": i64,
    //             }

}

impl SerializeMessage for PulsarData {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

impl DeserializeMessage for PulsarData {
    type Output = Result<PulsarData, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}
