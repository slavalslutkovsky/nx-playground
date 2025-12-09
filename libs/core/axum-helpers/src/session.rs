use tower_sessions::{MemoryStore, SessionManagerLayer, Expiry};
use tower_sessions::cookie::time::Duration;

/// Create auth and session layers for axum-login
///
/// # Arguments
/// * `secret` - Secret key for session signing
///
/// # Returns
/// Tuple of (session_layer, auth_layer) ready to be added to the axum router
pub fn create_session_layer(secret: &[u8]) -> SessionManagerLayer<MemoryStore> {
    let session_store = MemoryStore::default();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));

    session_layer
}
