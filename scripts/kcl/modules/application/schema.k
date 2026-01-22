"""
Application module schema - parent XRD that composes storage and database modules.
"""

import common.types

schema ApplicationSpec(types.BaseSpec):
    """Application specification that composes storage and database."""
    name: str
    storage: StorageConfig = None
    database: DatabaseConfig = None

    check:
        len(name) >= 1, "application name must be specified"
        len(name) <= 63, "application name must be 63 characters or less"
        storage or database, "at least one of storage or database must be configured"

schema StorageConfig:
    """Storage configuration for the application."""
    enabled: bool = True
    storageClass: str = "standard"
    versioning: bool = False
    encryption: bool = True

schema DatabaseConfig:
    """Database configuration for the application."""
    enabled: bool = True
    engine: str = "postgres"
    size: str = "small"
    storageGB: int = 20
    multiAZ: bool = False
    backup: BackupConfig = BackupConfig {}

    check:
        engine in ["postgres", "mysql"], "engine must be postgres or mysql"
        size in ["small", "medium", "large", "xlarge"], "size must be small, medium, large, or xlarge"
        storageGB >= 20, "storageGB must be at least 20"

schema BackupConfig:
    """Backup configuration for the database."""
    enabled: bool = True
    retentionDays: int = 7
