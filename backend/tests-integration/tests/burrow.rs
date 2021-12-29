use backend::models::error::*;
use backend::utils::mq::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Status;
use serde_json::json;
use tests_integration::get_client;
use tokio::runtime::Runtime;

#[test]
fn test_burrow() {
    // ---------- Prepare ----------
    // Init background task executor
    let client = get_client().lock();
    let rt = Runtime::new().unwrap();
    let h1 = rt.spawn(generate_trending());
    let h2 = rt.spawn(pulsar_relation());
    let h3 = rt.spawn(pulsar_typesense());
    let h4 = rt.spawn(pulsar_email());
    std::thread::sleep(std::time::Duration::from_secs(1));
    // generate a random name
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(12)
        .collect();
    // generate a random name
    let admin_name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(12)
        .collect();
    // ---------- Prepare ----------

    // set verification code
    client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    std::thread::sleep(std::time::Duration::from_secs(1));
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
    // Set up the admin account
    // set verification code
    client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", admin_name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    std::thread::sleep(std::time::Duration::from_secs(1));
    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": admin_name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", admin_name),
            "verification_code": "666666"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
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
    // user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // let response = client
    //     .post("/burrows")
    //     .json(&json!({
    //         "description": format!("First burrow of {}", name),
    //         "title": "Burrow 1"}))
    //     .remote("127.0.0.1:8000".parse().unwrap())
    //     .dispatch();
    // assert_eq!(response.status(), Status::Forbidden);
    // println!("{}", response.into_string().unwrap());
    std::thread::sleep(std::time::Duration::from_secs(2));

    // ---------- admin ----------
    // admin user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": admin_name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    let response = client
        .get("/admin/test?role=3")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // Get Uid of the burrow
    let response = client
        .post("/admin")
        .json(&json!({ "GetUserId": {"burrow_id": burrow_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let uid = response.into_json::<i64>().unwrap();
    // Ban the user with uid
    let response = client
        .post("/admin")
        .json(&json!({ "BanUser": {"uid": uid} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // admin user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // ---------- admin ----------

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
    // create burrow: perform a wrong action (User banned)
    let response = client
        .post("/burrows")
        .json(&json!({
            "description": format!("Second burrow of {}", name),
            "title": "Burrow 2"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::UserForbidden, "User not in a valid state",)
    );
    // admin user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    // ---------- admin ----------
    // admin user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": admin_name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // Reopen the user with uid
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenUser": {"uid": uid} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // admin user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // ---------- admin ----------

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
    std::thread::sleep(std::time::Duration::from_secs(2));
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
    std::thread::sleep(std::time::Duration::from_secs(2));
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
    std::thread::sleep(std::time::Duration::from_secs(2));
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
    std::thread::sleep(std::time::Duration::from_secs(2));
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
    std::thread::sleep(std::time::Duration::from_secs(2));
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
    assert_eq!(res[0].burrow.burrow_id, burrow_id + 1);
    assert_eq!(res[1].burrow.burrow_id, burrow_id);
    // unfollow burrow 2nd
    let response = client
        .post("/users/relation")
        .json(&json!({ "DeactivateFollow": burrow_id + 1 }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    std::thread::sleep(std::time::Duration::from_secs(1));
    // get following burrows of a user
    let response = client
        .get("/users/follow")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<Vec<backend::models::user::UserGetFollowResponse>>()
        .unwrap();
    assert_eq!(res.len(), 1);

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
        response.into_string().unwrap(),
        "{\"title\":\"默认洞\",\"description\":\"Hello World!\",\"posts\":[]}".to_string()
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

    // ---------- admin ----------
    // admin user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": admin_name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // Get Uid of the burrow
    let response = client
        .post("/admin")
        .json(&json!({ "GetUserId": {"burrow_id": burrow_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let uid = response.into_json::<i64>().unwrap();
    // Ban the user with uid
    let response = client
        .post("/admin")
        .json(&json!({ "BanUser": {"uid": uid} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // admin user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // ---------- admin ----------

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
    // update burrow: perform a wrong action (User banned)
    let response = client
        .patch(format!("/burrows/{}", burrow_id))
        .json(&json!({
            "description": format!("New Default burrow of {}", name),
            "title": "New Default burrow"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::UserForbidden, "User not in a valid state",)
    );
    // user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    // ---------- admin ----------
    // admin user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": admin_name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // Reopen the user with uid
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenUser": {"uid": uid} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // Ban the burrow with burrow_id
    let response = client
        .post("/admin")
        .json(&json!({ "BanBurrow": {"burrow_id": burrow_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // admin user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // ---------- admin ----------

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
    // update burrow: perform a wrong action (Invalid Burrow)
    let response = client
        .patch(format!("/burrows/{}", burrow_id))
        .json(&json!({
            "description": format!("New Default burrow of {}", name),
            "title": "New Default burrow"}))
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
    // check banned burrow title
    let response = client
        .get("/users/burrows")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<Vec<backend::models::burrow::BurrowMetadata>>()
        .unwrap();
    assert_eq!(res[4].title, "Admin has banned this burrow");
    // user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    // ---------- admin ----------
    // admin user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": admin_name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // Reopen the burrow with burrow_id
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenBurrow": {"burrow_id": burrow_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // admin user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // ---------- admin ----------

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
        response.into_string().unwrap(),
        format!("{{\"title\":\"New Default burrow\",\"description\":\"New Default burrow of {}\",\"posts\":[]}}", name)
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
    assert_eq!(res[0].burrow_id, burrow_id + 5);
    assert_eq!(res[4].burrow_id, burrow_id);

    // 9. test get_user_valid_burrow
    // get valid burrow of a user
    let response = client
        .get("/users/valid-burrows")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_string().unwrap(),
        format!(
            "[{},{},{},{},{}]",
            burrow_id,
            burrow_id + 2,
            burrow_id + 3,
            burrow_id + 4,
            burrow_id + 5
        )
    );

    // 10. test discard_burrow
    // ---------- admin ----------
    // admin user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": admin_name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // Ban the burrow with burrow_id
    let response = client
        .post("/admin")
        .json(&json!({ "BanBurrow": {"burrow_id": burrow_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // admin user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // ---------- admin ----------

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
            "title": ""}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_json::<ErrorResponse>().unwrap(),
        ErrorResponse::build(ErrorCode::EmptyField, "Burrow title cannot be empty",)
    );
    // user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // ---------- Clean up ----------
    h1.abort();
    h2.abort();
    h3.abort();
    h4.abort();
    std::thread::sleep(std::time::Duration::from_secs(1));
    // ---------- Clean up ----------
}
