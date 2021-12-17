mod common;
use backend::models::content::PostSection;
use backend::models::error::*;
use backend::utils::mq::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Status;
use serde_json::json;
use tokio::runtime::Runtime;

#[test]
fn integration_test() {
    let rt = Runtime::new().unwrap();
    rt.spawn(generate_trending());
    rt.spawn(pulsar_relation());
    rt.spawn(pulsar_typesense());
    rt.spawn(pulsar_email());
    test_connected();
    test_email();
    test_user();
    test_burrow();
    test_content();
}

// #[test]
fn test_connected() {
    let client = common::get_client().lock();
    let response = client
        .get("/health")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Ok");
}

// #[test]
fn test_email() {
    let client = common::get_client().lock();
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();
    // set verification code: Request Time 1
    let response = client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    std::thread::sleep(std::time::Duration::from_secs(2));
    // set verification code: Request Time 2
    let response = client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    std::thread::sleep(std::time::Duration::from_secs(2));
    // set verification code: Request Time 3
    let response = client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    std::thread::sleep(std::time::Duration::from_secs(2));
    // set verification code: Request Time 4 (RateLimit)
    let response = client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::TooManyRequests);
    println!("{}", response.into_string().unwrap());
    // set verification code
    let new_name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();
    let response = client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", new_name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    std::thread::sleep(std::time::Duration::from_secs(5));
    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": new_name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", new_name),
            "verification_code": "666666",
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    // set verification code: perform a wrong action (EmailDuplicate)
    let response = client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", new_name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    println!("{}", response.into_string().unwrap());
}

// #[test]
fn test_user() {
    let client = common::get_client().lock();
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();
    // sign up a user: perform a wrong action (wrong verification code)
    let new_name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();

    // 1. test user_sign_up
    // set verification code
    let response = client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    println!("{}", response.into_string().unwrap());
    std::thread::sleep(std::time::Duration::from_secs(5));
    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name),
            "verification_code": "666666"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    // sign up a user: perform a wrong action (illegal email address)
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": new_name,
            "password": "testpassword",
            "email": format!("{}@mails.tsignhua.edu.cn", new_name),
            "verification_code": "666666"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::EmailInvalid, "Invalid Email address.",)
    );
    // sign up a user: perform a wrong action (user name is empty)
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": "",
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name),
            "verification_code": "666666"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::EmptyField, "Empty username.",)
    );
    // sign up a user: perform a wrong action (duplicated email)
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": new_name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name),
            "verification_code": "666666"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::EmailDuplicate, "Duplicate Email address.",)
    );
    // sign up a user: perform a wrong action (duplicated name)
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", new_name),
            "verification_code": "666666"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::UsernameDuplicate, "Duplicate username.",)
    );

    // 2. test user_log_in
    // user log in
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // user log in: find old token
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // user log in: perform a wrong action (user not exist)
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": "usernotexist",
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::CredentialInvalid, "Wrong username or password.",)
    );
    // user log in: perform a wrong action (wrong password)
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": name,
            "password": "wrongpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::CredentialInvalid, "Wrong username or password.",)
    );
}

// #[test]
fn test_burrow() {
    // get the client
    let client = common::get_client().lock();
    // generate a random name
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();
    // set verification code
    client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    std::thread::sleep(std::time::Duration::from_secs(5));
    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name),
            "verification_code": "666666"}))
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
    assert_eq!(response.into_string().unwrap(), "Success");

    // 3. test create_burrow
    // create burrow: perform a wrong action (less in 24 hrs)
    let response = client
        .post("/burrows")
        .json(&json!({
            "description": format!("Test burrow of {}", name),
            "title": "Burrow test"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::TooManyRequests);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(
            ErrorCode::RateLimit,
            "User can only create a new burrow every 24 hours",
        )
    );
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
    // create burrow (2nd)
    let response = client
        .post("/burrows")
        .json(&json!({
            "description": format!("Second burrow of {}", name),
            "title": "Burrow 2"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    std::thread::sleep(std::time::Duration::from_secs(5));
    // create burrow: perform a wrong action (empty title)
    let response = client
        .post("/burrows")
        .json(&json!({
            "description": format!("Empty title burrow of {}", name),
            "title": ""}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::EmptyField, "Burrow title cannot be empty",)
    );
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
    println!("{}", response.into_string().unwrap());
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
    println!("{}", response.into_string().unwrap());
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
    println!("{}", response.into_string().unwrap());
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
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(
            ErrorCode::BurrowNumLimit,
            "Owned burrow amount is up to limit.",
        )
    );

    // follow burrow 1st
    let response = client
        .post("/users/relation")
        .json(&json!({ "ActivateFollow": burrow_id }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // follow burrow 2nd
    let response = client
        .post("/users/relation")
        .json(&json!({ "ActivateFollow": burrow_id + 1 }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    std::thread::sleep(std::time::Duration::from_secs(1));

    // 4. test get_follow
    // get following burrows of a user
    let response = client
        .get("/users/follow")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<Vec<backend::models::user::UserGetFollowResponse>>()
        .unwrap();
    assert_eq!(burrow_id + 1, res[0].burrow.burrow_id);
    assert_eq!(burrow_id, res[1].burrow.burrow_id);

    // 5. test get_total_burrow_count
    // get total burrow count
    let response = client
        .get("/burrows/total")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());

    // 6. test show_burrow
    // show burrow
    let response = client
        .get(format!("/burrows/{}", burrow_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        format!("{{\"title\":\"Default\",\"description\":\"\",\"posts\":[]}}"),
        response.into_string().unwrap()
    );
    // show burrow: perform a wrong action (burrow not exist)
    let response = client
        .get(format!("/burrows/{}", burrow_id + 10000))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::BurrowNotExist, "")
    );

    // 7. test update_burrow
    // update burrow
    let response = client
        .patch(format!("/burrows/{}", burrow_id))
        .json(&json!({
            "description": format!("New Default burrow of {}", name),
            "title": "New Default burrow"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // update burrow: perform a wrong action (empty burrow title)
    let response = client
        .patch(format!("/burrows/{}", burrow_id))
        .json(&json!({
            "description": format!("New Default burrow of {}", name),
            "title": ""}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::EmptyField, "Burrow title cannot be empty",)
    );
    // show burrow (after update)
    let response = client
        .get(format!("/burrows/{}", burrow_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        format!("{{\"title\":\"New Default burrow\",\"description\":\"New Default burrow of {}\",\"posts\":[]}}", name),
        response.into_string().unwrap()
    );

    // 8. test get_burrow
    // get burrow of a user
    let response = client
        .get("/users/burrows")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<Vec<backend::models::burrow::BurrowMetadata>>()
        .unwrap();
    assert_eq!(burrow_id + 4, res[0].burrow_id);
    assert_eq!(burrow_id, res[4].burrow_id);

    // 9. test get_user_valid_burrow
    // get valid burrow of a user
    let response = client
        .get("/users/valid-burrows")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        format!(
            "[{},{},{},{},{}]",
            burrow_id,
            burrow_id + 1,
            burrow_id + 2,
            burrow_id + 3,
            burrow_id + 4
        ),
        response.into_string().unwrap()
    );

    // 10. test discard_burrow
    // discard burrow
    let response = client
        .delete(format!("/burrows/{}", burrow_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // discard burrow: perform a wrong action (already discard)
    let response = client
        .delete(format!("/burrows/{}", burrow_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(
            ErrorCode::UserForbidden,
            "Burrow doesn't belong to current user or already be discarded",
        )
    );

    // update burrow: perform a wrong action (invalid burrow)
    let response = client
        .patch(format!("/burrows/{}", burrow_id))
        .json(&json!({
            "description": format!("New Third burrow of {}", name),
            "title": "new Burrow 3"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::EmptyField, "Burrow title cannot be empty",)
    );
}

// #[test]
fn test_content() {
    // get the client
    let client = common::get_client().lock();
    // generate a random name
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
        .collect();
    // set verification code
    client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    std::thread::sleep(std::time::Duration::from_secs(5));
    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", name),
            "verification_code": "666666"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::user::UserResponse>()
        .unwrap();
    let burrow_id = res.default_burrow;
    println!("Default Burrow id is {}", burrow_id);

    // user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    // follow the burrow
    let response = client
        .post("/users/relation")
        .json(&json!({ "ActivateFollow": burrow_id }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    // get burrow of a user to check post_num (before create any post)
    let response = client
        .get("/users/burrows")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<Vec<backend::models::burrow::BurrowMetadata>>()
        .unwrap();
    assert_eq!(burrow_id, res[0].burrow_id);
    assert_eq!(0, res[0].post_num);

    // 11. test create_post
    // create post 1
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("First post of {}", name),
            "burrow_id": burrow_id,
            "section": ["Learning"],
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
            "section": ["Learning"],
            "tag": ["NoTag"],
            "content": "This is a test post no.2"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::EmptyField, "Empty post title.",)
    );
    // create post: perform a wrong action (invalid section)
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("Third post of {}", name),
            "burrow_id": burrow_id,
            "section": [],
            "tag": ["NoTag"],
            "content": "This is a wrong test post"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::SectionInvalid, "Wrong Post Section.",)
    );
    // create post: perform a wrong action (invalid tag)
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("Third post of {}", name),
            "burrow_id": burrow_id,
            "section": ["Learning"],
            "tag": ["Tag1", "Tag2", "Tag3", "Tag4", "Tag5", "Tag6", "Tag7", "Tag8", "Tag9", "Tag10", ""],
            "content": "This is a wrong test post"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::SectionInvalid, "Wrong Post Tag.",)
    );
    // create post: perform a wrong action (invalid burrow)
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("Forth post of {}", name),
            "burrow_id": burrow_id + 10000,
            "section": ["Learning"],
            "tag": ["NoTag"],
            "content": "This is a test post no.4"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::BurrowInvalid, "")
    );
    // create post 2
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("Fifth post of {}", name),
            "burrow_id": burrow_id,
            "section": ["Learning"],
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
            "section": ["Life", "NSFW", "Learning"],
            "tag": ["NoTag"],
            "content": "This is a test post no.6"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // get burrow of a user to check post_num (after created post 1~3)
    let response = client
        .get("/users/burrows")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<Vec<backend::models::burrow::BurrowMetadata>>()
        .unwrap();
    assert_eq!(burrow_id, res[0].burrow_id);
    assert_eq!(3, res[0].post_num);

    // 12. test delete_post
    // delete post 2
    let response = client
        .delete(format!("/content/posts/{}", post_id + 1))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // delete post: perform a wrong action (post not exist)
    let response = client
        .delete(format!("/content/posts/{}", post_id + 10000))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(
            ErrorCode::PostNotExist,
            format!("Cannot find post {}", post_id + 10000),
        )
    );
    std::thread::sleep(std::time::Duration::from_secs(5));
    // delete post 3: perform a wrong action (out of time limit)
    let response = client
        .delete(format!("/content/posts/{}", post_id + 2))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(
            ErrorCode::UserForbidden,
            "Can only delete post within 2 minutes.",
        )
    );

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
    // follow the burrow
    let response = client
        .post("/users/relation")
        .json(&json!({ "ActivateFollow": new_burrow_id }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // create post 4 with new_burrow_id
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("Sixth post of {}", name),
            "burrow_id": new_burrow_id,
            "section": ["Learning"],
            "tag": ["NoTag"],
            "content": "This is a test post no.6"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // create post 5 with new_burrow_id and duplicated section and tag
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("Sixth post of {}", name),
            "burrow_id": new_burrow_id,
            "section": ["Learning", "Learning"],
            "tag": ["NoTag", "NoTag"],
            "content": "This is a test post no.6"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // 13. test trending
    // get trending
    let response = client
        .get("/trending")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());

    // collect post no.1
    let response = client
        .post("/users/relation")
        .json(&json!({ "ActivateCollection": post_id }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // collect post no.2 (post no.2 not exist)
    let response = client
        .post("/users/relation")
        .json(&json!({ "ActivateCollection": post_id + 1 }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // collect post no.3
    let response = client
        .post("/users/relation")
        .json(&json!({ "ActivateCollection": post_id + 2 }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // like post no.1
    let response = client
        .post("/users/relation")
        .json(&json!({ "ActivateLike": post_id }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    std::thread::sleep(std::time::Duration::from_secs(1));

    // get following burrows of a user, check if it's updated
    let response = client
        .get("/users/follow")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<Vec<backend::models::user::UserGetFollowResponse>>()
        .unwrap();
    assert_eq!(new_burrow_id, res[0].burrow.burrow_id);
    assert_eq!(true, res[0].is_update);
    assert_eq!(burrow_id, res[1].burrow.burrow_id);
    assert_eq!(true, res[1].is_update);

    // get trending: trending already exist
    let response = client
        .get("/trending")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());

    // 14. test get_total_post_count
    // get total post count
    let response = client
        .get("/content/posts/total")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());

    // 15. test create_reply
    // create reply for post no.1, using default burrow
    let response = client
        .post("/content/replies")
        .json(&json!({
            "post_id": post_id,
            "burrow_id": burrow_id,
            "content": "This is a test reply no.1 for post no.1"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::content::ReplyCreateResponse>()
        .unwrap();
    let reply_id = res.reply_id;
    println!("Reply Id: {}", reply_id);
    // create reply: perform a wrong action (invalid burrow)
    let response = client
        .post("/content/replies")
        .json(&json!({
            "post_id": post_id,
            "burrow_id": burrow_id + 10000,
            "content": "This is a test reply no.2 for post no.1"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::BurrowInvalid, "")
    );
    // create reply for post no.2: perform a wrong action (post not exist)
    let response = client
        .post("/content/replies")
        .json(&json!({
            "post_id": post_id + 1,
            "burrow_id": burrow_id,
            "content": "This is a test reply no.1 for post no.10000"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(
            ErrorCode::PostNotExist,
            format!("Cannot find post {}", post_id + 1),
        )
    );
    // create reply for post no.3, using default burrow
    let response = client
        .post("/content/replies")
        .json(&json!({
            "post_id": post_id + 2,
            "burrow_id": burrow_id,
            "content": "This is a test reply no.1 for post no.1"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    // create reply for post no.1, using new burrow
    let response = client
        .post("/content/replies")
        .json(&json!({
            "post_id": post_id,
            "burrow_id": new_burrow_id,
            "content": "This is a test reply no.1 for post no.1"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());

    std::thread::sleep(std::time::Duration::from_secs(1));

    // 16. test get_collection
    // get post collection of a user
    let response = client
        .get("/users/collection")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<Vec<backend::models::user::UserGetCollectionResponse>>()
        .unwrap();
    assert_eq!(post_id + 2, res[0].post.post_id);
    assert_eq!(true, res[0].is_update);
    assert_eq!(post_id, res[1].post.post_id);
    assert_eq!(true, res[0].is_update);

    // discard new burrow
    let response = client
        .delete(format!("/burrows/{}", new_burrow_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // delete post no.4: perform a wrong action (invalid burrow)
    let response = client
        .delete(format!("/content/posts/{}", post_id + 3))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::BurrowInvalid, "Not allowed to delete this post")
    );

    // 17. test read_post
    // get post no.1
    let response = client
        .get(format!("/content/posts/{}", post_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::content::PostPage>()
        .unwrap();
    assert_eq!(post_id, res.post_desc.post_id);
    assert_eq!(format!("First post of {}", name), res.post_desc.title);
    assert_eq!(3, res.post_desc.post_len);
    // TODO: match reply
    assert_eq!(reply_id, res.reply_page[1].reply_id);
    assert_eq!(
        "This is a test reply no.1 for post no.1".to_string(),
        res.reply_page[1].content
    );
    // get post no.2: perform a wrong action (post not exist)
    let response = client
        .get(format!("/content/posts/{}", post_id + 1))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(
            ErrorCode::PostNotExist,
            format!("Cannot find post {}", post_id + 1),
        )
    );
    // get post no.3
    let response = client
        .get(format!("/content/posts/{}", post_id + 2))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::content::PostPage>()
        .unwrap();
    assert_eq!(post_id + 2, res.post_desc.post_id);
    assert_eq!(2, res.post_desc.post_len);
    assert_eq!(
        vec![PostSection::Learning, PostSection::Life, PostSection::NSFW],
        res.post_desc.section
    );
    // get post no.4
    let response = client
        .get(format!("/content/posts/{}", post_id + 3))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::content::PostPage>()
        .unwrap();
    assert_eq!(post_id + 3, res.post_desc.post_id);
    assert_eq!(new_burrow_id, res.post_desc.burrow_id);
    // get post no.5 to test if tag and section is duplicated
    let response = client
        .get(format!("/content/posts/{}", post_id + 4))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::content::PostPage>()
        .unwrap();
    assert_eq!(post_id + 4, res.post_desc.post_id);
    assert_eq!(1, res.post_desc.post_len);
    assert_eq!(vec![PostSection::Learning], res.post_desc.section);
    assert_eq!(vec!["NoTag"], res.post_desc.tag);

    // 18. test read_post_list
    // get post list
    let response = client
        .get("/content/posts/list")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    println!("{}", response.into_string().unwrap());
    // TODO
    // // get post list with section
    // let response = client
    //     .get(format!("/content/posts/list?page=0&section=NSFW"))
    //     .remote("127.0.0.1:8000".parse().unwrap())
    //     .dispatch(); assert_eq!(response.status(), Status::Ok);
    // println!("{}", response.into_string().unwrap());
    // // get post list with section
    // let response = client
    //     .get(format!("/content/posts/list?section=Learning"))
    //     .remote("127.0.0.1:8000".parse().unwrap())
    //     .dispatch();
    // assert_eq!(response.status(), Status::Ok);
    // println!("{}", response.into_string().unwrap());

    // 19. test update_post
    // update post no.1
    let response = client
        .patch(format!("/content/posts/{}", post_id))
        .json(&json!({
            "title": format!("New First post of {}", name),
            "section": ["Life"],
            "tag": ["TestTag", "TestTag"]}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // update post no.2: perform a wrong action (post not exist)
    let response = client
        .patch(format!("/content/posts/{}", post_id + 1))
        .json(&json!({
            "title": format!("New wrong post of {}", name),
            "section": ["Life"],
            "tag": ["TestTag"]}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(
            ErrorCode::PostNotExist,
            format!("Cannot find post {}", post_id + 1),
        )
    );
    // update post no.3: perform a wrong action (empty title)
    let response = client
        .patch(format!("/content/posts/{}", post_id))
        .json(&json!({
            "title": "",
            "section": ["Life"],
            "tag": ["TestTag"]}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::EmptyField, "Empty post title.",)
    );
    // update post no.3: perform a wrong action (invalid section)
    let response = client
        .patch(format!("/content/posts/{}", post_id))
        .json(&json!({
            "title": format!("New post no.3 of {}", name),
            "section": [],
            "tag": ["TestTag"]}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::SectionInvalid, "Wrong Post Section.",)
    );
    // update post no.3: perform a wrong action (invalid tag)
    let response = client
        .patch(format!("/content/posts/{}", post_id))
        .json(&json!({
            "title": format!("New post no.3 of {}", name),
            "section": ["Learning"],
            "tag": ["Tag1", "Tag2", "Tag3", "Tag4", "Tag5", "Tag6", "Tag7", "Tag8", "Tag9", "Tag10", ""]}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::SectionInvalid, "Wrong Post Tag.",)
    );
    // update post no.4: perform a wrong action (invalid burrow)
    let response = client
        .patch(format!("/content/posts/{}", post_id + 3))
        .json(&json!({
            "title": format!("New wrong post of {}", name),
            "section": ["Life"],
            "tag": ["TestTag"]}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::BurrowInvalid, "Not allowed to update this post")
    );

    // 20. test update_reply
    // update reply 1-1
    let response = client
        .patch("/content/replies")
        .json(&json!({
            "post_id": post_id,
            "reply_id": reply_id,
            "content": "This is a updated reply no.1 for post no.1"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // update reply: perform a wrong action (reply not exist)
    let response = client
        .patch("/content/replies")
        .json(&json!({
            "post_id": post_id,
            "reply_id": reply_id + 100,
            "content": "This is a updated reply no.1 for post no.1"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(
            ErrorCode::PostNotExist,
            format!("Cannot find reply {}-{}", post_id, reply_id + 100),
        )
    );
    // update reply 1-2: perform a wrong action (invalid burrow)
    let response = client
        .patch("/content/replies")
        .json(&json!({
            "post_id": post_id,
            "reply_id": reply_id + 1,
            "content": "This is a updated reply no.2 for post no.1"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::BurrowInvalid, "")
    );

    // get post no.1 after update
    let response = client
        .get(format!("/content/posts/{}", post_id))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::content::PostPage>()
        .unwrap();
    assert_eq!(post_id, res.post_desc.post_id);
    assert_eq!(vec!["TestTag"], res.post_desc.tag);
    assert_eq!(format!("New First post of {}", name), res.post_desc.title);
    assert_eq!(reply_id, res.reply_page[1].reply_id);
    assert_eq!(
        "This is a updated reply no.1 for post no.1".to_string(),
        res.reply_page[1].content
    );
}

// #[test]
// fn test_storage() {
//     // // get the client
//     // let client = common::get_client().lock();
//     // // generate a random name
//     // let name: String = std::iter::repeat(())
//     //     .map(|()| thread_rng().sample(Alphanumeric))
//     //     .map(char::from)
//     //     .take(16)
//     //     .collect();

//     // // sign up a user
//     // let response = client
//     //     .post("/users/sign-up")
//     //     .json(&json!({
//     //             "username": name,
//     //             "password": "testpassword",
//     //             "email": format!("{}@mails.tsinghua.edu.cn", name)}))
//     //     .remote("127.0.0.1:8000".parse().unwrap())
//     //     .dispatch();
//     // assert_eq!(response.status(), Status::Ok);

//     // // user login
//     // let response = client
//     //     .post("/users/login")
//     //     .json(&json!({
//     //             "username": name,
//     //             "password": "testpassword"}))
//     //     .remote("127.0.0.1:8000".parse().unwrap())
//     //     .dispatch();
//     // assert_eq!(response.status(), Status::Ok);

//     // let response = client.post("/storage/images")
//     //     .header(ContentType::JPEG)
//     //     .body(r#"a;fklakdjfaoi;jflkasfasokfd"#)
//     //     .remote("127.0.0.1:8000".parse().unwrap())
//     //     .dispatch();
//     // assert_eq!(response.status(), Status::Ok);
//     // let image_name = response.into_string().unwrap();
//     // println!("{}", &image_name);

//     fn construct_headers() -> HeaderMap {
//         let mut headers = HeaderMap::new();
//         headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/jpeg"));
//         headers
//     }
//     let mut response = reqwest::blocking::Client::new()
//         .get("http://httpbin.org/image/jpeg")
//         .send()
//         .unwrap();
//     // if response.status().is_success() {
//     //     println!("{:?}", response.headers());
//     // }
//     let mut buf: Vec<u8> = vec![];
//     response.copy_to(&mut buf).unwrap();
//     let client = reqwest::blocking::Client::builder()
//         .cookie_store(true)
//         .build()
//         .unwrap();
//     client
//         .post("https://dev.thuburrow.com/users/login")
//         .json(&json!({
//             "username": "name1",
//             "password": "testpassword"}))
//         .send()
//         .unwrap();
//     let response = client
//         .post("https://dev.thuburrow.com/storage/images")
//         .headers(construct_headers())
//         .body(buf)
//         .send()
//         .unwrap();
//     println!("{:?}", response.text().unwrap());
// }
