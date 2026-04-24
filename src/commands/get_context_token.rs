use anyhow::{Context, Result, anyhow, bail};

use crate::{
    commands::account::{build_client, resolve_user_id},
    storage::{self, load_account},
    wechat::api::{WeixinApiClient, is_session_expired},
    wechat::models::InboundMessage,
};

pub async fn run(
    account: Option<usize>,
    user_id: Option<&str>,
    bot_token: Option<&str>,
    route_tag: Option<&str>,
) -> Result<()> {
    let target = resolve_target(account, user_id, bot_token, route_tag)?;
    let (user_id, client) = match target {
        Target::Saved { user_id, client } => (user_id, client),
        Target::Explicit { user_id, client } => (user_id, client),
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

#[derive(Debug)]
enum Target {
    Saved { user_id: String, client: WeixinApiClient },
    Explicit { user_id: String, client: WeixinApiClient },
}

fn resolve_target(
    account: Option<usize>,
    user_id: Option<&str>,
    bot_token: Option<&str>,
    route_tag: Option<&str>,
) -> Result<Target> {
    let using_explicit = user_id.is_some() || bot_token.is_some();

    if using_explicit {
        if account.is_some() {
            bail!("`--account` cannot be used with `--bot-token` / `--user-id`");
        }
        let user_id = user_id
            .ok_or_else(|| anyhow!("`--user-id` is required in explicit credential mode"))?
            .to_string();
        let bot_token = bot_token
            .ok_or_else(|| anyhow!("`--bot-token` is required in explicit credential mode"))?
            .to_string();

        return Ok(Target::Explicit {
            user_id,
            client: WeixinApiClient::new(&bot_token, route_tag.map(str::to_string)),
        });
    }

    if let Some(idx) = account {
        let data = load_account(idx)?;
        let user_id = data.user_id.clone();
        return Ok(Target::Saved {
            user_id,
            client: build_client(&data),
        });
    }

    let resolved_id = resolve_user_id(None)?;
    let session = storage::get_account_data(&resolved_id)
        .with_context(|| format!("failed to load account data for `{resolved_id}`"))?;
    Ok(Target::Saved {
        user_id: resolved_id,
        client: build_client(&session),
    })
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
    use super::*;

    #[test]
    fn test_account_with_explicit_creds_is_rejected() {
        let result = resolve_target(Some(0), None, Some("token"), None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("cannot be used with"),
            "Expected error about mixing modes, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_explicit_creds_missing_bot_token_fails() {
        let result = resolve_target(None, Some("user@im.wechat"), None, None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("bot-token"),
            "Expected error about missing bot-token, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_explicit_creds_missing_user_id_fails() {
        let result = resolve_target(None, None, Some("token"), None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("user-id"),
            "Expected error about missing user-id, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_account_with_user_id_is_rejected() {
        let result = resolve_target(Some(0), Some("user@im.wechat"), None, None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("cannot be used with"),
            "Expected error about cannot use together, got: {}",
            err_msg
        );
    }

}
