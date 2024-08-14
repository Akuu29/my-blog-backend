use crate::handler::ValidatedJson;
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
};
use blog_app::{
    model::auth::{
        auth::{SigninUser, SignupUser},
        i_auth_repository::IAuthRepository,
    },
    service::auth::auth_app_service::AuthAppService,
};
use std::sync::Arc;

pub async fn signup<T: IAuthRepository>(
    Extension(auth_app_service): Extension<Arc<AuthAppService<T>>>,
    ValidatedJson(payload): ValidatedJson<SignupUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_credentials = auth_app_service
        .signup(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(user_credentials)))
}

pub async fn signin<T: IAuthRepository>(
    Extension(auth_app_service): Extension<Arc<AuthAppService<T>>>,
    ValidatedJson(payload): ValidatedJson<SigninUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_credentials = auth_app_service
        .signin(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(user_credentials)))
}
