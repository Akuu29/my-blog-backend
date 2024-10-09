use crate::handler::ValidatedJson;
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use blog_app::service::{
    tokens::token_app_service::TokenAppService, users::user_app_service::UserAppService,
};
use blog_domain::model::{
    tokens::i_token_repository::ITokenRepository,
    users::{
        i_user_repository::IUserRepository,
        user::{NewUser, UpdateUser, UserRole},
    },
};
use sqlx::types::Uuid;
use std::sync::Arc;

pub async fn create<T: IUserRepository, U: ITokenRepository>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    ValidatedJson(payload): ValidatedJson<NewUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let access_token = bearer.token().to_string();
    let access_token_data = token_app_service
        .verify_access_token(&access_token)
        .await
        .map_err(|e| {
            tracing::info!("Failed to verify access token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    if access_token_data.claims.role != UserRole::Admin {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let user = user_app_service
        .create(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn find<T: IUserRepository>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = user_app_service
        .find(user_id)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(user)))
}

pub async fn update<T: IUserRepository>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Path(user_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = user_app_service
        .update(user_id, payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(user)))
}

pub async fn delete<T: IUserRepository>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    user_app_service
        .delete(user_id)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok(StatusCode::OK)
}
