use rocket::http::Status;
use tests_integration::get_client;

#[test]
fn test_connected() {
    let client = get_client().lock();
    let response = client
        .get("/health")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Ok");
    let response = client
        .get("/content/posts/undefined?page=0")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Ok");
    let response = client
        .get("/burrows/undefined?page=0")
        .remote("127.0.0.1:8000".parse().unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Ok");
}
