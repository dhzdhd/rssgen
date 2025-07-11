use anyhow::anyhow;
use axum::response::IntoResponse;

pub struct AppError(pub anyhow::Error);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl AppError {
    pub fn from_str<E>(err: E) -> Self
    where
        E: ToString,
    {
        AppError(anyhow!(err.to_string()))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = axum::http::StatusCode::INTERNAL_SERVER_ERROR;
        let body = format!("Internal Server Error: {}", self.0);
        (status, body).into_response()
    }
}
