use pulsar::{producer, Error as PulsarError};
use pulsar::{DeserializeMessage, Payload, SerializeMessage};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum SearchOperationType {
    New,
    Remove,
    Update,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SearchContentType {
    Burrow,
    Post,
    Reply,
}

#[derive(Serialize, Deserialize)]
pub struct PulsarSearchData {
    pub operation_type: SearchOperationType, //("new","remove", "update“)
    pub content_type: SearchContentType,     //("burrow","post", "reply")
    pub operation_time: DateTimeWithTimeZone,
    pub data: serde_json::Value,
    // Json format for PulsarData.data:

    //     new burrow:
    //             {
    //                 "burrow_id": i64,
    //                 "title": string,
    //                 "introduction": string,
    //             }
    //     new post:
    //             {
    //                 "post_id": i64,
    //                 "title": string,
    //                 "burrow_id": i64,
    //                 "section": string[],
    //                 "tags": string[],
    //                 "post_type": int32
    //             }
    //     new reply:
    //             {
    //                 "reply_id": i64
    //                 "post_id": i64,
    //                 "content": string
    //             }
    //     update burrow:
    //             {
    //                 "burrow_id": i64,
    //                 "title": string,
    //                 "introduction": string,
    //             }
    //     update post:
    //             {
    //                 "post_id": i64,
    //                 "tags": string[],
    //             }
    //     remove burrow:
    //             {
    //                 "burrow_id": i64
    //             }
    //     remove post:
    //             {
    //                 "post_id": i64,
    //             }
    //     remove reply:
    //             {
    //                 "reply_id": i64,
    //             }
}

impl SerializeMessage for PulsarSearchData {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

impl DeserializeMessage for PulsarSearchData {
    type Output = Result<PulsarSearchData, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub found:i64,
    pub hits: Vec<serde_json::Value>
}
impl SerializeMessage for SearchResult {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

impl DeserializeMessage for SearchResult {
    type Output = Result<PulsarSearchData, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}
