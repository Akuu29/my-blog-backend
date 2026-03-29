use crate::{
    error::AppError,
    model::{
        api_response::ApiResponse, auth_token::AuthToken, paged_body::PagedBody,
        paged_filter_query_param::PagedFilterQueryParam, validated_json::ValidatedJson,
        validated_query_param::ValidatedQueryParam,
    },
    service::cookie_service::CookieService,
};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::cookie::PrivateCookieJar;
use blog_app::service::{
    error::UsecaseError, tokens::token_app_service::TokenAppService,
    users::user_app_service::UserAppService,
};
use blog_domain::{
    config::EmailConfig,
    model::{
        tokens::{
            i_token_repository::ITokenRepository,
            token::ApiCredentials,
            token_string::{AccessTokenString, IdTokenString},
        },
        users::{
            email_cipher::EmailCipher,
            email_hash::EmailHash,
            i_user_repository::{IUserRepository, UserFilter},
            user::{NewUser, UpdateUser},
        },
    },
    service::error::DomainServiceError,
};
use sqlx::types::Uuid;
use std::{sync::Arc, time};

#[tracing::instrument(
    name = "sign_up",
    skip(
        token_app_service,
        user_app_service,
        cookie_service,
        email_config,
        token,
        jar
    )
)]
pub async fn sign_up<T, U>(
    Extension(token_app_service): Extension<Arc<TokenAppService<T>>>,
    Extension(user_app_service): Extension<Arc<UserAppService<U>>>,
    Extension(cookie_service): Extension<Arc<CookieService>>,
    Extension(email_config): Extension<Arc<EmailConfig>>,
    AuthToken(token): AuthToken<IdTokenString>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, AppError>
where
    T: ITokenRepository,
    U: IUserRepository,
{
    let start_time = time::Instant::now();

    let id_token_data = token_app_service
        .verify_id_token(token)
        .await
        .map_err(AppError::from)?;

    let id_token_claims = id_token_data.claims;

    let provider_name = id_token_claims
        .provider_name()
        .map_err(AppError::from)?;
    let exists_user = user_app_service
        .find_by_user_identity(&provider_name, &id_token_claims.sub())
        .await;

    let response = match exists_user {
        // TODO The driver layer is directly generating use case errors. This represents a leakage of responsibilities from the app layer.
        Ok(_) => Err(AppError::from(UsecaseError::DomainError(
            DomainServiceError::Conflict,
        ))),
        Err(e) => match &e {
            UsecaseError::DomainError(DomainServiceError::NotFound) => {
                let email = id_token_claims.email();
                let email_cipher =
                    EmailCipher::from_plaintext(&email, &email_config.encryption_key)
                        .map_err(AppError::from)?;
                let email_hash = EmailHash::from_plaintext(&email, &email_config.pepper);

                let new_user = NewUser::new(
                    &provider_name,
                    &id_token_claims.sub(),
                    email_cipher,
                    email_hash,
                    id_token_claims.email_verified(),
                );
                let user = user_app_service
                    .create(new_user)
                    .await
                    .map_err(AppError::from)?;

                let access_token = token_app_service
                    .generate_access_token(&user)
                    .map_err(AppError::from)?;

                let refresh_token = token_app_service
                    .generate_refresh_token(&user)
                    .map_err(AppError::from)?;
                let url_encoded_refresh_token = urlencoding::encode(&refresh_token).into_owned();
                let updated_jar = cookie_service.set_refresh_token(jar, &url_encoded_refresh_token);

                let api_credentials = ApiCredentials::new(&access_token, user);

                Ok(ApiResponse::new(
                    StatusCode::OK,
                    Some(serde_json::to_string(&api_credentials).unwrap()),
                    Some(updated_jar),
                ))
            }
            _ => Err(AppError::from(e)),
        },
    };

    let min_duration = time::Duration::from_millis(1000);
    let elapsed = start_time.elapsed();
    if elapsed < min_duration {
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
) -> Result<impl IntoResponse, AppError>
where
    T: ITokenRepository,
    U: IUserRepository,
{
    let start_time = time::Instant::now();

    let id_token_data = token_app_service
        .verify_id_token(token)
        .await
        .map_err(AppError::from)?;

    let id_token_claims = id_token_data.claims;

    let provider_name = id_token_claims
        .provider_name()
        .map_err(AppError::from)?;

    let exists_user = user_app_service
        .find_by_user_identity(&provider_name, &id_token_claims.sub())
        .await;

    let response = match exists_user {
        Ok(user) => {
            let access_token = token_app_service
                .generate_access_token(&user)
                .map_err(AppError::from)?;

            let refresh_token = token_app_service
                .generate_refresh_token(&user)
                .map_err(AppError::from)?;
            let url_encoded_refresh_token = urlencoding::encode(&refresh_token).into_owned();
            let updated_jar = cookie_service.set_refresh_token(jar, &url_encoded_refresh_token);

            let api_credentials = ApiCredentials::new(&access_token, user);

            Ok(ApiResponse::new(
                StatusCode::OK,
                Some(serde_json::to_string(&api_credentials).unwrap()),
                Some(updated_jar),
            ))
        }
        Err(e) => Err(AppError::from(e)),
    };

    let min_duration = time::Duration::from_millis(1000);
    let elapsed = start_time.elapsed();
    if elapsed < min_duration {
        tokio::time::sleep(min_duration - elapsed).await;
    }

    response
}

pub async fn all<T>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    ValidatedQueryParam(param): ValidatedQueryParam<PagedFilterQueryParam<UserFilter>>,
) -> Result<impl IntoResponse, AppError>
where
    T: IUserRepository,
{
    let mut pagination = param.pagination;
    // To check if there is a next page
    pagination.per_page += 1;

    let (mut users, total) = user_app_service
        .all(param.filter, pagination.clone())
        .await
        .map_err(AppError::from)?;

    let has_next = users.len() == pagination.per_page as usize;
    if has_next {
        users.pop();
    }

    let next_cursor = users.last().map(|user| user.id).or(None);
    let paged_body = PagedBody::new(users, next_cursor, has_next, total.value());

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&paged_body).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "find_user", skip(user_app_service))]
pub async fn find<T>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError>
where
    T: IUserRepository,
{
    let user = user_app_service
        .find(user_id)
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&user).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "update_user", skip(user_app_service, token_app_service, token))]
pub async fn update<T, U>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(user_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateUser>,
) -> Result<impl IntoResponse, AppError>
where
    T: IUserRepository,
    U: ITokenRepository,
{
    let token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(AppError::from)?;

    let user = user_app_service
        .update_with_auth(user_id, token_data.claims.sub(), payload)
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&user).unwrap()),
        None,
    ))
}

#[tracing::instrument(name = "delete_user", skip(user_app_service, token_app_service, token))]
pub async fn delete<T, U>(
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError>
where
    T: IUserRepository,
    U: ITokenRepository,
{
    let token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(AppError::from)?;

    // Use the new delete_with_auth method which includes authorization check
    user_app_service
        .delete_with_auth(user_id, token_data.claims.sub())
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::<()>::new(StatusCode::OK, None, None))
}
