use crate::pool::{Search, TypesenseSearch};
use crate::req::pulsar::*;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;

use serde_json::json;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/search",
        routes![
            search,
            search_burrow_keyword,
            retrieve_burrow,
            search_post_keyword,
            search_post_tag,
            retrieve_post
        ],
    )
}
///redirect to other functions.
#[post("/search", data = "<data>", format = "json")]
async fn search(data: Json<SearchRequest>) -> Redirect {
    match &data.area {
        Some(area) => match &area[..] {
            "burrow" => match (data.keyword.as_ref(), data.id, data.tag.as_ref()) {
                (Some(_keyword), None, None) => Redirect::temporary(uri!(search_burrow_keyword())),
                (None, Some(id), None) => Redirect::temporary(uri!(retrieve_burrow(id))),
                (None, None, Some(_tag)) => {
                    Redirect::temporary(format!("Invalid request. Cannot search burrow for tag."))
                }
                _ => Redirect::temporary(format!(
                    "Invalid request. Must search for either keyword or id."
                )),
            },
            "post" => match ((data.keyword.as_ref(), data.id, data.tag.as_ref())) {
                (Some(_keyword), None, None) => Redirect::temporary(uri!(search_post_keyword())),
                (None, Some(id), None) => Redirect::temporary(uri!(retrieve_post(id))),
                (None, None, Some(_tag)) => Redirect::temporary(uri!(search_post_tag())),
                _ => Redirect::temporary(format!(
                    "Invalid request. Must search for either keyword or id."
                )),
            },
            _ => Redirect::temporary(format!("Invalid request. Invalid area.")),
        },
        None => Redirect::temporary(format!("Invalid area. Area must not be null.")),
    }
}


///Search keyword in burrow title/introduction.
#[post("/burrow", data = "<data>", format = "json")]
async fn search_burrow_keyword(
    searchpool: Connection<TypesenseSearch>,
    data: Json<SearchRequest>,
) -> String {
    let client = searchpool.into_inner();
    let uri = match (data.keyword.as_ref(),data.tag.as_ref()){
        (Some(keyword),None) =>format!("/collections/burrows/documents/search?q={}&query_by=title,introduction&filter_by=&sort_by=burrow_id:desc",keyword),
        _ => panic!("Redirect search request went wrong.")
    };
    let response = match client.build_get(&uri).send().await {
        Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
        Err(e) => return format!("build_get send Error: {:?}", e),
    };
    let result: SearchResult = match serde_json::from_str(&response) {
        Ok(a) => a,
        Err(e) => panic!("{}", response),
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


///Retrieve burrow with id.
#[get("/burrow/<burrow_id>")]
async fn retrieve_burrow(searchpool: Connection<TypesenseSearch>, burrow_id: i64) -> String {
    let client = searchpool.into_inner();
    let uri = format!("/collections/burrows/documents/{}", burrow_id);
    match client.build_get(&uri).send().await {
        Ok(a) => a.text().await.unwrap(),
        Err(e) => panic!("build_get send Error: {:?}", e),
    }
}


///Search keyword in post title/replies. Result filter by tag.
#[post("/post", data = "<request>", format = "json")]
async fn search_post_keyword(
    searchpool1: Connection<TypesenseSearch>,
    searchpool2: Connection<TypesenseSearch>,
    request: Json<SearchRequest>,
) -> String {
    let keyword = request.keyword.as_ref().unwrap();
    let client_qby_title = searchpool1.into_inner();
    let client_qby_content = searchpool2.into_inner();
    let uri_qby_title = format!(
        "/collections/posts/documents/search?q={}&query_by=content&filter_by=&group_by=&sort_by=post_id:desc",
        keyword
    );
    let uri_qby_content = format!("/collections/replies/documents/search?q={}&query_by=content&filter_by=&group_by=post_id&sort_by=post_id:desc", keyword);
    let response_qby_title = match client_qby_title.build_get(&uri_qby_title).send().await {
        Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
        Err(e) => return format!("build_get send Error: {:?}", e),
    };
    let response_qby_content = match client_qby_content.build_get(&uri_qby_content).send().await {
        Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
        Err(e) => return format!("build_get send Error: {:?}", e),
    };
    let result_qby_title: SearchResult = match serde_json::from_str(&response_qby_title) {
        Ok(a) => a,
        Err(e) => panic!("{}", response_qby_title),
    };
    let result_qby_content: SearchResult = match serde_json::from_str(&response_qby_content) {
        Ok(a) => a,
        Err(e) => panic!("{}", response_qby_content),
    };
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


/// Seach post by tag.
#[post("/post", data = "<request>", format = "json")]
async fn search_post_tag(
    searchpool: Connection<TypesenseSearch>,
    request: Json<SearchRequest>,
) -> String {
    let tag = request.tag.as_ref().unwrap();
    let client = searchpool.into_inner();
    let uri = format!(
        "/collections/posts/documents/search?q={}&query_by=tags&filter_by=&group_by=&sort_by=post_id:desc",
        tag
    );

    let response = match client.build_get(&uri).send().await {
        Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
        Err(e) => return format!("build_get send Error: {:?}", e),
    };
    let result: SearchResult = match serde_json::from_str(&response) {
        Ok(a) => a,
        Err(e) => panic!("{}", response),
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
///Retrieve post with id.
#[get("/post/<post_id>")]
async fn retrieve_post(searchpool: Connection<TypesenseSearch>, post_id: i64) -> String {
    let client = searchpool.into_inner();
    let uri = format!("/collections/posts/documents/{}", post_id);
    match client.build_get(&uri).send().await {
        Ok(a) => a.json::<serde_json::Value>().await.unwrap().to_string(),
        Err(e) => return format!("build_get send Error: {:?}", e),
    }
}
