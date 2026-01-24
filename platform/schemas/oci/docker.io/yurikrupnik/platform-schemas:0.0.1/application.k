"""
Application schema - Full application stack specification.
Source of truth for XApplication XRD and compose-application function.
Composes XBucket + XDatabase as child resources.
"""
import .common as c

schema StorageConfig:
    """Storage configuration for application."""
    enabled: bool = True
    storageClass: str = "standard"

    check:
        storageClass in ["standard", "nearline", "coldline"], "invalid storageClass"

schema DatabaseConfig:
    """Database configuration for application."""
    enabled: bool = True
    engine: str = "postgres"
    size: str = "small"

    check:
        engine in ["postgres", "mysql"], "engine must be postgres or mysql"
        size in ["small", "medium", "large"], "size must be small, medium, or large"

schema ApplicationSpec(c.BaseSpec):
    """Full application stack specification."""
    provider: c.CloudProvider
    name: str
    region: str
    storage?: StorageConfig
    database?: DatabaseConfig

    check:
        len(name) > 0, "name is required"
        len(region) > 0, "region is required"

schema ApplicationStatus(c.BaseStatus):
    """Status fields for application resources."""
    bucketName?: str
    databaseEndpoint?: str

# OpenAPI v3 schema for XRD generation
APPLICATION_SPEC_PROPERTIES = {
    name = {
        type = "string"
        description = "Application name"
    }
    provider = c.PROVIDER_CLOUD_SCHEMA
    region = c.REGION_SCHEMA
    environment = c.ENVIRONMENT_SCHEMA
    storage = {
        type = "object"
        description = "Storage configuration"
        properties = {
            enabled = c._boolean_schema("Enable storage", True)
            storageClass = {
                type = "string"
                enum = ["standard", "nearline", "coldline"]
                default = "standard"
            }
        }
    }
    database = {
        type = "object"
        description = "Database configuration"
        properties = {
            enabled = c._boolean_schema("Enable database", True)
            engine = {
                type = "string"
                enum = ["postgres", "mysql"]
                default = "postgres"
            }
            size = {
                type = "string"
                enum = ["small", "medium", "large"]
                default = "small"
            }
        }
    }
    deletionPolicy = c.DELETION_POLICY_SCHEMA
}

APPLICATION_SPEC_REQUIRED = ["provider", "region", "name"]

APPLICATION_STATUS_PROPERTIES = {
    provider = {type = "string"}
    region = {type = "string"}
    bucketName = {type = "string"}
    databaseEndpoint = {type = "string"}
}

# XRD metadata
APPLICATION_XRD_NAME = "xapplications.platform.io"
APPLICATION_XRD_GROUP = "platform.io"
APPLICATION_XRD_KIND = "XApplication"
APPLICATION_XRD_PLURAL = "xapplications"
APPLICATION_XRD_CLAIM_KIND = "Application"
APPLICATION_XRD_CLAIM_PLURAL = "applications"
APPLICATION_XRD_COMPOSITION = "xapplication"
