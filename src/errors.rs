use crate::models::ErrorResponse;
use axum::{Json, http::StatusCode, response::IntoResponse};
use sqlx::Error;

pub enum AppError {
    NotFound,
    InternalError,
    BadRequest,
    HashingError,
    Unauthorized,
    Forbidden,
    Conflict,
    Validation(String),
    TooManyRequests,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            AppError::BadRequest => (StatusCode::BAD_REQUEST, "Bad request".to_string()),
            AppError::HashingError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            AppError::Conflict => (StatusCode::CONFLICT, "Conflict".to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, "Too many requests".to_string()),
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

impl From<Error> for AppError {
    fn from(value: Error) -> Self {
        match value {
            sqlx::Error::Database(db_err) => match db_err.code() {
                Some(c) if c == "23505" => AppError::Conflict,
                _ => {
                    tracing::error!(error = ?db_err, "database error");
                    AppError::InternalError
                }
            },
            sqlx::Error::RowNotFound => AppError::NotFound,
            other => {
                tracing::error!(error = ?other, "unexpected sqlx error");
                AppError::InternalError
            }
        }
    }
}
