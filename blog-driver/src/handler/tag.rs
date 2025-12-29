use crate::{
    error::AppError,
    model::{
        api_response::ApiResponse, auth_token::AuthToken, paged_body::PagedBody,
        paged_filter_query_param::PagedFilterQueryParam, validated_json::ValidatedJson,
        validated_query_param::ValidatedQueryParam,
    },
};
use axum::{
    extract::{Extension, Path},
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
use uuid::Uuid;

#[tracing::instrument(name = "create_tag", skip(tag_app_service, token_app_service, token))]
pub async fn create<T, U>(
    Extension(tag_app_service): Extension<Arc<TagAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<NewTag>,
) -> Result<impl IntoResponse, AppError>
where
    T: ITagRepository,
    U: ITokenRepository,
{
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| AppError::from(e))?;

    let tag = tag_app_service
        .create(access_token_data.claims.sub(), payload)
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&tag).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "all_tags", skip(tag_app_service))]
pub async fn all<T>(
    Extension(tag_app_service): Extension<Arc<TagAppService<T>>>,
    ValidatedQueryParam(param): ValidatedQueryParam<PagedFilterQueryParam<TagFilter>>,
) -> Result<impl IntoResponse, AppError>
where
    T: ITagRepository,
{
    let mut pagination = param.pagination;
    // To check if there is a next page
    pagination.per_page += 1;

    let (mut tags, total) = tag_app_service
        .all(param.filter, pagination.clone())
        .await
        .map_err(|e| AppError::from(e))?;

    let has_next = tags.len() == pagination.per_page as usize;
    if has_next {
        tags.pop();
    }

    let next_cursor = tags.last().map(|tag| tag.public_id).or(None);
    let paged_body = PagedBody::new(tags, next_cursor, has_next, total.value());

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&paged_body).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "delete_tag", skip(tag_app_service, token_app_service, token))]
pub async fn delete<T, U>(
    Extension(tag_app_service): Extension<Arc<TagAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(tag_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError>
where
    T: ITagRepository,
    U: ITokenRepository,
{
    token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| AppError::from(e))?;

    tag_app_service
        .delete(tag_id)
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}

#[tracing::instrument(
    name = "find_tags_by_article_id",
    skip(tags_attached_article_query_service)
)]
pub async fn find_tags_by_article_id<T>(
    Extension(tags_attached_article_query_service): Extension<Arc<T>>,
    Path(article_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError>
where
    T: ITagsAttachedArticleQueryService,
{
    let tags = tags_attached_article_query_service
        .find_tags_by_article_id(article_id)
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&tags).unwrap()),
        None,
    ))
}
