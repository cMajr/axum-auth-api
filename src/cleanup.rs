use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::models::{AppState, WindowState};

pub fn spawn_rate_limiter_cleanup<K: Eq + Hash + Send + 'static>(limiter: Arc<Mutex<HashMap<K, WindowState>>>, window_seconds: u64, cleanup_interval: u64) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(cleanup_interval)); // вот это интервал чистильщика
        loop {
            ticker.tick().await;
            let mut limiter = limiter.lock().expect("rate_limiter mutex poisoned");
            limiter.retain(|_key, window| window.window_start.elapsed() < Duration::from_secs(window_seconds));
        }
    });
}

pub fn clean_inactive_refresh_tokens(state: AppState) {
    let cleanup_interval_secs = 60 * 60;
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(cleanup_interval_secs));
        loop {
            ticker.tick().await;
            if let Err(e) = sqlx::query!("DELETE FROM refresh_tokens WHERE expires_at < NOW()").execute(&state.pool).await {
                tracing::error!("failed to clean up expired refresh tokens: {e}");
            }
        }
    });
}
