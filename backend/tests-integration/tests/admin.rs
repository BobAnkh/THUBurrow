use backend::utils::mq::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Status;
use serde_json::json;
use tests_integration::get_client;
use tokio::runtime::Runtime;

#[test]
fn test_admin() {
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
        .take(16)
        .collect();
    // generate a random name
    let new_name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(16)
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
    // create post 1
    let response = client
        .post("/content/posts")
        .json(&json!({
            "title": format!("First post of {}", name),
            "burrow_id": burrow_id,
            "section": ["Learning"],
            "tag": ["AdminTag"],
            "content": "This is a test post no.1"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response
        .into_json::<backend::models::content::PostCreateResponse>()
        .unwrap();
    let post_id = res.post_id;
    let response = client
        .post("/admin")
        .json(&json!({ "GetUserId": {"burrow_id": burrow_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    // user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // Set up the admin account
    // set verification code
    client
        .post("/users/email")
        .json(&json!({
            "email": format!("{}@mails.tsinghua.edu.cn", new_name)
        }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    std::thread::sleep(std::time::Duration::from_secs(1));
    // sign up a user
    let response = client
        .post("/users/sign-up")
        .json(&json!({
            "username": new_name,
            "password": "testpassword",
            "email": format!("{}@mails.tsinghua.edu.cn", new_name),
            "verification_code": "666666"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": new_name,
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

    // Reopen the burrow with burrow_id
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenBurrow": {"burrow_id": burrow_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    // Ban the post with post_id
    let response = client
        .post("/admin")
        .json(&json!({ "BanPost": {"post_id": post_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // Reopen the post with post_id
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenPost": {"post_id": post_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // Ban the reply with post_id and reply_id
    let response = client
        .post("/admin")
        .json(&json!({ "BanReply": {"post_id": post_id, "reply_id": 0} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // Reopen the reply with post_id and reply_id
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenReply": {"post_id": post_id, "reply_id": 0} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    // Create Admin
    let response = client
        .post("/admin")
        .json(&json!({ "CreateAdmin": {"uid": uid} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // Set Admin Role
    let response = client
        .post("/admin")
        .json(&json!({ "SetAdminRole": {"uid": uid, "role": 2} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");
    // Delete Admin
    let response = client
        .post("/admin")
        .json(&json!({ "DeleteAdmin": {"uid": uid} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    let response = client
        .post("/admin")
        .json(&json!({ "CreateAdmin": {"uid": uid} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    // user log out
    let response = client
        .get("/users/logout")
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

    // Ban the user with uid
    let response = client
        .post("/admin")
        .json(&json!({ "BanUser": {"uid": uid} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    // Reopen the user with uid
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenUser": {"uid": uid} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    // Ban the burrow with burrow_id
    let response = client
        .post("/admin")
        .json(&json!({ "BanBurrow": {"burrow_id": burrow_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);

    // Reopen the burrow with burrow_id
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenBurrow": {"burrow_id": burrow_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);

    // Ban the post with post_id
    let response = client
        .post("/admin")
        .json(&json!({ "BanPost": {"post_id": post_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    // Reopen the post with post_id
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenPost": {"post_id": post_id} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    // Ban the reply with post_id and reply_id
    let response = client
        .post("/admin")
        .json(&json!({ "BanReply": {"post_id": post_id, "reply_id": 0} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    // Reopen the reply with post_id and reply_id
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenReply": {"post_id": post_id, "reply_id": 0} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);

    // Ban the user with uid
    let response = client
        .post("/admin")
        .json(&json!({ "BanUser": {"uid": uid+10000} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    // Reopen the user with uid
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenUser": {"uid": uid+10000} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    // Ban the burrow with burrow_id
    let response = client
        .post("/admin")
        .json(&json!({ "BanBurrow": {"burrow_id": burrow_id+10000} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);

    // Reopen the burrow with burrow_id
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenBurrow": {"burrow_id": burrow_id+10000} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);

    // Ban the post with post_id
    let response = client
        .post("/admin")
        .json(&json!({ "BanPost": {"post_id": post_id+10000} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    // Reopen the post with post_id
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenPost": {"post_id": post_id+10000} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    // Ban the reply with post_id and reply_id
    let response = client
        .post("/admin")
        .json(&json!({ "BanReply": {"post_id": post_id+10000, "reply_id": 0} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    // Reopen the reply with post_id and reply_id
    let response = client
        .post("/admin")
        .json(&json!({ "ReopenReply": {"post_id": post_id+10000, "reply_id": 0} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);

    // Create Admin
    let response = client
        .post("/admin")
        .json(&json!({ "CreateAdmin": {"uid": uid} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    // Set Admin Role
    let response = client
        .post("/admin")
        .json(&json!({ "SetAdminRole": {"uid": uid, "role": 2} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    let response = client
        .post("/admin")
        .json(&json!({ "SetAdminRole": {"uid": uid+10000, "role": 2} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    // Delete Admin
    let response = client
        .post("/admin")
        .json(&json!({ "DeleteAdmin": {"uid": uid} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    let response = client
        .post("/admin")
        .json(&json!({ "DeleteAdmin": {"uid": uid+10000} }))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);

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
