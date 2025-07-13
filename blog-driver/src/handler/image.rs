use crate::{
    model::{
        api_response::ApiResponse,
        auth_token::AuthToken,
        error_message::{ErrorMessage, ErrorMessageKind},
        validated_image::ValidatedImage,
    },
    utils::{app_error::AppError, error_handler::ErrorHandler, error_log_kind::ErrorLogKind},
};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use blog_app::{
    query_service::article_image::i_article_image_query_service::IArticleImageQueryService,
    service::{
        images::image_app_service::ImageAppService, tokens::token_app_service::TokenAppService,
    },
};
use blog_domain::model::{
    images::i_image_repository::{IImageRepository, ImageFilter},
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use std::sync::Arc;

#[tracing::instrument(
    name = "create_image",
    skip(image_app_service, token_app_service, token)
)]
pub async fn create<T, U>(
    Extension(image_app_service): Extension<Arc<ImageAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedImage(new_image): ValidatedImage,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IImageRepository,
    U: ITokenRepository,
{
    let _token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify access token")
        });

    let image = image_app_service.create(new_image).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to create image")
    })?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&image).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "all_images", skip(image_app_service))]
pub async fn all<T>(
    Extension(image_app_service): Extension<Arc<ImageAppService<T>>>,
    Query(filter): Query<ImageFilter>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IImageRepository,
{
    let images = image_app_service.all(filter).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to get all images")
    })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&images).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "find_data", skip(image_app_service))]
pub async fn find_data<T>(
    Extension(image_app_service): Extension<Arc<ImageAppService<T>>>,
    Path(image_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IImageRepository,
{
    let image_data = image_app_service.find_data(image_id).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to find image data")
    })?;

    let response = ApiResponse::new(StatusCode::OK, Some(image_data.data.clone()), None)
        .with_header("Content-Type", &image_data.mime_type)
        .with_header("Content-Disposition", "attachment")
        .with_header("Content-Length", &image_data.data.len().to_string());

    Ok(response)
}

#[tracing::instrument(
    name = "delete_image",
    skip(
        image_app_service,
        token_app_service,
        article_image_query_service,
        token,
    )
)]
pub async fn delete<T, U, E>(
    Extension(image_app_service): Extension<Arc<ImageAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    Extension(article_image_query_service): Extension<Arc<E>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(image_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IImageRepository,
    U: ITokenRepository,
    E: IArticleImageQueryService,
{
    let token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify access token")
        })?;

    let is_image_owned_by_user = article_image_query_service
        .is_image_owned_by_user(image_id, token_data.claims.sub())
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to check if image is owned by user")
        })?;

    if !is_image_owned_by_user {
        let err_log_msg = "Image is not owned by user";
        tracing::error!(error.kind=%ErrorLogKind::Authorization, error.message=%err_log_msg);

        let err_msg = ErrorMessage::new(
            ErrorMessageKind::Forbidden,
            "Image is not owned by user".to_string(),
        );

        return Err(ApiResponse::new(
            StatusCode::FORBIDDEN,
            Some(serde_json::to_string(&err_msg).unwrap()),
            None,
        ));
    }

    image_app_service.delete(image_id).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to delete image")
    })?;

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}
