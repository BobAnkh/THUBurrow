use pulsar::{producer, Error as PulsarError};
use pulsar::{DeserializeMessage, Payload, SerializeMessage};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

use super::content::PostSection;
use super::search::*;

#[derive(Serialize, Deserialize)]
pub enum PulsarSearchData {
    CreateBurrow(PulsarSearchBurrowData),
    UpdateBurrow(PulsarSearchBurrowData),
    DeleteBurrow(i64),
    CreatePost(PulsarSearchPostData),
    UpdatePost(PulsarSearchPostData),
    DeletePost(i64),
    CreateReply(PulsarSearchReplyData),
    UpdateReply(PulsarSearchReplyData),
    DeleteReply(i64, i32),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PulsarSearchBurrowData {
    pub burrow_id: i64,
    pub title: String,
    pub description: String,
    pub update_time: DateTimeWithTimeZone,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PulsarSearchPostData {
    pub post_id: i64,
    pub title: String,
    pub burrow_id: i64,
    pub section: Vec<PostSection>,
    pub tag: Vec<String>,
    pub update_time: DateTimeWithTimeZone,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PulsarSearchReplyData {
    pub reply_id: i32,
    pub post_id: i64,
    pub burrow_id: i64,
    pub content: String,
    pub update_time: DateTimeWithTimeZone,
}

/// `{"ActivateLike":10}` or `{"DeactivateFollow": 10}`, where 10 is the post_id or burrow_id
#[derive(Serialize, Deserialize, Debug)]
pub enum RelationData {
    ActivateLike(i64),
    DeactivateLike(i64),
    ActivateCollection(i64),
    DeactivateCollection(i64),
    ActivateFollow(i64),
    DeactivateFollow(i64),
}

#[derive(Serialize, Deserialize)]
pub enum PulsarRelationData {
    ActivateLike(i64, i64),
    DeactivateLike(i64, i64),
    ActivateCollection(i64, i64),
    DeactivateCollection(i64, i64),
    ActivateFollow(i64, i64),
    DeactivateFollow(i64, i64),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PulsarSendEmail {
    pub email: String,
}

impl RelationData {
    pub fn to_pulsar(&self, uid: i64) -> PulsarRelationData {
        match self {
            RelationData::ActivateLike(post_id) => PulsarRelationData::ActivateLike(uid, *post_id),
            RelationData::DeactivateLike(post_id) => {
                PulsarRelationData::DeactivateLike(uid, *post_id)
            }
            RelationData::ActivateCollection(post_id) => {
                PulsarRelationData::ActivateCollection(uid, *post_id)
            }
            RelationData::DeactivateCollection(post_id) => {
                PulsarRelationData::DeactivateCollection(uid, *post_id)
            }
            RelationData::ActivateFollow(burrow_id) => {
                PulsarRelationData::ActivateFollow(uid, *burrow_id)
            }
            RelationData::DeactivateFollow(burrow_id) => {
                PulsarRelationData::DeactivateFollow(uid, *burrow_id)
            }
        }
    }
}

impl SerializeMessage for PulsarRelationData {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

impl DeserializeMessage for PulsarRelationData {
    type Output = Result<PulsarRelationData, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
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

impl SerializeMessage for PulsarSendEmail {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

impl DeserializeMessage for PulsarSendEmail {
    type Output = Result<PulsarSendEmail, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}

impl From<PulsarSearchBurrowData> for TypesenseBurrowData {
    fn from(burrow: PulsarSearchBurrowData) -> TypesenseBurrowData {
        TypesenseBurrowData {
            id: burrow.burrow_id.to_string(),
            burrow_id: burrow.burrow_id,
            title: burrow.title,
            description: burrow.description,
            update_time: burrow.update_time,
        }
    }
}

impl From<&PulsarSearchBurrowData> for TypesenseBurrowData {
    fn from(burrow: &PulsarSearchBurrowData) -> TypesenseBurrowData {
        TypesenseBurrowData {
            id: burrow.burrow_id.to_string(),
            burrow_id: burrow.burrow_id,
            title: burrow.title.to_owned(),
            description: burrow.description.to_owned(),
            update_time: burrow.update_time.to_owned(),
        }
    }
}

impl From<TypesenseBurrowData> for PulsarSearchBurrowData {
    fn from(burrow: TypesenseBurrowData) -> PulsarSearchBurrowData {
        PulsarSearchBurrowData {
            burrow_id: burrow.burrow_id,
            title: burrow.title,
            description: burrow.description,
            update_time: burrow.update_time,
        }
    }
}

impl From<&TypesenseBurrowData> for PulsarSearchBurrowData {
    fn from(burrow: &TypesenseBurrowData) -> PulsarSearchBurrowData {
        PulsarSearchBurrowData {
            burrow_id: burrow.burrow_id,
            title: burrow.title.to_owned(),
            description: burrow.description.to_owned(),
            update_time: burrow.update_time.to_owned(),
        }
    }
}

impl From<PulsarSearchPostData> for TypesensePostData {
    fn from(post: PulsarSearchPostData) -> TypesensePostData {
        TypesensePostData {
            id: post.post_id.to_string(),
            post_id: post.post_id,
            title: post.title,
            burrow_id: post.burrow_id,
            update_time: post.update_time,
            section: post.section,
            tag: post.tag,
        }
    }
}

impl From<&PulsarSearchPostData> for TypesensePostData {
    fn from(post: &PulsarSearchPostData) -> TypesensePostData {
        TypesensePostData {
            id: post.post_id.to_string(),
            post_id: post.post_id,
            title: post.title.to_owned(),
            burrow_id: post.burrow_id,
            update_time: post.update_time.to_owned(),
            section: post.section.to_owned(),
            tag: post.tag.to_owned(),
        }
    }
}

impl From<TypesensePostData> for PulsarSearchPostData {
    fn from(post: TypesensePostData) -> PulsarSearchPostData {
        PulsarSearchPostData {
            post_id: post.post_id,
            title: post.title,
            burrow_id: post.burrow_id,
            update_time: post.update_time,
            section: post.section,
            tag: post.tag,
        }
    }
}

impl From<&TypesensePostData> for PulsarSearchPostData {
    fn from(post: &TypesensePostData) -> PulsarSearchPostData {
        PulsarSearchPostData {
            post_id: post.post_id,
            title: post.title.to_owned(),
            burrow_id: post.burrow_id,
            update_time: post.update_time.to_owned(),
            section: post.section.to_owned(),
            tag: post.tag.to_owned(),
        }
    }
}

impl From<PulsarSearchReplyData> for TypesenseReplyData {
    fn from(reply: PulsarSearchReplyData) -> TypesenseReplyData {
        TypesenseReplyData {
            id: format!("{}-{}", reply.post_id, reply.reply_id),
            reply_id: reply.reply_id,
            post_id: reply.post_id,
            burrow_id: reply.burrow_id,
            content: reply.content,
            update_time: reply.update_time,
        }
    }
}

impl From<&PulsarSearchReplyData> for TypesenseReplyData {
    fn from(reply: &PulsarSearchReplyData) -> TypesenseReplyData {
        TypesenseReplyData {
            id: format!("{}-{}", reply.post_id, reply.reply_id),
            reply_id: reply.reply_id,
            post_id: reply.post_id,
            burrow_id: reply.burrow_id,
            content: reply.content.to_owned(),
            update_time: reply.update_time.to_owned(),
        }
    }
}

impl From<TypesenseReplyData> for PulsarSearchReplyData {
    fn from(reply: TypesenseReplyData) -> PulsarSearchReplyData {
        PulsarSearchReplyData {
            reply_id: reply.reply_id,
            post_id: reply.post_id,
            burrow_id: reply.burrow_id,
            content: reply.content,
            update_time: reply.update_time,
        }
    }
}

impl From<&TypesenseReplyData> for PulsarSearchReplyData {
    fn from(reply: &TypesenseReplyData) -> PulsarSearchReplyData {
        PulsarSearchReplyData {
            reply_id: reply.reply_id,
            post_id: reply.post_id,
            burrow_id: reply.burrow_id,
            content: reply.content.to_owned(),
            update_time: reply.update_time.to_owned(),
        }
    }
}
