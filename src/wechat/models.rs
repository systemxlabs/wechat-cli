use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MESSAGE_ITEM_TEXT: u64 = 1;
const MESSAGE_ITEM_IMAGE: u64 = 2;
const MESSAGE_ITEM_FILE: u64 = 4;
const MESSAGE_TYPE_BOT: u64 = 2;
const MESSAGE_STATE_FINISH: u64 = 2;
const MEDIA_ENCRYPT_TYPE_AES128_ECB: u64 = 1;

#[derive(Debug, Clone, Serialize)]
pub struct BaseInfo {
    pub channel_version: String,
}

impl BaseInfo {
    pub fn current() -> Self {
        Self {
            channel_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct EmptyResponse {}

#[derive(Debug, Deserialize)]
pub struct FetchQrCodeResponse {
    #[serde(default)]
    pub data: Option<QrCodeData>,
    #[serde(default)]
    pub qrcode_img_content: Option<String>,
    #[serde(default)]
    pub qrcode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QrCodeStatusResponse {
    #[serde(default)]
    pub data: Option<QrCodeData>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub bot_token: Option<String>,
    #[serde(default)]
    pub ilink_bot_id: Option<String>,
    #[serde(default)]
    pub ilink_user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QrCodeData {
    #[serde(default)]
    pub qrcode_url: Option<String>,
    #[serde(default)]
    pub qrcode_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub bot_token: Option<String>,
    #[serde(default)]
    pub ilink_bot_id: Option<String>,
    #[serde(default)]
    pub ilink_user_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GetUpdatesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_updates_buf: Option<String>,
    pub base_info: BaseInfo,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GetUpdatesResponse {
    #[serde(default)]
    pub get_updates_buf: Option<String>,
    #[serde(default)]
    pub msg_list: Option<Vec<InboundMessage>>,
    #[serde(default)]
    pub msgs: Option<Vec<InboundMessage>>,
}

impl GetUpdatesResponse {
    pub fn messages(&self) -> &[InboundMessage] {
        self.msg_list
            .as_deref()
            .or(self.msgs.as_deref())
            .unwrap_or(&[])
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct InboundMessage {
    #[serde(default)]
    pub from_user_id: String,
    #[serde(default)]
    pub to_user_id: String,
    #[serde(default)]
    pub context_token: String,
    #[serde(default)]
    pub item_list: Vec<InboundMessageItem>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct InboundMessageItem {
    #[serde(rename = "type")]
    pub item_type: u64,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub text_item: Option<TextItem>,
    #[serde(default)]
    pub voice_transcription_body: Option<String>,
    #[serde(default)]
    pub ref_item_list: Option<Vec<InboundMessageItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextItem {
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SendMessageRequest {
    pub msg: OutboundMessage,
    pub base_info: BaseInfo,
}

impl SendMessageRequest {
    pub fn new(to_user_id: impl Into<String>, context_token: impl Into<String>, item: OutboundMessageItem) -> Self {
        Self {
            msg: OutboundMessage::new(to_user_id, context_token, vec![item]),
            base_info: BaseInfo::current(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OutboundMessage {
    pub from_user_id: String,
    pub to_user_id: String,
    pub client_id: String,
    pub message_type: u64,
    pub message_state: u64,
    pub item_list: Vec<OutboundMessageItem>,
    pub context_token: String,
}

impl OutboundMessage {
    pub fn new(
        to_user_id: impl Into<String>,
        context_token: impl Into<String>,
        item_list: Vec<OutboundMessageItem>,
    ) -> Self {
        Self {
            from_user_id: String::new(),
            to_user_id: to_user_id.into(),
            client_id: generate_client_id(),
            message_type: MESSAGE_TYPE_BOT,
            message_state: MESSAGE_STATE_FINISH,
            item_list,
            context_token: context_token.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OutboundMessageItem {
    #[serde(rename = "type")]
    pub item_type: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_item: Option<TextItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_item: Option<ImageItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_item: Option<FileItem>,
}

impl OutboundMessageItem {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            item_type: MESSAGE_ITEM_TEXT,
            text_item: Some(TextItem { text: text.into() }),
            image_item: None,
            file_item: None,
        }
    }

    pub fn image(image_item: ImageItem) -> Self {
        Self {
            item_type: MESSAGE_ITEM_IMAGE,
            text_item: None,
            image_item: Some(image_item),
            file_item: None,
        }
    }

    pub fn file(file_item: FileItem) -> Self {
        Self {
            item_type: MESSAGE_ITEM_FILE,
            text_item: None,
            image_item: None,
            file_item: Some(file_item),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ImageItem {
    pub media: MediaRef,
    pub mid_size: u64,
}

impl ImageItem {
    pub fn from_uploaded(uploaded: &super::media::UploadedMedia) -> Self {
        Self {
            media: MediaRef::from_uploaded(uploaded),
            mid_size: uploaded.file_size_ciphertext,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FileItem {
    pub media: MediaRef,
    pub file_name: String,
    pub len: String,
}

impl FileItem {
    pub fn from_uploaded(uploaded: &super::media::UploadedMedia) -> Self {
        Self {
            media: MediaRef::from_uploaded(uploaded),
            file_name: uploaded.file_name.clone(),
            len: uploaded.file_size.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaRef {
    pub encrypt_query_param: String,
    pub aes_key: String,
    pub encrypt_type: u64,
}

impl MediaRef {
    pub fn from_uploaded(uploaded: &super::media::UploadedMedia) -> Self {
        Self {
            encrypt_query_param: uploaded.encrypt_query_param.clone(),
            aes_key: uploaded.aes_key_base64.clone(),
            encrypt_type: MEDIA_ENCRYPT_TYPE_AES128_ECB,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GetUploadUrlRequest {
    pub filekey: String,
    pub media_type: u64,
    pub to_user_id: String,
    pub rawsize: u64,
    pub rawfilemd5: String,
    pub filesize: u64,
    pub no_need_thumb: bool,
    pub aeskey: String,
    pub base_info: BaseInfo,
}

impl GetUploadUrlRequest {
    pub fn new(
        filekey: impl Into<String>,
        media_type: u64,
        to_user_id: impl Into<String>,
        rawsize: u64,
        rawfilemd5: impl Into<String>,
        filesize: u64,
        aeskey: impl Into<String>,
    ) -> Self {
        Self {
            filekey: filekey.into(),
            media_type,
            to_user_id: to_user_id.into(),
            rawsize,
            rawfilemd5: rawfilemd5.into(),
            filesize,
            no_need_thumb: true,
            aeskey: aeskey.into(),
            base_info: BaseInfo::current(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetUploadUrlResponse {
    #[serde(default)]
    pub upload_param: Option<String>,
}

impl FetchQrCodeResponse {
    pub fn qrcode_url(&self) -> Option<&str> {
        self.data
            .as_ref()
            .and_then(|data| data.qrcode_url.as_deref())
            .or(self.qrcode_img_content.as_deref())
    }

    pub fn qrcode_id(&self) -> Option<&str> {
        self.data
            .as_ref()
            .and_then(|data| data.qrcode_id.as_deref())
            .or(self.qrcode.as_deref())
    }
}

impl QrCodeStatusResponse {
    pub fn status(&self) -> &str {
        self.data
            .as_ref()
            .and_then(|data| data.status.as_deref())
            .or(self.status.as_deref())
            .unwrap_or("unknown")
    }

    pub fn bot_token(&self) -> Option<&str> {
        self.data
            .as_ref()
            .and_then(|data| data.bot_token.as_deref())
            .or(self.bot_token.as_deref())
    }

    pub fn ilink_bot_id(&self) -> Option<&str> {
        self.data
            .as_ref()
            .and_then(|data| data.ilink_bot_id.as_deref())
            .or(self.ilink_bot_id.as_deref())
    }

    pub fn ilink_user_id(&self) -> Option<&str> {
        self.data
            .as_ref()
            .and_then(|data| data.ilink_user_id.as_deref())
            .or(self.ilink_user_id.as_deref())
    }
}

impl GetUploadUrlResponse {
    pub fn upload_param(&self) -> Option<&str> {
        self.upload_param.as_deref()
    }
}

fn generate_client_id() -> String {
    format!("wechat-cli-{}", Uuid::new_v4().simple())
}
