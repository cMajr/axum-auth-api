use axum::{Json, extract::{Path, State}};

use crate::errors::AppError;
use crate::models::{AppState, PublicSearchUserResponse};

pub async fn get_user_by_id(State(state): State<AppState>, Path(id): Path<i32>) -> Result<Json<PublicSearchUserResponse>, AppError> {
    // Public endpoint: no authentication required, returns only public profile fields.
    let user = sqlx::query_as!(
        PublicSearchUserResponse,
        "SELECT id, username, bio FROM users WHERE id = $1",
        id
    )
    .fetch_optional(&state.pool)
    .await?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(AppError::NotFound),
    }
}
