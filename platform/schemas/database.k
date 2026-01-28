"""
Database schema - Provider-agnostic managed database specification.
Source of truth for XDatabase XRD and compose-database function.
Supports AWS RDS, GCP CloudSQL, and CloudNative-PG (kubernetes).
"""
import .common as c

type DatabaseEngine = "postgres" | "mysql"
type DatabaseSize = "small" | "medium" | "large" | "xlarge"

schema BackupConfig:
    """Backup configuration for the database."""
    enabled: bool = True
    retentionDays: int = 7
    preferredWindow: str = "03:00-04:00"

    check:
        retentionDays >= 0, "retentionDays must be non-negative"
        retentionDays <= 35, "retentionDays must be 35 or less"

schema SecretsConfig:
    """Secrets configuration for database credentials."""
    generatePassword: bool = True
    passwordSecretRef: str = ""
    usernameSecretRef: str = ""

schema DatabaseSpec(c.BaseSpec):
    """Provider-agnostic database specification."""
    provider: c.CloudProviderWithK8s
    region?: str  # Required for aws/gcp, ignored for kubernetes
    engine: DatabaseEngine = "postgres"
    engineVersion: str = "18"
    size: DatabaseSize = "small"
    storageGB: int = 20
    storageType: str = "gp3"  # AWS: gp2, gp3, io1; GCP: uses SSD
    multiAZ: bool = False  # AWS: Multi-AZ; GCP: REGIONAL availability
    publiclyAccessible: bool = False
    instances: int = 1  # CNPG only
    namespace: str = "default"  # kubernetes only
    backup: BackupConfig = BackupConfig {}
    secrets: SecretsConfig = SecretsConfig {}

    check:
        provider != "kubernetes" or engine == "postgres", "kubernetes provider only supports postgres (CloudNative-PG)"
        provider == "kubernetes" or region, "region is required for aws/gcp providers"
        instances >= 1, "instances must be at least 1"
        storageGB >= 20, "storageGB must be at least 20"
        storageGB <= 65536, "storageGB must be 65536 or less"

schema DatabaseStatus(c.BaseStatus):
    """Status fields for database resources."""
    engine?: str
    endpoint?: str

# Engine version defaults per provider
ENGINE_VERSION_DEFAULTS = {
    "postgres": {
        "aws": "15.4"
        "gcp": "POSTGRES_18"
    }
    "mysql": {
        "aws": "8.0.35"
        "gcp": "MYSQL_8_0"
    }
}

get_engine_version = lambda engine: DatabaseEngine, provider: str, version: str -> str {
    """Get engine version, using defaults if not specified."""
    version if version and version != "" else \
    ENGINE_VERSION_DEFAULTS[engine][provider] if engine in ENGINE_VERSION_DEFAULTS and provider in ENGINE_VERSION_DEFAULTS[engine] else \
    ENGINE_VERSION_DEFAULTS[engine]["aws"] if engine in ENGINE_VERSION_DEFAULTS else "15.4"
}

# Instance size mappings - split by provider to avoid mixed types
AWS_INSTANCE_SIZE_MAP = {
    "small": "db.t3.micro"
    "medium": "db.t3.small"
    "large": "db.t3.medium"
    "xlarge": "db.t3.large"
}

GCP_INSTANCE_SIZE_MAP = {
    "small": "db-f1-micro"
    "medium": "db-g1-small"
    "large": "db-custom-2-4096"
    "xlarge": "db-custom-4-8192"
}

K8S_INSTANCE_SIZE_MAP = {
    "small": {"cpu": "100m", "memory": "256Mi"}
    "medium": {"cpu": "500m", "memory": "512Mi"}
    "large": {"cpu": "1000m", "memory": "1Gi"}
    "xlarge": {"cpu": "2000m", "memory": "2Gi"}
}

get_instance_class = lambda provider: str, size: DatabaseSize -> any {
    """Get provider-specific instance class."""
    AWS_INSTANCE_SIZE_MAP[size] if provider == "aws" and size in AWS_INSTANCE_SIZE_MAP else \
    GCP_INSTANCE_SIZE_MAP[size] if provider == "gcp" and size in GCP_INSTANCE_SIZE_MAP else \
    K8S_INSTANCE_SIZE_MAP[size] if provider == "kubernetes" and size in K8S_INSTANCE_SIZE_MAP else \
    "db.t3.micro"
}

# Storage type mappings
STORAGE_TYPE_MAP = {
    "aws": {
        "standard": "gp2"
        "ssd": "gp3"
        "iops": "io1"
        "gp2": "gp2"
        "gp3": "gp3"
        "io1": "io1"
    }
}

get_storage_type = lambda provider: str, storageType: str -> str {
    """Get provider-specific storage type."""
    STORAGE_TYPE_MAP["aws"][storageType] if provider == "aws" and storageType in STORAGE_TYPE_MAP["aws"] else \
    "gp3" if provider == "aws" else "SSD"
}

# OpenAPI v3 schema for XRD generation
DATABASE_SPEC_PROPERTIES = {
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
        default = "18"
    }
    size = {
        type = "string"
        description = "Instance size"
        enum = ["small", "medium", "large", "xlarge"]
        default = "small"
    }
    storageGB = c._integer_schema("Storage size in GB (20-65536)", 20)
    storageType = {
        type = "string"
        description = "Storage type (AWS: gp2, gp3, io1; GCP: SSD)"
        default = "gp3"
    }
    multiAZ = c._boolean_schema("Enable Multi-AZ (AWS) or Regional availability (GCP)")
    publiclyAccessible = c._boolean_schema("Allow public access (not recommended)")
    instances = c._integer_schema("Number of instances (kubernetes/CNPG only)", 1)
    namespace = c.NAMESPACE_SCHEMA
    environment = c.ENVIRONMENT_SCHEMA
    deletionPolicy = c.DELETION_POLICY_SCHEMA
    tags = c.TAGS_SCHEMA
    backup = {
        type = "object"
        description = "Backup configuration"
        properties = {
            enabled = {
                type = "boolean"
                default = True
                description = "Enable automated backups"
            }
            retentionDays = {
                type = "integer"
                default = 7
                minimum = 0
                maximum = 35
                description = "Number of days to retain backups"
            }
            preferredWindow = {
                type = "string"
                default = "03:00-04:00"
                description = "Preferred backup window (UTC)"
            }
        }
    }
    secrets = {
        type = "object"
        description = "Secrets configuration for credentials"
        properties = {
            generatePassword = {
                type = "boolean"
                default = True
                description = "Auto-generate database password"
            }
            passwordSecretRef = {
                type = "string"
                description = "Reference to existing password secret"
            }
            usernameSecretRef = {
                type = "string"
                description = "Reference to existing username secret"
            }
        }
    }
}

DATABASE_SPEC_REQUIRED = ["provider", "engine"]

DATABASE_STATUS_PROPERTIES = {
    provider = {type = "string"}
    region = {type = "string"}
    engine = {type = "string"}
    endpoint = {type = "string"}
}

# XRD metadata
DATABASE_XRD_NAME = "xdatabases.database.platform.io"
DATABASE_XRD_GROUP = "database.platform.io"
DATABASE_XRD_KIND = "XDatabase"
DATABASE_XRD_PLURAL = "xdatabases"
DATABASE_XRD_CLAIM_KIND = "Database"
DATABASE_XRD_CLAIM_PLURAL = "databases"
DATABASE_XRD_COMPOSITION = "xdatabase"
