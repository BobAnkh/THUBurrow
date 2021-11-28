use pulsar::{producer, Error as PulsarError};
use pulsar::{DeserializeMessage, Payload, SerializeMessage};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum SearchOperationType {
    Create,
    Delete,
    Update,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SearchContentType {
    Burrow,
    Post,
    Reply,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RelationOperation {
    Activate,
    Deactivate,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RelationType {
    Like,
    Collection,
    Follow,
}

#[derive(Serialize, Deserialize)]
pub struct PulsarRelationData {
    pub relation_operation: RelationOperation,
    pub relation_type: RelationType,
}

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
///                 "reply_id": i32
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
///                 "post_type": int32,
///                 "post_state": int32,
///             }
///
///     update reply:
///             {
///                 "reply_id": i32
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
///                 "reply_id": i32,
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
    DeleteReply(i64, i32),
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
    pub reply_id: i32,
    pub post_id: i64,
    pub burrow_id: i64,
    pub content: String,
    pub reply_state: i32,
    pub update_time: DateTimeWithTimeZone,
}

#[derive(Serialize, Deserialize)]
pub struct TypesenseBurrowData {
    pub id: String,
    pub burrow_id: i64,
    pub title: String,
    pub introduction: String,
    pub update_time: String,
}

#[derive(Serialize, Deserialize)]
pub struct TypesensePostData {
    pub id: String,
    pub post_id: i64,
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
    pub id: String,
    pub reply_id: i32,
    pub post_id: i64,
    pub burrow_id: i64,
    pub content: String,
    pub update_time: String,
    pub reply_state: i32,
}

impl From<PulsarSearchBurrowData> for TypesenseBurrowData {
    fn from(burrow: PulsarSearchBurrowData) -> TypesenseBurrowData {
        TypesenseBurrowData {
            id: burrow.burrow_id.to_string(),
            burrow_id: burrow.burrow_id,
            title: burrow.title,
            introduction: burrow.introduction,
            update_time: burrow.update_time.to_rfc3339(),
        }
    }
}

impl From<&PulsarSearchBurrowData> for TypesenseBurrowData {
    fn from(burrow: &PulsarSearchBurrowData) -> TypesenseBurrowData {
        TypesenseBurrowData {
            id: burrow.burrow_id.to_string(),
            burrow_id: burrow.burrow_id,
            title: burrow.title.to_owned(),
            introduction: burrow.introduction.to_owned(),
            update_time: burrow.update_time.to_rfc3339(),
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
            id: post.post_id.to_string(),
            post_id: post.post_id,
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
            id: format!("{}={}", reply.post_id, reply.reply_id),
            reply_id: reply.reply_id,
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
            id: format!("{}={}", reply.post_id, reply.reply_id),
            reply_id: reply.reply_id,
            post_id: reply.post_id,
            burrow_id: reply.burrow_id,
            content: reply.content.to_owned(),
            update_time: reply.update_time.to_rfc3339(),
            reply_state: reply.reply_state,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SearchRequest2 {
    pub keyword: Option<String>,
    pub id: Option<usize>,
    pub tag: Option<String>,
    pub order: Option<String>,
    pub area: Option<String>,
    pub page: Option<i16>,
    pub filter_by: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub enum SearchRequest {
    SearchBurrowKeyword { keyword: String, page: usize },
    RetrieveBurrow { burrow_id: i64 },
    SearchPostKeyword { keyword: String, page: usize },
    SearchPostTag { tag: String, page: usize },
    RetrievePost { post_id: i64 },
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub found: i64,
    pub hits: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct GroupedSearchResult {
    pub found: i64,
    pub grouped_hits: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Keyword {
    pub keyword: String,
}
