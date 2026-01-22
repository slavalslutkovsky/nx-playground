"""
Storage module schema - provider-agnostic bucket specification.
"""

import common.types

type StorageClass = "standard" | "nearline" | "coldline" | "archive"
type AccessControl = "private" | "public-read"

schema BucketSpec(types.BaseSpec):
    """Provider-agnostic bucket specification."""
    name: str
    storageClass: StorageClass = "standard"
    versioning: bool = False
    encryption: bool = True
    publicAccess: bool = False
    cors: [CorsRule] = []
    lifecycle: [LifecycleRule] = []

    check:
        len(name) >= 3, "bucket name must be at least 3 characters"
        len(name) <= 63, "bucket name must be 63 characters or less"
        not publicAccess or encryption, "public buckets must have encryption enabled"

schema CorsRule:
    """CORS configuration for the bucket."""
    allowedOrigins: [str]
    allowedMethods: [str] = ["GET"]
    allowedHeaders: [str] = ["*"]
    maxAgeSeconds: int = 3600

    check:
        len(allowedOrigins) > 0, "at least one allowed origin must be specified"

schema LifecycleRule:
    """Lifecycle rule for automatic object management."""
    name: str
    prefix: str = ""
    enabled: bool = True
    expirationDays: int = 0
    transitionDays: int = 0
    transitionStorageClass: StorageClass = "nearline"

    check:
        expirationDays >= 0, "expirationDays must be non-negative"
        transitionDays >= 0, "transitionDays must be non-negative"
        expirationDays > 0 or transitionDays > 0, "either expirationDays or transitionDays must be specified"

# Storage class mappings between providers
STORAGE_CLASS_MAP = {
    "aws": {
        "standard": "STANDARD"
        "nearline": "STANDARD_IA"
        "coldline": "GLACIER"
        "archive": "DEEP_ARCHIVE"
    }
    "gcp": {
        "standard": "STANDARD"
        "nearline": "NEARLINE"
        "coldline": "COLDLINE"
        "archive": "ARCHIVE"
    }
}

map_storage_class = lambda provider: types.CloudProvider, storageClass: StorageClass -> str {
    """Map storage class to provider-specific value."""
    STORAGE_CLASS_MAP[provider][storageClass] if storageClass in STORAGE_CLASS_MAP[provider] else STORAGE_CLASS_MAP[provider]["standard"]
}
