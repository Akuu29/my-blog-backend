use crate::{
    model::{api_response::ApiResponse, auth_token::AuthToken, validated_json::ValidatedJson},
    utils::{app_error::AppError, error_handler::ErrorHandler},
};
use axum::{
    extract::{Extension, Path, Query},
    response::IntoResponse,
};
use blog_app::{
    query_service::tags_attached_article::i_tags_attached_article_query_service::ITagsAttachedArticleQueryService,
    service::{tags::tag_app_service::TagAppService, tokens::token_app_service::TokenAppService},
};
use blog_domain::model::{
    tags::{
        i_tag_repository::{ITagRepository, TagFilter},
        tag::NewTag,
    },
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use http::StatusCode;
use std::sync::Arc;

#[tracing::instrument(name = "create_tag", skip(tag_app_service, token_app_service, token))]
pub async fn create<T: ITagRepository, U: ITokenRepository>(
    Extension(tag_app_service): Extension<Arc<TagAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<NewTag>,
) -> Result<impl IntoResponse, ApiResponse<String>> {
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify access token")
        })?;

    let tag = tag_app_service
        .create(access_token_data.claims.sub(), payload)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to create tag")
        })?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&tag).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "all_tags", skip(tag_app_service))]
pub async fn all<T: ITagRepository>(
    Extension(tag_app_service): Extension<Arc<TagAppService<T>>>,
    Query(tag_filter): Query<TagFilter>,
) -> Result<impl IntoResponse, ApiResponse<String>> {
    let tags = tag_app_service.all(tag_filter).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to get all tags")
    })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&tags).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "delete_tag", skip(tag_app_service, token_app_service, token))]
pub async fn delete<T: ITagRepository, U: ITokenRepository>(
    Extension(tag_app_service): Extension<Arc<TagAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(tag_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<String>> {
    token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify access token")
        })?;

    tag_app_service.delete(tag_id).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to delete tag")
    })?;

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}

#[tracing::instrument(
    name = "find_tags_by_article_id",
    skip(tags_attached_article_query_service)
)]
pub async fn find_tags_by_article_id<T: ITagsAttachedArticleQueryService>(
    Extension(tags_attached_article_query_service): Extension<Arc<T>>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<String>> {
    let tags = tags_attached_article_query_service
        .find_tags_by_article_id(article_id)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to find tags by article id")
        })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&tags).unwrap()),
        None,
    ))
}
