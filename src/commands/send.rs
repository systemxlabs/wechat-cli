use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use mime_guess::mime;

use crate::{
    commands::account::{AccountSession, build_client, load_account_by_index},
    wechat::api::WeixinApiClient,
    wechat::media::{OutboundMediaKind, build_media_item, upload_media},
};

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

    match (text, file_path) {
        (Some(text), None) => send_text(&send_target, context_token, text).await,
        (None, Some(file_path)) => {
            send_media(&send_target, context_token, file_path, caption).await
        }
        (Some(_), Some(_)) => bail!("`--text` and `--file` cannot be used together"),
        (None, None) => bail!("one of `--text` or `--file` is required"),
    }
}

enum SendTarget {
    Saved(AccountSession),
    Explicit {
        user_id: String,
        client: WeixinApiClient,
        display_name: String,
    },
}

#[derive(Debug, PartialEq)]
enum SendTargetKind {
    Saved,
    Explicit,
}

async fn send_text(target: &SendTarget, context_token: Option<&str>, text: &str) -> Result<()> {
    match target {
        SendTarget::Saved(session) => {
            let client = build_client(session);
            let context_token = require_context_token(&session.user_id, context_token)?;
            client
                .send_text_message(&session.data.user_id, &context_token, text)
                .await
                .context("failed to send text message")?;
            println!("sent text message to `{}`", session.data.user_id);
        }
        SendTarget::Explicit {
            user_id,
            client,
            display_name,
        } => {
            let context_token = require_context_token(display_name, context_token)?;
            client
                .send_text_message(user_id, &context_token, text)
                .await
                .context("failed to send text message")?;
            println!("sent text message to `{user_id}` using {display_name}");
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
        SendTarget::Saved(session) => {
            let client = build_client(session);
            let context_token = require_context_token(&session.user_id, context_token)?;
            let uploaded = upload_media(&client, &session.data.user_id, file_path, media_kind)
                .await
                .with_context(|| format!("failed to upload `{}`", file_path.display()))?;
            let media_item = build_media_item(media_kind, &uploaded);

            client
                .send_media_message(&session.data.user_id, &context_token, caption, media_item)
                .await
                .with_context(|| format!("failed to send `{}`", file_path.display()))?;

            println!(
                "sent {} `{}` to `{}`",
                match media_kind {
                    OutboundMediaKind::Image => "image",
                    OutboundMediaKind::File => "file",
                },
                file_path.display(),
                session.data.user_id,
            );
        }
        SendTarget::Explicit {
            user_id,
            client,
            display_name,
        } => {
            let context_token = require_context_token(display_name, context_token)?;
            let uploaded = upload_media(client, user_id, file_path, media_kind)
                .await
                .with_context(|| format!("failed to upload `{}`", file_path.display()))?;
            let media_item = build_media_item(media_kind, &uploaded);

            client
                .send_media_message(user_id, &context_token, caption, media_item)
                .await
                .with_context(|| format!("failed to send `{}`", file_path.display()))?;

            println!(
                "sent {} `{}` to `{}` using {}",
                match media_kind {
                    OutboundMediaKind::Image => "image",
                    OutboundMediaKind::File => "file",
                },
                file_path.display(),
                user_id,
                display_name,
            );
        }
    }
    Ok(())
}

fn resolve_send_target(
    account: Option<usize>,
    user_id: Option<&str>,
    bot_token: Option<&str>,
    route_tag: Option<&str>,
) -> Result<SendTarget> {
    let kind = resolve_send_target_kind(account, user_id, bot_token, route_tag)?;

    match kind {
        SendTargetKind::Saved => Ok(SendTarget::Saved(load_account_by_index(account.unwrap())?)),
        SendTargetKind::Explicit => {
            let bot_token = bot_token
                .ok_or_else(|| anyhow!("`--bot-token` is required in explicit credential mode"))?;
            let user_id = user_id
                .ok_or_else(|| anyhow!("`--user-id` is required in explicit credential mode"))?;
            Ok(SendTarget::Explicit {
                user_id: user_id.to_string(),
                client: WeixinApiClient::new(bot_token, route_tag.map(str::to_string)),
                display_name: "explicit bot token".to_string(),
            })
        }
    }
}

fn resolve_send_target_kind(
    account: Option<usize>,
    user_id: Option<&str>,
    bot_token: Option<&str>,
    route_tag: Option<&str>,
) -> Result<SendTargetKind> {
    let using_explicit = bot_token.is_some() || route_tag.is_some();

    if using_explicit {
        if account.is_some() {
            bail!("`--account` cannot be used with `--bot-token` / `--route-tag`");
        }
        if bot_token.is_none() {
            bail!("`--bot-token` is required in explicit credential mode");
        }
        if user_id.is_none() {
            bail!("`--user-id` is required in explicit credential mode");
        }
        return Ok(SendTargetKind::Explicit);
    }

    if account.is_none() && user_id.is_none() {
        bail!("You must specify either `--account <index>` for a saved account, or use explicit credentials mode (`--bot-token` and `--user-id`)");
    }

    if let Some(_) = account {
        if user_id.is_some() {
            bail!("`--account` and `--user-id` cannot be used together in saved account mode");
        }
        return Ok(SendTargetKind::Saved);
    }

    if let Some(_) = user_id {
        bail!("Using `--user-id` to select a saved account is no longer supported. Please use `--account <index>` instead, or provide both `--bot-token` and `--user-id` for explicit credentials mode");
    }

    unreachable!()
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
    fn test_only_user_id_is_rejected() {
        let result = resolve_send_target_kind(None, Some("user@im.wechat"), None, None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("no longer supported"), "Expected error about user-id no longer supported, got: {}", err_msg);
    }

    #[test]
    fn test_no_auth_params_is_rejected() {
        let result = resolve_send_target_kind(None, None, None, None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("must specify either"), "Expected error about requiring auth params, got: {}", err_msg);
    }

    #[test]
    fn test_account_with_explicit_creds_is_rejected() {
        let result = resolve_send_target_kind(Some(0), None, Some("token"), None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("cannot be used with"), "Expected error about mixing modes, got: {}", err_msg);
    }

    #[test]
    fn test_explicit_creds_succeeds() {
        let result = resolve_send_target_kind(None, Some("user@im.wechat"), Some("token"), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SendTargetKind::Explicit);
    }

    #[test]
    fn test_explicit_creds_with_route_tag_succeeds() {
        let result = resolve_send_target_kind(None, Some("user@im.wechat"), Some("token"), Some("route"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SendTargetKind::Explicit);
    }

    #[test]
    fn test_account_only_succeeds() {
        let result = resolve_send_target_kind(Some(0), None, None, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SendTargetKind::Saved);
    }

    #[test]
    fn test_explicit_creds_missing_bot_token_fails() {
        let result = resolve_send_target_kind(None, Some("user@im.wechat"), None, Some("route"));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("bot-token"), "Expected error about missing bot-token, got: {}", err_msg);
    }

    #[test]
    fn test_explicit_creds_missing_user_id_fails() {
        let result = resolve_send_target_kind(None, None, Some("token"), None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("user-id"), "Expected error about missing user-id, got: {}", err_msg);
    }

    #[test]
    fn test_account_with_user_id_is_rejected() {
        let result = resolve_send_target_kind(Some(0), Some("user@im.wechat"), None, None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("cannot be used together"), "Expected error about cannot use together, got: {}", err_msg);
    }
}
