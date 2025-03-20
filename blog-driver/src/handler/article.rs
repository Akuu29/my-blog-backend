use crate::model::{
    api_response::ApiResponse, auth_token::AuthToken, paged_body::PagedBody,
    pagination::Pagination, validated_json::ValidatedJson,
    validated_query_param::ValidatedQueryParam,
};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::Query;
use blog_app::query_service::articles_by_tag::i_articles_by_tag_query_service::IArticlesByTagQueryService;
use blog_app::{
    service::articles::article_app_service::ArticleAppService,
    service::tokens::token_app_service::TokenAppService,
};
use blog_domain::model::{
    articles::{
        article::{NewArticle, UpdateArticle},
        i_article_repository::IArticleRepository,
    },
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use serde::Deserialize;
use std::sync::Arc;

pub async fn create_article<T: IArticleRepository, U: ITokenRepository>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<NewArticle>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            ApiResponse::new(StatusCode::UNAUTHORIZED, None, None)
        })?;

    let article = article_app_service
        .create(access_token_data.claims.sub(), payload)
        .await
        .or(Err(ApiResponse::new(StatusCode::BAD_REQUEST, None, None)))?;

    Ok(ApiResponse::new(StatusCode::CREATED, Some(article), None))
}

pub async fn find_article<T: IArticleRepository>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let article = article_app_service
        .find(article_id, None)
        .await
        .or(Err(ApiResponse::new(StatusCode::NOT_FOUND, None, None)))?;

    Ok(ApiResponse::new(StatusCode::OK, Some(article), None))
}

pub async fn all_articles<T: IArticleRepository>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    ValidatedQueryParam(pagination): ValidatedQueryParam<Pagination>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let articles = article_app_service
        .all(pagination.cursor, pagination.per_page)
        .await
        .or(Err(ApiResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            None,
            None,
        )))?;

    let next_cursor = articles.last().map(|article| article.id).or(None);
    let paged_body = PagedBody::new(articles, next_cursor);

    Ok(ApiResponse::new(StatusCode::OK, Some(paged_body), None))
}

pub async fn update_article<T: IArticleRepository, U: ITokenRepository>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    Path(article_id): Path<i32>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<UpdateArticle>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let _access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            ApiResponse::new(StatusCode::UNAUTHORIZED, None, None)
        })?;

    let article = article_app_service
        .update(article_id, payload)
        .await
        .or(Err(ApiResponse::new(StatusCode::BAD_REQUEST, None, None)))?;

    Ok(ApiResponse::new(StatusCode::OK, Some(article), None))
}

pub async fn delete_article<T: IArticleRepository, U: ITokenRepository>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let _access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            ApiResponse::new(StatusCode::UNAUTHORIZED, None, None)
        })?;

    article_app_service
        .delete(article_id)
        .await
        .map(|_| ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
        .unwrap_or(ApiResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            None,
            None,
        ));

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}

#[derive(Debug, Deserialize)]
pub struct TagIds {
    pub ids: Vec<i32>,
}
pub async fn find_articles_by_tag<T: IArticlesByTagQueryService>(
    Extension(articles_by_tag_query_service): Extension<Arc<T>>,
    Query(tag_ids): Query<TagIds>,
    ValidatedQueryParam(pagination): ValidatedQueryParam<Pagination>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let articles = articles_by_tag_query_service
        .find_article_title_by_tag(tag_ids.ids, pagination.cursor, pagination.per_page)
        .await
        .or(Err(ApiResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            None,
            None,
        )))?;

    let next_cursor = articles.last().map(|article| article.id).or(None);
    let paged_body = PagedBody::new(articles, next_cursor);

    Ok(ApiResponse::new(StatusCode::OK, Some(paged_body), None))
}
