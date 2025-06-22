use crate::{
    model::{api_response::ApiResponse, validated_json::ValidatedJson},
    utils::{app_error::AppError, error_handler::ErrorHandler},
};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
};
use blog_app::service::comments::comment_app_service::CommentAppService;
use blog_domain::model::comments::{
    comment::{NewComment, UpdateComment},
    i_comment_repository::ICommentRepository,
};
use std::sync::Arc;

#[tracing::instrument(name = "create_comment", skip(comment_app_service,))]
pub async fn create_comment<T>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    ValidatedJson(payload): ValidatedJson<NewComment>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: ICommentRepository,
{
    let comment = comment_app_service.create(payload).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to create comment")
    })?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&comment).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "find_comment", skip(comment_app_service))]
pub async fn find_comment<T>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: ICommentRepository,
{
    let comment = comment_app_service.find(id).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to find comment")
    })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&comment).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "find_by_article_id", skip(comment_app_service))]
pub async fn find_by_article_id<T>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: ICommentRepository,
{
    let comments = comment_app_service
        .find_by_article_id(article_id)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to find comments by article id")
        })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&comments).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "update_comment", skip(comment_app_service))]
pub async fn update_comment<T>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateComment>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: ICommentRepository,
{
    let comment = comment_app_service.update(id, payload).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to update comment")
    })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&comment).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "delete_comment", skip(comment_app_service))]
pub async fn delete_comment<T>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: ICommentRepository,
{
    comment_app_service.delete(id).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to delete comment")
    })?;

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}
