use crate::handler::ValidatedJson;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use blog_app::usecase::article::ArticleUsecase;
use blog_domain::{
    model::article::{NewArticle, UpdateArticle},
    repository::article::ArticleRepository,
};
use std::sync::Arc;

pub async fn create_article<T: ArticleRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUsecase<T>>>,
    ValidatedJson(payload): ValidatedJson<NewArticle>,
) -> Result<impl IntoResponse, StatusCode> {
    let article = article_usecase
        .create(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(article)))
}

pub async fn find_article<T: ArticleRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUsecase<T>>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let article = article_usecase
        .find(id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(article)))
}

pub async fn all_articles<T: ArticleRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUsecase<T>>>,
) -> Result<impl IntoResponse, StatusCode> {
    let articles = article_usecase
        .all()
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok((StatusCode::OK, Json(articles)))
}

pub async fn update_article<T: ArticleRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUsecase<T>>>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateArticle>,
) -> Result<impl IntoResponse, StatusCode> {
    let article = article_usecase
        .update(id, payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(article)))
}

pub async fn delete_article<T: ArticleRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUsecase<T>>>,
    Path(id): Path<i32>,
) -> StatusCode {
    article_usecase
        .delete(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    StatusCode::NO_CONTENT
}
