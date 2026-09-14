use axum::Json;
use axum::body::Body;
use axum::extract::FromRequest;
use axum::{extract::FromRequestParts, http::request::Parts};
use axum::http::Request;
use validator::Validate;

use crate::auth::verify_jwt;
use crate::errors::AppError;
use crate::models::AuthenticatedAdmin;
use crate::models::AuthenticatedUser;
use crate::models::UserRole;
use crate::models::AppState;

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let jwt = parts
            .headers
            .get("Authorization")
            .and_then(|t| t.to_str().ok())
            .and_then(|t| t.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        let claims = verify_jwt(jwt, &state.jwt_secret)?;
        Ok(AuthenticatedUser {
            role: claims.role,
            id: claims.id,
        })
    }
}

impl FromRequestParts<AppState> for AuthenticatedAdmin {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;
        match user.role {
            UserRole::Admin => Ok(AuthenticatedAdmin),
            _ => Err(AppError::Forbidden),
        }
    }
}

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: serde::de::DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(request: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(request, state).await.map_err(|_| AppError::BadRequest)?;
        value.validate().map_err(|e| AppError::Validation(e.to_string()))?;
        Ok(ValidatedJson(value))
    }
}
