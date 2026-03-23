use anyhow::{Result, bail};
use serde_json::Value;

use crate::{
    commands::account::{build_client, load_account},
    errors::Error,
};

pub async fn run(user_id: Option<&str>) -> Result<()> {
    let session = load_account(user_id)?;
    let client = build_client(&session);
    let user_id = session.user_id;
    let bot_id = session.data.bot_id;
    let mut consecutive_errors = 0u32;

    eprintln!(
        "waiting for the bound user to send a message for `{user_id}`; press Ctrl+C to stop"
    );

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

                if let Some(messages) = resp["msg_list"]
                    .as_array()
                    .or_else(|| resp["msgs"].as_array())
                {
                    for message in messages {
                        if let Some(context_token) = extract_context_token(&bot_id, message) {
                            println!("{context_token}");
                            return Ok(());
                        }
                    }
                }
            }
            Err(Error::SessionExpired) => {
                bail!("session expired for user `{user_id}`, re-run `wechat-cli login`");
            }
            Err(Error::Http { source }) if source.is_timeout() => {}
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

fn extract_context_token(bot_id: &str, message: &Value) -> Option<String> {
    let from_user_id = message["from_user_id"].as_str().unwrap_or("");
    let to_user_id = message["to_user_id"].as_str().unwrap_or("");
    let context_token = message["context_token"].as_str().unwrap_or("");

    if context_token.is_empty() {
        return None;
    }

    let is_user_to_bot = to_user_id == bot_id && !from_user_id.is_empty();
    let is_bot_to_user = from_user_id == bot_id && !to_user_id.is_empty();

    if is_user_to_bot || is_bot_to_user {
        Some(context_token.to_string())
    } else {
        None
    }
}
