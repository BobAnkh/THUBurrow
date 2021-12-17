use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

use super::content::PostSection;
use super::pulsar::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct TypesenseBurrowData {
    pub id: String,
    pub burrow_id: i64,
    pub title: String,
    pub description: String,
    pub update_time: DateTimeWithTimeZone,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TypesensePostData {
    pub id: String,
    pub post_id: i64,
    pub burrow_id: i64,
    pub title: String,
    pub section: Vec<PostSection>,
    pub tag: Vec<String>,
    pub update_time: DateTimeWithTimeZone,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TypesenseReplyData {
    pub id: String,
    pub post_id: i64,
    pub reply_id: i32,
    pub burrow_id: i64,
    pub content: String,
    pub update_time: DateTimeWithTimeZone,
}

#[derive(Serialize, Deserialize)]
pub enum SearchRequest {
    RetrieveBurrow { burrow_id: i64 },
    RetrievePost { post_id: i64 },
    SearchBurrowKeyword { keywords: Vec<String> },
    SearchPostKeyword { keywords: Vec<String> },
    SearchPostTag { tag: Vec<String> },
}

#[derive(Serialize, Deserialize)]
pub struct SearchBurrowResponse {
    pub found: i32,
    pub page: usize,
    pub burrows: Vec<PulsarSearchBurrowData>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchBurrowData {
    pub found: i32,
    pub page: usize,
    pub hits: Vec<SearchBurrowHit>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchBurrowHit {
    pub highlights: Vec<SearchHighlight>,
    pub document: TypesenseBurrowData,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchHighlight {
    pub field: String,
    pub snippet: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchMixResponse {
    pub posts: SearchPostResponse,
    pub replies: SearchReplyResponse,
}

#[derive(Serialize, Deserialize)]
pub struct SearchMixResult {
    pub results: (SearchPostData, SearchReplyData),
}

#[derive(Serialize, Deserialize)]
pub struct SearchPostResponse {
    pub found: i32,
    pub page: usize,
    pub posts: Vec<PulsarSearchPostData>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchPostData {
    pub found: i32,
    pub page: usize,
    pub hits: Vec<SearchPostHit>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchPostHit {
    pub highlights: Vec<SearchHighlight>,
    pub document: TypesensePostData,
}

#[derive(Serialize, Deserialize)]
pub struct SearchReplyResponse {
    pub found: i32,
    pub page: usize,
    pub replies: Vec<SearchReplyGroupResponse>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchReplyData {
    pub found: i32,
    pub page: usize,
    pub grouped_hits: Vec<SearchReplyGroupHit>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchReplyGroupResponse {
    pub post_id: i64,
    pub replies: Vec<PulsarSearchReplyData>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchReplyGroupHit {
    pub group_key: Vec<i64>,
    pub hits: Vec<SearchReplyHit>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchReplyHit {
    pub highlights: Vec<SearchHighlight>,
    pub document: TypesenseReplyData,
}

#[derive(Serialize, Deserialize)]
pub struct SearchParam {
    pub collection: String,
    pub q: String,
    pub query_by: String,
    pub filter_by: String,
    pub sort_by: String,
    pub group_by: String,
    pub highlight_fields: String,
}

#[derive(Serialize, Deserialize)]
pub struct MultiSearch {
    pub searches: Vec<SearchParam>,
}

impl From<SearchBurrowData> for SearchBurrowResponse {
    fn from(data: SearchBurrowData) -> SearchBurrowResponse {
        SearchBurrowResponse {
            found: data.found,
            page: data.page,
            burrows: data
                .hits
                .into_iter()
                .map(|hit| {
                    let mut document: PulsarSearchBurrowData = hit.document.into();
                    hit.highlights.iter().for_each(|f| {
                        if f.field == "title" {
                            document.title = f.snippet.clone() + "...";
                        } else if f.field == "description" {
                            document.description = f.snippet.clone() + "...";
                        }
                    });
                    document
                })
                .collect(),
        }
    }
}

impl From<&SearchBurrowData> for SearchBurrowResponse {
    fn from(data: &SearchBurrowData) -> SearchBurrowResponse {
        SearchBurrowResponse {
            found: data.found,
            page: data.page,
            burrows: data
                .hits
                .clone()
                .into_iter()
                .map(|hit| {
                    let mut document: PulsarSearchBurrowData = hit.document.into();
                    hit.highlights.iter().for_each(|f| {
                        if f.field == "title" {
                            document.title = f.snippet.clone() + "...";
                        } else if f.field == "description" {
                            document.description = f.snippet.clone() + "...";
                        }
                    });
                    document
                })
                .collect(),
        }
    }
}

impl From<SearchPostData> for SearchPostResponse {
    fn from(data: SearchPostData) -> SearchPostResponse {
        SearchPostResponse {
            found: data.found,
            page: data.page,
            posts: data
                .hits
                .into_iter()
                .map(|hit| {
                    let mut document: PulsarSearchPostData = hit.document.into();
                    hit.highlights.iter().for_each(|f| {
                        if f.field == "title" {
                            document.title = f.snippet.clone() + "...";
                        }
                    });
                    document
                })
                .collect(),
        }
    }
}

impl From<&SearchPostData> for SearchPostResponse {
    fn from(data: &SearchPostData) -> SearchPostResponse {
        SearchPostResponse {
            found: data.found,
            page: data.page,
            posts: data
                .hits
                .clone()
                .into_iter()
                .map(|hit| {
                    let mut document: PulsarSearchPostData = hit.document.into();
                    hit.highlights.iter().for_each(|f| {
                        if f.field == "title" {
                            document.title = f.snippet.clone() + "...";
                        }
                    });
                    document
                })
                .collect(),
        }
    }
}

impl From<SearchReplyGroupHit> for SearchReplyGroupResponse {
    fn from(data: SearchReplyGroupHit) -> SearchReplyGroupResponse {
        SearchReplyGroupResponse {
            post_id: *data.group_key.get(0).unwrap_or(&-1),
            replies: data
                .hits
                .into_iter()
                .map(|hit| {
                    let mut document: PulsarSearchReplyData = hit.document.into();
                    hit.highlights.iter().for_each(|f| {
                        if f.field == "content" {
                            document.content = f.snippet.clone() + "...";
                        }
                    });
                    document
                })
                .collect(),
        }
    }
}

impl From<&SearchReplyGroupHit> for SearchReplyGroupResponse {
    fn from(data: &SearchReplyGroupHit) -> SearchReplyGroupResponse {
        SearchReplyGroupResponse {
            post_id: *data.group_key.get(0).unwrap_or(&-1),
            replies: data
                .hits
                .clone()
                .into_iter()
                .map(|hit| {
                    let mut document: PulsarSearchReplyData = hit.document.into();
                    hit.highlights.iter().for_each(|f| {
                        if f.field == "content" {
                            document.content = f.snippet.clone() + "...";
                        }
                    });
                    document
                })
                .collect(),
        }
    }
}

impl From<SearchReplyData> for SearchReplyResponse {
    fn from(data: SearchReplyData) -> SearchReplyResponse {
        SearchReplyResponse {
            found: data.found,
            page: data.page,
            replies: data
                .grouped_hits
                .into_iter()
                .map(|hit| hit.into())
                .collect(),
        }
    }
}

impl From<&SearchReplyData> for SearchReplyResponse {
    fn from(data: &SearchReplyData) -> SearchReplyResponse {
        SearchReplyResponse {
            found: data.found,
            page: data.page,
            replies: data
                .grouped_hits
                .clone()
                .into_iter()
                .map(|hit| hit.into())
                .collect(),
        }
    }
}

impl From<SearchMixResult> for SearchMixResponse {
    fn from(data: SearchMixResult) -> SearchMixResponse {
        SearchMixResponse {
            posts: data.results.0.into(),
            replies: data.results.1.into(),
        }
    }
}

impl From<&SearchMixResult> for SearchMixResponse {
    fn from(data: &SearchMixResult) -> SearchMixResponse {
        SearchMixResponse {
            posts: data.results.0.clone().into(),
            replies: data.results.1.clone().into(),
        }
    }
}
