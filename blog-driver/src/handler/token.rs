use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use blog_adapter::repository::RepositoryError;
use blog_app::service::{
    tokens::token_app_service::TokenAppService, users::user_app_service::UserAppService,
};
use blog_domain::model::{
    tokens::{i_token_repository::ITokenRepository, token::ApiCredentials},
    users::{i_user_repository::IUserRepository, user::NewUser},
};
use std::sync::Arc;

pub async fn verify_id_token<S: ITokenRepository, T: IUserRepository>(
    Extension(token_app_service): Extension<Arc<TokenAppService<S>>>,
    Extension(user_app_service): Extension<Arc<UserAppService<T>>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, StatusCode> {
    let id_token = bearer.token().to_string();
    let id_token_data = token_app_service
        .verify_id_token(&id_token)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;
    let id_token_claims = id_token_data.claims;

    let exists_user = user_app_service.find_by_idp_sub(&id_token_claims.sub).await;
    match exists_user {
        Ok(user) => {
            let access_token = token_app_service
                .generate_access_token(user.id)
                .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

            let refresh_token = token_app_service
                .generate_refresh_token(user.id)
                .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

            let api_credentials = ApiCredentials::new(&access_token, &refresh_token);

            Ok((StatusCode::OK, Json(api_credentials)))
        }
        Err(e) => match e.downcast_ref::<RepositoryError>() {
            Some(RepositoryError::NotFound) => {
                let new_user = NewUser::default().new(&id_token_claims.email, &id_token_claims.sub);
                let new_user = user_app_service
                    .create(new_user)
                    .await
                    .or(Err(StatusCode::BAD_REQUEST))?;

                let access_token = token_app_service
                    .generate_access_token(new_user.id)
                    .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

                let refresh_token = token_app_service
                    .generate_refresh_token(new_user.id)
                    .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

                let api_credentials = ApiCredentials::new(&access_token, &refresh_token);

                return Ok((StatusCode::OK, Json(api_credentials)));
            }
            _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}
