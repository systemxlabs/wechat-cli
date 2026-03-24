use anyhow::{Result, bail};

use crate::{
    commands::account::{build_client, load_account},
    wechat::api::is_session_expired,
    wechat::models::InboundMessage,
};

pub async fn run(user_id: Option<&str>) -> Result<()> {
    let session = load_account(user_id)?;
    let client = build_client(&session);
    let user_id = session.user_id;
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
