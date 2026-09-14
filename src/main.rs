mod auth;
mod cleanup;
mod errors;
mod extractors;
mod handlers;
mod models;
mod rate_limiters;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower::ServiceBuilder;

use axum::Router;
use axum::routing::{get, post};
use tower_http::trace::TraceLayer;

use rate_limiters::{rate_limit, CLEANUP_INTERVAL_SECS, EMAIL_CLEANUP_INTERVAL_SECS, IP_CLEANUP_INTERVAL_SECS, RATE_LIMIT_WINDOW_SECONDS, EMAIL_RATE_LIMIT_WINDOW_SECONDS, IP_RATE_LIMIT_WINDOW_SECONDS};
use handlers::{delete_user, find_users_by_username, get_user_by_id, login_user, logout_user, register_user, update_access_token};
use models::AppState;

use crate::handlers::update_my_profile;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = AppState {
        pool,
        jwt_secret,
        rate_limiter: Arc::new(Mutex::new(HashMap::new())),
        login_email_limiter: Arc::new(Mutex::new(HashMap::new())),
        login_ip_limiter: Arc::new(Mutex::new(HashMap::new())),
    };

    cleanup::spawn_rate_limiter_cleanup(state.rate_limiter.clone(), RATE_LIMIT_WINDOW_SECONDS, CLEANUP_INTERVAL_SECS);
    cleanup::spawn_rate_limiter_cleanup(state.login_email_limiter.clone(), EMAIL_RATE_LIMIT_WINDOW_SECONDS, EMAIL_CLEANUP_INTERVAL_SECS);
    cleanup::spawn_rate_limiter_cleanup(state.login_ip_limiter.clone(), IP_RATE_LIMIT_WINDOW_SECONDS, IP_CLEANUP_INTERVAL_SECS);
    cleanup::clean_inactive_refresh_tokens(state.clone());

    let app = Router::new()
        .route("/users/{id}", get(get_user_by_id).delete(delete_user))
        .route("/search", get(find_users_by_username))
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .route("/auth/refresh", post(update_access_token))
        .route("/auth/logout", post(logout_user))
        .route("/update/me", post(update_my_profile))
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn_with_state(state.clone(), rate_limit))
                .layer(TraceLayer::new_for_http()),
        )
        .with_state(state);

    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener");

    tracing::info!("listening on {addr}");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("Failed to serve");
}
