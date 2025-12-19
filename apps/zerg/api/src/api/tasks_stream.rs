//! Stream-based tasks API routes (fire-and-forget).
//!
//! Returns 202 Accepted immediately after queueing the command.

use axum::Router;
use domain_tasks::{stream_async_router, StreamState};

pub fn router(state: &crate::state::AppState) -> Router {
    let stream_state = StreamState::new(state.redis.clone());
    stream_async_router(stream_state)
}
