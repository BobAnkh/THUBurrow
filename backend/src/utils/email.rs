use check_if_email_exists::syntax::check_syntax;
use check_if_email_exists::{check_email, CheckEmailInput, Reachable};
use chrono::Utc;
use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use hex;
use lazy_static::lazy_static;
use reqwest::header::HeaderMap;
use serde::Serialize;

use crate::config::email::*;

lazy_static! {
    static ref MAIL_DOMAINS: Vec<String> = vec![
        "tsinghua.edu.cn".to_string(),
        "mail.tsinghua.edu.cn".to_string(),
        "mails.tsinghua.edu.cn".to_string(),
        "thuburrow.com".to_string(),
    ];
}

#[derive(Debug, Clone)]
pub enum EmailExistMessage {
    EmailExist,
    EmailNotExist,
    EmailSyntaxError,
    MxNotExist,
    MiscInvalid,
    SmtpUnreachable,
    InternalServerError,
}

pub async fn check_email_exist(email_address: &str) -> (bool, EmailExistMessage) {
    let mut input = CheckEmailInput::new(vec![email_address.into()]);

    input
        .set_from_email("hello@thuburrow.com".into())
        .set_hello_name("thuburrow.com".into());

    let result = check_email(&input).await;

    let result = match result.get(0) {
        Some(res) => res,
        None => {
            return (false, EmailExistMessage::InternalServerError);
        }
    };
    if !result.syntax.is_valid_syntax {
        return (false, EmailExistMessage::EmailSyntaxError);
    }
    if result.is_reachable == Reachable::Invalid
        || result.is_reachable == Reachable::Unknown
        || !result.syntax.is_valid_syntax
    {
        return (false, EmailExistMessage::EmailNotExist);
    }
    match result.mx {
        Ok(ref mx) => {
            let records = mx
                .lookup
                .as_ref()
                .map(|lookup| {
                    lookup
                        .iter()
                        .map(|host| host.exchange().to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|_| Vec::new());
            if records.is_empty() {
                return (false, EmailExistMessage::MxNotExist);
            }
        }
        Err(_) => {
            return (false, EmailExistMessage::InternalServerError);
        }
    }
    match result.misc {
        Ok(ref misc) => {
            if misc.is_disposable || misc.is_role_account {
                return (false, EmailExistMessage::MiscInvalid);
            }
        }
        Err(_) => {
            return (false, EmailExistMessage::InternalServerError);
        }
    }
    match result.smtp {
        Ok(ref smtp) => {
            if !smtp.can_connect_smtp
                || smtp.has_full_inbox
                || !smtp.is_deliverable
                || smtp.is_disabled
            {
                return (false, EmailExistMessage::SmtpUnreachable);
            }
        }
        Err(_) => {
            return (false, EmailExistMessage::InternalServerError);
        }
    }
    (true, EmailExistMessage::EmailExist)
}

pub fn check_email_syntax(email_address: &str) -> bool {
    let syntax_result = check_syntax(email_address);
    if syntax_result.is_valid_syntax {
        MAIL_DOMAINS.contains(&syntax_result.domain)
    } else {
        false
    }
}

#[derive(Serialize)]
pub struct Template {
    #[serde(rename = "TemplateID")]
    pub template_id: i32,
    #[serde(rename = "TemplateData")]
    pub template_data: String,
}

#[derive(Serialize)]
pub struct Body {
    #[serde(rename = "FromEmailAddress")]
    pub from_email_address: String,
    #[serde(rename = "Destination")]
    pub destination: Vec<String>,
    #[serde(rename = "Template")]
    pub template: Template,
    #[serde(rename = "Subject")]
    pub subject: String,
}

fn hash(s: String) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(&s);
    hasher.result_str()
}

pub fn get_payload(param: &Body) -> String {
    let payload_str = serde_json::to_string(param).unwrap();
    let payload_vec: Vec<&str> = payload_str.split(':').collect();
    let payload = payload_vec.join(": ");
    let payload_vec: Vec<&str> = payload.split(',').collect();
    payload_vec.join(", ")
}

pub fn sign<'a>(key: &'a [u8], msg: &'a [u8]) -> Vec<u8> {
    let mut hmac = Hmac::new(Sha256::new(), key);
    hmac.input(msg);
    let result = hmac.result();
    let code = result.code();
    code.to_vec()
}

pub fn assemble_headers(timestamp: String) -> HeaderMap {
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

pub fn signature(param: &Body, timestamp: String, date: String) -> String {
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
    let payload = get_payload(param);
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

pub async fn send(user_email: String, verification_code: String) -> Result<String, reqwest::Error> {
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

    // operation type
    let operation: String = match verification_code.len() {
        6 => "注册".to_string(),
        _ => "找回密码".to_string(),
    };

    // generate body data
    let body = Body {
        from_email_address: "THUBurrow <no-reply@mail.thuburrow.com>".to_string(),
        destination: vec![user_email],
        template: Template {
            template_id: 22078,
            template_data: format!(
                "{{\\\"operation\\\":\"{}\",\\\"code\\\":\"{}\"}}",
                operation, verification_code
            ),
        },
        subject: "Verification Email".to_string(),
    };
    let payload = get_payload(&body);
    // println!("{}", payload);

    // generate authorization, set header
    let auth = signature(&body, timestamp.clone(), date.clone());
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
    fn test_check_email_syntax() {
        assert_eq!(check_email_syntax(""), false);
        assert_eq!(check_email_syntax("a"), false);
        assert_eq!(check_email_syntax("a@"), false);
        assert_eq!(check_email_syntax("a@b"), false);
        assert_eq!(check_email_syntax("a@b."), false);
        assert_eq!(check_email_syntax("test@mails.tsinghua.edu.cn"), true);
        assert_eq!(check_email_syntax("@mails.tsinghua.edu.cn"), false);
        assert_eq!(check_email_syntax("test@163.com"), false);
        assert_eq!(check_email_syntax("test()@mails.tsinghua.edu.cn"), false);
        assert_eq!(check_email_syntax("sys-learn2018@tsinghua.edu.cn"), true);
        assert_eq!(check_email_syntax("shetuan@mail.tsinghua.edu.cn"), true);
    }

    #[tokio::test]
    async fn test_check_email_exist() {
        assert_eq!(check_email_exist("").await.0, false);
        assert_eq!(
            check_email_exist("test@mails.tsinghua.edu.cn").await.0,
            false
        );
        assert_eq!(check_email_exist("test@163.com").await.0, false);
        assert_eq!(
            check_email_exist("test()@mails.tsinghua.edu.cn").await.0,
            false
        );
        assert_eq!(
            check_email_exist("sys-learn2018@tsinghua.edu.cn").await.0,
            true
        );
        assert_eq!(
            check_email_exist("shetuan@mail.tsinghua.edu.cn").await.0,
            true
        );
    }

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
        println!("{}", get_payload(&param));
        assert_eq!(get_payload(&param), r#"{"FromEmailAddress": "THUBurrow <noreply@testmail.thuburrow.com>", "Destination": ["abc@qq.com"], "Template": {"TemplateID": 21517, "TemplateData": "{\\\"code\\\": \"abc123\"}"}, "Subject": "Verification Email"}"#.to_string());
    }

    #[test]
    fn test_signature() {
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
        println!("{}", signature(&param, timestamp.clone(), date.clone()));
        assert_eq!(signature(&param, timestamp, date), "TC3-HMAC-SHA256 Credential=/2021-12-06/ses/tc3_request, SignedHeaders=content-type;host, Signature=8cd08830134ead51d3b488e84a14a148131caa5a81fa29a8366370ba39a6eb18".to_string());
    }
}
