use axum::extract::State;
use axum::http::StatusCode;

use crate::auth::hash_token;
use crate::errors::AppError;
use crate::extractors::ValidatedJson;
use crate::models::AuthenticatedUser;
use crate::models::{AppState, RefreshTokenRequest};

pub async fn logout_user(State(state): State<AppState>, user: AuthenticatedUser, ValidatedJson(payload): ValidatedJson<RefreshTokenRequest>) -> Result<StatusCode, AppError> {
    let result = sqlx::query!(
        "DELETE FROM refresh_tokens WHERE token = $1 AND user_id = $2", hash_token(&payload.token), user.id
    )
    .execute(&state.pool)
    .await?;

    match result.rows_affected() {
        0 => Err(AppError::NotFound),
        _ => Ok(StatusCode::OK),
    }
}
