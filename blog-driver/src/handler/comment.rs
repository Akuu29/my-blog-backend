use crate::handler::ValidatedJson;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use blog_app::service::comments::comment_app_service::CommentAppService;
use blog_domain::model::comments::{
    comment::{NewComment, UpdateComment},
    i_comment_repository::ICommentRepository,
};
use std::sync::Arc;

pub async fn create_comment<T: ICommentRepository>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    ValidatedJson(payload): ValidatedJson<NewComment>,
) -> Result<impl IntoResponse, StatusCode> {
    let comment = comment_app_service
        .create(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(comment)))
}

pub async fn find_comment<T: ICommentRepository>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let comment = comment_app_service
        .find(id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(comment)))
}

pub async fn find_by_article_id<T: ICommentRepository>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let comments = comment_app_service
        .find_by_article_id(article_id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(comments)))
}

pub async fn update_comment<T: ICommentRepository>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateComment>,
) -> Result<impl IntoResponse, StatusCode> {
    let comment = comment_app_service
        .update(id, payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(comment)))
}

pub async fn delete_comment<T: ICommentRepository>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Path(id): Path<i32>,
) -> StatusCode {
    comment_app_service
        .delete(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    StatusCode::NO_CONTENT
}
