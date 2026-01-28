"""
Multi-provider storage composition.
Creates AWS S3 or GCP GCS bucket based on provider field.
Uses modular provider implementations for clean separation.
"""

import schemas.bucket as schema
import .providers.aws as aws
import .providers.gcp as gcp

# Crossplane function inputs
oxr = option("params").oxr  # observed composite resource
_ocds = option("params").ocds or {}  # observed composed resources
dxr = option("params").dxr  # desired composite resource
dcds = option("params").dcds  # desired composed resources

# Type-annotated spec extraction (IDE autocomplete enabled)
_raw_spec = oxr.spec
_spec: schema.BucketSpec = schema.BucketSpec {
    provider = _raw_spec.provider
    name = _raw_spec.name
    region = _raw_spec.region
    environment = _raw_spec.environment or "dev"
    storageClass = _raw_spec.storageClass or "standard"
    versioning = _raw_spec.versioning or False
    encryption = _raw_spec.encryption if _raw_spec.encryption != None else True
    publicAccess = _raw_spec.publicAccess or False
    deletionPolicy = _raw_spec.deletionPolicy or "Delete"
    tags = _raw_spec.tags or {}
    cors = [schema.CorsRule {
        allowedOrigins = r.allowedOrigins
        allowedMethods = r.allowedMethods or ["GET"]
        allowedHeaders = r.allowedHeaders or ["*"]
        maxAgeSeconds = r.maxAgeSeconds or 3600
    } for r in _raw_spec.cors] if _raw_spec.cors else []
    lifecycle = [schema.LifecycleRule {
        name = r.name
        prefix = r.prefix or ""
        enabled = r.enabled if r.enabled != None else True
        expirationDays = r.expirationDays or 0
        transitionDays = r.transitionDays or 0
        transitionStorageClass = r.transitionStorageClass or "nearline"
    } for r in _raw_spec.lifecycle] if _raw_spec.lifecycle else []
}

# Generate provider-specific resources
_resources = aws.generate(_spec, _ocds) if _spec.provider == "aws" else gcp.generate(_spec, _ocds)

# Get provider-specific status
_status = aws.get_status(_spec, _ocds) if _spec.provider == "aws" else gcp.get_status(_spec, _ocds)

# Update composite resource status
# Using dict for hyphenated keys (cloud-url) that match jsonPath in additionalPrinterColumns
_dxr = {
    **oxr
    status = {
        provider = _status.provider
        region = _status.region
        bucketName = _status.bucketName
        url = _status.url
        id = _status.id
        "cloud-url" = _status.cloudUrl
    }
}

# Output items
items = _resources + [_dxr]
