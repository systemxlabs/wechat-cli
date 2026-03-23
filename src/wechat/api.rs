use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;
use snafu::ResultExt;

use crate::errors::{ApiSnafu, HttpSnafu, JsonSnafu, Result, SessionExpiredSnafu};
use crate::storage::ILINK_API_ROOT;

use super::models::{
    EmptyResponse, FetchQrCodeResponse, GetUpdatesRequest, GetUpdatesResponse,
    GetUploadUrlRequest, GetUploadUrlResponse, QrCodeStatusResponse, SendMessageRequest,
};

const SESSION_EXPIRED_ERRCODE: i64 = -14;

#[derive(Debug, Default, serde::Deserialize)]
struct ApiStatus {
    #[serde(default)]
    errcode: Option<i64>,
    #[serde(default)]
    errmsg: Option<String>,
    #[serde(default)]
    ret: Option<i64>,
    #[serde(default)]
    err_msg: Option<String>,
}

pub(crate) fn build_http_client() -> Client {
    Client::builder()
        .http1_only()
        .build()
        .expect("failed to build reqwest client")
}

fn random_wechat_uin() -> String {
    let raw = rand::random::<u32>().to_string();
    STANDARD.encode(raw.as_bytes())
}

pub struct WeixinApiClient {
    client: Client,
    token: String,
    route_tag: Option<String>,
}

impl WeixinApiClient {
    pub fn new(token: &str, route_tag: Option<String>) -> Self {
        Self {
            client: build_http_client(),
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

    async fn post_json<TReq, TResp>(&self, path: &str, body: &TReq, timeout: Duration) -> Result<TResp>
    where
        TReq: Serialize + ?Sized,
        TResp: DeserializeOwned,
    {
        let url = format!("{}/{}", ILINK_API_ROOT, path);
        let body_bytes = serde_json::to_vec(body).context(JsonSnafu)?;
        let response_bytes = self
            .client
            .post(&url)
            .headers(self.json_headers(body_bytes.len()))
            .body(body_bytes)
            .timeout(timeout)
            .send()
            .await
            .context(HttpSnafu)?
            .bytes()
            .await
            .context(HttpSnafu)?;

        Self::decode_response(&response_bytes)
    }

    async fn post_form<TResp>(
        &self,
        path: &str,
        form: &[(&str, &str)],
        timeout: Duration,
    ) -> Result<TResp>
    where
        TResp: DeserializeOwned,
    {
        let url = format!("{}/{}", ILINK_API_ROOT, path);
        let response_bytes = self
            .client
            .post(&url)
            .headers(self.auth_headers())
            .form(form)
            .timeout(timeout)
            .send()
            .await
            .context(HttpSnafu)?
            .bytes()
            .await
            .context(HttpSnafu)?;

        Self::decode_response(&response_bytes)
    }

    fn decode_response<T>(response_bytes: &[u8]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let status: ApiStatus = serde_json::from_slice(response_bytes).context(JsonSnafu)?;

        if let Some(code) = status.errcode {
            if code == SESSION_EXPIRED_ERRCODE {
                return Err(SessionExpiredSnafu.build());
            }
            if code != 0 {
                return Err(ApiSnafu {
                    code,
                    message: status.errmsg.unwrap_or_else(|| "unknown error".to_string()),
                }
                .build());
            }
        }

        if let Some(code) = status.ret {
            if code != 0 {
                return Err(ApiSnafu {
                    code,
                    message: status.err_msg.unwrap_or_else(|| "unknown error".to_string()),
                }
                .build());
            }
        }

        serde_json::from_slice(response_bytes).context(JsonSnafu)
    }

    pub async fn fetch_qr_code(&self) -> Result<FetchQrCodeResponse> {
        self.post_form(
            "ilink/bot/get_bot_qrcode",
            &[("bot_type", "3")],
            Duration::from_secs(30),
        )
        .await
    }

    pub async fn get_qr_code_status(&self, qrcode_id: &str) -> Result<QrCodeStatusResponse> {
        self.post_form(
            "ilink/bot/get_qrcode_status",
            &[("qrcode", qrcode_id)],
            Duration::from_secs(40),
        )
        .await
    }

    pub async fn get_updates(&self, buf: Option<&str>) -> Result<GetUpdatesResponse> {
        let body = GetUpdatesRequest {
            get_updates_buf: buf.map(str::to_string),
            base_info: super::models::BaseInfo::current(),
        };
        self.post_json("ilink/bot/getupdates", &body, Duration::from_secs(40))
            .await
    }

    pub async fn send_message(&self, body: &SendMessageRequest) -> Result<EmptyResponse> {
        self.post_json("ilink/bot/sendmessage", body, Duration::from_secs(30))
            .await
    }

    pub async fn send_text_message(
        &self,
        to_user_id: &str,
        context_token: &str,
        text: &str,
    ) -> Result<EmptyResponse> {
        let body = SendMessageRequest::new(
            to_user_id.to_string(),
            context_token.to_string(),
            super::models::OutboundMessageItem::text(text.to_string()),
        );
        self.send_message(&body).await
    }

    pub async fn send_media_message(
        &self,
        to_user_id: &str,
        context_token: &str,
        text: Option<&str>,
        media_item: super::models::OutboundMessageItem,
    ) -> Result<EmptyResponse> {
        if let Some(t) = text {
            self.send_text_message(to_user_id, context_token, t).await?;
        }

        let body = SendMessageRequest::new(
            to_user_id.to_string(),
            context_token.to_string(),
            media_item,
        );
        self.send_message(&body).await
    }

    pub async fn get_upload_url(
        &self,
        payload: &GetUploadUrlRequest,
    ) -> Result<GetUploadUrlResponse> {
        self.post_json("ilink/bot/getuploadurl", payload, Duration::from_secs(30))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_new() {
        let client = WeixinApiClient::new("tok_123", None);
        assert_eq!(client.token, "tok_123");
        assert!(client.route_tag.is_none());
    }
}
