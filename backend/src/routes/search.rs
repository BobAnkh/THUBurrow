use crate::pool::{Search, TypesenseSearch};
use crate::req::pulsar::*;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;

use serde_json::json;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/search", routes![search,])
}

#[post("/", data = "<data>", format = "json")]
async fn search(searchpool: Connection<TypesenseSearch>, data: Json<SearchRequest>) -> String {
    let client = searchpool.into_inner();
    match data.into_inner() {
        SearchRequest::SearchBurrowKeyword { keyword, page } => {
            let uri = format!("/collections/burrows/documents/search?q={}&query_by=title,introduction&filter_by=&sort_by=burrow_id:desc&page={}",keyword,page);
            let response = match client.build_get(&uri).send().await {
                Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
                Err(e) => return format!("build_get send Error: {:?}", e),
            };
            let result: SearchResult = match serde_json::from_str(&response) {
                Ok(a) => a,
                Err(e) => {
                    log::error!("{}", response);
                    return format!("{}", e);
                }
            };
            match result.found {
                0 => "No results!".to_string(),
                _ => json!({
                    "found":result.found,
                    "hits":result.hits
                })
                .to_string(),
            }
        }
        SearchRequest::RetrieveBurrow { burrow_id } => {
            let uri = format!("/collections/burrows/documents/{}", burrow_id);
            match client.build_get(&uri).send().await {
                Ok(a) => a.text().await.unwrap(),
                Err(e) => panic!("build_get send Error: {:?}", e),
            }
        }
        SearchRequest::SearchPostKeyword { keyword, page } => {
            let uri_qby_title = format!(
            "/collections/posts/documents/search?q={}&query_by=title&filter_by=&group_by=&sort_by=post_id:desc&page={}",
            keyword,page
        );
            let uri_qby_content = format!("/collections/replies/documents/search?q={}&query_by=content&filter_by=&group_by=post_id&sort_by=post_id:desc", keyword);
            let response_qby_title = match client.build_get(&uri_qby_title).send().await {
                Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
                Err(e) => return format!("build_get send Error: {:?}", e),
            };
            let response_qby_content = match client.build_get(&uri_qby_content).send().await {
                Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
                Err(e) => return format!("build_get send Error: {:?}", e),
            };
            let result_qby_title: SearchResult = match serde_json::from_str(&response_qby_title) {
                Ok(a) => a,
                Err(e) => {
                    log::error!("{}", response_qby_title);
                    return format!("{}", e);
                }
            };
            let result_qby_content: GroupedSearchResult =
                match serde_json::from_str(&response_qby_content) {
                    Ok(a) => a,
                    Err(e) => {
                        log::error!("{}", response_qby_content);
                        return format!("{}", e);
                    }
                };

            json!({
                "found": result_qby_title.found + result_qby_content.found,
                "hits":{
                    "result_qby_title":result_qby_title.hits ,
                    "result_qby_content":result_qby_content.grouped_hits
                }
            })
            .to_string()
        }
        SearchRequest::SearchPostTag { tag, page } => {
            let uri = format!(
            "/collections/posts/documents/search?q={}&query_by=tag&filter_by=&group_by=&sort_by=post_id:desc&page={}",
            tag,page
        );

            let response = match client.build_get(&uri).send().await {
                Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
                Err(e) => return format!("build_get send Error: {:?}", e),
            };
            let result: SearchResult = match serde_json::from_str(&response) {
                Ok(a) => a,
                Err(e) => {
                    log::error!("{}", response);
                    return format!("{}", e);
                }
            };
            match result.found {
                0 => "No results!".to_string(),
                _ => json!({
                    "found":result.found,
                    "hits":result.hits
                })
                .to_string(),
            }
        }
        SearchRequest::RetrievePost { post_id } => {
            let uri = format!("/collections/posts/documents/{}", post_id);
            match client.build_get(&uri).send().await {
                Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
                Err(e) => return format!("build_get send Error: {:?}", e),
            }
        }
    }
}
