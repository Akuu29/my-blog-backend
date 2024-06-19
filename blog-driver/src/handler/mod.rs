pub mod article;
pub mod auth;
pub mod comment;
pub mod user;

use axum::{
    async_trait,
    extract::{FromRequest, Json, Request},
    http::StatusCode,
};
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug)]
pub struct ValidatedJson<T>(T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    B: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &B) -> Result<Self, Self::Rejection> {
        let Json(val) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                let message = format!("Json parse error: {}", rejection);

                (StatusCode::BAD_REQUEST, message)
            })?;

        val.validate().map_err(|rejection| {
            let message = format!("Validation error: {}", rejection).replace("\n", ", ");

            (StatusCode::BAD_REQUEST, message)
        })?;

        Ok(ValidatedJson(val))
    }
}
