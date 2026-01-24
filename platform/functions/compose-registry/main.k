"""
Multi-provider container registry composition.
Creates AWS ECR, GCP Artifact Registry, or Harbor based on provider field.
Uses modular provider implementations for clean separation.
"""

import schemas.registry as schema
import .providers.aws as aws
import .providers.gcp as gcp
import .providers.kubernetes as k8s

# Crossplane function inputs
oxr = option("params").oxr
_ocds = option("params").ocds or {}
dxr = option("params").dxr
dcds = option("params").dcds

# Type-annotated spec extraction
_raw_spec = oxr.spec

# Handle optional boolean fields with defaults
_scan_on_push_raw = _raw_spec.scanOnPush
_encryption_raw = _raw_spec.encryption

_spec: schema.RegistrySpec = schema.RegistrySpec {
    provider = _raw_spec.provider
    name = _raw_spec.name
    region = _raw_spec.region or ""
    format = _raw_spec.format or "docker"
    immutableTags = _raw_spec.immutableTags or False
    scanOnPush = _scan_on_push_raw if _scan_on_push_raw != None else True
    encryption = _encryption_raw if _encryption_raw != None else True
    publicAccess = _raw_spec.publicAccess or False
    retentionDays = _raw_spec.retentionDays or 0
    namespace = _raw_spec.namespace or "default"
    environment = _raw_spec.environment or "dev"
    deletionPolicy = _raw_spec.deletionPolicy or "Delete"
}

# Generate provider-specific resources
_resources = if _spec.provider == "aws":
    aws.generate(_spec, _ocds)
elif _spec.provider == "gcp":
    gcp.generate(_spec, _ocds)
else:
    k8s.generate(_spec, _ocds)

# Get provider-specific status
_status = if _spec.provider == "aws":
    aws.get_status(_spec, _ocds)
elif _spec.provider == "gcp":
    gcp.get_status(_spec, _ocds)
else:
    k8s.get_status(_spec, _ocds)

# Update composite resource status
_dxr = {
    **oxr
    status = {
        provider = _status.provider
        region = _status.region
        registryUrl = _status.registryUrl
        repositoryName = _status.repositoryName
    }
}

# Output items
items = _resources + [_dxr]
