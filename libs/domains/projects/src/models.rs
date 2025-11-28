use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

/// Supported cloud providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum CloudProvider {
    Aws,
    Gcp,
    Azure,
}

/// Project deployment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ProjectStatus {
    /// Project is being set up
    Provisioning,
    /// Project is active and running
    Active,
    /// Project is temporarily suspended
    Suspended,
    /// Project is being deleted
    Deleting,
    /// Project has been archived
    Archived,
}

impl Default for ProjectStatus {
    fn default() -> Self {
        Self::Provisioning
    }
}

/// Environment type for the project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

/// Project entity - represents a cloud project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique identifier
    pub id: Uuid,
    /// Project name (must be unique per user)
    pub name: String,
    /// Owner of the project
    pub user_id: Uuid,
    /// Project description
    pub description: String,
    /// Cloud provider (AWS, GCP, Azure)
    pub cloud_provider: CloudProvider,
    /// Deployment region (e.g., us-east-1, europe-west1)
    pub region: String,
    /// Environment type
    pub environment: Environment,
    /// Current status
    pub status: ProjectStatus,
    /// Optional monthly budget limit in USD
    pub budget_limit: Option<f64>,
    /// Resource tags for organization
    pub tags: Vec<Tag>,
    /// Whether the project is enabled
    pub enabled: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Key-value tag for project organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

/// DTO for creating a new project
#[derive(Debug, Clone, Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub user_id: Uuid,
    #[serde(default)]
    pub description: String,
    pub cloud_provider: CloudProvider,
    pub region: String,
    #[serde(default)]
    pub environment: Environment,
    pub budget_limit: Option<f64>,
    #[serde(default)]
    pub tags: Vec<Tag>,
}

/// DTO for updating an existing project
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProject {
    pub name: Option<String>,
    pub description: Option<String>,
    pub region: Option<String>,
    pub environment: Option<Environment>,
    pub status: Option<ProjectStatus>,
    pub budget_limit: Option<f64>,
    pub tags: Option<Vec<Tag>>,
    pub enabled: Option<bool>,
}

/// Query filters for listing projects
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectFilter {
    pub user_id: Option<Uuid>,
    pub cloud_provider: Option<CloudProvider>,
    pub environment: Option<Environment>,
    pub status: Option<ProjectStatus>,
    pub enabled: Option<bool>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

fn default_limit() -> usize {
    50
}

impl Project {
    /// Create a new project from CreateProject DTO
    pub fn new(input: CreateProject) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: input.name,
            user_id: input.user_id,
            description: input.description,
            cloud_provider: input.cloud_provider,
            region: input.region,
            environment: input.environment,
            status: ProjectStatus::Provisioning,
            budget_limit: input.budget_limit,
            tags: input.tags,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Apply updates from UpdateProject DTO
    pub fn apply_update(&mut self, update: UpdateProject) {
        if let Some(name) = update.name {
            self.name = name;
        }
        if let Some(description) = update.description {
            self.description = description;
        }
        if let Some(region) = update.region {
            self.region = region;
        }
        if let Some(environment) = update.environment {
            self.environment = environment;
        }
        if let Some(status) = update.status {
            self.status = status;
        }
        if let Some(budget_limit) = update.budget_limit {
            self.budget_limit = Some(budget_limit);
        }
        if let Some(tags) = update.tags {
            self.tags = tags;
        }
        if let Some(enabled) = update.enabled {
            self.enabled = enabled;
        }
        self.updated_at = Utc::now();
    }
}
