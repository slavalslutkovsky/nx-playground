use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User information stored in session
/// This is a simple wrapper around the user data for axum-login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    /// User ID
    pub id: Uuid,
    /// User email
    pub email: String,
    /// User roles
    pub roles: Vec<String>,
}

impl SessionUser {
    /// Create a new session user
    pub fn new(id: Uuid, email: String, roles: Vec<String>) -> Self {
        Self { id, email, roles }
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }
}

// Implement axum-login's AuthUser trait for SessionUser
impl axum_login::AuthUser for SessionUser {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        // Use email as the session auth hash
        // In production, you might want to use a more secure approach
        self.email.as_bytes()
    }
}
