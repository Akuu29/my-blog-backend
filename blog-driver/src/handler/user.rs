use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
};
use blog_app::{repository::user::UserRepository, usecase::user::UserUseCase};
use std::sync::Arc;

pub async fn update<T: UserRepository>(
    Extension(user_use_case): Extension<Arc<UserUseCase<T>>>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = user_use_case
        .update()
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(user)))
}

pub async fn delete<T: UserRepository>(
    Extension(user_use_case): Extension<Arc<UserUseCase<T>>>,
) -> Result<impl IntoResponse, StatusCode> {
    user_use_case
        .delete()
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok(StatusCode::OK)
}
