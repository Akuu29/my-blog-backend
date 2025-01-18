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
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &B) -> Result<Self, Self::Rejection> {
        let token = parts.headers.typed_get::<Authorization<Bearer>>();

        match token {
            Some(Authorization(bearer)) => {
                let token = bearer.token().to_string();
                let token_instance: T = token.into();
                Ok(AuthToken(token_instance))
            }
            _ => Err(StatusCode::BAD_REQUEST),
        }
    }
}
