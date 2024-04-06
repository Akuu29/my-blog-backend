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
    Json(payload): Json<NewArticle>,
) -> Result<impl IntoResponse, StatusCode> {
    let article = article_usecase.create(payload).await;

    Ok((StatusCode::CREATED, Json(article)))
}

pub async fn find_article<T: ArticleRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUsecase<T>>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let article = article_usecase.find(id).await;

    Ok((StatusCode::OK, Json(article)))
}

pub async fn all_articles<T: ArticleRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUsecase<T>>>,
) -> Result<impl IntoResponse, StatusCode> {
    let articles = article_usecase.all().await;

    Ok((StatusCode::OK, Json(articles)))
}

pub async fn update_article<T: ArticleRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUsecase<T>>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateArticle>,
) -> Result<impl IntoResponse, StatusCode> {
    let article = article_usecase.update(id, payload).await;

    Ok((StatusCode::OK, Json(article)))
}

pub async fn delete_article<T: ArticleRepository>(
    Extension(article_usecase): Extension<Arc<ArticleUsecase<T>>>,
    Path(id): Path<i32>,
) -> StatusCode {
    article_usecase.delete(id).await;

    StatusCode::NO_CONTENT
}
