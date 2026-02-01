use crate::{
    error::{ErrorCode, ErrorResponse},
    model::api_response::ApiResponse,
};
use axum::extract::Multipart;
use axum::{
    async_trait,
    extract::{FromRequest, Request},
    http::StatusCode,
};
use blog_domain::model::images::image::{NewImage, StorageType};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug)]
pub struct ValidatedImage(pub NewImage);

#[async_trait]
impl<B> FromRequest<B> for ValidatedImage
where
    B: Send + Sync,
{
    type Rejection = ApiResponse<String>;

    #[tracing::instrument(name = "validated_image", skip(state))]
    async fn from_request(req: Request, state: &B) -> Result<Self, Self::Rejection> {
        let mut multipart = Multipart::from_request(req, state).await.unwrap();

        let mut file_data = Vec::default();
        let mut filename = None;
        let mut article_id = None;

        while let Some(field) = multipart.next_field().await.map_err(|e| {
            tracing::error!(error.kind="Unexpected", errror=%e.to_string());
            let err_res_body = ErrorResponse::new(
                ErrorCode::InvalidInput,
                format!("Failed to process multipart form data: {}", e),
            );
            ApiResponse::new(
                StatusCode::BAD_REQUEST,
                Some(serde_json::to_string(&err_res_body).unwrap()),
                None,
            )
        })? {
            let name = field
                .name()
                .ok_or_else(|| {
                    ApiResponse::new(
                        StatusCode::BAD_REQUEST,
                        Some(
                            serde_json::to_string(&ErrorResponse::new(
                                ErrorCode::InvalidInput,
                                "Missing field name".to_string(),
                            ))
                            .unwrap(),
                        ),
                        None,
                    )
                })?
                .to_string();

            match name.as_str() {
                "file" => {
                    let data = field.bytes().await.map_err(|e| {
                        tracing::error!(error.kind="Unexpected", errror=%e.to_string());
                        let err_msg = ErrorResponse::new(
                            ErrorCode::InvalidInput,
                            "Failed to read file data".to_string(),
                        );
                        ApiResponse::new(
                            StatusCode::BAD_REQUEST,
                            Some(serde_json::to_string(&err_msg).unwrap()),
                            None,
                        )
                    })?;
                    file_data = data.to_vec();
                }
                "filename" => {
                    filename = Some(field.text().await.map_err(|e| {
                        tracing::error!(error.kind="Unexpected", errror=%e.to_string());
                        let err_msg = ErrorResponse::new(
                            ErrorCode::InvalidInput,
                            "Failed to read filename".to_string(),
                        );
                        ApiResponse::new(
                            StatusCode::BAD_REQUEST,
                            Some(serde_json::to_string(&err_msg).unwrap()),
                            None,
                        )
                    })?);
                }
                "articleId" => {
                    article_id = Some(field.text().await.map_err(|e| {
                        tracing::error!(error.kind="Unexpected", errror=%e.to_string());
                        let err_msg = ErrorResponse::new(
                            ErrorCode::InvalidInput,
                            "Failed to read articleId".to_string(),
                        );
                        ApiResponse::new(
                            StatusCode::BAD_REQUEST,
                            Some(serde_json::to_string(&err_msg).unwrap()),
                            None,
                        )
                    })?);
                }
                _ => {}
            }
        }

        let kind =
            infer::get(&file_data).ok_or(ApiResponse::new(StatusCode::BAD_REQUEST, None, None))?;

        let article_public_id = match article_id.as_deref() {
            Some(id) => id.parse::<Uuid>().map_err(|e| {
                tracing::error!(error.kind="Unexpected", errror=%e.to_string());
                let err_msg = ErrorResponse::new(
                    ErrorCode::InvalidInput,
                    "Failed to parse articleId".to_string(),
                );
                ApiResponse::new(
                    StatusCode::BAD_REQUEST,
                    Some(serde_json::to_string(&err_msg).unwrap()),
                    None,
                )
            })?,
            None => {
                tracing::error!(error.kind = "Unexpected", errror = "articleId is required");
                let err_msg = ErrorResponse::new(
                    ErrorCode::InvalidInput,
                    "articleId is required".to_string(),
                );
                return Err(ApiResponse::new(
                    StatusCode::BAD_REQUEST,
                    Some(serde_json::to_string(&err_msg).unwrap()),
                    None,
                ));
            }
        };

        let new_image = NewImage {
            name: filename.unwrap(),
            mime_type: kind.mime_type().to_string(),
            data: file_data,
            url: None,
            storage_type: StorageType::Database,
            article_public_id,
        };

        new_image.validate().map_err(|e| {
            tracing::error!(error.kind ="Validation", error.message=%e.to_string());

            let (status_code, err_msg) = e
                .field_errors()
                .iter()
                .flat_map(|(_, errors)| errors.iter())
                .next()
                .map(|err| match err.code.as_ref() {
                    "INVALID_MIME_TYPE" => (
                        StatusCode::UNSUPPORTED_MEDIA_TYPE,
                        "Unsupported image format",
                    ),
                    "INVALID_DATA_LENGTH" => (
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "Image file size exceeds the maximum allowed",
                    ),
                    "INVALID_IMAGE_DIMENSION" => (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "Image dimensions do not meet requirements",
                    ),
                    _ => (StatusCode::BAD_REQUEST, "Invalid image"),
                })
                .unwrap_or((StatusCode::BAD_REQUEST, "Invalid image"));
            let res_body = ErrorResponse::new(ErrorCode::ValidationError, err_msg);

            ApiResponse::new(
                status_code,
                Some(serde_json::to_string(&res_body).unwrap()),
                None,
            )
        })?;

        Ok(ValidatedImage(new_image))
    }
}
