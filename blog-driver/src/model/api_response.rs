use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::PrivateCookieJar;
use serde::Serialize;

#[derive(Debug)]
pub struct ApiResponse<T> {
    status: StatusCode,
    body: T,
    cookies: Option<PrivateCookieJar>,
}

impl<T> ApiResponse<T> {
    pub fn new(status: StatusCode, body: T, cookies: Option<PrivateCookieJar>) -> Self {
        Self {
            status,
            body,
            cookies,
        }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        if let Some(jar) = self.cookies {
            let mut response = jar.into_response();
            *response.status_mut() = self.status;
            response
                .headers_mut()
                .insert("Content-Type", "application/json".parse().unwrap());
            *response.body_mut() = Body::from(serde_json::to_string(&self.body).unwrap());

            response
        } else {
            let response = Response::builder()
                .status(self.status)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&self.body).unwrap()))
                .unwrap();

            response
        }
    }
}
