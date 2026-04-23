fn resolve_send_target(
    account: Option<usize>,
    user_id: Option<&str>,
    bot_token: Option<&str>,
    route_tag: Option<&str>,
) -> Result<SendTarget> {
    let using_explicit = bot_token.is_some() || route_tag.is_some();

    if using_explicit {
        if account.is_some() {
            bail!("`--account` cannot be used with `--bot-token` / `--route-tag`");
        }

        let bot_token = bot_token
            .ok_or_else(|| anyhow!("`--bot-token` is required in explicit credential mode"))?;
        let user_id = user_id
            .ok_or_else(|| anyhow!("`--user-id` is required in explicit credential mode"))?;

        return Ok(SendTarget::Explicit {
            user_id: user_id.to_string(),
            client: WeixinApiClient::new(bot_token, route_tag.map(str::to_string)),
            display_name: "explicit bot token".to_string(),
        });
    }

    if account.is_none() && user_id.is_none() {
        bail!("You must specify either `--account <index>` for a saved account, or use explicit credentials mode (`--bot-token` and `--user-id`)");
    }

    if let Some(index) = account {
        if user_id.is_some() {
            bail!("`--account` and `--user-id` cannot be used together in saved account mode");
        }
        return Ok(SendTarget::Saved(load_account_by_index(index)?));
    }

    if let Some(_user_id) = user_id {
        bail!("Using `--user-id` to select a saved account is no longer supported. Please use `--account <index>` instead, or provide both `--bot-token` and `--user-id` for explicit credentials mode");
    }

    unreachable!()
}