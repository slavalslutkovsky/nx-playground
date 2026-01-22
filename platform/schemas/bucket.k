"""
Bucket schema - Provider-agnostic storage bucket specification.
Source of truth for XBucket XRD and compose-resources function.
"""
import .common as c

type StorageClass = "standard" | "nearline" | "coldline"

schema BucketSpec(c.BaseSpec):
    """Provider-agnostic storage bucket specification."""
    provider: c.CloudProvider
    name: str
    region: str
    storageClass: StorageClass = "standard"
    versioning: bool = False
    encryption: bool = True
    publicAccess: bool = False

    check:
        len(name) > 0, "name is required"
        len(region) > 0, "region is required"

schema BucketStatus(c.BaseStatus):
    """Status fields for bucket resources."""
    bucketName?: str
    url?: str
    id?: str
    cloudUrl?: str

# OpenAPI v3 schema for XRD generation
SPEC_PROPERTIES = {
    provider = c.PROVIDER_CLOUD_SCHEMA
    region = {
        type = "string"
        description = "Cloud region for the bucket"
    }
    name = {
        type = "string"
        description = "Name of the bucket (must be globally unique)"
    }
    environment = c.ENVIRONMENT_SCHEMA
    storageClass = {
        type = "string"
        description = "Storage class"
        enum = ["standard", "nearline", "coldline"]
        default = "standard"
    }
    versioning = c._boolean_schema("Enable versioning")
    encryption = c._boolean_schema("Enable encryption", True)
    publicAccess = c._boolean_schema("Allow public access")
    deletionPolicy = c.DELETION_POLICY_SCHEMA
}

SPEC_REQUIRED = ["provider", "region", "name"]

STATUS_PROPERTIES = {
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
XRD_NAME = "xbuckets.storage.platform.io"
XRD_GROUP = "storage.platform.io"
XRD_KIND = "XBucket"
XRD_PLURAL = "xbuckets"
XRD_CLAIM_KIND = "Bucket"
XRD_CLAIM_PLURAL = "buckets"
XRD_COMPOSITION = "xbucket"
