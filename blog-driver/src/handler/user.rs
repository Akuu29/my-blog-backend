use crate::handler::ValidatedJson;
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
};
use blog_adapter::{
    model::user::{SigninUser, SignupUser},
    repository::user::UserRepository,
};
use std::sync::Arc;

pub async fn signup<T: UserRepository>(
    Extension(user_repository): Extension<Arc<T>>,
    ValidatedJson(payload): ValidatedJson<SignupUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = user_repository
        .signup(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn signin<T: UserRepository>(
    Extension(user_repository): Extension<Arc<T>>,
    ValidatedJson(payload): ValidatedJson<SigninUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = user_repository
        .signin(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(user)))
}
