use pulsar::SerializeMessage;
use pulsar::{producer, Error as PulsarError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TestData {
    pub data: String,
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
