use crate::handler::ValidatedJson;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use blog_app::{
    service::articles::article_app_service::ArticleAppService,
    service::tokens::token_app_service::TokenAppService,
};
use blog_domain::model::{
    articles::{
        article::{NewArticle, UpdateArticle},
        i_article_repository::IArticleRepository,
    },
    tokens::i_token_repository::ITokenRepository,
};
use std::sync::Arc;

pub async fn create_article<T: IArticleRepository, U: ITokenRepository>(
    Extension(article_app_service): Extension<Arc<ArticleAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    ValidatedJson(payload): ValidatedJson<NewArticle>,
) -> Result<impl IntoResponse, StatusCode> {
    let access_token = bearer.token().to_string();
    let access_token_data = token_app_service
        .verify_access_token(&access_token)
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
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    ValidatedJson(payload): ValidatedJson<UpdateArticle>,
) -> Result<impl IntoResponse, StatusCode> {
    let access_token = bearer.token().to_string();
    let _access_token_data = token_app_service
        .verify_access_token(&access_token)
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
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let access_token = bearer.token().to_string();
    let _access_token_data = token_app_service
        .verify_access_token(&access_token)
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
