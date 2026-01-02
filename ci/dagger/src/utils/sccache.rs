use dagger_sdk::{Container, Query};
use eyre::Result;
use tracing::info;

use super::config::CiConfig;

/// Configure sccache for compilation caching
pub async fn configure(
    client: &Query,
    container: Container,
    config: &CiConfig,
) -> Result<Container> {
    let bucket = config
        .sccache_bucket
        .as_ref()
        .ok_or_else(|| eyre::eyre!("sccache bucket not configured"))?;

    info!("Configuring sccache with GCS bucket: {}", bucket);

    // Install sccache
    let container = container.with_exec(vec!["cargo", "install", "--locked", "sccache"]);

    // Mount local sccache cache volume as fallback
    let container = container.with_mounted_cache("/opt/sccache", client.cache_volume("sccache"));

    // Set base sccache environment
    let mut container = container
        .with_env_variable("RUSTC_WRAPPER", "/usr/local/cargo/bin/sccache")
        .with_env_variable("SCCACHE_DIR", "/opt/sccache")
        .with_env_variable("CARGO_INCREMENTAL", "0")
        .with_env_variable("SCCACHE_LOG", "error");

    // Configure GCS backend if credentials are available
    if let Ok(gcp_creds) = std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
        info!("GCP credentials found, configuring GCS backend");

        if let Ok(creds_content) = std::fs::read_to_string(&gcp_creds) {
            let secret = client.set_secret("gcp_creds", &creds_content);
            container = container
                .with_mounted_secret("/gcp/credentials.json", secret)
                .with_env_variable("GOOGLE_APPLICATION_CREDENTIALS", "/gcp/credentials.json")
                .with_env_variable("SCCACHE_GCS_BUCKET", bucket)
                .with_env_variable("SCCACHE_GCS_RW_MODE", "READ_WRITE")
                .with_env_variable("SCCACHE_GCS_KEY_PREFIX", "sccache");
        }
    } else {
        info!("No GCP credentials, using local sccache cache only");
    }

    Ok(container)
}
