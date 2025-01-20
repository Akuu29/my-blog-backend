use crate::model::{api_response::ApiResponse, auth_token::AuthToken};
use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar, SameSite};
use blog_adapter::db::utils::RepositoryError;
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
use cookie::time::Duration;
use std::sync::Arc;

pub async fn verify_id_token<S: ITokenRepository, T: IUserRepository>(
    Extension(token_app_service): Extension<Arc<TokenAppService<S>>>,
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    AuthToken(token): AuthToken<IdTokenString>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, StatusCode> {
    let id_token_data = token_app_service
        .verify_id_token(token)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;
    let id_token_claims = id_token_data.claims;

    let exists_user = user_app_service
        .find_by_idp_sub(&id_token_claims.sub())
        .await;
    match exists_user {
        Ok(user) => {
            let access_token = token_app_service
                .generate_access_token(&user)
                .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;
            let api_credentials = ApiCredentials::new(&access_token);

            let refresh_token = token_app_service
                .generate_refresh_token(&user)
                .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;
            let cookie = Cookie::build(("refresh_token", refresh_token))
                .http_only(true)
                .max_age(Duration::days(30))
                .path("/")
                .same_site(SameSite::None)
                .secure(true);
            let updated_jar = jar.add(cookie);

            let response = ApiResponse::new(StatusCode::OK, api_credentials, Some(updated_jar));

            Ok(response)
        }
        Err(e) => match e.downcast_ref::<RepositoryError>() {
            Some(RepositoryError::NotFound) => {
                let new_user =
                    NewUser::default().new(&id_token_claims.email(), &id_token_claims.sub());
                let user = user_app_service
                    .create(new_user)
                    .await
                    .or(Err(StatusCode::BAD_REQUEST))?;

                let access_token = token_app_service
                    .generate_access_token(&user)
                    .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

                let api_credentials = ApiCredentials::new(&access_token);

                let refresh_token = token_app_service
                    .generate_refresh_token(&user)
                    .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;
                let cookie = Cookie::build(("refresh_token", refresh_token))
                    .http_only(true)
                    .max_age(Duration::days(30))
                    .path("/")
                    .same_site(SameSite::None)
                    .secure(true);
                let updated_jar = jar.add(cookie);

                let response = ApiResponse::new(StatusCode::OK, api_credentials, Some(updated_jar));

                Ok(response)
            }
            _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

pub async fn refresh_access_token<S: ITokenRepository, T: IUserRepository>(
    Extension(token_app_service): Extension<Arc<TokenAppService<S>>>,
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, StatusCode> {
    let refresh_token = match jar.get("refresh_token") {
        Some(refresh_token) => RefreshTokenString(refresh_token.to_string()),
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let token_data = token_app_service
        .verify_refresh_token(refresh_token)
        .or(Err(StatusCode::BAD_REQUEST))?;

    let exists_user = user_app_service.find(token_data.claims.sub()).await;
    match exists_user {
        Ok(user) => {
            let access_token = token_app_service
                .generate_access_token(&user)
                .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;
            let api_credentials = ApiCredentials::new(&access_token);

            Ok((StatusCode::OK, Json(api_credentials)))
        }
        Err(e) => match e.downcast_ref::<RepositoryError>() {
            Some(RepositoryError::NotFound) => {
                return Err(StatusCode::BAD_REQUEST);
            }
            _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}
