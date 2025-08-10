use crate::{
    model::{
        api_response::ApiResponse,
        error_message::{ErrorMessage, ErrorMessageKind},
    },
    utils::error_log_kind::ErrorLogKind,
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
            tracing::error!(error.kind=%ErrorLogKind::Unexpected, errror=%e.to_string());
            let err_msg = ErrorMessage::new(
                ErrorMessageKind::BadRequest,
                format!("Failed to process multipart form data: {}", e),
            );
            ApiResponse::new(
                StatusCode::BAD_REQUEST,
                Some(serde_json::to_string(&err_msg).unwrap()),
                None,
            )
        })? {
            let name = field
                .name()
                .ok_or_else(|| {
                    ApiResponse::new(
                        StatusCode::BAD_REQUEST,
                        Some(
                            serde_json::to_string(&ErrorMessage::new(
                                ErrorMessageKind::BadRequest,
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
                        tracing::error!(error.kind=%ErrorLogKind::Unexpected, errror=%e.to_string());
                        let err_msg = ErrorMessage::new(
                            ErrorMessageKind::BadRequest,
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
                        tracing::error!(error.kind=%ErrorLogKind::Unexpected, errror=%e.to_string());
                        let err_msg = ErrorMessage::new(
                            ErrorMessageKind::BadRequest,
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
                        tracing::error!(error.kind=%ErrorLogKind::Unexpected, errror=%e.to_string());
                        let err_msg = ErrorMessage::new(
                            ErrorMessageKind::BadRequest,
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
                tracing::error!(error.kind=%ErrorLogKind::Unexpected, errror=%e.to_string());
                let err_msg = ErrorMessage::new(
                    ErrorMessageKind::BadRequest,
                    "Failed to parse articleId".to_string(),
                );
                ApiResponse::new(
                    StatusCode::BAD_REQUEST,
                    Some(serde_json::to_string(&err_msg).unwrap()),
                    None,
                )
            })?,
            None => {
                tracing::error!(error.kind=%ErrorLogKind::Unexpected, errror="articleId is required");
                let err_msg = ErrorMessage::new(
                    ErrorMessageKind::BadRequest,
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
            let err_log_msg = e.to_string();
            tracing::error!(error.message=%err_log_msg);

            let err_msg = ErrorMessage::new(
                ErrorMessageKind::Validation,
                "Failed to validate image".to_string(),
            );

            let field_errors = e.field_errors();
            for field in field_errors.iter() {
                for error in field.1.iter() {
                    match error.code.as_ref() {
                        "INVALID_MIME_TYPE" => {
                            return ApiResponse::new(
                                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                                Some(serde_json::to_string(&err_msg).unwrap()),
                                None,
                            );
                        }
                        "INVALID_DATA_LENGTH" => {
                            return ApiResponse::new(
                                StatusCode::PAYLOAD_TOO_LARGE,
                                Some(serde_json::to_string(&err_msg).unwrap()),
                                None,
                            );
                        }
                        "INVALID_IMAGE_DIMENSION" => {
                            return ApiResponse::new(
                                StatusCode::UNPROCESSABLE_ENTITY,
                                Some(serde_json::to_string(&err_msg).unwrap()),
                                None,
                            );
                        }
                        _ => return ApiResponse::new(StatusCode::BAD_REQUEST, None, None),
                    }
                }
            }

            ApiResponse::new(
                StatusCode::BAD_REQUEST,
                Some(serde_json::to_string(&err_msg).unwrap()),
                None,
            )
        })?;

        Ok(ValidatedImage(new_image))
    }
}
