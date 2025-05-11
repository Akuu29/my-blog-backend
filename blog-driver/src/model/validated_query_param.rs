use axum::{
    async_trait,
    extract::{FromRequest, Request},
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

    async fn from_request(req: Request, _: &B) -> Result<Self, Self::Rejection> {
        let query_string = req.uri().query().unwrap_or_default();
        let val = serde_qs::from_str::<T>(query_string).map_err(|rejection| {
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
