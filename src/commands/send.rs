use std::io::{self, IsTerminal, Read};
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use mime_guess::mime;

use crate::{
    commands::account::build_client,
    storage::load_account,
    wechat::{
        api::WeixinApiClient,
        media::{OutboundMediaKind, build_media_item, upload_media},
    },
};

#[derive(Debug, PartialEq)]
pub enum SendContent {
    Text(String),
    File(std::path::PathBuf),
}

pub fn resolve_send_content(
    text: Option<&str>,
    file_path: Option<&Path>,
    stdin_reader: &mut dyn Read,
    stdin_is_pipe: bool,
) -> Result<SendContent> {
    match (text, file_path) {
        (Some(text), None) => Ok(SendContent::Text(text.to_string())),
        (None, Some(file_path)) => Ok(SendContent::File(file_path.to_path_buf())),
        (Some(_), Some(_)) => bail!("`--text` and `--file` cannot be used together"),
        (None, None) => {
            if !stdin_is_pipe {
                bail!(
                    "one of `--text`, `--file`, or piped stdin is required"
                );
            }
            let mut buf = String::new();
            stdin_reader.read_to_string(&mut buf)?;
            if buf.trim().is_empty() {
                bail!("stdin is empty; provide text via `--text` or pipe content");
            }
            Ok(SendContent::Text(buf))
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn run(
    account: Option<usize>,
    user_id: Option<&str>,
    bot_token: Option<&str>,
    route_tag: Option<&str>,
    context_token: Option<&str>,
    text: Option<&str>,
    file_path: Option<&Path>,
    caption: Option<&str>,
) -> Result<()> {
    let send_target = resolve_send_target(account, user_id, bot_token, route_tag)?;

    let stdin_is_pipe = !io::stdin().is_terminal();
    let content = resolve_send_content(text, file_path, &mut io::stdin(), stdin_is_pipe)?;

    match content {
        SendContent::Text(text) => send_text(&send_target, context_token, &text).await,
        SendContent::File(file_path) => {
            send_media(&send_target, context_token, &file_path, caption).await
        }
    }
}

#[derive(Debug)]
pub enum SendTarget {
    Saved {
        user_id: String,
        client: WeixinApiClient,
    },
    Explicit {
        user_id: String,
        client: WeixinApiClient,
    },
}

async fn send_text(target: &SendTarget, context_token: Option<&str>, text: &str) -> Result<()> {
    match target {
        SendTarget::Saved { user_id, client } => {
            let context_token = require_context_token(user_id, context_token)?;
            client
                .send_text_message(user_id, &context_token, text)
                .await
                .context("failed to send text message")?;
            println!("sent text message to `{user_id}`");
        }
        SendTarget::Explicit { user_id, client } => {
            let context_token = require_context_token(user_id, context_token)?;
            client
                .send_text_message(user_id, &context_token, text)
                .await
                .context("failed to send text message")?;
            println!("sent text message to `{user_id}` using explicit bot token");
        }
    }
    Ok(())
}

async fn send_media(
    target: &SendTarget,
    context_token: Option<&str>,
    file_path: &Path,
    caption: Option<&str>,
) -> Result<()> {
    if !file_path.is_file() {
        bail!("file `{}` does not exist", file_path.display());
    }

    let media_kind = detect_media_kind(file_path);
    match target {
        SendTarget::Saved { user_id, client } => {
            let context_token = require_context_token(user_id, context_token)?;
            let uploaded = upload_media(client, user_id, file_path, media_kind)
                .await
                .with_context(|| format!("failed to upload `{}`", file_path.display()))?;
            let media_item = build_media_item(media_kind, &uploaded);

            client
                .send_media_message(user_id, &context_token, caption, media_item)
                .await
                .with_context(|| format!("failed to send `{}`", file_path.display()))?;

            println!(
                "sent {} `{}` to `{user_id}`",
                match media_kind {
                    OutboundMediaKind::Image => "image",
                    OutboundMediaKind::File => "file",
                },
                file_path.display(),
            );
        }
        SendTarget::Explicit { user_id, client } => {
            let context_token = require_context_token(user_id, context_token)?;
            let uploaded = upload_media(client, user_id, file_path, media_kind)
                .await
                .with_context(|| format!("failed to upload `{}`", file_path.display()))?;
            let media_item = build_media_item(media_kind, &uploaded);

            client
                .send_media_message(user_id, &context_token, caption, media_item)
                .await
                .with_context(|| format!("failed to send `{}`", file_path.display()))?;

            println!(
                "sent {} `{}` to `{user_id}` using explicit bot token",
                match media_kind {
                    OutboundMediaKind::Image => "image",
                    OutboundMediaKind::File => "file",
                },
                file_path.display(),
            );
        }
    }
    Ok(())
}

pub fn resolve_send_target(
    account: Option<usize>,
    user_id: Option<&str>,
    bot_token: Option<&str>,
    route_tag: Option<&str>,
) -> Result<SendTarget> {
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

        return Ok(SendTarget::Explicit {
            user_id,
            client: WeixinApiClient::new(&bot_token, route_tag.map(str::to_string)),
        });
    }

    if let Some(idx) = account {
        let data = load_account(idx)?;
        let user_id = data.user_id.clone();
        return Ok(SendTarget::Saved {
            user_id,
            client: build_client(&data),
        });
    }

    bail!(
        "You must specify either `--account <index>` for a saved account, or use explicit credentials mode (`--bot-token` and `--user-id`)"
    )
}

fn detect_media_kind(file_path: &Path) -> OutboundMediaKind {
    match mime_guess::from_path(file_path).first() {
        Some(mime_type) if mime_type.type_() == mime::IMAGE => OutboundMediaKind::Image,
        _ => OutboundMediaKind::File,
    }
}

fn require_context_token(account_label: &str, explicit: Option<&str>) -> Result<String> {
    if let Some(context_token) = explicit {
        return Ok(context_token.to_string());
    }

    bail!(
        "missing `--context-token` for `{account_label}`; run `wechat-cli get-context-token` for the bound user and pass the printed token"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_auth_params_is_rejected() {
        let result = resolve_send_target(None, None, None, None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("must specify either"),
            "Expected error about requiring auth params, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_account_with_explicit_creds_is_rejected() {
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
    fn test_explicit_creds_missing_bot_token_fails() {
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
    fn test_explicit_creds_missing_user_id_fails() {
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
    fn test_account_with_user_id_is_rejected() {
        let result = resolve_send_target(Some(0), Some("user@im.wechat"), None, None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("cannot be used with"),
            "Expected error about cannot use together, got: {}",
            err_msg
        );
    }

    // Tests for resolve_send_content

    #[test]
    fn test_text_takes_priority_over_stdin() {
        let mut stdin = "stdin content".as_bytes();
        let result = resolve_send_content(Some("hello"), None, &mut stdin, true);
        assert_eq!(result.unwrap(), SendContent::Text("hello".to_string()));
    }

    #[test]
    fn test_file_takes_priority_over_stdin() {
        let mut stdin = "stdin content".as_bytes();
        let path = Path::new("/tmp/file.txt");
        let result = resolve_send_content(None, Some(path), &mut stdin, true);
        assert_eq!(result.unwrap(), SendContent::File(path.to_path_buf()));
    }

    #[test]
    fn test_stdin_used_when_no_text_or_file_and_pipe() {
        let mut stdin = "hello from stdin".as_bytes();
        let result = resolve_send_content(None, None, &mut stdin, true);
        assert_eq!(
            result.unwrap(),
            SendContent::Text("hello from stdin".to_string())
        );
    }

    #[test]
    fn test_error_when_no_text_or_file_and_terminal_stdin() {
        let mut stdin = "".as_bytes();
        let result = resolve_send_content(None, None, &mut stdin, false);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("one of `--text`, `--file`, or piped stdin"),
            "Expected error about requiring text/file/stdin, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_error_when_both_text_and_file() {
        let mut stdin = "".as_bytes();
        let path = Path::new("/tmp/file.txt");
        let result = resolve_send_content(Some("hello"), Some(path), &mut stdin, false);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("cannot be used together"),
            "Expected error about mutually exclusive, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_error_when_stdin_empty() {
        let mut stdin = "   ".as_bytes();
        let result = resolve_send_content(None, None, &mut stdin, true);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("stdin is empty"),
            "Expected error about empty stdin, got: {}",
            err_msg
        );
    }
}
