use crate::{
    model::{
        api_response::ApiResponse,
        error_message::{ErrorMessage, ErrorMessageKind},
    },
    utils::error_log_kind::ErrorLogKind,
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::headers::{authorization::Bearer, Authorization, HeaderMapExt};
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
                let err_msg_msg = "No token provided";
                tracing::error!(error.kind=%ErrorLogKind::Authentication, error.message=%err_msg_msg);

                let err_msg =
                    ErrorMessage::new(ErrorMessageKind::Unauthorized, err_msg_msg.to_string());
                Err(ApiResponse::new(
                    StatusCode::BAD_REQUEST,
                    Some(serde_json::to_string(&err_msg).unwrap()),
                    None,
                ))
            }
        }
    }
}
