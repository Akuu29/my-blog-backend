use crate::model::{
    api_response::ApiResponse,
    error_response::{ErrorCode, ErrorResponse},
};
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
    type Rejection = ApiResponse<String>;

    #[tracing::instrument(name = "validated_query_param")]
    async fn from_request(req: Request, _: &B) -> Result<Self, Self::Rejection> {
        let query_string = req.uri().query().unwrap_or_default();
        let qs_config = serde_qs::Config::new(5, false);
        let val = qs_config.deserialize_str::<T>(query_string).map_err(|e| {
            let res_body = ErrorResponse::new(
                ErrorCode::InvalidInput,
                format!("Invalid query param: {}", e),
            );
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

        Ok(ValidatedQueryParam(val))
    }
}
