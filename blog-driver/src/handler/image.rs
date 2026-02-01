use crate::{
    error::AppError,
    model::{api_response::ApiResponse, auth_token::AuthToken, validated_image::ValidatedImage},
};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use blog_app::service::{
    images::image_app_service::ImageAppService, tokens::token_app_service::TokenAppService,
};
use blog_domain::model::{
    images::i_image_repository::{IImageRepository, ImageFilter},
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use std::sync::Arc;
use uuid::Uuid;

#[tracing::instrument(
    name = "create_image",
    skip_all,
    err,
    fields(image.mime_type)
)]
pub async fn create<TokenRepo, U>(
    Extension(image_app_service): Extension<Arc<ImageAppService<U>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<TokenRepo>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedImage(new_image): ValidatedImage,
) -> Result<impl IntoResponse, AppError>
where
    TokenRepo: ITokenRepository,
    U: IImageRepository,
{
    let _token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| AppError::from(e))?;

    tracing::Span::current().record("image.mime_type", &new_image.mime_type);

    let image = image_app_service
        .create(new_image)
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&image).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "all_images", skip_all, err)]
pub async fn all<T>(
    Extension(image_app_service): Extension<Arc<ImageAppService<T>>>,
    Query(filter): Query<ImageFilter>,
) -> Result<impl IntoResponse, AppError>
where
    T: IImageRepository,
{
    let images = image_app_service
        .all(filter)
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&images).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "find_data", skip_all, err, fields(image.id = %image_id))]
pub async fn find_data<T>(
    Extension(image_app_service): Extension<Arc<ImageAppService<T>>>,
    Path(image_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError>
where
    T: IImageRepository,
{
    let image_data = image_app_service
        .find_data(image_id)
        .await
        .map_err(|e| AppError::from(e))?;

    let response = ApiResponse::new(StatusCode::OK, Some(image_data.data.clone()), None)
        .with_header("Content-Type", &image_data.mime_type)
        .with_header("Content-Disposition", "attachment")
        .with_header("Content-Length", &image_data.data.len().to_string());

    Ok(response)
}

#[tracing::instrument(
    name = "delete_image",
    skip_all,
    err,
    fields(
        image.id = %image_id,
        user.id
    )
)]
pub async fn delete<TokenRepo, U>(
    Extension(image_app_service): Extension<Arc<ImageAppService<U>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<TokenRepo>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(image_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError>
where
    TokenRepo: ITokenRepository,
    U: IImageRepository,
{
    let token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| AppError::from(e))?;

    tracing::Span::current().record("user.id", &token_data.claims.sub().to_string());

    // Use the new delete_with_auth method which includes authorization check
    image_app_service
        .delete_with_auth(image_id, token_data.claims.sub())
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}
