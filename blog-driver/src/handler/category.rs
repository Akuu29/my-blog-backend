use crate::{
    model::{api_response::ApiResponse, auth_token::AuthToken, validated_json::ValidatedJson},
    utils::{app_error::AppError, error_handler::ErrorHandler},
};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use blog_app::service::{
    categories::category_app_service::CategoryAppService,
    tokens::token_app_service::TokenAppService,
};
use blog_domain::model::{
    categories::{
        category::{CategoryFilter, NewCategory, UpdateCategory},
        i_category_repository::ICategoryRepository,
    },
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use std::sync::Arc;
use uuid::Uuid;

#[tracing::instrument(
    name = "create_category",
    skip(category_app_service, token_app_service, token)
)]
pub async fn create_category<T, U>(
    Extension(category_app_service): Extension<Arc<CategoryAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<NewCategory>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: ICategoryRepository,
    U: ITokenRepository,
{
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify access token")
        })?;

    let category = category_app_service
        .create(access_token_data.claims.sub(), payload)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to create category")
        })?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&category).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "get_all_categories", skip(category_app_service))]
pub async fn all_categories<T>(
    Extension(category_app_service): Extension<Arc<CategoryAppService<T>>>,
    Query(category_filter): Query<CategoryFilter>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: ICategoryRepository,
{
    let categories = category_app_service
        .all(category_filter)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to get all categories")
        })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&categories).unwrap()),
        None,
    ))
}

#[tracing::instrument(
    name = "update_category",
    skip(category_app_service, token_app_service, token)
)]
pub async fn update_category<T, U>(
    Extension(category_app_service): Extension<Arc<CategoryAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(category_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateCategory>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: ICategoryRepository,
    U: ITokenRepository,
{
    let _access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify access token")
        })?;

    let category = category_app_service
        .update(category_id, payload)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to update category")
        })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&category).unwrap()),
        None,
    ))
}

#[tracing::instrument(
    name = "delete_category",
    skip(category_app_service, token_app_service, token)
)]
pub async fn delete_category<T, U>(
    Extension(category_app_service): Extension<Arc<CategoryAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(category_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: ICategoryRepository,
    U: ITokenRepository,
{
    let _access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify access token")
        })?;

    category_app_service
        .delete(category_id)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to delete category")
        })?;

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}
