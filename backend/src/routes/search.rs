use rocket::{Build, Rocket};
use rocket_db_pools::Connection;

use crate::pool::{Search, TypesenseSearch};
use crate::req::pulsar::*;

use serde_json::json;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/search",
        routes![
            search_burrow_keyword,
            search_burrow_id,
            search_post_keyword,
            search_post_id
        ],
    )
}
///Search keyword in burrow title/introduction.
#[get("/burrow/<keyword>", rank = 2)]
async fn search_burrow_keyword(searchpool: Connection<TypesenseSearch>, keyword: &str) -> String {
    let client = searchpool.into_inner();
    let uri = format!("/collections/burrows/documents/search?q={}&query_by=title,introduction&filter_by=&sort_by=burrow_id", keyword);
    let response = match client.build_get(&uri).send().await {
        Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
        Err(e) => return format!("build_get send Error: {:?}", e),
    };
    let result: SearchResult = serde_json::from_str(&response).unwrap();
    match result.found {
        0 => "No results!".to_string(),
        _ => json!({
            "found":result.found,
            "hits":result.hits
        })
        .to_string(),
    }
}
///Search burrow with id.
#[get("/burrow/<burrow_id>", rank = 1)]
async fn search_burrow_id(searchpool: Connection<TypesenseSearch>, burrow_id: i64) -> String {
    let client = searchpool.into_inner();
    let uri = format!(
        "/collections/burrows/documents/search?q={}&query_by=burrow_id&filter_by=&sort_by=burrow_id",
        burrow_id
    );
    let response = match client.build_get(&uri).send().await {
        Ok(a) => a.text().await.unwrap(),
        Err(e) => return format!("build_get send Error: {:?}", e),
    };
    let result: SearchResult = serde_json::from_str(&response).unwrap();
    match result.found {
        0 => "No results!".to_string(),
        _ => json!({
            "found":result.found,
            "hits":result.hits
        })
        .to_string(),
    }
}
///Search keyword in post title/replies. Result filter by tag.
#[get("/post/<keyword>/<tag>")]
async fn search_post_keyword(
    searchpool1: Connection<TypesenseSearch>,
    searchpool2: Connection<TypesenseSearch>,
    keyword: &str,
    tag: &str,
) -> String {
    let client_qby_title = searchpool1.into_inner();
    let client_qby_content = searchpool2.into_inner();
    let uri_qby_title = format!(
        "/collections/posts/documents/search?q={}&query_by=content&filter_by=&group_by=&sort_by=post_id",
        keyword
    );
    let uri_qby_content = format!("/collections/replies/documents/search?q={}&query_by=content&filter_by=tag:= {}&group_by=post_id&sort_by=post_id", keyword,tag);
    let response_qby_title = match client_qby_title.build_get(&uri_qby_title).send().await {
        Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
        Err(e) => return format!("build_get send Error: {:?}", e),
    };
    let response_qby_content = match client_qby_content.build_get(&uri_qby_content).send().await {
        Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
        Err(e) => return format!("build_get send Error: {:?}", e),
    };
    let result_qby_title: SearchResult = serde_json::from_str(&response_qby_title).unwrap();
    let result_qby_content: SearchResult = serde_json::from_str(&response_qby_content).unwrap();
    // match result_qby_title.found {
    //     0 => "No results!".to_string(),
    //     _ => json!({
    //         "found":result.found,
    //         "hits":result.hits
    //     })
    //     .to_string(),
    // }
    json!({
        "result_qby_title": result_qby_title.hits,
        "result_qby_content":result_qby_content.hits
    })
    .to_string()
}
///Search post with id.
#[get("/post/<post_id>")]
async fn search_post_id(searchpool: Connection<TypesenseSearch>, post_id: i64) -> String {
    let client = searchpool.into_inner();
    let uri = format!(
        "/collections/posts/documents/search?q={}&query_by=post_id&filter_by=&sort_by=post_id",
        post_id
    );
    let response = match client.build_get(&uri).send().await {
        Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
        Err(e) => return format!("build_get send Error: {:?}", e),
    };
    let result: SearchResult = serde_json::from_str(&response).unwrap();
    match result.found {
        0 => "No results!".to_string(),
        _ => json!({
            "found":result.found,
            "hits":result.hits
        })
        .to_string(),
    }
}
