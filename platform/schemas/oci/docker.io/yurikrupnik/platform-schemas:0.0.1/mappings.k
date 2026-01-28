"""
Provider-specific mappings for resource sizing and configuration.
Centralizes cloud provider translations for storage classes, instance types, etc.
"""

# =============================================================================
# Storage Class Mappings
# =============================================================================

# Map generic storage class to GCP storage class
GCP_STORAGE_CLASS_MAP = {
    "standard": "STANDARD"
    "nearline": "NEARLINE"
    "coldline": "COLDLINE"
    "archive": "ARCHIVE"
}

get_gcp_storage_class = lambda storage_class: str -> str {
    GCP_STORAGE_CLASS_MAP[storage_class] if storage_class in GCP_STORAGE_CLASS_MAP else "STANDARD"
}

# AWS S3 storage classes (for reference, S3 uses different mechanism)
AWS_STORAGE_CLASS_MAP = {
    "standard": "STANDARD"
    "nearline": "STANDARD_IA"
    "coldline": "GLACIER"
    "archive": "DEEP_ARCHIVE"
}

get_aws_storage_class = lambda storage_class: str -> str {
    AWS_STORAGE_CLASS_MAP[storage_class] if storage_class in AWS_STORAGE_CLASS_MAP else "STANDARD"
}

# =============================================================================
# Database Instance Size Mappings
# =============================================================================

# AWS RDS instance classes
AWS_RDS_INSTANCE_CLASS_MAP = {
    "small": "db.t3.micro"
    "medium": "db.t3.small"
    "large": "db.t3.medium"
    "xlarge": "db.t3.large"
}

get_aws_rds_instance_class = lambda size: str -> str {
    AWS_RDS_INSTANCE_CLASS_MAP[size] if size in AWS_RDS_INSTANCE_CLASS_MAP else "db.t3.micro"
}

# GCP CloudSQL tiers
GCP_CLOUDSQL_TIER_MAP = {
    "small": "db-f1-micro"
    "medium": "db-g1-small"
    "large": "db-custom-2-4096"
    "xlarge": "db-custom-4-8192"
}

get_gcp_cloudsql_tier = lambda size: str -> str {
    GCP_CLOUDSQL_TIER_MAP[size] if size in GCP_CLOUDSQL_TIER_MAP else "db-f1-micro"
}

# CloudNative-PG resource configurations
CNPG_RESOURCES_MAP = {
    "small": {
        "requests": {"memory": "256Mi", "cpu": "100m"}
        "limits": {"memory": "512Mi", "cpu": "500m"}
    }
    "medium": {
        "requests": {"memory": "512Mi", "cpu": "250m"}
        "limits": {"memory": "1Gi", "cpu": "1000m"}
    }
    "large": {
        "requests": {"memory": "1Gi", "cpu": "500m"}
        "limits": {"memory": "2Gi", "cpu": "2000m"}
    }
    "xlarge": {
        "requests": {"memory": "2Gi", "cpu": "1000m"}
        "limits": {"memory": "4Gi", "cpu": "4000m"}
    }
}

get_cnpg_resources = lambda size: str -> {str:any} {
    CNPG_RESOURCES_MAP[size] if size in CNPG_RESOURCES_MAP else CNPG_RESOURCES_MAP["small"]
}

# =============================================================================
# Database Engine Mappings
# =============================================================================

# GCP CloudSQL database version format
get_gcp_database_version = lambda engine: str, version: str -> str {
    "POSTGRES_${version}" if engine == "postgres" else "MYSQL_${version}"
}

# =============================================================================
# Network CIDR Defaults
# =============================================================================

# Default CIDR blocks for network creation
DEFAULT_VPC_CIDR = "10.0.0.0/16"

# Default subnet configurations
DEFAULT_SUBNETS_GCP = [
    {name = "public", cidrBlock = "10.0.1.0/24", type = "public"}
    {name = "private", cidrBlock = "10.0.2.0/24", type = "private"}
]

DEFAULT_SUBNETS_AWS = [
    {name = "public-a", cidrBlock = "10.0.1.0/24", type = "public", availabilityZone = "a"}
    {name = "public-b", cidrBlock = "10.0.2.0/24", type = "public", availabilityZone = "b"}
    {name = "private-a", cidrBlock = "10.0.10.0/24", type = "private", availabilityZone = "a"}
    {name = "private-b", cidrBlock = "10.0.11.0/24", type = "private", availabilityZone = "b"}
]

get_default_subnets = lambda provider: str -> [any] {
    DEFAULT_SUBNETS_AWS if provider == "aws" else DEFAULT_SUBNETS_GCP
}
