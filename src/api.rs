use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::Client;
use serde_json::Value;
use snafu::ResultExt;
use uuid::Uuid;

use crate::errors::{ApiSnafu, HttpSnafu, Result, SessionExpiredSnafu};

const SESSION_EXPIRED_ERRCODE: i64 = -14;
const MESSAGE_ITEM_TEXT: u64 = 1;
const MESSAGE_TYPE_BOT: u64 = 2;
const MESSAGE_STATE_FINISH: u64 = 2;

pub(crate) fn build_http_client() -> Client {
    Client::builder()
        .http1_only()
        .build()
        .expect("failed to build reqwest client")
}

fn build_base_info() -> Value {
    serde_json::json!({
        "channel_version": env!("CARGO_PKG_VERSION"),
    })
}

fn random_wechat_uin() -> String {
    let raw = rand::random::<u32>().to_string();
    STANDARD.encode(raw.as_bytes())
}

fn generate_client_id() -> String {
    format!("weixin-agent-{}", Uuid::new_v4().simple())
}

/// HTTP client wrapper for the `WeChat` iLink Bot API.
///
/// Handles authentication headers, request signing, and automatic
/// session-expiry detection on every response.
pub struct WeixinApiClient {
    client: Client,
    base_url: String,
    token: String,
    route_tag: Option<String>,
}

impl WeixinApiClient {
    /// Creates a new API client targeting `base_url` with the given bearer
    /// `token`.
    pub fn new(base_url: &str, token: &str, route_tag: Option<String>) -> Self {
        Self {
            client: build_http_client(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            route_tag,
        }
    }

    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("authorizationtype"),
            HeaderValue::from_static("ilink_bot_token"),
        );
        headers.insert(
            HeaderName::from_static("x-wechat-uin"),
            HeaderValue::from_str(&random_wechat_uin()).unwrap(),
        );
        if !self.token.is_empty() {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap(),
            );
        }
        if let Some(ref tag) = self.route_tag {
            headers.insert(
                HeaderName::from_static("skroutetag"),
                HeaderValue::from_str(tag).unwrap(),
            );
        }
        headers
    }

    fn json_headers(&self, content_length: usize) -> reqwest::header::HeaderMap {
        use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE, HeaderValue};
        let mut headers = self.auth_headers();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            CONTENT_LENGTH,
            HeaderValue::from_str(&content_length.to_string()).unwrap(),
        );
        headers
    }

    async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.post_with_timeout(path, body, Duration::from_secs(30))
            .await
    }

    async fn post_form_with_timeout(
        &self,
        path: &str,
        form: &[(&str, &str)],
        timeout: Duration,
    ) -> Result<Value> {
        let url = format!("{}/{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .headers(self.auth_headers())
            .form(form)
            .timeout(timeout)
            .send()
            .await
            .context(HttpSnafu)?
            .json::<Value>()
            .await
            .context(HttpSnafu)?;

        Self::check_api_error(resp)
    }

    async fn post_with_timeout(
        &self,
        path: &str,
        body: &Value,
        timeout: Duration,
    ) -> Result<Value> {
        let url = format!("{}/{}", self.base_url, path);
        let body_text = serde_json::to_string(body).context(crate::errors::JsonSnafu)?;
        let resp = self
            .client
            .post(&url)
            .headers(self.json_headers(body_text.as_bytes().len()))
            .body(body_text)
            .timeout(timeout)
            .send()
            .await
            .context(HttpSnafu)?
            .json::<Value>()
            .await
            .context(HttpSnafu)?;

        Self::check_api_error(resp)
    }

    fn check_api_error(resp: Value) -> Result<Value> {
        if let Some(code) = resp.get("errcode").and_then(serde_json::Value::as_i64) {
            if code == SESSION_EXPIRED_ERRCODE {
                return Err(SessionExpiredSnafu.build());
            }
            if code != 0 {
                let msg = resp
                    .get("errmsg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string();
                return Err(ApiSnafu { code, message: msg }.build());
            }
        }

        if let Some(code) = resp.get("ret").and_then(serde_json::Value::as_i64) {
            if code != 0 {
                let msg = resp
                    .get("err_msg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string();
                return Err(ApiSnafu { code, message: msg }.build());
            }
        }

        Ok(resp)
    }

    /// Requests a new login QR code from the API.
    pub async fn fetch_qr_code(&self) -> Result<Value> {
        self.post_form_with_timeout(
            "ilink/bot/get_bot_qrcode",
            &[("bot_type", "3")],
            Duration::from_secs(30),
        )
        .await
    }

    /// Polls the current scan status for the given `qrcode_id`.
    pub async fn get_qr_code_status(&self, qrcode_id: &str) -> Result<Value> {
        self.post_form_with_timeout(
            "ilink/bot/get_qrcode_status",
            &[("qrcode", qrcode_id)],
            Duration::from_secs(40),
        )
        .await
    }

    /// Long-polls for new incoming messages, optionally resuming from `buf`.
    pub async fn get_updates(&self, buf: Option<&str>) -> Result<Value> {
        let mut body = serde_json::json!({
            "base_info": build_base_info(),
        });
        if let Some(b) = buf {
            body["get_updates_buf"] = Value::String(b.to_string());
        }
        self.post_with_timeout("ilink/bot/getupdates", &body, Duration::from_secs(40))
            .await
    }

    /// Sends a plain-text message to `to_user_id`.
    pub async fn send_text_message(
        &self,
        to_user_id: &str,
        context_token: &str,
        text: &str,
    ) -> Result<Value> {
        let body = serde_json::json!({
            "msg": {
                "from_user_id": "",
                "to_user_id": to_user_id,
                "client_id": generate_client_id(),
                "message_type": MESSAGE_TYPE_BOT,
                "message_state": MESSAGE_STATE_FINISH,
                "item_list": [{
                    "type": MESSAGE_ITEM_TEXT,
                    "text_item": {
                        "text": text
                    }
                }],
                "context_token": context_token
            },
            "base_info": build_base_info()
        });
        self.post("ilink/bot/sendmessage", &body).await
    }

    /// Sends a media message (image, video, or file) to `to_user_id`.
    pub async fn send_media_message(
        &self,
        to_user_id: &str,
        context_token: &str,
        text: Option<&str>,
        media_item: &Value,
    ) -> Result<Value> {
        if let Some(t) = text {
            self.send_text_message(to_user_id, context_token, t).await?;
        }
        let body = serde_json::json!({
            "msg": {
                "from_user_id": "",
                "to_user_id": to_user_id,
                "client_id": generate_client_id(),
                "message_type": MESSAGE_TYPE_BOT,
                "message_state": MESSAGE_STATE_FINISH,
                "item_list": [media_item.clone()],
                "context_token": context_token
            },
            "base_info": build_base_info()
        });
        self.post("ilink/bot/sendmessage", &body).await
    }
    /// Requests upload metadata for an outbound media item.
    pub async fn get_upload_url(&self, payload: &Value) -> Result<Value> {
        let mut body = payload.clone();
        body["base_info"] = build_base_info();
        self.post("ilink/bot/getuploadurl", &body).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_new() {
        let client = WeixinApiClient::new("https://example.com/", "tok_123", None);
        assert_eq!(client.base_url, "https://example.com");
        assert_eq!(client.token, "tok_123");
        assert!(client.route_tag.is_none());
    }
}
