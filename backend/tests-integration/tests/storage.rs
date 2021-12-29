use backend::utils::mq::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::StatusCode;
use rocket::http::{ContentType, Header, Status};
use serde_json::json;
use tests_integration::get_client;
use tokio::runtime::Runtime;

#[test]
fn test_storage() {
    // ---------- Prepare ----------
    // Init background task executor
    let client = get_client().lock();
    let rt = Runtime::new().unwrap();
    let h4 = rt.spawn(pulsar_email());
    std::thread::sleep(std::time::Duration::from_secs(1));
    // generate a random name
    let name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(15)
        .collect();
    let admin_name: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(15)
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

    // user login
    let response = client
        .post("/users/login")
        .json(&json!({
            "username": name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    //get an jepg from httpbin
    let mut jpeg_buf: Vec<u8> = vec![];
    let jpeg: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(1600)
        .collect();
    match reqwest::blocking::Client::new()
        .get("http://httpbin.org/image/jpeg")
        .send()
    {
        Ok(mut r) => match r.status() {
            StatusCode::OK => {
                r.copy_to(&mut jpeg_buf).unwrap();
            }
            _ => {
                jpeg_buf = jpeg.into_bytes();
            }
        },
        Err(_) => {
            jpeg_buf = jpeg.into_bytes();
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
    let png: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(1600)
        .collect();
    match reqwest::blocking::Client::new()
        .get("http://httpbin.org/image/jpeg")
        .send()
    {
        Ok(mut r) => match r.status() {
            StatusCode::OK => {
                r.copy_to(&mut png_buf).unwrap();
            }
            _ => {
                png_buf = png.into_bytes();
            }
        },
        Err(_) => {
            png_buf = png.into_bytes();
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
    assert_eq!(response.status(), Status::Forbidden);
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

    // store a jpeg
    let response = client
        .post("/storage/images")
        .header(ContentType::JPEG)
        .body(jpeg_buf.clone())
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    // store a jpeg
    let response = client
        .post("/storage/images")
        .header(ContentType::JPEG)
        .body(jpeg_buf.clone())
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::TooManyRequests);

    let empty_image: Vec<u8> = Vec::new();
    // store a jpeg
    let response = client
        .post("/storage/images")
        .header(ContentType::JPEG)
        .body(empty_image)
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);

    // user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

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
            "username": admin_name,
            "password": "testpassword"}))
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    let response = client
        .get("/admin/test?role=3")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Success");

    //list image
    let response = client
        .get("/storage/images")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let res = response.into_string().unwrap();
    println!("{}", res);

    // user log out
    let response = client
        .get("/users/logout")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // ---------- Clean up ----------
    h4.abort();
    std::thread::sleep(std::time::Duration::from_secs(1));
    // ---------- Clean up ----------
}
