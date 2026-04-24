use anyhow::{Context, Result, anyhow, bail};
use log::{info, warn};

use crate::{
    storage,
    wechat::{
        api::WeixinApiClient,
        models::{FetchQrCodeResponse, QrCodeStatusResponse},
    },
};

pub async fn fetch_qrcode() -> Result<FetchQrCodeResponse> {
    let client = WeixinApiClient::new("", None);
    client.fetch_qr_code().await
}

pub async fn fetch_qrcode_status(qrcode_id: &str) -> Result<QrCodeStatusResponse> {
    let client = WeixinApiClient::new("", None);
    client.get_qr_code_status(qrcode_id).await
}

pub async fn login() -> Result<String> {
    let qr_resp = fetch_qrcode().await?;
    let qrcode_url = qr_resp
        .qrcode_url()
        .context("Login failed: no qrcode_url")?;
    let qrcode_id = qr_resp.qrcode_id().context("Login failed: no qrcode_id")?;

    let qr = qrcode::QrCode::new(qrcode_url.as_bytes())
        .map_err(|e| anyhow!("Login failed: QR generation failed: {e}"))?;
    let image = qr
        .render::<char>()
        .quiet_zone(true)
        .module_dimensions(2, 1)
        .build();
    println!("{image}");
    println!("Scan the QR code above with WeChat to login");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let status_resp = fetch_qrcode_status(qrcode_id).await?;

        match status_resp.status() {
            "wait" => {}
            "scaned" => {
                info!("QR code scanned, waiting for confirmation...");
            }
            "expired" => {
                bail!("QR code expired");
            }
            "confirmed" => {
                let bot_token = status_resp
                    .bot_token()
                    .context("Login failed: no bot_token")?;
                let user_id = status_resp
                    .ilink_user_id()
                    .context("Login failed: no ilink_user_id")?;
                let account_id = user_id.to_string();

                let account_data = storage::AccountData {
                    bot_token: bot_token.to_string(),
                    saved_at: chrono::Utc::now().to_rfc3339(),
                    user_id: user_id.to_string(),
                    route_tag: None,
                };
                storage::save_account_data(&account_data)?;

                info!("Login successful! User ID: {account_id}");
                return Ok(account_id);
            }
            other => {
                warn!("Unknown QR status: {other}");
            }
        }
    }
}
