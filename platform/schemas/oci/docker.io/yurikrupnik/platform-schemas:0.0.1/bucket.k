"""
Bucket schema - Provider-agnostic storage bucket specification.
Source of truth for XBucket XRD and compose-resources function.
"""
import .common as c

type StorageClass = "standard" | "nearline" | "coldline" | "archive"

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

schema BucketSpec(c.BaseSpec):
    """Provider-agnostic storage bucket specification."""
    provider: c.CloudProvider
    name: str
    region: str
    storageClass: StorageClass = "standard"
    versioning: bool = False
    encryption: bool = True
    publicAccess: bool = False
    cors: [CorsRule] = []
    lifecycle: [LifecycleRule] = []

    check:
        len(name) >= 3, "bucket name must be at least 3 characters"
        len(name) <= 63, "bucket name must be 63 characters or less"
        len(region) > 0, "region is required"
        not publicAccess or encryption, "public buckets must have encryption enabled"

schema BucketStatus(c.BaseStatus):
    """Status fields for bucket resources."""
    bucketName?: str
    url?: str
    id?: str
    cloudUrl?: str

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

map_storage_class = lambda provider: c.CloudProvider, storageClass: StorageClass -> str {
    """Map storage class to provider-specific value."""
    STORAGE_CLASS_MAP[provider][storageClass] if storageClass in STORAGE_CLASS_MAP[provider] else STORAGE_CLASS_MAP[provider]["standard"]
}

# OpenAPI v3 schema for XRD generation
BUCKET_SPEC_PROPERTIES = {
    provider = c.PROVIDER_CLOUD_SCHEMA
    region = {
        type = "string"
        description = "Cloud region for the bucket"
    }
    name = {
        type = "string"
        description = "Name of the bucket (must be globally unique, 3-63 characters)"
        minLength = 3
        maxLength = 63
    }
    environment = c.ENVIRONMENT_SCHEMA
    storageClass = {
        type = "string"
        description = "Storage class"
        enum = ["standard", "nearline", "coldline", "archive"]
        default = "standard"
    }
    versioning = c._boolean_schema("Enable versioning")
    encryption = c._boolean_schema("Enable encryption", True)
    publicAccess = c._boolean_schema("Allow public access (requires encryption)")
    deletionPolicy = c.DELETION_POLICY_SCHEMA
    tags = c.TAGS_SCHEMA
    cors = {
        type = "array"
        description = "CORS rules for the bucket"
        items = {
            type = "object"
            properties = {
                allowedOrigins = {
                    type = "array"
                    items = {type = "string"}
                    description = "Origins allowed to access the bucket"
                }
                allowedMethods = {
                    type = "array"
                    items = {type = "string", enum = ["GET", "PUT", "POST", "DELETE", "HEAD"]}
                    default = ["GET"]
                    description = "HTTP methods allowed"
                }
                allowedHeaders = {
                    type = "array"
                    items = {type = "string"}
                    default = ["*"]
                    description = "Headers allowed in requests"
                }
                maxAgeSeconds = {
                    type = "integer"
                    default = 3600
                    description = "Time in seconds for browser caching"
                }
            }
            required = ["allowedOrigins"]
        }
    }
    lifecycle = {
        type = "array"
        description = "Lifecycle rules for automatic object management"
        items = {
            type = "object"
            properties = {
                name = {
                    type = "string"
                    description = "Name of the lifecycle rule"
                }
                prefix = {
                    type = "string"
                    default = ""
                    description = "Object prefix to match"
                }
                enabled = {
                    type = "boolean"
                    default = True
                    description = "Whether the rule is enabled"
                }
                expirationDays = {
                    type = "integer"
                    default = 0
                    description = "Days after which objects are deleted (0 = disabled)"
                }
                transitionDays = {
                    type = "integer"
                    default = 0
                    description = "Days after which objects transition to another storage class"
                }
                transitionStorageClass = {
                    type = "string"
                    enum = ["standard", "nearline", "coldline", "archive"]
                    default = "nearline"
                    description = "Target storage class for transition"
                }
            }
            required = ["name"]
        }
    }
}

BUCKET_SPEC_REQUIRED = ["provider", "region", "name"]

BUCKET_STATUS_PROPERTIES = {
    provider = {type = "string"}
    region = {type = "string"}
    bucketName = {type = "string"}
    url = {type = "string"}
    id = {type = "string"}
    "cloud-url" = {type = "string"}
}

# Additional printer columns for kubectl get output
ADDITIONAL_PRINTER_COLUMNS = [
    {
        name = "url"
        $type = "string"
        jsonPath = ".status.url"
    }
    {
        name = "id"
        $type = "string"
        jsonPath = ".status.id"
    }
    {
        name = "cloud-url"
        $type = "string"
        jsonPath = '.status.cloud-url'
    }
]

# XRD metadata
BUCKET_XRD_NAME = "xbuckets.storage.platform.io"
BUCKET_XRD_GROUP = "storage.platform.io"
BUCKET_XRD_KIND = "XBucket"
BUCKET_XRD_PLURAL = "xbuckets"
BUCKET_XRD_CLAIM_KIND = "Bucket"
BUCKET_XRD_CLAIM_PLURAL = "buckets"
BUCKET_XRD_COMPOSITION = "xbucket"
