use std::path::Path;

use anyhow::{Context, Result, bail};
use mime_guess::mime;

use crate::{
    account::{build_client, load_account},
    context,
    media::{OutboundMediaKind, build_media_item, upload_media},
};

pub async fn send(
    user_id: Option<&str>,
    context_token: Option<&str>,
    text: Option<&str>,
    file_path: Option<&Path>,
    caption: Option<&str>,
) -> Result<()> {
    match (text, file_path) {
        (Some(text), None) => send_text(user_id, context_token, text).await,
        (None, Some(file_path)) => send_media(user_id, context_token, file_path, caption).await,
        (Some(_), Some(_)) => bail!("`--text` and `--file` cannot be used together"),
        (None, None) => bail!("one of `--text` or `--file` is required"),
    }
}

async fn send_text(user_id: Option<&str>, context_token: Option<&str>, text: &str) -> Result<()> {
    let session = load_account(user_id)?;
    let target_user_id = &session.data.user_id;
    let context_token = resolve_context_token(&session.user_id, context_token)?;
    let client = build_client(&session);
    client
        .send_text_message(target_user_id, &context_token, text)
        .await
        .context("failed to send text message")?;
    println!(
        "sent text message to bound user `{}` using bot `{}`",
        session.data.user_id, session.data.bot_id
    );
    Ok(())
}

async fn send_media(
    user_id: Option<&str>,
    context_token: Option<&str>,
    file_path: &Path,
    caption: Option<&str>,
) -> Result<()> {
    if !file_path.is_file() {
        bail!("file `{}` does not exist", file_path.display());
    }

    let session = load_account(user_id)?;
    let target_user_id = &session.data.user_id;
    let context_token = resolve_context_token(&session.user_id, context_token)?;
    let client = build_client(&session);

    let media_kind = detect_media_kind(file_path);
    let uploaded = upload_media(&client, target_user_id, file_path, media_kind)
        .await
        .with_context(|| format!("failed to upload `{}`", file_path.display()))?;
    let media_item = build_media_item(media_kind, &uploaded);

    client
        .send_media_message(target_user_id, &context_token, caption, &media_item)
        .await
        .with_context(|| format!("failed to send `{}`", file_path.display()))?;

    println!(
        "sent {} `{}` to bound user `{}` using bot `{}`",
        match media_kind {
            OutboundMediaKind::Image => "image",
            OutboundMediaKind::File => "file",
        },
        file_path.display(),
        session.data.user_id,
        session.data.bot_id
    );
    Ok(())
}

fn detect_media_kind(file_path: &Path) -> OutboundMediaKind {
    match mime_guess::from_path(file_path).first() {
        Some(mime_type) if mime_type.type_() == mime::IMAGE => OutboundMediaKind::Image,
        _ => OutboundMediaKind::File,
    }
}

fn resolve_context_token(user_id: &str, explicit: Option<&str>) -> Result<String> {
    if let Some(context_token) = explicit {
        return Ok(context_token.to_string());
    }

    if let Some(cached) = context::get_primary_context(user_id).with_context(|| {
        format!("failed to load cached context for bound user under `{user_id}`")
    })? {
        return Ok(cached.context_token);
    }

    bail!(
        "no cached context_token for bound user under `{user_id}`; run `wechat-cli get-context-token --user-id {user_id}` first or pass `--context-token`"
    )
}
