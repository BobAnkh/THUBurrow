use check_if_email_exists::syntax::check_syntax;
use check_if_email_exists::{check_email, CheckEmailInput, Reachable};
use lazy_static::lazy_static;

lazy_static! {
    static ref MAIL_DOMAINS: Vec<String> = vec![
        "mail.tsinghua.edu.cn".to_string(),
        "mails.tsinghua.edu.cn".to_string()
    ];
}

pub async fn check_email_exist(email_address: &str) -> (bool, String) {
    let mut input = CheckEmailInput::new(vec![email_address.into()]);

    input
        .set_from_email("hello@thuburrow.com".into())
        .set_hello_name("thuburrow.com".into());

    let result = check_email(&input).await;

    let result = match result.get(0) {
        Some(res) => res,
        None => {
            return (false, "Internal Server Error".to_string());
        }
    };
    if result.is_reachable == Reachable::Invalid
        || result.is_reachable == Reachable::Unknown
        || result.syntax.is_valid_syntax == false
    {
        return (false, "Email Address not Exists".to_string());
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
                return (false, "Mx Records not Exists".to_string());
            }
        }
        Err(_) => {
            return (false, "Internal Server Error".to_string());
        }
    }
    match result.misc {
        Ok(ref misc) => {
            if misc.is_disposable || misc.is_role_account {
                return (false, "Misc Invalid".to_string());
            }
        }
        Err(_) => {
            return (false, "Internal Server Error".to_string());
        }
    }
    match result.smtp {
        Ok(ref smtp) => {
            if !smtp.can_connect_smtp
                || smtp.has_full_inbox
                || !smtp.is_deliverable
                || smtp.is_disabled
            {
                return (false, "Smtp Unreachable".to_string());
            }
        }
        Err(_) => {
            return (false, "Internal Server Error".to_string());
        }
    }
    (true, "".to_string())
}

pub fn check_email_syntax(email_address: &str) -> bool {
    let syntax_result = check_syntax(email_address);
    if syntax_result.is_valid_syntax {
        MAIL_DOMAINS.contains(&syntax_result.domain)
    } else {
        false
    }
}
