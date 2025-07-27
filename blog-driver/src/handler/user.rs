use crate::{
    model::{api_response::ApiResponse, auth_token::AuthToken, validated_json::ValidatedJson},
    service::cookie_service::CookieService,
    utils::{app_error::AppError, error_handler::ErrorHandler},
};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::cookie::PrivateCookieJar;
use blog_adapter::utils::repository_error::RepositoryError;
use blog_app::service::{
    tokens::token_app_service::TokenAppService, users::user_app_service::UserAppService,
};
use blog_domain::model::{
    tokens::{
        i_token_repository::ITokenRepository, token::ApiCredentials, token_string::IdTokenString,
    },
    users::{
        i_user_repository::IUserRepository,
        user::{NewUser, UpdateUser},
    },
};
use sqlx::types::Uuid;
use std::{sync::Arc, time};

#[tracing::instrument(
    name = "sign_up",
    skip(token_app_service, user_app_service, cookie_service, token, jar)
)]
pub async fn sign_up<T, U>(
    Extension(token_app_service): Extension<Arc<TokenAppService<T>>>,
    Extension(user_app_service): Extension<Arc<UserAppService<U>>>,
    Extension(cookie_service): Extension<Arc<CookieService>>,
    AuthToken(token): AuthToken<IdTokenString>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: ITokenRepository,
    U: IUserRepository,
{
    let start_time = time::Instant::now();

    let id_token_data = token_app_service
        .verify_id_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to sign up")
        })?;

    let id_token_claims = id_token_data.claims;

    let provider_name = id_token_claims.provider_name().map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to sign up")
    })?;
    let exists_user = user_app_service
        .find_by_user_identity(&provider_name, &id_token_claims.sub())
        .await;

    let response = match exists_user {
        Ok(_) => {
            let app_err = AppError::Unexpected("User already exists".to_string());
            Err(app_err.handle_error("Failed to sign up"))
        }
        Err(e) => match e.downcast_ref::<RepositoryError>() {
            Some(RepositoryError::NotFound) => {
                let new_user = NewUser::new(
                    &provider_name,
                    &id_token_claims.sub(),
                    &id_token_claims.email(),
                    id_token_claims.email_verified(),
                );
                let user = user_app_service.create(new_user).await.map_err(|e| {
                    let app_err = AppError::from(e);
                    app_err.handle_error("Failed to sign up")
                })?;

                let access_token = token_app_service
                    .generate_access_token(&user)
                    .map_err(|e| {
                        let app_err = AppError::from(e);
                        app_err.handle_error("Failed to sign up")
                    })?;

                let api_credentials = ApiCredentials::new(&access_token);

                let refresh_token =
                    token_app_service
                        .generate_refresh_token(&user)
                        .map_err(|e| {
                            let app_err = AppError::from(e);
                            app_err.handle_error("Failed to sign up")
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
                Err(app_err.handle_error("Failed to sign up"))
            }
        },
    };

    let min_duration = time::Duration::from_millis(1000);
    let elapsed = start_time.elapsed();
    if elapsed > min_duration {
        tokio::time::sleep(min_duration - elapsed).await;
    }

    response
}

#[tracing::instrument(
    name = "sign_in",
    skip(token_app_service, user_app_service, cookie_service, token, jar)
)]
pub async fn sign_in<T, U>(
    Extension(token_app_service): Extension<Arc<TokenAppService<T>>>,
    Extension(user_app_service): Extension<Arc<UserAppService<U>>>,
    Extension(cookie_service): Extension<Arc<CookieService>>,
    AuthToken(token): AuthToken<IdTokenString>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: ITokenRepository,
    U: IUserRepository,
{
    let start_time = time::Instant::now();

    let id_token_data = token_app_service
        .verify_id_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to sign in")
        })?;

    let id_token_claims = id_token_data.claims;

    let provider_name = id_token_claims.provider_name().map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to sign in")
    })?;

    let exists_user = user_app_service
        .find_by_user_identity(&provider_name, &id_token_claims.sub())
        .await;

    let response = match exists_user {
        Ok(user) => {
            let access_token = token_app_service
                .generate_access_token(&user)
                .map_err(|e| {
                    let app_err = AppError::from(e);
                    app_err.handle_error("Failed to sign in")
                })?;
            let api_credentials = ApiCredentials::new(&access_token);

            let refresh_token = token_app_service
                .generate_refresh_token(&user)
                .map_err(|e| {
                    let app_err = AppError::from(e);
                    app_err.handle_error("Failed to sign in")
                })?;
            let url_encoded_refresh_token = urlencoding::encode(&refresh_token).into_owned();
            let updated_jar = cookie_service.set_refresh_token(jar, &url_encoded_refresh_token);

            Ok(ApiResponse::new(
                StatusCode::OK,
                Some(serde_json::to_string(&api_credentials).unwrap()),
                Some(updated_jar),
            ))
        }
        Err(e) => {
            let app_err = AppError::from(e);
            Err(app_err.handle_error("Failed to sign in"))
        }
    };

    let min_duration = time::Duration::from_millis(1000);
    let elapsed = start_time.elapsed();
    if elapsed > min_duration {
        tokio::time::sleep(min_duration - elapsed).await;
    }

    response
}

#[tracing::instrument(name = "find_user", skip(user_app_service))]
pub async fn find<T>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IUserRepository,
{
    let user = user_app_service.find(user_id).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to find user")
    })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&user).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "update_user", skip(user_app_service))]
pub async fn update<T>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Path(user_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateUser>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IUserRepository,
{
    let user = user_app_service
        .update(user_id, payload)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to update user")
        })?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&user).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "delete_user", skip(user_app_service))]
pub async fn delete<T>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IUserRepository,
{
    user_app_service.delete(user_id).await.map_err(|e| {
        let app_err = AppError::from(e);
        app_err.handle_error("Failed to delete user")
    })?;

    Ok(ApiResponse::<()>::new(StatusCode::OK, None, None))
}
