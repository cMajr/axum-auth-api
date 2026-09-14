use axum::{
    Json,
    extract::State
};

use crate::extractors::ValidatedJson;
use crate::errors::AppError;
use crate::models::{AppState, AuthenticatedUser, ProfileResponse, ProfileUpdateRequest};

pub async fn update_my_profile(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<ProfileUpdateRequest>
) -> Result<Json<ProfileResponse>, AppError> {
    if payload.username.is_none() && payload.bio.is_none() && payload.date_of_birth.is_none() {
        return Err(AppError::BadRequest)
    }

    let updated_user_data = sqlx::query_as!(
        ProfileResponse,
        "UPDATE users SET
        username = COALESCE($1, username),
        bio = COALESCE($2, bio),
        date_of_birth = COALESCE($3, date_of_birth),
        updated_at = NOW()
        WHERE id = $4
        RETURNING username, bio, date_of_birth, updated_at",
        payload.username,
        payload.bio,
        payload.date_of_birth,
        user.id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(updated_user_data))
}
