use rocket::{Build, Rocket};
use rocket_db_pools::Connection;

use crate::pool::{Search, TypesenseSearch};
use crate::req::pulsar::*;

use serde_json::json;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/search", routes![search_burrow,])
}

#[get("/burrow/<keyword>")]
async fn search_burrow(searchpool: Connection<TypesenseSearch>, keyword: &str) -> String {
    let client = searchpool.into_inner();
    let uri = format!("/collections/burrows/documents/search?q={}&query_by=title,introduction&filter_by=&sort_by=", keyword);
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

#[get("/post/<keyword>/<tag>")]
async fn search_post_tag(mut searchpool: Connection<TypesenseSearch>, keyword: &str, tag:&str) -> String {
    let client = searchpool.into_inner();
    let uri = format!("/collections/posts/documents/search?q={}&query_by=title&filter_by=tag:= {}&sort_by=last_modified_time", keyword,tag);
    let response = match client.build_get(&uri).send().await {
        Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
        Err(e) => return format!("build_get send Error: {:?}", e),
    };
    let result: SearchResult = serde_json::from_str(&response).unwrap();
    match result.found {
        0 => format!("No results!"),
        _ => json!({
            "found":result.found,
            "hits":result.hits
        })
        .to_string(),
    }
}
