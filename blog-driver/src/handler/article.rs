use crate::{
    error::AppError,
    model::{
        api_response::ApiResponse, auth_token::AuthToken, paged_body::PagedBody,
        paged_filter_query_param::PagedFilterQueryParam, validated_json::ValidatedJson,
        validated_query_param::ValidatedQueryParam,
    },
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
    tags::i_tag_repository::ITagRepository,
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use std::sync::Arc;
use uuid::Uuid;

#[tracing::instrument(
    name = "create_article",
    skip(article_app_service, token_app_service, token)
)]
pub async fn create_article<T, U, W>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T, W>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<NewArticle>,
) -> Result<impl IntoResponse, AppError>
where
    T: IArticleRepository,
    U: ITokenRepository,
    W: ITagRepository,
{
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| AppError::from(e))?;

    let article = article_app_service
        .create(access_token_data.claims.sub(), payload)
        .await
        .map_err(|e| AppError::from(e))?;

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
) -> Result<impl IntoResponse, AppError>
where
    T: IArticleRepository,
    W: ITagRepository,
{
    let article = article_app_service
        .find(article_id, ArticleFilter::default())
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&article).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "all_articles", skip(article_app_service))]
pub async fn all_articles<T, U>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T, U>>>,
    ValidatedQueryParam(param): ValidatedQueryParam<PagedFilterQueryParam<ArticleFilter>>,
) -> Result<impl IntoResponse, AppError>
where
    T: IArticleRepository,
    U: ITagRepository,
{
    let mut pagination = param.pagination;
    // To check if there is a next page
    pagination.per_page += 1;

    let (mut articles, total) = article_app_service
        .all(param.filter, pagination.clone())
        .await
        .map_err(|e| AppError::from(e))?;

    let has_next = articles.len() == pagination.per_page as usize;
    if has_next {
        articles.pop();
    }
    let next_cursor = articles.last().map(|article| article.public_id).or(None);
    let paged_body = PagedBody::new(articles, next_cursor, has_next, total.value());

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
) -> Result<impl IntoResponse, AppError>
where
    T: IArticleRepository,
    U: ITokenRepository,
    W: ITagRepository,
{
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| AppError::from(e))?;

    let user_id = access_token_data.claims.sub();

    let article = article_app_service
        .update(user_id, article_id, payload)
        .await
        .map_err(|e| AppError::from(e))?;

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
pub async fn delete_article<T, U, V>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T, V>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(article_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError>
where
    T: IArticleRepository,
    U: ITokenRepository,
    V: ITagRepository,
{
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| AppError::from(e))?;

    let user_id = access_token_data.claims.sub();

    article_app_service
        .delete(user_id, article_id)
        .await
        .map(|_| ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
        .map_err(|e| AppError::from(e))?;

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}

#[tracing::instrument(
    name = "attach_tags",
    skip(article_app_service, token_app_service, token)
)]
pub async fn attach_tags<T, U, V>(
    Extension(token_app_service): Extension<Arc<TokenAppService<T>>>,
    Extension(article_app_service): Extension<Arc<ArticleAppService<U, V>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(article_id): Path<Uuid>,
    Json(tag_ids): Json<Vec<Uuid>>,
) -> Result<impl IntoResponse, AppError>
where
    T: ITokenRepository,
    U: IArticleRepository,
    V: ITagRepository,
{
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| AppError::from(e))?;

    let user_id = access_token_data.claims.sub();

    article_app_service
        .attach_tags(user_id, article_id, tag_ids)
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(ApiResponse::<()>::new(StatusCode::OK, None, None))
}

#[tracing::instrument(name = "find_articles_by_tag", skip(articles_by_tag_query_service))]
pub async fn find_articles_by_tag<T>(
    Extension(articles_by_tag_query_service): Extension<Arc<T>>,
    ValidatedQueryParam(query_param): ValidatedQueryParam<
        PagedFilterQueryParam<ArticlesByTagFilter>,
    >,
) -> Result<impl IntoResponse, AppError>
where
    T: IArticlesByTagQueryService,
{
    let filter = query_param.filter;
    let mut pagination = query_param.pagination;
    // To check if there is a next page
    pagination.per_page += 1;

    let (mut articles, total) = articles_by_tag_query_service
        .find_article_title_by_tag(filter, pagination.clone())
        .await
        .map_err(|e| AppError::from(e))?;

    let has_next = articles.len() == pagination.per_page as usize;
    if has_next {
        articles.pop();
    }
    let next_cursor = articles.last().map(|article| article.public_id).or(None);
    let paged_body = PagedBody::new(articles, next_cursor, has_next, total.value());

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&paged_body).unwrap()),
        None,
    ))
}
