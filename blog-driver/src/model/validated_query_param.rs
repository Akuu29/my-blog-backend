use crate::{
    model::{
        api_response::ApiResponse,
        error_message::{ErrorMessage, ErrorMessageKind},
    },
    utils::error_log_kind::ErrorLogKind,
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
        let val = serde_qs::from_str::<T>(query_string).map_err(|e| {
            tracing::error!(error.kind=%ErrorLogKind::Validation, error.message=%e.to_string());

            let err_msg = ErrorMessage::new(
                ErrorMessageKind::BadRequest,
                format!("Invalid query param: {}", e),
            );
            ApiResponse::new(
                StatusCode::BAD_REQUEST,
                Some(serde_json::to_string(&err_msg).unwrap()),
                None,
            )
        })?;

        val.validate().map_err(|e| {
            tracing::error!(error.kind=%ErrorLogKind::Validation, error.message=%e.to_string());

            let err_msg = ErrorMessage::new(
                ErrorMessageKind::Validation,
                format!("Validation error: {}", e).replace("\n", ", "),
            );
            ApiResponse::new(
                StatusCode::BAD_REQUEST,
                Some(serde_json::to_string(&err_msg).unwrap()),
                None,
            )
        })?;

        Ok(ValidatedQueryParam(val))
    }
}
