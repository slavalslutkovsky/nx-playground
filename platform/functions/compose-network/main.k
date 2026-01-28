"""
Multi-provider network composition.
Creates AWS VPC or GCP Network with subnets based on provider field.
Uses modular provider implementations for clean separation.
"""

import schemas.network as schema
import schemas.regions as regions
import .providers.aws as aws
import .providers.gcp as gcp

# Crossplane function inputs
oxr = option("params").oxr
_ocds = option("params").ocds or {}
dxr = option("params").dxr
dcds = option("params").dcds

# Type-annotated spec extraction
_raw_spec = oxr.spec

# Parse subnets if provided
_subnets = [
    schema.SubnetSpec {
        name = s.name
        cidrBlock = s.cidrBlock
        type = s.type or "private"
        availabilityZone = s.availabilityZone if s.availabilityZone else None
    }
    for s in _raw_spec.subnets
] if _raw_spec.subnets else None

_spec: schema.NetworkSpec = schema.NetworkSpec {
    provider = _raw_spec.provider
    name = _raw_spec.name
    region = _raw_spec.region or "europe-west1"
    cidrBlock = _raw_spec.cidrBlock or "10.0.0.0/16"
    subnets = _subnets
    enableDnsSupport = _raw_spec.enableDnsSupport if _raw_spec.enableDnsSupport != None else True
    enableDnsHostnames = _raw_spec.enableDnsHostnames if _raw_spec.enableDnsHostnames != None else True
    environment = _raw_spec.environment or "dev"
    deletionPolicy = _raw_spec.deletionPolicy or "Delete"
}

# Generate provider-specific resources
_resources = aws.generate(_spec, _ocds) if _spec.provider == "aws" else gcp.generate(_spec, _ocds)

# Get provider-specific status
_status = aws.get_status(_spec, _ocds) if _spec.provider == "aws" else gcp.get_status(_spec, _ocds)

# Update composite resource status
_dxr = {
    **oxr
    status = {
        provider = _status.provider
        region = _status.region
        networkId = _status.networkId
        cidrBlock = _status.cidrBlock
    }
}

# Output items
items = _resources + [_dxr]
