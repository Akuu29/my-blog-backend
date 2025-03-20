use axum::{
    async_trait,
    extract::{FromRequest, Query, Request},
    http::StatusCode,
};
use serde::de::DeserializeOwned;
use validator::Validate;
#[derive(Debug)]
pub struct ValidatedQueryParam<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedQueryParam<T>
where
    T: DeserializeOwned + Validate,
    B: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &B) -> Result<Self, Self::Rejection> {
        let Query(val) = Query::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid query param: {}", rejection),
                )
            })?;

        val.validate().map_err(|rejection| {
            (
                StatusCode::BAD_REQUEST,
                format!("Validation error: {}", rejection),
            )
        })?;

        Ok(ValidatedQueryParam(val))
    }
}
