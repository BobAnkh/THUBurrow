use pulsar::{producer, Error as PulsarError};
use pulsar::{DeserializeMessage, Payload, SerializeMessage};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

/// Json format for PulsarSearchData.data:

///     create burrow:
///             {
///                 "burrow_id": i64,
///                 "title": string,
///                 "introduction": string,
///             }
///
///     create post:
///             {
///                 "post_id": i64,
///                 "title": string,
///                 "burrow_id": i64,
///                 "section": string[],
///                 "tag": string[],
///                 "post_type": int32,
///                 "post_state": int32,
///             }
///
///     create reply:
///             {
///                 "reply_id": i64
///                 "post_id": i64,
///                 "burrow_id": i64,
///                 "content": string,
///                 "reply_state": int32,
///             }
///
///     update burrow:
///             {
///                 "burrow_id": i64,
///                 "title": string,
///                 "introduction": string,
///             }
///
///     update post:
///             {
///                 "post_id": i64,
///                 "title": string,
///                 "burrow_id": i64,
///                 "section": string[],
///                 "tag": string[],
///             }
///     update reply:
///             {
///                 "reply_id": i64
///                 "post_id": i64,
///                 "burrow_id": i64,
///                 "content": string,
///                 "reply_state": int32,
///             }
///
///     delete burrow:
///             {
///                 "burrow_id": i64
///             }
///
///     delete post:
///             {
///                 "post_id": i64,
///             }
///
///     delete reply:
///             {
///                 "reply_id": i64,
///             }
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
    DeleteReply(i64),
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
pub struct PulsarSearchBurrowData {
    pub burrow_id: i64,
    pub title: String,
    pub introduction: String,
    pub update_time: DateTimeWithTimeZone,
}

#[derive(Serialize, Deserialize)]
pub struct PulsarSearchPostData {
    pub post_id: i64,
    pub title: String,
    pub burrow_id: i64,
    pub section: Vec<String>,
    pub tag: Vec<String>,
    pub post_type: i32,
    pub post_state: i32,
    pub update_time: DateTimeWithTimeZone,
}

#[derive(Serialize, Deserialize)]
pub struct PulsarSearchReplyData {
    pub reply_id: i64,
    pub post_id: i64,
    pub burrow_id: i64,
    pub content: String,
    pub reply_state: i32,
    pub update_time: DateTimeWithTimeZone,
}

#[derive(Serialize, Deserialize)]
pub struct TypesenseBurrowData {
    pub id: i64,
    pub title: String,
    pub introduction: String,
    pub update_time: String,
}

#[derive(Serialize, Deserialize)]
pub struct TypesensePostData {
    pub id: i64,
    pub title: String,
    pub burrow_id: i64,
    pub update_time: String,
    pub post_type: i32,
    pub post_state: i32,
    pub section: Vec<String>,
    pub tag: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TypesenseReplyData {
    pub id: i64,
    pub post_id: i64,
    pub burrow_id: i64,
    pub content: String,
    pub update_time: String,
    pub reply_state: i32,
}

impl From<PulsarSearchBurrowData> for TypesenseBurrowData {
    fn from(burrow: PulsarSearchBurrowData) -> TypesenseBurrowData {
        TypesenseBurrowData {
            id: burrow.burrow_id,
            title: burrow.title,
            introduction: burrow.introduction,
            update_time: burrow.update_time.to_rfc3339(),
        }
    }
}

impl From<&PulsarSearchBurrowData> for TypesenseBurrowData {
    fn from(burrow: &PulsarSearchBurrowData) -> TypesenseBurrowData {
        TypesenseBurrowData {
            id: burrow.burrow_id,
            title: burrow.title.to_owned(),
            introduction: burrow.introduction.to_owned(),
            update_time: burrow.update_time.to_rfc3339(),
        }
    }
}

impl From<PulsarSearchPostData> for TypesensePostData {
    fn from(post: PulsarSearchPostData) -> TypesensePostData {
        TypesensePostData {
            id: post.post_id,
            title: post.title,
            burrow_id: post.burrow_id,
            update_time: post.update_time.to_rfc3339(),
            post_type: post.post_type,
            post_state: post.post_state,
            section: post.section,
            tag: post.tag,
        }
    }
}

impl From<&PulsarSearchPostData> for TypesensePostData {
    fn from(post: &PulsarSearchPostData) -> TypesensePostData {
        TypesensePostData {
            id: post.post_id,
            title: post.title.to_owned(),
            burrow_id: post.burrow_id,
            update_time: post.update_time.to_rfc3339(),
            post_type: post.post_type,
            post_state: post.post_state,
            section: post.section.to_owned(),
            tag: post.tag.to_owned(),
        }
    }
}

impl From<PulsarSearchReplyData> for TypesenseReplyData {
    fn from(reply: PulsarSearchReplyData) -> TypesenseReplyData {
        TypesenseReplyData {
            id: reply.reply_id,
            post_id: reply.post_id,
            burrow_id: reply.burrow_id,
            content: reply.content,
            update_time: reply.update_time.to_rfc3339(),
            reply_state: reply.reply_state,
        }
    }
}

impl From<&PulsarSearchReplyData> for TypesenseReplyData {
    fn from(reply: &PulsarSearchReplyData) -> TypesenseReplyData {
        TypesenseReplyData {
            id: reply.reply_id,
            post_id: reply.post_id,
            burrow_id: reply.burrow_id,
            content: reply.content.to_owned(),
            update_time: reply.update_time.to_rfc3339(),
            reply_state: reply.reply_state,
        }
    }
}

/// `{"ActivateLike":10}`, where 10 is the post_id or uid_followed
#[derive(Serialize, Deserialize)]
pub enum RelationData {
    ActivateLike(i64),
    DeactivateLike(i64),
    ActivateCollection(i64),
    DeactivateCollection(i64),
    ActivateFollow(i64),
    DeactivateFollow(i64),
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
            RelationData::ActivateFollow(uid_followed) => {
                PulsarRelationData::ActivateFollow(uid, *uid_followed)
            }
            RelationData::DeactivateFollow(uid_followed) => {
                PulsarRelationData::DeactivateFollow(uid, *uid_followed)
            }
        }
    }
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
