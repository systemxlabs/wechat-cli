use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub target_user_id: String,
    pub context_token: String,
    pub updated_at: DateTime<Utc>,
    pub last_text: Option<String>,
    pub item_types: Vec<u64>,
}

fn storage_root() -> Result<PathBuf> {
    let home = dirs::home_dir().context("failed to resolve home directory")?;
    Ok(home.join(".cache").join("wechat-cli"))
}

fn context_file(user_id: &str) -> Result<PathBuf> {
    Ok(storage_root()?
        .join("contexts")
        .join(format!("{user_id}.json")))
}

fn read_context(path: &Path) -> Result<Option<ConversationContext>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read context cache `{}`", path.display()))?;
    let context = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse context cache `{}`", path.display()))?;
    Ok(Some(context))
}

pub fn cache_context(user_id: &str, context: ConversationContext) -> Result<()> {
    let path = context_file(user_id)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    let content =
        serde_json::to_string_pretty(&context).context("failed to serialize context cache")?;
    std::fs::write(&path, content)
        .with_context(|| format!("failed to write context cache `{}`", path.display()))?;
    Ok(())
}

pub fn get_primary_context(user_id: &str) -> Result<Option<ConversationContext>> {
    let path = context_file(user_id)?;
    read_context(&path)
}
