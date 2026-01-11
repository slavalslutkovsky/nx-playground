//! AWS SES provider with HashiCorp Vault integration
//!
//! Fetches dynamic AWS credentials from Vault's AWS secrets engine.
//! Credentials are automatically refreshed before expiration.
//!
//! ## Setup
//!
//! 1. Enable AWS secrets engine in Vault:
//!    ```bash
//!    vault secrets enable aws
//!    vault write aws/config/root access_key=... secret_key=... region=us-east-1
//!    vault write aws/roles/ses-sender \
//!        credential_type=iam_user \
//!        policy_arns=arn:aws:iam::aws:policy/AmazonSESFullAccess \
//!        default_ttl=1h max_ttl=24h
//!    ```
//!
//! 2. Configure Kubernetes auth:
//!    ```bash
//!    vault auth enable kubernetes
//!    vault write auth/kubernetes/config kubernetes_host="https://kubernetes.default.svc"
//!    vault write auth/kubernetes/role/email-worker \
//!        bound_service_account_names=email-worker \
//!        bound_service_account_namespaces=zerg \
//!        policies=ses-sender ttl=1h
//!    ```
//!
//! 3. Set environment variables:
//!    - `VAULT_ADDR` - Vault server address (e.g., https://vault.example.com)
//!    - `VAULT_ROLE` - Kubernetes auth role name
//!    - `VAULT_AWS_ROLE` - AWS secrets engine role name
//!    - `AWS_SES_REGION` - SES region
//!    - `EMAIL_FROM_ADDRESS` - Sender email
//!    - `EMAIL_FROM_NAME` - Sender name (optional)

use crate::models::Email;
use crate::provider::{EmailProvider, SendResult};
use async_trait::async_trait;
use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::Client;
use eyre::{eyre, Result};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use vaultrs::api::aws::requests::GenerateCredentialsRequest;
use vaultrs::auth::kubernetes;
use vaultrs::aws;
use vaultrs::client::{Client as VaultClientTrait, VaultClient, VaultClientSettingsBuilder};

const K8S_SA_TOKEN_PATH: &str = "/var/run/secrets/kubernetes.io/serviceaccount/token";
const REFRESH_BUFFER_SECS: u64 = 300; // Refresh 5 minutes before expiry
const DEFAULT_CREDENTIAL_TTL_SECS: u64 = 3600; // Default 1 hour TTL

/// AWS SES provider with Vault-managed credentials
pub struct SesVaultProvider {
    vault_client: VaultClient,
    vault_aws_role: String,
    vault_aws_mount: String,
    region: String,
    from_email: String,
    from_name: String,
    /// Cached SES client and expiry
    client_cache: Arc<RwLock<Option<CachedClient>>>,
}

struct CachedClient {
    client: Client,
    expires_at: u64,
}

/// Configuration for SesVaultProvider
#[derive(Debug, Clone)]
pub struct SesVaultConfig {
    /// Vault server address
    pub vault_addr: String,
    /// Kubernetes auth role
    pub vault_k8s_role: String,
    /// Kubernetes auth mount point (default: "kubernetes")
    pub vault_k8s_mount: String,
    /// AWS secrets engine role
    pub vault_aws_role: String,
    /// AWS secrets engine mount point (default: "aws")
    pub vault_aws_mount: String,
    /// AWS region for SES
    pub region: String,
    /// Sender email address
    pub from_email: String,
    /// Sender name
    pub from_name: String,
}

impl SesVaultConfig {
    /// Create config from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            vault_addr: std::env::var("VAULT_ADDR").map_err(|_| eyre!("VAULT_ADDR not set"))?,
            vault_k8s_role: std::env::var("VAULT_ROLE").map_err(|_| eyre!("VAULT_ROLE not set"))?,
            vault_k8s_mount: std::env::var("VAULT_K8S_MOUNT")
                .unwrap_or_else(|_| "kubernetes".to_string()),
            vault_aws_role: std::env::var("VAULT_AWS_ROLE")
                .map_err(|_| eyre!("VAULT_AWS_ROLE not set"))?,
            vault_aws_mount: std::env::var("VAULT_AWS_MOUNT").unwrap_or_else(|_| "aws".to_string()),
            region: std::env::var("AWS_SES_REGION")
                .or_else(|_| std::env::var("AWS_REGION"))
                .map_err(|_| eyre!("AWS_SES_REGION not set"))?,
            from_email: std::env::var("EMAIL_FROM_ADDRESS")
                .or_else(|_| std::env::var("SES_FROM_EMAIL"))
                .map_err(|_| eyre!("EMAIL_FROM_ADDRESS not set"))?,
            from_name: std::env::var("EMAIL_FROM_NAME")
                .unwrap_or_else(|_| "Notifications".to_string()),
        })
    }
}

impl SesVaultProvider {
    /// Create a new SesVaultProvider with the given configuration
    pub async fn new(config: SesVaultConfig) -> Result<Self> {
        // Read Kubernetes service account token
        let k8s_token = tokio::fs::read_to_string(K8S_SA_TOKEN_PATH)
            .await
            .map_err(|e| eyre!("Failed to read K8s SA token: {}", e))?;

        // Create Vault client
        let vault_settings = VaultClientSettingsBuilder::default()
            .address(&config.vault_addr)
            .build()
            .map_err(|e| eyre!("Invalid Vault settings: {}", e))?;

        let mut vault_client = VaultClient::new(vault_settings)
            .map_err(|e| eyre!("Failed to create Vault client: {}", e))?;

        // Authenticate to Vault using Kubernetes auth
        info!(role = %config.vault_k8s_role, "Authenticating to Vault via Kubernetes auth");

        let auth_response = kubernetes::login(
            &vault_client,
            &config.vault_k8s_mount,
            &config.vault_k8s_role,
            &k8s_token,
        )
        .await
        .map_err(|e| eyre!("Vault Kubernetes auth failed: {}", e))?;

        // Set the token on the client
        vault_client.set_token(&auth_response.client_token);

        info!("Successfully authenticated to Vault");

        Ok(Self {
            vault_client,
            vault_aws_role: config.vault_aws_role,
            vault_aws_mount: config.vault_aws_mount,
            region: config.region,
            from_email: config.from_email,
            from_name: config.from_name,
            client_cache: Arc::new(RwLock::new(None)),
        })
    }

    /// Create from environment variables
    pub async fn from_env() -> Result<Self> {
        let config = SesVaultConfig::from_env()?;
        Self::new(config).await
    }

    /// Get or create an SES client with valid credentials
    async fn get_client(&self) -> Result<Client> {
        // Check if we have a cached client that's still valid
        {
            let cache = self.client_cache.read().await;
            if let Some(cached) = cache.as_ref() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if cached.expires_at > now + REFRESH_BUFFER_SECS {
                    return Ok(cached.client.clone());
                }

                debug!("Cached AWS credentials expiring soon, refreshing");
            }
        }

        // Fetch new credentials from Vault
        let client = self.refresh_credentials().await?;
        Ok(client)
    }

    /// Fetch fresh AWS credentials from Vault and create a new SES client
    async fn refresh_credentials(&self) -> Result<Client> {
        info!(role = %self.vault_aws_role, "Fetching AWS credentials from Vault");

        // Generate credentials from Vault AWS secrets engine
        let creds = aws::roles::credentials(
            &self.vault_client,
            &self.vault_aws_mount,
            &self.vault_aws_role,
            Some(&mut GenerateCredentialsRequest::builder()),
        )
        .await
        .map_err(|e| eyre!("Failed to fetch AWS credentials from Vault: {}", e))?;

        debug!(
            access_key = %creds.access_key,
            has_session_token = creds.security_token.is_some(),
            "Got AWS credentials from Vault"
        );

        // Calculate expiry time (use default TTL since response doesn't include lease_duration)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expires_at = now + DEFAULT_CREDENTIAL_TTL_SECS;

        // Create AWS credentials provider
        let credentials = aws_sdk_sesv2::config::Credentials::new(
            &creds.access_key,
            &creds.secret_key,
            creds.security_token.clone(),
            Some(SystemTime::now() + Duration::from_secs(DEFAULT_CREDENTIAL_TTL_SECS)),
            "vault",
        );

        // Build SES client
        let config = aws_sdk_sesv2::Config::builder()
            .region(aws_sdk_sesv2::config::Region::new(self.region.clone()))
            .credentials_provider(credentials)
            .build();

        let client = Client::from_conf(config);

        // Cache the client
        {
            let mut cache = self.client_cache.write().await;
            *cache = Some(CachedClient {
                client: client.clone(),
                expires_at,
            });
        }

        info!(
            expires_in_secs = DEFAULT_CREDENTIAL_TTL_SECS,
            "AWS credentials cached"
        );

        Ok(client)
    }

    /// Format email address with name
    fn format_address(&self, email: &str, name: Option<&str>) -> String {
        match name {
            Some(n) if !n.is_empty() => format!("{} <{}>", n, email),
            _ => email.to_string(),
        }
    }
}

#[async_trait]
impl EmailProvider for SesVaultProvider {
    async fn send(&self, email: &Email) -> Result<SendResult> {
        let client = self.get_client().await?;

        // Build destination
        let mut destination = Destination::builder().to_addresses(&email.to);

        for cc in &email.cc {
            destination = destination.cc_addresses(cc);
        }

        for bcc in &email.bcc {
            destination = destination.bcc_addresses(bcc);
        }

        let destination = destination.build();

        // Build body
        let mut body = Body::builder();

        if let Some(text) = &email.body_text {
            body = body.text(Content::builder().data(text).charset("UTF-8").build()?);
        }

        if let Some(html) = &email.body_html {
            body = body.html(Content::builder().data(html).charset("UTF-8").build()?);
        }

        let body = body.build();

        // Build message
        let message = Message::builder()
            .subject(
                Content::builder()
                    .data(&email.subject)
                    .charset("UTF-8")
                    .build()?,
            )
            .body(body)
            .build();

        // Build email content
        let email_content = EmailContent::builder().simple(message).build();

        // Get from address
        let from_email = email.from.as_ref().unwrap_or(&self.from_email);
        let from_address = self.format_address(from_email, Some(&self.from_name));

        debug!(
            to = %email.to,
            subject = %email.subject,
            from = %from_address,
            "Sending email via AWS SES (Vault credentials)"
        );

        // Send the email
        let mut request = client
            .send_email()
            .from_email_address(&from_address)
            .destination(destination)
            .content(email_content);

        if let Some(reply_to) = &email.reply_to {
            request = request.reply_to_addresses(reply_to);
        }

        let response = request.send().await.map_err(|e| {
            error!(error = %e, "AWS SES send failed");

            let err_str = e.to_string();
            if err_str.contains("ExpiredToken") || err_str.contains("credentials") {
                // Clear cache on credential errors
                warn!("AWS credentials may be expired, will refresh on next request");
                eyre!("authentication failed (credentials expired): {}", e)
            } else if err_str.contains("Throttling") || err_str.contains("rate") {
                eyre!("rate limit exceeded: {}", e)
            } else {
                eyre!("SES error: {}", e)
            }
        })?;

        let message_id = response.message_id().unwrap_or(&email.id).to_string();

        debug!(
            message_id = %message_id,
            "Email sent successfully via AWS SES (Vault)"
        );

        Ok(SendResult { message_id })
    }

    async fn health_check(&self) -> Result<()> {
        // Verify we can get credentials from Vault
        let client = self.get_client().await?;

        // Verify SES access
        client
            .get_account()
            .send()
            .await
            .map_err(|e| eyre!("AWS SES health check failed: {}", e))?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "aws-ses-vault"
    }
}
