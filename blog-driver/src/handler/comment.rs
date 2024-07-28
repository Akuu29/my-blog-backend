use crate::handler::ValidatedJson;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use blog_app::usecase::comment::CommentUseCase;
use blog_domain::model::comments::{
    comment::{NewComment, UpdateComment},
    i_comment_repository::CommentRepository,
};
use std::sync::Arc;

pub async fn create_comment<T: CommentRepository>(
    Extension(comment_use_case): Extension<Arc<CommentUseCase<T>>>,
    ValidatedJson(payload): ValidatedJson<NewComment>,
) -> Result<impl IntoResponse, StatusCode> {
    let comment = comment_use_case
        .create(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(comment)))
}

pub async fn find_comment<T: CommentRepository>(
    Extension(comment_use_case): Extension<Arc<CommentUseCase<T>>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let comment = comment_use_case
        .find(id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(comment)))
}

pub async fn find_by_article_id<T: CommentRepository>(
    Extension(comment_use_case): Extension<Arc<CommentUseCase<T>>>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let comments = comment_use_case
        .find_by_article_id(article_id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(comments)))
}

pub async fn update_comment<T: CommentRepository>(
    Extension(comment_use_case): Extension<Arc<CommentUseCase<T>>>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateComment>,
) -> Result<impl IntoResponse, StatusCode> {
    let comment = comment_use_case
        .update(id, payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(comment)))
}

pub async fn delete_comment<T: CommentRepository>(
    Extension(comment_use_case): Extension<Arc<CommentUseCase<T>>>,
    Path(id): Path<i32>,
) -> StatusCode {
    comment_use_case
        .delete(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    StatusCode::NO_CONTENT
}
