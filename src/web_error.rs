use anyhow::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub struct WebError {
    status: StatusCode,
    error: Error,
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
