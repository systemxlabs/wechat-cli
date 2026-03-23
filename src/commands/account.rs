use anyhow::{Context, Result, anyhow, bail};

use crate::{
    storage::{self, AccountConfig, AccountData},
    wechat::api::WeixinApiClient,
};

#[derive(Debug)]
pub struct AccountSession {
    pub user_id: String,
    pub data: AccountData,
    pub config: Option<AccountConfig>,
}

pub fn resolve_user_id(explicit: Option<&str>) -> Result<String> {
    if let Some(user_id) = explicit {
        return Ok(user_id.to_string());
    }

    let user_ids = storage::get_account_ids().context("failed to load saved users")?;
    user_ids
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no saved user found, run `wechat-cli login` first"))
}

pub fn load_account(user_id: Option<&str>) -> Result<AccountSession> {
    let user_id = resolve_user_id(user_id)?;
    let user_ids = storage::get_account_ids().context("failed to load saved users")?;
    if !user_ids.iter().any(|saved_id| saved_id == &user_id) {
        bail!("user `{user_id}` not found");
    }

    let data = storage::get_account_data(&user_id)
        .with_context(|| format!("failed to load account data for `{user_id}`"))?;
    let config = storage::get_account_config(&user_id);

    Ok(AccountSession {
        user_id,
        data,
        config,
    })
}

pub fn build_client(session: &AccountSession) -> WeixinApiClient {
    WeixinApiClient::new(
        &session.data.base_url,
        &session.data.token,
        session
            .config
            .as_ref()
            .and_then(|config| config.route_tag.clone()),
    )
}

pub fn list_accounts() -> Result<Vec<AccountSession>> {
    let user_ids = storage::get_account_ids().context("failed to load saved users")?;
    let mut accounts = Vec::with_capacity(user_ids.len());
    for user_id in user_ids {
        let data = storage::get_account_data(&user_id)
            .with_context(|| format!("failed to load account data for `{user_id}`"))?;
        let config = storage::get_account_config(&user_id);
        accounts.push(AccountSession {
            user_id,
            data,
            config,
        });
    }
    Ok(accounts)
}

pub fn print_accounts() -> Result<()> {
    let accounts = list_accounts()?;
    if accounts.is_empty() {
        println!("no saved users");
        return Ok(());
    }

    for entry in accounts {
        let route_tag = entry
            .config
            .as_ref()
            .and_then(|config| config.route_tag.as_deref())
            .unwrap_or("-");
        println!("user_id: {}", entry.user_id);
        println!("bot_id: {}", entry.data.bot_id);
        println!("base_url: {}", entry.data.base_url);
        println!("saved_at: {}", entry.data.saved_at);
        println!("route_tag: {route_tag}");
        println!();
    }

    Ok(())
}
