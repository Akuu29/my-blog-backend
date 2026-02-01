use crate::{
    error::AppError, model::api_response::ApiResponse, service::cookie_service::CookieService,
};
use axum::{extract::Extension, http::StatusCode, response::IntoResponse};
use axum_extra::extract::cookie::PrivateCookieJar;
use blog_app::service::{
    tokens::token_app_service::TokenAppService, users::user_app_service::UserAppService,
};
use blog_domain::model::{
    tokens::{
        i_token_repository::ITokenRepository, token::ApiCredentials,
        token_string::RefreshTokenString,
    },
    users::i_user_repository::IUserRepository,
};
use std::sync::Arc;

#[tracing::instrument(
    name = "refresh_access_token",
    skip(token_app_service, user_app_service, cookie_service, jar)
)]
pub async fn refresh_access_token<T, U>(
    Extension(token_app_service): Extension<Arc<TokenAppService<T>>>,
    Extension(user_app_service): Extension<Arc<UserAppService<U>>>,
    Extension(cookie_service): Extension<Arc<CookieService>>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, AppError>
where
    T: ITokenRepository,
    U: IUserRepository,
{
    let refresh_token = cookie_service
        .get_refresh_token(&jar)
        .map_err(|e| AppError::from(e))?;
    let refresh_token = RefreshTokenString(refresh_token);

    let token_data = token_app_service
        .verify_refresh_token(refresh_token)
        .map_err(|e| AppError::from(e))?;

    let exists_user = user_app_service.find(token_data.claims.sub()).await;
    match exists_user {
        Ok(user) => {
            let access_token = token_app_service
                .generate_access_token(&user)
                .map_err(|e| AppError::from(e))?;
            let api_credentials = ApiCredentials::new(&access_token, user);

            Ok(ApiResponse::new(
                StatusCode::OK,
                Some(serde_json::to_string(&api_credentials).unwrap()),
                None,
            ))
        }
        Err(e) => Err(AppError::from(e)),
    }
}

#[tracing::instrument(name = "reset_refresh_token", skip(cookie_service, jar))]
pub async fn reset_refresh_token(
    Extension(cookie_service): Extension<Arc<CookieService>>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let updated_jar = cookie_service.clear_refresh_token(jar);

    Ok(ApiResponse::<()>::new(
        StatusCode::OK,
        None,
        Some(updated_jar),
    ))
}
