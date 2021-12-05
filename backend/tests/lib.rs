mod common;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Status;
use serde_json::json;

#[test]
fn test_connected() {
    let client = common::get_client().lock().unwrap();
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
    let client = common::get_client().lock().unwrap();
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();
    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": format!("{}", name),
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
            "username": format!("{}", name),
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
            "username": format!("{}", name),
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
    let client = common::get_client().lock().unwrap();
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();
    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": format!("{}", name),
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
            "username": format!("{}", name),
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
            "username": format!("{}", name),
            "password": "wrongpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    println!("{}", response.into_string().unwrap());
}

#[test]
fn test_burrow() {
    // get the client
    let client = common::get_client().lock().unwrap();
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
            "username": format!("{}", name),
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name)}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::req::user::UserResponse>()
        .unwrap();
    let burrow_id = res.default_burrow;

    // user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": format!("{}", name),
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
    let res = response
        .into_json::<backend::req::burrow::BurrowCreateResponse>()
        .unwrap();
    let burrow_id = res.burrow_id;
    println!("Burrow Id: {}", burrow_id);

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
        .put(format!("/burrows/{}", burrow_id))
        .json(&json!({
            "description": format!("New Third burrow of {}", name),
            "title": "New Burrow 3"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // update burrow: perform a wrong action (missing burrow title)
    let response = client
        .put(format!("/burrows/{}", burrow_id))
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
        .get("/users/burrow")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("Burrow ids are: {}", response.into_string().unwrap());

    // get valid burrow of a user
    let response = client
        .get("/users/valid-burrow")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("Burrow ids are: {}", response.into_string().unwrap());

    // get following burrows of a user
    let response = client
        .get("/users/follow")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());

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
        .put(format!("/burrows/{}", burrow_id))
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
    let client = common::get_client().lock().unwrap();
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
            "username": format!("{}", name),
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name)}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::req::user::UserResponse>()
        .unwrap();
    let burrow_id = res.default_burrow;

    // user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": format!("{}", name),
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // println!("{}", response.into_string().unwrap());

    // create post
    let response = client
        .post("/content/post")
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
        .into_json::<backend::req::content::PostCreateResponse>()
        .unwrap();
    let post_id = res.post_id;
    println!("Post Id: {}", post_id);
    // create post: perform a wrong action (empty title)
    let response = client
        .post("/content/post")
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
        .post("/content/post")
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
        .post("/content/post")
        .json(&json!({
            "title": format!("Forth post of {}", name),
            "burrow_id": burrow_id + 10,
            "section": ["TestSection"],
            "tag": ["NoTag"],
            "content": "This is a test post no.4"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    println!("{}", response.into_string().unwrap());

    // get post
    let response = client
        .get(format!("/content/post/{}", post_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    // get post: perform a wrong action (post not exsit)
    let response = client
        .get(format!("/content/post/{}", post_id + 10))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    println!("{}", response.into_string().unwrap());

    // get post list
    let response = client
        .get("/content/post/list")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
}

// #[test]
// fn test_trending() {
//     // get the client
//     let client = common::get_client().lock().unwrap();
//     // generate a random name
//     let name: String = std::iter::repeat(())
//         .map(|()| thread_rng().sample(Alphanumeric))
//         .map(char::from)
//         .take(16)
//         .collect();

//     // sign up a user
//     let response = client
//         .post("/users/sign-up")
//         .json(&json!({
//             "username": format!("{}", name),
//             "password": "testpassword",
//             "email": format!("{}@mails.tsinghua.edu.cn", name)}))
//         .remote("127.0.0.1:8000".parse().unwrap())
//         .dispatch();
//     assert_eq!(response.status(), Status::Ok);

//     // user login
//     let response = client
//         .post("/users/login")
//         .json(&json!({
//             "username": format!("{}", name),
//             "password": "testpassword"}))
//         .remote("127.0.0.1:8000".parse().unwrap())
//         .dispatch();
//     assert_eq!(response.status(), Status::Ok);
//     // println!("{}", response.into_string().unwrap());
// }
