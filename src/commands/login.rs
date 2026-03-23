use snafu::OptionExt;
use tracing::{info, warn};

use crate::{
    errors::{LoginFailedSnafu, QrCodeExpiredSnafu, Result},
    storage::{self, DEFAULT_BASE_URL},
    wechat::api::WeixinApiClient,
};

#[derive(Debug, Clone, Default)]
pub struct LoginOptions {
    pub base_url: Option<String>,
}

pub async fn login(options: LoginOptions) -> Result<String> {
    let base_url = options.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL);
    let client = WeixinApiClient::new(base_url, "", None);

    let qr_resp = client.fetch_qr_code().await?;
    let qrcode_url = qr_resp.qrcode_url().context(LoginFailedSnafu {
        reason: "no qrcode_url",
    })?;
    let qrcode_id = qr_resp.qrcode_id().context(LoginFailedSnafu {
        reason: "no qrcode_id",
    })?;

    let qr = qrcode::QrCode::new(qrcode_url.as_bytes()).map_err(|e| {
        LoginFailedSnafu {
            reason: format!("QR generation failed: {e}"),
        }
        .build()
    })?;
    let image = qr
        .render::<char>()
        .quiet_zone(true)
        .module_dimensions(2, 1)
        .build();
    println!("{image}");
    println!("Scan the QR code above with WeChat to login");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let status_resp = client.get_qr_code_status(qrcode_id).await?;

        match status_resp.status() {
            "wait" => {}
            "scaned" => {
                info!("QR code scanned, waiting for confirmation...");
            }
            "expired" => {
                return Err(QrCodeExpiredSnafu.build());
            }
            "confirmed" => {
                let token = status_resp.bot_token().context(LoginFailedSnafu {
                    reason: "no bot_token",
                })?;
                let bot_id = status_resp.ilink_bot_id().context(LoginFailedSnafu {
                    reason: "no ilink_bot_id",
                })?;
                let user_id = status_resp.ilink_user_id().context(LoginFailedSnafu {
                    reason: "no ilink_user_id",
                })?;
                let base = status_resp.base_url().unwrap_or(base_url);
                let account_id = user_id.to_string();

                let account_data = storage::AccountData {
                    token: token.to_string(),
                    saved_at: chrono::Utc::now().to_rfc3339(),
                    base_url: base.to_string(),
                    bot_id: bot_id.to_string(),
                    user_id: user_id.to_string(),
                };
                storage::save_account_data(&account_id, &account_data)?;

                info!("Login successful! User ID: {account_id}");
                return Ok(account_id);
            }
            other => {
                warn!("Unknown QR status: {other}");
            }
        }
    }
}
