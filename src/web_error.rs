use anyhow::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub struct WebError {
    status: StatusCode,
    error: Error,
}

impl WebError {
    pub fn new(status: StatusCode, error: Error) -> Self {
        Self { status, error }
    }

    pub fn bad_request<E: Into<Error>>(error: E) -> Self {
        WebError::new(StatusCode::BAD_REQUEST, error.into())
    }
}

impl<E: Into<Error>> From<E> for WebError {
    fn from(value: E) -> Self {
        WebError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: value.into(),
        }
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        (self.status, self.error.to_string()).into_response()
    }
}
