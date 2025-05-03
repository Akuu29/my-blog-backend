use axum::{
    body::Body,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::PrivateCookieJar;

#[derive(Debug)]
pub struct ApiResponse<T> {
    status: StatusCode,
    body: Option<T>,
    cookies: Option<PrivateCookieJar>,
    headers: HeaderMap,
}

impl<T> ApiResponse<T> {
    pub fn new(status: StatusCode, body: Option<T>, cookies: Option<PrivateCookieJar>) -> Self {
        Self {
            status,
            body,
            cookies,
            headers: HeaderMap::new(),
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
            if let Ok(header_value) = HeaderValue::from_str(value) {
                self.headers.insert(header_name, header_value);
            }
        }

        self
    }
}

impl<T: Into<Body>> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let mut response = if let Some(jar) = self.cookies {
            jar.into_response()
        } else {
            Response::default()
        };

        *response.status_mut() = self.status;
        *response.body_mut() = self.body.map(Into::into).unwrap_or_else(|| Body::empty());

        for (name, value) in self.headers.iter() {
            response.headers_mut().insert(name, value.clone());
        }

        response
    }
}
