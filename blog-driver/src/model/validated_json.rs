use crate::model::{
    api_response::ApiResponse,
    error_response::{ErrorCode, ErrorResponse},
};
use axum::{
    async_trait,
    extract::{FromRequest, Json, Request},
    http::StatusCode,
};
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    B: Send + Sync,
{
    type Rejection = ApiResponse<String>;

    #[tracing::instrument(name = "validated_json", skip(state))]
    async fn from_request(req: Request, state: &B) -> Result<Self, Self::Rejection> {
        let Json(val) = Json::<T>::from_request(req, state).await.map_err(|e| {
            let res_body =
                ErrorResponse::new(ErrorCode::InvalidInput, format!("Json parse error: {}", e));
            ApiResponse::new(
                StatusCode::BAD_REQUEST,
                Some(serde_json::to_string(&res_body).unwrap()),
                None,
            )
        })?;

        val.validate().map_err(|e| {
            let res_body = ErrorResponse::new(
                ErrorCode::ValidationError,
                format!("Validation error: {}", e).replace("\n", ", "),
            );
            ApiResponse::new(
                StatusCode::BAD_REQUEST,
                Some(serde_json::to_string(&res_body).unwrap()),
                None,
            )
        })?;

        Ok(ValidatedJson(val))
    }
}
