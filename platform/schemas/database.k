"""
Database schema - Provider-agnostic managed database specification.
Source of truth for XDatabase XRD and compose-database function.
Supports AWS RDS, GCP CloudSQL, and CloudNative-PG (kubernetes).
"""
import .common as c

type DatabaseEngine = "postgres" | "mysql"
type DatabaseSize = "small" | "medium" | "large"

schema DatabaseSpec(c.BaseSpec):
    """Provider-agnostic database specification."""
    provider: c.CloudProviderWithK8s
    region?: str  # Required for aws/gcp, ignored for kubernetes
    engine: DatabaseEngine = "postgres"
    engineVersion: str = "18"
    size: DatabaseSize = "small"
    storageGB: int = 20
    instances: int = 1  # CNPG only
    namespace: str = "default"  # kubernetes only

    check:
        provider != "kubernetes" or engine == "postgres", "kubernetes provider only supports postgres (CloudNative-PG)"
        provider == "kubernetes" or region, "region is required for aws/gcp providers"
        instances >= 1, "instances must be at least 1"
        storageGB >= 1, "storageGB must be at least 1"

schema DatabaseStatus(c.BaseStatus):
    """Status fields for database resources."""
    engine?: str
    endpoint?: str

# OpenAPI v3 schema for XRD generation
SPEC_PROPERTIES = {
    provider = c.PROVIDER_WITH_K8S_SCHEMA | {
        description = "Cloud provider (aws, gcp) or kubernetes for CloudNative-PG"
    }
    region = {
        type = "string"
        description = "Cloud region (required for aws/gcp, ignored for kubernetes)"
    }
    engine = {
        type = "string"
        description = "Database engine (kubernetes only supports postgres)"
        enum = ["postgres", "mysql"]
        default = "postgres"
    }
    engineVersion = {
        type = "string"
        description = "Database engine version"
        default = "15"
    }
    size = {
        type = "string"
        description = "Instance size"
        enum = ["small", "medium", "large"]
        default = "small"
    }
    storageGB = c._integer_schema("Storage size in GB", 20)
    instances = c._integer_schema("Number of instances (kubernetes/CNPG only)", 1)
    namespace = c.NAMESPACE_SCHEMA
    environment = c.ENVIRONMENT_SCHEMA
    deletionPolicy = c.DELETION_POLICY_SCHEMA
}

SPEC_REQUIRED = ["provider", "engine"]

STATUS_PROPERTIES = {
    provider = {type = "string"}
    region = {type = "string"}
    engine = {type = "string"}
    endpoint = {type = "string"}
}

# XRD metadata
XRD_NAME = "xdatabases.database.platform.io"
XRD_GROUP = "database.platform.io"
XRD_KIND = "XDatabase"
XRD_PLURAL = "xdatabases"
XRD_CLAIM_KIND = "Database"
XRD_CLAIM_PLURAL = "databases"
XRD_COMPOSITION = "xdatabase"
