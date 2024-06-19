use crate::handler::ValidatedJson;
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
};
use blog_app::{
    model::auth::{SigninUser, SignupUser},
    repository::auth::AuthRepository,
    usecase::auth::AuthUseCase,
};
use std::sync::Arc;

pub async fn signup<T: AuthRepository>(
    Extension(auth_use_case): Extension<Arc<AuthUseCase<T>>>,
    ValidatedJson(payload): ValidatedJson<SignupUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_credentials = auth_use_case
        .signup(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(user_credentials)))
}

pub async fn signin<T: AuthRepository>(
    Extension(auth_use_case): Extension<Arc<AuthUseCase<T>>>,
    ValidatedJson(payload): ValidatedJson<SigninUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_credentials = auth_use_case
        .signin(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(user_credentials)))
}
