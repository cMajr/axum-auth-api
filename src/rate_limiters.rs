use std::collections::HashMap;
use std::hash::Hash;
use std::net::{IpAddr, SocketAddr};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::{ConnectInfo, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::errors::AppError;
use crate::models::{AppState, WindowState};

pub const MAX_REQUESTS_PER_WINDOW: u64 = 30;
pub const RATE_LIMIT_WINDOW_SECONDS: u64 = 60;

// LOGIN RATE LIMITER
pub const MAX_EMAIL_REQUESTS_PER_WINDOW: u64 = 5;
pub const EMAIL_RATE_LIMIT_WINDOW_SECONDS: u64 = 60 * 5;

pub const MAX_IP_REQUESTS_PER_WINDOW: u64 = 50;
pub const IP_RATE_LIMIT_WINDOW_SECONDS: u64 = 60 * 5;

// CLEANUP INTERVAL
pub const CLEANUP_INTERVAL_SECS: u64 = 60;
pub const EMAIL_CLEANUP_INTERVAL_SECS: u64 = 600;
pub const IP_CLEANUP_INTERVAL_SECS: u64 = 600;

pub async fn rate_limit(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Response {
    {
        let user_ip = addr.ip();
        let mut rate_limiter = state
            .rate_limiter
            .lock()
            .expect("rate_limiter mutex poisoned — state may be inconsistent, aborting");

        let window = rate_limiter.entry(user_ip).or_insert_with(|| WindowState {
            counter: 0,
            window_start: Instant::now(),
        });

        window.reset_if_expired(Duration::from_secs(RATE_LIMIT_WINDOW_SECONDS));

        if window.counter >= MAX_REQUESTS_PER_WINDOW {
            return AppError::TooManyRequests.into_response();
        }

        window.counter += 1;
    }

    next.run(request).await
}

pub fn login_email_rate_limiter(email_limiter: &Mutex<HashMap<String, WindowState>>, email: &str) -> Result<(), AppError> {
    check_rate_limiter(email_limiter, email.to_string(), EMAIL_RATE_LIMIT_WINDOW_SECONDS, MAX_EMAIL_REQUESTS_PER_WINDOW)
}

pub fn login_ip_rate_limiter(ip_limiter: &Mutex<HashMap<IpAddr, WindowState>>, user_ip: IpAddr) -> Result<(), AppError> {
    check_rate_limiter(ip_limiter, user_ip, IP_RATE_LIMIT_WINDOW_SECONDS, MAX_IP_REQUESTS_PER_WINDOW)
}

fn check_rate_limiter<K: Eq + Hash>(
    limiter: &Mutex<HashMap<K, WindowState>>,
    key: K,
    limit_secs: u64,
    max_reqs: u64,
) -> Result<(), AppError> {
    let mut limiter = limiter.lock().expect("mutex poisoned, aborting");
    let window = limiter.entry(key).or_insert_with(|| WindowState {
        counter: 0,
        window_start: Instant::now(),
    });

    window.reset_if_expired(Duration::from_secs(limit_secs));

    if window.counter >= max_reqs {
        return Err(AppError::TooManyRequests);
    }

    window.counter += 1;

    Ok(())
}

impl WindowState {
    fn reset_if_expired(&mut self, window: Duration) {
        if self.window_start.elapsed() > window {
            self.counter = 0;
            self.window_start = Instant::now();
        }
    }
}
