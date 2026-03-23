use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use snafu::{IntoError, ResultExt};

use crate::{
    Result,
    errors::{IoSnafu, JsonSnafu},
};

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
    /// Bearer token used to authenticate API requests.
    pub token: String,
    /// ISO-8601 timestamp of when this data was saved.
    #[serde(rename = "savedAt")]
    pub saved_at: String,
    /// The current iLink bot ID associated with this login session.
    #[serde(rename = "botId")]
    pub bot_id: String,
    /// The iLink user ID associated with this account.
    #[serde(rename = "userId")]
    pub user_id: String,
}

/// Per-user configuration loaded from local storage.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountConfig {
    /// Optional routing tag sent as a header on every API request.
    #[serde(default)]
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
    let data = std::fs::read_to_string(&path).context(IoSnafu)?;
    serde_json::from_str(&data).context(JsonSnafu)
}

fn save_accounts_file(accounts: &AccountsFile) -> Result<()> {
    let path = accounts_file_path();
    std::fs::create_dir_all(path.parent().unwrap()).context(IoSnafu)?;
    let json = serde_json::to_string_pretty(accounts).context(JsonSnafu)?;
    std::fs::write(&path, json).context(IoSnafu)?;
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
        .ok_or_else(|| {
            IoSnafu
                .into_error(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("account `{account_id}` not found"),
                ))
        })
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
            token: data.token.clone(),
            saved_at: data.saved_at.clone(),
            bot_id: data.bot_id.clone(),
            user_id: data.user_id.clone(),
        };
    } else {
        accounts.accounts.push(AccountData {
            token: data.token.clone(),
            saved_at: data.saved_at.clone(),
            bot_id: data.bot_id.clone(),
            user_id: data.user_id.clone(),
        });
    }
    save_accounts_file(&accounts)
}

/// Loads the optional per-user configuration, returning `None` if absent.
pub fn get_account_config(account_id: &str) -> Option<AccountConfig> {
    let path = storage_root()
        .join("config")
        .join(format!("{account_id}.json"));
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_data_serde() {
        let data = AccountData {
            token: "tok_abc".to_string(),
            saved_at: "2025-01-01T00:00:00Z".to_string(),
            bot_id: "bot_123".to_string(),
            user_id: "user_123".to_string(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: AccountData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.token, data.token);
        assert_eq!(deserialized.saved_at, data.saved_at);
        assert_eq!(deserialized.bot_id, data.bot_id);
        assert_eq!(deserialized.user_id, data.user_id);
    }

    #[test]
    fn test_account_config_serde() {
        let config = AccountConfig {
            route_tag: Some("tag_xyz".to_string()),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AccountConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.route_tag, Some("tag_xyz".to_string()));
    }

    #[test]
    fn test_account_config_default_route_tag() {
        let json = r"{}";
        let config: AccountConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.route_tag, None);
    }
}
