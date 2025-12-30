//! Logging middleware for automatic request/response observability
//!
//! This middleware provides:
//! - Automatic request/response logging
//! - Request ID generation and propagation
//! - Processing time measurement
//! - Error detection and structured logging
//! - Distributed tracing context via spans

use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use uuid::Uuid;

/// Logging middleware for all HTTP requests
///
/// This middleware automatically logs:
/// - Request start (method, URI, request ID)
/// - Request completion (status, duration, errors)
/// - Contextual information via tracing spans
///
/// # Example
///
/// ```ignore
/// use axum::Router;
/// use tower::ServiceBuilder;
///
/// let app = Router::new()
///     .route("/", get(handler))
///     .layer(ServiceBuilder::new()
///         .layer(axum::middleware::from_fn(logging_middleware)));
/// ```
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    // Generate unique request ID for tracking
    let request_id = Uuid::new_v4();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();

    // Create a tracing span with request context
    // All logs within this request will automatically include these fields
    let span = tracing::info_span!(
        "http_request",
        request.id = %request_id,
        request.method = %method,
        request.uri = %uri,
        request.version = ?version,
        response.status = tracing::field::Empty,
        duration_ms = tracing::field::Empty,
    );

    // Enter the span so all subsequent logs inherit the context
    let _enter = span.enter();

    // Log request start
    tracing::info!("Request started");

    // Start timing
    let start = Instant::now();

    // Execute the request handler
    let response = next.run(request).await;

    // Calculate duration
    let duration = start.elapsed();
    let status = response.status();

    // Record response information in the span
    span.record("response.status", status.as_u16());
    span.record("duration_ms", duration.as_millis() as u64);

    // Log based on status code
    match status.as_u16() {
        // Success (2xx)
        200..=299 => {
            tracing::info!(
                response.status = %status,
                duration_ms = duration.as_millis(),
                "Request completed successfully"
            );
        }
        // Redirection (3xx)
        300..=399 => {
            tracing::info!(
                response.status = %status,
                duration_ms = duration.as_millis(),
                "Request redirected"
            );
        }
        // Client error (4xx)
        400..=499 => {
            tracing::warn!(
                response.status = %status,
                duration_ms = duration.as_millis(),
                error.category = "ClientError",
                "Client error"
            );
        }
        // Server error (5xx)
        500..=599 => {
            tracing::error!(
                response.status = %status,
                duration_ms = duration.as_millis(),
                error.category = "ServerError",
                "Server error"
            );
        }
        _ => {
            tracing::warn!(
                response.status = %status,
                duration_ms = duration.as_millis(),
                "Unusual status code"
            );
        }
    }

    response
}

/// Metrics middleware for recording request metrics
///
/// This can be used alongside logging_middleware for additional metrics collection
/// (e.g., Prometheus, StatsD, etc.)
pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().path().to_string();

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    let status = response.status();

    // TODO: Record metrics to your metrics backend
    // For example:
    // metrics::histogram!("http_request_duration_ms", duration.as_millis() as f64,
    //     "method" => method.as_str(),
    //     "path" => uri.as_str(),
    //     "status" => status.as_u16().to_string()
    // );

    tracing::debug!(
        method = %method,
        path = %uri,
        status = %status,
        duration_ms = duration.as_millis(),
        "Request metrics recorded"
    );

    response
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use axum::{
//         Router,
//         body::Body,
//         http::{Request, StatusCode},
//         middleware::from_fn,
//         response::IntoResponse,
//         routing::get,
//     };
//     use tower::ServiceExt;

//     async fn test_handler() -> impl IntoResponse {
//         (StatusCode::OK, "test response")
//     }

//     async fn error_handler() -> impl IntoResponse {
//         (StatusCode::INTERNAL_SERVER_ERROR, "error response")
//     }

//     #[tokio::test]
//     async fn test_logging_middleware_success() {
//         let app = Router::new()
//             .route("/test", get(test_handler))
//             .layer(from_fn(logging_middleware));

//         let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

//         let response = app.oneshot(request).await.unwrap();
//         assert_eq!(response.status(), StatusCode::OK);
//     }

//     #[tokio::test]
//     async fn test_logging_middleware_error() {
//         let app = Router::new()
//             .route("/error", get(error_handler))
//             .layer(from_fn(logging_middleware));

//         let request = Request::builder()
//             .uri("/error")
//             .body(Body::empty())
//             .unwrap();

//         let response = app.oneshot(request).await.unwrap();
//         assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
//     }
// }
