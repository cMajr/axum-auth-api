use axum::{Json, extract::{Query, State}};

use validator::Validate;

use crate::errors::AppError;
use crate::models::{AppState, PublicSearchUserResponse, UsersByUsernameQuery};

pub async fn find_users_by_username(
    State(state): State<AppState>,
    Query(params): Query<UsersByUsernameQuery>,
) -> Result<Json<Vec<PublicSearchUserResponse>>, AppError> {
    params.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let escaped = params.username.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_");
    let pattern = format!("%{}%", escaped);
    let users = sqlx::query_as!(
        PublicSearchUserResponse,
        "SELECT id, username, bio FROM users WHERE username ILIKE $1 ESCAPE '\\' LIMIT 20",
        pattern
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(users))
}
