mod common;
use backend::models::error::{ErrorCode, ErrorCode::*, ErrorMessage, ErrorResponse};
use backend::models::search::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::StatusCode;
use rocket::http::{ContentType, Header, Status};
use serde_json::json;
use std::fs;

#[test]
fn test_connected() {
    let client = common::get_client().lock();
    let response = client
        .get("/health")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // println!("{}", response.into_string().unwrap());
    assert_eq!(response.into_string().unwrap(), "Ok");
}

#[test]
fn test_signup() {
    let client = common::get_client().lock();
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();
    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name)}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    // sign up a user: perform a wrong action (illegal email address)
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": name,
            "password": "testpassword",
            "email": format!("{}@mails.tsignhua.edu.cn", name)}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    println!("{}", response.into_string().unwrap());
    // sign up a user: perform a wrong action (duplicated name and email)
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name)}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    println!("{}", response.into_string().unwrap());
    // sign up a user: perform a wrong action (user name is empty)
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": "",
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name)}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    println!("{}", response.into_string().unwrap());
}

#[test]
fn test_login_signup() {
    let client = common::get_client().lock();
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();
    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name)}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    // user log in
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // println!("{}", response.into_string().unwrap());
    // user log in: perform a wrong action (user not exsit)
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": "usernotexsit",
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    println!("{}", response.into_string().unwrap());
    // user log in: perform a wrong action (wrong password)
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": name,
            "password": "wrongpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    println!("{}", response.into_string().unwrap());
}

#[test]
fn test_burrow() {
    // get the client
    let client = common::get_client().lock();
    // generate a random name
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();

    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name)}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::user::UserResponse>()
        .unwrap();
    let burrow_id = res.default_burrow;

    // user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // println!("{}", response.into_string().unwrap());

    // create burrow: perform a wrong action
    let response = client
        .post("/burrows")
        .json(&json!({
            "description": format!("Test burrow of {}", name),
            "title": "Burrow test"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    println!("{}", response.into_string().unwrap());
    // let response = client
    //     .post("/burrows")
    //     .json(&json!({
    //         "description": format!("First burrow of {}", name),
    //         "title": "Burrow 1"}))
    //     .remote("127.0.0.1:8000".parse().unwrap())
    //     .dispatch();
    // assert_eq!(response.status(), Status::Forbidden);
    // println!("{}", response.into_string().unwrap());

    std::thread::sleep(std::time::Duration::from_secs(5));

    // follow the burrow
    let response = client
        .post("/users/relation")
        .json(&json!({ "ActivateFollow": burrow_id }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{:?}", response.into_string());
    // get following burrows of a user
    let response = client
        .get("/users/follow")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());

    // create burrow: perform a correct action
    let response = client
        .post("/burrows")
        .json(&json!({
            "description": format!("Second burrow of {}", name),
            "title": "Burrow 2"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // println!("{}", response.into_string().unwrap());

    // create burrow: perform a wrong action (amount up to limit)
    std::thread::sleep(std::time::Duration::from_secs(5));
    // create burrow (3rd)
    let response = client
        .post("/burrows")
        .json(&json!({
            "description": format!("Third burrow of {}", name),
            "title": "Burrow 3"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("Burrow Id: {}", response.into_string().unwrap());
    std::thread::sleep(std::time::Duration::from_secs(5));
    // create burrow (4th)
    let response = client
        .post("/burrows")
        .json(&json!({
            "description": format!("Forth burrow of {}", name),
            "title": "Burrow 4"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("Burrow Id: {}", response.into_string().unwrap());
    std::thread::sleep(std::time::Duration::from_secs(5));
    // create burrow (5th)
    let response = client
        .post("/burrows")
        .json(&json!({
            "description": format!("Fifth burrow of {}", name),
            "title": "Burrow 5"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("Burrow Id: {}", response.into_string().unwrap());
    std::thread::sleep(std::time::Duration::from_secs(5));
    // create burrow: perform a wrong action (6th)
    let response = client
        .post("/burrows")
        .json(&json!({
            "description": format!("Sixth burrow of {}", name),
            "title": "Burrow 6"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    println!("Burrow Id: {}", response.into_string().unwrap());

    // show burrow
    let response = client
        .get(format!("/burrows/{}", burrow_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    // show burrow: perform a wrong action (cannot find the burrow)
    let response = client
        .get(format!("/burrows/{}", burrow_id + 10))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    println!("{}", response.into_string().unwrap());

    // update burrow
    let response = client
        .patch(format!("/burrows/{}", burrow_id))
        .json(&json!({
            "description": format!("New Third burrow of {}", name),
            "title": "New Burrow 3"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // update burrow: perform a wrong action (missing burrow title)
    let response = client
        .patch(format!("/burrows/{}", burrow_id))
        .json(&json!({
            "description": format!("New Third burrow of {}", name),
            "title": ""}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);

    // show burrow (after update)
    let response = client
        .get(format!("/burrows/{}", burrow_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());

    // get burrow of a user
    let response = client
        .get("/users/burrows")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("Burrow ids are: {}", response.into_string().unwrap());

    // get valid burrow of a user
    let response = client
        .get("/users/valid-burrows")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("Burrow ids are: {}", response.into_string().unwrap());

    // discard burrow
    let response = client
        .delete(format!("/burrows/{}", burrow_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{:?}", response.into_string());
    // discard burrow: perform a wrong action (already discard)
    let response = client
        .delete(format!("/burrows/{}", burrow_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    println!("{}", response.into_string().unwrap());

    // update burrow: perform a wrong action (invalid burrow)
    let response = client
        .patch(format!("/burrows/{}", burrow_id))
        .json(&json!({
            "description": format!("New Third burrow of {}", name),
            "title": ""}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn test_content() {
    // get the client
    let client = common::get_client().lock();
    // generate a random name
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();

    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name)}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::user::UserResponse>()
        .unwrap();
    let burrow_id = res.default_burrow;

    // user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // println!("{}", response.into_string().unwrap());

    // create post 1
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("First post of {}", name),
            "burrow_id": burrow_id,
            "section": ["TestSection"],
            "tag": ["NoTag"],
            "content": "This is a test post no.1"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::content::PostCreateResponse>()
        .unwrap();
    let post_id = res.post_id;
    println!("Post Id: {}", post_id);
    // create post: perform a wrong action (empty title)
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": "",
            "burrow_id": burrow_id,
            "section": ["TestSection"],
            "tag": ["NoTag"],
            "content": "This is a test post no.2"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    println!("{}", response.into_string().unwrap());
    // create post: perform a wrong action (empty section)
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("Third post of {}", name),
            "burrow_id": burrow_id,
            "section": [],
            "tag": ["NoTag"],
            "content": "This is a test post no.3"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    println!("{}", response.into_string().unwrap());
    // TODO
    // create post: perform a wrong action (invalid section)
    // create post: perform a wrong action (invalid burrow)
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("Forth post of {}", name),
            "burrow_id": burrow_id + 10000,
            "section": ["TestSection"],
            "tag": ["NoTag"],
            "content": "This is a test post no.4"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    println!("{}", response.into_string().unwrap());
    // create post 2
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("Fifth post of {}", name),
            "burrow_id": burrow_id,
            "section": ["TestSection"],
            "tag": ["NoTag"],
            "content": "This is a test post no.5"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // create post 3
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("Sixth post of {}", name),
            "burrow_id": burrow_id,
            "section": ["TestSection"],
            "tag": ["NoTag"],
            "content": "This is a test post no.6"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // delete post 2
    let response = client
        .delete(format!("/content/posts/{}", post_id + 1))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{:?}", response.into_string());

    std::thread::sleep(std::time::Duration::from_secs(5));
    // delete post 3: perform a wrong action (out of time limit)
    let response = client
        .delete(format!("/content/posts/{}", post_id + 2))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    println!("{:?}", response.into_string());

    // create burrow
    let response = client
        .post("/burrows")
        .json(&json!({
            "description": format!("First burrow of {}", name),
            "title": "Burrow 1"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::burrow::BurrowCreateResponse>()
        .unwrap();
    let new_burrow_id = res.burrow_id;
    println!("Burrow Id: {}", new_burrow_id);
    // create post 4 with new_burrow_id
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("Sixth post of {}", name),
            "burrow_id": new_burrow_id,
            "section": ["TestSection"],
            "tag": ["NoTag"],
            "content": "This is a test post no.6"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // discard new burrow
    let response = client
        .delete(format!("/burrows/{}", new_burrow_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{:?}", response.into_string());
    // delete post no.4: perform a wrong action (invalid burrow)
    let response = client
        .delete(format!("/content/posts/{}", post_id + 3))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    println!("{:?}", response.into_string());

    // collect post no.1
    let response = client
        .post("/users/relation")
        .json(&json!({ "ActivateCollection": post_id }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{:?}", response.into_string());
    // like post no.1
    let response = client
        .post("/users/relation")
        .json(&json!({ "ActivateLike": post_id }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{:?}", response.into_string());

    // get post no.1
    let response = client
        .get(format!("/content/posts/{}", post_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    // get post no.2: perform a wrong action (post not exsit)
    let response = client
        .get(format!("/content/posts/{}", post_id + 1))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    println!("{}", response.into_string().unwrap());
    // get post no.3
    let response = client
        .get(format!("/content/posts/{}", post_id + 2))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());

    // get post list
    let response = client
        .get("/content/posts/list")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());

    // TODO
    // test trending interface

    // update post no.1
    let response = client
        .patch(format!("/content/posts/{}", post_id))
        .json(&json!({
            "title": format!("New First post of {}", name),
            "section": ["NewTestSection"],
            "tag": ["TestTag"]}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // update post no.2: perform a wrong action (post not exist)
    let response = client
        .patch(format!("/content/posts/{}", post_id + 1))
        .json(&json!({
            "title": format!("New wrong post of {}", name),
            "section": ["NewTestSection"],
            "tag": ["TestTag"]}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    // update post no.4: perform a wrong action (invalid burrow)
    let response = client
        .patch(format!("/content/posts/{}", post_id + 3))
        .json(&json!({
            "title": format!("New wrong post of {}", name),
            "section": ["NewTestSection"],
            "tag": ["TestTag"]}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
}
#[test]
fn test_search() {
    // get the client
    let client = common::get_client().lock();
    // generate a random name
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();

    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name)}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::user::UserResponse>()
        .unwrap();
    let burrow_id = res.default_burrow;

    // user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // println!("{}", response.into_string().unwrap());
    std::thread::sleep(std::time::Duration::from_secs(5));

    // create burrow
    let response = client
        .post("/burrows")
        .json(&json!({
            "description": format!("Created burrow of {}", name),
            "title": "Created Burrow"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<serde_json::Value>().unwrap();
    let created_burrow_id: i64 = serde_json::to_string(&res["burrow_id"])
        .unwrap()
        .parse::<i64>()
        .unwrap();
    println!("Created Burrow Id: {}", created_burrow_id);
    // println!("{}", response.into_string().unwrap());

    // create post
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("First post of {}", name),
            "burrow_id": burrow_id,
            "section": ["TestSection"],
            "tag": ["NoTag","政治相关"],
            "content": "search test"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::content::PostCreateResponse>()
        .unwrap();
    let post_id = res.post_id;
    println!("Post Id: {}", post_id);
    std::thread::sleep(std::time::Duration::from_secs(3));

    // retrieve burrow
    let response = client
        .post("/search".to_string())
        .json(&json!(SearchRequest::RetrieveBurrow {
            burrow_id: created_burrow_id
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(res["title"], "Created Burrow".to_string());
    // println!("Retrieve result: {}", response.into_string().unwrap());

    // retrieve burrow  (invalid burrow_id)
    let response = client
        .post("/search".to_string())
        .json(&json!(SearchRequest::RetrieveBurrow { burrow_id: -1 }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    let res = response.into_json::<ErrorResponse>().unwrap();
    assert_eq!(res.error.code, ErrorCode::BurrowNotExist);
    assert_eq!(res.error.message, "Cannot find burrow -1".to_string());

    // search burrow by keyword
    let response = client
        .post("/search".to_string())
        .json(&SearchRequest::SearchBurrowKeyword {
            keywords: vec!["Created".to_string()],
        })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(&res["burrows"][0]["burrow_id"], created_burrow_id);
    // println!("Search result: {}", response.into_string().unwrap());

    // // search burrow by keyword  (empty keyword vector)
    // let response = client
    //     .post(format!("/search/?{}", 1))
    //     .json(&SearchRequest::SearchBurrowKeyword { keywords: vec![] })
    //     .remote("127.0.0.1:8000".parse().unwrap())
    //     .dispatch();
    // assert_eq!(response.status(), Status::Ok);
    // println!("Search result: {}", response.into_string().unwrap());

    // search burrow by keyword  (repeat keyword vector)
    let response = client
        .post("/search".to_string())
        .json(&SearchRequest::SearchBurrowKeyword {
            keywords: vec![
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
                "created".to_string(),
            ],
        })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchBurrowResponse>().unwrap();
    // println!("{}",response.into_string().unwrap());
    assert_eq!(res.burrows[0].burrow_id, created_burrow_id);

    // retrieve post
    let response = client
        .post("/search".to_string())
        .json(&json!(SearchRequest::RetrievePost { post_id: 1 }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    let res = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(res["post_desc"]["post_id"], 1);
    // println!("Retrieve result: {}", response.into_string().unwrap());

    // search post by keyword
    let response = client
        .post("/search".to_string())
        .json(&SearchRequest::SearchPostKeyword {
            keywords: vec!["test".to_string()],
        })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchMixResponse>().unwrap();
    assert_eq!(res.replies.replies[0].post_id, post_id);

    // search post by keyword   (special characters)
    let response = client
        .post(format!("/search/?{}", 1))
        .json(&SearchRequest::SearchPostKeyword {
            keywords: vec!["❤❥웃유♋☮✌☏☢☠✔☑♚▲♪✈✞÷↑↓◆◇⊙■□△▽¿─│♥❣♂♀☿Ⓐ✍✉☣☤✘☒♛▼♫⌘☪≈←→◈◎☉★☆⊿※¡━┃♡ღツ☼☁❅♒✎©®™Σ✪✯☭➳卐√↖↗●◐Θ◤◥︻〖〗┄┆℃℉°✿ϟ☃☂✄¢€£∞✫★½✡×↙↘○◑⊕◣◢︼【】┅┇☽☾✚〓▂▃▄▅▆▇█▉▊▋▌▍▎▏↔↕☽☾の•▸◂▴▾┈┊①②③④⑤⑥⑦⑧⑨⑩ⅠⅡⅢⅣⅤⅥⅦⅧⅨⅩ㍿▓♨♛❖♓☪✙┉┋☹☺☻تヅツッシÜϡﭢ™℠℗©®♥❤❥❣❦❧♡۵웃유ღ♋♂♀☿☼☀☁☂☄☾☽❄☃☈⊙☉℃℉❅✺ϟ☇♤♧♡♢♠♣♥♦☜☞☝✍☚☛☟✌✽✾✿❁❃❋❀⚘☑✓✔√☐☒✗✘ㄨ✕✖✖⋆✢✣✤✥❋✦✧✩✰✪✫✬✭✮✯❂✡★✱✲✳✴✵✶✷✸✹✺✻✼❄❅❆❇❈❉❊†☨✞✝☥☦☓☩☯☧☬☸✡♁✙♆。，、＇：∶；?‘’“”〝〞ˆˇ﹕︰﹔﹖﹑•¨….¸;！´？！～—ˉ｜‖＂〃｀@﹫¡¿﹏﹋﹌︴々﹟#﹩$﹠&﹪%*﹡﹢﹦﹤‐￣¯―﹨ˆ˜﹍﹎+=<＿_-ˇ~﹉﹊（）〈〉‹›﹛﹜『』〖〗［］《》〔〕{}「」【】︵︷︿︹︽_﹁﹃︻︶︸﹀︺︾ˉ﹂﹄︼☩☨☦✞✛✜✝✙✠✚†‡◉○◌◍◎●◐◑◒◓◔◕◖◗❂☢⊗⊙◘◙◍⅟½⅓⅕⅙⅛⅔⅖⅚⅜¾⅗⅝⅞⅘≂≃≄≅≆≇≈≉≊≋≌≍≎≏≐≑≒≓≔≕≖≗≘≙≚≛≜≝≞≟≠≡≢≣≤≥≦≧≨≩⊰⊱⋛⋚∫∬∭∮∯∰∱∲∳%℅‰‱㊣㊎㊍㊌㊋㊏㊐㊊㊚㊛㊤㊥㊦㊧㊨㊒㊞㊑㊒㊓㊔㊕㊖㊗㊘㊜㊝㊟㊠㊡㊢㊩㊪㊫㊬㊭㊮㊯㊰㊙㉿囍♔♕♖♗♘♙♚♛♜♝♞♟ℂℍℕℙℚℝℤℬℰℯℱℊℋℎℐℒℓℳℴ℘ℛℭ℮ℌℑℜℨ♪♫♩♬♭♮♯°øⒶ☮✌☪✡☭✯卐✐✎✏✑✒✍✉✁✂✃✄✆✉☎☏➟➡➢➣➤➥➦➧➨➚➘➙➛➜➝➞➸♐➲➳⏎➴➵➶➷➸➹➺➻➼➽←↑→↓↔↕↖↗↘↙↚↛↜↝↞↟↠↡↢↣↤↥↦↧↨➫➬➩➪➭➮➯➱↩↪↫↬↭↮↯↰↱↲↳↴↵↶↷↸↹↺↻↼↽↾↿⇀⇁⇂⇃⇄⇅⇆⇇⇈⇉⇊⇋⇌⇍⇎⇏⇐⇑⇒⇓⇔⇕⇖⇗⇘⇙⇚⇛⇜⇝⇞⇟⇠⇡⇢⇣⇤⇥⇦⇧⇨⇩⇪➀➁➂➃➄➅➆➇➈➉➊➋➌➍➎➏➐➑➒➓㊀㊁㊂㊃㊄㊅㊆㊇㊈㊉ⒶⒷⒸⒹⒺⒻⒼⒽⒾⒿⓀⓁⓂⓃⓄⓅⓆⓇⓈⓉⓊⓋⓌⓍⓎⓏⓐⓑⓒⓓⓔⓕⓖⓗⓘⓙⓚⓛⓜⓝⓞⓟⓠⓡⓢⓣⓤⓥⓦⓧⓨⓩ⒜⒝⒞⒟⒠⒡⒢⒣⒤⒥⒦⒧⒨⒩⒪⒫⒬⒭⒮⒯⒰⒱⒲⒳⒴⒵ⅠⅡⅢⅣⅤⅥⅦⅧⅨⅩⅪⅫⅬⅭⅮⅯⅰⅱⅲⅳⅴⅵⅶⅷⅸⅹⅺⅻⅼⅽⅾⅿ┌┍┎┏┐┑┒┓└┕┖┗┘┙┚┛├┝┞┟┠┡┢┣┤┥┦┧┨┩┪┫┬┭┮┯┰┱┲┳┴┵┶┷┸┹┺┻┼┽┾┿╀╁╂╃╄╅╆╇╈╉╊╋╌╍╎╏═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟╠╡╢╣╤╥╦╧╨╩╪╫╬◤◥◄►▶◀◣◢▲▼◥▸◂▴▾△▽▷◁⊿▻◅▵▿▹◃❏❐❑❒▀▁▂▃▄▅▆▇▉▊▋█▌▍▎▏▐░▒▓▔▕■□▢▣▤▥▦▧▨▩▪▫▬▭▮▯㋀㋁㋂㋃㋄㋅㋆㋇㋈㋉㋊㋋㏠㏡㏢㏣㏤㏥㏦㏧㏨㏩㏪㏫㏬㏭㏮㏯㏰㏱㏲㏳㏴㏵㏶㏷㏸㏹㏺㏻㏼㏽㏾㍙㍚㍛㍜㍝㍞㍟㍠㍡㍢㍣㍤㍥㍦㍧㍨㍩㍪㍫㍬㍭㍮㍯㍰㍘☰☲☱☴☵☶☳☷☯
            ♠♣♧♡♥❤❥❣♂♀✲☀☼☾☽◐◑☺☻☎☏✿❀№↑↓←→√×÷★℃℉°◆◇⊙■□△▽¿½☯✡㍿卍卐♂♀✚〓㎡♪♫♩♬㊚㊛囍㊒㊖Φ♀♂‖$@*&#※卍卐Ψ♫♬♭♩♪♯♮⌒¶∮‖€￡¥$
            ①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳⓪⓿❶❷❸❹❺❻❼❽❾❿⓫⓬⓭⓮⓯⓰⓱⓲⓳⓴⓵⓶⓷⓸⓹⓺⓻⓼⓽⓾㊀㊁㊂㊃㊄㊅㊆㊇㊈㊉㈠㈡㈢㈣㈤㈥㈦㈧㈨㈩⑴⑵⑶⑷⑸⑹⑺⑻⑼⑽⑾⑿⒀⒁⒂⒃⒄⒅⒆⒇⒈⒉⒊⒋⒌⒍⒎⒏⒐⒑⒒⒓⒔⒕⒖⒗⒘⒙⒚⒛ⅠⅡⅢⅣⅤⅥⅦⅧⅨⅩⅪⅫⅰⅱⅲⅳⅴⅵⅶⅷⅸⅹⒶⒷⒸⒹⒺⒻⒼⒽⒾⒿⓀⓁⓂⓃⓄⓅⓆⓇⓈⓉⓊⓋⓌⓍⓎⓏⓐⓑⓒⓓⓔⓕⓖⓗⓘⓙⓚⓛⓜⓝⓞⓟⓠⓡⓢⓣⓤⓥⓦⓧⓨⓩ⒜⒝⒞⒟⒠⒡⒢⒣⒤⒥⒦⒧⒨⒩⒪⒫⒬⒭⒮⒯⒰⒱⒲⒳⒴⒵
            ﹢﹣×÷±+-*/^=≌∽≦≧≒﹤﹥≈≡≠≤≥≮≯∷∶∝∞∧∨∑∏∪∩∈∵∴⊥∥∠⌒⊙√∛∜∟⊿㏒㏑%‰⅟½⅓⅕⅙⅐⅛⅑⅒⅔¾⅖⅗⅘⅚⅜⅝⅞≂≃≄≅≆≇≉≊≋≍≎≏≐≑≓≔≕≖≗≘≙≚≛≜≝≞≟≢≣≨≩⊰⊱⋛⋚∫∮∬∭∯∰∱∲∳℅øπ∀∁∂∃∄∅∆∇∉∊∋∌∍∎∐−∓∔∕∖∗∘∙∡∢∣∤∦∸∹∺∻∼∾∿≀≁≪≫≬≭≰≱≲≳≴≵≶≷≸≹≺≻≼≽≾≿⊀⊁⊂⊃⊄⊅⊆⊇⊈⊉⊊⊋⊌⊍⊎⊏⊐⊑⊒⊓⊔⊕⊖⊗⊘⊚⊛⊜⊝⊞⊟⊠⊡⊢⊣⊤⊦⊧⊨⊩⊪⊫⊬⊭⊮⊯⊲⊳⊴⊵⊶⊷⊸⊹⊺⊻⊼⊽⊾⋀⋁⋂⋃⋄⋅⋆⋇⋈⋉⋊⋋⋌⋍⋎⋏⋐⋑⋒⋓⋔⋕⋖⋗⋘⋙⋜⋝⋞⋟⋠⋡⋢⋣⋤⋥⋦⋧⋨⋩⋪⋫⋬⋭⋮⋯⋰⋱⋲⋳⋴⋵⋶⋷⋸⋹⋺⋻⋼⋽⋾⋿ⅠⅡⅢⅣⅤⅥⅦⅧⅨⅩⅪⅫⅬⅭⅮⅯↁↂↃↅↆↇↈ↉↊↋■□▢▣▤▥▦▧▨▩▪▫▬▭▮▯▰▱▲△▴▵▶▷▸▹►▻▼▽▾▿◀◁◂◃◄◅◆◇◈◉◊○◌◍◎●◐◑◒◓◔◕◖◗◘◙◚◛◜◝◞◟◠◡◢◣◤◥◦◧◨◩◪◫◬◭◮◯◰◱◲◳◴◵◶◷◸◹◺◿◻◼◽◾⏢⏥⌓⌔⌖".to_string()]
        })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(res["posts"]["found"], 0);
    // println!("Search result: {}", response.into_string().unwrap());

    // search post by tag
    let response = client
        .post(format!("/search/?{}", 1))
        .json(&json!(SearchRequest::SearchPostTag {
            tag: vec!["政治相关".to_string()]
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<SearchPostResponse>().unwrap();
    assert_eq!(res.posts[0].post_id, post_id);
    // println!("Search result: {}", response.into_string().unwrap());

    // search post by tag   (empty tag vector)
    let response = client
        .post(format!("/search/?{}", 1))
        .json(&SearchRequest::SearchPostTag { tag: vec![] })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    let res = response.into_json::<ErrorResponse>().unwrap();

    assert_eq!(res.error.code, ErrorCode::EmptyField);
    assert_eq!(res.error.message, format!("Tags should not be empty"));
    // ErrorResponse::build(ErrorCode::EmptyField,format!("Tags should not be empty")));

    // discard burrow
    let response = client
        .delete(format!("/burrows/{}", burrow_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_string().unwrap();
    assert_eq!(res, format!("Success"));

    //retrieve a discarded burrow
    let response = client
        .post(format!("/search/?{}", 1))
        .json(&SearchRequest::RetrieveBurrow {
            burrow_id: burrow_id,
        })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_json::<serde_json::Value>().unwrap();
    assert_eq!(res["title"], "Default".to_string());
    // println!("Retrieve result: {}", response.into_string().unwrap());

    //retrieve a non-exist post
    let response = client
        .post(format!("/search/?{}", 1))
        .json(&SearchRequest::RetrievePost { post_id: -1 })
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
    let res = response.into_json::<ErrorResponse>().unwrap();
    // println!("Retrieve result: {}", response.into_string().unwrap());
    assert_eq!(res.error.code, ErrorCode::PostNotExist);
    assert_eq!(res.error.message, "Cannot find post -1".to_string());
}

#[test]
fn test_storage() {
    let client = common::get_client().lock();
    // generate a random name
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();

    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
        "username": name,
        "password": "testpassword",
        "email": format!("{}@mails.tsinghua.edu.cn", name)}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::user::UserResponse>()
        .unwrap();
    let burrow_id = res.default_burrow;

    // user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // println!("{}", response.into_string().unwrap());
    std::thread::sleep(std::time::Duration::from_secs(5));

    //get an jepg from httpbin
    let mut jpeg_buf: Vec<u8> = vec![];
    match reqwest::blocking::Client::new()
        .get("http://httpbin.org/image/jpeg")
        .send()
    {
        Ok(mut r) => match r.status() {
            StatusCode::OK => {
                r.copy_to(&mut jpeg_buf).unwrap();
            }
            _ => {
                let jpeg: String = std::iter::repeat(())
                    .map(|()| thread_rng().sample(Alphanumeric))
                    .map(char::from)
                    .take(1600)
                    .collect();
                jpeg_buf = jpeg.into_bytes();
            }
        },
        Err(_) => {
            jpeg_buf = "a;fklakdjfaoi;jflkasfasokfd".to_string().into_bytes();
        }
    };

    // store a jpeg
    let response = client
        .post("/storage/images")
        .header(ContentType::JPEG)
        .body(jpeg_buf.clone())
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let jepg_name = response.into_string().unwrap();

    //get an png from httpbin
    let mut png_buf: Vec<u8> = vec![];
    match reqwest::blocking::Client::new()
        .get("http://httpbin.org/image/jpeg")
        .send()
    {
        Ok(mut r) => match r.status() {
            StatusCode::OK => {
                r.copy_to(&mut png_buf).unwrap();
            }
            _ => {
                let jpeg: String = std::iter::repeat(())
                    .map(|()| thread_rng().sample(Alphanumeric))
                    .map(char::from)
                    .take(1600)
                    .collect();
                jpeg_buf = jpeg.into_bytes();
            }
        },
        Err(_) => {
            png_buf = "a;fklakdjfaoi;jflkasfasokfd".to_string().into_bytes();
        }
    };

    // store a png
    let response = client
        .post("/storage/images")
        .header(ContentType::PNG)
        .body(png_buf.clone())
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let png_name = response.into_string().unwrap();

    //list image
    let response = client
        .get("/storage/images")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_string().unwrap();
    println!("{}", res);

    //download jpeg image
    let response = client
        .get(format!("/storage/images/{}", jepg_name))
        .header(Header::new("Referer", "https://thuburrow.com/"))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_bytes().unwrap();
    assert_eq!(res, jpeg_buf);

    //download png image
    let response = client
        .get(format!("/storage/images/{}", png_name))
        .header(Header::new("Referer", "https://thuburrow.com/"))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_bytes().unwrap();
    assert_eq!(res, png_buf);
}
