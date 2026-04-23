use anyhow::{Context, Result, anyhow, bail};

use crate::{
    storage::{self, AccountData},
    wechat::api::WeixinApiClient,
};

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

pub fn load_account_by_index(index: usize) -> Result<AccountData> {
    let accounts = list_accounts()?;
    accounts
        .into_iter()
        .nth(index)
        .ok_or_else(|| anyhow!("account index `{index}` not found"))
}

pub fn build_client(data: &AccountData) -> WeixinApiClient {
    WeixinApiClient::new(&data.bot_token, data.route_tag.clone())
}

pub fn list_accounts() -> Result<Vec<AccountData>> {
    let user_ids = storage::get_account_ids().context("failed to load saved users")?;
    let mut accounts = Vec::with_capacity(user_ids.len());
    for user_id in user_ids {
        let data = storage::get_account_data(&user_id)
            .with_context(|| format!("failed to load account data for `{user_id}`"))?;
        accounts.push(data);
    }
    Ok(accounts)
}

pub fn print_accounts() -> Result<()> {
    let accounts = list_accounts()?;
    if accounts.is_empty() {
        println!("no saved users");
        return Ok(());
    }

    for (index, entry) in accounts.into_iter().enumerate() {
        let route_tag = entry.route_tag.as_deref().unwrap_or("-");
        println!("account: {index}");
        println!("user_id: {}", entry.user_id);
        println!("saved_at: {}", entry.saved_at);
        println!("route_tag: {route_tag}");
        println!();
    }

    Ok(())
}

pub fn delete_account(index: Option<usize>, user_id: Option<&str>) -> Result<()> {
    let user_id = match (index, user_id) {
        (Some(index), None) => load_account_by_index(index)?.user_id,
        (None, Some(user_id)) => {
            let resolved_id = resolve_user_id(Some(user_id))?;
            let user_ids = storage::get_account_ids().context("failed to load saved users")?;
            if !user_ids.iter().any(|saved_id| saved_id == &resolved_id) {
                bail!("user `{resolved_id}` not found");
            }
            resolved_id
        }
        _ => bail!("exactly one of `--account` or `--user-id` is required"),
    };

    storage::delete_account_data(&user_id)?;
    println!("deleted account `{user_id}`");
    Ok(())
}

pub fn add_account(user_id: &str, bot_token: &str, route_tag: Option<&str>) -> Result<()> {
    if !user_id.ends_with("@im.wechat") {
        bail!("user_id `{user_id}` must end with `@im.wechat`");
    }
    if bot_token.trim().is_empty() {
        bail!("bot_token cannot be empty");
    }

    let data = AccountData {
        bot_token: bot_token.to_string(),
        saved_at: chrono::Utc::now().to_rfc3339(),
        user_id: user_id.to_string(),
        route_tag: route_tag.map(str::to_string),
    };
    storage::save_account_data(user_id, &data)?;

    println!("saved account `{user_id}`");
    Ok(())
}
