use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::net::IpAddr;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};
use validator::Validate;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub rate_limiter: Arc<Mutex<HashMap<IpAddr, WindowState>>>,
    pub login_email_limiter: Arc<Mutex<HashMap<String, WindowState>>>,
    pub login_ip_limiter: Arc<Mutex<HashMap<IpAddr, WindowState>>>,
}

pub struct WindowState {
    pub counter: u64,
    pub window_start: Instant,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}

#[derive(Deserialize, Validate)]
pub struct UserRegisterRequest {
    #[validate(length(min = 4, max = 20, message = "Name must be 4-20 characters"))]
    pub username: String,
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    #[validate(length(min = 8, max = 120, message = "Password must be 8-120 characters"))]
    pub password: String,
    pub date_of_birth: Option<chrono::NaiveDate>,
}

#[derive(Deserialize, Validate)]
pub struct UserLoginRequest {
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    pub password: String,
}

pub struct UserCredentials {
    pub id: i32,
    pub role: UserRole,
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct UsersByUsernameQuery {
    #[validate(length(min = 4, max = 20, message = "Name must be 4-20 characters"))]
    pub username: String,
}

#[derive(Serialize)]
pub struct UserRegisterResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
    pub role: UserRole,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct PublicSearchUserResponse {
    pub id: i32,
    pub username: String,
    pub bio: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: i32,
    pub exp: u64,
    pub role: UserRole,
}

#[derive(Serialize)]
pub struct AuthTokensUserResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct UpdatedTokensResponse {
    pub refresh_token: String,
    pub access_token: String,
}

#[derive(Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "token must not be empty"))]
    pub token: String,
}

pub struct AuthenticatedUser {
    pub role: UserRole,
    pub id: i32,
}

pub struct AuthenticatedAdmin;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Deserialize, Validate)]
pub struct ProfileUpdateRequest {
    #[validate(length(min = 4, max = 20, message = "name must be 4-20 characters"))]
    pub username: Option<String>,
    #[validate(length(max = 300, message = "bio must be at most 300 characters"))]
    pub bio: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
}

#[derive(Serialize)]
pub struct ProfileResponse {
    pub username: String,
    pub bio: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
