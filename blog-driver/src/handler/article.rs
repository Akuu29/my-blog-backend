use crate::model::{
    api_response::ApiResponse, auth_token::AuthToken, paged_body::PagedBody,
    validated_json::ValidatedJson, validated_query_param::ValidatedQueryParam,
};
use axum::{
    extract::{Extension, Path},
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
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

pub async fn create_article<T, U>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<NewArticle>,
) -> Result<impl IntoResponse, ApiResponse<()>>
where
    T: IArticleRepository,
    U: ITokenRepository,
{
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

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&article).unwrap()),
        None,
    ))
}

pub async fn find_article<T>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<()>>
where
    T: IArticleRepository,
{
    let article = article_app_service
        .find(article_id, ArticleFilter::default())
        .await
        .or(Err(ApiResponse::new(StatusCode::NOT_FOUND, None, None)))?;

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
    pub article: ArticleFilter,
    #[serde(flatten)]
    #[validate(nested)]
    pub pagination: Pagination,
}

pub async fn all_articles<T>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    // QsQuery(query_param): QsQuery<AllArticlesQueryParam>,
    ValidatedQueryParam(query_param): ValidatedQueryParam<AllArticlesQueryParam>,
) -> Result<impl IntoResponse, ApiResponse<()>>
where
    T: IArticleRepository,
{
    let article_filter = query_param.article;
    let pagination = query_param.pagination;

    let articles = article_app_service
        .all(article_filter, pagination)
        .await
        .or(Err(ApiResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            None,
            None,
        )))?;

    let next_cursor = articles.last().map(|article| article.id).or(None);
    let paged_body = PagedBody::new(articles, next_cursor);

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&paged_body).unwrap()),
        None,
    ))
}

pub async fn update_article<T, U>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    Path(article_id): Path<i32>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<UpdateArticle>,
) -> Result<impl IntoResponse, ApiResponse<()>>
where
    T: IArticleRepository,
    U: ITokenRepository,
{
    let _access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            ApiResponse::new(StatusCode::UNAUTHORIZED, None, None)
        })?;

    let pre_article = article_app_service
        .find(article_id, ArticleFilter::default())
        .await
        .or(Err(ApiResponse::new(StatusCode::NOT_FOUND, None, None)))?;

    if (payload.title.is_none() && pre_article.title.is_none())
        || (payload.body.is_none() && pre_article.body.is_none())
    {
        return Err(ApiResponse::new(StatusCode::BAD_REQUEST, None, None));
    }

    let article = article_app_service
        .update(article_id, payload)
        .await
        .or(Err(ApiResponse::new(StatusCode::BAD_REQUEST, None, None)))?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&article).unwrap()),
        None,
    ))
}

pub async fn delete_article<T, U>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<()>>
where
    T: IArticleRepository,
    U: ITokenRepository,
{
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

#[derive(Debug, Deserialize, Validate)]
pub struct FindArticlesByTagQueryParam {
    #[serde(flatten)]
    #[validate(nested)]
    pub filter: ArticlesByTagFilter,
    #[serde(flatten)]
    #[validate(nested)]
    pub pagination: Pagination,
}

pub async fn find_articles_by_tag<T>(
    Extension(articles_by_tag_query_service): Extension<Arc<T>>,
    ValidatedQueryParam(query_param): ValidatedQueryParam<FindArticlesByTagQueryParam>,
) -> Result<impl IntoResponse, ApiResponse<()>>
where
    T: IArticlesByTagQueryService,
{
    let filter = query_param.filter;
    let pagination = query_param.pagination;

    let articles = articles_by_tag_query_service
        .find_article_title_by_tag(filter, pagination)
        .await
        .or(Err(ApiResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            None,
            None,
        )))?;

    let next_cursor = articles.last().map(|article| article.id).or(None);
    let paged_body = PagedBody::new(articles, next_cursor);

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&paged_body).unwrap()),
        None,
    ))
}
