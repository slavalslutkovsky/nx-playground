"""
Common types shared across all platform resources.
These types are the source of truth for XRD generation and composition functions.
"""

# Type aliases for provider options
type CloudProvider = "aws" | "gcp"
type CloudProviderWithK8s = "aws" | "gcp" | "kubernetes"
type Environment = "dev" | "staging" | "prod"
type DeletionPolicy = "Delete" | "Orphan"

schema BaseSpec:
    """Base specification shared by all platform resources."""
    provider: str
    environment: Environment = "dev"
    deletionPolicy: DeletionPolicy = "Delete"

    check:
        provider in ["aws", "gcp", "kubernetes"], "provider must be aws, gcp, or kubernetes"

schema BaseStatus:
    """Base status shared by all platform resources."""
    provider?: str
    region?: str

# OpenAPI schema generation helpers
_string_schema = lambda description: str, enum_values: [str] = [], default_value: str = "" -> {str:any} {
    result: {str:any} = {type = "string", description = description}
    if enum_values:
        result |= {"enum" = enum_values}
    if default_value:
        result |= {"default" = default_value}
    result
}

_boolean_schema = lambda description: str, default_value: bool = False -> {str:any} {
    {type = "boolean", description = description, "default" = default_value}
}

_integer_schema = lambda description: str, default_value: int = 0 -> {str:any} {
    {type = "integer", description = description, "default" = default_value}
}

# Common property schemas for reuse
PROVIDER_CLOUD_SCHEMA = _string_schema("Cloud provider (aws or gcp)", ["aws", "gcp"])
PROVIDER_WITH_K8S_SCHEMA = _string_schema("Cloud provider (aws, gcp) or kubernetes for in-cluster", ["aws", "gcp", "kubernetes"])
ENVIRONMENT_SCHEMA = _string_schema("Environment (dev, staging, prod)", ["dev", "staging", "prod"], "dev")
DELETION_POLICY_SCHEMA = _string_schema("Deletion policy", ["Delete", "Orphan"], "Delete")
REGION_SCHEMA = _string_schema("Cloud region")
NAME_SCHEMA = _string_schema("Resource name")
NAMESPACE_SCHEMA = _string_schema("Target namespace for kubernetes provider", [], "default")
