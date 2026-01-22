"""
Storage module composition - KCL for Crossplane function-kcl pipeline.
This file runs inside function-kcl and switches on provider.
"""

import .schema
import .providers.aws as aws
import .providers.gcp as gcp

# Get the observed composite resource (OXR) from Crossplane
oxr = option("params").oxr
ocds = option("params").ocds or {}

# Extract spec from the XR
_spec = schema.BucketSpec {
    provider = oxr.spec.provider
    environment = oxr.spec.get("environment", "dev")
    region = oxr.spec.region
    deletionPolicy = oxr.spec.get("deletionPolicy", "Delete")
    tags = oxr.spec.get("tags", {})
    name = oxr.spec.name
    storageClass = oxr.spec.get("storageClass", "standard")
    versioning = oxr.spec.get("versioning", False)
    encryption = oxr.spec.get("encryption", True)
    publicAccess = oxr.spec.get("publicAccess", False)
    cors = oxr.spec.get("cors", [])
    lifecycle = oxr.spec.get("lifecycle", [])
}

# Switch on provider to generate resources
_generate = lambda provider: str -> [any] {
    if provider == "aws":
        aws.generate(_spec)
    elif provider == "gcp":
        gcp.generate(_spec)
    else:
        []
}

# Generate resources based on provider
_resources = _generate(_spec.provider)

# Set desired composite resource state
dxr = {
    **oxr
    status: {
        provider: _spec.provider
        region: _spec.region
        bucketName: _spec.name
    }
}

# Output items for Crossplane
items = _resources + [dxr]
