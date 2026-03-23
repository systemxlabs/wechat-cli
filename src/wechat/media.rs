use std::path::Path;

use aes::Aes128;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use block_padding::Pkcs7;
use cipher::{BlockEncryptMut as _, KeyInit};
use ecb;
use snafu::ResultExt;

use crate::{
    errors::{ApiSnafu, HttpSnafu, IoSnafu},
    storage::CDN_BASE_URL,
};

use super::api::{WeixinApiClient, build_http_client};
use super::models::{FileItem, GetUploadUrlRequest, ImageItem, OutboundMessageItem};

type Aes128EcbEnc = ecb::Encryptor<Aes128>;

const UPLOAD_MEDIA_IMAGE: u64 = 1;
const UPLOAD_MEDIA_FILE: u64 = 3;
#[derive(Debug, Clone)]
pub struct UploadedMedia {
    pub encrypt_query_param: String,
    pub aes_key_base64: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_size_ciphertext: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum OutboundMediaKind {
    Image,
    File,
}

pub fn encrypt_aes_ecb(key: &[u8; 16], data: &[u8]) -> Vec<u8> {
    let enc = Aes128EcbEnc::new(key.into());
    enc.encrypt_padded_vec_mut::<Pkcs7>(data)
}

pub async fn upload_media(
    api_client: &WeixinApiClient,
    to_user_id: &str,
    file_path: &Path,
    kind: OutboundMediaKind,
) -> crate::Result<UploadedMedia> {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    let data = std::fs::read(file_path).context(IoSnafu)?;
    let file_size = data.len() as u64;
    let rawfilemd5 = format!("{:x}", md5::compute(&data));

    let key: [u8; 16] = rand::random();
    let aes_key_hex = hex::encode(key);
    let encrypted = encrypt_aes_ecb(&key, &data);
    let file_size_ciphertext = encrypted.len() as u64;
    let filekey: [u8; 16] = rand::random();
    let filekey_hex = hex::encode(filekey);
    let media_type = match kind {
        OutboundMediaKind::Image => UPLOAD_MEDIA_IMAGE,
        OutboundMediaKind::File => UPLOAD_MEDIA_FILE,
    };

    let upload_info = api_client
        .get_upload_url(&GetUploadUrlRequest::new(
            filekey_hex.clone(),
            media_type,
            to_user_id.to_string(),
            file_size,
            rawfilemd5,
            file_size_ciphertext,
            aes_key_hex.clone(),
        ))
        .await?;
    let upload_param = upload_info.upload_param().ok_or_else(|| {
        ApiSnafu {
            code: -1_i64,
            message: "no upload_param in response".to_owned(),
        }
        .build()
    })?;
    let upload_url = reqwest::Url::parse_with_params(
        &format!("{CDN_BASE_URL}/upload"),
        [
            ("encrypted_query_param", upload_param),
            ("filekey", filekey_hex.as_str()),
        ],
    )
    .map_err(|e| {
        ApiSnafu {
            code: -1_i64,
            message: format!("invalid upload url: {e}"),
        }
        .build()
    })?;

    let client = build_http_client();
    let resp = client
        .post(upload_url)
        .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
        .body(encrypted)
        .send()
        .await
        .context(HttpSnafu)?;
    let resp = resp.error_for_status().context(HttpSnafu)?;
    let encrypt_query_param = resp
        .headers()
        .get("x-encrypted-param")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            ApiSnafu {
                code: -1_i64,
                message: "CDN upload response missing x-encrypted-param".to_owned(),
            }
            .build()
        })?;

    Ok(UploadedMedia {
        encrypt_query_param: encrypt_query_param.to_string(),
        aes_key_base64: STANDARD.encode(aes_key_hex.as_bytes()),
        file_name: file_name.to_string(),
        file_size,
        file_size_ciphertext,
    })
}

pub fn build_media_item(kind: OutboundMediaKind, uploaded: &UploadedMedia) -> OutboundMessageItem {
    match kind {
        OutboundMediaKind::Image => OutboundMessageItem::image(ImageItem::from_uploaded(uploaded)),
        OutboundMediaKind::File => OutboundMessageItem::file(FileItem::from_uploaded(uploaded)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_aes_ecb_expands_to_block_size() {
        let key = [0x42u8; 16];
        let plaintext = b"Hello, WeChat media encryption!";
        let encrypted = encrypt_aes_ecb(&key, plaintext);
        assert_eq!(encrypted.len() % 16, 0);
        assert!(encrypted.len() >= plaintext.len());
    }

    #[test]
    fn test_build_file_media_item() {
        let item = build_media_item(
            OutboundMediaKind::File,
            &UploadedMedia {
                encrypt_query_param: "param".to_string(),
                aes_key_base64: "key".to_string(),
                file_name: "demo.txt".to_string(),
                file_size: 12,
                file_size_ciphertext: 16,
            },
        );
        assert_eq!(item.item_type, 4);
        assert_eq!(
            item.file_item.as_ref().map(|file| file.file_name.as_str()),
            Some("demo.txt")
        );
        assert_eq!(
            item.file_item.as_ref().map(|file| file.len.as_str()),
            Some("12")
        );
    }
}
