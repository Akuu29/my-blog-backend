use crate::{handler::ValidatedJson, model::auth_token::AuthToken};
use axum::{
    extract::{Extension, Path, Query},
    response::IntoResponse,
    Json,
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
) -> Result<impl IntoResponse, StatusCode> {
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    let tag = tag_app_service
        .create(access_token_data.claims.sub(), payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(tag)))
}

pub async fn all<T: ITagRepository>(
    Extension(tag_app_service): Extension<Arc<TagAppService<T>>>,
    Query(tag_filter): Query<TagFilter>,
) -> Result<impl IntoResponse, StatusCode> {
    let tags = tag_app_service
        .all(tag_filter)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok((StatusCode::OK, Json(tags)))
}

pub async fn delete<T: ITagRepository, U: ITokenRepository>(
    Extension(tag_app_service): Extension<Arc<TagAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(tag_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    token_app_service
        .verify_access_token(token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    tag_app_service
        .delete(tag_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    Ok(StatusCode::NO_CONTENT)
}

pub async fn find_tags_by_article_id<T: ITagsAttachedArticleQueryService>(
    Extension(tags_attached_article_query_service): Extension<Arc<T>>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let tags = tags_attached_article_query_service
        .find_tags_by_article_id(article_id)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok((StatusCode::OK, Json(tags)))
}
