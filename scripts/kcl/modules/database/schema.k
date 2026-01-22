"""
Database module schema - provider-agnostic database specification.
"""

import common.types

type DatabaseEngine = "postgres" | "mysql"
type InstanceSize = "small" | "medium" | "large" | "xlarge"

schema DatabaseSpec(types.BaseSpec):
    """Provider-agnostic database specification."""
    name: str
    engine: DatabaseEngine = "postgres"
    engineVersion: str = ""
    size: InstanceSize = "small"
    storageGB: int = 20
    storageType: str = "gp3"
    multiAZ: bool = False
    publiclyAccessible: bool = False
    backup: BackupConfig = BackupConfig {}
    secrets: SecretsConfig = SecretsConfig {}

    check:
        len(name) >= 1, "database name must be specified"
        len(name) <= 63, "database name must be 63 characters or less"
        storageGB >= 20, "storage must be at least 20GB"
        storageGB <= 65536, "storage must be 65536GB or less"

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

# Engine version defaults
ENGINE_VERSION_DEFAULTS = {
    postgres: {
        aws: "15.4"
        gcp: "POSTGRES_18"
    }
    mysql: {
        aws: "8.0.35"
        gcp: "MYSQL_8_0"
    }
}

get_engine_version = lambda engine: DatabaseEngine, provider: types.CloudProvider, version: str -> str {
    """Get engine version, using defaults if not specified."""
    if version:
        version
    else:
        ENGINE_VERSION_DEFAULTS[engine].get(provider, ENGINE_VERSION_DEFAULTS[engine]["aws"])
}

# Instance size mappings
INSTANCE_SIZE_MAP = {
    aws: {
        small: "db.t3.micro"
        medium: "db.t3.small"
        large: "db.t3.medium"
        xlarge: "db.t3.large"
    }
    gcp: {
        small: "db-f1-micro"
        medium: "db-g1-small"
        large: "db-custom-2-4096"
        xlarge: "db-custom-4-8192"
    }
}

get_instance_class = lambda provider: types.CloudProvider, size: InstanceSize -> str {
    """Get provider-specific instance class."""
    INSTANCE_SIZE_MAP[provider].get(size, INSTANCE_SIZE_MAP[provider]["small"])
}

# Storage type mappings
STORAGE_TYPE_MAP = {
    aws: {
        standard: "gp2"
        ssd: "gp3"
        iops: "io1"
    }
}

get_storage_type = lambda provider: types.CloudProvider, storageType: str -> str {
    """Get provider-specific storage type."""
    if provider == "aws":
        STORAGE_TYPE_MAP[provider].get(storageType, "gp3")
    else:
        "SSD"  # GCP always uses SSD
}
