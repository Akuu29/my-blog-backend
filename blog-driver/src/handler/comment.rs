use crate::model::{api_response::ApiResponse, validated_json::ValidatedJson};
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

pub async fn create_comment<T: ICommentRepository>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    ValidatedJson(payload): ValidatedJson<NewComment>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let comment = comment_app_service
        .create(payload)
        .await
        .or(Err(ApiResponse::new(StatusCode::BAD_REQUEST, None, None)))?;

    Ok(ApiResponse::new(StatusCode::CREATED, Some(comment), None))
}

pub async fn find_comment<T: ICommentRepository>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let comment = comment_app_service.find(id).await.or(Err(ApiResponse::new(
        StatusCode::NOT_FOUND,
        None,
        None,
    )))?;

    Ok(ApiResponse::new(StatusCode::OK, Some(comment), None))
}

pub async fn find_by_article_id<T: ICommentRepository>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Path(article_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let comments = comment_app_service
        .find_by_article_id(article_id)
        .await
        .or(Err(ApiResponse::new(StatusCode::NOT_FOUND, None, None)))?;

    Ok(ApiResponse::new(StatusCode::OK, Some(comments), None))
}

pub async fn update_comment<T: ICommentRepository>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateComment>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let comment = comment_app_service
        .update(id, payload)
        .await
        .or(Err(ApiResponse::new(StatusCode::BAD_REQUEST, None, None)))?;

    Ok(ApiResponse::new(StatusCode::OK, Some(comment), None))
}

pub async fn delete_comment<T: ICommentRepository>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    comment_app_service
        .delete(id)
        .await
        .map(|_| ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
        .unwrap_or(ApiResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            None,
            None,
        ));

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}
