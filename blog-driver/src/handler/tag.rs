use crate::model::{
    api_response::ApiResponse, auth_token::AuthToken, validated_json::ValidatedJson,
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
    tags::{i_tag_repository::ITagRepository, tag::NewTag, tag_filter::TagFilter},
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use http::StatusCode;
use std::sync::Arc;

pub async fn create<T: ITagRepository, U: ITokenRepository>(
    Extension(tag_app_service): Extension<Arc<TagAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<NewTag>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            ApiResponse::new(StatusCode::UNAUTHORIZED, None, None)
        })?;

    let tag = tag_app_service
        .create(access_token_data.claims.sub(), payload)
        .await
        .or(Err(ApiResponse::new(StatusCode::BAD_REQUEST, None, None)))?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&tag).unwrap()),
        None,
    ))
}

pub async fn all<T: ITagRepository>(
    Extension(tag_app_service): Extension<Arc<TagAppService<T>>>,
    Query(tag_filter): Query<TagFilter>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let tags = tag_app_service
        .all(tag_filter)
        .await
        .or(Err(ApiResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            None,
            None,
        )))?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&tags).unwrap()),
        None,
    ))
}

pub async fn delete<T: ITagRepository, U: ITokenRepository>(
    Extension(tag_app_service): Extension<Arc<TagAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(tag_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    token_app_service
        .verify_access_token(token)
        .await
        .map_err(|_| ApiResponse::new(StatusCode::UNAUTHORIZED, None, None))?;

    tag_app_service
        .delete(tag_id)
        .await
        .map(|_| ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
        .unwrap_or(ApiResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            None,
            None,
        ));

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}

pub async fn find_tags_by_article_id<T: ITagsAttachedArticleQueryService>(
    Extension(tags_attached_article_query_service): Extension<Arc<T>>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let tags = tags_attached_article_query_service
        .find_tags_by_article_id(article_id)
        .await
        .or(Err(ApiResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            None,
            None,
        )))?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&tags).unwrap()),
        None,
    ))
}
