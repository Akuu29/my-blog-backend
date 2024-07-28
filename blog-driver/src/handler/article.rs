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
    repository::token::TokenRepository,
    usecase::{article::ArticleUseCase, token::TokenUseCase},
};
use blog_domain::model::articles::{
    article::{NewArticle, UpdateArticle},
    i_article_repository::ArticleRepository,
};
use std::sync::Arc;

pub async fn create_article<T: ArticleRepository, U: TokenRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUseCase<T>>>,
    Extension(token_usecase): Extension<Arc<TokenUseCase<U>>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    ValidatedJson(payload): ValidatedJson<NewArticle>,
) -> Result<impl IntoResponse, StatusCode> {
    let access_token = bearer.token().to_string();
    let _verified_token = token_usecase
        .verify_access_token(&access_token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    let article = article_usecase
        .create(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(article)))
}

pub async fn find_article<T: ArticleRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUseCase<T>>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let article = article_usecase
        .find(id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(article)))
}

pub async fn all_articles<T: ArticleRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUseCase<T>>>,
) -> Result<impl IntoResponse, StatusCode> {
    let articles = article_usecase
        .all()
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok((StatusCode::OK, Json(articles)))
}

pub async fn update_article<T: ArticleRepository, U: TokenRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUseCase<T>>>,
    Extension(token_usecase): Extension<Arc<TokenUseCase<U>>>,
    Path(id): Path<i32>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    ValidatedJson(payload): ValidatedJson<UpdateArticle>,
) -> Result<impl IntoResponse, StatusCode> {
    let access_token = bearer.token().to_string();
    let _verified_token = token_usecase
        .verify_access_token(&access_token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    let article = article_usecase
        .update(id, payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(article)))
}

pub async fn delete_article<T: ArticleRepository, U: TokenRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUseCase<T>>>,
    Extension(token_usecase): Extension<Arc<TokenUseCase<U>>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let access_token = bearer.token().to_string();
    let _verified_token = token_usecase
        .verify_access_token(&access_token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    article_usecase
        .delete(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    Ok(StatusCode::NO_CONTENT)
}
