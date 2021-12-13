use check_if_email_exists::syntax::check_syntax;
use check_if_email_exists::{check_email, CheckEmailInput, Reachable};
use lazy_static::lazy_static;

lazy_static! {
    static ref MAIL_DOMAINS: Vec<String> = vec![
        "tsinghua.edu.cn".to_string(),
        "mail.tsinghua.edu.cn".to_string(),
        "mails.tsinghua.edu.cn".to_string()
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
}
