use crate::{
    model::{
        api_response::ApiResponse, auth_token::AuthToken, paged_body::PagedBody,
        validated_json::ValidatedJson, validated_query_param::ValidatedQueryParam,
    },
    utils::{app_error::AppError, error_handler::ErrorHandler},
};
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use blog_app::{
    query_service::articles_by_tag::i_articles_by_tag_query_service::{
        ArticlesByTagFilter, IArticlesByTagQueryService,
    },
    service::articles::article_app_service::ArticleAppService,
    service::tokens::token_app_service::TokenAppService,
};
use blog_domain::model::{
    articles::{
        article::{NewArticle, UpdateArticle},
        i_article_repository::{ArticleFilter, IArticleRepository},
    },
    common::pagination::Pagination,
    tags::i_tag_repository::ITagRepository,
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[tracing::instrument(
    name = "create_article",
    skip(article_app_service, token_app_service, token)
)]
pub async fn create_article<T, U, W>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T, W>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<NewArticle>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IArticleRepository,
    U: ITokenRepository,
    W: ITagRepository,
{
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify access token")
        })?;

    let article = article_app_service
        .create(access_token_data.claims.sub(), payload)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to create article")
        })?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&article).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "find_article", skip(article_app_service))]
pub async fn find_article<T, W>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T, W>>>,
    Path(article_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IArticleRepository,
    W: ITagRepository,
{
    let article = article_app_service
        .find(article_id, ArticleFilter::default())
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Article not found")
        })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&article).unwrap()),
        None,
    ))
}

#[derive(Debug, Deserialize, Validate)]
pub struct AllArticlesQueryParam {
    #[serde(flatten)]
    #[validate(nested)]
    pub article_filter: ArticleFilter,
    #[serde(flatten)]
    #[validate(nested)]
    pub pagination: Pagination,
}

#[tracing::instrument(name = "all_articles", skip(article_app_service))]
pub async fn all_articles<T, U>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T, U>>>,
    ValidatedQueryParam(query_param): ValidatedQueryParam<AllArticlesQueryParam>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IArticleRepository,
    U: ITagRepository,
{
    let article_filter = query_param.article_filter;
    let pagination = query_param.pagination;

    let articles = article_app_service
        .all(article_filter, pagination)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to get all articles")
        })?;

    let next_cursor = articles.last().map(|article| article.public_id).or(None);
    let paged_body = PagedBody::new(articles, next_cursor);

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&paged_body).unwrap()),
        None,
    ))
}

#[tracing::instrument(
    name = "update_article",
    skip(article_app_service, token_app_service, token)
)]
pub async fn update_article<T, U, W>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T, W>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    Path(article_id): Path<Uuid>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<UpdateArticle>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IArticleRepository,
    U: ITokenRepository,
    W: ITagRepository,
{
    let _access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify access token")
        })?;

    let article = article_app_service
        .update(article_id, payload)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to update article")
        })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&article).unwrap()),
        None,
    ))
}

#[tracing::instrument(
    name = "delete_article",
    skip(article_app_service, token_app_service, token)
)]
pub async fn delete_article<T, U, W>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T, W>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(article_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IArticleRepository,
    U: ITokenRepository,
    W: ITagRepository,
{
    let _access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to delete article")
        })?;

    article_app_service
        .delete(article_id)
        .await
        .map(|_| ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to delete article")
        })?;

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}

#[tracing::instrument(
    name = "attach_tags",
    skip(article_app_service, token_app_service, token)
)]
pub async fn attach_tags<U, V, W>(
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    Extension(article_app_service): Extension<Arc<ArticleAppService<V, W>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(article_id): Path<Uuid>,
    Json(tag_ids): Json<Vec<Uuid>>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    U: ITokenRepository,
    V: IArticleRepository,
    W: ITagRepository,
{
    let _access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify access token")
        })?;

    article_app_service
        .attach_tags(article_id, tag_ids)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to attach tags to article")
        })?;

    Ok(ApiResponse::<()>::new(StatusCode::OK, None, None))
}

#[derive(Debug, Deserialize, Validate)]
pub struct FindArticlesByTagQueryParam {
    #[serde(flatten)]
    #[validate(nested)]
    pub filter: ArticlesByTagFilter,
    #[serde(flatten)]
    #[validate(nested)]
    pub pagination: Pagination,
}

impl std::fmt::Display for FindArticlesByTagQueryParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[tracing::instrument(name = "find_articles_by_tag", skip(articles_by_tag_query_service))]
pub async fn find_articles_by_tag<T>(
    Extension(articles_by_tag_query_service): Extension<Arc<T>>,
    ValidatedQueryParam(query_param): ValidatedQueryParam<FindArticlesByTagQueryParam>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IArticlesByTagQueryService,
{
    let filter = query_param.filter;
    let pagination = query_param.pagination;

    let articles = articles_by_tag_query_service
        .find_article_title_by_tag(filter, pagination)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to find articles by tag")
        })?;

    let next_cursor = articles.last().map(|article| article.public_id).or(None);
    let paged_body = PagedBody::new(articles, next_cursor);

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&paged_body).unwrap()),
        None,
    ))
}
