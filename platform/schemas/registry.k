"""
Registry schema - Provider-agnostic container registry specification.
Source of truth for XRegistry XRD and compose-registry function.
Supports AWS ECR, GCP Artifact Registry, and Harbor (kubernetes).
"""
import .common as c

type RegistryFormat = "docker" | "npm" | "maven" | "python"

schema RegistrySpec(c.BaseSpec):
    """Provider-agnostic container registry specification."""
    provider: c.CloudProviderWithK8s
    name: str
    region?: str  # Required for aws/gcp
    format: RegistryFormat = "docker"  # GCP Artifact Registry only
    immutableTags: bool = False
    scanOnPush: bool = True  # ECR only
    encryption: bool = True
    publicAccess: bool = False
    retentionDays: int = 0  # 0 = no expiration
    namespace: str = "default"  # kubernetes only

    check:
        len(name) > 0, "name is required"
        provider == "kubernetes" or region, "region is required for aws/gcp providers"
        retentionDays >= 0, "retentionDays must be non-negative"

schema RegistryStatus(c.BaseStatus):
    """Status fields for registry resources."""
    registryUrl?: str
    repositoryName?: str

# OpenAPI v3 schema for XRD generation
SPEC_PROPERTIES = {
    provider = c.PROVIDER_WITH_K8S_SCHEMA | {
        description = "Cloud provider (aws, gcp) or kubernetes for in-cluster registry"
    }
    name = {
        type = "string"
        description = "Registry/repository name"
    }
    region = {
        type = "string"
        description = "Cloud region (required for aws/gcp)"
    }
    format = {
        type = "string"
        description = "Repository format (docker, npm, maven, etc.) - GCP Artifact Registry only"
        enum = ["docker", "npm", "maven", "python"]
        default = "docker"
    }
    immutableTags = c._boolean_schema("Prevent tag overwrites (ECR/Artifact Registry)")
    scanOnPush = c._boolean_schema("Enable vulnerability scanning on push (ECR only)", True)
    encryption = c._boolean_schema("Enable encryption", True)
    publicAccess = c._boolean_schema("Allow public access")
    retentionDays = c._integer_schema("Image retention in days (0 = no expiration)")
    namespace = c.NAMESPACE_SCHEMA
    environment = c.ENVIRONMENT_SCHEMA
    deletionPolicy = c.DELETION_POLICY_SCHEMA
}

SPEC_REQUIRED = ["provider", "name"]

STATUS_PROPERTIES = {
    provider = {type = "string"}
    region = {type = "string"}
    registryUrl = {type = "string"}
    repositoryName = {type = "string"}
}

# XRD metadata
XRD_NAME = "xregistries.registry.platform.io"
XRD_GROUP = "registry.platform.io"
XRD_KIND = "XRegistry"
XRD_PLURAL = "xregistries"
XRD_CLAIM_KIND = "Registry"
XRD_CLAIM_PLURAL = "registries"
XRD_COMPOSITION = "xregistry"
