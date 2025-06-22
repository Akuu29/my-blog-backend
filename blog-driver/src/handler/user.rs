use crate::{
    model::{
        api_response::ApiResponse,
        auth_token::AuthToken,
        error_message::{ErrorMessage, ErrorMessageKind},
        validated_json::ValidatedJson,
    },
    utils::{app_error::AppError, error_handler::ErrorHandler, error_log_kind::ErrorLogKind},
};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
};
use blog_app::service::{
    tokens::token_app_service::TokenAppService, users::user_app_service::UserAppService,
};
use blog_domain::model::{
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
    users::{
        i_user_repository::IUserRepository,
        user::{NewUser, UpdateUser, UserRole},
    },
};
use sqlx::types::Uuid;
use std::sync::Arc;

#[tracing::instrument(name = "create_user", skip(user_app_service, token_app_service, token))]
pub async fn create<T, U>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<NewUser>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IUserRepository,
    U: ITokenRepository,
{
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify access token")
        })?;

    if access_token_data.claims.role != UserRole::Admin {
        let err_log_msg = "User is not permitted to create user";
        tracing::error!(error.kind=%ErrorLogKind::Authorization, error.message=%err_log_msg);

        let err_msg = ErrorMessage::new(
            ErrorMessageKind::Forbidden,
            "User is not permitted to create user".to_string(),
        );
        return Err(ApiResponse::new(
            StatusCode::FORBIDDEN,
            Some(serde_json::to_string(&err_msg).unwrap()),
            None,
        ));
    };

    let user = user_app_service.create(payload).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to create user")
    })?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&user).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "find_user", skip(user_app_service))]
pub async fn find<T>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IUserRepository,
{
    let user = user_app_service.find(user_id).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to find user")
    })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&user).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "update_user", skip(user_app_service))]
pub async fn update<T>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Path(user_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateUser>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IUserRepository,
{
    let user = user_app_service
        .update(user_id, payload)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to update user")
        })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&user).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "delete_user", skip(user_app_service))]
pub async fn delete<T>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IUserRepository,
{
    user_app_service.delete(user_id).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to delete user")
    })?;

    Ok(ApiResponse::<()>::new(StatusCode::OK, None, None))
}
