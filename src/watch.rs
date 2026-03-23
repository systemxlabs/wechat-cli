use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::{
    account::{build_client, load_account},
    context::{self, ConversationContext},
    errors::Error,
    storage,
};

pub async fn get_context_token(user_id: Option<&str>) -> Result<()> {
    let session = load_account(user_id)?;
    let client = build_client(&session);
    let user_id = session.user_id;
    let bot_id = session.data.bot_id;
    let mut buf = storage::get_updates_buf(&user_id);
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
            result = client.get_updates(buf.as_deref()) => result,
        };

        match result {
            Ok(resp) => {
                consecutive_errors = 0;
                if let Some(new_buf) = resp["get_updates_buf"].as_str() {
                    buf = Some(new_buf.to_string());
                    storage::save_updates_buf(&user_id, new_buf)
                        .context("failed to save get_updates_buf")?;
                }

                if let Some(messages) = resp["msg_list"]
                    .as_array()
                    .or_else(|| resp["msgs"].as_array())
                {
                    for message in messages {
                        if let Some(context_token) = handle_message(&user_id, &bot_id, message)? {
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

fn handle_message(
    user_id: &str,
    bot_id: &str,
    message: &Value,
) -> Result<Option<String>> {
    let from_user_id = message["from_user_id"].as_str().unwrap_or("").to_string();
    let to_user_id = message["to_user_id"].as_str().unwrap_or("").to_string();
    let peer_user_id = conversation_peer_id(bot_id, &from_user_id, &to_user_id).to_string();
    let context_token = message["context_token"].as_str().unwrap_or("").to_string();
    let item_list = message["item_list"].as_array().cloned().unwrap_or_default();
    let item_types = item_list
        .iter()
        .filter_map(|item| item["type"].as_u64())
        .collect::<Vec<_>>();
    let text = body_from_item_list(&item_list);

    if !peer_user_id.is_empty() && !context_token.is_empty() {
        context::cache_context(
            user_id,
            ConversationContext {
                target_user_id: peer_user_id.clone(),
                context_token: context_token.clone(),
                updated_at: chrono::Utc::now(),
                last_text: if text.is_empty() {
                    None
                } else {
                    Some(text.clone())
                },
                item_types: item_types.clone(),
            },
        )?;
        return Ok(Some(context_token));
    }
    Ok(None)
}

fn conversation_peer_id<'a>(bot_id: &str, from_user_id: &'a str, to_user_id: &'a str) -> &'a str {
    if to_user_id == bot_id && !from_user_id.is_empty() {
        from_user_id
    } else if from_user_id == bot_id && !to_user_id.is_empty() {
        to_user_id
    } else {
        to_user_id
    }
}

fn body_from_item_list(item_list: &[Value]) -> String {
    let mut parts = vec![];
    for item in item_list {
        match item["type"].as_u64().unwrap_or(0) {
            0 => {
                if let Some(body) = item["body"].as_str() {
                    parts.push(body.to_string());
                }
            }
            1 => {
                if let Some(text) = item["text_item"]["text"].as_str() {
                    parts.push(text.to_string());
                }
            }
            5 => {
                if let Some(transcription) = item["voice_transcription_body"].as_str() {
                    parts.push(transcription.to_string());
                }
            }
            7 => {
                if let Some(ref_list) = item["ref_item_list"].as_array() {
                    let ref_text = body_from_item_list(ref_list);
                    if !ref_text.is_empty() {
                        parts.push(format!("> {ref_text}"));
                    }
                }
            }
            _ => {}
        }
    }
    parts.join("\n")
}
