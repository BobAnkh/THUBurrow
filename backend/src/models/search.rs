//! Models for search

use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

use super::content::PostSection;
use super::pulsar::*;

/// Burrow struct in typesense database
///
/// ## Fields
///
/// - `id`: Burrow id in String for searchengine index
/// - `burrow_id`: Burrow id in i64
/// - `title`: Burrow title in String
/// - `description`: Burrow description in String
/// - `update_time`: Update time in DataTimeWithTimeZone struct
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TypesenseBurrowData {
    pub id: String,
    pub burrow_id: i64,
    pub title: String,
    pub description: String,
    pub update_time: DateTimeWithTimeZone,
}

/// Post struct in typesense database
///
/// ## Fields
///
/// - `id`: Post id in String for searchengine index
/// - `post_id`: Post id in i64
/// - `burrow_id`: i64 id of burrow to which the post belongs
/// - `title`: Post title in String
/// - `section`: vector of Postsection
/// - `tag`: vector of tag in String
/// - `update_time`: Update time in DataTimeWithTimeZone struct
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TypesensePostData {
    pub id: String,
    pub post_id: i64,
    pub burrow_id: i64,
    pub title: String,
    pub section: Vec<PostSection>,
    pub tag: Vec<String>,
    pub update_time: DateTimeWithTimeZone,
}

/// Reply struct in typesense database
///
/// ## Fields
///
/// - `id`: Reply id in String for searchengine index
/// - `reply_id`: Reply id in i64
/// - `post_id`: i64 id of post to which the reply belongs
/// - `burrow_id`: i64 id of burrow to which the post belongs
/// - `content`: Reply content in String
/// - `update_time`: Update time in DataTimeWithTimeZone struct
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TypesenseReplyData {
    pub id: String,
    pub post_id: i64,
    pub reply_id: i32,
    pub burrow_id: i64,
    pub content: String,
    pub update_time: DateTimeWithTimeZone,
}

/// Different kinds of search request
///
/// ## Fields
///
/// - `SearchRequest::RetrieveBurrow`: Retrieve burrow with param `burrow_id` in i64
/// - `SearchRequest::RetrievePost`: Retrieve post with param `post_id` in i64
/// - `SearchRequest::SearchBurrowKeyword`: Search burrow with param `keywords` in vector of String
/// - `SearchRequest::SearchPostKeyword`: Search post with param `keywords` in vector of String
/// - `SearchRequest::SearchPostTag`: Search post with param `tag` in vector of String
///
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum SearchRequest {
    RetrieveBurrow { burrow_id: i64 },
    RetrievePost { post_id: i64 },
    SearchBurrowKeyword { keywords: Vec<String> },
    SearchPostKeyword { keywords: Vec<String> },
    SearchPostTag { tag: Vec<String> },
}

/// Response struct for burrow search
///
/// ## Fields
///
/// - `found`: Number of found burrow in i32
/// - `page`: Page index of result
/// - `burrows`: Vector of search result in struct PulsarSearchBurrowData
///
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SearchBurrowResponse {
    pub found: i32,
    pub page: usize,
    pub burrows: Vec<PulsarSearchBurrowData>,
}

/// Search result for burrow search
///
/// ## Fields
///
/// - `found`: Number of found burrow in i32
/// - `page`: Page index of result
/// - `hits`: Vector of search hit in struct SearchBurrowHit
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchBurrowData {
    pub found: i32,
    pub page: usize,
    pub hits: Vec<SearchBurrowHit>,
}

/// Search hit for burrow
///
/// ## Fields
///
/// - `highlight`: Vector of SeachHighlight
/// - `document`: Burrow data in struct TypesenseBurrowData
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchBurrowHit {
    pub highlights: Vec<SearchHighlight>,
    pub document: TypesenseBurrowData,
}

/// Search highlight
///
/// ## Fields
///
/// - `field`:  The field that the keyword hit in String
/// - `snippet`: A slice of context around hitword in String
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchHighlight {
    pub field: String,
    pub snippet: String,
}

/// Response struct for post/reply mix search
///
/// ## Fields
///
/// - `post`: Response for post search
/// - `replies`: Response for reply seach
///
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SearchMixResponse {
    pub posts: SearchPostResponse,
    pub replies: SearchReplyResponse,
}

/// Search result for post/reply mix search
///
/// ## Fields
///
/// - `results`: A tuple (SeachPostData, SeachReplyData)
///
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SearchMixResult {
    pub results: (SearchPostData, SearchReplyData),
}

/// Response struct for post search
///
/// ## Fields
///
/// - `found`: Number of found in i32
/// - `page`: Page index of result
/// - `burrows`: Vector of search result in struct PulsarSearchPostData
///
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SearchPostResponse {
    pub found: i32,
    pub page: usize,
    pub posts: Vec<PulsarSearchPostData>,
}

/// Search result for post search
///
/// ## Fields
///
/// - `found`: Number of found in i32
/// - `page`: Page index of result
/// - `hits`: Vector of search hit in struct SearchPostHit
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchPostData {
    pub found: i32,
    pub page: usize,
    pub hits: Vec<SearchPostHit>,
}

/// Search hit for post
///
/// ## Fields
///
/// - `highlight`: Vector of SeachHighlight
/// - `document`: Post data in struct TypesenseBurrowData
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchPostHit {
    pub highlights: Vec<SearchHighlight>,
    pub document: TypesensePostData,
}

/// Response struct for reply search
///
/// ## Fields
///
/// - `found`: Number of found in i32
/// - `page`: Page index of result
/// - `burrows`: Vector of search result in struct SearchReplyGroupResponse
///
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SearchReplyResponse {
    pub found: i32,
    pub page: usize,
    pub replies: Vec<SearchReplyGroupResponse>,
}

/// Search result for reply search
///
/// ## Fields
///
/// - `found`: Number of found in i32
/// - `page`: Page index of result
/// - `grouped_hits`: Vector of search hit in struct SearchReplyGroupHit
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchReplyData {
    pub found: i32,
    pub page: usize,
    pub grouped_hits: Vec<SearchReplyGroupHit>,
}

/// Grouped search result of reply from the same post
///
/// ## Fields
///
/// - `post_id`: Post id in i64
/// - `replies`: Vector of hit reply in struct PulsarSearchReplyData
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchReplyGroupResponse {
    pub post_id: i64,
    pub replies: Vec<PulsarSearchReplyData>,
}

/// Grouped search hit for reply
///
/// ## Fields
///
/// - `goup_key`: Vector of index in i64
/// - `hits`: Vector of hit reply in struct SearchReplyHit
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchReplyGroupHit {
    pub group_key: Vec<i64>,
    pub hits: Vec<SearchReplyHit>,
}

/// Grouped hit of single reply
///
/// ## Fields
///
/// - `highlight`: Vector of SeachHighlight
/// - `document`: Reply data in struct TypesenseReplyData
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchReplyHit {
    pub highlights: Vec<SearchHighlight>,
    pub document: TypesenseReplyData,
}

/// Search parameters
///
/// ## Fields
///
/// - `collection`: The collection to search in String, burrows/posts/replies
/// - `q`: Search keyword in String
/// - `query_by`: The field to query
/// - `filter_by`: Filter condition in String
/// - `sort_by`: Sort result condition in String
/// - `group_by`: Group condition in String
/// - `highlight_fields`: Highlight fields in String
///
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SearchParam {
    pub collection: String,
    pub q: String,
    pub query_by: String,
    pub filter_by: String,
    pub sort_by: String,
    pub group_by: String,
    pub highlight_fields: String,
}

/// A vector of search parameters
///
/// ## Fields
///
/// - `searches`: Vector of struct SearchParam
///
#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;
    use chrono::Utc;

    #[test]
    fn testfrom_searchburrowresponse() {
        let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let highlight = SearchHighlight {
            field: "test_field".to_string(),
            snippet: "test_snippet".to_string(),
        };
        let document = TypesenseBurrowData {
            id: "test_id".to_string(),
            burrow_id: 999i64,
            title: "test_title".to_string(),
            description: "test_description".to_string(),
            update_time: now,
        };
        let document2 = document.clone();
        let sbhit = SearchBurrowHit {
            highlights: vec![highlight],
            document,
        };
        let sbdata = SearchBurrowData {
            found: 999i32,
            page: 999usize,
            hits: vec![sbhit],
        };
        let sbdata2 = sbdata.clone();
        let psbdata: PulsarSearchBurrowData = document2.into();
        // let psbdata_2: PulsarSearchBurrowData = (&document3).into();
        // assert_eq!(psbdata_1, psbdata_2);
        let sbresponse = SearchBurrowResponse {
            found: 999i32,
            page: 999usize,
            burrows: vec![psbdata],
        };
        let data_to_res: SearchBurrowResponse = sbdata.into();
        let data_to_res2: SearchBurrowResponse = (&sbdata2).into();
        assert_eq!(data_to_res, sbresponse);
        assert_eq!(data_to_res2, sbresponse);
    }

    #[test]
    fn testfrom_searchpostresponse() {
        let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let highlight = SearchHighlight {
            field: "test_field".to_string(),
            snippet: "test_snippet".to_string(),
        };
        let document = TypesensePostData {
            id: "test_id".to_string(),
            burrow_id: 999i64,
            post_id: 999i64,
            title: "test_title".to_string(),
            section: vec![PostSection::Learning],
            tag: vec!["test_tag".to_string()],
            update_time: now,
        };
        let document2 = document.clone();
        let sphit = SearchPostHit {
            highlights: vec![highlight],
            document,
        };
        let spdata = SearchPostData {
            found: 999i32,
            page: 999usize,
            hits: vec![sphit],
        };
        let spdata2 = spdata.clone();
        let pspdata: PulsarSearchPostData = document2.into();
        // let psbdata_2: PulsarSearchBurrowData = (&document3).into();
        // assert_eq!(psbdata_1, psbdata_2);
        let spresponse = SearchPostResponse {
            found: 999i32,
            page: 999usize,
            posts: vec![pspdata],
        };
        let data_to_res: SearchPostResponse = spdata.into();
        let data_to_res2: SearchPostResponse = (&spdata2).into();
        assert_eq!(data_to_res, spresponse);
        assert_eq!(data_to_res2, spresponse);
    }

    #[test]
    fn testfrom_searchreplyresponse() {
        let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let highlight = SearchHighlight {
            field: "test_field".to_string(),
            snippet: "test_snippet".to_string(),
        };
        let document = TypesenseReplyData {
            id: "test_id".to_string(),
            burrow_id: 999i64,
            post_id: 999i64,
            reply_id: 999i32,
            update_time: now,
            content: "test_content".to_string(),
        };
        let document2 = document.clone();
        let srhit = SearchReplyHit {
            highlights: vec![highlight],
            document,
        };
        let grouped_hits = SearchReplyGroupHit {
            group_key: vec![999i64],
            hits: vec![srhit],
        };
        let srdata = SearchReplyData {
            found: 999i32,
            page: 999usize,
            grouped_hits: vec![grouped_hits],
        };
        let srdata2 = srdata.clone();
        let psrdata: PulsarSearchReplyData = document2.into();
        let srgresponse = SearchReplyGroupResponse {
            post_id: 999i64,
            replies: vec![psrdata],
        };
        // let psbdata_2: PulsarSearchBurrowData = (&document3).into();
        // assert_eq!(psbdata_1, psbdata_2);
        let srresponse = SearchReplyResponse {
            found: 999i32,
            page: 999usize,
            replies: vec![srgresponse],
        };
        let data_to_res: SearchReplyResponse = srdata.into();
        let data_to_res2: SearchReplyResponse = (&srdata2).into();
        assert_eq!(data_to_res, srresponse);
        assert_eq!(data_to_res2, srresponse);
    }

    #[test]
    fn testfrom_srgresponse() {
        let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let highlight = SearchHighlight {
            field: "test_field".to_string(),
            snippet: "test_snippet".to_string(),
        };
        let document = TypesenseReplyData {
            id: "test_id".to_string(),
            burrow_id: 999i64,
            post_id: 999i64,
            reply_id: 999i32,
            update_time: now,
            content: "test_content".to_string(),
        };
        let document2 = document.clone();
        let srhit = SearchReplyHit {
            highlights: vec![highlight],
            document,
        };
        let grouped_hits = SearchReplyGroupHit {
            group_key: vec![999i64],
            hits: vec![srhit],
        };
        let grouped_hits2 = grouped_hits.clone();
        // let srdata = SearchReplyData {
        //     found: 999i32,
        //     page: 999usize,
        //     grouped_hits: vec![grouped_hits],
        // };
        // let srdata2 = srdata.clone();
        let psrdata: PulsarSearchReplyData = document2.into();
        let srghit_to_srgres: SearchReplyGroupResponse = grouped_hits.into();
        let srghit_to_srgres2: SearchReplyGroupResponse = (&grouped_hits2).into();
        let srgresponse = SearchReplyGroupResponse {
            post_id: 999i64,
            replies: vec![psrdata],
        };
        assert_eq!(srghit_to_srgres, srgresponse);
        assert_eq!(srghit_to_srgres2, srgresponse);
    }

    #[test]
    fn testfrom_smresponse() {
        let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let highlight = SearchHighlight {
            field: "test_field".to_string(),
            snippet: "test_snippet".to_string(),
        };
        let highlight2 = highlight.clone();
        let rdocument = TypesenseReplyData {
            id: "test_id".to_string(),
            burrow_id: 999i64,
            post_id: 999i64,
            reply_id: 999i32,
            update_time: now,
            content: "test_content".to_string(),
        };
        let rdocument2 = rdocument.clone();
        let srhit = SearchReplyHit {
            highlights: vec![highlight],
            document: rdocument,
        };
        let grouped_hits = SearchReplyGroupHit {
            group_key: vec![999i64],
            hits: vec![srhit],
        };
        let srdata = SearchReplyData {
            found: 999i32,
            page: 999usize,
            grouped_hits: vec![grouped_hits],
        };
        let pdocument = TypesensePostData {
            id: "test_id".to_string(),
            burrow_id: 999i64,
            post_id: 999i64,
            title: "test_title".to_string(),
            section: vec![PostSection::Learning],
            tag: vec!["test_tag".to_string()],
            update_time: now,
        };
        let pdocument2 = pdocument.clone();
        let sphit = SearchPostHit {
            highlights: vec![highlight2],
            document: pdocument,
        };
        let spdata = SearchPostData {
            found: 999i32,
            page: 999usize,
            hits: vec![sphit],
        };
        let smresult = SearchMixResult {
            results: (spdata, srdata),
        };
        let smresult2 = smresult.clone();
        let psrdata: PulsarSearchReplyData = rdocument2.into();
        let pspdata: PulsarSearchPostData = pdocument2.into();
        let srgresponse = SearchReplyGroupResponse {
            post_id: 999i64,
            replies: vec![psrdata],
        };

        let srresponse = SearchReplyResponse {
            found: 999i32,
            page: 999usize,
            replies: vec![srgresponse],
        };
        let spresponse = SearchPostResponse {
            found: 999i32,
            page: 999usize,
            posts: vec![pspdata],
        };
        let smresponse = SearchMixResponse {
            posts: spresponse,
            replies: srresponse,
        };
        let smresult_to_res: SearchMixResponse = smresult.into();
        let smresult_to_res2: SearchMixResponse = (&smresult2).into();
        assert_eq!(smresult_to_res, smresponse);
        assert_eq!(smresult_to_res2, smresponse);
    }
}
