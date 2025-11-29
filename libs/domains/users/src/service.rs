use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{UserError, UserResult};
use crate::models::{CreateUser, Role, UpdateUser, User, UserFilter, UserResponse};
use crate::repository::UserRepository;

/// Service layer for User business logic
#[derive(Clone)]
pub struct UserService<R: UserRepository> {
    repository: Arc<R>,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    /// Create a new user with password hashing
    pub async fn create_user(&self, input: CreateUser) -> UserResult<UserResponse> {
        // Validate input
        self.validate_create(&input)?;

        // Hash password
        let password_hash = self.hash_password(&input.password)?;

        // Parse roles
        let roles: Vec<Role> = input.roles.iter().filter_map(|r| r.parse().ok()).collect();

        let user = User::new(input.email, input.name, password_hash, roles);

        let created = self.repository.create(user).await?;
        Ok(created.into())
    }

    /// Get a user by ID
    pub async fn get_user(&self, id: Uuid) -> UserResult<UserResponse> {
        let user = self
            .repository
            .get_by_id(id)
            .await?
            .ok_or(UserError::NotFound(id))?;

        Ok(user.into())
    }

    /// Get a user by email
    pub async fn get_user_by_email(&self, email: &str) -> UserResult<UserResponse> {
        let user = self.repository.get_by_email(email).await?.ok_or_else(|| {
            UserError::Validation(format!("User with email '{}' not found", email))
        })?;

        Ok(user.into())
    }

    /// List users with filters
    pub async fn list_users(&self, filter: UserFilter) -> UserResult<(Vec<UserResponse>, usize)> {
        let total = self.repository.count(filter.clone()).await?;
        let users = self.repository.list(filter).await?;
        let responses: Vec<UserResponse> = users.into_iter().map(|u| u.into()).collect();
        Ok((responses, total))
    }

    /// Update a user
    pub async fn update_user(&self, id: Uuid, input: UpdateUser) -> UserResult<UserResponse> {
        // Validate input
        self.validate_update(&input)?;

        // Get existing user
        let mut user = self
            .repository
            .get_by_id(id)
            .await?
            .ok_or(UserError::NotFound(id))?;

        // Hash new password if provided
        let new_password_hash = if let Some(ref password) = input.password {
            Some(self.hash_password(password)?)
        } else {
            None
        };

        // Check for duplicate email if email is being changed
        if let Some(ref new_email) = input.email {
            if new_email.to_lowercase() != user.email.to_lowercase() {
                if self.repository.email_exists(new_email).await? {
                    return Err(UserError::DuplicateEmail(new_email.clone()));
                }
            }
        }

        user.apply_update(input, new_password_hash);

        let updated = self.repository.update(user).await?;
        Ok(updated.into())
    }

    /// Delete a user
    pub async fn delete_user(&self, id: Uuid) -> UserResult<()> {
        let deleted = self.repository.delete(id).await?;

        if !deleted {
            return Err(UserError::NotFound(id));
        }

        Ok(())
    }

    /// Verify user credentials (for login)
    pub async fn verify_credentials(
        &self,
        email: &str,
        password: &str,
    ) -> UserResult<UserResponse> {
        let user = self
            .repository
            .get_by_email(email)
            .await?
            .ok_or(UserError::InvalidCredentials)?;

        if !self.verify_password(password, &user.password_hash)? {
            return Err(UserError::InvalidCredentials);
        }

        Ok(user.into())
    }

    /// Verify email (mark as verified)
    pub async fn verify_email(&self, id: Uuid) -> UserResult<UserResponse> {
        let mut user = self
            .repository
            .get_by_id(id)
            .await?
            .ok_or(UserError::NotFound(id))?;

        user.email_verified = true;
        user.updated_at = chrono::Utc::now();

        let updated = self.repository.update(user).await?;
        Ok(updated.into())
    }

    /// Change user password
    pub async fn change_password(
        &self,
        id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> UserResult<()> {
        let mut user = self
            .repository
            .get_by_id(id)
            .await?
            .ok_or(UserError::NotFound(id))?;

        // Verify current password
        if !self.verify_password(current_password, &user.password_hash)? {
            return Err(UserError::InvalidCredentials);
        }

        // Validate new password
        self.validate_password(new_password)?;

        // Hash and update
        user.password_hash = self.hash_password(new_password)?;
        user.updated_at = chrono::Utc::now();

        self.repository.update(user).await?;
        Ok(())
    }

    // Password helpers

    fn hash_password(&self, password: &str) -> UserResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| UserError::PasswordHash(e.to_string()))
    }

    fn verify_password(&self, password: &str, hash: &str) -> UserResult<bool> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| UserError::PasswordHash(e.to_string()))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    // Validation helpers

    fn validate_create(&self, input: &CreateUser) -> UserResult<()> {
        self.validate_email(&input.email)?;
        self.validate_name(&input.name)?;
        self.validate_password(&input.password)?;
        Ok(())
    }

    fn validate_update(&self, input: &UpdateUser) -> UserResult<()> {
        if let Some(ref email) = input.email {
            self.validate_email(email)?;
        }
        if let Some(ref name) = input.name {
            self.validate_name(name)?;
        }
        if let Some(ref password) = input.password {
            self.validate_password(password)?;
        }
        Ok(())
    }

    fn validate_email(&self, email: &str) -> UserResult<()> {
        if email.trim().is_empty() {
            return Err(UserError::Validation("Email cannot be empty".to_string()));
        }

        if !email.contains('@') || !email.contains('.') {
            return Err(UserError::Validation("Invalid email format".to_string()));
        }

        if email.len() > 255 {
            return Err(UserError::Validation(
                "Email cannot exceed 255 characters".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_name(&self, name: &str) -> UserResult<()> {
        if name.trim().is_empty() {
            return Err(UserError::Validation("Name cannot be empty".to_string()));
        }

        if name.len() > 100 {
            return Err(UserError::Validation(
                "Name cannot exceed 100 characters".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_password(&self, password: &str) -> UserResult<()> {
        if password.len() < 8 {
            return Err(UserError::Validation(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        if password.len() > 128 {
            return Err(UserError::Validation(
                "Password cannot exceed 128 characters".to_string(),
            ));
        }

        Ok(())
    }
}
