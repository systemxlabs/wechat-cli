use anyhow::{Result, bail};

use crate::{
    storage::{self, AccountData, load_accounts},
    wechat::api::WeixinApiClient,
};

pub fn build_client(data: &AccountData) -> WeixinApiClient {
    WeixinApiClient::new(&data.bot_token, data.route_tag.clone())
}

pub fn print_accounts() -> Result<()> {
    let accounts = load_accounts()?;
    if accounts.is_empty() {
        println!("no saved accounts");
        return Ok(());
    }

    for (index, entry) in accounts.into_iter().enumerate() {
        let route_tag = entry.route_tag.as_deref().unwrap_or("-");
        println!("account: {index}");
        println!("bot_token: {}", entry.bot_token);
        println!("user_id: {}", entry.user_id);
        println!("route_tag: {route_tag}");
        println!("saved_at: {}", entry.saved_at);
        println!();
    }

    Ok(())
}

pub fn delete_account(index: usize) -> Result<()> {
    let deleted = storage::delete_account(index)?;
    println!("deleted account `{deleted:?}`");
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
    storage::save_account_data(&data)?;

    println!("saved account `{user_id}`");
    Ok(())
}
