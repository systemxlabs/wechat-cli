use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use mime_guess::mime;

use crate::{
    commands::account::{AccountSession, build_client, load_account, load_account_by_index},
    storage::DEFAULT_BASE_URL,
    wechat::api::WeixinApiClient,
    wechat::media::{OutboundMediaKind, build_media_item, upload_media},
};

pub async fn run(
    account: Option<usize>,
    user_id: Option<&str>,
    token: Option<&str>,
    base_url: Option<&str>,
    route_tag: Option<&str>,
    context_token: Option<&str>,
    text: Option<&str>,
    file_path: Option<&Path>,
    caption: Option<&str>,
) -> Result<()> {
    let send_target = resolve_send_target(account, user_id, token, base_url, route_tag)?;

    match (text, file_path) {
        (Some(text), None) => send_text(&send_target, context_token, text).await,
        (None, Some(file_path)) => send_media(&send_target, context_token, file_path, caption).await,
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

async fn send_text(target: &SendTarget, context_token: Option<&str>, text: &str) -> Result<()> {
    match target {
        SendTarget::Saved(session) => {
            let client = build_client(session);
            let context_token = require_context_token(&session.user_id, context_token)?;
            client
                .send_text_message(&session.data.user_id, &context_token, text)
                .await
                .context("failed to send text message")?;
            println!(
                "sent text message to `{}` using bot `{}`",
                session.data.user_id, session.data.bot_id
            );
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
                "sent {} `{}` to `{}` using bot `{}`",
                match media_kind {
                    OutboundMediaKind::Image => "image",
                    OutboundMediaKind::File => "file",
                },
                file_path.display(),
                session.data.user_id,
                session.data.bot_id,
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
    token: Option<&str>,
    base_url: Option<&str>,
    route_tag: Option<&str>,
) -> Result<SendTarget> {
    let using_explicit = token.is_some() || base_url.is_some() || route_tag.is_some();

    if using_explicit {
        if account.is_some() {
            bail!("`--account` cannot be used with `--token` / `--base-url` / `--route-tag`");
        }

        let token = token.ok_or_else(|| anyhow!("`--token` is required in explicit credential mode"))?;
        let user_id =
            user_id.ok_or_else(|| anyhow!("`--user-id` is required in explicit credential mode"))?;
        let base_url = base_url.unwrap_or(DEFAULT_BASE_URL);

        return Ok(SendTarget::Explicit {
            user_id: user_id.to_string(),
            client: WeixinApiClient::new(base_url, token, route_tag.map(str::to_string)),
            display_name: format!("explicit token at `{base_url}`"),
        });
    }

    if account.is_some() && user_id.is_some() {
        bail!("`--account` and `--user-id` cannot be used together in saved account mode");
    }

    if let Some(index) = account {
        return Ok(SendTarget::Saved(load_account_by_index(index)?));
    }

    if let Some(user_id) = user_id {
        return Ok(SendTarget::Saved(load_account(Some(user_id))?));
    }

    Ok(SendTarget::Saved(load_account_by_index(0)?))
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
