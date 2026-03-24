use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

/// Fixed `WeChat` iLink API root.
pub const ILINK_API_ROOT: &str = "https://ilinkai.weixin.qq.com";

/// Base URL for downloading encrypted media from the `WeChat` CDN.
pub const CDN_BASE_URL: &str = "https://novac2c.cdn.weixin.qq.com/c2c";

fn storage_root() -> PathBuf {
    dirs::home_dir()
        .expect("no home directory")
        .join(".config")
        .join("wechat-cli")
}

fn accounts_file_path() -> PathBuf {
    storage_root().join("accounts.json")
}

/// Persisted authentication credentials for a single `WeChat` account.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountData {
    /// Bearer bot token used to authenticate API requests.
    #[serde(rename = "botToken")]
    pub bot_token: String,
    /// ISO-8601 timestamp of when this data was saved.
    #[serde(rename = "savedAt")]
    pub saved_at: String,
    /// The iLink user ID associated with this account.
    #[serde(rename = "userId")]
    pub user_id: String,
    /// Optional routing tag sent as a header on every API request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_tag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct AccountsFile {
    accounts: Vec<AccountData>,
}

fn load_accounts_file() -> Result<AccountsFile> {
    let path = accounts_file_path();
    if !path.exists() {
        return Ok(AccountsFile::default());
    }
    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read accounts file `{}`", path.display()))?;
    serde_json::from_str(&data).context("failed to parse accounts file")
}

fn save_accounts_file(accounts: &AccountsFile) -> Result<()> {
    let path = accounts_file_path();
    std::fs::create_dir_all(path.parent().unwrap()).with_context(|| {
        format!(
            "failed to create storage directory `{}`",
            path.parent().unwrap().display()
        )
    })?;
    let json =
        serde_json::to_string_pretty(accounts).context("failed to serialize accounts file")?;
    std::fs::write(&path, json)
        .with_context(|| format!("failed to write accounts file `{}`", path.display()))?;
    Ok(())
}

/// Returns the list of saved stable user IDs from local storage.
pub fn get_account_ids() -> Result<Vec<String>> {
    let accounts = load_accounts_file()?;
    Ok(accounts
        .accounts
        .into_iter()
        .map(|account| account.user_id)
        .filter(|id| id.ends_with("@im.wechat"))
        .collect())
}

/// Loads the saved credentials for the given stable user ID.
pub fn get_account_data(account_id: &str) -> Result<AccountData> {
    let accounts = load_accounts_file()?;
    accounts
        .accounts
        .into_iter()
        .find(|account| account.user_id == account_id)
        .ok_or_else(|| anyhow!("account `{account_id}` not found"))
}

/// Saves credentials for the given stable user ID to local storage.
pub fn save_account_data(account_id: &str, data: &AccountData) -> Result<()> {
    let mut accounts = load_accounts_file()?;
    if let Some(existing) = accounts
        .accounts
        .iter_mut()
        .find(|account| account.user_id == account_id)
    {
        *existing = AccountData {
            bot_token: data.bot_token.clone(),
            saved_at: data.saved_at.clone(),
            user_id: data.user_id.clone(),
            route_tag: data.route_tag.clone(),
        };
    } else {
        accounts.accounts.push(AccountData {
            bot_token: data.bot_token.clone(),
            saved_at: data.saved_at.clone(),
            user_id: data.user_id.clone(),
            route_tag: data.route_tag.clone(),
        });
    }
    save_accounts_file(&accounts)
}

/// Deletes credentials for the given stable user ID from local storage.
pub fn delete_account_data(account_id: &str) -> Result<()> {
    let mut accounts = load_accounts_file()?;
    let original_len = accounts.accounts.len();
    accounts
        .accounts
        .retain(|account| account.user_id != account_id);
    if accounts.accounts.len() == original_len {
        return Err(anyhow!("account `{account_id}` not found"));
    }
    save_accounts_file(&accounts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_data_serde() {
        let data = AccountData {
            bot_token: "tok_abc".to_string(),
            saved_at: "2025-01-01T00:00:00Z".to_string(),
            user_id: "user_123".to_string(),
            route_tag: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: AccountData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.bot_token, data.bot_token);
        assert_eq!(deserialized.saved_at, data.saved_at);
        assert_eq!(deserialized.user_id, data.user_id);
    }

    #[test]
    fn test_account_data_with_route_tag() {
        let data = AccountData {
            bot_token: "tok_abc".to_string(),
            saved_at: "2025-01-01T00:00:00Z".to_string(),
            user_id: "user_123".to_string(),
            route_tag: Some("tag_xyz".to_string()),
        };
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: AccountData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.route_tag, Some("tag_xyz".to_string()));
    }

    #[test]
    fn test_account_data_without_route_tag() {
        let json = r#"{"botToken":"tok_abc","savedAt":"2025-01-01T00:00:00Z","userId":"user_123"}"#;
        let data: AccountData = serde_json::from_str(json).unwrap();
        assert_eq!(data.route_tag, None);
    }
}
