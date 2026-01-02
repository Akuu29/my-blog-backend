use crate::{
    error::AppError,
    model::{
        api_response::ApiResponse, auth_token::AuthToken, paged_body::PagedBody,
        paged_filter_query_param::PagedFilterQueryParam, validated_json::ValidatedJson,
        validated_query_param::ValidatedQueryParam,
    },
};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
};
use blog_app::service::{
    comments::comment_app_service::CommentAppService, tokens::token_app_service::TokenAppService,
};
use blog_domain::model::{
    comments::{
        comment::{NewComment, UpdateComment},
        i_comment_repository::{CommentFilter, ICommentRepository},
    },
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use std::sync::Arc;
use uuid::Uuid;

#[tracing::instrument(
    name = "create_comment",
    skip(comment_app_service, token_app_service, token)
)]
pub async fn create_comment<T, U>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    token: Option<AuthToken<AccessTokenString>>,
    ValidatedJson(payload): ValidatedJson<NewComment>,
) -> Result<impl IntoResponse, AppError>
where
    T: ICommentRepository,
    U: ITokenRepository,
{
    // Determine user identification based on token presence
    let user_public_id = match token {
        Some(AuthToken(token_string)) => {
            // Logged-in user: extract ID from access token
            let token_data = token_app_service
                .verify_access_token(token_string)
                .await
                .map_err(|e| AppError::from(e))?;
            Some(token_data.claims.sub())
        }
        None => {
            // Guest user: user_name must be provided
            if payload.user_name.is_none() {
                return Err(AppError::ValidationFailed(
                    "user_name is required for guest users".to_string(),
                ));
            }
            None
        }
    };

    let comment = comment_app_service
        .create(user_public_id, payload)
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&comment).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "find_comment", skip(comment_app_service))]
pub async fn find_comment<T>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Path(comment_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError>
where
    T: ICommentRepository,
{
    let comment = comment_app_service
        .find(comment_id)
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&comment).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "all_comments", skip(comment_app_service))]
pub async fn all_comments<T>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    ValidatedQueryParam(param): ValidatedQueryParam<PagedFilterQueryParam<CommentFilter>>,
) -> Result<impl IntoResponse, AppError>
where
    T: ICommentRepository,
{
    let mut pagination = param.pagination;
    // To Check if there is a next page
    pagination.per_page += 1;

    let (mut comments, total) = comment_app_service
        .all(param.filter, pagination.clone())
        .await
        .map_err(|e| AppError::from(e))?;

    let has_next = comments.len() == pagination.per_page as usize;
    if has_next {
        comments.pop();
    }
    let next_cursor = comments.last().map(|comment| comment.public_id).or(None);
    let paged_body = PagedBody::new(comments, next_cursor, has_next, total.value());

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&paged_body).unwrap()),
        None,
    ))
}

#[tracing::instrument(
    name = "update_comment",
    skip(comment_app_service, token_app_service, token)
)]
pub async fn update_comment<T, U>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(comment_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateComment>,
) -> Result<impl IntoResponse, AppError>
where
    T: ICommentRepository,
    U: ITokenRepository,
{
    let token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| AppError::from(e))?;

    let comment = comment_app_service
        .update_with_auth(comment_id, token_data.claims.sub(), payload)
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&comment).unwrap()),
        None,
    ))
}

#[tracing::instrument(
    name = "delete_comment",
    skip(comment_app_service, token_app_service, token)
)]
pub async fn delete_comment<T, U>(
    Extension(comment_app_service): Extension<Arc<CommentAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(comment_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError>
where
    T: ICommentRepository,
    U: ITokenRepository,
{
    let token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| AppError::from(e))?;

    comment_app_service
        .delete_with_auth(comment_id, token_data.claims.sub())
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}
