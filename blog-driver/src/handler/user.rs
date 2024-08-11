use crate::handler::ValidatedJson;
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use blog_app::service::users::user_app_service::UserAppService;
use blog_domain::model::users::{
    i_user_repository::IUserRepository,
    user::{NewUser, UpdateUser},
};
use std::sync::Arc;

pub async fn create<T: IUserRepository>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    ValidatedJson(payload): ValidatedJson<NewUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = user_app_service
        .create(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn find<T: IUserRepository>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = user_app_service
        .find(id)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(user)))
}

pub async fn update<T: IUserRepository>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = user_app_service
        .update(id, payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(user)))
}

pub async fn delete<T: IUserRepository>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    user_app_service
        .delete(id)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok(StatusCode::OK)
}
