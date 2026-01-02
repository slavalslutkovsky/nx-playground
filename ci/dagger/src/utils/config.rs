/// CI configuration derived from CLI arguments and environment
#[derive(Debug, Clone)]
pub struct CiConfig {
    /// Run in CI mode with affected detection
    pub ci_mode: bool,
    /// GCS bucket for sccache (if configured)
    pub sccache_bucket: Option<String>,
    /// Base SHA for nx affected
    pub base_sha: String,
    /// Head SHA for nx affected
    pub head_sha: String,
}

impl CiConfig {
    pub fn new(
        ci_mode: bool,
        sccache_bucket: Option<String>,
        base_sha: Option<String>,
        head_sha: Option<String>,
    ) -> Self {
        Self {
            ci_mode,
            sccache_bucket,
            base_sha: base_sha.unwrap_or_else(|| "origin/main".to_string()),
            head_sha: head_sha.unwrap_or_else(|| "HEAD".to_string()),
        }
    }

    /// Check if sccache is configured
    pub fn has_sccache(&self) -> bool {
        self.sccache_bucket.is_some()
    }
}
