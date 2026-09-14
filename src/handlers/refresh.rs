use crate::auth::{create_jwt, create_refresh_exp, create_refresh_token, hash_token};
use crate::errors::AppError;
use crate::extractors::ValidatedJson;
use crate::models::{AppState, RefreshTokenRequest, UpdatedTokensResponse, UserRole};
use axum::extract::{Json, State};

pub async fn update_access_token(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RefreshTokenRequest>,
) -> Result<Json<UpdatedTokensResponse>, AppError> {
    let mut tx = state.pool.begin().await?;

    let update_row = sqlx::query!(
        "UPDATE refresh_tokens rt
        SET used = true
        FROM users u
        WHERE rt.token = $1
        AND rt.used = false
        AND rt.expires_at > NOW()
        AND u.id = rt.user_id
        RETURNING rt.user_id, u.role AS \"role: UserRole\"",
        hash_token(&payload.token)
    )
    .fetch_optional(&mut *tx)
    .await?;

    match update_row {
        Some(row) => {
            let new_refresh_token = create_refresh_token();
            let new_refresh_exp = create_refresh_exp();

            sqlx::query!(
                "INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)",
                row.user_id,
                hash_token(&new_refresh_token),
                new_refresh_exp
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;

            let new_access_token = create_jwt(row.user_id, row.role, &state.jwt_secret)?;

            Ok(Json(UpdatedTokensResponse {
                refresh_token: new_refresh_token,
                access_token: new_access_token,
            }))
        }
        None => {
            drop(tx);
            let existing = sqlx::query!(
                "SELECT user_id, used FROM refresh_tokens WHERE token = $1",
                hash_token(&payload.token)
            )
            .fetch_optional(&state.pool)
            .await?;

            match existing {
                Some(row) => {
                    if row.used {
                        tracing::warn!(
                            user_id = row.user_id,
                            "refresh token reuse detected — revoking all sessions for user"
                        );
                        sqlx::query!("DELETE FROM refresh_tokens WHERE user_id = $1", row.user_id)
                            .execute(&state.pool)
                            .await?;

                        return Err(AppError::Unauthorized);
                    }

                    Err(AppError::Unauthorized)
                }
                None => Err(AppError::Unauthorized),
            }
        }
    }
}
