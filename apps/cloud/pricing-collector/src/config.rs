//! Configuration for the pricing collector

use core_config::FromEnv;
use database::postgres::PostgresConfig;
use eyre::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub environment: String,
    pub database: PostgresConfig,
    pub aws: AwsConfig,
    pub azure: AzureConfig,
    pub gcp: GcpConfig,
    /// Default regions to collect if not specified
    pub default_regions: Vec<String>,
}

fn default_environment() -> String {
    "development".to_string()
}

fn default_regions() -> Vec<String> {
    vec![
        // AWS
        "us-east-1".to_string(),
        "us-west-2".to_string(),
        "eu-west-1".to_string(),
        // Azure
        "eastus".to_string(),
        "westus2".to_string(),
        "westeurope".to_string(),
        // GCP
        "us-central1".to_string(),
        "us-east1".to_string(),
        "europe-west1".to_string(),
    ]
}

#[derive(Debug, Clone, Default)]
pub struct AwsConfig {
    /// AWS region for API calls
    pub region: String,

    /// Use IAM role (preferred) or access keys
    pub role_arn: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,

    /// Enabled regions for price collection
    pub regions: Vec<String>,

    /// Enable AWS price collection
    pub enabled: bool,
}

fn default_aws_region() -> String {
    "us-east-1".to_string()
}

fn default_aws_regions() -> Vec<String> {
    vec![
        "us-east-1".to_string(),
        "us-east-2".to_string(),
        "us-west-1".to_string(),
        "us-west-2".to_string(),
        "eu-west-1".to_string(),
        "eu-central-1".to_string(),
    ]
}

#[derive(Debug, Clone, Default)]
pub struct AzureConfig {
    /// Azure tenant ID
    pub tenant_id: Option<String>,
    /// Azure client ID
    pub client_id: Option<String>,
    /// Azure client secret
    pub client_secret: Option<String>,
    /// Azure subscription ID
    pub subscription_id: Option<String>,

    /// Enabled regions for price collection
    pub regions: Vec<String>,

    /// Enable Azure price collection
    pub enabled: bool,
}

fn default_azure_regions() -> Vec<String> {
    vec![
        "eastus".to_string(),
        "eastus2".to_string(),
        "westus".to_string(),
        "westus2".to_string(),
        "westeurope".to_string(),
        "northeurope".to_string(),
    ]
}

#[derive(Debug, Clone, Default)]
pub struct GcpConfig {
    /// GCP project ID
    pub project_id: Option<String>,
    /// Service account key JSON (base64 encoded or file path)
    pub service_account_key: Option<String>,

    /// Enabled regions for price collection
    pub regions: Vec<String>,

    /// Enable GCP price collection
    pub enabled: bool,
}

fn default_gcp_regions() -> Vec<String> {
    vec![
        "us-central1".to_string(),
        "us-east1".to_string(),
        "us-west1".to_string(),
        "europe-west1".to_string(),
        "europe-west2".to_string(),
    ]
}

fn default_true() -> bool {
    true
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let config = Config {
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            database: <PostgresConfig as FromEnv>::from_env()?,
            aws: AwsConfig {
                region: std::env::var("AWS_REGION").unwrap_or_else(|_| default_aws_region()),
                role_arn: std::env::var("AWS_ROLE_ARN").ok(),
                access_key_id: std::env::var("AWS_ACCESS_KEY_ID").ok(),
                secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY").ok(),
                regions: std::env::var("AWS_REGIONS")
                    .map(|s| s.split(',').map(|r| r.trim().to_string()).collect())
                    .unwrap_or_else(|_| default_aws_regions()),
                enabled: std::env::var("AWS_ENABLED")
                    .map(|s| s.parse().unwrap_or(true))
                    .unwrap_or(true),
            },
            azure: AzureConfig {
                tenant_id: std::env::var("AZURE_TENANT_ID").ok(),
                client_id: std::env::var("AZURE_CLIENT_ID").ok(),
                client_secret: std::env::var("AZURE_CLIENT_SECRET").ok(),
                subscription_id: std::env::var("AZURE_SUBSCRIPTION_ID").ok(),
                regions: std::env::var("AZURE_REGIONS")
                    .map(|s| s.split(',').map(|r| r.trim().to_string()).collect())
                    .unwrap_or_else(|_| default_azure_regions()),
                enabled: std::env::var("AZURE_ENABLED")
                    .map(|s| s.parse().unwrap_or(true))
                    .unwrap_or(true),
            },
            gcp: GcpConfig {
                project_id: std::env::var("GCP_PROJECT_ID").ok(),
                service_account_key: std::env::var("GCP_SERVICE_ACCOUNT_KEY").ok(),
                regions: std::env::var("GCP_REGIONS")
                    .map(|s| s.split(',').map(|r| r.trim().to_string()).collect())
                    .unwrap_or_else(|_| default_gcp_regions()),
                enabled: std::env::var("GCP_ENABLED")
                    .map(|s| s.parse().unwrap_or(true))
                    .unwrap_or(true),
            },
            default_regions: default_regions(),
        };

        Ok(config)
    }
}
