use argon2::{Argon2, PasswordVerifier, password_hash::PasswordHash};
use axum::extract::ConnectInfo;
use axum::{Json, extract::State};
use std::net::SocketAddr;

use crate::auth::{create_jwt, create_refresh_exp, create_refresh_token, hash_token};
use crate::errors::AppError;
use crate::extractors::ValidatedJson;
use crate::models::{AppState, AuthTokensUserResponse, UserCredentials, UserLoginRequest, UserRole};
use crate::rate_limiters::{login_email_rate_limiter, login_ip_rate_limiter};

const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$XdSrjAp6xBCG+ZgwVh7pDw$v/Tqm+92TtpQGyBkKhvdLt7SPR+hVW7J5aJyk4n4ayE";

fn verify_password(payload_password: &[u8], hash_from_db: &str) -> Result<(), AppError> {
    let parsed_hash = PasswordHash::new(hash_from_db).map_err(|e| {
        tracing::error!(error = ?e, "failed to parse stored password hash");
        AppError::InternalError
    })?;

    Argon2::default()
        .verify_password(payload_password, &parsed_hash)
        .map_err(|_| AppError::Unauthorized)
}

pub async fn login_user(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ValidatedJson(payload): ValidatedJson<UserLoginRequest>,
) -> Result<Json<AuthTokensUserResponse>, AppError> {
    let email = payload.email.to_lowercase();
    let user_ip = addr.ip();

    login_email_rate_limiter(&state.login_email_limiter, &email)?;
    login_ip_rate_limiter(&state.login_ip_limiter, user_ip)?;

    let user_req = sqlx::query_as!(
        UserCredentials,
        "SELECT password, id, role as \"role: UserRole\" FROM users WHERE email = $1",
        email
    )
    .fetch_optional(&state.pool)
    .await?;

    match user_req {
        Some(u) => {
            verify_password(payload.password.as_bytes(), &u.password)?;

            let access_token = create_jwt(u.id, u.role, &state.jwt_secret)?;
            let refresh_token = create_refresh_token();
            let refresh_exp = create_refresh_exp();

            sqlx::query!(
                "INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)",
                u.id,
                hash_token(&refresh_token),
                refresh_exp
            )
            .execute(&state.pool)
            .await?;

            Ok(Json(AuthTokensUserResponse {
                access_token,
                refresh_token,
            }))
        }
        None => {
            let _ = verify_password(payload.password.as_bytes(), DUMMY_HASH);
            Err(AppError::Unauthorized)
        }
    }
}
