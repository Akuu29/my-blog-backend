use axum::{
    extract::{Extension, Request},
    http::{header::AUTHORIZATION, StatusCode},
    response::IntoResponse,
};
use blog_app::{repository::token::TokenRepository, usecase::token::TokenUseCase};
use std::sync::Arc;

pub async fn verify_id_token<T: TokenRepository>(
    Extension(token_use_case): Extension<Arc<TokenUseCase<T>>>,
    req: Request,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_str()
        .or(Err(StatusCode::BAD_REQUEST))?;

    let id_token = auth_header.replace("Bearer ", "");
    let _ = token_use_case
        .verify_id_token(&id_token)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok(StatusCode::OK)
}

pub async fn verify_access_token<T: TokenRepository>(
    Extension(token_use_case): Extension<Arc<TokenUseCase<T>>>,
    req: Request,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_str()
        .or(Err(StatusCode::BAD_REQUEST))?;

    let access_token = auth_header.replace("Bearer ", "");
    let _ = token_use_case
        .verify_access_token(&access_token)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok(StatusCode::OK)
}
