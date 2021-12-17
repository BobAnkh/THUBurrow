use chrono::Utc;
use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use hex;
use lazy_static::lazy_static;
use reqwest::header::HeaderMap;
use serde::Serialize;
use std::env;

lazy_static! {
    static ref SECRET_ID: String = env::var("SECRET_ID").ok().unwrap_or_else(|| "".to_string());
    static ref SECRET_KEY: String = env::var("SECRET_KEY")
        .ok()
        .unwrap_or_else(|| "".to_string());
}

#[derive(Serialize)]
struct Template {
    #[serde(rename = "TemplateID")]
    template_id: i32,
    #[serde(rename = "TemplateData")]
    template_data: String,
}

#[derive(Serialize)]
struct Body {
    #[serde(rename = "FromEmailAddress")]
    from_email_address: String,
    #[serde(rename = "Destination")]
    destination: Vec<String>,
    #[serde(rename = "Template")]
    template: Template,
    #[serde(rename = "Subject")]
    subject: String,
}

fn hash(s: String) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(&s);
    hasher.result_str()
}

fn gen_payload(param: &Body) -> String {
    let payload_str = serde_json::to_string(param).unwrap();
    let payload_vec: Vec<&str> = payload_str.split(':').collect();
    let payload = payload_vec.join(": ");
    let payload_vec: Vec<&str> = payload.split(',').collect();
    payload_vec.join(", ")
}

fn sign<'a>(key: &'a [u8], msg: &'a [u8]) -> Vec<u8> {
    let mut hmac = Hmac::new(Sha256::new(), key);
    hmac.input(msg);
    let result = hmac.result();
    let code = result.code();
    code.to_vec()
}

fn assemble_headers(timestamp: String) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Host", "ses.tencentcloudapi.com".parse().unwrap());
    headers.insert(
        "Content-Type",
        "application/json; charset=utf-8".parse().unwrap(),
    );
    headers.insert("X-TC-Action", "SendEmail".parse().unwrap());
    headers.insert("X-TC-Timestamp", timestamp.parse().unwrap());
    headers.insert("X-TC-Version", "2020-10-02".parse().unwrap());
    headers.insert("X-TC-Region", "ap-hongkong".parse().unwrap());
    headers
}

fn gen_auth(param: &Body, timestamp: String, date: String) -> String {
    // define signature parameters
    let secret_id = &*SECRET_ID;
    let secret_key = &*SECRET_KEY;
    let host = "ses.tencentcloudapi.com".to_string();

    let service = "ses".to_string();
    let algorithm = "TC3-HMAC-SHA256";
    // step 1: attach standard request string
    let http_request_method = "POST".to_string();
    let canonical_uri = "/".to_string();
    let canonical_querystring = "".to_string();
    let ct = "application/json; charset=utf-8".to_string();
    let canonical_headers = format!("content-type:{}\nhost:{}\n", ct, host);
    let signed_headers = "content-type;host".to_string();
    let payload = gen_payload(param);
    let hashed_request_payload = hash(payload);

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        http_request_method,
        canonical_uri,
        canonical_querystring,
        canonical_headers,
        signed_headers,
        hashed_request_payload
    );
    // println!("{}", canonical_request);

    // step 2: attach string to sign
    let credential_scope = format!("{}/{}/tc3_request", date, service);
    let hashed_canonical_request = hash(canonical_request);
    let string_to_sign = format!(
        "{}\n{}\n{}\n{}",
        algorithm, timestamp, credential_scope, hashed_canonical_request
    );
    // println!("{}", string_to_sign);

    // step 3: generate signature
    let key = "TC3".to_string() + secret_key;
    let secret_date = sign(key.as_bytes(), date.as_bytes());
    let secret_service = sign(&secret_date, service.as_bytes());
    let secret_signing = sign(&secret_service, "tc3_request".as_bytes());
    let mut hmac = Hmac::new(Sha256::new(), &secret_signing);
    hmac.input(string_to_sign.as_bytes());
    let signature = hex::encode(hmac.result().code());
    // println!("{}", signature);

    // step 4: attach Authorization
    format!(
        "{} Credential={}/{}, SignedHeaders={}, Signature={}",
        algorithm, secret_id, credential_scope, signed_headers, signature
    )
}

pub async fn post(user_email: String, verification_code: &str) -> Result<String, reqwest::Error> {
    // create client
    let client = reqwest::Client::new();
    let now = Utc::now();
    let timestamp = now.timestamp().to_string();
    let date = now.format("%Y-%m-%d").to_string();
    // let timestamp = 1638791815.to_string();
    // let host = "ses.tencentcloudapi.com".to_string();
    // let action = "SendEmail".to_string();
    // let region = "ap-hongkong".to_string();
    // let version = "2020-10-02".to_string();

    // generate header
    let mut headers = assemble_headers(timestamp.clone());

    // generate body data
    // let verification_code = 123456;
    let body = Body {
        from_email_address: "THUBurrow <no-reply@mail.thuburrow.com>".to_string(),
        destination: vec![user_email],
        template: Template {
            template_id: 21517,
            template_data: format!("{{\\\"code\\\":\"{}\"}}", verification_code),
        },
        subject: "Verification Email".to_string(),
    };
    let payload = gen_payload(&body);
    // println!("{}", payload);

    // generate authorization, set header
    let auth = gen_auth(&body, timestamp.clone(), date.clone());
    headers.insert("Authorization", auth.parse().unwrap());
    // let req = format!("\ncurl -X POST https://{} -H \"Authorization: {}\" -H \"Content-Type: application/json; charset=utf-8\" -H \"Host: {}\" -H \"X-TC-Action: {}\" -H \"X-TC-Timestamp: {}\" -H \"X-TC-Version: {}\" -H \"X-TC-Region: {}\" -d \'{}\'"
    //     , host
    //     , auth
    //     , host
    //     , action
    //     , timestamp
    //     , version
    //     , region
    //     , payload
    // );
    // println!("{}", req);
    // send request
    let response = client
        .post("https://ses.tencentcloudapi.com")
        .headers(headers)
        .body(payload)
        .send()
        .await?;
    // println!("{}", response);
    Ok(response.text().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_hex() {
        assert_eq!(hex::encode(b"hello"), "68656c6c6f".to_string());
    }

    #[test]
    fn test_sign() {
        assert_eq!(
            sign(b"key", b"msg"),
            hex::decode("2d93cbc1be167bcb1637a4a23cbff01a7878f0c50ee833954ea5221bb1b8c628")
                .unwrap()
        );
    }

    #[test]
    fn test_assemble_headers() {
        let timestamp = "1638791815".to_string();
        let headers = assemble_headers(timestamp);
        assert_eq!(headers["Host"], "ses.tencentcloudapi.com");
        assert_eq!(headers["Content-Type"], "application/json; charset=utf-8");
        assert!(headers.contains_key("X-TC-Timestamp"));
        assert_eq!(headers["X-TC-Region"], "ap-hongkong");
        assert_eq!(headers["X-TC-Action"], "SendEmail");
        assert_eq!(headers["X-TC-Version"], "2020-10-02");
    }

    #[test]
    fn test_gen_payload() {
        let param = Body {
            from_email_address: "THUBurrow <noreply@testmail.thuburrow.com>".to_string(),
            destination: vec!["abc@qq.com".to_string()],
            template: Template {
                template_id: 21517,
                template_data: format!("{{\\\"code\\\":\"{}\"}}", "abc123"),
            },
            subject: "Verification Email".to_string(),
        };
        println!("{}", gen_payload(&param));
        assert_eq!(gen_payload(&param), r#"{"FromEmailAddress": "THUBurrow <noreply@testmail.thuburrow.com>", "Destination": ["abc@qq.com"], "Template": {"TemplateID": 21517, "TemplateData": "{\\\"code\\\": \"abc123\"}"}, "Subject": "Verification Email"}"#.to_string());
    }

    #[test]
    fn test_gen_auth() {
        let param = Body {
            from_email_address: "THUBurrow <noreply@testmail.thuburrow.com>".to_string(),
            destination: vec!["abc@qq.com".to_string()],
            template: Template {
                template_id: 21517,
                template_data: format!("{{\\\"code\\\":\"{}\"}}", "abc123"),
            },
            subject: "Verification Email".to_string(),
        };
        let timestamp = 1638791815.to_string();
        let date = "2021-12-06".to_string();
        println!("{}", gen_auth(&param, timestamp.clone(), date.clone()));
        assert_eq!(gen_auth(&param, timestamp, date), "TC3-HMAC-SHA256 Credential=/2021-12-06/ses/tc3_request, SignedHeaders=content-type;host, Signature=8cd08830134ead51d3b488e84a14a148131caa5a81fa29a8366370ba39a6eb18".to_string());
    }
}
