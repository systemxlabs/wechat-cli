use anyhow::{Context, Result};
use serde::Serialize;

use crate::commands::login::{fetch_qrcode, fetch_qrcode_status};

#[derive(Debug, Serialize)]
pub struct QrcodeOutput {
    pub qrcode_id: String,
    pub qrcode_url: String,
}

#[derive(Debug, Serialize)]
pub struct QrcodeStatusOutput {
    pub qrcode_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

pub async fn print_qrcode() -> Result<()> {
    let response = fetch_qrcode().await?;
    let output = QrcodeOutput {
        qrcode_id: response
            .qrcode_id()
            .context("Login failed: no qrcode_id")?
            .to_string(),
        qrcode_url: response
            .qrcode_url()
            .context("Login failed: no qrcode_url")?
            .to_string(),
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

pub async fn print_qrcode_status(qrcode_id: &str) -> Result<()> {
    let response = fetch_qrcode_status(qrcode_id).await?;
    let output = QrcodeStatusOutput {
        qrcode_id: qrcode_id.to_string(),
        status: response.status().to_string(),
        bot_token: response.bot_token().map(str::to_string),
        user_id: response.ilink_user_id().map(str::to_string),
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
