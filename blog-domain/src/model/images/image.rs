use image::ImageFormat;
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::FromRow,
    types::chrono::{DateTime, Local},
};
use std::fmt;
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ImageData {
    pub mime_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ImageDataProps {
    #[serde(rename = "id")]
    pub public_id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub url: Option<String>,
    pub storage_type: String,
    #[serde(rename = "articleId")]
    pub article_public_id: Uuid,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "storage_type", rename_all = "lowercase")]
pub enum StorageType {
    Database,
}

impl fmt::Display for StorageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageType::Database => write!(f, "database"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NewImage {
    #[validate(length(min = 1, max = 255, message = "name length must be 1 to 255"))]
    pub name: String,
    #[validate(custom(function = "validate_mime_type"))]
    pub mime_type: String,
    #[validate(
        length(
            max = 5000000,
            code = "INVALID_DATA_LENGTH",
            message = "data length must be 5MB or less"
        ),
        custom(function = "validate_data_dimension")
    )]
    pub data: Vec<u8>,
    pub url: Option<String>,
    pub storage_type: StorageType,
    #[serde(rename = "articleId")]
    pub article_public_id: Uuid,
}

fn validate_mime_type(mime_type: &str) -> Result<(), ValidationError> {
    const ALLOWED_IMAGE_MIME_TYPES: [&str; 5] = [
        "image/jpg",
        "image/jpeg",
        "image/png",
        "image/gif",
        "image/webp",
    ];

    if ALLOWED_IMAGE_MIME_TYPES.contains(&mime_type) {
        Ok(())
    } else {
        Err(ValidationError::new("INVALID_MIME_TYPE"))
    }
}

fn validate_data_dimension(data: &[u8]) -> Result<(), ValidationError> {
    const MAX_IMAGE_WIDTH: u32 = 1920;
    const MAX_IMAGE_HEIGHT: u32 = 1080;

    let kind = infer::get(data).unwrap();
    let format = match kind.extension() {
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "png" => ImageFormat::Png,
        "gif" => ImageFormat::Gif,
        "webp" => ImageFormat::WebP,
        _ => return Err(ValidationError::new("INVALID_MIME_TYPE")),
    };

    let image = image::load_from_memory_with_format(data, format).unwrap();
    if image.width() > MAX_IMAGE_WIDTH || image.height() > MAX_IMAGE_HEIGHT {
        return Err(ValidationError::new("INVALID_IMAGE_DIMENSION"));
    }

    Ok(())
}
