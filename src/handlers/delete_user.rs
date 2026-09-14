use axum::{extract::{Path, State}, http::StatusCode};

use crate::errors::AppError;
use crate::models::{AppState, AuthenticatedAdmin};

pub async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
    _admin: AuthenticatedAdmin,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&state.pool)
        .await?;

    match result.rows_affected() {
        0 => Err(AppError::NotFound),
        _ => Ok(StatusCode::NO_CONTENT),
    }
}
