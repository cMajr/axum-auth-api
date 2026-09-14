use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, rand_core::OsRng}};
use axum::{Json, extract::State, http::StatusCode};

use crate::errors::AppError;
use crate::extractors::ValidatedJson;
use crate::models::{AppState, UserRegisterRequest, UserRegisterResponse, UserRole};

pub async fn register_user(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UserRegisterRequest>,
) -> Result<(StatusCode, Json<UserRegisterResponse>), AppError> {
    let email = payload.email.to_lowercase();
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| {
            tracing::error!(error = ?e, "failed to hash password");
            AppError::HashingError
        })?
        .to_string();

    let registered_user = sqlx::query_as!(
        UserRegisterResponse,
        "INSERT INTO users (username, email, password, date_of_birth) VALUES ($1, $2, $3, $4) RETURNING id, username, email, date_of_birth, bio, created_at, role as \"role: UserRole\"",
        payload.username,
        email,
        hash,
        payload.date_of_birth,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(registered_user)))
}
