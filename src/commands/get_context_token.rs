use anyhow::{Result, bail};

use crate::{
    commands::send::{SendTarget, resolve_send_target},
    wechat::api::is_session_expired,
    wechat::models::InboundMessage,
};

pub async fn run(
    account: Option<usize>,
    user_id: Option<&str>,
    bot_token: Option<&str>,
    route_tag: Option<&str>,
) -> Result<()> {
    let target = resolve_send_target(account, user_id, bot_token, route_tag)?;

    let (user_id, client) = match target {
        SendTarget::Saved { user_id, client } => (user_id, client),
        SendTarget::Explicit { user_id, client } => (user_id, client),
    };

    let mut consecutive_errors = 0u32;

    eprintln!("waiting for the bound user to send a message for `{user_id}`; press Ctrl+C to stop");

    loop {
        let result = tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                eprintln!("stopped");
                return Ok(());
            }
            result = client.get_updates(None) => result,
        };

        match result {
            Ok(resp) => {
                consecutive_errors = 0;

                for message in resp.messages() {
                    if let Some(context_token) = extract_context_token(&user_id, message) {
                        println!("{context_token}");
                        return Ok(());
                    }
                }
            }
            Err(err) if is_session_expired(&err) => {
                bail!("session expired for user `{user_id}`, re-run `wechat-cli login`");
            }
            Err(err) if is_timeout_error(&err) => {}
            Err(err) => {
                consecutive_errors += 1;
                eprintln!("get-context-token error ({consecutive_errors}): {err}");
                if consecutive_errors >= 3 {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                    consecutive_errors = 0;
                } else {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    }
}

fn is_timeout_error(err: &anyhow::Error) -> bool {
    err.chain()
        .find_map(|cause| cause.downcast_ref::<reqwest::Error>())
        .is_some_and(reqwest::Error::is_timeout)
}

fn extract_context_token(user_id: &str, message: &InboundMessage) -> Option<String> {
    if message.context_token.is_empty() {
        return None;
    }

    if message.from_user_id == user_id && !message.to_user_id.is_empty() {
        Some(message.context_token.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::send::resolve_send_target;

    #[test]
    fn test_shared_resolve_rejects_mixed_modes() {
        let result = resolve_send_target(Some(0), None, Some("token"), None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("cannot be used with"),
            "Expected error about mixing modes, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_shared_resolve_rejects_missing_bot_token() {
        let result = resolve_send_target(None, Some("user@im.wechat"), None, None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("bot-token"),
            "Expected error about missing bot-token, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_shared_resolve_rejects_missing_user_id() {
        let result = resolve_send_target(None, None, Some("token"), None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("user-id"),
            "Expected error about missing user-id, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_shared_resolve_rejects_account_with_user_id() {
        let result = resolve_send_target(Some(0), Some("user@im.wechat"), None, None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("cannot be used with"),
            "Expected error about cannot use together, got: {}",
            err_msg
        );
    }
}
