use crate::{handler::ValidatedJson, model::auth_token::AuthToken};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::Query;
use blog_adapter::query_service::articles_by_tag::articles_tag_query_service::ArticlesByTagQueryService;
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
) -> Result<impl IntoResponse, StatusCode> {
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    let article = article_app_service
        .create(access_token_data.claims.sub(), payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(article)))
}

pub async fn find_article<T: IArticleRepository>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let article = article_app_service
        .find(article_id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(article)))
}

pub async fn all_articles<T: IArticleRepository>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
) -> Result<impl IntoResponse, StatusCode> {
    let articles = article_app_service
        .all()
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok((StatusCode::OK, Json(articles)))
}

pub async fn update_article<T: IArticleRepository, U: ITokenRepository>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    Path(article_id): Path<i32>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<UpdateArticle>,
) -> Result<impl IntoResponse, StatusCode> {
    let _access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    let article = article_app_service
        .update(article_id, payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(article)))
}

pub async fn delete_article<T: IArticleRepository, U: ITokenRepository>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let _access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    article_app_service
        .delete(article_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct TagIds {
    pub ids: Option<Vec<String>>,
}
pub async fn find_articles_by_tag<T: IArticlesByTagQueryService>(
    Extension(articles_by_tag_query_service): Extension<Arc<ArticlesByTagQueryService>>,
    Query(tag_ids): Query<TagIds>,
) -> Result<impl IntoResponse, StatusCode> {
    let articles = articles_by_tag_query_service
        .find_article_title_by_tag(tag_ids.ids)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok((StatusCode::OK, Json(articles)))
}
