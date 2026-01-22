"""
Database module composition - KCL for Crossplane function-kcl pipeline.
This file runs inside function-kcl and switches on provider.
"""

import .schema
import .providers.aws as aws
import .providers.gcp as gcp

# Get the observed composite resource (OXR) from Crossplane
oxr = option("params").oxr
ocds = option("params").ocds or {}

# Extract spec from the XR
_spec = schema.DatabaseSpec {
    provider = oxr.spec.provider
    environment = oxr.spec.get("environment", "dev")
    region = oxr.spec.region
    deletionPolicy = oxr.spec.get("deletionPolicy", "Delete")
    tags = oxr.spec.get("tags", {})
    name = oxr.spec.name
    engine = oxr.spec.get("engine", "postgres")
    engineVersion = oxr.spec.get("engineVersion", "")
    size = oxr.spec.get("size", "small")
    storageGB = oxr.spec.get("storageGB", 20)
    storageType = oxr.spec.get("storageType", "gp3")
    multiAZ = oxr.spec.get("multiAZ", False)
    publiclyAccessible = oxr.spec.get("publiclyAccessible", False)
    backup = schema.BackupConfig {
        enabled = oxr.spec.get("backup", {}).get("enabled", True)
        retentionDays = oxr.spec.get("backup", {}).get("retentionDays", 7)
        preferredWindow = oxr.spec.get("backup", {}).get("preferredWindow", "03:00-04:00")
    }
    secrets = schema.SecretsConfig {
        generatePassword = oxr.spec.get("secrets", {}).get("generatePassword", True)
        passwordSecretRef = oxr.spec.get("secrets", {}).get("passwordSecretRef", "")
    }
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
        engine: _spec.engine
        instanceName: _spec.name
        size: _spec.size
    }
}

# Output items for Crossplane
items = _resources + [dxr]
