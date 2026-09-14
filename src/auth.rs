use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::errors::AppError;
use crate::models::{Claims, UserRole};

const ACCESS_TOKEN_TTL_SECONDS: u64 = 15 * 60;
pub const REFRESH_TOKEN_TTL_DAYS: i64 = 7;

pub fn create_jwt(user_id: i32, role: UserRole, jwt_secret: &str) -> Result<String, AppError> {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::InternalError)?
        .as_secs()
        + ACCESS_TOKEN_TTL_SECONDS;
    let claims = Claims { id: user_id, exp, role };
    let jwt_token = encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes())).map_err(|e| {
        tracing::error!(error = ?e, "failed to encode jwt");
        AppError::InternalError
    })?;
    Ok(jwt_token)
}

pub fn verify_jwt(req_token: &str, jwt_secret: &str) -> Result<Claims, AppError> {
    let token = decode::<Claims>(req_token, &DecodingKey::from_secret(jwt_secret.as_bytes()), &Validation::default())
        .map_err(|_| AppError::Unauthorized)?;
    let claims = token.claims;
    Ok(claims)
}

pub fn create_refresh_token() -> String {
    Uuid::new_v4().to_string()
}

pub fn create_refresh_exp() -> DateTime<Utc> {
    chrono::Utc::now() + chrono::Duration::days(REFRESH_TOKEN_TTL_DAYS)
}

pub fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    hex::encode(digest)
}
