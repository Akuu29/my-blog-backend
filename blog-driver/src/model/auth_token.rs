use crate::{
    error::{ErrorCode, ErrorResponse},
    model::api_response::ApiResponse,
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::headers::{Authorization, HeaderMapExt, authorization::Bearer};
use blog_domain::model::tokens::token_string::TokenString;

pub struct AuthToken<T>(pub T);

#[async_trait]
impl<T, B> FromRequestParts<B> for AuthToken<T>
where
    B: Send + 'static,
    T: TokenString + From<String>,
{
    type Rejection = ApiResponse<String>;

    #[tracing::instrument(name = "auth_token", skip(_state))]
    async fn from_request_parts(parts: &mut Parts, _state: &B) -> Result<Self, Self::Rejection> {
        let token = parts.headers.typed_get::<Authorization<Bearer>>();

        match token {
            Some(Authorization(bearer)) => {
                let token = bearer.token().to_string();
                let token_instance: T = token.into();
                Ok(AuthToken(token_instance))
            }
            _ => {
                let err_msg = "No token provided";
                tracing::error!(error.kind="Authentication", error.message=%err_msg);

                let err_res_body = ErrorResponse::new(ErrorCode::Unauthorized, err_msg);
                Err(ApiResponse::new(
                    StatusCode::UNAUTHORIZED,
                    Some(serde_json::to_string(&err_res_body).unwrap()),
                    None,
                ))
            }
        }
    }
}
