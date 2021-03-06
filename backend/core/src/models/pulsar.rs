//! Models of message passed to pulsar

use pulsar::{producer, DeserializeMessage, Error as PulsarError, Payload, SerializeMessage};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

use super::content::PostSection;
use super::search::*;

/// Content operation sent to search engine
///
/// ## Fields
///
/// - `PulsarSearchData::CreateBurrow`: Create a new burrow with struct PulsarSearchBurrowData
/// - `PulsarSearchData::UpdateBurrow`: Update a burrow with struct PulsarSearchBurrowData
/// - `PulsarSearchData::DeleteBurrow`: Delete a burrow with burrow id in i64
/// - `PulsarSearchData::CreatePost`: Create a new post with struct PulsarSearchPostData
/// - `PulsarSearchData::UpdatePost`: Update a post with struct PulsarSearchPostData
/// - `PulsarSearchData::DeletePost`: Delete a burrow with post id in i64
/// - `PulsarSearchData::CreateReply`: Create a new reply with struct PulsarSearchReplyData
/// - `PulsarSearchData::UpdateReply`: Update a reply with struct PulsarSearchReplyData
/// - `PulsarSearchData::DeleteReply`: Delete a reply with (post_id, reply_id) in (i64, i32)
///
#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

/// Burrow data sent to search engine
///
/// ## Fields
///
/// - `burrow_id`: Burrow id in i64
/// - `title`: Burrow title in String
/// - `description`: Burrow description in String
/// - `update_time`: Update time in DataTimeWithTimeZone struct
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PulsarSearchBurrowData {
    pub burrow_id: i64,
    pub title: String,
    pub description: String,
    pub update_time: DateTimeWithTimeZone,
}

/// Post data sent to search engine
///
/// ## Fields
///
/// - `post_id`: Post id in i64
/// - `burrow_id`: i64 id of burrow to which the post belongs
/// - `title`: Post title in String
/// - `section`: vector of Postsection
/// - `tag`: vector of tag in String
/// - `update_time`: Update time in DataTimeWithTimeZone struct
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PulsarSearchPostData {
    pub post_id: i64,
    pub title: String,
    pub burrow_id: i64,
    pub section: Vec<PostSection>,
    pub tag: Vec<String>,
    pub update_time: DateTimeWithTimeZone,
}

/// Reply data sent to search engine
///
/// ## Fields
///
/// - `reply_id`: Reply id in i64
/// - `post_id`: i64 id of post to which the reply belongs
/// - `burrow_id`: i64 id of burrow to which the post belongs
/// - `content`: Reply content in String
/// - `update_time`: Update time in DataTimeWithTimeZone struct
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PulsarSearchReplyData {
    pub reply_id: i32,
    pub post_id: i64,
    pub burrow_id: i64,
    pub content: String,
    pub update_time: DateTimeWithTimeZone,
}

/// User relation operation sent to task executor
///
/// Example of request of frontend: `{"ActivateLike":10}` or `{"DeactivateFollow": 10}`, where 10 is the post_id or burrow_id
///
/// ## Fields
///
/// - `RelationData::ActivateLike`: activate like
/// - `RelationData::DeactivateLike`: deactivate like
/// - `RelationData::ActivateCollection`: activate collection
/// - `RelationData::DeactivateCollection`: deactivate collection
/// - `RelationData::ActivateFollow`: activate follow
/// - `RelationData::DeactivateFollow`: deactivate follow
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum RelationData {
    ActivateLike(i64),
    DeactivateLike(i64),
    ActivateCollection(i64),
    DeactivateCollection(i64),
    ActivateFollow(i64),
    DeactivateFollow(i64),
}

/// Wrap RelationData with uid
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum PulsarRelationData {
    ActivateLike(i64, i64),
    DeactivateLike(i64, i64),
    ActivateCollection(i64, i64),
    DeactivateCollection(i64, i64),
    ActivateFollow(i64, i64),
    DeactivateFollow(i64, i64),
}

/// Email operation sent to task executor
///
/// ## Fields
///
/// - `PulsarSendEmail::Sign`: send sign-up email
/// - `PulsarSendEmail::Reset`: send reset password email
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PulsarSendEmail {
    Sign { email: String },
    Reset { email: String },
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, Utc};

    #[test]
    fn test_into_typesense_burrow_data() {
        let burrow_id = 100;
        let title = "test_title".to_string();
        let description = "This is for test.".to_string();
        let update_time = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let data: PulsarSearchBurrowData = PulsarSearchBurrowData {
            burrow_id,
            title: title.clone(),
            description: description.clone(),
            update_time,
        };
        let ref_data = &data;
        let ref_into_data: TypesenseBurrowData = ref_data.into();
        let into_data: TypesenseBurrowData = data.into();
        let target = TypesenseBurrowData {
            id: burrow_id.to_string(),
            burrow_id,
            title,
            description,
            update_time,
        };
        assert_eq!(into_data, target);
        assert_eq!(ref_into_data, target);
    }

    #[test]
    fn test_into_pulsar_search_burrow_data() {
        let burrow_id = 100;
        let title = "test_title".to_string();
        let description = "This is for test.".to_string();
        let update_time = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let data = TypesenseBurrowData {
            id: burrow_id.to_string(),
            burrow_id,
            title: title.clone(),
            description: description.clone(),
            update_time,
        };
        let ref_data = &data;
        let ref_into_data: PulsarSearchBurrowData = ref_data.into();
        let into_data: PulsarSearchBurrowData = data.into();
        let target = PulsarSearchBurrowData {
            burrow_id,
            title,
            description,
            update_time,
        };
        assert_eq!(into_data, target);
        assert_eq!(ref_into_data, target);
    }

    #[test]
    fn test_into_typesense_post_data() {
        let post_id = 100;
        let title = "test_title".to_string();
        let burrow_id = 100;
        let update_time = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let section = vec![PostSection::NSFW, PostSection::Learning];
        let tag = vec!["test".to_string()];
        let data = PulsarSearchPostData {
            post_id,
            title: title.clone(),
            burrow_id,
            section: section.clone(),
            tag: tag.clone(),
            update_time,
        };
        let ref_data = &data;
        let ref_into_data: TypesensePostData = ref_data.into();
        let into_data: TypesensePostData = data.into();
        let target = TypesensePostData {
            id: post_id.to_string(),
            post_id,
            title,
            burrow_id,
            update_time,
            section,
            tag,
        };
        assert_eq!(ref_into_data, target);
        assert_eq!(into_data, target);
    }

    #[test]
    fn test_into_pulsar_search_post_data() {
        let post_id = 100;
        let title = "test_title".to_string();
        let burrow_id = 100;
        let update_time = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let section = vec![PostSection::NSFW, PostSection::Learning];
        let tag = vec!["test".to_string()];
        let data = TypesensePostData {
            id: post_id.to_string(),
            post_id,
            title: title.clone(),
            burrow_id,
            update_time,
            section: section.clone(),
            tag: tag.clone(),
        };
        let ref_data = &data;
        let ref_into_data: PulsarSearchPostData = ref_data.into();
        let into_data: PulsarSearchPostData = data.into();
        let target = PulsarSearchPostData {
            post_id,
            title,
            burrow_id,
            section,
            tag,
            update_time,
        };
        assert_eq!(ref_into_data, target);
        assert_eq!(into_data, target);
    }

    #[test]
    fn test_into_typesense_reply_data() {
        let post_id = 100;
        let reply_id = 1;
        let burrow_id = 100;
        let content = "test_content".to_string();
        let update_time = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let data = PulsarSearchReplyData {
            reply_id,
            post_id,
            burrow_id,
            content: content.clone(),
            update_time,
        };
        let ref_data = &data;
        let ref_into_data: TypesenseReplyData = ref_data.into();
        let into_data: TypesenseReplyData = data.into();
        let target = TypesenseReplyData {
            id: format!("{}-{}", post_id, reply_id),
            reply_id,
            post_id,
            burrow_id,
            content,
            update_time,
        };
        assert_eq!(ref_into_data, target);
        assert_eq!(into_data, target);
    }

    #[test]
    fn test_into_pulsar_search_reply_data() {
        let post_id = 100;
        let reply_id = 1;
        let burrow_id = 100;
        let content = "test_content".to_string();
        let update_time = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let data = TypesenseReplyData {
            id: format!("{}-{}", post_id, reply_id),
            reply_id,
            post_id,
            burrow_id,
            content: content.clone(),
            update_time,
        };
        let ref_data = &data;
        let ref_into_data: PulsarSearchReplyData = ref_data.into();
        let into_data: PulsarSearchReplyData = data.into();
        let target = PulsarSearchReplyData {
            reply_id,
            post_id,
            burrow_id,
            content,
            update_time,
        };
        assert_eq!(ref_into_data, target);
        assert_eq!(into_data, target);
    }

    #[test]
    fn test_to_pulsar() {
        let post_id = &100;
        let uid = 100000;
        let burrow_id = &1;
        assert_eq!(
            RelationData::ActivateLike(*post_id).to_pulsar(uid),
            PulsarRelationData::ActivateLike(uid, *post_id)
        );
        assert_eq!(
            RelationData::DeactivateLike(*post_id).to_pulsar(uid),
            PulsarRelationData::DeactivateLike(uid, *post_id)
        );
        assert_eq!(
            RelationData::ActivateCollection(*post_id).to_pulsar(uid),
            PulsarRelationData::ActivateCollection(uid, *post_id)
        );
        assert_eq!(
            RelationData::DeactivateCollection(*post_id).to_pulsar(uid),
            PulsarRelationData::DeactivateCollection(uid, *post_id)
        );
        assert_eq!(
            RelationData::ActivateFollow(*burrow_id).to_pulsar(uid),
            PulsarRelationData::ActivateFollow(uid, *burrow_id)
        );
        assert_eq!(
            RelationData::DeactivateFollow(*burrow_id).to_pulsar(uid),
            PulsarRelationData::DeactivateFollow(uid, *burrow_id)
        );
    }
}
