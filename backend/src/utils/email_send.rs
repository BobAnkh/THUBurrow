use reqwest::header::HeaderMap;
use serde::Serialize;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use chrono::Utc;
use rustc_serialize::hex::ToHex;

pub static SECRET_ID: &str = "AKIDRhhQt50e290bw0nIWUBaiaBVYJo30g5D";
pub static SECRET_KEY: &str = "rRloZ3GCqDJ9fwwg8KFsEJ7dttb2D7k7";

#[allow(non_snake_case)]
#[derive(Serialize)]
struct Template {
    TemplateID: i32,
    TemplateData: String,
}
#[allow(non_snake_case)]
#[derive(Serialize)]
struct Body {
    FromEmailAddress: String,
    ReplyToAddresses: String,
    Destination: Vec<String>,
    Template: Template,
    Subject: String,
}

fn gen_auth(param: &Body, timestamp: String) -> String {
    // 密钥参数
    let secret_id = SECRET_ID;
    let secret_key = SECRET_KEY;
    let host = "ses.tencentcloudapi.com".to_string();
    let date = Utc::now().format("%Y-%m-%d").to_string();
    let service = "ses".to_string();
    let algorithm = "TC3-HMAC-SHA256";
    // step 1: 拼接规范请求串
    let http_request_method = "POST".to_string();
    let canonical_uri = "/".to_string();
    let canonical_querystring = "".to_string();
    let ct = "application/json; charset=utf-8".to_string();
    let payload_str = serde_json::to_string(param).unwrap();
    let payload_vec: Vec<&str> = payload_str.split(":").collect();
    let payload = payload_vec.join(": ");
    let payload_vec: Vec<&str> = payload.split(",").collect();
    let payload = payload_vec.join(", ");
    println!("{}", payload);
    let canonical_headers = format!("content-type:{}\nhost:{}\n", ct, host);
    let signed_headers = "content-type;host".to_string();
    let mut hasher = Sha256::new();
    hasher.input_str(&payload);
    let hashed_request_payload = hasher.result_str();
    let canonical_request = format!("{}\n{}\n{}\n{}\n{}\n{}"
        , http_request_method
        , canonical_uri
        , canonical_querystring
        , canonical_headers
        , signed_headers
        , hashed_request_payload
    );
    println!("{}", canonical_request);

    // step 2: 拼接待签名字符串
    let credential_scope = format!("{}/{}/tc3_request", date, service);
    let mut hasher = Sha256::new();
    hasher.input_str(&canonical_request);
    let hashed_canonical_request = hasher.result_str();
    let string_to_sign = format!("{}\n{}\n{}\n{}"
        , algorithm
        , timestamp.to_string()
        , credential_scope
        , hashed_canonical_request
    );
    println!("{}", string_to_sign);

    // step 3: 计算签名
    fn sign<'a>(key: &'a [u8], msg: &'a [u8]) -> Vec<u8> {
        let mut hmac = Hmac::new(Sha256::new(), key);
        hmac.input(msg);
        let result = hmac.result();
        let code = result.code();
        code.to_vec()
    }
    let key = "TC3".to_string() + &secret_key;
    let secret_date = sign(key.as_bytes(), date.as_bytes());
    // println!("{}", secret_date.to_hex());
    let secret_service = sign(&secret_date, service.as_bytes());
    // println!("{}", secret_service.to_hex());
    let secret_signing = sign(&secret_service, "tc3_request".as_bytes());
    // println!("{}", secret_signing.to_hex());
    let mut hmac = Hmac::new(Sha256::new(), &secret_signing);
    hmac.input(string_to_sign.as_bytes());
    let signature = hmac.result().code().to_hex();
    println!("{}", signature);

    // step 4: 拼接 Authorization
    format!("{} Credential={}/{}, SignedHeaders={}, Signature={}"
        , algorithm
        , secret_id
        , credential_scope
        , signed_headers
        , signature
    )
}

pub async fn post(user_email: String, verification_code: i32) -> Result<String, reqwest::Error>{
    // create client
    let client = reqwest::Client::new();

    // generate header
    let mut headers = HeaderMap::new();
    let timestamp = Utc::now().timestamp().to_string();
    // let timestamp = 1638791815.to_string();
    let host = "ses.tencentcloudapi.com".to_string();
    let action = "SendEmail".to_string();
    let region = "ap-hongkong".to_string();
    let version = "2020-10-02".to_string();
    headers.insert("Host", "ses.tencentcloudapi.com".parse().unwrap());
    headers.insert("Content-Type", "application/json; charset=utf-8".parse().unwrap());
    headers.insert("X-TC-Action", "SendEmail".parse().unwrap());
    headers.insert("X-TC-Timestamp", timestamp.parse().unwrap());
    headers.insert("X-TC-Version", "2020-10-02".parse().unwrap());
    headers.insert("X-TC-Region", "ap-hongkong".parse().unwrap());

    // generate body data
    // let verification_code = 123456;
    let body = Body {
        FromEmailAddress: "THUBurrow <noreply@testmail.thuburrow.com>".to_string(),
        ReplyToAddresses: "1194392480@qq.com".to_string(),
        // Destination: vec!["gsr18@mails.tsinghua.edu.cn".to_string()],
        Destination: vec![user_email],
        Template: Template {
            TemplateID: 21517,
            TemplateData: format!("{{\\\"code\\\":{}}}", verification_code),
        },
        Subject: "Verification Email".to_string(),
    };

    let payload_str = serde_json::to_string(&body).unwrap();
    let payload_vec: Vec<&str> = payload_str.split(":").collect();
    let payload = payload_vec.join(": ");
    let payload_vec: Vec<&str> = payload.split(",").collect();
    let payload = payload_vec.join(", ");

    // generate authorization, set header
    let auth = gen_auth(&body, timestamp.clone());
    headers.insert("Authorization",auth.parse().unwrap());
    let req = format!("\ncurl -X POST https://{} -H \"Authorization: {}\" -H \"Content-Type: application/json; charset=utf-8\" -H \"Host: {}\" -H \"X-TC-Action: {}\" -H \"X-TC-Timestamp: {}\" -H \"X-TC-Version: {}\" -H \"X-TC-Region: {}\" -d \'{}\'"
        , host
        , auth
        , host
        , action
        , timestamp
        , version
        , region
        , payload
    );
    println!("{}", req);

    // send request
    let response = client.post("https://ses.tencentcloudapi.com").headers(headers).body(payload).send().await?;
    // println!("{}", response);
    Ok(response.text().await?)
}
