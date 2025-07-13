use crate::{
    model::{api_response::ApiResponse, auth_token::AuthToken},
    service::cookie_service::CookieService,
    utils::{app_error::AppError, error_handler::ErrorHandler},
};
use axum::{extract::Extension, http::StatusCode, response::IntoResponse};
use axum_extra::extract::cookie::PrivateCookieJar;
use blog_adapter::utils::repository_error::RepositoryError;
use blog_app::service::{
    tokens::token_app_service::TokenAppService, users::user_app_service::UserAppService,
};
use blog_domain::model::{
    tokens::{
        i_token_repository::ITokenRepository,
        token::ApiCredentials,
        token_string::{IdTokenString, RefreshTokenString},
    },
    users::{i_user_repository::IUserRepository, user::NewUser},
};
use std::sync::Arc;

// TODO CSRF Token
#[tracing::instrument(
    name = "verify_id_token",
    skip(token_app_service, user_app_service, cookie_service, token, jar)
)]
pub async fn verify_id_token<S, T>(
    Extension(token_app_service): Extension<Arc<TokenAppService<S>>>,
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Extension(cookie_service): Extension<Arc<CookieService>>,
    AuthToken(token): AuthToken<IdTokenString>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    S: ITokenRepository,
    T: IUserRepository,
{
    let id_token_data = token_app_service
        .verify_id_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify id token")
        })?;
    let id_token_claims = id_token_data.claims;

    let exists_user = user_app_service
        .find_by_idp_sub(&id_token_claims.sub())
        .await;
    match exists_user {
        Ok(user) => {
            let access_token = token_app_service
                .generate_access_token(&user)
                .map_err(|e| {
                    let app_err = AppError::from(e);
                    app_err.handle_error("Failed to generate access token")
                })?;
            let api_credentials = ApiCredentials::new(&access_token);

            let refresh_token = token_app_service
                .generate_refresh_token(&user)
                .map_err(|e| {
                    let app_err = AppError::from(e);
                    app_err.handle_error("Failed to generate refresh token")
                })?;
            let url_encoded_refresh_token = urlencoding::encode(&refresh_token).into_owned();
            let updated_jar = cookie_service.set_refresh_token(jar, &url_encoded_refresh_token);

            Ok(ApiResponse::new(
                StatusCode::OK,
                Some(serde_json::to_string(&api_credentials).unwrap()),
                Some(updated_jar),
            ))
        }
        Err(e) => match e.downcast_ref::<RepositoryError>() {
            Some(RepositoryError::NotFound) => {
                let new_user =
                    NewUser::default().new(&id_token_claims.email(), &id_token_claims.sub());
                let user = user_app_service.create(new_user).await.map_err(|e| {
                    let app_err = AppError::from(e);
                    app_err.handle_error("Failed to create user")
                })?;

                let access_token = token_app_service
                    .generate_access_token(&user)
                    .map_err(|e| {
                        let app_err = AppError::from(e);
                        app_err.handle_error("Failed to generate access token")
                    })?;

                let api_credentials = ApiCredentials::new(&access_token);

                let refresh_token =
                    token_app_service
                        .generate_refresh_token(&user)
                        .map_err(|e| {
                            let app_err = AppError::from(e);
                            app_err.handle_error("Failed to generate refresh token")
                        })?;
                let url_encoded_refresh_token = urlencoding::encode(&refresh_token).into_owned();
                let updated_jar = cookie_service.set_refresh_token(jar, &url_encoded_refresh_token);

                Ok(ApiResponse::new(
                    StatusCode::OK,
                    Some(serde_json::to_string(&api_credentials).unwrap()),
                    Some(updated_jar),
                ))
            }
            _ => {
                let app_err = AppError::from(e);
                Err(app_err.handle_error("Failed to verify id token"))
            }
        },
    }
}

#[tracing::instrument(
    name = "refresh_access_token",
    skip(token_app_service, user_app_service, cookie_service, jar)
)]
pub async fn refresh_access_token<S, T>(
    Extension(token_app_service): Extension<Arc<TokenAppService<S>>>,
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Extension(cookie_service): Extension<Arc<CookieService>>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    S: ITokenRepository,
    T: IUserRepository,
{
    let refresh_token = cookie_service.get_refresh_token(&jar).map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to get refresh token")
    })?;
    let refresh_token = RefreshTokenString(refresh_token);

    let token_data = token_app_service
        .verify_refresh_token(refresh_token)
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify refresh token")
        })?;

    let exists_user = user_app_service.find(token_data.claims.sub()).await;
    match exists_user {
        Ok(user) => {
            let access_token = token_app_service
                .generate_access_token(&user)
                .map_err(|e| {
                    let app_err = AppError::from(e);
                    app_err.handle_error("Failed to generate access token")
                })?;
            let api_credentials = ApiCredentials::new(&access_token);

            Ok(ApiResponse::new(
                StatusCode::OK,
                Some(serde_json::to_string(&api_credentials).unwrap()),
                None,
            ))
        }
        Err(e) => {
            let app_err = AppError::from(e);
            Err(app_err.handle_error("Failed to find user"))
        }
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
